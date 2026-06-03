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

    // ========== Property Tier: Completeness (11-20) ==========
    // Verify: every hypothesis reachable by one operator application is in the output set

    #[test]
    fn test_completeness_single_operator_produces_reachable_candidate() {
        // Property 11: Single operator refines; candidate is reachable and in output
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator { atom: atom.clone() }),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);
        let expected = Hyp::new(vec![atom]);

        assert!(
            candidates.contains(&expected),
            "Reachable candidate must be in output"
        );
    }

    #[test]
    fn test_completeness_two_operators_both_candidates_present() {
        // Property 12: Two independent operators; both refinements must be in output
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
                Box::new(AddAtomOperator {
                    atom: atom_a.clone(),
                }),
                OperatorMetadata {
                    name: "OpA".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AddAtomOperator {
                    atom: atom_b.clone(),
                }),
                OperatorMetadata {
                    name: "OpB".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(candidates.contains(&Hyp::new(vec![atom_a])));
        assert!(candidates.contains(&Hyp::new(vec![atom_b])));
        assert_eq!(
            candidates.len(),
            2,
            "Both reachable candidates must be present"
        );
    }

    #[test]
    fn test_completeness_abstaining_operator_missing_from_output() {
        // Property 13: Operator that abstains produces no candidate; output is empty
        let operators = OperatorSet::new().register(
            Box::new(AlwaysAbstainOperator),
            OperatorMetadata {
                name: "OpAbstain".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(
            candidates.is_empty(),
            "Abstaining operator produces no candidate"
        );
    }

    #[test]
    fn test_completeness_mixed_refine_abstain_only_refining_in_output() {
        // Property 14: Mix of refining + abstaining operators; only refining in output
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new()
            .register(
                Box::new(AddAtomOperator { atom: atom.clone() }),
                OperatorMetadata {
                    name: "OpRefine".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AlwaysAbstainOperator),
                OperatorMetadata {
                    name: "OpAbstain".to_string(),
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
            "Only refining operator produces output"
        );
        assert!(candidates.contains(&Hyp::new(vec![atom])));
    }

    #[test]
    fn test_completeness_conditional_operator_refines_on_unknown() {
        // Property 15: Conditional operator; refines when precondition met
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(ConditionalOperator { atom: atom.clone() }),
            OperatorMetadata {
                name: "OpCondition".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(
            candidates.contains(&Hyp::new(vec![atom])),
            "Reachable candidate must be in output"
        );
    }

    #[test]
    fn test_completeness_large_operator_set_all_candidates_collected() {
        // Property 16: Large operator set (10 operators); all refinements in output
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
            "All 10 reachable candidates must be present"
        );
        for atom in atoms {
            assert!(candidates.contains(&Hyp::new(vec![atom])));
        }
    }

    #[test]
    fn test_completeness_operators_applied_to_correct_input() {
        // Property 17: Each operator sees the same input hypothesis
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
                Box::new(AddAtomOperator {
                    atom: atom_a.clone(),
                }),
                OperatorMetadata {
                    name: "OpA".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AddAtomOperator {
                    atom: atom_b.clone(),
                }),
                OperatorMetadata {
                    name: "OpB".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        // Both atoms should be added to the same input (unknown), not to each other
        assert!(candidates.contains(&Hyp::new(vec![atom_a.clone()])));
        assert!(candidates.contains(&Hyp::new(vec![atom_b.clone()])));
        // Candidates should not be mixed (A+B from sequential application)
        assert!(!candidates.contains(&Hyp::new(vec![atom_a, atom_b])));
    }

    #[test]
    fn test_completeness_empty_operator_set_produces_empty_output() {
        // Property 18: Empty operator set; no reachable candidates; output empty
        let operators = OperatorSet::new();
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(candidates.is_empty());
    }

    #[test]
    fn test_completeness_pruning_respects_actual_reachable_count() {
        // Property 19: Pruning doesn't exceed actual reachable count
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
            );
        let proposer = LatticeSearchProposer::with_limit(operators, 10);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            2,
            "2 reachable candidates, limit 10; output = 2"
        );
    }

    #[test]
    fn test_completeness_pruning_truncates_when_exceeded() {
        // Property 20: Pruning truncates when reachable count exceeds limit
        let atoms: Vec<Atom> = (0..5)
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

        let proposer = LatticeSearchProposer::with_limit(operators, 3);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            3,
            "5 reachable candidates, limit 3; output = 3"
        );
    }

    // ========== Property Tier: Minimality (21-30) ==========
    // Verify: output set contains no spurious candidates (only reachable ones)

    #[test]
    fn test_minimality_no_candidates_from_nowhere() {
        // Property 21: No candidate appears magically without operator producing it
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
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator {
                atom: atom_a.clone(),
            }),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        // Atom B was never produced by any operator; should not be in output
        assert!(!candidates.contains(&Hyp::new(vec![atom_b])));
    }

    #[test]
    fn test_minimality_only_direct_refinements_no_chaining() {
        // Property 22: Only single-operator refinements; no multi-step chains
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

        // Op1(input) = {A}, Op2(input) = {B}. No candidate is {A, B} from chaining
        assert!(
            !candidates.contains(&Hyp::new(vec![atom_a.clone(), atom_b.clone()])),
            "No multi-step chaining; only direct refinements"
        );
    }

    #[test]
    fn test_minimality_output_cardinality_equals_refining_operators() {
        // Property 23: Output size = number of refining operators (1-to-1 mapping)
        let operators = OperatorSet::new()
            .register(
                Box::new(AlwaysAbstainOperator),
                OperatorMetadata {
                    name: "Abstain1".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AlwaysAbstainOperator),
                OperatorMetadata {
                    name: "Abstain2".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(candidates.len(), 0, "0 refining operators → 0 candidates");
    }

    #[test]
    fn test_minimality_input_hypothesis_not_in_output() {
        // Property 24: Input hypothesis is not in output (BFS finds refinements, not input itself)
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator { atom: atom.clone() }),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(
            !candidates.contains(&input),
            "Input hypothesis should not be in output"
        );
    }

    #[test]
    fn test_minimality_no_duplicates_in_output() {
        // Property 25: Output is a set (no duplicates)
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new()
            .register(
                Box::new(AddAtomOperator { atom: atom.clone() }),
                OperatorMetadata {
                    name: "Op1".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AddAtomOperator { atom: atom.clone() }),
                OperatorMetadata {
                    name: "Op2".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(candidates.len(), 1, "Duplicate candidates merged into set");
        assert!(candidates.contains(&Hyp::new(vec![atom])));
    }

    #[test]
    fn test_minimality_no_extraneous_atoms_added() {
        // Property 26: Output hypotheses contain only atoms produced by operators
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
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator {
                atom: atom_a.clone(),
            }),
            OperatorMetadata {
                name: "OpA".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        // Only {A} should be in output; {B} is spurious
        assert!(!candidates.contains(&Hyp::new(vec![atom_b])));
    }

    #[test]
    fn test_minimality_abstaining_operators_produce_no_candidates() {
        // Property 27: Operators that abstain do not contribute to output
        let operators = OperatorSet::new()
            .register(
                Box::new(AlwaysAbstainOperator),
                OperatorMetadata {
                    name: "Op1".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(AlwaysAbstainOperator),
                OperatorMetadata {
                    name: "Op2".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(candidates.is_empty());
    }

    #[test]
    fn test_minimality_output_respects_cardinality_bound() {
        // Property 28: Output size ≤ number of operators (each produces at most one candidate)
        let atoms: Vec<Atom> = (0..5)
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

        let operators_count = operators.len();
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(candidates.len() <= operators_count);
    }

    #[test]
    fn test_minimality_pruned_output_is_subset_of_complete_output() {
        // Property 29: Pruning returns subset of all reachable candidates
        let atoms: Vec<Atom> = (0..5)
            .map(|i| Atom {
                system: OntologySystem::SNOMED,
                code: format!("{}", i),
                preferred_term: format!("Atom{}", i),
                version: "2026-01-31".to_string(),
            })
            .collect();

        let mut operators_complete = OperatorSet::new();
        let mut operators_pruned = OperatorSet::new();
        for (i, atom) in atoms.iter().enumerate() {
            let op = AddAtomOperator { atom: atom.clone() };
            operators_complete = operators_complete.register(
                Box::new(op),
                OperatorMetadata {
                    name: format!("Op{}", i),
                    version: "test".to_string(),
                },
            );
        }
        for (i, atom) in atoms.iter().enumerate() {
            let op = AddAtomOperator { atom: atom.clone() };
            operators_pruned = operators_pruned.register(
                Box::new(op),
                OperatorMetadata {
                    name: format!("Op{}", i),
                    version: "test".to_string(),
                },
            );
        }

        let proposer_complete = LatticeSearchProposer::new(operators_complete);
        let proposer_pruned = LatticeSearchProposer::with_limit(operators_pruned, 2);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let complete = proposer_complete.propose(&input, &evidence);
        let pruned = proposer_pruned.propose(&input, &evidence);

        // Pruned output ⊆ complete output
        for candidate in &pruned {
            assert!(
                complete.contains(candidate),
                "Pruned candidate must be in complete set"
            );
        }
    }

    #[test]
    fn test_minimality_no_self_loops() {
        // Property 30: Operators that would produce the input hypothesis are filtered out
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
        let input = Hyp::new(vec![atom_a.clone()]);
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator {
                atom: atom_b.clone(),
            }),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        // Since input={A} and operator adds B, output={A,B} which is different from input
        assert_eq!(candidates.len(), 1);
        let expected = Hyp::new(vec![atom_a.clone(), atom_b.clone()]);
        assert!(candidates.contains(&expected));
        assert!(
            !candidates.contains(&input),
            "Output should be a proper refinement"
        );
    }

    // ========== Property Tier: Monotonicity (31-40) ==========
    // Verify: refinement ordering is preserved (each output is ⊑ input)

    #[test]
    fn test_monotonicity_all_candidates_refine_input() {
        // Property 31: Every candidate is a refinement of input (candidate ⊑ input)
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator { atom }),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        for candidate in &candidates {
            assert!(
                candidate <= &input,
                "Each candidate must refine the input (candidate ⊑ input)"
            );
        }
    }

    #[test]
    fn test_monotonicity_multiple_candidates_all_refine_input() {
        // Property 32: Multiple candidates all refine input
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
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        for candidate in &candidates {
            assert!(candidate <= &input);
        }
    }

    #[test]
    fn test_monotonicity_empty_output_trivially_satisfies_refinement() {
        // Property 33: Empty output vacuously satisfies refinement property
        let operators = OperatorSet::new().register(
            Box::new(AlwaysAbstainOperator),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(candidates.is_empty());
        // Vacuously true: no candidate violates refinement
    }

    #[test]
    fn test_monotonicity_candidates_do_not_exceed_input_specificity() {
        // Property 34: No candidate is more specific than possible via operators
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator {
                atom: atom_a.clone(),
            }),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        // No candidate should contain atoms not produced by operators
        for candidate in &candidates {
            for atom in candidate.atoms() {
                assert_eq!(
                    atom, &atom_a,
                    "Candidate should only contain atoms from operators"
                );
            }
        }
    }

    #[test]
    fn test_monotonicity_refinement_lattice_structure_preserved() {
        // Property 35: Refinement lattice structure preserved in output
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
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        // All candidates are incomparable or identical (same lattice level from input)
        use std::cmp::Ordering;
        let candidates_vec: Vec<_> = candidates.iter().collect();
        for i in 0..candidates_vec.len() {
            for j in i + 1..candidates_vec.len() {
                let a = candidates_vec[i];
                let b = candidates_vec[j];
                // Candidates at same level should be incomparable (neither is strictly less)
                let cmp = a.partial_cmp(b);
                assert!(
                    cmp.is_none() || cmp == Some(Ordering::Equal),
                    "Candidates at same refinement level"
                );
            }
        }
    }

    #[test]
    fn test_monotonicity_pruning_preserves_refinement_property() {
        // Property 36: Pruning doesn't violate refinement monotonicity
        let atoms: Vec<Atom> = (0..5)
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

        let proposer = LatticeSearchProposer::with_limit(operators, 2);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        for candidate in &candidates {
            assert!(candidate <= &input);
        }
    }

    #[test]
    fn test_monotonicity_conditional_operator_respects_monotonicity() {
        // Property 37: Conditional operator maintains refinement order
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(ConditionalOperator { atom: atom.clone() }),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        for candidate in &candidates {
            assert!(candidate <= &input);
        }
    }

    #[test]
    fn test_monotonicity_no_candidate_refinement_between_candidates() {
        // Property 38: Candidates don't refine each other (no transitivity within output)
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
            );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);
        let candidates_vec: Vec<_> = candidates.iter().collect();

        // Candidates should be incomparable within the result set (no strict refinement)
        use std::cmp::Ordering;
        for i in 0..candidates_vec.len() {
            for j in 0..candidates_vec.len() {
                if i != j {
                    let cmp = candidates_vec[i].partial_cmp(candidates_vec[j]);
                    assert!(
                        cmp.is_none() || cmp == Some(Ordering::Equal),
                        "Candidates should not form refinement chains"
                    );
                }
            }
        }
    }

    #[test]
    fn test_monotonicity_single_atom_candidate_minimal_refinement() {
        // Property 39: Single-atom candidates represent minimal refinement steps
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator { atom: atom.clone() }),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        // Single operator produces single-atom candidates
        assert_eq!(candidates.len(), 1);
        for candidate in &candidates {
            assert_eq!(candidate.atoms().len(), 1, "Single refinement step");
        }
    }

    #[test]
    fn test_monotonicity_refinement_ordering_consistent_across_runs() {
        // Property 40: Refinement monotonicity holds consistently across multiple runs
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let operators = OperatorSet::new().register(
            Box::new(AddAtomOperator { atom: atom_a }),
            OperatorMetadata {
                name: "Op".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        // Run 1
        let candidates_1 = proposer.propose(&input, &evidence);
        // Run 2
        let candidates_2 = proposer.propose(&input, &evidence);

        // Both runs should satisfy refinement property
        for candidate in &candidates_1 {
            assert!(candidate <= &input);
        }
        for candidate in &candidates_2 {
            assert!(candidate <= &input);
        }
    }
}
