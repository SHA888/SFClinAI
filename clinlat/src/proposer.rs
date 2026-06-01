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
use crate::operator::Evidence;
use std::collections::HashSet;

/// A set of candidate hypotheses.
///
/// This is a finite set returned by a refinement proposer. Empty sets are valid
/// (indicate the proposer has no candidates). Non-empty sets are not pre-filtered
/// by the proposer; filtering is the substrate's responsibility (DEF-PS-15).
pub type CandidateSet = HashSet<Hyp>;

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
pub trait RefinementProposer {
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
}
