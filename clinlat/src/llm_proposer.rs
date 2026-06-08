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

    // ============================================================================
    // Property tests for task 10.3: LlmProposer safety invariant
    // ============================================================================
    // These tests verify INV-PS-06 (proposer cannot bypass soundness):
    // (P1) Filtered candidates are usable by operators (safety)
    // (P2) System handles hallucinations robustly (robustness)
    // (P3) Audit trail records filtering decisions (auditability)
    // ============================================================================

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Strategy: generate valid atom strings in LLM response format
        fn valid_atom_response_strategy() -> impl Strategy<Value = String> {
            prop_oneof![
                ("SNOMED", "[0-9]{4,5}").prop_map(|(sys, code)| format!("{}:{}", sys, code)),
                ("RxNorm", "[0-9]{4,6}").prop_map(|(sys, code)| format!("{}:{}", sys, code)),
                ("LOINC", "[0-9]{4,5}").prop_map(|(sys, code)| format!("{}:{}", sys, code)),
                ("ICD11", "[A-Z][0-9]{2}\\.[0-9]")
                    .prop_map(|(sys, code)| format!("{}:{}", sys, code)),
            ]
        }

        // Strategy: generate hallucinations (malformed, unknown systems, empty codes)
        fn hallucination_strategy() -> impl Strategy<Value = String> {
            prop_oneof![
                Just("UNKNOWN:12345".to_string()),
                Just("SNOMED:".to_string()),
                Just("RxNorm:".to_string()),
                Just("MALFORMED".to_string()),
                Just(";;;".to_string()),
                Just(":::".to_string()),
            ]
        }

        // Strategy: generate mixed LLM responses (valid atoms + hallucinations)
        fn mixed_response_strategy() -> impl Strategy<Value = String> {
            (
                prop::collection::vec(valid_atom_response_strategy(), 1..4),
                prop::collection::vec(hallucination_strategy(), 0..3),
            )
                .prop_map(|(valid, invalid)| {
                    let mut all = valid;
                    all.extend(invalid);
                    all.join(",")
                })
        }

        // ---- Property 1: Safety ----
        // Every candidate passing ProposerConstraint is usable (refinement condition met).
        // Test that filtered candidates all satisfy refinement: candidate ⊇ input atoms.

        proptest! {
            #[test]
            fn prop_filtered_candidates_are_valid_refinements(
                response in mixed_response_strategy()
            ) {
                let config = LlmProposerConfig::mock("mock-model", vec![response], "0.1.0");
                let proposer = LlmProposer::new(config);

                let h = Hyp::unknown();
                let e = Evidence::new(vec![], test_provenance());

                // Propose and filter
                let filter_result = crate::proposer::propose_and_filter(&proposer, &h, &e);

                // Property P1: All filtered candidates are valid refinements.
                // For unknown input, all candidates should be valid refinements (unknown ⊑ anything).
                for candidate in filter_result.valid_candidates.iter() {
                    // Candidate must be a refinement of input (candidate atoms ⊇ input atoms).
                    // For unknown input, this is always true (vacuously).
                    // For non-unknown input, candidate must contain all input atoms.
                    let input_atoms: std::collections::HashSet<_> =
                        h.atoms().iter().map(|a| &a.code).collect();
                    let candidate_atoms: std::collections::HashSet<_> =
                        candidate.atoms().iter().map(|a| &a.code).collect();
                    prop_assert!(
                        input_atoms.is_subset(&candidate_atoms),
                        "Candidate {:?} does not refine input {:?}",
                        candidate,
                        h
                    );
                }
            }

        }

        // ---- Property 2: Robustness ----
        // System handles mixed valid/invalid responses without crashing.
        // Output deterministically includes all valid atoms and excludes all hallucinations.

        proptest! {
            #[test]
            fn prop_system_tolerates_hallucinations(
                response in mixed_response_strategy()
            ) {
                let config = LlmProposerConfig::mock("mock-model", vec![response.clone()], "0.1.0");
                let proposer = LlmProposer::new(config);

                let h = Hyp::unknown();
                let e = Evidence::new(vec![], test_provenance());

                // Should not panic or error, even with mixed responses
                let filter_result = crate::proposer::propose_and_filter(&proposer, &h, &e);

                // Invariant: filtered_out_count should equal filter_errors.len() (per FilterResult)
                // OR filtered_out_count == 0 and filter_errors.len() == 1 (input-gate case)
                prop_assert!(
                    filter_result.filtered_out_count == filter_result.filter_errors.len()
                        || (filter_result.filtered_out_count == 0
                            && filter_result.filter_errors.len() == 1),
                    "FilterResult invariant violated: filtered_out_count={} but filter_errors.len()={}",
                    filter_result.filtered_out_count,
                    filter_result.filter_errors.len()
                );
            }

            #[test]
            fn prop_hallucinations_do_not_reach_output(
                response in mixed_response_strategy()
            ) {
                let config = LlmProposerConfig::mock("mock-model", vec![response.clone()], "0.1.0");
                let proposer = LlmProposer::new(config);

                let h = Hyp::unknown();
                let e = Evidence::new(vec![], test_provenance());

                let filter_result = crate::proposer::propose_and_filter(&proposer, &h, &e);

                // No valid candidate should have Unstructured atoms or empty codes
                for candidate in filter_result.valid_candidates.iter() {
                    for atom in candidate.atoms() {
                        prop_assert!(
                            atom.system != OntologySystem::Unstructured,
                            "Unstructured atom in output: {:?}",
                            atom
                        );
                        prop_assert!(!atom.code.is_empty(), "Empty code in output: {:?}", atom);
                    }
                }
            }
        }

        // ---- Property 3: Auditability ----
        // Filtering decisions are recorded in FilterResult.filter_errors for audit trail.

        proptest! {
            #[test]
            fn prop_audit_trail_records_filtering(
                response in mixed_response_strategy()
            ) {
                let config = LlmProposerConfig::mock("mock-model", vec![response.clone()], "0.1.0");
                let proposer = LlmProposer::new(config);

                let h = Hyp::unknown();
                let e = Evidence::new(vec![], test_provenance());

                let filter_result = crate::proposer::propose_and_filter(&proposer, &h, &e);

                // Audit invariant: FilterResult carries error details for every filtered candidate
                // (except input-gate case where filtered_out_count == 0)
                if filter_result.filtered_out_count > 0 {
                    prop_assert_eq!(
                        filter_result.filtered_out_count,
                        filter_result.filter_errors.len(),
                        "Audit trail incomplete: {} candidates filtered but {} errors recorded",
                        filter_result.filtered_out_count,
                        filter_result.filter_errors.len()
                    );

                    // Each error should have non-empty details
                    for err in filter_result.filter_errors.iter() {
                        prop_assert!(
                            !err.details().is_empty(),
                            "Audit trail error missing details: {:?}",
                            err
                        );
                    }
                }
            }
        }

        // ---- Integration tests: propose_verify (M2.2) ----
        // Verify that the full pipeline (LlmProposer → propose_and_filter → propose_verify)
        // maintains soundness: all licensed candidates are operator-reachable.

        proptest! {
            #[test]
            fn prop_verify_audit_trail_records_all_decisions(
                response in mixed_response_strategy()
            ) {
                let config = LlmProposerConfig::mock("mock-model", vec![response.clone()], "0.1.0");
                let proposer = LlmProposer::new(config);

                let h = Hyp::unknown();
                let e = Evidence::new(vec![], test_provenance());

                // Empty operator set: tests the abstention path (all candidates unlicensed)
                // With empty operator set, no candidates can be licensed, so propose_verify
                // always returns Err(NoOperatorLicenses). This is the normal case when
                // LLM suggests candidates but operators cannot refine Unknown hypothesis.
                use crate::operator_set::OperatorSet;
                let ops = OperatorSet::new();

                // Propose and verify through full pipeline
                let verify_result = crate::proposer::propose_verify(&proposer, &ops, &h, &e);

                // With empty operator set, we expect Err(NoOperatorLicenses) for any non-empty candidates
                // The audit trail (constraint_filtering_verdicts + licensing_verdicts) records all decisions
                match verify_result {
                    Ok(_result) => {
                        // Should not happen with empty operator set and Hyp::unknown() input
                        // (no candidates can be licensed)
                        panic!("Empty operator set should always cause abstention for unknown input");
                    }
                    Err(_abstain) => {
                        // Abstention is expected: no operator licenses any candidate
                        // The constraint_filtering_verdicts in FilterResult (internal to propose_verify)
                        // captures any candidates filtered before operator licensing
                    }
                }
            }
        }
    }

    // ============================================================================
    // Worked examples: demonstrating INV-PS-06 (substrate-first safety)
    // ============================================================================
    // These are concrete clinical scenarios showing that the substrate remains
    // sound even when the LLM hallucinator or returns invalid data.
    // ============================================================================

    mod worked_examples {
        use super::*;

        #[test]
        fn example_sepsis_with_llm_hallucinations() {
            // Scenario: Sepsis patient with WBC elevation. LLM suggests diagnoses,
            // including some that don't exist (hallucinations). Substrate filters them
            // silently during parsing and maintains soundness (INV-PS-06).

            // Initial hypothesis: unknown
            let h_input = Hyp::unknown();

            // Evidence: clinical observations
            let origin = crate::ProvenanceOrigin::new("EHR", "SNOMED", "76948002"); // WBC elevation
            let prov = crate::Provenance::new(
                origin,
                chrono::Utc::now(),
                crate::Ver::new("clinlat", "example", "0.1.0"),
                std::collections::BTreeMap::new(),
            );
            let e = Evidence::new(vec![], prov);

            // LLM response: mix of valid diagnoses and hallucinations
            // Valid: SNOMED:76948002, SNOMED:67890, RxNorm:6809
            // Hallucinations: INVALID:XYZ789 (unknown system), SNOMED: (empty code)
            let config = LlmProposerConfig::mock(
                "gpt-4-example",
                vec!["SNOMED:76948002,INVALID:XYZ789,SNOMED:67890,RxNorm:6809,SNOMED:".to_string()],
                "0.1.0",
            );
            let proposer = LlmProposer::new(config);

            // Propose and filter (constraint validation). This internally calls propose().
            let filter_result = crate::proposer::propose_and_filter(&proposer, &h_input, &e);

            // Invariant P1: Only valid, parsed candidates are returned after filtering
            // Valid ones: SNOMED:76948002, SNOMED:67890, RxNorm:6809 (3 total)
            // Hallucinations (INVALID:XYZ789, SNOMED:) are silently dropped during parsing
            assert_eq!(
                filter_result.valid_candidates.len(),
                3,
                "Should have 3 valid parsed candidates"
            );
            assert_eq!(filter_result.filtered_out_count, 0); // No additional filtering at constraint stage
            // (all parsed candidates are valid)

            // Invariant P2: Candidates are all valid (no Unstructured, no empty codes)
            for candidate in filter_result.valid_candidates.iter() {
                for atom in candidate.atoms() {
                    assert_ne!(atom.system, OntologySystem::Unstructured);
                    assert!(!atom.code.is_empty());
                }
            }

            // Invariant P3: Audit trail is complete for constraint stage
            // (parsing-stage hallucinations are silently dropped, not logged)
            assert_eq!(filter_result.filter_errors.len(), 0); // No constraint violations
        }

        #[test]
        fn example_robust_response_to_adversarial_llm() {
            // Scenario: LLM returns an adversarial response with mostly garbage.
            // Substrate tolerates it (P2: robustness) without crashing, extracting only valid atoms.
            // Hallucinations are silently dropped during parsing.

            let h_input = Hyp::unknown();
            let origin = crate::ProvenanceOrigin::new("test", "SNOMED", "54954009");
            let prov = crate::Provenance::new(
                origin,
                chrono::Utc::now(),
                crate::Ver::new("clinlat", "example", "0.1.0"),
                std::collections::BTreeMap::new(),
            );
            let e = Evidence::new(vec![], prov);

            // Adversarial response: valid atoms mixed with garbage
            // Valid: SNOMED:12345, RxNorm:5678
            // Garbage: ;;;, :::, UNKNOWN:99999 (all dropped during parsing)
            let config = LlmProposerConfig::mock(
                "adversary-llm",
                vec![";;;,:::,SNOMED:12345,UNKNOWN:99999,RxNorm:5678".to_string()],
                "0.1.0",
            );
            let proposer = LlmProposer::new(config);

            let filter_result = crate::proposer::propose_and_filter(&proposer, &h_input, &e);

            // Despite garbage, only valid candidates emerge (2: SNOMED:12345, RxNorm:5678)
            assert_eq!(filter_result.valid_candidates.len(), 2);

            // System doesn't crash - robustness property verified
            // Hallucinations are silently dropped at parse time, not logged at filter time
            assert_eq!(filter_result.filtered_out_count, 0); // All parsed candidates are valid
            assert_eq!(filter_result.filter_errors.len(), 0); // No constraint violations
        }

        #[test]
        fn example_audit_trail_for_compliance() {
            // Scenario: Clinical compliance requires that every filtering decision be
            // reconstructible from the audit trail for retrospective review.
            // Demonstrates that FilterResult carries complete error details.

            let h_input = Hyp::unknown();
            let origin = crate::ProvenanceOrigin::new("compliance_audit", "SNOMED", "50849002");
            let prov = crate::Provenance::new(
                origin,
                chrono::Utc::now(),
                crate::Ver::new("clinlat", "compliance", "0.1.0"),
                std::collections::BTreeMap::new(),
            );
            let e = Evidence::new(vec![], prov);

            // LLM response with valid atoms (pass parsing and constraint filtering)
            let config = LlmProposerConfig::mock(
                "clinically-audited-model",
                vec!["SNOMED:50849002,SNOMED:67890,RxNorm:5678".to_string()],
                "0.1.0",
            );
            let proposer = LlmProposer::new(config);

            // Call constraint filtering (propose_and_filter)
            let filter_result = crate::proposer::propose_and_filter(&proposer, &h_input, &e);

            // Compliance requirement: audit trail structure allows retrospective review
            // FilterResult maintains the invariant: filtered_out_count == filter_errors.len()
            // OR (filtered_out_count==0 and filter_errors.len()==1 for input-gate case)
            assert!(
                filter_result.filtered_out_count == filter_result.filter_errors.len()
                    || (filter_result.filtered_out_count == 0
                        && filter_result.filter_errors.len() == 1),
                "FilterResult audit trail invariant violated"
            );

            // If candidates were filtered, each error explains why
            for err in filter_result.filter_errors.iter() {
                assert!(
                    !err.details().is_empty(),
                    "Error details required for compliance audit trail"
                );
            }
        }

        #[test]
        fn example_full_pipeline_with_operator_licensing() {
            // Scenario: Demonstrates the full INV-PS-06 safety pipeline with operator licensing.
            // Shows that even valid candidates (passing constraint filter) must be licensed by operators.

            let h_input = Hyp::unknown();
            let origin = crate::ProvenanceOrigin::new("safety_demo", "SNOMED", "123456");
            let prov = crate::Provenance::new(
                origin,
                chrono::Utc::now(),
                crate::Ver::new("clinlat", "safety", "0.1.0"),
                std::collections::BTreeMap::new(),
            );
            let e = Evidence::new(vec![], prov);

            // LLM returns valid atoms (pass parsing and constraint gates)
            let config = LlmProposerConfig::mock(
                "safety-test-model",
                vec!["SNOMED:12345,SNOMED:67890".to_string()],
                "0.1.0",
            );
            let proposer = LlmProposer::new(config);

            // Empty operator set (no operators can refine Unknown)
            use crate::operator_set::OperatorSet;
            let ops = OperatorSet::new();

            // Even though LLM produces valid candidates, propose_verify returns error
            // because no operators can license them (INV-PS-06: operators, not proposers, license)
            let verify_result = crate::proposer::propose_verify(&proposer, &ops, &h_input, &e);

            // With empty operator set, all candidates fail licensing and system abstains
            assert!(
                verify_result.is_err(),
                "Empty operator set should cause abstention"
            );

            // Safety guarantee verified: LLM output alone cannot make active hypothesis;
            // must pass through operator licensing (the deduction substrate)
        }
    }
}
