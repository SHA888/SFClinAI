//! Abstention reasons for deduction operators.
//!
//! Implements DEF-PS-10 (five abstention reason variants).

/// Reasons why an operator may decline to refine a hypothesis.
///
/// An operator abstains by returning an `AbstainReason` in an `Outcome::Abstain`.
/// The reason communicates to the clinician why the operator could not proceed.
///
/// Implements DEF-PS-10 (five abstention types).
///
/// # v0.1.0 Simplification
///
/// Each variant carries a `&'static str` message rather than structured detail.
/// In v0.2.0+, variants will carry rich context (missing lab values, conflicting evidence, etc.).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbstainReason {
    /// Insufficient evidence to make a determination (e.g., missing lab values).
    InsufficientEvidence(&'static str),
    /// Evidence is outside the operator's training distribution.
    OutOfDistribution(&'static str),
    /// Multiple equally valid refinements exist (ambiguity).
    AmbiguousRefinement(&'static str),
    /// Operator precondition is unmet (e.g., required field missing).
    OperatorPreconditionUnmet(&'static str),
    /// Evidence or hypothesis refers to concepts outside the operator's ontology.
    OntologyOutOfScope(&'static str),
}

impl AbstainReason {
    /// Returns the message associated with this abstention reason.
    pub fn message(&self) -> &'static str {
        match self {
            AbstainReason::InsufficientEvidence(msg) => msg,
            AbstainReason::OutOfDistribution(msg) => msg,
            AbstainReason::AmbiguousRefinement(msg) => msg,
            AbstainReason::OperatorPreconditionUnmet(msg) => msg,
            AbstainReason::OntologyOutOfScope(msg) => msg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abstain_reason_message() {
        let reason = AbstainReason::InsufficientEvidence("FiO₂ value missing");
        assert_eq!(reason.message(), "FiO₂ value missing");
    }

    #[test]
    fn test_abstain_variants() {
        let _ = AbstainReason::InsufficientEvidence("test");
        let _ = AbstainReason::OutOfDistribution("test");
        let _ = AbstainReason::AmbiguousRefinement("test");
        let _ = AbstainReason::OperatorPreconditionUnmet("test");
        let _ = AbstainReason::OntologyOutOfScope("test");
    }
}
