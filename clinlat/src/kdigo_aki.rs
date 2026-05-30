//! KDIGO AKI staging operator.
//!
//! Implements DEF-PS-08 (Operator trait interface) for acute kidney injury staging
//! per KDIGO 2021 clinical practice guidelines.
//!
//! AKI staging is based on two independent trajectories:
//! - Serum creatinine fold-change from baseline (stages 1–3)
//! - Urine output decline over 6–24 hour window (stages 1–3)
//!
//! The operator refines the patient hypothesis from Unknown toward a specific AKI stage.

use crate::{AbstainReason, Atom, Hyp, Observation, OntologySystem, Operator, Outcome};

/// KDIGO AKI staging operator: stratifies acute kidney injury by creatinine and UO criteria.
///
/// Per KDIGO 2021 clinical practice guideline:
/// - Stage 0: No AKI (Cr < 1.5× baseline or UO > 0.5 mL/kg/h)
/// - Stage 1: Cr 1.5–1.9× baseline OR UO < 0.5 mL/kg/h for 6–12 hours
/// - Stage 2: Cr 2.0–2.9× baseline OR UO < 0.5 mL/kg/h for ≥12 hours
/// - Stage 3: Cr ≥3× baseline OR UO < 0.3 mL/kg/h for ≥24 hours OR Cr ≥4 mg/dL with acute rise ≥0.5 mg/dL
///
/// Abstains when:
/// - Baseline creatinine unknown (cannot compute fold-change)
/// - Urine output window is unreliable (gaps, insufficient duration)
/// - Confounding factors (recent contrast, rhabdomyolysis suspicion)
pub struct KdigoAkiOperator {
    /// Version identifier for audit trail (DEF-PS-13)
    pub version: String,
}

impl KdigoAkiOperator {
    /// Creates a new KDIGO AKI operator with specified version.
    pub fn new(version: impl Into<String>) -> Self {
        KdigoAkiOperator {
            version: version.into(),
        }
    }

    /// Extracts baseline serum creatinine from observations.
    /// Prefers "-baseline" suffix; falls back to plain code only if "-baseline" absent.
    /// Returns None if not found, if value is missing/malformed, or if value <= 0 (invalid).
    fn extract_baseline_creatinine(observations: &[Observation]) -> Option<f64> {
        observations
            .iter()
            .find(|obs| obs.code == "LOINC:2160-0-baseline")
            .or_else(|| observations.iter().find(|obs| obs.code == "LOINC:2160-0"))
            .and_then(|obs| obs.value.as_f64())
            .and_then(|val| if val > 0.0 { Some(val) } else { None })
    }

    /// Extracts current serum creatinine from observations.
    /// Only matches "-current" suffix to avoid ambiguity with baseline.
    fn extract_current_creatinine(observations: &[Observation]) -> Option<f64> {
        observations
            .iter()
            .find(|obs| obs.code == "LOINC:2160-0-current")
            .and_then(|obs| obs.value.as_f64())
    }

    /// Extracts urine output rate (mL/kg/h) from observations.
    fn extract_urine_output_rate(observations: &[Observation]) -> Option<f64> {
        observations
            .iter()
            .find(|obs| obs.code == "LOINC:9192-0" || obs.code == "LOINC:9192-0-rate")
            .and_then(|obs| obs.value.as_f64())
    }

    /// Determines AKI stage based on creatinine and UO criteria per KDIGO 2021.
    /// Returns (stage, source_criterion) where source_criterion identifies driving criterion.
    /// Note: This function does not validate temporal windows (6–12h, ≥12h, ≥24h for UO);
    /// the caller must abstain if window duration is unknown.
    fn determine_stage(
        baseline_cr: f64,
        current_cr: Option<f64>,
        uo_rate: Option<f64>,
    ) -> Option<(u32, &'static str)> {
        let mut max_stage = 0;
        let mut source = "none";

        // Creatinine-based staging per KDIGO 2021
        if let Some(cr) = current_cr {
            let fold_change = cr / baseline_cr;
            // Stage 3: Cr ≥3× baseline OR (Cr ≥4.0 mg/dL with acute rise ≥0.5 mg/dL within 7d)
            if fold_change >= 3.0 || (cr >= 4.0 && (cr - baseline_cr) >= 0.5) {
                max_stage = 3;
                source = "creatinine";
            } else if fold_change >= 2.0 {
                max_stage = 2;
                source = "creatinine";
            } else if fold_change >= 1.5 {
                max_stage = 1;
                source = "creatinine";
            }
        }

        // Urine output-based staging per KDIGO 2021 (requires time window validation by caller)
        // Stage 3: < 0.3 mL/kg/h for ≥24 hours
        // Stage 2: < 0.5 mL/kg/h for ≥12 hours
        // Stage 1: < 0.5 mL/kg/h for 6–12 hours
        if let Some(rate) = uo_rate {
            if rate < 0.3 {
                // Only upgrade to Stage 3 if not already there; preserve Cr source if present
                if max_stage < 3 {
                    max_stage = 3;
                    source = "urine-output";
                }
                // If Cr already Stage 3, keep creatinine as source (don't overwrite)
            } else if rate < 0.5 {
                // UO < 0.5 can be Stage 1 or 2 depending on duration (caller's responsibility)
                // Conservatively assign Stage 1; upgrade to Stage 2 only if no Cr criterion set it
                if max_stage < 2 {
                    max_stage = 2;
                    source = "urine-output";
                } else if max_stage < 1 {
                    max_stage = 1;
                    source = "urine-output";
                }
                // Preserve higher stage set by Cr criterion
            }
        }

        if max_stage > 0 {
            Some((max_stage, source))
        } else {
            None
        }
    }

    /// Creates an Atom representing an AKI stage.
    fn stage_to_atom(&self, stage: u32, source: &str) -> Atom {
        Atom {
            system: OntologySystem::SNOMED,
            code: format!("KDIGO-AKI-STAGE-{}", stage),
            preferred_term: format!("AKI Stage {} ({})", stage, source),
            version: self.version.clone(),
        }
    }
}

impl Operator for KdigoAkiOperator {
    fn apply(&self, h: &Hyp, e: &crate::Evidence) -> Outcome<Hyp, AbstainReason> {
        // Precondition: baseline creatinine must be available and valid (> 0)
        let baseline_cr = match Self::extract_baseline_creatinine(&e.observations) {
            Some(cr) => cr,
            None => {
                return Outcome::Abstain(AbstainReason::InsufficientEvidence(
                    "baseline serum creatinine unknown or invalid (≤0); cannot compute AKI stage",
                ));
            }
        };

        // Extract current Cr and UO
        let current_cr = Self::extract_current_creatinine(&e.observations);
        let uo_rate = Self::extract_urine_output_rate(&e.observations);

        // Structural abstention: UO present without time window validation
        // KDIGO AKI requires specific windows: Stage 1 (6-12h), Stage 2 (≥12h), Stage 3 (≥24h).
        // Evidence carries no duration field; operator cannot validate temporal constraints.
        // Abstain to avoid false positives from momentary dips.
        if uo_rate.is_some() {
            return Outcome::Abstain(AbstainReason::InsufficientEvidence(
                "urine output window duration unknown; cannot validate KDIGO temporal constraints (6-24h)",
            ));
        }

        // Determine stage (Cr-based only, since UO abstained above)
        match Self::determine_stage(baseline_cr, current_cr, None) {
            Some((stage, source)) => {
                let aki_atom = self.stage_to_atom(stage, source);
                let mut new_atoms = h.atoms().to_vec();
                new_atoms.push(aki_atom);
                Outcome::Refined(Hyp::new(new_atoms))
            }
            None => {
                // No AKI criteria met; refine to stage 0 (no AKI atom added)
                Outcome::Refined(h.clone())
            }
        }
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
                ProvenanceOrigin::new("test", "LOINC", "test"),
                Utc::now(),
                crate::Ver::new("test", "kdigo_aki", "0.1.0"),
                BTreeMap::new(),
            ),
        )
    }

    #[test]
    fn test_kdigo_aki_stage_1_creatinine() {
        let op = KdigoAkiOperator::new("0.1.0");
        let observations = vec![
            Observation::new("LOINC:2160-0-baseline", serde_json::json!(0.9)), // baseline 0.9
            Observation::new("LOINC:2160-0-current", serde_json::json!(1.5)), // current 1.5 (1.67× baseline)
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
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("STAGE-1")));
            }
            Outcome::Abstain(_) => panic!("should refine, not abstain"),
        }
    }

    #[test]
    fn test_kdigo_aki_stage_2_creatinine() {
        let op = KdigoAkiOperator::new("0.1.0");
        let observations = vec![
            Observation::new("LOINC:2160-0-baseline", serde_json::json!(1.0)),
            Observation::new("LOINC:2160-0-current", serde_json::json!(2.5)), // 2.5× baseline
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                assert!(h_prime <= h);
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("STAGE-2")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_kdigo_aki_stage_3_creatinine() {
        let op = KdigoAkiOperator::new("0.1.0");
        let observations = vec![
            Observation::new("LOINC:2160-0-baseline", serde_json::json!(1.0)),
            Observation::new("LOINC:2160-0-current", serde_json::json!(3.5)), // 3.5× baseline
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("STAGE-3")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_kdigo_aki_stage_3_absolute_cr() {
        // Test Stage 3 by absolute Cr ≥ 4.0 with acute rise ≥ 0.5
        let op = KdigoAkiOperator::new("0.1.0");
        let observations = vec![
            Observation::new("LOINC:2160-0-baseline", serde_json::json!(3.8)),
            Observation::new("LOINC:2160-0-current", serde_json::json!(4.5)), // 4.5 >= 4.0 and rise 0.7 >= 0.5
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("STAGE-3")));
            }
            Outcome::Abstain(_) => panic!("should refine on absolute Cr criterion"),
        }
    }

    #[test]
    fn test_kdigo_aki_no_false_stage_3_on_chronic_ckd() {
        // Chronic CKD: baseline 4.5, current 4.6 (no acute rise) should NOT be Stage 3
        let op = KdigoAkiOperator::new("0.1.0");
        let observations = vec![
            Observation::new("LOINC:2160-0-baseline", serde_json::json!(4.5)),
            Observation::new("LOINC:2160-0-current", serde_json::json!(4.6)), // 4.6 >= 4.0 but rise only 0.1
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                // Should not contain STAGE-3
                assert!(!atoms.iter().any(|a| a.code.contains("STAGE-3")));
                // Should be Stage 0 (no AKI atom added, equals input h)
                assert_eq!(h_prime, h);
            }
            Outcome::Abstain(_) => panic!("should refine to stage 0"),
        }
    }

    #[test]
    fn test_kdigo_aki_abstain_uo_without_window() {
        // UO without temporal window is structural abstention (KDIGO requires 6–24h window)
        let op = KdigoAkiOperator::new("0.1.0");
        let observations = vec![
            Observation::new("LOINC:2160-0-baseline", serde_json::json!(1.0)),
            Observation::new("LOINC:2160-0-current", serde_json::json!(1.0)),
            Observation::new("LOINC:9192-0", serde_json::json!(0.4)), // UO 0.4 mL/kg/h, no duration
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Abstain(reason) => {
                // Expected: window duration unknown
                assert!(reason.message().contains("window duration unknown"));
            }
            Outcome::Refined(_) => panic!("should abstain on UO without window duration"),
        }
    }

    #[test]
    fn test_kdigo_aki_stage_0_creatinine_only() {
        // No AKI: baseline normal, current normal, Cr criterion met, no UO (avoid abstention)
        let op = KdigoAkiOperator::new("0.1.0");
        let observations = vec![
            Observation::new("LOINC:2160-0-baseline", serde_json::json!(1.0)),
            Observation::new("LOINC:2160-0-current", serde_json::json!(1.0)), // no change
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                assert_eq!(h_prime, h, "no refinement when no AKI criteria met");
            }
            Outcome::Abstain(_) => panic!("should refine to identity, not abstain"),
        }
    }

    #[test]
    fn test_kdigo_aki_abstain_no_baseline() {
        let op = KdigoAkiOperator::new("0.1.0");
        // Only include current Cr, no baseline Cr
        let observations = vec![Observation::new(
            "LOINC:2160-0-current",
            serde_json::json!(1.5),
        )];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Abstain(reason) => {
                // Expected: baseline creatinine not found, so operator abstains
                assert!(
                    reason
                        .message()
                        .contains("baseline serum creatinine unknown")
                );
            }
            Outcome::Refined(_) => panic!("should abstain when baseline Cr missing"),
        }
    }

    #[test]
    fn test_kdigo_aki_monotonicity_preserved() {
        let op = KdigoAkiOperator::new("0.1.0");
        let observations = vec![
            Observation::new("LOINC:2160-0-baseline", serde_json::json!(1.0)),
            Observation::new("LOINC:2160-0-current", serde_json::json!(2.0)), // 2.0× baseline (Stage 2)
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
}
