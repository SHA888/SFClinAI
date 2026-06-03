//! Exhaustive lattice-search proposer.
//!
//! Implements deterministic, breadth-first search over all hypotheses reachable
//! from the input via single operator application. Trivially sound by construction:
//! every candidate is provably reachable by exactly one operator (DEF-PS-15).
//!
//! Implements the reference proposer for Phase 9 (M2.4).

use crate::hyp::Hyp;
use crate::operator::Evidence;
use crate::operator_set::OperatorSet;
use crate::outcome::Outcome;
use crate::proposer::{CandidateSet, RefinementProposer};

/// Exhaustive lattice-search proposer.
///
/// Searches all hypotheses reachable from the input hypothesis via a single
/// application of each operator in the operator set. Returns the union of all
/// refinements (or abstentions are skipped).
///
/// **Soundness**: Every candidate in the output set has a provenance: it was
/// produced by applying a specific operator to the input. When candidates are
/// later passed through `propose_verify` (soundness gate), they will be licensed
/// because `OperatorSet.apply_set()` will recognize them as reachable.
///
/// **Completeness**: For a fixed operator set, the output is complete: it contains
/// all hypotheses reachable by one operator application. Corollary: for small
/// operator sets (≤5), the set is exhaustive and typically small.
///
/// **Pruning**: For large operator sets, candidates may exceed a threshold.
/// Implement pruning (e.g., candidate-count cap or depth limit) to avoid
/// memory exhaustion.
pub struct LatticeSearchProposer {
    /// Operator set to search over.
    operators: OperatorSet,
    /// Maximum number of candidates to return. If exceeded, truncate to this limit.
    /// Defaults to no limit (None). Set to Some(N) for bounded proposers.
    max_candidates: Option<usize>,
}

impl LatticeSearchProposer {
    /// Creates a new lattice-search proposer with the given operator set.
    pub fn new(operators: OperatorSet) -> Self {
        LatticeSearchProposer {
            operators,
            max_candidates: None,
        }
    }

    /// Creates a new lattice-search proposer with the given operator set and
    /// candidate count limit.
    pub fn with_limit(operators: OperatorSet, max_candidates: usize) -> Self {
        LatticeSearchProposer {
            operators,
            max_candidates: Some(max_candidates),
        }
    }
}

impl RefinementProposer for LatticeSearchProposer {
    fn propose(&self, h: &Hyp, e: &Evidence) -> CandidateSet {
        let mut candidates = CandidateSet::new();

        for op in self.operators.iter_operators() {
            match op.apply(h, e) {
                Outcome::Refined(h_prime) => {
                    candidates.insert(h_prime);
                }
                Outcome::Abstain(_) => {
                    // Skip abstentions; proposer only collects refinements
                }
            }
        }

        // Apply pruning if max_candidates is set
        if let Some(max) = self.max_candidates {
            if candidates.len() > max {
                // Truncate to max_candidates. Note: HashSet iteration order is undefined,
                // so the actual subset returned may vary across runs. This is acceptable
                // for proposer output (downstream soundness verification is order-independent).
                candidates = candidates.into_iter().take(max).collect();
            }
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abstain::AbstainReason;
    use crate::operator::Operator;
    use crate::operator_set::OperatorMetadata;
    use crate::outcome::Outcome;
    use crate::provenance::{Provenance, ProvenanceOrigin};
    use crate::version::Ver;
    use crate::{Atom, OntologySystem};
    use chrono::Utc;
    use std::collections::BTreeMap;

    fn test_provenance() -> Provenance {
        Provenance::new(
            ProvenanceOrigin::new("test", "test", "test"),
            Utc::now(),
            Ver::new("test", "test", "0.1.0"),
            BTreeMap::new(),
        )
    }

    // Fixture operators for property testing

    /// Adds a fixed atom to any hypothesis (simple refinement).
    struct AddAtomOperator {
        atom: Atom,
    }
    impl Operator for AddAtomOperator {
        fn apply(&self, h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
            let mut atoms = h.atoms().to_vec();
            atoms.push(self.atom.clone());
            Outcome::Refined(Hyp::new(atoms))
        }
    }

    /// Always abstains (produces no candidates).
    struct AlwaysAbstainOperator;
    impl Operator for AlwaysAbstainOperator {
        fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
            Outcome::Abstain(AbstainReason::InsufficientEvidence("fixture abstain"))
        }
    }

    /// Conditionally refines based on input: if input is unknown, add atom; else abstain.
    struct ConditionalOperator {
        atom: Atom,
    }
    impl Operator for ConditionalOperator {
        fn apply(&self, h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
            if h.atoms().is_empty() {
                let atoms = vec![self.atom.clone()];
                Outcome::Refined(Hyp::new(atoms))
            } else {
                Outcome::Abstain(AbstainReason::OperatorPreconditionUnmet(
                    "input not unknown",
                ))
            }
        }
    }

    #[test]
    fn test_lattice_search_single_operator_refines() {
        // Property 1: Single operator that refines returns the refinement.
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator {
                atom: atom_a.clone(),
            }),
            OperatorMetadata {
                name: "AddHypoxemia".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            1,
            "Single refining operator should produce 1 candidate"
        );
        let expected = Hyp::new(vec![atom_a]);
        assert!(
            candidates.contains(&expected),
            "Candidate should be the refined hypothesis"
        );
    }

    #[test]
    fn test_lattice_search_single_operator_abstains() {
        // Property 2: Single operator that abstains returns empty set.
        let operators = OperatorSet::new().register(
            Box::new(AlwaysAbstainOperator),
            OperatorMetadata {
                name: "AlwaysAbstain".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(
            candidates.is_empty(),
            "Abstaining operator should produce no candidates"
        );
    }

    #[test]
    fn test_lattice_search_multiple_operators_union() {
        // Property 3: Multiple operators produce the union of their refinements.
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom_b = Atom {
            system: OntologySystem::LOINC,
            code: "2019-8".to_string(),
            preferred_term: "CO2 level".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new()
            .register(
                Box::new(AddAtomOperator {
                    atom: atom_a.clone(),
                }),
                OperatorMetadata {
                    name: "Op1".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AddAtomOperator {
                    atom: atom_b.clone(),
                }),
                OperatorMetadata {
                    name: "Op2".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            2,
            "Two refining operators should produce 2 candidates"
        );
        let expected_a = Hyp::new(vec![atom_a]);
        let expected_b = Hyp::new(vec![atom_b]);
        assert!(
            candidates.contains(&expected_a),
            "Should contain refinement from Op1"
        );
        assert!(
            candidates.contains(&expected_b),
            "Should contain refinement from Op2"
        );
    }

    #[test]
    fn test_lattice_search_mixed_operators() {
        // Property 4: Mix of refining and abstaining operators.
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new()
            .register(
                Box::new(AddAtomOperator {
                    atom: atom_a.clone(),
                }),
                OperatorMetadata {
                    name: "Refine".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AlwaysAbstainOperator),
                OperatorMetadata {
                    name: "Abstain".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            1,
            "One refining + one abstaining should produce 1 candidate"
        );
        let expected = Hyp::new(vec![atom_a]);
        assert!(
            candidates.contains(&expected),
            "Should contain the single refinement"
        );
    }

    #[test]
    fn test_lattice_search_empty_operator_set() {
        // Property 5: Empty operator set produces no candidates.
        let operators = OperatorSet::new();
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(
            candidates.is_empty(),
            "Empty operator set should produce no candidates"
        );
    }

    #[test]
    fn test_lattice_search_pruning_respects_limit() {
        // Property 6: Candidate-count pruning respects max_candidates limit.
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom_b = Atom {
            system: OntologySystem::SNOMED,
            code: "2".to_string(),
            preferred_term: "B".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom_c = Atom {
            system: OntologySystem::SNOMED,
            code: "3".to_string(),
            preferred_term: "C".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new()
            .register(
                Box::new(AddAtomOperator { atom: atom_a }),
                OperatorMetadata {
                    name: "Op1".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AddAtomOperator { atom: atom_b }),
                OperatorMetadata {
                    name: "Op2".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AddAtomOperator { atom: atom_c }),
                OperatorMetadata {
                    name: "Op3".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::with_limit(operators, 2);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            2,
            "Pruning should cap candidates at limit of 2"
        );
    }

    #[test]
    fn test_lattice_search_no_limit_allows_all() {
        // Property 7: Without limit, all candidates are collected.
        let atoms: Vec<Atom> = (0..10)
            .map(|i| Atom {
                system: OntologySystem::SNOMED,
                code: format!("{}", i),
                preferred_term: format!("Atom{}", i),
                version: "2026-01-31".to_string(),
            })
            .collect();

        let mut operators = OperatorSet::new();
        for (i, atom) in atoms.iter().enumerate() {
            operators = operators.register(
                Box::new(AddAtomOperator { atom: atom.clone() }),
                OperatorMetadata {
                    name: format!("Op{}", i),
                    version: "test".to_string(),
                },
            );
        }

        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            10,
            "No limit should allow all 10 candidates"
        );
    }

    #[test]
    fn test_lattice_search_conditional_operator() {
        // Property 8: Conditional operator refines only when precondition met.
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(ConditionalOperator {
                atom: atom_a.clone(),
            }),
            OperatorMetadata {
                name: "Conditional".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            1,
            "Conditional operator should refine on unknown input"
        );
        let expected = Hyp::new(vec![atom_a]);
        assert!(candidates.contains(&expected));
    }

    #[test]
    fn test_lattice_search_conditional_operator_abstains_on_nonempty() {
        // Property 9: Conditional operator abstains when input is not unknown.
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operator_atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "Condition".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(ConditionalOperator {
                atom: operator_atom,
            }),
            OperatorMetadata {
                name: "Conditional".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::new(vec![atom_a]);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(
            candidates.is_empty(),
            "Conditional operator should abstain on non-unknown input"
        );
    }

    #[test]
    fn test_lattice_search_completeness_all_operators_tried() {
        // Property 10: All operators are tried, even after one abstains.
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom_b = Atom {
            system: OntologySystem::SNOMED,
            code: "2".to_string(),
            preferred_term: "B".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new()
            .register(
                Box::new(AlwaysAbstainOperator),
                OperatorMetadata {
                    name: "Abstain".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AddAtomOperator {
                    atom: atom_a.clone(),
                }),
                OperatorMetadata {
                    name: "Refine1".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AddAtomOperator {
                    atom: atom_b.clone(),
                }),
                OperatorMetadata {
                    name: "Refine2".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            2,
            "All non-abstaining operators should be tried"
        );
        let expected_a = Hyp::new(vec![atom_a]);
        let expected_b = Hyp::new(vec![atom_b]);
        assert!(candidates.contains(&expected_a));
        assert!(candidates.contains(&expected_b));
    }
}
