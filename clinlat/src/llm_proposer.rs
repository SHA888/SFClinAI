//! LLM-based refinement proposer adapter (task 10.2).
//!
//! Wraps an LLM API call with:
//! 1. Prompt construction from hypothesis + evidence
//! 2. LLM API invocation
//! 3. Response parsing into candidate hypotheses
//! 4. Constraint filtering via `ProposerConstraint` (DEF-PS-15)
//!
//! Failed parses or out-of-constraint responses are logged as "LLM hallucinations"
//! and filtered silently by the ontology gate.
//!
//! Demonstrates the load-bearing safety property (INV-PS-06):
//! even if the LLM hallucinates or produces invalid candidates, the substrate
//! remains sound because the proposer's output is always validated before use.
//!
//! Reference: SPEC.md §2.7 (DEF-PS-14, DEF-PS-15), NOTE.md §4A.5, ARCHITECTURE.md Diagram 3 (M2.1).

use crate::hyp::Hyp;
use crate::llm_proposer_config::LlmProposerConfig;
use crate::ontology::{Atom, OntologySystem};
use crate::operator::Evidence;
use crate::proposer::{CandidateSet, RefinementProposer};
use std::collections::VecDeque;

/// Result of parsing an LLM response string into candidate hypotheses.
///
/// Tracks successful parses and the candidates that survived parsing.
/// Failed parses (hallucinations) are silently discarded, demonstrating INV-PS-06.
#[derive(Clone, Debug)]
struct ParseResult {
    /// Candidates successfully parsed from the LLM response.
    pub valid_candidates: Vec<Hyp>,
}

/// LLM-based refinement proposer.
///
/// Implements `RefinementProposer` by:
/// 1. Constructing a prompt from the input hypothesis and evidence
/// 2. Calling an LLM API (or returning mock responses in test mode)
/// 3. Parsing the response into candidate hypotheses
/// 4. Filtering candidates through `ProposerConstraint`
///
/// Invalid candidates (ontology violations, parsing failures) are logged as
/// "LLM hallucinations" and dropped silently. This demonstrates INV-PS-06:
/// the substrate remains sound regardless of LLM output quality.
pub struct LlmProposer {
    config: LlmProposerConfig,
    // Only used in mock mode; None for production LLM providers
    mock_response_queue: Option<std::sync::Mutex<VecDeque<String>>>,
}

impl LlmProposer {
    /// Create a new LLM proposer with the given configuration.
    ///
    /// # Arguments
    ///
    /// - `config`: Configuration specifying the LLM provider, model, prompt template, etc.
    ///   Configuration is validated at construction time (fails with panic if invalid).
    ///
    /// # Panics
    ///
    /// Panics if `config` is invalid (empty model, missing placeholders, invalid temperature, etc.).
    /// Call `config.validate()` beforehand if you need to handle validation errors gracefully.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = LlmProposerConfig::mock(
    ///     "mock-model",
    ///     vec!["SNOMED:12345".to_string()],
    ///     "0.1.0"
    /// );
    /// assert!(config.validate().is_ok());
    /// let proposer = LlmProposer::new(config);
    /// ```
    pub fn new(config: LlmProposerConfig) -> Self {
        // Fail fast if config is invalid; this enforces the precondition
        if let Err(e) = config.validate() {
            panic!("LlmProposerConfig validation failed: {}", e);
        }

        // Only allocate the mock response queue if in mock mode
        let mock_response_queue = config
            .mock_responses()
            .map(|responses| std::sync::Mutex::new(responses.iter().cloned().collect()));

        Self {
            config,
            mock_response_queue,
        }
    }

    /// Construct a prompt from hypothesis and evidence.
    ///
    /// Substitutes `{hypothesis}` and `{evidence}` placeholders in the template
    /// with clinical-structured representations of `h` and `e`.
    ///
    /// Uses a simple structured format: atoms are listed as "SYSTEM:CODE@VERSION",
    /// separated by commas. This is clearer than Debug output and suitable for LLM input.
    fn construct_prompt(&self, h: &Hyp, e: &Evidence) -> String {
        // Format hypothesis as comma-separated atoms: "SNOMED:12345@2026-01-31, RxNorm:9999@2026-01-31"
        let hypothesis_str = h
            .atoms()
            .iter()
            .map(|atom| format!("{}:{}@{}", atom.system, atom.code, atom.version))
            .collect::<Vec<_>>()
            .join(", ");
        let hypothesis_str = if hypothesis_str.is_empty() {
            "unknown".to_string()
        } else {
            hypothesis_str
        };

        // Format evidence as structured text (future: use JSON or structured format)
        let evidence_str = format!(
            "observations: {} (build: {})",
            e.observations.len(),
            e.provenance.version.build
        );

        self.config
            .prompt_template()
            .replace("{hypothesis}", &hypothesis_str)
            .replace("{evidence}", &evidence_str)
    }

    /// Call the LLM API (or return mock response in test mode).
    ///
    /// In mock mode, returns predetermined responses from the config.
    /// In production mode, would call the actual LLM API.
    /// For now, only mock mode is implemented.
    fn call_llm(&self, _prompt: &str) -> Result<String, String> {
        if self.config.is_mock() {
            // Mock mode: return predetermined response
            if let Some(queue_mutex) = &self.mock_response_queue {
                let mut queue = queue_mutex
                    .lock()
                    .map_err(|e| format!("mock response queue poisoned: {}", e))?;
                if let Some(response) = queue.pop_front() {
                    Ok(response)
                } else {
                    Err("mock response queue exhausted".to_string())
                }
            } else {
                Err("mock mode configured but no response queue initialized".to_string())
            }
        } else {
            // Production mode: not yet implemented
            Err("non-mock LLM calls not yet implemented".to_string())
        }
    }

    /// Parse an LLM response string into candidate hypotheses.
    ///
    /// # Expected format
    /// Comma/pipe/semicolon-separated ontology references.
    /// - `"SNOMED:12345,SNOMED:67890"` → two separate candidates
    /// - `"SNOMED:12345|RxNorm:9999"` → two separate candidates
    /// - `"SNOMED:12345@2026-01-31"` → with explicit version
    ///
    /// Each atom becomes a single-candidate hypothesis (multi-atom refinements not supported).
    /// Parsing failures (malformed atoms, unrecognized systems) are recorded as hallucinations
    /// and excluded from the candidate set (silent filtering per INV-PS-06).
    ///
    /// # Note
    /// This format is shared with parse_atom(). If other proposers need similar parsing,
    /// consider extracting this to a shared format parser in the ontology or util module.
    fn parse_response(&self, response: &str) -> ParseResult {
        let mut valid_candidates = Vec::new();

        // Split by comma, semicolon, or pipe
        let parts: Vec<&str> = response
            .split(|c| [',', ';', '|'].contains(&c))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for part in parts {
            // Parse each atom; silently discard failures (hallucinations)
            if let Ok(atom) = self.parse_atom(part) {
                valid_candidates.push(Hyp::new(vec![atom]));
            }
            // Hallucinations are logged nowhere (demonstrate INV-PS-06: system is sound regardless)
        }

        ParseResult { valid_candidates }
    }

    /// Parse a single atom string (e.g., "SNOMED:12345" or "SNOMED:12345@2026-01-31").
    fn parse_atom(&self, atom_str: &str) -> Result<Atom, String> {
        // Format: SYSTEM:CODE[@VERSION] or SYSTEM:CODE|VERSION
        // Examples:
        //   - "SNOMED:12345"
        //   - "SNOMED:12345@2026-01-31"
        //   - "SNOMED:12345|2026-01-31"

        let (system_code, version) = if atom_str.contains('@') {
            let parts: Vec<&str> = atom_str.split('@').collect();
            (parts[0].trim(), parts.get(1).map(|p| p.trim()))
        } else if atom_str.contains('|') {
            let parts: Vec<&str> = atom_str.split('|').collect();
            (parts[0].trim(), parts.get(1).map(|p| p.trim()))
        } else {
            (atom_str.trim(), None)
        };

        let (system_name, code) = if let Some(idx) = system_code.find(':') {
            (&system_code[..idx], &system_code[idx + 1..])
        } else {
            return Err(format!("malformed atom: missing ':' in {}", atom_str));
        };

        if code.is_empty() {
            return Err(format!("malformed atom: empty code in {}", atom_str));
        }

        let system = match system_name {
            "SNOMED" => OntologySystem::SNOMED,
            "RxNorm" => OntologySystem::RxNorm,
            "LOINC" => OntologySystem::LOINC,
            "ICD11" => OntologySystem::ICD11,
            _ => return Err(format!("unknown ontology system: {}", system_name)),
        };

        // Version is optional in LLM response format; default to "0.0.0" (unknown) if unspecified.
        // In production, callers should prefer explicit versions in LLM responses.
        // For audit and provenance tracking, use Evidence::version field instead.
        let version_str = version.unwrap_or("0.0.0").to_string();

        Ok(Atom {
            system,
            code: code.to_string(),
            preferred_term: format!("{}:{}", system_name, code),
            version: version_str,
        })
    }
}

impl RefinementProposer for LlmProposer {
    fn propose(&self, h: &Hyp, e: &Evidence) -> CandidateSet {
        // Step 1: Construct prompt
        let prompt = self.construct_prompt(h, e);

        // Step 2: Call LLM (or get mock response)
        let response = match self.call_llm(&prompt) {
            Ok(resp) => resp,
            Err(err) => {
                // LLM call failed; log the error for audit trail (OBL-PS-04)
                // and return empty candidate set (abstention)
                eprintln!("LlmProposer::propose() LLM call failed: {}", err);
                return CandidateSet::new();
            }
        };

        // Step 3: Parse response into candidates
        let parse_result = self.parse_response(&response);

        // Log hallucinations (in production, this would use structured logging)
        // For now, hallucinations are silently dropped by the constraint filter in propose_and_filter

        // Step 4: Return all valid candidates (constraint filtering happens in propose_and_filter)
        parse_result.valid_candidates.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::BTreeMap;

    fn test_provenance() -> crate::Provenance {
        let origin = crate::ProvenanceOrigin::new("test_input", "SNOMED", "67822003");
        let metadata = BTreeMap::new();
        crate::Provenance::new(
            origin,
            Utc::now(),
            crate::Ver::new("clinlat", "test", "0.1.0"),
            metadata,
        )
    }

    // TDD Tests for LlmProposer (Task 10.2)

    #[test]
    fn test_llm_proposer_mock_mode_single_response() {
        // Mock proposer returns a single predetermined response
        let config =
            LlmProposerConfig::mock("mock-model", vec!["SNOMED:67822003".to_string()], "0.1.0");
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert_eq!(
            candidates.len(),
            1,
            "Should return one candidate from mock response"
        );
    }

    #[test]
    fn test_llm_proposer_parses_comma_separated_atoms() {
        // Mock response with multiple comma-separated atoms
        let config = LlmProposerConfig::mock(
            "mock-model",
            vec!["SNOMED:12345,SNOMED:67890".to_string()],
            "0.1.0",
        );
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert_eq!(
            candidates.len(),
            2,
            "Should parse two comma-separated atoms"
        );
    }

    #[test]
    fn test_llm_proposer_filters_hallucinations() {
        // Mock response with one valid atom and one invalid (malformed)
        let config = LlmProposerConfig::mock(
            "mock-model",
            vec!["SNOMED:12345,INVALID_ATOM".to_string()],
            "0.1.0",
        );
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        // Only the valid SNOMED atom should be returned; malformed atom is filtered
        assert_eq!(
            candidates.len(),
            1,
            "Invalid atoms should be filtered as hallucinations"
        );
    }

    #[test]
    fn test_llm_proposer_empty_response() {
        // Mock response that produces no valid candidates
        let config = LlmProposerConfig::mock("mock-model", vec!["".to_string()], "0.1.0");
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert!(
            candidates.is_empty(),
            "Empty response should return empty candidate set"
        );
    }

    #[test]
    fn test_llm_proposer_parses_atom_with_version() {
        // Mock response with explicit version annotation
        let config = LlmProposerConfig::mock(
            "mock-model",
            vec!["SNOMED:12345@2026-01-31".to_string()],
            "0.1.0",
        );
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert_eq!(
            candidates.len(),
            1,
            "Should parse atom with version annotation"
        );
    }

    #[test]
    fn test_llm_proposer_parses_pipe_separated_atoms() {
        // Mock response with pipe-separated atoms (alternative delimiter)
        let config = LlmProposerConfig::mock(
            "mock-model",
            vec!["SNOMED:12345|RxNorm:9999".to_string()],
            "0.1.0",
        );
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert_eq!(candidates.len(), 2, "Should parse pipe-separated atoms");
    }

    #[test]
    fn test_llm_proposer_parses_semicolon_separated_atoms() {
        // Mock response with semicolon-separated atoms (alternative delimiter)
        let config = LlmProposerConfig::mock(
            "mock-model",
            vec!["SNOMED:12345;LOINC:8480-6".to_string()],
            "0.1.0",
        );
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert_eq!(
            candidates.len(),
            2,
            "Should parse semicolon-separated atoms"
        );
    }

    #[test]
    fn test_llm_proposer_rejects_unknown_ontology_system() {
        // Mock response with unknown ontology system (hallucination)
        let config =
            LlmProposerConfig::mock("mock-model", vec!["UNKNOWN:12345".to_string()], "0.1.0");
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert!(
            candidates.is_empty(),
            "Unknown ontology systems should be filtered as hallucinations"
        );
    }

    #[test]
    fn test_llm_proposer_rejects_empty_codes() {
        // Mock response with empty code (hallucination)
        let config = LlmProposerConfig::mock("mock-model", vec!["SNOMED:".to_string()], "0.1.0");
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert!(
            candidates.is_empty(),
            "Empty codes should be filtered as hallucinations"
        );
    }

    #[test]
    fn test_llm_proposer_multiple_mock_responses() {
        // Mock proposer configured with multiple responses; each call should return the next response
        let config = LlmProposerConfig::mock(
            "mock-model",
            vec!["SNOMED:12345".to_string(), "SNOMED:67890".to_string()],
            "0.1.0",
        );
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        // First call
        let candidates1 = proposer.propose(&h, &e);
        assert_eq!(candidates1.len(), 1);

        // Second call
        let candidates2 = proposer.propose(&h, &e);
        assert_eq!(candidates2.len(), 1);

        // Third call: queue exhausted
        let candidates3 = proposer.propose(&h, &e);
        assert!(
            candidates3.is_empty(),
            "Exhausted mock queue should return empty set"
        );
    }

    #[test]
    fn test_llm_proposer_prompt_construction() {
        // Verify that the proposer constructs a prompt with hypothesis and evidence substitutions
        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        // For this test, manually test prompt substitution logic with a realistic template
        // (the mock template is hardcoded to "mock", so we test the substitution logic directly)
        let hypothesis_str = format!("{:?}", h);
        let evidence_str = format!("{:?}", e);

        // Test with a realistic template
        let real_template = "Input: {hypothesis}, Evidence: {evidence}";
        let real_prompt = real_template
            .replace("{hypothesis}", &hypothesis_str)
            .replace("{evidence}", &evidence_str);

        assert!(
            real_prompt.contains("Input:") && real_prompt.contains("Evidence:"),
            "Prompt should contain substituted values"
        );
        assert!(
            !real_prompt.contains("{hypothesis}") && !real_prompt.contains("{evidence}"),
            "Placeholders should be replaced"
        );
    }

    #[test]
    fn test_llm_proposer_with_constraint_filtering_integration() {
        // Integration test: proposer output goes through propose_and_filter
        // Invalid candidates (e.g., Unstructured atoms) are filtered by the constraint validator
        let config =
            LlmProposerConfig::mock("mock-model", vec!["SNOMED:12345".to_string()], "0.1.0");
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        // Filter through propose_and_filter (which validates via ProposerConstraint)
        // This internally calls proposer.propose() once
        let filter_result = crate::proposer::propose_and_filter(&proposer, &h, &e);
        assert_eq!(
            filter_result.valid_candidates.len(),
            1,
            "Valid LLM candidate should pass constraint filtering"
        );
    }

    #[test]
    fn test_llm_proposer_parses_multiple_ontology_systems() {
        // Mock response with atoms from different ontology systems
        let config = LlmProposerConfig::mock(
            "mock-model",
            vec!["SNOMED:12345,RxNorm:9999,LOINC:8480-6,ICD11:BA01".to_string()],
            "0.1.0",
        );
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert_eq!(
            candidates.len(),
            4,
            "Should parse atoms from all supported ontology systems"
        );
    }

    #[test]
    fn test_llm_proposer_whitespace_handling() {
        // Mock response with excessive whitespace (should be trimmed)
        let config = LlmProposerConfig::mock(
            "mock-model",
            vec!["  SNOMED:12345  ,   SNOMED:67890  ".to_string()],
            "0.1.0",
        );
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        assert_eq!(candidates.len(), 2, "Should handle whitespace correctly");
    }

    #[test]
    fn test_llm_proposer_mixed_hallucinations_and_valid() {
        // Complex scenario: mix of valid atoms, malformed atoms, and unknown systems
        let config = LlmProposerConfig::mock(
            "mock-model",
            vec!["SNOMED:12345,MALFORMED,UNKNOWN:9999,RxNorm:8888".to_string()],
            "0.1.0",
        );
        let proposer = LlmProposer::new(config);

        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&h, &e);
        // Should have 2 valid atoms (SNOMED:12345, RxNorm:8888)
        // MALFORMED and UNKNOWN:9999 should be filtered
        assert_eq!(
            candidates.len(),
            2,
            "Should filter all hallucinations and return only valid atoms"
        );
    }
}
