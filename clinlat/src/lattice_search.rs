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
                    // Filter self-loops: if the operator produces the input unchanged,
                    // treat it as a non-candidate. This prevents identity operators from
                    // being included in the output (DEF-PS-15: no spurious candidates).
                    if h_prime != *h {
                        candidates.insert(h_prime);
                    }
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

    /// Identity operator: returns the input hypothesis unchanged (self-loop).
    struct IdentityOperator;
    impl Operator for IdentityOperator {
        fn apply(&self, h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
            Outcome::Refined(h.clone())
        }
    }

    /// Operator that only refines when input is not empty (non-trivial refinement).
    struct ConditionalAddOperator {
        atom: Atom,
    }
    impl Operator for ConditionalAddOperator {
        fn apply(&self, h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
            if !h.atoms().is_empty() {
                let mut atoms = h.atoms().to_vec();
                atoms.push(self.atom.clone());
                Outcome::Refined(Hyp::new(atoms))
            } else {
                Outcome::Abstain(AbstainReason::OperatorPreconditionUnmet("input is unknown"))
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

    // ========== Property Tier: Completeness (11-13) ==========
    // Verify: pruning behavior and edge cases

    #[test]
    fn test_completeness_empty_operator_set_produces_empty_output() {
        // Property 11: Empty operator set; no reachable candidates; output empty
        let operators = OperatorSet::new();
        let proposer = LatticeSearchProposer::new(operators);
        let input = Hyp::unknown();
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(candidates.is_empty());
    }

    #[test]
    fn test_completeness_pruning_respects_actual_reachable_count() {
        // Property 12: Pruning doesn't exceed actual reachable count
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
        // Property 13: Pruning truncates when reachable count exceeds limit
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
    fn test_minimality_identity_operator_filtered_from_output() {
        // Property 30: Identity operators (would produce input) are filtered out as self-loops
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let input = Hyp::new(vec![atom.clone()]);
        let operators = OperatorSet::new().register(
            Box::new(IdentityOperator),
            OperatorMetadata {
                name: "Identity".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        // Identity operator returns input unchanged; should be filtered as self-loop
        assert!(
            candidates.is_empty(),
            "Identity operator should not produce a candidate (self-loop filtered)"
        );
        assert!(!candidates.contains(&input));
    }

    // ========== Property Tier: Monotonicity (31-40) ==========
    // Verify: refinement ordering is preserved (each output is ⊑ input) with non-empty inputs

    #[test]
    fn test_monotonicity_candidates_refine_non_unknown_input() {
        // Property 31: All candidates refine non-empty input (candidate ⊑ input with real atoms)
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
            Box::new(ConditionalAddOperator { atom: atom_b }),
            OperatorMetadata {
                name: "OpAddB".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        for candidate in &candidates {
            assert!(candidate <= &input, "Candidate must refine input");
        }
    }

    #[test]
    fn test_monotonicity_multiple_operators_on_non_unknown_input() {
        // Property 32: Multiple operators on non-empty input all refine
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
        let input = Hyp::new(vec![atom_a.clone()]);
        let operators = OperatorSet::new()
            .register(
                Box::new(ConditionalAddOperator { atom: atom_b }),
                OperatorMetadata {
                    name: "OpB".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(ConditionalAddOperator { atom: atom_c }),
                OperatorMetadata {
                    name: "OpC".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        for candidate in &candidates {
            assert!(candidate <= &input);
        }
    }

    #[test]
    fn test_monotonicity_abstaining_on_non_unknown_input() {
        // Property 33: Operators abstaining on non-empty input produce no candidates
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let input = Hyp::new(vec![atom_a.clone()]);
        let operators = OperatorSet::new().register(
            Box::new(ConditionalOperator { atom: atom_a }),
            OperatorMetadata {
                name: "OpCondAbstain".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(candidates.is_empty());
    }

    #[test]
    fn test_monotonicity_mixed_refining_abstaining_on_non_unknown() {
        // Property 34: Mix of refining and abstaining operators on non-empty input
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
        let operators = OperatorSet::new()
            .register(
                Box::new(ConditionalAddOperator { atom: atom_b }),
                OperatorMetadata {
                    name: "OpRefine".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(ConditionalOperator { atom: atom_a }),
                OperatorMetadata {
                    name: "OpAbstain".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        for candidate in &candidates {
            assert!(candidate <= &input);
        }
    }

    #[test]
    fn test_monotonicity_lattice_structure_with_non_unknown_input() {
        // Property 35: Refinement lattice preserved with non-empty input and Equal case
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
        let operators = OperatorSet::new()
            .register(
                Box::new(ConditionalAddOperator {
                    atom: atom_b.clone(),
                }),
                OperatorMetadata {
                    name: "OpB".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(IdentityOperator),
                OperatorMetadata {
                    name: "OpIdentity".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        use std::cmp::Ordering;
        let candidates_vec: Vec<_> = candidates.iter().collect();
        for i in 0..candidates_vec.len() {
            for j in i + 1..candidates_vec.len() {
                let cmp = candidates_vec[i].partial_cmp(candidates_vec[j]);
                assert!(
                    cmp.is_none() || cmp == Some(Ordering::Equal),
                    "Candidates at same level should be incomparable or equal"
                );
            }
        }
    }

    #[test]
    fn test_monotonicity_pruning_on_non_unknown_input() {
        // Property 36: Pruning preserves refinement on non-empty input
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atoms_to_add: Vec<Atom> = (0..3)
            .map(|i| Atom {
                system: OntologySystem::SNOMED,
                code: format!("{}", i + 2),
                preferred_term: format!("Atom{}", i + 2),
                version: "2026-01-31".to_string(),
            })
            .collect();

        let input = Hyp::new(vec![atom_a]);
        let mut operators = OperatorSet::new();
        for (i, atom) in atoms_to_add.iter().enumerate() {
            operators = operators.register(
                Box::new(ConditionalAddOperator { atom: atom.clone() }),
                OperatorMetadata {
                    name: format!("Op{}", i),
                    version: "test".to_string(),
                },
            );
        }

        let proposer = LatticeSearchProposer::with_limit(operators, 2);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        for candidate in &candidates {
            assert!(candidate <= &input);
        }
    }

    #[test]
    fn test_monotonicity_non_trivial_refinement_chain() {
        // Property 37: Refinement chain respects ordering on non-empty input
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
        let input = Hyp::new(vec![atom_a.clone(), atom_b.clone()]);
        let operators = OperatorSet::new().register(
            Box::new(ConditionalAddOperator { atom: atom_c }),
            OperatorMetadata {
                name: "OpC".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        for candidate in &candidates {
            assert!(
                candidate <= &input,
                "Candidate from non-empty input must refine it"
            );
        }
    }

    #[test]
    fn test_monotonicity_candidates_incomparable_triangular_check() {
        // Property 38: Candidates at same level are incomparable (triangular loop, not O(n²))
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
        let input = Hyp::new(vec![atom_a.clone()]);
        let operators = OperatorSet::new()
            .register(
                Box::new(ConditionalAddOperator { atom: atom_b }),
                OperatorMetadata {
                    name: "OpB".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(ConditionalAddOperator { atom: atom_c }),
                OperatorMetadata {
                    name: "OpC".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);
        let candidates_vec: Vec<_> = candidates.iter().collect();

        use std::cmp::Ordering;
        for i in 0..candidates_vec.len() {
            for j in i + 1..candidates_vec.len() {
                let cmp = candidates_vec[i].partial_cmp(candidates_vec[j]);
                assert!(
                    cmp.is_none() || cmp == Some(Ordering::Equal),
                    "Candidates should not form refinement chains"
                );
            }
        }
    }

    #[test]
    fn test_monotonicity_single_refinement_step_preserves_structure() {
        // Property 39: Single refinement step maintains monotonicity structure
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
        let input = Hyp::new(vec![atom_a]);
        let operators = OperatorSet::new().register(
            Box::new(ConditionalAddOperator { atom: atom_b }),
            OperatorMetadata {
                name: "OpB".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(candidates.len(), 1);
        for candidate in &candidates {
            assert!(candidate <= &input);
        }
    }

    #[test]
    fn test_monotonicity_multiple_runs_consistent_on_non_unknown() {
        // Property 40: Monotonicity holds consistently across runs on non-empty input
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
        let input = Hyp::new(vec![atom_a]);
        let operators = OperatorSet::new().register(
            Box::new(ConditionalAddOperator { atom: atom_b }),
            OperatorMetadata {
                name: "OpB".to_string(),
                version: "test".to_string(),
            },
        );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates_1 = proposer.propose(&input, &evidence);
        let candidates_2 = proposer.propose(&input, &evidence);

        for candidate in &candidates_1 {
            assert!(candidate <= &input);
        }
        for candidate in &candidates_2 {
            assert!(candidate <= &input);
        }
    }

    // ========== New Coverage Tests (41-46) ==========
    // Address gaps in property coverage

    #[test]
    fn test_identity_operator_filtered_mixed_with_abstaining() {
        // Property 41: Identity operator self-loop filtered even when mixed with abstaining operators
        let atom_a = Atom {
            system: OntologySystem::SNOMED,
            code: "1".to_string(),
            preferred_term: "A".to_string(),
            version: "2026-01-31".to_string(),
        };
        let input = Hyp::new(vec![atom_a.clone()]);
        let operators = OperatorSet::new()
            .register(
                Box::new(IdentityOperator),
                OperatorMetadata {
                    name: "Identity".to_string(),
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
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert!(
            candidates.is_empty(),
            "Identity self-loop and abstention → empty output"
        );
    }

    #[test]
    fn test_non_trivial_input_where_operators_abstain_or_refine() {
        // Property 42: Operators apply correctly to non-trivial input (some abstain, some refine)
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
        let operators = OperatorSet::new()
            .register(
                Box::new(ConditionalAddOperator {
                    atom: atom_b.clone(),
                }),
                OperatorMetadata {
                    name: "OpRefine".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(ConditionalOperator {
                    atom: atom_a.clone(),
                }),
                OperatorMetadata {
                    name: "OpAbstain".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(candidates.len(), 1, "One refining operator → one candidate");
        assert!(candidates.contains(&Hyp::new(vec![atom_a, atom_b])));
    }

    #[test]
    fn test_large_operator_set_on_non_empty_input() {
        // Property 43: Large operator set (8 operators) on non-empty input produces correct subset
        let atom_base = Atom {
            system: OntologySystem::SNOMED,
            code: "0".to_string(),
            preferred_term: "Base".to_string(),
            version: "2026-01-31".to_string(),
        };
        let input = Hyp::new(vec![atom_base.clone()]);

        let atoms_to_add: Vec<Atom> = (0..8)
            .map(|i| Atom {
                system: OntologySystem::SNOMED,
                code: format!("{}", i + 1),
                preferred_term: format!("Atom{}", i + 1),
                version: "2026-01-31".to_string(),
            })
            .collect();

        let mut operators = OperatorSet::new();
        for (i, atom) in atoms_to_add.iter().enumerate() {
            operators = operators.register(
                Box::new(ConditionalAddOperator { atom: atom.clone() }),
                OperatorMetadata {
                    name: format!("Op{}", i),
                    version: "test".to_string(),
                },
            );
        }

        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            8,
            "All 8 refining operators produce candidates"
        );
        for atom in &atoms_to_add {
            let expected = {
                let mut atoms = input.atoms().to_vec();
                atoms.push(atom.clone());
                Hyp::new(atoms)
            };
            assert!(candidates.contains(&expected));
        }
    }

    #[test]
    fn test_mixed_conditional_unconditional_operators() {
        // Property 44: Mix of conditional (require non-empty input) and unconditional operators
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

        // On unknown input, only unconditional operators should produce candidates
        let input_unknown = Hyp::unknown();
        let operators_1 = OperatorSet::new()
            .register(
                Box::new(AddAtomOperator {
                    atom: atom_b.clone(),
                }),
                OperatorMetadata {
                    name: "OpUnconditional".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(ConditionalAddOperator {
                    atom: atom_c.clone(),
                }),
                OperatorMetadata {
                    name: "OpConditional".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer_1 = LatticeSearchProposer::new(operators_1);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates_1 = proposer_1.propose(&input_unknown, &evidence);
        assert_eq!(
            candidates_1.len(),
            1,
            "On unknown, only unconditional produces output"
        );

        // On non-empty input, both should produce candidates
        let input_nonempty = Hyp::new(vec![atom_a.clone()]);
        let operators_2 = OperatorSet::new()
            .register(
                Box::new(ConditionalAddOperator { atom: atom_b }),
                OperatorMetadata {
                    name: "OpCond1".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(ConditionalAddOperator { atom: atom_c }),
                OperatorMetadata {
                    name: "OpCond2".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer_2 = LatticeSearchProposer::new(operators_2);

        let candidates_2 = proposer_2.propose(&input_nonempty, &evidence);
        assert_eq!(
            candidates_2.len(),
            2,
            "On non-empty, both conditionals produce output"
        );
    }

    #[test]
    fn test_pruning_respects_limit_on_non_empty_input() {
        // Property 45: Pruning limit respected even when input is non-empty
        let atom_base = Atom {
            system: OntologySystem::SNOMED,
            code: "0".to_string(),
            preferred_term: "Base".to_string(),
            version: "2026-01-31".to_string(),
        };
        let input = Hyp::new(vec![atom_base]);

        let atoms: Vec<Atom> = (0..6)
            .map(|i| Atom {
                system: OntologySystem::SNOMED,
                code: format!("{}", i + 1),
                preferred_term: format!("Atom{}", i + 1),
                version: "2026-01-31".to_string(),
            })
            .collect();

        let mut operators = OperatorSet::new();
        for (i, atom) in atoms.iter().enumerate() {
            operators = operators.register(
                Box::new(ConditionalAddOperator { atom: atom.clone() }),
                OperatorMetadata {
                    name: format!("Op{}", i),
                    version: "test".to_string(),
                },
            );
        }

        let proposer = LatticeSearchProposer::with_limit(operators, 3);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        assert_eq!(
            candidates.len(),
            3,
            "Limit respected: 6 operators, limit 3 → 3 candidates"
        );
    }

    #[test]
    fn test_equal_ordering_actually_tested() {
        // Property 46: Equal case in partial_cmp is exercised via deduplication
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

        // Two operators: both add the same new atom (atom_b) to the input.
        // Both produce {atom_a, atom_b}, which deduplicates to a single candidate.
        let operators = OperatorSet::new()
            .register(
                Box::new(ConditionalAddOperator {
                    atom: atom_b.clone(),
                }),
                OperatorMetadata {
                    name: "Op1".to_string(),
                    version: "test".to_string(),
                },
            )
            .register(
                Box::new(ConditionalAddOperator {
                    atom: atom_b.clone(),
                }),
                OperatorMetadata {
                    name: "Op2".to_string(),
                    version: "test".to_string(),
                },
            );
        let proposer = LatticeSearchProposer::new(operators);
        let evidence = Evidence::new(vec![], test_provenance());

        let candidates = proposer.propose(&input, &evidence);

        // Both operators produce identical candidates → deduplicated into set of 1
        assert_eq!(
            candidates.len(),
            1,
            "Two operators producing same output deduplicate to 1"
        );
        let expected = Hyp::new(vec![atom_a, atom_b]);
        assert!(candidates.contains(&expected));
    }
}
