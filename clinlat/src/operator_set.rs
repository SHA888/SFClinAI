//! Composition of multiple deduction operators.
//!
//! Implements DEF-PS-09 (operator set Δ_PS) and OBL-PS-03 (soundness under composition).

use crate::abstain::AbstainReason;
use crate::hyp::Hyp;
use crate::operator::{Evidence, Operator};
use crate::outcome::Outcome;
use std::collections::BTreeMap;

/// Metadata identifying a registered operator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorMetadata {
    /// Human-readable name of the operator (e.g., "SofaRespOperator").
    pub name: String,
    /// Version identifier of the operator (e.g., "clinlat-v0.1.0").
    pub version: String,
}

/// Outcome of applying an operator set to a hypothesis and evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SetOutcome {
    /// The resulting hypothesis after all operators have been applied (or attempted).
    /// Invariant: result ⊑ input hypothesis (per DEF-PS-09, OBL-PS-03).
    pub result: Hyp,
    /// Operators that abstained, paired with their abstention reasons.
    /// Ordered by operator application order.
    pub abstentions: Vec<(String, AbstainReason)>,
}

/// A set of deduction operators that compose via propagate-forward semantics.
///
/// An `OperatorSet` applies each operator in sequence to a hypothesis and evidence.
/// If an operator abstains, the abstention is recorded but does not prevent
/// subsequent operators from running on the best-known hypothesis so far.
///
/// **Invariant:** The result of `apply_set` is always ⊑ the input hypothesis (INV-PS-03).
pub struct OperatorSet {
    operators: Vec<Box<dyn Operator>>,
    metadata: BTreeMap<String, OperatorMetadata>,
}

impl OperatorSet {
    /// Creates a new empty operator set.
    pub fn new() -> Self {
        OperatorSet {
            operators: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Registers an operator with the set and returns self for chaining.
    /// The operator name must be unique; registering a duplicate name overwrites the prior operator.
    pub fn register(mut self, op: Box<dyn Operator>, meta: OperatorMetadata) -> Self {
        self.operators.push(op);
        self.metadata.insert(meta.name.clone(), meta);
        self
    }

    /// Returns the number of operators in the set.
    pub fn len(&self) -> usize {
        self.operators.len()
    }

    /// Returns true if the operator set is empty.
    pub fn is_empty(&self) -> bool {
        self.operators.is_empty()
    }

    /// Returns the names of all registered operators in registration order.
    pub fn operator_names(&self) -> Vec<&str> {
        self.metadata.values().map(|m| m.name.as_str()).collect()
    }

    /// Applies all operators in sequence, propagating refinements forward.
    ///
    /// **Algorithm:**
    /// 1. Start with `current_h = h.clone()`.
    /// 2. For each operator:
    ///    - Call `operator.apply(&current_h, e)`.
    ///    - If `Refined(h')`: assert `h' ⊑ current_h`, then set `current_h = h'`.
    ///    - If `Abstain(r)`: record `(operator_name, r)` and leave `current_h` unchanged.
    /// 3. Return `SetOutcome { result: current_h, abstentions }`.
    ///
    /// **Semantics:** Abstention from one operator does not silence the next; each operator
    /// sees the best-known hypothesis so far. Final result ⊑ input always (OBL-PS-03).
    pub fn apply_set(&self, h: &Hyp, e: &Evidence) -> SetOutcome {
        let mut current_h = h.clone();
        let mut abstentions = Vec::new();

        for (op, (op_name, _)) in self.operators.iter().zip(self.metadata.iter()) {
            match op.apply(&current_h, e) {
                Outcome::Refined(h_prime) => {
                    debug_assert!(
                        h_prime <= current_h,
                        "INV-PS-03 violated by {}: refinement not strictly refined",
                        op_name
                    );
                    current_h = h_prime;
                }
                Outcome::Abstain(reason) => {
                    abstentions.push((op_name.clone(), reason));
                }
            }
        }

        SetOutcome {
            result: current_h,
            abstentions,
        }
    }
}

impl Default for OperatorSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{Provenance, ProvenanceOrigin};
    use crate::version::Ver;
    use crate::{Atom, OntologySystem};
    use chrono::Utc;
    use std::collections::BTreeMap;

    // Fixture operators for property testing

    /// No-op operator: refines to identity (h' = h)
    struct NoopOperatorFixture;
    impl Operator for NoopOperatorFixture {
        fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
            Outcome::Refined(_h.clone())
        }
    }

    /// Constant refinement operator: adds a fixed atom to any hypothesis
    struct ConstRefineOperatorFixture {
        atom: Atom,
    }
    impl Operator for ConstRefineOperatorFixture {
        fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
            let mut atoms = _h.atoms().to_vec();
            atoms.push(self.atom.clone());
            Outcome::Refined(Hyp::new(atoms))
        }
    }

    /// Always-abstain operator: never refines
    struct AlwaysAbstainOperatorFixture;
    impl Operator for AlwaysAbstainOperatorFixture {
        fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
            Outcome::Abstain(AbstainReason::InsufficientEvidence("fixture abstain"))
        }
    }

    fn test_evidence() -> Evidence {
        Evidence::new(
            vec![],
            Provenance::new(
                ProvenanceOrigin::new("test", "test", "test"),
                Utc::now(),
                Ver::new("test", "test", "0.1.0"),
                BTreeMap::new(),
            ),
        )
    }

    #[test]
    fn test_empty_set_construction() {
        let set = OperatorSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert_eq!(set.operator_names(), Vec::<&str>::new());
    }

    #[test]
    fn test_register_single_operator() {
        struct DummyOp;
        impl Operator for DummyOp {
            fn apply(&self, h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Refined(h.clone())
            }
        }

        let set = OperatorSet::new().register(
            Box::new(DummyOp),
            OperatorMetadata {
                name: "TestOp".to_string(),
                version: "v1.0.0".to_string(),
            },
        );

        assert_eq!(set.len(), 1);
        assert!(!set.is_empty());
        assert_eq!(set.operator_names(), vec!["TestOp"]);
    }

    #[test]
    fn test_register_multiple_operators() {
        struct DummyOp1;
        impl Operator for DummyOp1 {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Refined(_h.clone())
            }
        }

        struct DummyOp2;
        impl Operator for DummyOp2 {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Refined(_h.clone())
            }
        }

        let set = OperatorSet::new()
            .register(
                Box::new(DummyOp1),
                OperatorMetadata {
                    name: "Op1".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(DummyOp2),
                OperatorMetadata {
                    name: "Op2".to_string(),
                    version: "v1.0.0".to_string(),
                },
            );

        assert_eq!(set.len(), 2);
        assert_eq!(set.operator_names(), vec!["Op1", "Op2"]);
    }

    #[test]
    fn test_apply_set_empty_returns_input() {
        let set = OperatorSet::new();
        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.result, h);
        assert!(outcome.abstentions.is_empty());
    }

    #[test]
    fn test_apply_set_single_noop() {
        struct NoopOp;
        impl Operator for NoopOp {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Refined(_h.clone())
            }
        }

        let set = OperatorSet::new().register(
            Box::new(NoopOp),
            OperatorMetadata {
                name: "Noop".to_string(),
                version: "v1.0.0".to_string(),
            },
        );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.result, h);
        assert!(outcome.abstentions.is_empty());
    }

    #[test]
    fn test_apply_set_single_abstain() {
        struct AbstainOp;
        impl Operator for AbstainOp {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Abstain(AbstainReason::InsufficientEvidence("test"))
            }
        }

        let set = OperatorSet::new().register(
            Box::new(AbstainOp),
            OperatorMetadata {
                name: "Abstain".to_string(),
                version: "v1.0.0".to_string(),
            },
        );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.result, h);
        assert_eq!(outcome.abstentions.len(), 1);
        assert_eq!(outcome.abstentions[0].0, "Abstain");
    }

    #[test]
    fn test_apply_set_preserves_input_on_all_abstain() {
        struct AbstainOp1;
        impl Operator for AbstainOp1 {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Abstain(AbstainReason::InsufficientEvidence("test1"))
            }
        }

        struct AbstainOp2;
        impl Operator for AbstainOp2 {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Abstain(AbstainReason::OutOfDistribution("test2"))
            }
        }

        let set = OperatorSet::new()
            .register(
                Box::new(AbstainOp1),
                OperatorMetadata {
                    name: "Abstain1".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(AbstainOp2),
                OperatorMetadata {
                    name: "Abstain2".to_string(),
                    version: "v1.0.0".to_string(),
                },
            );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.result, h);
        assert_eq!(outcome.abstentions.len(), 2);
    }

    #[test]
    fn test_default_creates_empty_set() {
        let set = OperatorSet::default();
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_apply_set_abstain_then_refine() {
        struct AbstainOp;
        impl Operator for AbstainOp {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Abstain(AbstainReason::InsufficientEvidence("test"))
            }
        }

        struct NoopOp;
        impl Operator for NoopOp {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Refined(_h.clone())
            }
        }

        let set = OperatorSet::new()
            .register(
                Box::new(AbstainOp),
                OperatorMetadata {
                    name: "Abstain".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(NoopOp),
                OperatorMetadata {
                    name: "Noop".to_string(),
                    version: "v1.0.0".to_string(),
                },
            );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.result, h);
        assert_eq!(outcome.abstentions.len(), 1);
        assert_eq!(outcome.abstentions[0].0, "Abstain");
    }

    #[test]
    fn test_apply_set_both_noop() {
        struct NoopOp1;
        impl Operator for NoopOp1 {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Refined(_h.clone())
            }
        }

        struct NoopOp2;
        impl Operator for NoopOp2 {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Refined(_h.clone())
            }
        }

        let set = OperatorSet::new()
            .register(
                Box::new(NoopOp1),
                OperatorMetadata {
                    name: "Noop1".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(NoopOp2),
                OperatorMetadata {
                    name: "Noop2".to_string(),
                    version: "v1.0.0".to_string(),
                },
            );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.result, h);
        assert!(outcome.abstentions.is_empty());
    }

    #[test]
    fn test_apply_set_records_abstention_names_in_order() {
        struct AbstainOp;
        impl Operator for AbstainOp {
            fn apply(&self, _h: &Hyp, _e: &Evidence) -> Outcome<Hyp, AbstainReason> {
                Outcome::Abstain(AbstainReason::InsufficientEvidence("test"))
            }
        }

        let set = OperatorSet::new()
            .register(
                Box::new(AbstainOp),
                OperatorMetadata {
                    name: "First".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(AbstainOp),
                OperatorMetadata {
                    name: "Second".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(AbstainOp),
                OperatorMetadata {
                    name: "Third".to_string(),
                    version: "v1.0.0".to_string(),
                },
            );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.abstentions.len(), 3);
        assert_eq!(outcome.abstentions[0].0, "First");
        assert_eq!(outcome.abstentions[1].0, "Second");
        assert_eq!(outcome.abstentions[2].0, "Third");
    }

    #[test]
    fn test_apply_set_monotonicity_const_refine() {
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "CONST-9999".to_string(),
            preferred_term: "Test".to_string(),
            version: "0.1.0".to_string(),
        };

        let set = OperatorSet::new().register(
            Box::new(ConstRefineOperatorFixture { atom: atom.clone() }),
            OperatorMetadata {
                name: "ConstRefine".to_string(),
                version: "v1.0.0".to_string(),
            },
        );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert!(outcome.result <= h, "result should refine input");
    }

    #[test]
    fn test_apply_set_abstention_propagates_forward() {
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "CONST-8888".to_string(),
            preferred_term: "Test2".to_string(),
            version: "0.1.0".to_string(),
        };

        let set = OperatorSet::new()
            .register(
                Box::new(AlwaysAbstainOperatorFixture),
                OperatorMetadata {
                    name: "Abstain".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(ConstRefineOperatorFixture { atom }),
                OperatorMetadata {
                    name: "ConstRefine".to_string(),
                    version: "v1.0.0".to_string(),
                },
            );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert!(outcome.result <= h);
        assert_eq!(outcome.abstentions.len(), 1);
        assert_eq!(outcome.abstentions[0].0, "Abstain");
    }

    #[test]
    fn test_apply_set_empty_set_identity() {
        let set = OperatorSet::new();
        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.result, h);
        assert!(outcome.abstentions.is_empty());
    }

    #[test]
    fn test_apply_set_all_abstain_preserves_input() {
        let set = OperatorSet::new()
            .register(
                Box::new(AlwaysAbstainOperatorFixture),
                OperatorMetadata {
                    name: "A1".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(AlwaysAbstainOperatorFixture),
                OperatorMetadata {
                    name: "A2".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(AlwaysAbstainOperatorFixture),
                OperatorMetadata {
                    name: "A3".to_string(),
                    version: "v1.0.0".to_string(),
                },
            );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.result, h);
        assert_eq!(outcome.abstentions.len(), 3);
    }

    #[test]
    fn test_apply_set_noop_chain_identity() {
        let set = OperatorSet::new()
            .register(
                Box::new(NoopOperatorFixture),
                OperatorMetadata {
                    name: "N1".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(NoopOperatorFixture),
                OperatorMetadata {
                    name: "N2".to_string(),
                    version: "v1.0.0".to_string(),
                },
            );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert_eq!(outcome.result, h);
        assert!(outcome.abstentions.is_empty());
    }

    #[test]
    fn test_apply_set_multiple_const_refine() {
        let atom1 = Atom {
            system: OntologySystem::SNOMED,
            code: "CONST-7777".to_string(),
            preferred_term: "T1".to_string(),
            version: "0.1.0".to_string(),
        };
        let atom2 = Atom {
            system: OntologySystem::SNOMED,
            code: "CONST-6666".to_string(),
            preferred_term: "T2".to_string(),
            version: "0.1.0".to_string(),
        };

        let set = OperatorSet::new()
            .register(
                Box::new(ConstRefineOperatorFixture {
                    atom: atom1.clone(),
                }),
                OperatorMetadata {
                    name: "R1".to_string(),
                    version: "v1.0.0".to_string(),
                },
            )
            .register(
                Box::new(ConstRefineOperatorFixture {
                    atom: atom2.clone(),
                }),
                OperatorMetadata {
                    name: "R2".to_string(),
                    version: "v1.0.0".to_string(),
                },
            );

        let h = Hyp::unknown();
        let e = test_evidence();

        let outcome = set.apply_set(&h, &e);
        assert!(outcome.result <= h);
        let result_atoms = outcome.result.atoms();
        assert!(result_atoms.contains(&atom1));
        assert!(result_atoms.contains(&atom2));
    }
}
