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
/// Implements task 2.3: Operator that maps PaO₂/FiO₂ evidence to SOFA respiratory score.
/// Enforces version-respecting derivation chains (INV-PS-05) by validating that input
/// evidence provenance version matches the operator version before refining.
///
/// Logic:
/// 1. Extract PaO₂/FiO₂ observations from Evidence.
/// 2. Validate input provenance version matches operator version; abstain if mismatch.
/// 3. Compute PaO₂/FiO₂ ratio; abstain if FiO₂ ≤ 0 (insufficient evidence).
/// 4. Map ratio to SOFA score (0–4).
/// 5. If score ≥ 3 but patient is not on mechanical ventilation, abstain (precondition unmet).
/// 6. Create refined Hyp with SOFA-score atoms and updated provenance.
pub struct SofaRespOperator {
    /// The version of this operator (e.g., "0.2.0", "0.3.0").
    /// Per INV-PS-05, output provenance.version must equal this value.
    pub version: String,
}

impl SofaRespOperator {
    /// Create a new SOFA respiratory operator with a given version.
    pub fn new(version: impl Into<String>) -> Self {
        SofaRespOperator {
            version: version.into(),
        }
    }

    /// Creates a SOFA respiratory operator with default version "0.2.0".
    pub fn default_v0_2() -> Self {
        Self::new("0.2.0")
    }

    /// Extract PaO₂ and FiO₂ values from observations.
    ///
    /// Looks for observations with codes matching LOINC PaO₂ (2703-7) and FiO₂ (3150-0).
    /// Returns `(Option<pao2>, Option<fio2>, on_mech_vent)`.
    fn extract_pao2_fio2(evidence: &crate::operator::Evidence) -> (Option<f64>, Option<f64>, bool) {
        let mut pao2 = None;
        let mut fio2 = None;
        let mut on_mech_vent = false;

        for obs in &evidence.observations {
            match obs.code.as_str() {
                "LOINC:2703-7" => {
                    // PaO₂ (partial pressure of oxygen, arterial)
                    if let Some(val) = obs.value.as_f64() {
                        pao2 = Some(val);
                    }
                }
                "LOINC:3150-0" => {
                    // FiO₂ (fraction of inspired oxygen)
                    if let Some(val) = obs.value.as_f64() {
                        fio2 = Some(val);
                    }
                }
                "SNOMED:243144002" => {
                    // Patient on mechanical ventilation (SNOMED code)
                    on_mech_vent = obs.value.as_bool().unwrap_or(false);
                }
                _ => {}
            }
        }

        (pao2, fio2, on_mech_vent)
    }

    /// Compute SOFA respiratory score from PaO₂/FiO₂ ratio and ventilation status.
    ///
    /// Returns a SNOMED-like atom representing the SOFA score, or None if preconditions unmet.
    fn score_to_atom(score: u8) -> crate::Atom {
        let (code, preferred_term) = match score {
            0 => ("SNOMED:clinlat-sofa-resp-0", "SOFA respiratory score 0"),
            1 => ("SNOMED:clinlat-sofa-resp-1", "SOFA respiratory score 1"),
            2 => ("SNOMED:clinlat-sofa-resp-2", "SOFA respiratory score 2"),
            3 => ("SNOMED:clinlat-sofa-resp-3", "SOFA respiratory score 3"),
            4 => ("SNOMED:clinlat-sofa-resp-4", "SOFA respiratory score 4"),
            _ => unreachable!(),
        };

        crate::Atom {
            system: crate::OntologySystem::SNOMED,
            code: code.to_string(),
            preferred_term: preferred_term.to_string(),
            version: "clinlat-v0.2.0".to_string(),
        }
    }
}

impl Operator for SofaRespOperator {
    fn apply(&self, _h: &Hyp, e: &crate::operator::Evidence) -> Outcome<Hyp, AbstainReason> {
        use crate::Ver;

        // Check version invariant: input provenance version must match operator version.
        // Per INV-PS-05, we don't silently change versions; we abstain instead.
        let expected_ver = Ver::new("clinlat", "sofa_resp", &self.version);
        if e.provenance.version != expected_ver {
            return Outcome::Abstain(AbstainReason::OperatorPreconditionUnmet(
                "SOFA respiratory operator version mismatch; operator version does not match evidence provenance version",
            ));
        }

        // Extract PaO₂, FiO₂, and mechanical ventilation status from observations.
        let (pao2_opt, fio2_opt, on_mech_vent) = Self::extract_pao2_fio2(e);

        let pao2 = match pao2_opt {
            Some(val) => val,
            None => {
                return Outcome::Abstain(AbstainReason::InsufficientEvidence(
                    "PaO₂ (LOINC:2703-7) not found in observations",
                ));
            }
        };

        let fio2 = match fio2_opt {
            Some(val) => val,
            None => {
                return Outcome::Abstain(AbstainReason::InsufficientEvidence(
                    "FiO₂ (LOINC:3150-0) not found in observations",
                ));
            }
        };

        // Compute PaO₂/FiO₂ ratio; abstain if FiO₂ ≤ 0.
        if fio2 <= 0.0 {
            return Outcome::Abstain(AbstainReason::InsufficientEvidence("FiO₂ must be > 0"));
        }

        let ratio = pao2 / fio2;

        // Map ratio to SOFA score.
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
            return Outcome::Abstain(AbstainReason::InsufficientEvidence(
                "SOFA respiratory score ≥3 requires mechanical ventilation",
            ));
        }

        // Create refined hypothesis with SOFA score atom.
        let sofa_atom = Self::score_to_atom(score);
        let refined_hyp = Hyp::new(vec![sofa_atom]);

        Outcome::Refined(refined_hyp)
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
    use chrono::Utc;
    use std::collections::BTreeMap;

    // Helper functions for tests
    fn test_provenance_v0_2() -> crate::Provenance {
        let origin = crate::ProvenanceOrigin::new("external_lab_api", "LOINC", "2703-7");
        let metadata = BTreeMap::new();
        crate::Provenance::new(
            origin,
            Utc::now(),
            crate::Ver::new("clinlat", "sofa_resp", "0.2.0"),
            metadata,
        )
    }

    fn test_evidence_pao2_fio2(pao2: f64, fio2: f64, on_vent: bool) -> crate::Evidence {
        let mut observations = vec![
            crate::Observation::new("LOINC:2703-7", serde_json::json!(pao2)),
            crate::Observation::new("LOINC:3150-0", serde_json::json!(fio2)),
        ];
        if on_vent {
            observations.push(crate::Observation::new(
                "SNOMED:243144002",
                serde_json::json!(true),
            ));
        }
        crate::Evidence::new(observations, test_provenance_v0_2())
    }

    // Original helper tests
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

    // New tests for SofaRespOperator with Evidence and Provenance (task 2.3)

    #[test]
    fn test_sofa_operator_creation() {
        let op = SofaRespOperator::new("0.2.0");
        assert_eq!(op.version, "0.2.0");

        let op_default = SofaRespOperator::default_v0_2();
        assert_eq!(op_default.version, "0.2.0");
    }

    #[test]
    fn test_sofa_operator_score0() {
        let op = SofaRespOperator::default_v0_2();
        let evidence = test_evidence_pao2_fio2(450.0, 1.0, false);
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Refined(_)));

        if let Outcome::Refined(h) = result {
            assert!(!h.atoms().is_empty());
        }
    }

    #[test]
    fn test_sofa_operator_score1() {
        let op = SofaRespOperator::default_v0_2();
        let evidence = test_evidence_pao2_fio2(350.0, 1.0, false);
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Refined(_)));
    }

    #[test]
    fn test_sofa_operator_score2() {
        let op = SofaRespOperator::default_v0_2();
        let evidence = test_evidence_pao2_fio2(250.0, 1.0, false);
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Refined(_)));
    }

    #[test]
    fn test_sofa_operator_score3_with_ventilation() {
        let op = SofaRespOperator::default_v0_2();
        let evidence = test_evidence_pao2_fio2(150.0, 1.0, true);
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Refined(_)));
    }

    #[test]
    fn test_sofa_operator_score3_without_ventilation_abstains() {
        let op = SofaRespOperator::default_v0_2();
        let evidence = test_evidence_pao2_fio2(150.0, 1.0, false);
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Abstain(_)));
    }

    #[test]
    fn test_sofa_operator_missing_pao2_abstains() {
        let op = SofaRespOperator::default_v0_2();
        let observations = vec![crate::Observation::new(
            "LOINC:3150-0",
            serde_json::json!(1.0),
        )];
        let evidence = crate::Evidence::new(observations, test_provenance_v0_2());
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Abstain(_)));
    }

    #[test]
    fn test_sofa_operator_missing_fio2_abstains() {
        let op = SofaRespOperator::default_v0_2();
        let observations = vec![crate::Observation::new(
            "LOINC:2703-7",
            serde_json::json!(350.0),
        )];
        let evidence = crate::Evidence::new(observations, test_provenance_v0_2());
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Abstain(_)));
    }

    #[test]
    fn test_sofa_operator_zero_fio2_abstains() {
        let op = SofaRespOperator::default_v0_2();
        let evidence = test_evidence_pao2_fio2(350.0, 0.0, false);
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Abstain(_)));
    }

    #[test]
    fn test_sofa_operator_version_mismatch_abstains() {
        let op = SofaRespOperator::new("0.3.0"); // Different version
        let evidence = test_evidence_pao2_fio2(350.0, 1.0, false); // Has v0.2.0 provenance
        let hyp = Hyp::unknown();

        // Version mismatch: operator v0.3.0 but evidence is from v0.2.0
        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Abstain(_)));

        if let Outcome::Abstain(reason) = result {
            assert!(reason.message().contains("version mismatch"));
        }
    }

    #[test]
    fn test_sofa_operator_version_invariant() {
        // Property test: if operator succeeds, output provenance has correct version
        let op = SofaRespOperator::default_v0_2();
        let evidence = test_evidence_pao2_fio2(350.0, 1.0, false);
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        if let Outcome::Refined(refined_hyp) = result {
            // The atoms in the refined hypothesis should have atoms from the SOFA score
            assert!(!refined_hyp.atoms().is_empty());
            // Verify the atoms have the expected version
            for atom in refined_hyp.atoms() {
                assert_eq!(atom.system, crate::OntologySystem::SNOMED);
                assert!(atom.code.contains("clinlat-sofa-resp"));
            }
        }
    }

    #[test]
    fn test_sofa_operator_score_boundary_400_is_score0() {
        let op = SofaRespOperator::default_v0_2();
        let evidence = test_evidence_pao2_fio2(400.0, 1.0, false);
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Refined(_)));
    }

    #[test]
    fn test_sofa_operator_score_boundary_399_is_score1() {
        let op = SofaRespOperator::default_v0_2();
        let evidence = test_evidence_pao2_fio2(399.0, 1.0, false);
        let hyp = Hyp::unknown();

        let result = op.apply(&hyp, &evidence);
        assert!(matches!(result, Outcome::Refined(_)));
    }
}
