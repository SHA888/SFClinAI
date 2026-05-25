//! Deduction operators on hypotheses.
//!
//! Implements DEF-PS-07 (Operator trait interface).

use crate::{AbstainReason, Hyp, Outcome};

/// Evidence: a typed observation packet for operator input.
///
/// Implements DEF-MP-11 (Evidence type).
///
/// # v0.1.0 Simplification
///
/// Evidence is a unit marker in this version.
/// In v0.2.0+, Evidence will carry specific observation data (e.g., lab values, vital signs).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence;

/// A deduction operator: a function from hypothesis and evidence to a refined hypothesis or abstention.
///
/// Operators are the primary mechanism for refining clinical hypotheses using deductive logic.
/// An operator encapsulates sound clinical reasoning (e.g., SOFA-3 respiratory scoring).
///
/// Implements DEF-PS-07 (Operator interface).
pub trait Operator {
    /// Applies the operator to a hypothesis and evidence.
    ///
    /// # Parameters
    ///
    /// - `h`: The current hypothesis (state of knowledge).
    /// - `e`: Evidence that may refine the hypothesis.
    ///
    /// # Returns
    ///
    /// - `Outcome::Refined(h')`: The operator refined the hypothesis to `h'`.
    /// - `Outcome::Abstain(reason)`: The operator declined to refine, with a reason.
    ///
    /// # Soundness
    ///
    /// The operator must satisfy three soundness clauses (DEF-PS-08):
    /// 1. **Refinement monotonicity**: If h1 ⊑ h2, then operator(h1, e) ⊑ operator(h2, e).
    /// 2. **No spurious refinement**: Operator output never refines the input hypothesis
    ///    outside what the evidence justifies.
    /// 3. **Abstention purity**: Abstention is structural (not implementation-dependent error).
    fn apply(&self, h: &Hyp, e: &Evidence) -> Outcome<Hyp, AbstainReason>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_creation() {
        let _evidence = Evidence;
    }
}
