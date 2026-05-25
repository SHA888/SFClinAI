//! Ontology adapter infrastructure for binding clinical atoms to external ontology systems.
//!
//! This module provides:
//! - [`OntologySystem`]: Enumeration of supported ontology systems (SNOMED, RxNorm, LOINC, ICD-11).
//! - [`Atom`]: A resolved ontology code with preferred term and version information.
//! - [`OntologyAdapter`]: Trait for resolving codes to atoms and checking compatibility.
//! - [`OntologyError`]: Error type for resolution failures.
//!
//! ## Architecture
//!
//! Per [SPEC.md § 2.3][1] (DEF-PS-03, DEF-PS-04, INV-PS-01, OBL-PS-01):
//!
//! Atoms replace the `&'static str` placeholder from v0.1.0, enabling real ontology binding.
//! Adapters abstract access to SNOMED CT, RxNorm, LOINC, and ICD-11 with a cache-agnostic trait.
//! Concrete implementations (tasks 1.2–1.5) provide in-memory LRU caching with offline snapshot fallback
//! per [DESIGN-D2][2].
//!
//! ## Example
//!
//! ```ignore
//! use clinlat::ontology::{OntologyAdapter, OntologySystem};
//! use std::sync::Arc;
//!
//! // Imagine a SNOMEDAdapter that loads snapshots at startup
//! let snomed: Arc<dyn OntologyAdapter> = Arc::new(SNOMEDAdapter::new("data/snomed-2026-01-31.json.gz")?);
//!
//! // Resolve a SNOMED code to an Atom
//! let atom = snomed.resolve_atom("67822003").await?;  // hypoxemia
//! assert_eq!(atom.preferred_term, "Hypoxemia");
//!
//! // Check compatibility with another atom
//! let other_atom = snomed.resolve_atom("67822003").await?;
//! assert!(snomed.validate_compatibility(&atom, &other_atom));
//! ```
//!
//! [1]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#23-atom-definition
//! [2]: https://github.com/SHA888/SFClinAI/blob/main/DESIGN-D2-ontology.md

use std::fmt;

/// Identifies a supported clinical ontology system.
///
/// Per [SPEC.md § 2.1][1] (DEF-PS-03):
/// The patient-state substrate supports four external ontology systems plus unstructured text.
/// Each system has unique code formats and governance structures.
///
/// [1]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#21-ontology-systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OntologySystem {
    /// SNOMED CT (Systematized Nomenclature of Medicine Clinical Terms).
    /// Maintained by SNOMED International. Used for diagnoses, findings, procedures.
    /// Codes: integer strings (e.g., "67822003" for hypoxemia).
    SNOMED,

    /// RxNorm (National Library of Medicine standardized nomenclature for clinical drugs).
    /// Maintained by NIH NLM. Used for prescriptions, formulary management.
    /// Codes: integer strings (e.g., "1049589" for Lisinopril 10mg).
    RxNorm,

    /// LOINC (Logical Observation Identifiers Names and Codes).
    /// Maintained by Regenstrief Institute. Used for lab tests, vital signs, obstetric observations.
    /// Codes: numeric+dash format (e.g., "2160-0" for creatinine serum).
    LOINC,

    /// ICD-11 (International Classification of Diseases, 11th Revision).
    /// Maintained by WHO. Used for diagnoses at various granularities.
    /// Codes: alphanumeric (e.g., "BA47" for essential hypertension).
    ICD11,

    /// Unstructured free text (fallback for clinical notes, non-coded evidence).
    /// Not resolved against external ontologies; used for clinician input and audit trails.
    Unstructured,
}

impl fmt::Display for OntologySystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OntologySystem::SNOMED => write!(f, "SNOMED"),
            OntologySystem::RxNorm => write!(f, "RxNorm"),
            OntologySystem::LOINC => write!(f, "LOINC"),
            OntologySystem::ICD11 => write!(f, "ICD11"),
            OntologySystem::Unstructured => write!(f, "Unstructured"),
        }
    }
}

/// A resolved clinical concept from an external ontology system.
///
/// Per [SPEC.md § 2.2][1] (DEF-PS-04):
/// An Atom binds a clinical observation or diagnosis to a standardized code,
/// capturing the preferred term and version for audit trail compliance (OBL-PS-01).
///
/// Atoms replace the `&'static str` placeholder from v0.1.0, enabling:
/// - **Interoperability**: Direct mapping to EHR systems using SNOMED/RxNorm/LOINC/ICD-11.
/// - **Auditability**: Each atom carries its ontology version, supporting regulatory traceability.
/// - **Compatibility checking**: Two atoms are compatible iff they encode the same concept in the same version.
///
/// [1]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#22-atom-definition
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Atom {
    /// Which ontology system this atom belongs to.
    pub system: OntologySystem,

    /// Code in that system (e.g., "67822003" for SNOMED hypoxemia, "2160-0" for LOINC creatinine).
    pub code: String,

    /// Preferred term in this version of the ontology (e.g., "Hypoxemia", "Creatinine serum").
    pub preferred_term: String,

    /// Ontology version this atom was resolved from (e.g., "2026-01-31" for SNOMED CT Edition 2026-01-31).
    /// Version string format is system-specific; no semantic parsing is required.
    pub version: String,
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} ({})", self.system, self.code, self.preferred_term)
    }
}

/// Error type for ontology adapter operations.
///
/// Returned by [`OntologyAdapter::resolve_atom`] when resolution fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OntologyError {
    /// Code not found in this ontology system or version.
    CodeNotFound {
        code: String,
        system: OntologySystem,
    },

    /// Network error while resolving from an online API (connectivity loss, API unavailable).
    NetworkError {
        system: OntologySystem,
        description: String,
    },

    /// Code format is invalid for this system (e.g., non-numeric SNOMED code, malformed LOINC).
    InvalidCodeFormat { code: String },

    /// Offline snapshot unavailable (e.g., snapshot file not found or corrupted).
    OfflineSnapshotUnavailable {
        system: OntologySystem,
        description: String,
    },
}

impl fmt::Display for OntologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OntologyError::CodeNotFound { code, system } => {
                write!(f, "Code not found: {} in {}", code, system)
            }
            OntologyError::NetworkError {
                system,
                description,
            } => {
                write!(f, "Network error resolving {}: {}", system, description)
            }
            OntologyError::InvalidCodeFormat { code } => {
                write!(f, "Invalid code format: {}", code)
            }
            OntologyError::OfflineSnapshotUnavailable {
                system,
                description,
            } => {
                write!(
                    f,
                    "Offline snapshot unavailable for {}: {}",
                    system, description
                )
            }
        }
    }
}

impl std::error::Error for OntologyError {}

/// Enumeration of caching modes for ontology adapters.
///
/// Per [DESIGN-D2][1] (M1 caching strategy):
/// Adapters can operate in different modes depending on deployment constraints.
///
/// [1]: https://github.com/SHA888/SFClinAI/blob/main/DESIGN-D2-ontology.md
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    /// Prefer online API calls; fallback to offline snapshot on network failure.
    /// Suitable for well-connected institutional deployments (M5+).
    Online,

    /// Use offline snapshots only; fail if snapshot unavailable.
    /// Suitable for air-gapped or low-connectivity environments.
    Offline,

    /// Use in-memory cache only; no network or filesystem fallback.
    /// Suitable for testing and deterministic deployments (M1–M4 default).
    CacheOnly,
}

/// Trait for resolving ontology codes to atoms.
///
/// Per [SPEC.md § 2.3][1] (DEF-PS-03) and [DESIGN-D2][2]:
///
/// An OntologyAdapter abstracts access to an external ontology system (SNOMED, RxNorm, LOINC, ICD-11).
/// Concrete implementations (SNOMEDAdapter, RxNormAdapter, LoincAdapter, Icd11Adapter) own their caching
/// and fallback strategies (in-memory LRU with offline snapshots in M1–M4; Redis backing in M5+).
///
/// [1]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#23-ontology-systems
/// [2]: https://github.com/SHA888/SFClinAI/blob/main/DESIGN-D2-ontology.md
#[async_trait::async_trait]
pub trait OntologyAdapter: Send + Sync {
    /// Resolve a code in this adapter's ontology to an Atom.
    ///
    /// # Arguments
    /// * `code` - The code to resolve (e.g., "67822003" for SNOMED, "2160-0" for LOINC).
    ///
    /// # Errors
    /// Returns [`OntologyError`] if:
    /// - The code is not found in this ontology.
    /// - The code format is invalid (e.g., non-numeric SNOMED code).
    /// - Network is unavailable (if mode is Online and snapshot is absent).
    /// - Offline snapshot is unavailable (if mode is Offline or fallback is needed).
    ///
    /// # Async
    /// Marked async to support future network API calls (M5+).
    /// In M1, even offline snapshot loading may be async for compatibility.
    async fn resolve_atom(&self, code: &str) -> Result<Atom, OntologyError>;

    /// Check if two atoms are compatible (represent the same concept in the same version).
    ///
    /// Per [SPEC.md § 2.4][1] (DEF-PS-04):
    /// Two atoms are compatible iff they have the same system, code, and version.
    /// This is the substrate's mechanism for ensuring that hypotheses don't mix concepts
    /// across ontology versions or editions.
    ///
    /// Default implementation checks system, code, and version equality.
    /// Adapters may override for semantically richer compatibility (e.g., concept subsumption in M5+).
    ///
    /// [1]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#24-compatibility
    fn validate_compatibility(&self, atom1: &Atom, atom2: &Atom) -> bool {
        atom1.system == atom2.system && atom1.code == atom2.code && atom1.version == atom2.version
    }

    /// Return the ontology version this adapter uses.
    ///
    /// Used for audit trails (OBL-PS-01) and version-aware provenance (M5 temporal evolution).
    /// Format is system-specific (e.g., "2026-01-31" for SNOMED CT Edition dates).
    fn ontology_version(&self) -> &str;

    /// Return the caching mode this adapter operates in.
    fn cache_mode(&self) -> CacheMode;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ontology_system_display() {
        assert_eq!(OntologySystem::SNOMED.to_string(), "SNOMED");
        assert_eq!(OntologySystem::RxNorm.to_string(), "RxNorm");
        assert_eq!(OntologySystem::LOINC.to_string(), "LOINC");
        assert_eq!(OntologySystem::ICD11.to_string(), "ICD11");
        assert_eq!(OntologySystem::Unstructured.to_string(), "Unstructured");
    }

    #[test]
    fn test_atom_creation() {
        let atom = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        assert_eq!(atom.system, OntologySystem::SNOMED);
        assert_eq!(atom.code, "67822003");
        assert_eq!(atom.preferred_term, "Hypoxemia");
        assert_eq!(atom.version, "2026-01-31");
    }

    #[test]
    fn test_atom_compatibility_same() {
        let atom1 = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom2 = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };

        // Default implementation in trait
        let adapter = MockAdapter;
        assert!(adapter.validate_compatibility(&atom1, &atom2));
    }

    #[test]
    fn test_atom_compatibility_different_version() {
        let atom1 = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom2 = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2025-01-31".to_string(),
        };

        let adapter = MockAdapter;
        assert!(!adapter.validate_compatibility(&atom1, &atom2));
    }

    #[test]
    fn test_atom_compatibility_different_code() {
        let atom1 = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom2 = Atom {
            system: OntologySystem::SNOMED,
            code: "67822004".to_string(), // Different code
            preferred_term: "Different concept".to_string(),
            version: "2026-01-31".to_string(),
        };

        let adapter = MockAdapter;
        assert!(!adapter.validate_compatibility(&atom1, &atom2));
    }

    #[test]
    fn test_ontology_error_display() {
        let err = OntologyError::CodeNotFound {
            code: "67822003".to_string(),
            system: OntologySystem::SNOMED,
        };
        assert_eq!(err.to_string(), "Code not found: 67822003 in SNOMED");
    }

    // Mock adapter for testing trait defaults
    struct MockAdapter;

    #[async_trait::async_trait]
    impl OntologyAdapter for MockAdapter {
        async fn resolve_atom(&self, _code: &str) -> Result<Atom, OntologyError> {
            Err(OntologyError::CodeNotFound {
                code: "mock".to_string(),
                system: OntologySystem::SNOMED,
            })
        }

        fn ontology_version(&self) -> &str {
            "1.0.0"
        }

        fn cache_mode(&self) -> CacheMode {
            CacheMode::CacheOnly
        }
    }
}
