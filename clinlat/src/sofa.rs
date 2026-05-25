//! SOFA-3 respiratory component (PaO₂/FiO₂ ratio) deduction operator.
//!
//! Implements diagnostic scoring for respiratory distress severity in sepsis.
//! Based on Sepsis-3 definitions (Singer et al. 2016, JAMA).
//!
//! Reference: Vincent et al. (1996) on SOFA scoring; Singer et al. (2016) on Sepsis-3.

use crate::{AbstainReason, Hyp, Operator, Outcome};

// SOFA respiratory score thresholds (PaO₂/FiO₂ ratio in mmHg).
//
// Implements task 2.1: Encode SOFA respiratory thresholds per Sepsis-3.
const SOFA_SCORE_0_MIN: f64 = 400.0; // ≥400: score 0 (no respiratory dysfunction)
const SOFA_SCORE_1_MIN: f64 = 300.0; // 300–399: score 1
const SOFA_SCORE_2_MIN: f64 = 200.0; // 200–299: score 2
const SOFA_SCORE_3_MIN: f64 = 100.0; // 100–199: score 3 (requires mechanical ventilation)
// <100: score 4 (requires mechanical ventilation)

/// Evidence for SOFA respiratory scoring.
///
/// Implements task 2.2: Evidence type for SOFA respiratory component.
/// Carries:
/// - `pao2`: Arterial oxygen partial pressure (mmHg)
/// - `fio2`: Fraction of inspired oxygen (0.0–1.0)
/// - `on_mech_vent`: Whether patient is on mechanical ventilation
///
/// Note: The mechanical ventilation flag is required because Sepsis-3 defines
/// scores 3 and 4 only for intubated patients.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SofaRespEvidence {
    pub pao2: f64,
    pub fio2: f64,
    pub on_mech_vent: bool,
}

impl SofaRespEvidence {
    /// Creates a new SOFA respiratory evidence entry.
    pub fn new(pao2: f64, fio2: f64, on_mech_vent: bool) -> Self {
        SofaRespEvidence {
            pao2,
            fio2,
            on_mech_vent,
        }
    }

    /// Computes the PaO₂/FiO₂ ratio.
    ///
    /// Returns `None` if FiO₂ is zero or negative (invalid).
    pub fn pao2_fio2_ratio(&self) -> Option<f64> {
        if self.fio2 > 0.0 {
            Some(self.pao2 / self.fio2)
        } else {
            None
        }
    }
}

/// SOFA respiratory hypothesis variants.
///
/// Implements task 2.3: Hypothesis space for SOFA respiratory.
/// Variants represent different severity levels:
/// - `Unknown`: No diagnosis yet (top element in refinement order).
/// - `Score{N}`: SOFA score 0–4, from mild to severe respiratory dysfunction.
///
/// Refinement order: `Unknown` ⊐ each `Score{N}` (Unknown is least specific).
/// Compatibility: each `Score{N}` is compatible only with itself and `Unknown`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SofaRespHypothesis {
    Unknown,
    Score0,
    Score1,
    Score2,
    Score3,
    Score4,
}

impl SofaRespHypothesis {
    /// Returns the SOFA score as an integer (0–4), or `None` for `Unknown`.
    pub fn score(&self) -> Option<u8> {
        match self {
            SofaRespHypothesis::Unknown => None,
            SofaRespHypothesis::Score0 => Some(0),
            SofaRespHypothesis::Score1 => Some(1),
            SofaRespHypothesis::Score2 => Some(2),
            SofaRespHypothesis::Score3 => Some(3),
            SofaRespHypothesis::Score4 => Some(4),
        }
    }
}

/// The SOFA-3 respiratory deduction operator.
///
/// Implements task 2.4: Operator that maps PaO₂/FiO₂ evidence to SOFA respiratory score.
///
/// Logic:
/// 1. Compute PaO₂/FiO₂ ratio.
/// 2. If FiO₂ ≤ 0, abstain (insufficient evidence).
/// 3. Map ratio to SOFA score (0–4).
/// 4. If score ≥ 3 but patient is not on mechanical ventilation, abstain (precondition unmet).
/// 5. Otherwise, refine to the computed score.
pub struct SofaRespOperator;

impl Operator for SofaRespOperator {
    fn apply(&self, _h: &Hyp, _e: &crate::operator::Evidence) -> Outcome<Hyp, AbstainReason> {
        // In v0.1.0, we work with a unit Evidence type. Real evidence will be passed in v0.2.
        // For now, return an abstention indicating we need proper evidence structure.
        Outcome::Abstain(AbstainReason::InsufficientEvidence(
            "SOFA respiratory operator requires structured evidence (v0.2+)",
        ))
    }
}

/// Standalone function for SOFA respiratory scoring.
///
/// This function implements the actual scoring logic, independent of the generic
/// `Operator` trait (which uses unit `Evidence` in v0.1.0).
///
/// # Arguments
///
/// - `ratio`: PaO₂/FiO₂ ratio in mmHg.
/// - `on_mech_vent`: Whether patient is on mechanical ventilation.
///
/// # Returns
///
/// - `Some(score)`: The computed SOFA respiratory score (0–4).
/// - `None`: If preconditions are unmet (score ≥3 without ventilation).
///
/// # Note
///
/// This is a temporary implementation for v0.1.0. In v0.2.0+, the operator will
/// accept structured `Evidence` and return `Outcome<Hyp, AbstainReason>`.
pub fn score_from_ratio(ratio: f64, on_mech_vent: bool) -> Option<u8> {
    let score = if ratio >= SOFA_SCORE_0_MIN {
        0
    } else if ratio >= SOFA_SCORE_1_MIN {
        1
    } else if ratio >= SOFA_SCORE_2_MIN {
        2
    } else if ratio >= SOFA_SCORE_3_MIN {
        3
    } else {
        4
    };

    // Sepsis-3: Scores 3 and 4 require mechanical ventilation.
    if score >= 3 && !on_mech_vent {
        return None;
    }

    Some(score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pao2_fio2_ratio_valid() {
        let evidence = SofaRespEvidence::new(350.0, 1.0, false);
        assert_eq!(evidence.pao2_fio2_ratio(), Some(350.0));
    }

    #[test]
    fn test_pao2_fio2_ratio_zero_fio2() {
        let evidence = SofaRespEvidence::new(350.0, 0.0, false);
        assert_eq!(evidence.pao2_fio2_ratio(), None);
    }

    #[test]
    fn test_pao2_fio2_ratio_negative_fio2() {
        let evidence = SofaRespEvidence::new(350.0, -0.5, false);
        assert_eq!(evidence.pao2_fio2_ratio(), None);
    }

    #[test]
    fn test_score_from_ratio_score0() {
        let ratio = 400.0;
        assert_eq!(score_from_ratio(ratio, false), Some(0));
    }

    #[test]
    fn test_score_from_ratio_score1() {
        let ratio = 350.0;
        assert_eq!(score_from_ratio(ratio, false), Some(1));
    }

    #[test]
    fn test_score_from_ratio_score2() {
        let ratio = 250.0;
        assert_eq!(score_from_ratio(ratio, false), Some(2));
    }

    #[test]
    fn test_score_from_ratio_score3_with_vent() {
        let ratio = 150.0;
        assert_eq!(score_from_ratio(ratio, true), Some(3));
    }

    #[test]
    fn test_score_from_ratio_score3_without_vent() {
        let ratio = 150.0;
        // Score 3 requires mechanical ventilation; abstain if not ventilated.
        assert_eq!(score_from_ratio(ratio, false), None);
    }

    #[test]
    fn test_score_from_ratio_score4_with_vent() {
        let ratio = 80.0;
        assert_eq!(score_from_ratio(ratio, true), Some(4));
    }

    #[test]
    fn test_score_from_ratio_score4_without_vent() {
        let ratio = 80.0;
        // Score 4 requires mechanical ventilation.
        assert_eq!(score_from_ratio(ratio, false), None);
    }

    #[test]
    fn test_sofa_resp_hypothesis_score() {
        assert_eq!(SofaRespHypothesis::Unknown.score(), None);
        assert_eq!(SofaRespHypothesis::Score0.score(), Some(0));
        assert_eq!(SofaRespHypothesis::Score1.score(), Some(1));
        assert_eq!(SofaRespHypothesis::Score2.score(), Some(2));
        assert_eq!(SofaRespHypothesis::Score3.score(), Some(3));
        assert_eq!(SofaRespHypothesis::Score4.score(), Some(4));
    }
}
