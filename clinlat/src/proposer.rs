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

/// Result of filtering proposer output through constraint validation.
///
/// Returned by `propose_and_filter` to track valid candidates, filtered count, and errors.
///
/// # Invariant
///
/// In the normal (post-proposer) path: `filtered_out_count == filter_errors.len()`.
/// The single exception is an input-gate rejection, where `filtered_out_count == 0`
/// (no proposer output was produced) but `filter_errors.len() == 1` (the input-gate
/// `ConstraintError` is recorded). Callers can distinguish the two cases by checking
/// `filtered_out_count == 0 && !filter_errors.is_empty()`.
#[derive(Clone, Debug)]
pub struct FilterResult {
    /// Candidates that passed the ontology-bounded and operator-reachable checks.
    pub valid_candidates: CandidateSet,
    /// Number of proposer output candidates rejected by the output-side gate.
    /// Zero when the function returns early due to an input-gate failure (no candidates
    /// were produced); in that case `filter_errors` carries the input-gate error.
    pub filtered_out_count: usize,
    /// Structured errors for audit trail and debugging.
    ///
    /// In the normal path: one entry per rejected output candidate (ontology or
    /// reachability violation). In the input-gate early-return path: one entry
    /// for the input-side validation failure, with `filtered_out_count == 0`.
    pub filter_errors: Vec<ConstraintError>,
}

impl FilterResult {
    /// Create a new filter result.
    ///
    /// # Panics (debug only)
    ///
    /// Panics in debug builds if the post-proposer invariant
    /// `filtered_out_count == filter_errors.len()` is violated without being an
    /// input-gate case (`filtered_out_count == 0, filter_errors.len() == 1`).
    pub fn new(
        valid_candidates: CandidateSet,
        filtered_out_count: usize,
        filter_errors: Vec<ConstraintError>,
    ) -> Self {
        // Assert the invariant: filtered_out_count must equal filter_errors.len(),
        // except in the input-gate early-return case where filtered_out_count == 0
        // and filter_errors holds exactly one input-gate error.
        debug_assert!(
            filtered_out_count == filter_errors.len()
                || (filtered_out_count == 0 && filter_errors.len() == 1),
            "FilterResult invariant violated: filtered_out_count={} but filter_errors.len()={}; \
             these must be equal except when filtered_out_count==0 (input-gate rejection)",
            filtered_out_count,
            filter_errors.len()
        );
        Self {
            valid_candidates,
            filtered_out_count,
            filter_errors,
        }
    }

    /// Check if all candidates passed filtering (no errors).
    pub fn all_passed(&self) -> bool {
        self.filter_errors.is_empty()
    }

    /// Check if any candidates passed filtering.
    pub fn has_valid_candidates(&self) -> bool {
        !self.valid_candidates.is_empty()
    }
}

/// Adapter that calls a proposer and filters output through constraint validation.
///
/// This is the implementation of the **output-side gate** in Diagram 3 (M2.1).
/// It enforces DEF-PS-15 codomain constraints on every candidate returned by a proposer,
/// preventing invalid hypotheses from reaching the deduction operators (soundness gate).
///
/// # Semantics
///
/// Given a proposer `π`, hypothesis `h`, and evidence `e`:
///
/// 1. Call `π(h, e)` to get a set of candidates.
/// 2. For each candidate in the set:
///    - Validate it against `ProposerConstraint` (ontology-bounded + operator-reachable).
///    - If valid, add to result set.
///    - If invalid, record error and increment filtered count.
/// 3. Return `FilterResult` with valid set, filtered count, and error list.
///
/// # Audit trail (OBL-PS-04)
///
/// OBL-PS-04 requires that every filtering decision be reconstructible. Currently this
/// function returns structured `ConstraintError` values in `FilterResult.filter_errors`
/// for downstream recording; it does **not** itself emit log output.
///
/// TODO (OBL-PS-04): wire structured logging here once a tracing subscriber is
/// wired into the clinlat kernel (e.g., `tracing::debug!("propose_and_filter: \
/// candidate {:?} rejected: {}", candidate, err)`). Tracking: task 8.4 / 8.5.
pub fn propose_and_filter(
    proposer: &dyn RefinementProposer,
    h: &Hyp,
    e: &Evidence,
) -> FilterResult {
    // Validate input hypothesis before calling proposer (input-side gate per M2.1)
    let constraint = ProposerConstraint::new();
    if let Err(input_error) = constraint.validate_input(h, e) {
        // Input is invalid: return empty result with error (proposer never called)
        return FilterResult::new(CandidateSet::new(), 0, vec![input_error]);
    }

    // Call proposer to get candidates
    let candidates = proposer.propose(h, e);
    let total_count = candidates.len();

    // Filter candidates through constraint validator
    let mut valid_candidates = CandidateSet::new();
    let mut filter_errors = Vec::new();

    for candidate in candidates.iter() {
        match constraint.validate(candidate, h, e) {
            Ok(()) => {
                valid_candidates.insert(candidate.clone());
            }
            Err(err) => {
                filter_errors.push(err);
            }
        }
    }

    let filtered_out_count = total_count - valid_candidates.len();

    FilterResult::new(valid_candidates, filtered_out_count, filter_errors)
}

/// Result of propose_verify: candidates licensed by the soundness gate.
///
/// Returned by `propose_verify` to track which candidates passed the soundness gate
/// (operator licensing check). The audit trail records verdicts for traceability.
///
/// # Invariant
///
/// `licensed_candidates ⊆ input_candidates` (filtering, never expansion).
/// If `licensed_candidates.is_empty()`, the substrate should return
/// `Err(AbstainReason::NoOperatorLicenses)` instead of this type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyResult {
    /// Candidates whose atoms are reachable by the operator set (licensed).
    pub licensed_candidates: CandidateSet,
    /// Candidates that were rejected by the soundness gate (operator-unreachable).
    pub unlicensed_candidates: CandidateSet,
    /// Audit trail: which candidates passed/failed and why.
    /// Format: (candidate_atoms_string, is_licensed, operator_set_result_atoms_string)
    pub licensing_verdicts: Vec<(String, bool, String)>,
    /// Constraint filtering audit trail (OBL-PS-04): candidates rejected before operator licensing.
    /// Format: (candidate_atoms_string, constraint_error_message)
    pub constraint_filtering_verdicts: Vec<(String, String)>,
}

impl VerifyResult {
    /// Create a new verify result.
    ///
    /// # Panics (debug only)
    /// Panics in debug builds if licensed and unlicensed candidates are not disjoint.
    pub fn new(
        licensed_candidates: CandidateSet,
        unlicensed_candidates: CandidateSet,
        licensing_verdicts: Vec<(String, bool, String)>,
        constraint_filtering_verdicts: Vec<(String, String)>,
    ) -> Self {
        debug_assert!(
            licensed_candidates.is_disjoint(&unlicensed_candidates),
            "VerifyResult invariant violated: candidate appears in both licensed and unlicensed"
        );
        Self {
            licensed_candidates,
            unlicensed_candidates,
            licensing_verdicts,
            constraint_filtering_verdicts,
        }
    }

    /// Check if any candidates were licensed.
    pub fn has_licensed_candidates(&self) -> bool {
        !self.licensed_candidates.is_empty()
    }

    /// Check if all candidates were unlicensed.
    pub fn all_unlicensed(&self) -> bool {
        !self.unlicensed_candidates.is_empty() && self.licensed_candidates.is_empty()
    }
}

/// Adapter that routes proposer output through the soundness-verification gate.
///
/// This is the implementation of the **soundness-verification (SV) node** in Diagram 3 (M2.2).
/// It enforces DEF-PS-08 licensing on every candidate accepted by `propose_and_filter`,
/// preventing unsound hypotheses from becoming active.
///
/// # Semantics
///
/// Given proposer output (via `propose_and_filter`), operator set, hypothesis, and evidence:
///
/// 1. Call `propose_and_filter` to get valid candidates (constraint-passing).
/// 2. Apply operator set to input hypothesis to get the soundness-verified result.
/// 3. For each valid candidate, check if its atoms are included in the result:
///    - If candidate.atoms ⊆ result.atoms, the candidate is licensed (on the path to the result).
///    - Otherwise, the candidate is unlicensed (no operator path justifies it).
/// 4. Return `Ok(VerifyResult)` with licensed candidates and audit trail, or
///    `Err(AbstainReason::NoOperatorLicenses)` if all candidates are unlicensed.
///
/// # Invariant (INV-PS-06 Enforcement)
///
/// Every hypothesis that emerges from propose_verify is guaranteed to be reachable
/// by applying some subset of the operator set to the input hypothesis.
/// This is the load-bearing safety property: learned-component output cannot bypass soundness.
///
/// # Audit Trail (OBL-PS-04)
///
/// The audit trail captures all filtering decisions across both constraint validation
/// and operator licensing. This enables full reconstruction of why a candidate was accepted or rejected.
/// - `constraint_filtering_verdicts`: candidates rejected by ProposerConstraint (DEF-PS-15)
/// - `licensing_verdicts`: candidates that passed constraint validation and their operator licensing outcomes
pub fn propose_verify(
    proposer: &dyn RefinementProposer,
    operators: &crate::operator_set::OperatorSet,
    h: &Hyp,
    e: &Evidence,
) -> Result<VerifyResult, crate::abstain::AbstainReason> {
    // Step 1: Filter proposer output through constraint validator (propose_and_filter)
    let filter_result = propose_and_filter(proposer, h, e);

    // Step 1a: Check for input-gate rejection (OBL-PS-01 constraint on input hypothesis)
    // propose_and_filter returns this when validate_input fails (e.g., Unstructured atoms in input)
    // Key distinction: filtered_out_count == 0 AND filter_errors non-empty means input-gate failure
    // filtered_out_count == 0 AND filter_errors empty means proposer ran and returned nothing (normal path)
    if filter_result.valid_candidates.is_empty()
        && filter_result.filtered_out_count == 0
        && !filter_result.filter_errors.is_empty()
    {
        // Input hypothesis failed ontology-bounded constraint; this is not an operator-licensing failure
        return Err(crate::abstain::AbstainReason::OntologyOutOfScope(
            "input hypothesis is ontology-unbounded (OBL-PS-01 constraint violation)",
        ));
    }

    // Step 1b: Audit trail for constraint filtering (proposals rejected before operator licensing)
    let constraint_filtering_verdicts: Vec<(String, String)> = filter_result
        .filter_errors
        .iter()
        .map(|err| {
            let msg = if err.ontology_bounded_failed() {
                "ontology-bounded constraint violated"
            } else {
                "operator-reachable constraint violated"
            }
            .to_string();
            ("<filtered by constraint>".to_string(), msg)
        })
        .collect();

    // Step 2: Apply operator set to get soundness-verified result
    let set_outcome = operators.apply_set(h, e);
    let result_atoms = set_outcome.result.atoms();
    let result_atoms_str = format!(
        "{{{}}}",
        result_atoms
            .iter()
            .map(|a| format!("{}:{}@{}", a.system, a.code, a.version))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Step 3: Check each valid candidate for licensing
    let mut licensed_candidates = CandidateSet::new();
    let mut unlicensed_candidates = CandidateSet::new();
    let mut licensing_verdicts = Vec::new();

    for candidate in filter_result.valid_candidates.iter() {
        let candidate_atoms = candidate.atoms();
        let candidate_atoms_str = format!(
            "{{{}}}",
            candidate_atoms
                .iter()
                .map(|a| format!("{}:{}@{}", a.system, a.code, a.version))
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Check if candidate atoms are a subset of result atoms (licensed by operator set)
        // Special case: Hyp::unknown() is licensed only if result is also Hyp::unknown()
        // (identity semantics: empty set ⊆ empty set, but only for "no refinement" case)
        let is_licensed = if candidate_atoms.is_empty() && result_atoms.is_empty() {
            // Both unknown: identity case, licensed only if input was also unknown
            h.atoms().is_empty()
        } else {
            candidate_atoms
                .iter()
                .all(|c_atom| result_atoms.iter().any(|r_atom| r_atom == c_atom))
        };

        if is_licensed {
            licensed_candidates.insert(candidate.clone());
            licensing_verdicts.push((candidate_atoms_str, true, result_atoms_str.clone()));
        } else {
            unlicensed_candidates.insert(candidate.clone());
            licensing_verdicts.push((candidate_atoms_str, false, result_atoms_str.clone()));
        }
    }

    // Step 4: Return result or abstention
    if licensed_candidates.is_empty() {
        Err(crate::abstain::AbstainReason::NoOperatorLicenses(
            "no proposer candidates licensed by operator set",
        ))
    } else {
        Ok(VerifyResult::new(
            licensed_candidates,
            unlicensed_candidates,
            licensing_verdicts,
            constraint_filtering_verdicts,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::OntologySystem;
    use crate::{Atom, ProvenanceOrigin, Ver};
    use chrono::Utc;
    use std::collections::BTreeMap;

    struct TestProposer {
        candidates: Vec<Hyp>,
    }

    impl TestProposer {
        fn new(candidates: Vec<Hyp>) -> Self {
            Self { candidates }
        }

        fn empty() -> Self {
            Self { candidates: vec![] }
        }
    }

    impl RefinementProposer for TestProposer {
        fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
            self.candidates.iter().cloned().collect()
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
        let _proposer: Box<dyn RefinementProposer> = Box::new(TestProposer::empty());
    }

    #[test]
    fn test_proposer_returns_candidate_set() {
        let proposer = TestProposer::empty();
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
        let proposer = TestProposer::empty();
        let candidates = proposer.propose(&h, &e);
        assert_eq!(candidates.len(), 0, "Test proposer returns empty set");
    }

    #[test]
    fn test_proposer_returns_set_not_option() {
        // Verify the type signature: returns Set<Hyp>, not Option<Hyp> or Vec<Hyp>.
        // This enforces the "no decision-making" constraint at the type level.
        let proposer = TestProposer::empty();
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
    fn test_propose_and_filter_accepts_valid_candidates() {
        // Proposer returns valid candidates; all should pass filtering
        let input = Hyp::unknown();
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let valid_candidate = Hyp::new(vec![atom_a]);
        let proposer = TestProposer::new(vec![valid_candidate.clone()]);
        let evidence = Evidence::new(vec![], test_provenance());

        let result = propose_and_filter(&proposer, &input, &evidence);
        assert_eq!(
            result.valid_candidates.len(),
            1,
            "Valid candidate should pass filtering"
        );
        assert_eq!(result.filtered_out_count, 0);
        assert!(result.all_passed(), "No errors should be recorded");
    }

    #[test]
    fn test_propose_and_filter_rejects_invalid_candidates() {
        // Proposer returns invalid candidate (Unstructured atom); should be filtered
        let input = Hyp::unknown();
        let invalid_candidate = Hyp::new(vec![Atom {
            system: OntologySystem::Unstructured,
            code: "bad".to_string(),
            preferred_term: "Invalid".to_string(),
            version: "2026-01-31".to_string(),
        }]);
        let proposer = TestProposer::new(vec![invalid_candidate]);
        let evidence = Evidence::new(vec![], test_provenance());

        let result = propose_and_filter(&proposer, &input, &evidence);
        assert_eq!(
            result.valid_candidates.len(),
            0,
            "Invalid candidate should be filtered"
        );
        assert_eq!(result.filtered_out_count, 1);
        assert!(!result.all_passed(), "Should record filtering error");
        assert_eq!(result.filter_errors.len(), 1);
        assert!(result.filter_errors[0].ontology_bounded_failed());
    }

    #[test]
    fn test_propose_and_filter_mixed_candidates() {
        // Proposer returns mix of valid and invalid; should filter selectively
        let input = Hyp::unknown();
        let valid_atom = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let invalid_atom = Atom {
            system: OntologySystem::Unstructured,
            code: "bad".to_string(),
            preferred_term: "Invalid".to_string(),
            version: "2026-01-31".to_string(),
        };
        let proposer = TestProposer::new(vec![
            Hyp::new(vec![valid_atom]),
            Hyp::new(vec![invalid_atom]),
        ]);
        let evidence = Evidence::new(vec![], test_provenance());

        let result = propose_and_filter(&proposer, &input, &evidence);
        assert_eq!(
            result.valid_candidates.len(),
            1,
            "One valid candidate should pass"
        );
        assert_eq!(
            result.filtered_out_count, 1,
            "One invalid candidate should be filtered"
        );
        assert_eq!(result.filter_errors.len(), 1);
    }

    #[test]
    fn test_propose_and_filter_rejects_invalid_input() {
        // Input hypothesis has Unstructured atom; proposer should not be called
        let invalid_input = Hyp::new(vec![Atom {
            system: OntologySystem::Unstructured,
            code: "bad".to_string(),
            preferred_term: "Invalid".to_string(),
            version: "2026-01-31".to_string(),
        }]);
        let proposer = TestProposer::empty(); // Would not be called
        let evidence = Evidence::new(vec![], test_provenance());

        let result = propose_and_filter(&proposer, &invalid_input, &evidence);
        assert_eq!(result.valid_candidates.len(), 0);
        assert_eq!(
            result.filter_errors.len(),
            1,
            "Input gate should reject invalid hypothesis"
        );
        assert!(result.filter_errors[0].ontology_bounded_failed());
    }

    #[test]
    fn test_filter_result_all_passed() {
        let result = FilterResult::new(vec![Hyp::unknown()].into_iter().collect(), 0, vec![]);
        assert!(result.all_passed());
        assert!(result.has_valid_candidates());
    }

    #[test]
    fn test_verify_result_disjoint_check() {
        // VerifyResult should enforce disjointness of licensed and unlicensed candidates
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let candidate_a = Hyp::new(vec![atom_a]);
        let mut licensed = CandidateSet::new();
        let unlicensed = CandidateSet::new();
        licensed.insert(candidate_a.clone());

        // Create with disjoint sets - should succeed
        let result = VerifyResult::new(licensed.clone(), unlicensed, vec![], vec![]);
        assert_eq!(result.licensed_candidates.len(), 1);
        assert!(result.unlicensed_candidates.is_empty());
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

    // TDD tests for propose_verify (task 8.5)

    use crate::operator::Operator;
    use crate::operator_set::{OperatorMetadata, OperatorSet};
    use crate::outcome::Outcome;

    /// Refining fixture operator: adds a specific atom to any hypothesis
    struct RefiningOperatorFixture {
        atom_to_add: Atom,
    }

    impl Operator for RefiningOperatorFixture {
        fn apply(&self, h: &Hyp, _e: &Evidence) -> Outcome<Hyp, crate::abstain::AbstainReason> {
            let mut atoms = h.atoms().to_vec();
            atoms.push(self.atom_to_add.clone());
            Outcome::Refined(Hyp::new(atoms))
        }
    }

    #[test]
    fn test_propose_verify_licenses_candidates_in_operator_result() {
        // Proposer generates {A}, operator set produces {A, B}.
        // Candidate {A} should be licensed (atoms ⊆ result atoms).
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

        // Proposer returns {A}
        let candidate = Hyp::new(vec![atom_a.clone()]);
        let proposer = TestProposer::new(vec![candidate.clone()]);

        // Operator set adds B to input {A}
        let input = Hyp::new(vec![atom_a.clone()]);
        let mut operators = OperatorSet::new();
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddB".to_string(),
                version: "test".to_string(),
            },
        );

        let evidence = Evidence::new(vec![], test_provenance());
        let result = propose_verify(&proposer, &operators, &input, &evidence);

        assert!(
            result.is_ok(),
            "propose_verify should succeed when candidates are licensed"
        );
        let verify_result = result.unwrap();
        assert!(verify_result.has_licensed_candidates());
        assert_eq!(verify_result.licensed_candidates.len(), 1);
    }

    #[test]
    fn test_propose_verify_rejects_unlicensed_candidates() {
        // Proposer generates {A}, operator produces {B}.
        // Candidate {A} should be unlicensed (atoms ⊄ result atoms).
        // Function should return Err(NoOperatorLicenses).
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

        // Proposer returns {A}
        let candidate = Hyp::new(vec![atom_a.clone()]);
        let proposer = TestProposer::new(vec![candidate]);

        // Operator set replaces input (unknown) with {B}
        let input = Hyp::unknown();
        let mut operators = OperatorSet::new();
        // Create a refining operator that just adds B
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddB".to_string(),
                version: "test".to_string(),
            },
        );

        let evidence = Evidence::new(vec![], test_provenance());
        let result = propose_verify(&proposer, &operators, &input, &evidence);

        assert!(
            result.is_err(),
            "propose_verify should return Err when all candidates are unlicensed"
        );
        match result.unwrap_err() {
            crate::abstain::AbstainReason::NoOperatorLicenses(_) => {
                // Expected
            }
            other => panic!("Expected NoOperatorLicenses, got {:?}", other),
        }
    }

    #[test]
    fn test_propose_verify_all_candidates_licensed() {
        // Proposer generates {A} and {B}, operator produces {A, B}.
        // Both candidates should be licensed.
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

        // Proposer returns {A} and {B}
        let candidate_a = Hyp::new(vec![atom_a.clone()]);
        let candidate_b = Hyp::new(vec![atom_b.clone()]);
        let proposer = TestProposer::new(vec![candidate_a, candidate_b]);

        // Operator adds both A and B to input (unknown)
        let input = Hyp::unknown();
        let mut operators = OperatorSet::new();
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_a.clone(),
            }),
            OperatorMetadata {
                name: "AddA".to_string(),
                version: "test".to_string(),
            },
        );
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddB".to_string(),
                version: "test".to_string(),
            },
        );

        let evidence = Evidence::new(vec![], test_provenance());
        let result = propose_verify(&proposer, &operators, &input, &evidence);

        assert!(result.is_ok());
        let verify_result = result.unwrap();
        assert_eq!(
            verify_result.licensed_candidates.len(),
            2,
            "Both candidates should be licensed"
        );
        assert!(
            verify_result.unlicensed_candidates.is_empty(),
            "No candidates should be unlicensed"
        );
    }

    #[test]
    fn test_propose_verify_truly_mixed_licensed_unlicensed() {
        // Proposer generates {A} and {C}, operator produces {A, B}.
        // {A} is licensed (A ⊆ {A, B}), {C} is unlicensed (C ⊄ {A, B}).
        // Despite mixed outcomes, the function should return Ok (not all unlicensed).
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
        let atom_c = Atom {
            system: OntologySystem::SNOMED,
            code: "2081003".to_string(),
            preferred_term: "Sepsis".to_string(),
            version: "2026-01-31".to_string(),
        };

        // Proposer returns {A} and {C}
        let candidate_a = Hyp::new(vec![atom_a.clone()]);
        let candidate_c = Hyp::new(vec![atom_c.clone()]);
        let proposer = TestProposer::new(vec![candidate_a, candidate_c]);

        // Operator produces {A, B} (adds A and B to input)
        let input = Hyp::unknown();
        let mut operators = OperatorSet::new();
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_a.clone(),
            }),
            OperatorMetadata {
                name: "AddA".to_string(),
                version: "test".to_string(),
            },
        );
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddB".to_string(),
                version: "test".to_string(),
            },
        );

        let evidence = Evidence::new(vec![], test_provenance());
        let result = propose_verify(&proposer, &operators, &input, &evidence);

        // Should return Ok (because A is licensed, even though C is not)
        assert!(result.is_ok());
        let verify_result = result.unwrap();
        assert_eq!(
            verify_result.licensed_candidates.len(),
            1,
            "atom A should be licensed"
        );
        assert_eq!(
            verify_result.unlicensed_candidates.len(),
            1,
            "atom C should be unlicensed"
        );
    }

    #[test]
    fn test_propose_verify_empty_proposer_output() {
        // Proposer returns no candidates. propose_verify should return
        // Err(NoOperatorLicenses) because there are no candidates to license.
        let proposer = TestProposer::empty();
        let input = Hyp::unknown();
        let operators = OperatorSet::new();
        let evidence = Evidence::new(vec![], test_provenance());

        let result = propose_verify(&proposer, &operators, &input, &evidence);

        assert!(
            result.is_err(),
            "Empty proposer output should result in NoOperatorLicenses"
        );
        match result.unwrap_err() {
            crate::abstain::AbstainReason::NoOperatorLicenses(_) => {
                // Expected
            }
            other => panic!("Expected NoOperatorLicenses, got {:?}", other),
        }
    }

    #[test]
    fn test_propose_verify_with_constraint_filtering() {
        // Proposer generates {valid} and {invalid (Unstructured)}.
        // Only {valid} passes propose_and_filter.
        // {valid} is then checked against operator licensing.
        let valid_atom = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let invalid_atom = Atom {
            system: OntologySystem::Unstructured,
            code: "bad".to_string(),
            preferred_term: "Invalid".to_string(),
            version: "2026-01-31".to_string(),
        };

        let valid_candidate = Hyp::new(vec![valid_atom.clone()]);
        let invalid_candidate = Hyp::new(vec![invalid_atom]);
        let proposer = TestProposer::new(vec![valid_candidate.clone(), invalid_candidate]);

        // Operator refines to {valid}
        let input = Hyp::unknown();
        let mut operators = OperatorSet::new();
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: valid_atom.clone(),
            }),
            OperatorMetadata {
                name: "AddValid".to_string(),
                version: "test".to_string(),
            },
        );

        let evidence = Evidence::new(vec![], test_provenance());
        let result = propose_verify(&proposer, &operators, &input, &evidence);

        // Should succeed because the valid candidate is licensed
        assert!(result.is_ok());
        let verify_result = result.unwrap();
        // Only the valid candidate should be in licensed_candidates
        // (invalid was filtered by propose_and_filter)
        assert_eq!(verify_result.licensed_candidates.len(), 1);
    }

    #[test]
    fn test_propose_verify_audit_trail() {
        // Verify that licensing_verdicts audit trail is populated correctly.
        // When some candidates are licensed, verify that audit trail records all verdicts.
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

        // Proposer returns {A} and {B}
        let candidate_a = Hyp::new(vec![atom_a.clone()]);
        let candidate_b = Hyp::new(vec![atom_b.clone()]);
        let proposer = TestProposer::new(vec![candidate_a, candidate_b]);

        // Operator produces {A, B}: both atoms
        let input = Hyp::unknown();
        let mut operators = OperatorSet::new();
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_a.clone(),
            }),
            OperatorMetadata {
                name: "AddA".to_string(),
                version: "test".to_string(),
            },
        );
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddB".to_string(),
                version: "test".to_string(),
            },
        );

        let evidence = Evidence::new(vec![], test_provenance());
        let result = propose_verify(&proposer, &operators, &input, &evidence);

        // Should succeed with both candidates licensed
        assert!(result.is_ok());
        let verify_result = result.unwrap();
        // Audit trail should have verdicts for both candidates
        assert_eq!(verify_result.licensing_verdicts.len(), 2);
        // Both should be licensed (true in verdicts)
        assert!(
            verify_result
                .licensing_verdicts
                .iter()
                .all(|(_, is_licensed, _)| *is_licensed),
            "Audit trail should show both candidates as licensed"
        );
    }

    // INV-PS-06 Structural Enforcement Test (Task 8.6)
    // Property-test tier: ≥10 cases demonstrating that no out-of-bounds candidate
    // can escape the constraint and licensing filters, regardless of proposer behavior.

    /// Adversarial proposer that returns Unstructured atoms (OBL-PS-01 violation)
    struct UnstructuredProposer;
    impl RefinementProposer for UnstructuredProposer {
        fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
            let mut candidates = CandidateSet::new();
            candidates.insert(Hyp::new(vec![Atom {
                system: OntologySystem::Unstructured,
                code: "bad".to_string(),
                preferred_term: "Invalid".to_string(),
                version: "2026-01-31".to_string(),
            }]));
            candidates
        }
    }

    /// Adversarial proposer that returns empty codes (codec validation)
    struct EmptyCodeProposer;
    impl RefinementProposer for EmptyCodeProposer {
        fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
            let mut candidates = CandidateSet::new();
            candidates.insert(Hyp::new(vec![Atom {
                system: OntologySystem::SNOMED,
                code: "".to_string(),
                preferred_term: "Empty Code".to_string(),
                version: "2026-01-31".to_string(),
            }]));
            candidates
        }
    }

    /// Adversarial proposer that returns mixed valid+invalid atoms
    struct MixedProposer;
    impl RefinementProposer for MixedProposer {
        fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
            let mut candidates = CandidateSet::new();
            candidates.insert(Hyp::new(vec![
                Atom {
                    system: OntologySystem::SNOMED,
                    code: "67822003".to_string(),
                    preferred_term: "Hypoxemia".to_string(),
                    version: "2026-01-31".to_string(),
                },
                Atom {
                    system: OntologySystem::Unstructured,
                    code: "bad".to_string(),
                    preferred_term: "Invalid".to_string(),
                    version: "2026-01-31".to_string(),
                },
            ]));
            candidates
        }
    }

    /// Adversarial proposer that returns purely unstructured candidates
    struct PurelyUnstructuredProposer;
    impl RefinementProposer for PurelyUnstructuredProposer {
        fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
            let mut candidates = CandidateSet::new();
            // Multiple unstructured candidates
            for i in 0..3 {
                candidates.insert(Hyp::new(vec![Atom {
                    system: OntologySystem::Unstructured,
                    code: format!("bad{}", i),
                    preferred_term: format!("Invalid {}", i),
                    version: "2026-01-31".to_string(),
                }]));
            }
            candidates
        }
    }

    /// Adversarial proposer that returns non-refining candidates (violates DEF-PS-15 clause 2)
    struct NonRefiningProposer;
    impl RefinementProposer for NonRefiningProposer {
        fn propose(&self, h: &Hyp, _e: &Evidence) -> CandidateSet {
            let mut candidates = CandidateSet::new();
            // Return unknown (fewer atoms than input if input has any)
            if !h.atoms().is_empty() {
                candidates.insert(Hyp::unknown());
            }
            candidates
        }
    }

    #[test]
    fn test_inv_ps_06_unstructured_atoms_filtered() {
        // Property 1: Unstructured atoms must be filtered by constraint validation
        let proposer = UnstructuredProposer;
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let filter_result = propose_and_filter(&proposer, &input, &evidence);
        assert_eq!(
            filter_result.valid_candidates.len(),
            0,
            "Unstructured atoms must be rejected at constraint stage"
        );
        assert!(!filter_result.all_passed(), "Filter should record errors");
    }

    #[test]
    fn test_inv_ps_06_empty_codes_filtered() {
        // Property 2: Empty codes must be filtered by constraint validation
        let proposer = EmptyCodeProposer;
        let input = Hyp::unknown();
        let _operators = OperatorSet::new();
        let evidence = Evidence::new(vec![], test_provenance());

        let filter_result = propose_and_filter(&proposer, &input, &evidence);
        assert_eq!(
            filter_result.valid_candidates.len(),
            0,
            "Empty codes must be rejected at constraint stage"
        );
    }

    #[test]
    fn test_inv_ps_06_mixed_valid_invalid_filtered() {
        // Property 3: Mixed atom sets with invalid atoms must be fully filtered
        let proposer = MixedProposer;
        let input = Hyp::unknown();
        let _operators = OperatorSet::new();
        let evidence = Evidence::new(vec![], test_provenance());

        let filter_result = propose_and_filter(&proposer, &input, &evidence);
        assert_eq!(
            filter_result.valid_candidates.len(),
            0,
            "Candidates with any Unstructured atoms must be rejected"
        );
    }

    #[test]
    fn test_inv_ps_06_purely_unstructured_proposer_blocked() {
        // Property 4: Even proposer returning only unstructured atoms is blocked
        let proposer = PurelyUnstructuredProposer;
        let input = Hyp::unknown();
        let _operators = OperatorSet::new();
        let evidence = Evidence::new(vec![], test_provenance());

        let filter_result = propose_and_filter(&proposer, &input, &evidence);
        assert!(
            filter_result.valid_candidates.is_empty(),
            "All unstructured candidates must be filtered"
        );
        assert_eq!(
            filter_result.filtered_out_count, 3,
            "All 3 unstructured candidates rejected"
        );
    }

    #[test]
    fn test_inv_ps_06_non_refining_candidates_filtered() {
        // Property 5: Non-refining candidates fail the operator-reachable check
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let input = Hyp::new(vec![atom_a]);
        let proposer = NonRefiningProposer;
        let _operators = OperatorSet::new();
        let evidence = Evidence::new(vec![], test_provenance());

        let filter_result = propose_and_filter(&proposer, &input, &evidence);
        assert_eq!(
            filter_result.valid_candidates.len(),
            0,
            "Non-refining candidates (Hyp::unknown() when input has atoms) must be filtered"
        );
    }

    #[test]
    fn test_inv_ps_06_empty_proposer_output_safe() {
        // Property 6: Empty proposer output (no candidates) is safe (no unsound candidates escape)
        let proposer = TestProposer::empty();
        let input = Hyp::unknown();
        let _operators = OperatorSet::new();
        let evidence = Evidence::new(vec![], test_provenance());

        let filter_result = propose_and_filter(&proposer, &input, &evidence);
        assert!(
            filter_result.valid_candidates.is_empty(),
            "Empty proposer output means no candidates to filter — structural safety holds"
        );
    }

    #[test]
    fn test_inv_ps_06_propose_verify_rejects_all_unlicensed() {
        // Property 7: When proposer returns valid but unlicensed candidates,
        // propose_verify must reject them (no refinement by operator set).
        // This tests the operator-licensing gate end-to-end.

        struct ValidButUnlicensedProposer;
        impl RefinementProposer for ValidButUnlicensedProposer {
            fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
                let mut candidates = CandidateSet::new();
                // Valid atom (SNOMED, non-empty), but won't be produced by operators
                candidates.insert(Hyp::new(vec![Atom {
                    system: OntologySystem::SNOMED,
                    code: "999999".to_string(),
                    preferred_term: "WillNotBeProduced".to_string(),
                    version: "2026-01-31".to_string(),
                }]));
                candidates
            }
        }

        let proposer = ValidButUnlicensedProposer;
        let input = Hyp::unknown();
        let operators = OperatorSet::new();
        let evidence = Evidence::new(vec![], test_provenance());

        // propose_verify should reject the candidate because it's not licensed
        let result = propose_verify(&proposer, &operators, &input, &evidence);
        assert!(
            result.is_err(),
            "Valid but unlicensed candidates should be rejected by licensing gate"
        );
        match result.unwrap_err() {
            crate::abstain::AbstainReason::NoOperatorLicenses(_) => {
                // Expected: licensing gate rejected the unlicensed candidate
            }
            other => panic!("Expected NoOperatorLicenses, got {:?}", other),
        }
    }

    #[test]
    fn test_inv_ps_06_full_pipeline_structural_property() {
        // Property 8: Full pipeline (propose_and_filter → propose_verify) guarantees
        // only ontology-bounded, operator-licensed hypotheses emerge.
        // This is the comprehensive structural test covering the end-to-end contract.

        struct AdversarialProposer;
        impl RefinementProposer for AdversarialProposer {
            fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
                let mut candidates = CandidateSet::new();
                // Try various violations
                candidates.insert(Hyp::new(vec![Atom {
                    system: OntologySystem::Unstructured,
                    code: "free_text".to_string(),
                    preferred_term: "Should fail".to_string(),
                    version: "2026-01-31".to_string(),
                }]));
                candidates.insert(Hyp::new(vec![Atom {
                    system: OntologySystem::SNOMED,
                    code: "".to_string(),
                    preferred_term: "Empty code".to_string(),
                    version: "2026-01-31".to_string(),
                }]));
                candidates
            }
        }

        let proposer = AdversarialProposer;
        let input = Hyp::unknown();
        let operators = OperatorSet::new();
        let evidence = Evidence::new(vec![], test_provenance());

        // Step 1: Constraint filtering (propose_and_filter)
        let filter_result = propose_and_filter(&proposer, &input, &evidence);
        assert!(
            filter_result.valid_candidates.is_empty(),
            "All adversarial candidates should be filtered by constraint validation"
        );

        // Step 2: Licensing verification (propose_verify) — empty valid_candidates triggers abstention
        let verify_result = propose_verify(&proposer, &operators, &input, &evidence);
        assert!(
            verify_result.is_err(),
            "propose_verify should abstain when all candidates are filtered"
        );
        // Verify the correct error type: NoOperatorLicenses (not OntologyOutOfScope)
        match verify_result.unwrap_err() {
            crate::abstain::AbstainReason::NoOperatorLicenses(_) => {
                // Expected: after filtering, no candidates left to license
            }
            other => panic!("Expected NoOperatorLicenses, got {:?}", other),
        }
    }

    #[test]
    fn test_inv_ps_06_input_gate_blocks_invalid_input() {
        // Property 9: Input-side gate (propose_and_filter's validate_input) blocks
        // invalid input hypotheses, preventing the proposer from running on bad state.

        let invalid_input = Hyp::new(vec![Atom {
            system: OntologySystem::Unstructured,
            code: "bad_input".to_string(),
            preferred_term: "Invalid input".to_string(),
            version: "2026-01-31".to_string(),
        }]);

        let proposer = TestProposer::empty();
        let evidence = Evidence::new(vec![], test_provenance());

        let filter_result = propose_and_filter(&proposer, &invalid_input, &evidence);
        assert!(
            filter_result.valid_candidates.is_empty(),
            "Invalid input hypothesis should be rejected at input-side gate"
        );
        assert_eq!(
            filter_result.filter_errors.len(),
            1,
            "Input-side validation failure recorded"
        );
    }

    #[test]
    fn test_inv_ps_06_ontology_bounded_subset_safety() {
        // Property 10: Even when proposer returns a valid ontology-bounded subset,
        // if not licensed by operators, it's filtered at the licensing stage.

        struct SubsetProposer {
            atom: Atom,
        }
        impl RefinementProposer for SubsetProposer {
            fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
                let mut candidates = CandidateSet::new();
                candidates.insert(Hyp::new(vec![self.atom.clone()]));
                candidates
            }
        }

        let atom = Atom {
            system: OntologySystem::RxNorm,
            code: "99999".to_string(),
            preferred_term: "NeverProduced".to_string(),
            version: "2026-01-31".to_string(),
        };

        let proposer = SubsetProposer { atom: atom.clone() };
        let input = Hyp::unknown();
        let mut operators = OperatorSet::new();
        // Add operator that produces a DIFFERENT atom
        operators = operators.register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: Atom {
                    system: OntologySystem::SNOMED,
                    code: "67822003".to_string(),
                    preferred_term: "Hypoxemia".to_string(),
                    version: "2026-01-31".to_string(),
                },
            }),
            OperatorMetadata {
                name: "AddHypoxemia".to_string(),
                version: "test".to_string(),
            },
        );

        let evidence = Evidence::new(vec![], test_provenance());

        // Constraint filtering passes (RxNorm is valid)
        let filter_result = propose_and_filter(&proposer, &input, &evidence);
        assert_eq!(
            filter_result.valid_candidates.len(),
            1,
            "Valid RxNorm candidate passes constraint filtering"
        );

        // But licensing gate rejects it (not produced by operators)
        let verify_result = propose_verify(&proposer, &operators, &input, &evidence);
        assert!(
            verify_result.is_err(),
            "Valid but unlicensed RxNorm candidate must be rejected by licensing gate"
        );
        match verify_result.unwrap_err() {
            crate::abstain::AbstainReason::NoOperatorLicenses(_) => {
                // Expected: structurally demonstrates two-layer filtering
            }
            other => panic!("Expected NoOperatorLicenses, got {:?}", other),
        }
    }

    // ========== Substrate-Invariance Tests (Task 11.2) ==========
    // Property: substrate refinement is independent of proposer architecture.
    // Feed identical evidence through two different proposers; assert identical
    // post-soundness-gate outcomes.

    use crate::lattice_search::LatticeSearchProposer;
    use crate::llm_proposer::LlmProposer;
    use crate::llm_proposer_config::LlmProposerConfig;

    fn si_atom_hypoxemia() -> Atom {
        Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        }
    }

    fn si_atom_ards() -> Atom {
        Atom {
            system: OntologySystem::SNOMED,
            code: "3723001".to_string(),
            preferred_term: "ARDS".to_string(),
            version: "2026-01-31".to_string(),
        }
    }

    #[test]
    fn test_substrate_invariance_paired_proposers_simple_case() {
        // Property 1: Two proposers on same input → identical licensed candidates
        // Setup: simple hypothesis with one atom, one operator
        let atom_a = si_atom_hypoxemia();
        let atom_b = si_atom_ards();

        let input = Hyp::new(vec![atom_a.clone()]);
        let evidence = Evidence::new(vec![], test_provenance());

        // Build operator set for LatticeSearch (for constructing proposer)
        let operators_for_proposer = OperatorSet::new().register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddARDS".to_string(),
                version: "test".to_string(),
            },
        );
        let lattice_proposer = LatticeSearchProposer::new(operators_for_proposer);

        // Build shared operator set for soundness verification (used by both proposers)
        let operators_verify = OperatorSet::new().register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddARDS".to_string(),
                version: "test".to_string(),
            },
        );

        // Proposer 2: LlmProposer (mock)
        let expected_candidate_str = format!(
            "SNOMED:{}@2026-01-31, SNOMED:{}@2026-01-31",
            atom_a.code, atom_b.code
        );
        let config = LlmProposerConfig::mock("test-model", vec![expected_candidate_str], "0.1.0");
        let llm_proposer = LlmProposer::new(config);

        // Run both through propose_verify with the same operator set
        let result_lattice =
            propose_verify(&lattice_proposer, &operators_verify, &input, &evidence);
        let result_llm = propose_verify(&llm_proposer, &operators_verify, &input, &evidence);

        assert!(
            result_lattice.is_ok(),
            "LatticeSearchProposer should succeed"
        );
        assert!(result_llm.is_ok(), "LlmProposer should succeed");

        let verify_lattice = result_lattice.unwrap();
        let verify_llm = result_llm.unwrap();

        assert_eq!(
            verify_lattice.licensed_candidates, verify_llm.licensed_candidates,
            "Substrate-invariance: identical input → identical licensed candidates"
        );
    }

    #[test]
    fn test_substrate_invariance_both_hallucinate_differently() {
        // Property 2: LLM can hallucinate (invalid atoms in response);
        // invalid atoms are filtered by parse_response; post-gate outcomes identical
        let atom_a = si_atom_hypoxemia();
        let atom_b = si_atom_ards();

        let input = Hyp::new(vec![atom_a.clone()]);
        let evidence = Evidence::new(vec![], test_provenance());

        // Proposer 1: LatticeSearchProposer (produces only reachable candidates)
        let operators_for_proposer = OperatorSet::new().register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddARDS".to_string(),
                version: "test".to_string(),
            },
        );
        let lattice_proposer = LatticeSearchProposer::new(operators_for_proposer);

        // Proposer 2: LlmProposer (mock returns valid candidate on line 1, hallucinated invalid candidate on line 2)
        // Line 1: valid candidate with two atoms
        // Line 2: hallucinated candidate with malformed atom (no colon, won't parse)
        let response_with_hallucination = format!(
            "SNOMED:{}@2026-01-31, SNOMED:{}@2026-01-31\nMALFORMED_GARBAGE",
            atom_a.code, atom_b.code
        );
        let config =
            LlmProposerConfig::mock("test-model", vec![response_with_hallucination], "0.1.0");
        let llm_proposer = LlmProposer::new(config);

        // Build shared operator set for licensing verification
        let operators_verify = OperatorSet::new().register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddARDS".to_string(),
                version: "test".to_string(),
            },
        );

        let result_lattice =
            propose_verify(&lattice_proposer, &operators_verify, &input, &evidence);
        let result_llm = propose_verify(&llm_proposer, &operators_verify, &input, &evidence);

        assert!(result_lattice.is_ok());
        assert!(result_llm.is_ok());

        let verify_lattice = result_lattice.unwrap();
        let verify_llm = result_llm.unwrap();

        assert_eq!(
            verify_lattice.licensed_candidates, verify_llm.licensed_candidates,
            "Even with LLM hallucinations, licensed candidates are identical (hallucinations filtered)"
        );
    }

    #[test]
    fn test_substrate_invariance_multiple_paired_cases() {
        // Property 3: Substrate invariance holds across multiple cases (≥10 pairs)
        // Each iteration uses a different refinement atom but identical substrate behavior
        let atom_base = si_atom_hypoxemia();

        // Generate 10 atoms for paired testing (6+ digit codes per SNOMED CT)
        let atoms_to_test: Vec<Atom> = (0..10)
            .map(|i| Atom {
                system: OntologySystem::SNOMED,
                code: format!("{:06}", 100000 + i),
                preferred_term: format!("TestAtom{}", i),
                version: "2026-01-31".to_string(),
            })
            .collect();

        let input = Hyp::new(vec![atom_base.clone()]);
        let evidence = Evidence::new(vec![], test_provenance());

        for (i, atom) in atoms_to_test.iter().enumerate() {
            let op_name = format!("OpAdd{}", i);

            // Build operator set for proposer
            let operators_for_proposer = OperatorSet::new().register(
                Box::new(RefiningOperatorFixture {
                    atom_to_add: atom.clone(),
                }),
                OperatorMetadata {
                    name: op_name.clone(),
                    version: "test".to_string(),
                },
            );

            let lattice_proposer = LatticeSearchProposer::new(operators_for_proposer);

            // Mock LLM with the same expected response
            let expected_response = format!(
                "SNOMED:{}@2026-01-31, SNOMED:{}@2026-01-31",
                atom_base.code, atom.code
            );
            let config = LlmProposerConfig::mock("test-model", vec![expected_response], "0.1.0");
            let llm_proposer = LlmProposer::new(config);

            // Build shared operator set for licensing verification
            let operators_verify = OperatorSet::new().register(
                Box::new(RefiningOperatorFixture {
                    atom_to_add: atom.clone(),
                }),
                OperatorMetadata {
                    name: op_name.clone(),
                    version: "test".to_string(),
                },
            );

            let result_lattice =
                propose_verify(&lattice_proposer, &operators_verify, &input, &evidence);
            let result_llm = propose_verify(&llm_proposer, &operators_verify, &input, &evidence);

            assert!(
                result_lattice.is_ok(),
                "Case {}: LatticeSearch should succeed",
                i
            );
            assert!(result_llm.is_ok(), "Case {}: LlmProposer should succeed", i);

            let verify_lattice = result_lattice.unwrap();
            let verify_llm = result_llm.unwrap();

            assert_eq!(
                verify_lattice.licensed_candidates, verify_llm.licensed_candidates,
                "Case {}: licensed candidates must match",
                i
            );
        }
    }

    #[test]
    fn test_substrate_invariance_both_abstain_no_candidates() {
        // Property 4: When both proposers produce no candidates, both abstain identically
        let atom_a = si_atom_hypoxemia();

        let input = Hyp::new(vec![atom_a]);
        let evidence = Evidence::new(vec![], test_provenance());

        // Operator set for proposer (empty → no candidates possible)
        let operators_for_proposer = OperatorSet::new();

        // Proposer 1: LatticeSearch on empty operator set → empty output
        let lattice_proposer = LatticeSearchProposer::new(operators_for_proposer);

        // Proposer 2: LLM mock with empty response → empty output
        let config = LlmProposerConfig::mock(
            "test-model",
            vec!["".to_string()], // Empty response (no candidates)
            "0.1.0",
        );
        let llm_proposer = LlmProposer::new(config);

        // Shared empty operator set for verification
        let operators_verify = OperatorSet::new();
        let result_lattice =
            propose_verify(&lattice_proposer, &operators_verify, &input, &evidence);
        let result_llm = propose_verify(&llm_proposer, &operators_verify, &input, &evidence);

        // Both should abstain (Err with NoOperatorLicenses)
        assert!(
            result_lattice.is_err(),
            "LatticeSearch on empty operators should abstain"
        );
        assert!(
            result_llm.is_err(),
            "LlmProposer on empty operators should abstain"
        );

        match (result_lattice.unwrap_err(), result_llm.unwrap_err()) {
            (
                crate::abstain::AbstainReason::NoOperatorLicenses(_),
                crate::abstain::AbstainReason::NoOperatorLicenses(_),
            ) => {
                // Expected: both abstain for the same reason
            }
            other => panic!(
                "Expected both to abstain with NoOperatorLicenses, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_substrate_invariance_llm_mixed_valid_invalid_responses() {
        // Property 5: LLM parsing returns mix of valid and invalid atoms in response;
        // invalid atoms filtered by parse_response; post-gate outcomes identical to deterministic
        let atom_a = si_atom_hypoxemia();
        let atom_b = si_atom_ards();

        let input = Hyp::new(vec![atom_a.clone()]);
        let evidence = Evidence::new(vec![], test_provenance());

        // Operator set for proposer
        let operators_for_proposer = OperatorSet::new().register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddARDS".to_string(),
                version: "test".to_string(),
            },
        );

        let lattice_proposer = LatticeSearchProposer::new(operators_for_proposer);

        // LLM mock with valid atoms AND unstructured garbage in same response
        // The garbage will be silently filtered by parse_response
        let response_mixed = format!(
            "SNOMED:{}@2026-01-31, SNOMED:{}@2026-01-31, Unstructured free text",
            atom_a.code, atom_b.code
        );

        let config = LlmProposerConfig::mock("test-model", vec![response_mixed], "0.1.0");
        let llm_proposer = LlmProposer::new(config);

        // Build shared operator set for licensing verification
        let operators_verify = OperatorSet::new().register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddARDS".to_string(),
                version: "test".to_string(),
            },
        );

        let result_lattice =
            propose_verify(&lattice_proposer, &operators_verify, &input, &evidence);
        let result_llm = propose_verify(&llm_proposer, &operators_verify, &input, &evidence);

        assert!(result_lattice.is_ok());
        assert!(result_llm.is_ok());

        let verify_lattice = result_lattice.unwrap();
        let verify_llm = result_llm.unwrap();

        // Both should have identical licensed candidates
        // (LLM's invalid response is filtered, leaving only the valid one)
        assert_eq!(
            verify_lattice.licensed_candidates, verify_llm.licensed_candidates,
            "Invalid LLM responses are filtered; licensed candidates identical"
        );
    }

    #[test]
    fn test_substrate_invariance_operator_licensing_decides_outcome() {
        // Property 6: Licensing gate decides outcome uniformly across proposers
        // Both proposers return valid-but-unlicensed candidates → both abstain identically
        let atom_a = si_atom_hypoxemia();
        let atom_b = si_atom_ards();
        let atom_unlicensed = Atom {
            system: OntologySystem::SNOMED,
            code: "999999".to_string(),
            preferred_term: "WontBeLicensed".to_string(),
            version: "2026-01-31".to_string(),
        };

        let input = Hyp::new(vec![atom_a.clone()]);
        let evidence = Evidence::new(vec![], test_provenance());

        // Build shared operator set for verification (only licenses atom_b, not atom_unlicensed)
        let operators_verify = OperatorSet::new().register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddARDS".to_string(),
                version: "test".to_string(),
            },
        );

        // Proposer 1: Returns unlicensed candidate (valid parse, but not in operator output)
        struct UnlicensedProposer {
            atom: Atom,
        }
        impl RefinementProposer for UnlicensedProposer {
            fn propose(&self, _h: &Hyp, _e: &Evidence) -> CandidateSet {
                let mut set = CandidateSet::new();
                set.insert(Hyp::new(vec![self.atom.clone()]));
                set
            }
        }

        let unlicensed_proposer = UnlicensedProposer {
            atom: atom_unlicensed.clone(),
        };

        // Proposer 2: LLM mock with same unlicensed candidate
        let unlicensed_response = format!("SNOMED:{}@2026-01-31", atom_unlicensed.code);
        let config = LlmProposerConfig::mock("test-model", vec![unlicensed_response], "0.1.0");
        let llm_proposer = LlmProposer::new(config);

        let result_unlicensed =
            propose_verify(&unlicensed_proposer, &operators_verify, &input, &evidence);
        let result_llm = propose_verify(&llm_proposer, &operators_verify, &input, &evidence);

        // Both should abstain (neither has licensed candidates)
        assert!(result_unlicensed.is_err(), "Unlicensed candidate → abstain");
        assert!(result_llm.is_err(), "LLM unlicensed candidate → abstain");

        // Both abstain for the same reason: NoOperatorLicenses
        match (result_unlicensed.unwrap_err(), result_llm.unwrap_err()) {
            (
                crate::abstain::AbstainReason::NoOperatorLicenses(_),
                crate::abstain::AbstainReason::NoOperatorLicenses(_),
            ) => {
                // Expected: identical licensing behavior
            }
            other => panic!(
                "Expected both to abstain with NoOperatorLicenses, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_substrate_invariance_audit_trails_both_populated() {
        // Property 7: Both proposers populate audit trails identically (content equality, not just count)
        let atom_a = si_atom_hypoxemia();
        let atom_b = si_atom_ards();

        let input = Hyp::new(vec![atom_a.clone()]);
        let evidence = Evidence::new(vec![], test_provenance());

        let operators_for_proposer = OperatorSet::new().register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddARDS".to_string(),
                version: "test".to_string(),
            },
        );

        let lattice_proposer = LatticeSearchProposer::new(operators_for_proposer);

        let response = format!(
            "SNOMED:{}@2026-01-31, SNOMED:{}@2026-01-31",
            atom_a.code, atom_b.code
        );
        let config = LlmProposerConfig::mock("test-model", vec![response], "0.1.0");
        let llm_proposer = LlmProposer::new(config);

        // Build shared operator set for licensing verification
        let operators_verify = OperatorSet::new().register(
            Box::new(RefiningOperatorFixture {
                atom_to_add: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "AddARDS".to_string(),
                version: "test".to_string(),
            },
        );

        let result_lattice =
            propose_verify(&lattice_proposer, &operators_verify, &input, &evidence);
        let result_llm = propose_verify(&llm_proposer, &operators_verify, &input, &evidence);

        assert!(result_lattice.is_ok());
        assert!(result_llm.is_ok());

        let verify_lattice = result_lattice.unwrap();
        let verify_llm = result_llm.unwrap();

        // Both should have populated licensing_verdicts
        assert!(
            !verify_lattice.licensing_verdicts.is_empty(),
            "LatticeSearch should populate licensing verdicts"
        );
        assert!(
            !verify_llm.licensing_verdicts.is_empty(),
            "LlmProposer should populate licensing verdicts"
        );

        // Both should have the same number of verdicts and identical content (same candidates licensed)
        // The identical licensed_candidates assertion in Property 1 ensures the refinements match;
        // this assertion confirms the audit trail records the same decisions.
        assert_eq!(
            verify_lattice.licensing_verdicts.len(),
            verify_llm.licensing_verdicts.len(),
            "Audit trail verdict counts must match"
        );

        // Verify the audit trails describe identical licensing outcomes:
        // both should have recorded the same candidate being licensed by the same operator
        assert_eq!(
            verify_lattice.licensed_candidates, verify_llm.licensed_candidates,
            "Audit trails must describe identical refined hypotheses (same licensed candidates)"
        );
    }
}
