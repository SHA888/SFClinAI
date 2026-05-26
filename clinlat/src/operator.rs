//! Deduction operators on hypotheses.
//!
//! Implements DEF-PS-07 (Operator trait interface).

use serde::{Deserialize, Serialize};

use crate::{AbstainReason, Atom, Hyp, OntologySystem, Outcome, Provenance};

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

/// Extract atoms from evidence observations and map to hypothesis.
///
/// Implements DEF-PS-05 and DEF-PS-06 (α_PS: Obs → Hyp).
/// Maps a set of clinical observations to the most refined hypothesis they entail.
///
/// # Algorithm
///
/// 1. Parse each observation's code in "SYSTEM:CODE" format.
/// 2. Resolve the code to an Atom with system, code, preferred_term, version.
/// 3. Collect all atoms into a Hyp.
///
/// The resulting Hyp is the most refined hypothesis consistent with the observations:
/// the greatest lower bound of atoms that the evidence entails.
///
/// # Behavior
///
/// - If `e.observations` is empty, returns `Hyp::unknown()` (top element, no information).
/// - If all observations fail to parse, returns `Hyp::unknown()`.
/// - Otherwise, returns a `Hyp` containing all successfully parsed atoms.
///
/// # Example
///
/// ```
/// # use clinlat::{Observation, Evidence, Provenance, ProvenanceOrigin};
/// # use chrono::Utc;
/// # use std::collections::BTreeMap;
/// # use clinlat::operator::abstract_evidence;
/// let obs = Observation::new("LOINC:2160-0", serde_json::json!(98.0)).with_unit("mg/dL");
/// let origin = ProvenanceOrigin::new("lab_system", "LOINC", "2160-0");
/// let prov = Provenance::new(
///     origin,
///     Utc::now(),
///     clinlat::Ver::new("clinlat", "lab_ingest", "0.1.0"),
///     BTreeMap::new(),
/// );
/// let evidence = Evidence::new(vec![obs], prov);
/// let hyp = abstract_evidence(&evidence);
/// assert!(!hyp.atoms().is_empty());
/// ```
pub fn abstract_evidence(e: &Evidence) -> Hyp {
    let mut atoms = Vec::new();

    for obs in &e.observations {
        if let Some(atom) = parse_observation_code(&obs.code, &e.provenance) {
            atoms.push(atom);
        }
    }

    if atoms.is_empty() {
        Hyp::unknown()
    } else {
        Hyp::new(atoms)
    }
}

/// Parse observation code "SYSTEM:CODE" into an Atom.
///
/// Extracts system and code from hyphen-separated format and derives
/// version from provenance. Returns None if code format is invalid.
fn parse_observation_code(code: &str, prov: &Provenance) -> Option<Atom> {
    let parts: Vec<&str> = code.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    let system_str = parts[0];
    let code_part = parts[1];

    let system = match system_str {
        "SNOMED" => OntologySystem::SNOMED,
        "LOINC" => OntologySystem::LOINC,
        "RxNorm" => OntologySystem::RxNorm,
        "ICD11" => OntologySystem::ICD11,
        _ => return None,
    };

    let version = prov.version.build.clone();

    Some(Atom {
        system,
        code: code_part.to_string(),
        preferred_term: format!("{} ({})", code_part, system_str),
        version,
    })
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

    #[test]
    fn test_abstract_evidence_single_observation() {
        let observations =
            vec![Observation::new("LOINC:2160-0", serde_json::json!(98.0)).with_unit("mg/dL")];
        let provenance = test_provenance();
        let evidence = Evidence::new(observations, provenance);

        let hyp = abstract_evidence(&evidence);

        assert!(!hyp.atoms().is_empty());
        assert_eq!(hyp.atoms().len(), 1);
        let atom = &hyp.atoms()[0];
        assert_eq!(atom.system, crate::OntologySystem::LOINC);
        assert_eq!(atom.code, "2160-0");
    }

    #[test]
    fn test_abstract_evidence_multiple_observations() {
        let observations = vec![
            Observation::new("LOINC:2160-0", serde_json::json!(98.0)).with_unit("mg/dL"),
            Observation::new("SNOMED:67822003", serde_json::json!(true)),
        ];
        let provenance = test_provenance();
        let evidence = Evidence::new(observations, provenance);

        let hyp = abstract_evidence(&evidence);

        assert_eq!(hyp.atoms().len(), 2);
        let atom1 = &hyp.atoms()[0];
        let atom2 = &hyp.atoms()[1];
        assert_eq!(atom1.system, crate::OntologySystem::LOINC);
        assert_eq!(atom2.system, crate::OntologySystem::SNOMED);
    }

    #[test]
    fn test_abstract_evidence_empty_observations() {
        let observations = vec![];
        let provenance = test_provenance();
        let evidence = Evidence::new(observations, provenance);

        let hyp = abstract_evidence(&evidence);

        assert_eq!(hyp.atoms().len(), 0);
        assert!(hyp == Hyp::unknown());
    }

    #[test]
    fn test_abstract_evidence_invalid_code_format() {
        let observations = vec![
            Observation::new("invalid_code", serde_json::json!(1)),
            Observation::new("LOINC:2160-0", serde_json::json!(98.0)),
        ];
        let provenance = test_provenance();
        let evidence = Evidence::new(observations, provenance);

        let hyp = abstract_evidence(&evidence);

        assert_eq!(hyp.atoms().len(), 1);
        assert_eq!(hyp.atoms()[0].code, "2160-0");
    }

    #[test]
    fn test_abstract_evidence_version_from_provenance() {
        use crate::ProvenanceOrigin;
        let observations = vec![Observation::new("SNOMED:67822003", serde_json::json!(true))];
        let origin = ProvenanceOrigin::new("test_source", "SNOMED", "67822003");
        let prov = Provenance::new(
            origin,
            Utc::now(),
            crate::Ver::new("clinlat", "test_op", "1.2.3"),
            BTreeMap::new(),
        );
        let evidence = Evidence::new(observations, prov);

        let hyp = abstract_evidence(&evidence);

        assert!(!hyp.atoms().is_empty());
        let atom = &hyp.atoms()[0];
        assert_eq!(atom.version, "1.2.3");
    }
}
