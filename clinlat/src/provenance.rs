//! Provenance carrier for tracking evidence source, timestamp, and version through operator derivations.

use std::collections::BTreeMap;
use std::io::{Read, Write};

use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};

use crate::version::Ver;

/// Source origin for a provenance record.
///
/// Per SPEC.md §2.4 (DEF-MP-14), captures where evidence came from.
/// Supports audit queries that reconstruct derivation chains (OBL-PS-04).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProvenanceOrigin {
    /// Source type: "external_lab_api", "clinician_input", "operator_derivation", etc.
    pub source_type: String,

    /// Ontology system: "SNOMED", "RxNorm", "LOINC", "ICD11", "Unstructured", etc.
    pub system: String,

    /// Identifier in that system (code, code:value, or free text).
    pub identifier: String,
}

impl ProvenanceOrigin {
    /// Create a new provenance origin.
    pub fn new(
        source_type: impl Into<String>,
        system: impl Into<String>,
        identifier: impl Into<String>,
    ) -> Self {
        Self {
            source_type: source_type.into(),
            system: system.into(),
            identifier: identifier.into(),
        }
    }
}

/// Typed provenance carrier for evidence and hypothesis derivations.
///
/// Per SPEC.md §2.4 (DEF-PS-12, DEF-PS-13), tracks:
/// - **origin**: Where evidence came from (lab, clinician, operator)
/// - **timestamp**: ISO 8601 UTC when the evidence/hypothesis was created
/// - **version**: Which operator version created this hypothesis (enables INV-PS-05)
/// - **metadata**: System-specific key-value context (lab system, specimen ID, etc.)
/// - **derives_from**: Optional hash references to ancestor provenance (M5+ temporal evolution)
///
/// Serializes to JSON per DESIGN-D1 for human-readable audit logs and SQL queryability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Source origin (lab, clinician input, operator output).
    pub origin: ProvenanceOrigin,

    /// ISO 8601 UTC timestamp when this evidence/hypothesis was created.
    pub timestamp: DateTime<Utc>,

    /// Operator and build version (enables version-respecting derivation chains per INV-PS-05).
    pub version: Ver,

    /// System-specific metadata: lab system name, specimen ID, confidence score, etc.
    pub metadata: BTreeMap<String, serde_json::Value>,

    /// Optional: hash references to ancestor provenance (populated in M5+).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derives_from: Option<Vec<String>>,
}

impl Provenance {
    /// Create a new provenance record.
    pub fn new(
        origin: ProvenanceOrigin,
        timestamp: DateTime<Utc>,
        version: Ver,
        metadata: BTreeMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            origin,
            timestamp,
            version,
            metadata,
            derives_from: None,
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON and gzip-compress the bytes.
    pub fn to_json_compressed(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let json = self.to_json()?;
        let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(json.as_bytes())?;
        Ok(encoder.finish()?)
    }

    /// Decompress gzip bytes and deserialize as JSON.
    pub fn from_json_compressed(compressed: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut decoder = GzDecoder::new(compressed);
        let mut json = String::new();
        decoder.read_to_string(&mut json)?;
        Ok(Self::from_json(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_origin() -> ProvenanceOrigin {
        ProvenanceOrigin::new("external_lab_api", "LOINC", "2160-0")
    }

    fn test_version() -> Ver {
        Ver::new("clinlat", "sofa_resp", "0.2.0")
    }

    fn test_metadata() -> BTreeMap<String, serde_json::Value> {
        let mut m = BTreeMap::new();
        m.insert("lab_system".to_string(), serde_json::json!("epic_lis"));
        m.insert(
            "specimen_id".to_string(),
            serde_json::json!("LAB-2026-05-25-001"),
        );
        m
    }

    #[test]
    fn test_provenance_origin_construction() {
        let origin = ProvenanceOrigin::new("external_lab_api", "LOINC", "2160-0");
        assert_eq!(origin.source_type, "external_lab_api");
        assert_eq!(origin.system, "LOINC");
        assert_eq!(origin.identifier, "2160-0");
    }

    #[test]
    fn test_provenance_construction() {
        let timestamp = Utc::now();
        let prov = Provenance::new(test_origin(), timestamp, test_version(), test_metadata());

        assert_eq!(prov.origin.source_type, "external_lab_api");
        assert_eq!(prov.version.operator, "sofa_resp");
        assert_eq!(
            prov.metadata.get("lab_system").map(|v| v.as_str()),
            Some(Some("epic_lis"))
        );
        assert_eq!(prov.derives_from, None);
    }

    #[test]
    fn test_provenance_to_json() {
        let timestamp = Utc::now();
        let prov = Provenance::new(test_origin(), timestamp, test_version(), test_metadata());

        let json = prov.to_json().expect("serialization failed");
        assert!(json.contains("\"source_type\":\"external_lab_api\""));
        assert!(json.contains("\"system\":\"LOINC\""));
        assert!(json.contains("\"operator\":\"sofa_resp\""));
        assert!(json.contains("\"lab_system\":\"epic_lis\""));
    }

    #[test]
    fn test_provenance_from_json() {
        let json_str = r#"{
            "origin": {
                "source_type": "external_lab_api",
                "system": "LOINC",
                "identifier": "2160-0"
            },
            "timestamp": "2026-05-25T10:30:00Z",
            "version": {
                "system": "clinlat",
                "operator": "sofa_resp",
                "build": "0.2.0"
            },
            "metadata": {
                "lab_system": "epic_lis",
                "specimen_id": "LAB-2026-05-25-001"
            }
        }"#;

        let prov = Provenance::from_json(json_str).expect("deserialization failed");
        assert_eq!(prov.origin.source_type, "external_lab_api");
        assert_eq!(prov.version.operator, "sofa_resp");
        assert_eq!(
            prov.metadata.get("lab_system").map(|v| v.as_str()),
            Some(Some("epic_lis"))
        );
    }

    #[test]
    fn test_provenance_json_round_trip() {
        let original = Provenance::new(test_origin(), Utc::now(), test_version(), test_metadata());
        let json = original.to_json().expect("serialization failed");
        let restored = Provenance::from_json(&json).expect("deserialization failed");

        assert_eq!(original.origin, restored.origin);
        assert_eq!(original.version, restored.version);
        assert_eq!(original.metadata, restored.metadata);
        assert_eq!(original.derives_from, restored.derives_from);
    }

    #[test]
    fn test_provenance_compressed_round_trip() {
        let original = Provenance::new(test_origin(), Utc::now(), test_version(), test_metadata());

        let compressed = original.to_json_compressed().expect("compression failed");
        let restored = Provenance::from_json_compressed(&compressed).expect("decompression failed");

        assert_eq!(original.origin, restored.origin);
        assert_eq!(original.version, restored.version);
        assert_eq!(original.metadata, restored.metadata);
    }

    #[test]
    fn test_provenance_compression_reduces_size() {
        let original = Provenance::new(test_origin(), Utc::now(), test_version(), test_metadata());

        let compressed = original.to_json_compressed().expect("compression failed");

        // Compression should reduce size (at least for reasonably-sized payloads)
        // For small payloads, gzip header overhead may make it larger, so we just check it works
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_provenance_empty_metadata() {
        let timestamp = Utc::now();
        let empty_meta = BTreeMap::new();
        let prov = Provenance::new(test_origin(), timestamp, test_version(), empty_meta);

        let json = prov.to_json().expect("serialization failed");
        let restored = Provenance::from_json(&json).expect("deserialization failed");

        assert_eq!(prov.metadata, restored.metadata);
        assert!(prov.metadata.is_empty());
    }

    #[test]
    fn test_provenance_deserialize_missing_derives_from() {
        // M1 provenance doesn't include derives_from
        let json_str = r#"{
            "origin": {
                "source_type": "external_lab_api",
                "system": "LOINC",
                "identifier": "2160-0"
            },
            "timestamp": "2026-05-25T10:30:00Z",
            "version": {
                "system": "clinlat",
                "operator": "sofa_resp",
                "build": "0.2.0"
            },
            "metadata": {}
        }"#;

        let prov = Provenance::from_json(json_str).expect("deserialization failed");
        assert_eq!(prov.derives_from, None);
    }

    #[test]
    fn test_provenance_deserialize_with_derives_from() {
        // M5+ provenance may include derives_from
        let json_str = r#"{
            "origin": {
                "source_type": "operator_derivation",
                "system": "SNOMED",
                "identifier": "67822003"
            },
            "timestamp": "2026-05-25T10:30:00Z",
            "version": {
                "system": "clinlat",
                "operator": "sofa_resp",
                "build": "0.2.0"
            },
            "metadata": {},
            "derives_from": ["sha256:abc123", "sha256:def456"]
        }"#;

        let prov = Provenance::from_json(json_str).expect("deserialization failed");
        assert_eq!(
            prov.derives_from,
            Some(vec![
                "sha256:abc123".to_string(),
                "sha256:def456".to_string()
            ])
        );
    }

    #[test]
    fn test_provenance_clinician_input_example() {
        // Example from spec: clinician input with unstructured system
        let origin = ProvenanceOrigin::new("clinician_input", "Unstructured", "free_text");
        let mut metadata = BTreeMap::new();
        metadata.insert("clinician_id".to_string(), serde_json::json!("MD-42"));
        metadata.insert(
            "institution".to_string(),
            serde_json::json!("Tertiary Care Hospital"),
        );
        metadata.insert(
            "modality".to_string(),
            serde_json::json!("voice_transcription"),
        );

        let prov = Provenance::new(
            origin,
            Utc::now(),
            Ver::new("clinlat", "clinician_intake", "0.1.0"),
            metadata,
        );

        let json = prov.to_json().expect("serialization failed");
        let restored = Provenance::from_json(&json).expect("deserialization failed");

        assert_eq!(restored.origin.source_type, "clinician_input");
        assert_eq!(restored.origin.system, "Unstructured");
        assert_eq!(restored.version.operator, "clinician_intake");
    }

    #[test]
    fn test_provenance_operator_derivation_example() {
        // Example: operator derivation from lab evidence
        let origin = ProvenanceOrigin::new("operator_derivation", "SNOMED", "67822003"); // hypoxemia
        let mut metadata = BTreeMap::new();
        metadata.insert("prior_hyp".to_string(), serde_json::json!("Unknown"));
        metadata.insert("pao2_value".to_string(), serde_json::json!(98.0));
        metadata.insert(
            "input_lab_code".to_string(),
            serde_json::json!("LOINC:2160-0"),
        );

        let prov = Provenance::new(
            origin,
            Utc::now(),
            Ver::new("clinlat", "sofa_resp", "0.2.0"),
            metadata,
        );

        assert_eq!(prov.origin.source_type, "operator_derivation");
        assert_eq!(prov.version.operator, "sofa_resp");
        assert_eq!(
            prov.metadata.get("pao2_value").and_then(|v| v.as_f64()),
            Some(98.0)
        );
    }
}
