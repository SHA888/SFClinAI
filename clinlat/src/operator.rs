//! Deduction operators on hypotheses.
//!
//! Implements DEF-PS-07 (Operator trait interface).

use serde::{Deserialize, Serialize};

use crate::{AbstainReason, Hyp, Outcome, Provenance};

/// A single clinical observation (lab value, vital sign, finding, etc.).
///
/// Implements DEF-MP-10 (Observation type). Each observation is a code-value pair
/// with optional contextual metadata (unit, source system).
///
/// # Examples
/// - Lab value: code="LOINC:2160-0" (glucose), value=98, unit="mg/dL"
/// - Vital sign: code="SNOMED:6797001" (systolic BP), value=120, unit="mmHg"
/// - Clinical finding: code="SNOMED:67822003" (hypoxemia), value=true
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// Ontology code: "LOINC:2160-0", "SNOMED:67822003", etc.
    pub code: String,

    /// The observed value (number, string, boolean, null, or array).
    /// Uses serde_json::Value for flexibility across different observation types.
    pub value: serde_json::Value,

    /// Optional unit of measurement (e.g., "mg/dL", "mmHg", "%").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Optional source system (e.g., "Epic LIS", "Cerner", "manual_entry").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl Observation {
    /// Create a new observation.
    pub fn new(code: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            code: code.into(),
            value,
            unit: None,
            source: None,
        }
    }

    /// Set the unit of measurement.
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set the source system.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// Evidence: a packet of clinical observations with provenance.
///
/// Implements DEF-MP-11 (Evidence type per SPEC.md §2.4).
/// Carries a collection of observations (lab values, vitals, findings)
/// and their provenance (source, timestamp, version).
///
/// Evidence is immutable once created; provenance cannot be changed post-hoc
/// to preserve audit trail integrity (OBL-PS-04).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    /// Collection of clinical observations (lab values, vitals, findings).
    pub observations: Vec<Observation>,

    /// Provenance: tracks source, timestamp, version for audit trail fidelity.
    pub provenance: Provenance,
}

impl Evidence {
    /// Create new evidence from observations and provenance.
    pub fn new(observations: Vec<Observation>, provenance: Provenance) -> Self {
        Self {
            observations,
            provenance,
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

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
    use std::collections::BTreeMap;

    use super::*;
    use chrono::Utc;

    fn test_observation() -> Observation {
        Observation::new("LOINC:2160-0", serde_json::json!(98.0))
            .with_unit("mg/dL")
            .with_source("Epic LIS")
    }

    fn test_provenance() -> Provenance {
        use crate::ProvenanceOrigin;
        let origin = ProvenanceOrigin::new("external_lab_api", "LOINC", "2160-0");
        let mut metadata = BTreeMap::new();
        metadata.insert("lab_system".to_string(), serde_json::json!("epic_lis"));
        Provenance::new(
            origin,
            Utc::now(),
            crate::Ver::new("clinlat", "lab_ingest", "0.1.0"),
            metadata,
        )
    }

    #[test]
    fn test_observation_creation() {
        let obs = test_observation();
        assert_eq!(obs.code, "LOINC:2160-0");
        assert_eq!(obs.value, serde_json::json!(98.0));
        assert_eq!(obs.unit, Some("mg/dL".to_string()));
        assert_eq!(obs.source, Some("Epic LIS".to_string()));
    }

    #[test]
    fn test_observation_without_unit_or_source() {
        let obs = Observation::new("SNOMED:67822003", serde_json::json!(true));
        assert_eq!(obs.code, "SNOMED:67822003");
        assert_eq!(obs.unit, None);
        assert_eq!(obs.source, None);
    }

    #[test]
    fn test_evidence_creation() {
        let observations = vec![test_observation()];
        let provenance = test_provenance();
        let evidence = Evidence::new(observations, provenance);

        assert_eq!(evidence.observations.len(), 1);
        assert_eq!(evidence.observations[0].code, "LOINC:2160-0");
        assert!(!evidence.provenance.metadata.is_empty());
    }

    #[test]
    fn test_evidence_multiple_observations() {
        let observations = vec![
            Observation::new("LOINC:2160-0", serde_json::json!(98.0)).with_unit("mg/dL"),
            Observation::new("SNOMED:6797001", serde_json::json!(120)).with_unit("mmHg"),
        ];
        let evidence = Evidence::new(observations, test_provenance());

        assert_eq!(evidence.observations.len(), 2);
        assert_eq!(evidence.observations[0].code, "LOINC:2160-0");
        assert_eq!(evidence.observations[1].code, "SNOMED:6797001");
    }

    #[test]
    fn test_evidence_json_serialization() {
        let evidence = Evidence::new(vec![test_observation()], test_provenance());
        let json = evidence.to_json().expect("serialization failed");
        assert!(json.contains("\"code\":\"LOINC:2160-0\""));
        assert!(json.contains("\"observations\""));
        assert!(json.contains("\"provenance\""));
    }

    #[test]
    fn test_evidence_json_round_trip() {
        let original = Evidence::new(vec![test_observation()], test_provenance());
        let json = original.to_json().expect("serialization failed");
        let restored = Evidence::from_json(&json).expect("deserialization failed");

        assert_eq!(original.observations.len(), restored.observations.len());
        assert_eq!(original.observations[0].code, restored.observations[0].code);
        assert_eq!(original.provenance.origin, restored.provenance.origin);
    }

    #[test]
    fn test_observation_with_array_value() {
        let obs = Observation::new("CUSTOM:array_test", serde_json::json!([1, 2, 3]));
        assert_eq!(obs.value, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_observation_with_string_value() {
        let obs = Observation::new("CUSTOM:text_test", serde_json::json!("clinical finding"));
        assert_eq!(obs.value, serde_json::json!("clinical finding"));
    }
}
