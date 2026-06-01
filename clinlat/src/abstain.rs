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
/// # v0.1.0 Simplification — TODO: Upgrade in v0.2.0+
///
/// Each variant currently carries a `&'static str` message rather than structured detail.
/// Per SPEC.md DEF-PS-10, variants should carry:
/// - `InsufficientEvidence(missing: Set<RequiredObservation>)` — names which observations are missing
/// - `OutOfDistribution(detail: OodReport)` — characterizes the OOD region (e.g., age range, lab bounds)
/// - `AmbiguousRefinement(candidates: Set<Hyp>, rationale: Provenance)` — enumerates competing hypotheses
/// - `OperatorPreconditionUnmet(operator: OperatorName, condition: PreconditionId)` — names the condition
/// - `OntologyOutOfScope(atoms: Set<AtomId>)` — names the unknown atoms
///
/// This enables downstream audit tooling (e.g., logging, debugging, automated escalation) to
/// machine-classify which observation or precondition is the blocker, rather than parsing
/// unstructured text. See SPEC.md §7 / OQ-PS-10.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbstainReason {
    /// Insufficient evidence to make a determination (e.g., missing lab values).
    /// TODO (v0.2.0+): replace `&'static str` with `Set<RequiredObservation>` per DEF-PS-10.
    InsufficientEvidence(&'static str),
    /// Evidence is outside the operator's training distribution.
    /// TODO (v0.2.0+): replace `&'static str` with `OodReport` per DEF-PS-10.
    OutOfDistribution(&'static str),
    /// Multiple equally valid refinements exist (ambiguity).
    /// TODO (v0.2.0+): replace `&'static str` with `(Set<Hyp>, Provenance)` per DEF-PS-10.
    AmbiguousRefinement(&'static str),
    /// Operator precondition is unmet (e.g., required field missing).
    /// TODO (v0.2.0+): replace `&'static str` with `(OperatorName, PreconditionId)` per DEF-PS-10.
    OperatorPreconditionUnmet(&'static str),
    /// Evidence or hypothesis refers to concepts outside the operator's ontology.
    /// TODO (v0.2.0+): replace `&'static str` with `Set<AtomId>` per DEF-PS-10.
    OntologyOutOfScope(&'static str),
    /// No operator in the set licenses any of the proposed candidates (DEF-PS-12/13, soundness gate M2.2).
    /// When all candidates from the proposer are rejected by the soundness-verification gate,
    /// the substrate returns this reason instead of silently dropping them.
    /// TODO (v0.2.0+): replace `&'static str` with structured detail `(candidates_rejected: usize, operator_verdicts: ...)`.
    NoOperatorLicenses(&'static str),
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
            AbstainReason::NoOperatorLicenses(msg) => msg,
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
        let _ = AbstainReason::NoOperatorLicenses("test");
    }
}
