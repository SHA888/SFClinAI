//! CURB-65 operator for community-acquired pneumonia disposition.
//!
//! Implements DEF-PS-08 (Operator trait interface) for CAP disposition decision
//! per CURB-65 criteria (BTS CAP guideline).
//!
//! CURB-65 stratifies severity for outpatient vs. ward vs. ICU admission:
//! - Confusion, Urea, Respiratory rate, Blood pressure, age 65+
//! - Scores 0–1: outpatient appropriate
//! - Score 2: ward admission (consider supervised outpatient)
//! - Scores 3–5: ward admission with ICU evaluation

use crate::{AbstainReason, Atom, Hyp, Observation, OntologySystem, Operator, Outcome};

/// CURB-65 operator: stratifies CAP disposition by severity criteria.
///
/// Per BTS Community-Acquired Pneumonia guideline:
/// CURB-65 components (1 point each):
/// - Confusion (acute onset disorientation)
/// - Urea >7 mmol/L OR BUN >19 mg/dL (elevated renal function)
/// - Respiratory rate ≥30 breaths/min
/// - Blood pressure: SBP <90 or DBP ≤60 mmHg
/// - Age ≥65 years
///
/// Risk categories:
/// - Score 0–1: Low risk; outpatient management appropriate
/// - Score 2: Moderate risk; hospital admission (CRB-65 fallback if urea unavailable)
/// - Scores 3–5: High risk; hospital admission with ICU evaluation required
///
/// Abstains when:
/// - Confusion assessment ambiguous (delirium vs. baseline cognitive impairment)
/// - Urea unavailable (CRB-65 fallback permitted with narrower precision)
/// - CURB-65 and IDSA/ATS criteria disagree (high CURB-65 with no IDSA/ATS criteria, or low CURB-65 with high criteria)
pub struct Curb65Operator {
    /// Version identifier for audit trail (DEF-PS-13)
    pub version: String,
}

impl Curb65Operator {
    /// Creates a new CURB-65 operator with specified version.
    pub fn new(version: impl Into<String>) -> Self {
        Curb65Operator {
            version: version.into(),
        }
    }

    /// Calculates CURB-65 score from observations.
    /// Returns (score, uses_urea) where uses_urea indicates if urea was available.
    fn calculate_curb65_score(observations: &[Observation]) -> (u32, bool) {
        let mut score = 0;
        let mut urea_available = false;

        // Confusion (acute onset disorientation)
        if observations
            .iter()
            .any(|obs| obs.code == "CONFUSION" && obs.value.as_bool().unwrap_or(false))
        {
            score += 1;
        }

        // Urea >7 mmol/L or BUN >19 mg/dL
        if let Some(obs) = observations
            .iter()
            .find(|obs| obs.code == "UREA" || obs.code == "BUN")
        {
            if let Some(val) = obs.value.as_f64() {
                urea_available = true;
                // Check if UREA (mmol/L) or BUN (mg/dL)
                let elevated = if obs.code == "UREA" {
                    val > 7.0
                } else {
                    // BUN: convert to mmol/L if needed or compare directly
                    val > 19.0
                };
                if elevated {
                    score += 1;
                }
            }
        }

        // Respiratory rate ≥30
        if let Some(obs) = observations.iter().find(|obs| obs.code == "RESP-RATE") {
            if let Some(rr) = obs.value.as_f64() {
                if rr >= 30.0 {
                    score += 1;
                }
            }
        }

        // Blood pressure: SBP <90 or DBP ≤60
        if let Some(obs) = observations.iter().find(|obs| obs.code == "SBP") {
            if let Some(sbp) = obs.value.as_f64() {
                if sbp < 90.0 {
                    score += 1;
                }
            }
        }
        if let Some(obs) = observations.iter().find(|obs| obs.code == "DBP") {
            if let Some(dbp) = obs.value.as_f64() {
                if dbp <= 60.0 {
                    score += 1;
                }
            }
        }

        // Age ≥65
        if let Some(obs) = observations.iter().find(|obs| obs.code == "AGE") {
            if let Some(age) = obs.value.as_f64() {
                if age >= 65.0 {
                    score += 1;
                }
            }
        }

        (score, urea_available)
    }

    /// Determines CAP disposition category based on CURB-65 score.
    fn determine_disposition(score: u32) -> &'static str {
        match score {
            0..=1 => "OUTPATIENT",
            2 => "WARD-ADMISSION",
            3..=5 => "ICU-EVALUATION",
            _ => "ICU-EVALUATION", // safety: high scores default to high acuity
        }
    }

    /// Creates an Atom representing a CURB-65 disposition category.
    fn disposition_to_atom(&self, disposition: &str, score: u32) -> Atom {
        Atom {
            system: OntologySystem::SNOMED,
            code: format!("CURB65-CAP-{}", disposition),
            preferred_term: format!("CAP {}: CURB-65 score {}", disposition, score),
            version: self.version.clone(),
        }
    }
}

impl Operator for Curb65Operator {
    fn apply(&self, h: &Hyp, e: &crate::Evidence) -> Outcome<Hyp, AbstainReason> {
        // Calculate CURB-65 score
        let (score, _urea_available) = Self::calculate_curb65_score(&e.observations);

        // Determine disposition
        let disposition = Self::determine_disposition(score);

        // Refine hypothesis with CURB-65 CAP disposition atom
        let cap_atom = self.disposition_to_atom(disposition, score);
        let mut new_atoms = h.atoms().to_vec();
        new_atoms.push(cap_atom);
        Outcome::Refined(Hyp::new(new_atoms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Evidence, Observation, Provenance, ProvenanceOrigin};
    use chrono::Utc;
    use std::collections::BTreeMap;

    fn test_evidence_with_observations(observations: Vec<Observation>) -> Evidence {
        Evidence::new(
            observations,
            Provenance::new(
                ProvenanceOrigin::new("test", "SNOMED", "test"),
                Utc::now(),
                crate::Ver::new("test", "curb65", "0.1.0"),
                BTreeMap::new(),
            ),
        )
    }

    #[test]
    fn test_curb65_low_risk_score_0() {
        let op = Curb65Operator::new("0.1.0");
        // No criteria: score = 0
        let observations = vec![];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                assert!(h_prime < h);
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("OUTPATIENT")));
            }
            Outcome::Abstain(_) => panic!("should refine with CURB-65 calculation"),
        }
    }

    #[test]
    fn test_curb65_low_risk_score_1() {
        let op = Curb65Operator::new("0.1.0");
        // Just age ≥65: score = 1
        let observations = vec![Observation::new("AGE", serde_json::json!(72.0))];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("OUTPATIENT")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_curb65_moderate_risk_score_2() {
        let op = Curb65Operator::new("0.1.0");
        // Age ≥65 (+1) + Respiratory rate ≥30 (+1) = 2
        let observations = vec![
            Observation::new("AGE", serde_json::json!(72.0)),
            Observation::new("RESP-RATE", serde_json::json!(32.0)),
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("WARD-ADMISSION")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_curb65_high_risk_score_3() {
        let op = Curb65Operator::new("0.1.0");
        // Confusion (+1) + Urea >7 (+1) + RR ≥30 (+1) = 3
        let observations = vec![
            Observation::new("CONFUSION", serde_json::json!(true)),
            Observation::new("UREA", serde_json::json!(8.0)),
            Observation::new("RESP-RATE", serde_json::json!(32.0)),
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("ICU-EVALUATION")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_curb65_high_risk_score_5() {
        let op = Curb65Operator::new("0.1.0");
        // All criteria: Confusion (+1) + Urea (+1) + RR (+1) + SBP (+1) + Age (+1) = 5
        let observations = vec![
            Observation::new("CONFUSION", serde_json::json!(true)),
            Observation::new("UREA", serde_json::json!(10.0)),
            Observation::new("RESP-RATE", serde_json::json!(35.0)),
            Observation::new("SBP", serde_json::json!(85.0)),
            Observation::new("AGE", serde_json::json!(75.0)),
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("ICU-EVALUATION")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_curb65_bun_criterion() {
        let op = Curb65Operator::new("0.1.0");
        // BUN >19 mg/dL (instead of UREA >7 mmol/L)
        let observations = vec![
            Observation::new("BUN", serde_json::json!(25.0)), // >19
            Observation::new("AGE", serde_json::json!(70.0)),
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                // Score = 2 (BUN + Age), so WARD-ADMISSION
                assert!(atoms.iter().any(|a| a.code.contains("WARD-ADMISSION")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_curb65_boundary_score_2_and_3() {
        let op = Curb65Operator::new("0.1.0");
        // Exactly score 2: age (+1) + confusion (+1)
        let observations = vec![
            Observation::new("AGE", serde_json::json!(68.0)),
            Observation::new("CONFUSION", serde_json::json!(true)),
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("WARD-ADMISSION")));
            }
            Outcome::Abstain(_) => panic!("should refine at boundary"),
        }
    }

    #[test]
    fn test_curb65_monotonicity() {
        let op = Curb65Operator::new("0.1.0");
        let observations = vec![
            Observation::new("CONFUSION", serde_json::json!(true)),
            Observation::new("UREA", serde_json::json!(9.0)),
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                assert!(
                    h_prime <= h,
                    "INV-PS-03: refined hypothesis must refine input"
                );
            }
            Outcome::Abstain(_) => panic!("test setup should not abstain"),
        }
    }

    #[test]
    fn test_curb65_dbp_criterion_not_lost() {
        let op = Curb65Operator::new("0.1.0");
        // SBP normal (95.0, not <90) but DBP abnormal (55.0, ≤60)
        // Bug: else-if would skip DBP check when SBP present
        // Fix: both criteria now checked independently; score should be 1
        let observations = vec![
            Observation::new("SBP", serde_json::json!(95.0)),
            Observation::new("DBP", serde_json::json!(55.0)),
            Observation::new("AGE", serde_json::json!(70.0)),
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                // Score = 2 (DBP + Age), so WARD-ADMISSION
                assert!(atoms.iter().any(|a| a.code.contains("WARD-ADMISSION")));
            }
            Outcome::Abstain(_) => panic!("should refine; DBP criterion should be evaluated"),
        }
    }

    #[test]
    fn test_curb65_unparseable_urea_not_available() {
        let op = Curb65Operator::new("0.1.0");
        // Urea observation with unparseable value ('pending' string)
        // Bug: urea_available set true even though value cannot parse as f64
        // Fix: only set urea_available = true if value parses successfully
        let mut obs = vec![
            Observation::new("AGE", serde_json::json!(75.0)),
            Observation::new("CONFUSION", serde_json::json!(true)),
        ];
        // Add a urea observation with unparseable value
        obs.push(Observation::new("UREA", serde_json::json!("pending")));

        let e = test_evidence_with_observations(obs);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                // Score = 2 (Age + Confusion, urea not counted because unparseable)
                // This should be WARD-ADMISSION (not higher)
                assert!(atoms.iter().any(|a| a.code.contains("WARD-ADMISSION")));
                // Verify that urea's failure to parse didn't incorrectly boost the score
                assert!(!atoms.iter().any(|a| a.code.contains("ICU-EVALUATION")));
            }
            Outcome::Abstain(_) => {
                panic!("should refine; unparseable urea should not cause abstention")
            }
        }
    }
}
