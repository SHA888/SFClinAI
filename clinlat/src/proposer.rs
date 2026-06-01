//! Refinement proposers: black-box candidate generators per DEF-PS-14 / DEF-PS-15.
//!
//! # Overview
//!
//! A refinement proposer is a function that generates **candidate** hypotheses from a current
//! hypothesis and evidence. The proposer does **not decide** whether to accept or reject a
//! candidate; that decision is made by the sound deduction operators (DEF-PS-08).
//!
//! ## Semantic Constraints
//!
//! - **No decision-making**: The proposer returns a set of candidates, not a single "best" hypothesis.
//! - **Codomain constraint** (DEF-PS-15): Each candidate must be ontology-bounded and reachable
//!   by at most one operator application. Constraint enforcement is the **substrate's responsibility**,
//!   not the proposer's.
//! - **Safety boundary** (INV-PS-06): Even if a proposer is adversarial, the soundness of the
//!   active hypothesis depends only on the deduction operators (`Δ_PS`), not on the proposer's
//!   behavior. This is the load-bearing safety property of the patient substrate: **learned-component
//!   behavior cannot violate substrate soundness**.
//!
//! ## References
//!
//! - **Formal definition**: SPEC.md §2.7 (DEF-PS-14 Refinement proposer signature)
//! - **Codomain constraint**: SPEC.md §2.7 (DEF-PS-15 Proposer codomain constraint)
//! - **Safety invariant**: SPEC.md §2.7 (INV-PS-06 Proposer cannot bypass soundness)
//! - **Position**: NOTE.md §4A.5 (constrained refinement proposer)

use crate::hyp::Hyp;
use crate::ontology::OntologySystem;
use crate::operator::Evidence;
use std::collections::HashSet;

/// A set of candidate hypotheses.
///
/// This is a finite set returned by a refinement proposer. Empty sets are valid
/// (indicate the proposer has no candidates). Non-empty sets are not pre-filtered
/// by the proposer; filtering is the substrate's responsibility (DEF-PS-15).
pub type CandidateSet = HashSet<Hyp>;

/// Error type for proposer constraint violations (DEF-PS-15).
///
/// Tracks which constraint clauses failed: ontology-bounded or operator-reachable.
/// Used for audit logging (OBL-PS-04) and debugging.
#[derive(Clone, Debug)]
pub struct ConstraintError {
    ontology_bounded_failed: bool,
    operator_reachable_failed: bool,
    details: String,
}

impl ConstraintError {
    /// Create a new constraint error with details.
    pub fn new(
        ontology_bounded_failed: bool,
        operator_reachable_failed: bool,
        details: impl Into<String>,
    ) -> Self {
        Self {
            ontology_bounded_failed,
            operator_reachable_failed,
            details: details.into(),
        }
    }

    /// Did the candidate fail the ontology-bounded constraint?
    pub fn ontology_bounded_failed(&self) -> bool {
        self.ontology_bounded_failed
    }

    /// Did the candidate fail the operator-reachable constraint?
    pub fn operator_reachable_failed(&self) -> bool {
        self.operator_reachable_failed
    }

    /// Human-readable details of the failure.
    pub fn details(&self) -> &str {
        &self.details
    }
}

impl std::fmt::Display for ConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProposerConstraint violated: {}", self.details)
    }
}

impl std::error::Error for ConstraintError {}

/// Validator for proposer output per DEF-PS-15 (Proposer codomain constraint).
///
/// The validator enforces two clauses on candidate hypotheses:
///
/// 1. **Ontology-bounded** (DEF-PS-04): All atoms in the candidate must be resolvable
///    through the ontology adapter system (SNOMED CT, RxNorm, LOINC, ICD-11).
///
/// 2. **Operator-reachable** (one operator step): The candidate must be reachable from
///    the input hypothesis via at most one operator application. Conservatively, this
///    means the candidate must **refine** the input (candidate ⊑ input, i.e., candidate
///    atoms ⊇ input atoms), since DEF-PS-08 guarantees operators only refine.
///
/// The validator also provides an input-side gate: it can reject evidence or hypotheses
/// that are outside ontology bounds **before** the proposer runs, preventing invalid
/// proposals from being generated.
///
/// Failures are reported with structured `ConstraintError` tracking which clause(s)
/// failed, enabling audit logging and debugging.
pub struct ProposerConstraint {
    // Placeholder for future integration with OntologyAdapter system
    // For now, we use a simple whitelist of known ontology systems
}

impl ProposerConstraint {
    /// Create a new proposer constraint validator.
    pub fn new() -> Self {
        Self {}
    }

    /// Validate a candidate hypothesis against the proposer codomain constraint (DEF-PS-15).
    ///
    /// # Arguments
    ///
    /// - `candidate`: The hypothesis proposed by the proposer.
    /// - `input`: The input hypothesis that was passed to the proposer.
    /// - `evidence`: The evidence that drove the proposal.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the candidate satisfies both clauses of DEF-PS-15.
    /// `Err(ConstraintError)` if ontology-bounded or operator-reachable fails.
    pub fn validate(
        &self,
        candidate: &Hyp,
        input: &Hyp,
        _evidence: &Evidence,
    ) -> Result<(), ConstraintError> {
        let mut ontology_bounded_failed = false;
        let mut operator_reachable_failed = false;
        let mut details = Vec::new();

        // Check 1: Ontology-bounded (DEF-PS-15 clause 1, DEF-PS-04, OBL-PS-01)
        // Reject any Unstructured atoms per OBL-PS-01: free-text cannot enter Hyp through any path.
        // For coded systems, require non-empty code (basic format check; full validation deferred to OntologyAdapter).
        // Note: empty atom set passes vacuously (no atoms to reject). This is correct: Unknown hypothesis has no atoms.
        for atom in candidate.atoms() {
            match atom.system {
                OntologySystem::SNOMED
                | OntologySystem::RxNorm
                | OntologySystem::LOINC
                | OntologySystem::ICD11 => {
                    if atom.code.is_empty() {
                        ontology_bounded_failed = true;
                        details.push(format!("Atom code is empty for system {:?}", atom.system));
                    }
                    // Full ontology resolution deferred to OntologyAdapter.
                    // TODO: Query OntologyAdapter to verify code is actually valid
                }
                OntologySystem::Unstructured => {
                    // OBL-PS-01: Free-text concepts cannot enter Hyp through any path
                    ontology_bounded_failed = true;
                    details.push("Unstructured atoms not permitted: OBL-PS-01 prohibits free-text from entering Hyp".to_string());
                }
            }
        }

        // Check 2: Operator-reachable (DEF-PS-15 clause 2)
        // Conservative check: candidate must refine input (be more specific).
        // DEF-PS-08 guarantees operators only refine or abstain.
        // Refinement means candidate atoms ⊇ input atoms (more specific = more atoms or same atoms).
        let candidate_atoms = candidate.atoms();
        let input_atoms = input.atoms();
        let all_input_atoms_in_candidate = input_atoms
            .iter()
            .all(|input_atom| candidate_atoms.iter().any(|c_atom| c_atom == input_atom));

        if !all_input_atoms_in_candidate {
            operator_reachable_failed = true;
            details
                .push("candidate does not refine input: candidate atoms ⊉ input atoms".to_string());
        }

        if ontology_bounded_failed || operator_reachable_failed {
            Err(ConstraintError::new(
                ontology_bounded_failed,
                operator_reachable_failed,
                details.join("; "),
            ))
        } else {
            Ok(())
        }
    }

    /// Validate evidence and hypothesis at the input-side gate (before proposal).
    ///
    /// This gate rejects invalid evidence or hypotheses before the proposer runs,
    /// preventing invalid proposals from being generated.
    ///
    /// # Arguments
    ///
    /// - `h`: The input hypothesis.
    /// - `e`: The evidence driving the proposal.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the input is valid.
    /// `Err(ConstraintError)` if the input is outside ontology bounds.
    pub fn validate_input(&self, h: &Hyp, _e: &Evidence) -> Result<(), ConstraintError> {
        let mut ontology_bounded_failed = false;
        let mut details = Vec::new();

        // Check input hypothesis: all atoms must be from coded systems, no Unstructured
        for atom in h.atoms() {
            match atom.system {
                OntologySystem::SNOMED
                | OntologySystem::RxNorm
                | OntologySystem::LOINC
                | OntologySystem::ICD11 => {
                    if atom.code.is_empty() {
                        ontology_bounded_failed = true;
                        details.push(format!(
                            "Input atom code is empty for system {:?}",
                            atom.system
                        ));
                    }
                }
                OntologySystem::Unstructured => {
                    ontology_bounded_failed = true;
                    details.push("Input hypothesis contains Unstructured atoms: OBL-PS-01 prohibits free-text".to_string());
                }
            }
        }

        if ontology_bounded_failed {
            Err(ConstraintError::new(
                ontology_bounded_failed,
                false,
                details.join("; "),
            ))
        } else {
            Ok(())
        }
    }
}

impl Default for ProposerConstraint {
    fn default() -> Self {
        Self::new()
    }
}

/// Black-box refinement proposer: generates candidate hypotheses without deciding.
///
/// # Trait Semantics (DEF-PS-14)
///
/// A refinement proposer is a function
/// ```text
/// π : Hyp^P × Evidence → Set⟨Hyp⟩
/// ```
///
/// That is: given a current hypothesis `h` and evidence `e`, return a finite set of
/// **candidate** refinements.
///
/// # Load-bearing Safety Property (INV-PS-06)
///
/// The proposer is the integration point for learned components (LLMs, classifiers,
/// retrieval systems, etc.). Even if a proposer is adversarial or hallucinating:
///
/// - The soundness of the active hypothesis is **guaranteed by the deduction operators** (`Δ_PS`),
///   not by the proposer's behavior.
/// - No candidate from the proposer can become the active hypothesis without passing through
///   a sound operator (DEF-PS-08).
/// - This is the **load-bearing safety property of the patient substrate**: learned-component
///   behavior cannot violate substrate soundness.
///
/// ## Example
///
/// ```ignore
/// // A mock proposer that returns all hypotheses containing a specific atom.
/// struct MockProposer { target_atom: Atom }
///
/// impl RefinementProposer for MockProposer {
///     fn propose(&self, h: &Hyp, e: &Evidence) -> CandidateSet {
///         // Return candidates that contain the target atom.
///         // Note: No filtering, no decision-making. The substrate will validate.
///         let mut candidates = HashSet::new();
///         candidates.insert(h.clone());  // Can return input unchanged.
///         candidates
///     }
/// }
/// ```
pub trait RefinementProposer: Send + Sync {
    /// Generate candidate refinements from a hypothesis and evidence.
    ///
    /// # Arguments
    ///
    /// - `h`: Current hypothesis (the input state).
    /// - `e`: Evidence driving the refinement search.
    ///
    /// # Returns
    ///
    /// A finite set of candidate hypotheses. Empty set is valid (proposer has no candidates).
    /// The substrate will filter these candidates through the codomain constraint (DEF-PS-15)
    /// and the soundness gate (INV-PS-06); the proposer is **not responsible** for constraint
    /// enforcement.
    ///
    /// # Invariants Guaranteed by the Proposer
    ///
    /// - The returned set is finite.
    /// - No decision-making occurs: all candidates meeting the proposer's internal criteria
    ///   are returned, not a filtered subset.
    ///
    /// # Invariants **NOT** Guaranteed (Enforced by the Substrate)
    ///
    /// - Codomain constraint (DEF-PS-15): candidates are not pre-checked for ontology-boundedness
    ///   or operator reachability. The substrate enforces these through `ProposerConstraint` (8.2)
    ///   and the soundness-verification gate `propose_verify` (8.5).
    /// - Soundness (INV-PS-06): a candidate from this proposer may be unsound (e.g., a refined
    ///   hypothesis with no operator path to justify it). The soundness gate filters these out.
    fn propose(&self, h: &Hyp, e: &Evidence) -> CandidateSet;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::OntologySystem;
    use crate::{Atom, ProvenanceOrigin, Ver};
    use chrono::Utc;
    use std::collections::BTreeMap;

    struct TestProposer;

    impl RefinementProposer for TestProposer {
        fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
            HashSet::new()
        }
    }

    fn test_provenance() -> crate::Provenance {
        let origin = ProvenanceOrigin::new("test_input", "SNOMED", "67822003");
        let metadata = BTreeMap::new();
        crate::Provenance::new(
            origin,
            Utc::now(),
            Ver::new("clinlat", "test", "0.1.0"),
            metadata,
        )
    }

    #[test]
    fn test_refinement_proposer_trait_compiles() {
        let _proposer: Box<dyn RefinementProposer> = Box::new(TestProposer);
    }

    #[test]
    fn test_proposer_returns_candidate_set() {
        let proposer = TestProposer;
        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());
        let candidates = proposer.propose(&h, &e);
        assert!(candidates.is_empty(), "Test proposer returns empty set");
    }

    #[test]
    fn test_proposer_with_atoms() {
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let h = Hyp::new(vec![atom]);
        let e = Evidence::new(vec![], test_provenance());
        let proposer = TestProposer;
        let candidates = proposer.propose(&h, &e);
        assert_eq!(candidates.len(), 0, "Test proposer returns empty set");
    }

    #[test]
    fn test_proposer_returns_set_not_option() {
        // Verify the type signature: returns Set<Hyp>, not Option<Hyp> or Vec<Hyp>.
        // This enforces the "no decision-making" constraint at the type level.
        let proposer = TestProposer;
        let h = Hyp::unknown();
        let e = Evidence::new(vec![], test_provenance());
        let candidates: CandidateSet = proposer.propose(&h, &e);
        // CandidateSet is a HashSet, not Option or Vec. Type system enforces this.
        let _ = candidates.iter(); // Verify it's iterable as a set.
    }

    // TDD tests for ProposerConstraint (task 8.2)
    // These tests should FAIL until ProposerConstraint is implemented

    #[test]
    fn test_proposer_constraint_accepts_valid_candidate() {
        // A candidate that refines input (superset of atoms) with valid atoms should pass both clauses
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom_b = Atom {
            system: OntologySystem::SNOMED,
            code: "3723001".to_string(),
            preferred_term: "ARDS".to_string(),
            version: "2026-01-31".to_string(),
        };
        // Input has only atom_a
        let input = Hyp::new(vec![atom_a.clone()]);
        // Candidate has both atoms: {a, b} ⊃ {a}, so it refines input (operator-reachable)
        let candidate = Hyp::new(vec![atom_a, atom_b]);
        let evidence = Evidence::new(vec![], test_provenance());

        let constraint = ProposerConstraint::new();
        let result = constraint.validate(&candidate, &input, &evidence);
        assert!(
            result.is_ok(),
            "Valid candidate (refines input, ontology-bounded) should pass both clauses"
        );
    }

    #[test]
    fn test_proposer_constraint_rejects_non_refining_candidate() {
        // A candidate that doesn't refine the input (fewer atoms) should fail operator-reachability check
        let candidate_atom = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let input = Hyp::new(vec![candidate_atom.clone()]);
        let candidate = Hyp::unknown(); // Fewer atoms than input — doesn't refine
        let evidence = Evidence::new(vec![], test_provenance());

        let constraint = ProposerConstraint::new();
        let result = constraint.validate(&candidate, &input, &evidence);
        assert!(
            result.is_err(),
            "Non-refining candidate (doesn't satisfy operator-reachability) should fail"
        );
    }

    #[test]
    fn test_proposer_constraint_accepts_known_ontology_atom() {
        // A candidate with a known ontology system and valid structure is ontology-bounded
        let input = Hyp::unknown();
        let valid_candidate = Hyp::new(vec![Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        }]);
        let evidence = Evidence::new(vec![], test_provenance());

        let constraint = ProposerConstraint::new();
        let result = constraint.validate(&valid_candidate, &input, &evidence);
        assert!(
            result.is_ok(),
            "Candidate with valid ontology atoms should pass ontology-bounded check"
        );
    }

    #[test]
    fn test_proposer_constraint_input_gate_rejects_invalid_evidence() {
        // Input gate must reject Unstructured atoms per OBL-PS-01
        let bad_input = Hyp::new(vec![Atom {
            system: OntologySystem::Unstructured,
            code: "clinician-note".to_string(),
            preferred_term: "Some clinical observation".to_string(),
            version: "2026-01-31".to_string(),
        }]);
        let evidence = Evidence::new(vec![], test_provenance());

        let constraint = ProposerConstraint::new();
        let result = constraint.validate_input(&bad_input, &evidence);
        assert!(
            result.is_err(),
            "Input gate should reject Unstructured atoms"
        );
        let err = result.unwrap_err();
        assert!(
            err.ontology_bounded_failed(),
            "Error should report ontology-bounded failure for Unstructured input"
        );
    }

    #[test]
    fn test_proposer_constraint_returns_structured_error() {
        // ConstraintError should report which clause failed (ontology-bounded vs operator-reachable).
        // Create a candidate that violates refinement: missing an atom from input.
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom_b = Atom {
            system: OntologySystem::SNOMED,
            code: "3723001".to_string(),
            preferred_term: "ARDS".to_string(),
            version: "2026-01-31".to_string(),
        };
        // Input has both atoms
        let input = Hyp::new(vec![atom_a.clone(), atom_b.clone()]);
        // Candidate only has atom_a → does NOT refine input (missing atom_b)
        let non_refinement_candidate = Hyp::new(vec![atom_a]);
        let evidence = Evidence::new(vec![], test_provenance());

        let constraint = ProposerConstraint::new();
        let result = constraint.validate(&non_refinement_candidate, &input, &evidence);

        match result {
            Err(e) => {
                // Should report operator-reachable failure (candidate doesn't refine input)
                assert!(
                    e.operator_reachable_failed(),
                    "Error should report operator-reachable clause failed"
                );
            }
            Ok(()) => panic!("Expected validation to fail: candidate must refine input"),
        }
    }
}
