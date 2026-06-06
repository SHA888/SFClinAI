//! Configuration for LLM-based refinement proposer.
//!
//! Implements DEF-PS-14 (Refinement proposer signature) by specifying the configuration
//! for an LLM proposer that can generate candidate hypotheses.
//!
//! Supports multiple LLM providers (OpenAI, Anthropic, custom endpoints) and offline
//! mock mode for deterministic testing (Phase 10.3).
//!
//! Reference: SPEC.md §2.7, NOTE.md §4A.5, ARCHITECTURE.md Diagram 5 (LLM proposer slot).

use serde::{Deserialize, Serialize};

/// LLM provider endpoint specification.
///
/// Specifies which LLM API to call. Supports major cloud providers (OpenAI, Anthropic),
/// custom endpoints (e.g., local LLMs like Ollama), and mock mode for testing.
///
/// In production, API credentials are supplied via environment variables or other
/// secure mechanisms, not embedded in the config.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LlmProvider {
    /// OpenAI API (gpt-4, gpt-3.5-turbo, etc.)
    /// Default endpoint: <https://api.openai.com/v1/chat/completions>
    OpenAI,

    /// Anthropic API (claude-opus, claude-sonnet, etc.)
    /// Default endpoint: <https://api.anthropic.com/v1/messages>
    Anthropic,

    /// Custom or self-hosted LLM endpoint (e.g., Ollama, vLLM, local model server)
    /// Caller must specify the full endpoint URL.
    Custom { endpoint: String },

    /// Mock LLM for offline testing and CI.
    /// Returns predetermined responses from `mock_responses` in `LlmProposerConfig`.
    /// Deterministic (uses seed for reproducibility).
    Mock,
}

impl LlmProvider {
    /// Returns the default endpoint URL for well-known providers.
    /// Returns `None` for Mock (no network endpoint) and Custom (caller-specified).
    pub fn default_endpoint(&self) -> Option<&str> {
        match self {
            LlmProvider::OpenAI => Some("https://api.openai.com/v1/chat/completions"),
            LlmProvider::Anthropic => Some("https://api.anthropic.com/v1/messages"),
            LlmProvider::Custom { endpoint } => Some(endpoint.as_str()),
            LlmProvider::Mock => None,
        }
    }

    /// Returns true if this is the mock provider.
    pub fn is_mock(&self) -> bool {
        matches!(self, LlmProvider::Mock)
    }

    /// Returns the provider name as a string (for logging, versioning).
    pub fn name(&self) -> &str {
        match self {
            LlmProvider::OpenAI => "openai",
            LlmProvider::Anthropic => "anthropic",
            LlmProvider::Custom { .. } => "custom",
            LlmProvider::Mock => "mock",
        }
    }
}

/// Configuration for LLM-based refinement proposer.
///
/// Encapsulates all parameters needed for an LLM to generate candidate hypotheses:
/// - **provider**: Where to call (OpenAI, Anthropic, local, or mock)
/// - **model**: Model identifier (e.g., "gpt-4", "claude-opus-4-8")
/// - **prompt_template**: Template string for constructing the prompt
/// - **max_tokens**: Maximum tokens in the LLM response
/// - **temperature**: Sampling temperature (0.0 = deterministic, 1.0+ = random)
/// - **seed**: Optional seed for reproducible sampling (if provider supports it)
/// - **version**: Version string for provenance and audit trails
/// - **mock_responses**: Predetermined responses for mock mode (testing)
///
/// **Soundness properties**:
/// - Satisfies DEF-PS-14 (Refinement proposer signature) when wrapped in `LlmProposer`
/// - Used by `LlmProposer` to construct prompts and call LLM APIs
/// - Output is validated by `ProposerConstraint` (DEF-PS-15) after LLM response is parsed
/// - INV-PS-06 (Proposer cannot bypass soundness) is enforced by the substrate,
///   not by the proposer or its config
///
/// **Version semantics (INV-PS-05)**:
/// The `version` field tracks when the config (model, prompt, params) changed.
/// When used by `LlmProposer`, this version is stored in the Evidence provenance,
/// ensuring that hypothesis refinements can be traced back to a specific LLM config.
///
/// **Mock mode for testing**:
/// When `provider == Mock`, the `LlmProposer` returns predetermined responses
/// from `mock_responses` instead of calling an LLM API. This enables:
/// - Deterministic tests (with `seed`)
/// - Simulation of LLM hallucinations (invalid candidates for safety testing)
/// - CI/CD testing without API keys or network access
/// - Property-testing the safety invariant (INV-PS-06) by exercising the
///   proposer constraint filter on known invalid candidates
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LlmProposerConfig {
    /// LLM provider endpoint.
    provider: LlmProvider,

    /// Model identifier (e.g., "gpt-4-turbo", "claude-opus-4-8").
    /// Must be a valid model name for the chosen provider.
    model: String,

    /// Prompt template for constructing the LLM prompt.
    /// Should contain placeholders like {hypothesis} and {evidence} that will be
    /// filled in by the proposer when calling the LLM.
    ///
    /// Example:
    /// ```text
    /// You are a clinical diagnostic assistant. Given the current hypothesis and evidence,
    /// suggest plausible next diagnostic refinements.
    ///
    /// Current hypothesis: {hypothesis}
    /// Evidence: {evidence}
    ///
    /// Suggest refinements (comma-separated SNOMED codes):
    /// ```
    prompt_template: String,

    /// Maximum tokens in the LLM response.
    /// Typical range: 100–500 (for diagnostic refinement suggestions).
    max_tokens: usize,

    /// Sampling temperature (0.0 = deterministic, 1.0+ = random).
    /// - 0.0: Always pick the highest-probability token (deterministic)
    /// - 0.7: Default; balanced randomness
    /// - 1.0+: High randomness; useful for generating diverse hallucinations in testing
    temperature: f32,

    /// Optional seed for reproducible sampling (if provider supports it).
    /// When set, the same input should produce the same output across runs
    /// (provider permitting).
    seed: Option<u64>,

    /// Version identifier for this config (e.g., "0.1.0", "0.2.0").
    /// Incremented when model, prompt, or parameters change materially.
    /// Used in Evidence provenance (INV-PS-05) to track which version of the
    /// LLM made which refinement suggestion.
    version: String,

    /// Predetermined responses for mock mode (testing).
    /// When `provider == Mock`, `LlmProposer` will return these responses
    /// in sequence instead of calling an LLM API.
    ///
    /// Each response should be a string representation of candidate hypotheses
    /// (e.g., "SNOMED:12345,SNOMED:67890" or raw hypothesis JSON).
    /// The proposer will parse and filter these through the constraint validator.
    ///
    /// For safety testing (task 10.3), include responses that are:
    /// - Valid refinements (pass `ProposerConstraint`)
    /// - Invalid/hallucinated (fail constraint, filtered silently)
    /// - Edge cases (non-existent SOFA bands, etc.)
    mock_responses: Option<Vec<String>>,
}

impl LlmProposerConfig {
    /// Create a new LLM proposer config with required parameters.
    /// Does NOT validate; call validate() after construction to check constraints.
    pub fn new(
        provider: LlmProvider,
        model: impl Into<String>,
        prompt_template: impl Into<String>,
        max_tokens: usize,
        temperature: f32,
        version: impl Into<String>,
    ) -> Self {
        LlmProposerConfig {
            provider,
            model: model.into(),
            prompt_template: prompt_template.into(),
            max_tokens,
            temperature,
            seed: None,
            version: version.into(),
            mock_responses: None,
        }
    }

    /// Returns a reference to the provider.
    pub fn provider(&self) -> &LlmProvider {
        &self.provider
    }

    /// Returns a reference to the model identifier.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Returns a reference to the prompt template.
    pub fn prompt_template(&self) -> &str {
        &self.prompt_template
    }

    /// Returns the maximum tokens.
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// Returns the sampling temperature.
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Returns the random seed (if set).
    pub fn seed(&self) -> Option<u64> {
        self.seed
    }

    /// Returns a reference to the version identifier.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns a reference to the mock responses (if set).
    pub fn mock_responses(&self) -> Option<&Vec<String>> {
        self.mock_responses.as_ref()
    }

    /// Create a new OpenAI proposer config.
    pub fn openai(
        model: impl Into<String>,
        prompt_template: impl Into<String>,
        max_tokens: usize,
        temperature: f32,
        version: impl Into<String>,
    ) -> Self {
        Self::new(
            LlmProvider::OpenAI,
            model,
            prompt_template,
            max_tokens,
            temperature,
            version,
        )
    }

    /// Create a new Anthropic proposer config.
    pub fn anthropic(
        model: impl Into<String>,
        prompt_template: impl Into<String>,
        max_tokens: usize,
        temperature: f32,
        version: impl Into<String>,
    ) -> Self {
        Self::new(
            LlmProvider::Anthropic,
            model,
            prompt_template,
            max_tokens,
            temperature,
            version,
        )
    }

    /// Create a new custom endpoint proposer config.
    pub fn custom(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        prompt_template: impl Into<String>,
        max_tokens: usize,
        temperature: f32,
        version: impl Into<String>,
    ) -> Self {
        Self::new(
            LlmProvider::Custom {
                endpoint: endpoint.into(),
            },
            model,
            prompt_template,
            max_tokens,
            temperature,
            version,
        )
    }

    /// Create a new mock proposer config for offline testing.
    pub fn mock(
        model: impl Into<String>,
        responses: Vec<String>,
        version: impl Into<String>,
    ) -> Self {
        LlmProposerConfig {
            provider: LlmProvider::Mock,
            model: model.into(),
            prompt_template: "mock".to_string(), // Not used in mock mode
            max_tokens: 0,                       // Not used in mock mode
            temperature: 0.0,                    // Deterministic in mock mode
            seed: None,
            version: version.into(),
            mock_responses: Some(responses),
        }
    }

    /// Set the random seed for reproducible sampling.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Returns `true` if this is a mock config.
    pub fn is_mock(&self) -> bool {
        self.provider.is_mock()
    }

    /// Returns the endpoint URL for this config.
    /// Returns `None` for mock configs.
    pub fn endpoint(&self) -> Option<&str> {
        self.provider.default_endpoint()
    }

    /// Returns the number of mock responses available (if in mock mode).
    pub fn mock_response_count(&self) -> Option<usize> {
        self.mock_responses.as_ref().map(|r| r.len())
    }

    /// Validates that the config is self-consistent.
    /// Returns `Err(String)` if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        // Check model name is not empty
        if self.model.is_empty() {
            return Err("model name cannot be empty".to_string());
        }

        // Check prompt template is not empty and contains required placeholders (for non-mock)
        if !self.is_mock() {
            if self.prompt_template.is_empty() {
                return Err("prompt_template cannot be empty".to_string());
            }
            // Warn about missing placeholders (but don't fail; proposer can customize as needed)
            if !self.prompt_template.contains("{hypothesis}") {
                return Err(
                    "prompt_template should contain {hypothesis} placeholder for LlmProposer"
                        .to_string(),
                );
            }
            if !self.prompt_template.contains("{evidence}") {
                return Err(
                    "prompt_template should contain {evidence} placeholder for LlmProposer"
                        .to_string(),
                );
            }
        }

        // Check provider-specific temperature ranges
        match &self.provider {
            LlmProvider::OpenAI => {
                if self.temperature < 0.0 || self.temperature > 2.0 {
                    return Err(format!(
                        "OpenAI temperature {} out of range [0.0, 2.0]",
                        self.temperature
                    ));
                }
            }
            LlmProvider::Anthropic => {
                if self.temperature < 0.0 || self.temperature > 1.0 {
                    return Err(format!(
                        "Anthropic temperature {} out of range [0.0, 1.0]",
                        self.temperature
                    ));
                }
            }
            LlmProvider::Custom { .. } => {
                // Custom endpoint; allow [0.0, 2.0] as a reasonable range
                if self.temperature < 0.0 || self.temperature > 2.0 {
                    return Err(format!(
                        "temperature {} out of range [0.0, 2.0]",
                        self.temperature
                    ));
                }
            }
            LlmProvider::Mock => {
                // Mock mode is deterministic
            }
        }

        // Check max_tokens is positive (except mock mode)
        if self.max_tokens == 0 && !self.is_mock() {
            return Err("max_tokens must be positive (except in mock mode)".to_string());
        }

        // Check mock mode consistency
        if self.is_mock()
            && (self.mock_responses.is_none()
                || self.mock_responses.as_ref().is_none_or(|r| r.is_empty()))
        {
            return Err("mock mode requires at least one response in mock_responses".to_string());
        }

        // Check custom endpoint is not empty and has a valid URL scheme
        if let LlmProvider::Custom { endpoint } = &self.provider {
            if endpoint.is_empty() {
                return Err("custom endpoint cannot be empty".to_string());
            }
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                return Err(
                    "custom endpoint must start with http:// or https:// (got: {})"
                        .to_string()
                        .replace("{}", endpoint),
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_config_creation() {
        let config = LlmProposerConfig::openai("gpt-4", "You are helpful", 256, 0.7, "0.1.0");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_tokens, 256);
        assert!(!config.is_mock());
        assert_eq!(
            config.endpoint(),
            Some("https://api.openai.com/v1/chat/completions")
        );
    }

    #[test]
    fn test_mock_config_creation() {
        let responses = vec!["resp1".to_string(), "resp2".to_string()];
        let config = LlmProposerConfig::mock("mock-model", responses, "0.1.0");
        assert!(config.is_mock());
        assert_eq!(config.mock_response_count(), Some(2));
        assert_eq!(config.endpoint(), None);
    }

    #[test]
    fn test_custom_endpoint_config() {
        let config = LlmProposerConfig::custom(
            "http://localhost:8000/v1/completions",
            "local-model",
            "prompt",
            100,
            0.5,
            "0.1.0",
        );
        assert_eq!(
            config.endpoint(),
            Some("http://localhost:8000/v1/completions")
        );
    }

    #[test]
    fn test_config_validation() {
        // Valid config with required placeholders
        let config = LlmProposerConfig::openai(
            "gpt-4",
            "Given hypothesis: {hypothesis}, evidence: {evidence}",
            256,
            0.7,
            "0.1.0",
        );
        assert!(config.validate().is_ok());

        // Invalid: missing {hypothesis} placeholder
        let config = LlmProposerConfig::openai("gpt-4", "evidence: {evidence}", 256, 0.7, "0.1.0");
        assert!(config.validate().is_err());

        // Invalid: missing {evidence} placeholder
        let config =
            LlmProposerConfig::openai("gpt-4", "hypothesis: {hypothesis}", 256, 0.7, "0.1.0");
        assert!(config.validate().is_err());

        // Invalid: Anthropic temperature too high
        let config = LlmProposerConfig::anthropic(
            "claude-opus",
            "Given hypothesis: {hypothesis}, evidence: {evidence}",
            256,
            1.5,
            "0.1.0",
        );
        assert!(config.validate().is_err());

        // Invalid: mock mode without responses
        let config = LlmProposerConfig::mock("mock", vec![], "0.1.0");
        assert!(config.validate().is_err());

        // Valid: mock mode with responses
        let config = LlmProposerConfig::mock("mock", vec!["resp1".to_string()], "0.1.0");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_seed_builder() {
        let config = LlmProposerConfig::openai("gpt-4", "prompt", 256, 0.7, "0.1.0").with_seed(42);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_provider_names() {
        assert_eq!(LlmProvider::OpenAI.name(), "openai");
        assert_eq!(LlmProvider::Anthropic.name(), "anthropic");
        assert_eq!(
            LlmProvider::Custom {
                endpoint: "http://localhost:8000".to_string()
            }
            .name(),
            "custom"
        );
        assert_eq!(LlmProvider::Mock.name(), "mock");
    }
}
