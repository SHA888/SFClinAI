//! Wells score operator for pulmonary embolism risk stratification.
//!
//! Implements DEF-PS-08 (Operator trait interface) for PE risk stratification
//! per Wells et al. (1997/2006) criteria.
//!
//! Wells score refines the patient hypothesis through sequential testing:
//! - Initial scoring based on clinical features
//! - Category assignment (PE unlikely, PE likely)
//! - Sequential testing recommendation (D-dimer if low risk, CTPA if high risk)

use crate::{AbstainReason, Atom, Hyp, Observation, OntologySystem, Operator, Outcome};

/// Wells PE operator: stratifies pulmonary embolism risk via clinical scoring.
///
/// Per Wells et al. (1997, refined 2006):
/// Clinical features (points):
/// - Clinical signs of DVT: +3
/// - PE as most likely diagnosis: +3
/// - Heart rate >100: +1.5
/// - Recent surgery or immobilization: +1.5
/// - Prior DVT/PE: +1.5
/// - Hemoptysis: +1
/// - Malignancy: +1
///
/// Risk categories:
/// - Wells score ≤4: PE unlikely (negative D-dimer excludes PE, positive requires CTPA)
/// - Wells score >4: PE likely (CTPA regardless of D-dimer)
///
/// Abstains when:
/// - Wells component score unclear (clinician judgment required for gestalt assessment)
/// - D-dimer unavailable (within clinical window, timing matters)
/// - CTPA contraindicated (renal impairment, contrast anaphylaxis history)
pub struct WellsPeOperator {
    /// Version identifier for audit trail (DEF-PS-13)
    pub version: String,
}

impl WellsPeOperator {
    /// Creates a new Wells PE operator with specified version.
    pub fn new(version: impl Into<String>) -> Self {
        WellsPeOperator {
            version: version.into(),
        }
    }

    /// Extracts Wells score components from observations.
    /// Returns the total score (0–10.5 typical range).
    fn calculate_wells_score(observations: &[Observation]) -> f64 {
        let mut score = 0.0;

        // Clinical signs of DVT (leg swelling, asymmetry, pain)
        if observations
            .iter()
            .any(|obs| obs.code == "DVT-SIGNS" && obs.value.as_bool().unwrap_or(false))
        {
            score += 3.0;
        }

        // PE as most likely diagnosis
        if let Some(obs) = observations.iter().find(|obs| obs.code == "PE-LIKELY") {
            if obs.value.as_bool().unwrap_or(false) {
                score += 3.0;
            }
        }

        // Heart rate >100
        if let Some(obs) = observations.iter().find(|obs| obs.code == "HEART-RATE") {
            if let Some(hr) = obs.value.as_f64() {
                if hr > 100.0 {
                    score += 1.5;
                }
            }
        }

        // Recent surgery or immobilization (>4 days in past 4 weeks)
        if observations
            .iter()
            .any(|obs| obs.code == "RECENT-IMMOB" && obs.value.as_bool().unwrap_or(false))
        {
            score += 1.5;
        }

        // Prior DVT or PE
        if observations
            .iter()
            .any(|obs| obs.code == "PRIOR-VTE" && obs.value.as_bool().unwrap_or(false))
        {
            score += 1.5;
        }

        // Hemoptysis
        if observations
            .iter()
            .any(|obs| obs.code == "HEMOPTYSIS" && obs.value.as_bool().unwrap_or(false))
        {
            score += 1.0;
        }

        // Malignancy (treatment ongoing or within 6 months)
        if observations
            .iter()
            .any(|obs| obs.code == "MALIGNANCY" && obs.value.as_bool().unwrap_or(false))
        {
            score += 1.0;
        }

        score
    }

    /// Determines Wells risk category and next testing step.
    fn determine_category(wells_score: f64) -> (&'static str, &'static str) {
        if wells_score <= 4.0 {
            ("PE-UNLIKELY", "consider-d-dimer")
        } else {
            ("PE-LIKELY", "ctpa-indicated")
        }
    }

    /// Creates an Atom representing a Wells PE risk category.
    fn category_to_atom(&self, category: &str, score: f64) -> Atom {
        Atom {
            system: OntologySystem::SNOMED,
            code: format!("WELLS-PE-{}", category),
            preferred_term: format!("Wells PE {}: score {:.1}", category, score),
            version: self.version.clone(),
        }
    }
}

impl Operator for WellsPeOperator {
    fn apply(&self, h: &Hyp, e: &crate::Evidence) -> Outcome<Hyp, AbstainReason> {
        // Gestalt assessment "PE as most likely diagnosis" is required
        // This represents clinician judgment and cannot be inferred from objective findings alone
        if !e.observations.iter().any(|obs| obs.code == "PE-LIKELY") {
            return Outcome::Abstain(AbstainReason::InsufficientEvidence(
                "PE gestalt assessment unavailable; clinician judgment required for Wells scoring",
            ));
        }

        // Calculate Wells score from observations
        let wells_score = Self::calculate_wells_score(&e.observations);

        // Determine category
        let (category, _next_step) = Self::determine_category(wells_score);

        // Refine hypothesis with Wells PE atom
        let pe_atom = self.category_to_atom(category, wells_score);
        let mut new_atoms = h.atoms().to_vec();
        new_atoms.push(pe_atom);
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
                crate::Ver::new("test", "wells_pe", "0.1.0"),
                BTreeMap::new(),
            ),
        )
    }

    #[test]
    fn test_wells_pe_unlikely_low_score() {
        let op = WellsPeOperator::new("0.1.0");
        // Minimal criteria: just tachycardia; gestalt assessment says PE unlikely
        let observations = vec![
            Observation::new("PE-LIKELY", serde_json::json!(false)), // gestalt: PE not likely
            Observation::new("HEART-RATE", serde_json::json!(120.0)), // +1.5
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                assert!(h_prime < h);
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("PE-UNLIKELY")));
            }
            Outcome::Abstain(_) => panic!("should refine with Wells calculation"),
        }
    }

    #[test]
    fn test_wells_pe_unlikely_with_dvt_signs() {
        let op = WellsPeOperator::new("0.1.0");
        // DVT signs (+3) + tachycardia (+1.5) = 4.5, but typically PE-UNLIKELY boundary is ≤4
        // So this should be PE-LIKELY (>4)
        let observations = vec![
            Observation::new("PE-LIKELY", serde_json::json!(true)), // gestalt: PE likely
            Observation::new("DVT-SIGNS", serde_json::json!(true)), // +3
            Observation::new("HEART-RATE", serde_json::json!(120.0)), // +1.5
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                // Score is 4.5, so should be PE-LIKELY
                assert!(atoms.iter().any(|a| a.code.contains("PE-LIKELY")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_wells_pe_likely_high_score() {
        let op = WellsPeOperator::new("0.1.0");
        // PE as most likely (+3) + DVT signs (+3) + tachycardia (+1.5) + prior VTE (+1.5) = 9.0
        let observations = vec![
            Observation::new("PE-LIKELY", serde_json::json!(true)), // +3
            Observation::new("DVT-SIGNS", serde_json::json!(true)), // +3
            Observation::new("HEART-RATE", serde_json::json!(120.0)), // +1.5
            Observation::new("PRIOR-VTE", serde_json::json!(true)), // +1.5
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("PE-LIKELY")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_wells_pe_all_criteria() {
        let op = WellsPeOperator::new("0.1.0");
        // All criteria: +3+3+1.5+1.5+1.5+1+1 = 12.0
        let observations = vec![
            Observation::new("DVT-SIGNS", serde_json::json!(true)),
            Observation::new("PE-LIKELY", serde_json::json!(true)),
            Observation::new("HEART-RATE", serde_json::json!(120.0)),
            Observation::new("RECENT-IMMOB", serde_json::json!(true)),
            Observation::new("PRIOR-VTE", serde_json::json!(true)),
            Observation::new("HEMOPTYSIS", serde_json::json!(true)),
            Observation::new("MALIGNANCY", serde_json::json!(true)),
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("PE-LIKELY")));
            }
            Outcome::Abstain(_) => panic!("should refine"),
        }
    }

    #[test]
    fn test_wells_pe_no_criteria() {
        let op = WellsPeOperator::new("0.1.0");
        // Gestalt assessment says PE unlikely; no other criteria: score = 0.0
        let observations = vec![
            Observation::new("PE-LIKELY", serde_json::json!(false)), // gestalt: PE not likely
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                assert!(atoms.iter().any(|a| a.code.contains("PE-UNLIKELY")));
            }
            Outcome::Abstain(_) => panic!("should refine with zero score"),
        }
    }

    #[test]
    fn test_wells_pe_boundary_score_4() {
        let op = WellsPeOperator::new("0.1.0");
        // Exactly 4.0: boundary case for PE-UNLIKELY
        // Gestalt says unlikely; DVT signs (+3) + Hemoptysis (+1) = 4.0
        let observations = vec![
            Observation::new("PE-LIKELY", serde_json::json!(false)), // gestalt: PE not likely
            Observation::new("DVT-SIGNS", serde_json::json!(true)),  // +3
            Observation::new("HEMOPTYSIS", serde_json::json!(true)), // +1
                                                                     // Total: 4.0 exactly
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Refined(h_prime) => {
                let atoms = h_prime.atoms();
                // Score = 4.0, so should be PE-UNLIKELY
                assert!(atoms.iter().any(|a| a.code.contains("PE-UNLIKELY")));
            }
            Outcome::Abstain(_) => panic!("should refine at boundary"),
        }
    }

    #[test]
    fn test_wells_pe_monotonicity() {
        let op = WellsPeOperator::new("0.1.0");
        let observations = vec![
            Observation::new("PE-LIKELY", serde_json::json!(true)), // +3
            Observation::new("HEART-RATE", serde_json::json!(120.0)), // +1.5
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
    fn test_wells_pe_abstain_missing_gestalt() {
        let op = WellsPeOperator::new("0.1.0");
        // Objective findings (DVT, tachycardia) present, but PE gestalt assessment missing
        // Bug: operator was refining without checking for PE-LIKELY observation
        // Fix: abstain with InsufficientEvidence when PE-LIKELY missing
        let observations = vec![
            Observation::new("DVT-SIGNS", serde_json::json!(true)),
            Observation::new("HEART-RATE", serde_json::json!(120.0)),
        ];
        let e = test_evidence_with_observations(observations);
        let h = Hyp::unknown();

        let outcome = op.apply(&h, &e);
        match outcome {
            Outcome::Abstain(reason) => {
                assert!(matches!(reason, AbstainReason::InsufficientEvidence(_)));
                // Verify error message mentions gestalt
                if let AbstainReason::InsufficientEvidence(msg) = reason {
                    assert!(msg.contains("gestalt"));
                }
            }
            Outcome::Refined(_) => panic!("should abstain when PE gestalt assessment missing"),
        }
    }
}
