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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
/// **Identity semantics**: Atom identity is determined by (system, code, version) only.
/// The preferred_term is a display label and does not affect equality or hashing.
/// This ensures atoms with identical semantics but different term labels (e.g., "Hypoxemia" vs "Low blood oxygen")
/// are treated as equal in HashSet/HashMap and by the `PartialOrd` refinement order.
///
/// [1]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#22-atom-definition
#[derive(Debug, Clone)]
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

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self.system == other.system && self.code == other.code && self.version == other.version
    }
}

impl Eq for Atom {}

impl std::hash::Hash for Atom {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.system.hash(state);
        self.code.hash(state);
        self.version.hash(state);
    }
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

/// SNOMED CT ontology adapter with in-memory LRU cache and offline snapshot fallback.
///
/// Per [DESIGN-D2][1] and [SPEC.md DEF-PS-03][2]:
/// SNOMEDAdapter resolves SNOMED CT codes to atoms using an in-memory LRU cache backed by
/// an offline snapshot. In M1, snapshots are hand-coded test data. Real SNOMED CT API
/// integration (if needed) defers to M5+.
///
/// [1]: https://github.com/SHA888/SFClinAI/blob/main/DESIGN-D2-ontology.md
/// [2]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#def-ps-03
pub struct SNOMEDAdapter {
    /// In-memory LRU cache for frequently accessed codes.
    cache: std::sync::Arc<tokio::sync::Mutex<lru::LruCache<String, Atom>>>,
    /// Offline snapshot: pre-loaded SNOMED CT codes.
    snapshot: std::sync::Arc<std::collections::HashMap<String, Atom>>,
    /// SNOMED CT Edition version (e.g., "2026-01-31").
    version: String,
    /// Caching mode for this adapter.
    mode: CacheMode,
}

impl SNOMEDAdapter {
    /// Create a new SNOMEDAdapter.
    ///
    /// # Arguments
    /// * `snapshot` - Pre-loaded SNOMED codes as a HashMap.
    /// * `cache_size` - LRU cache capacity (e.g., 10000 for typical pilot).
    /// * `version` - SNOMED CT Edition version (e.g., "2026-01-31").
    /// * `mode` - Caching mode (CacheOnly for M1 with in-memory snapshots).
    ///
    /// # Example
    /// ```ignore
    /// use clinlat::ontology::{SNOMEDAdapter, Atom, OntologySystem, CacheMode};
    /// use std::collections::HashMap;
    ///
    /// let mut snapshot = HashMap::new();
    /// snapshot.insert(
    ///     "67822003".to_string(),
    ///     Atom {
    ///         system: OntologySystem::SNOMED,
    ///         code: "67822003".to_string(),
    ///         preferred_term: "Hypoxemia".to_string(),
    ///         version: "2026-01-31".to_string(),
    ///     },
    /// );
    ///
    /// let adapter = SNOMEDAdapter::new(snapshot, 10000, "2026-01-31", CacheMode::CacheOnly);
    /// ```
    pub fn new(
        snapshot: std::collections::HashMap<String, Atom>,
        cache_size: usize,
        version: impl Into<String>,
        mode: CacheMode,
    ) -> Self {
        let cache_size = if cache_size == 0 { 1024 } else { cache_size };
        SNOMEDAdapter {
            cache: std::sync::Arc::new(tokio::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(cache_size).unwrap(),
            ))),
            snapshot: std::sync::Arc::new(snapshot),
            version: version.into(),
            mode,
        }
    }

    /// Create a test adapter with standard SNOMED examples.
    ///
    /// This is a convenience constructor for testing. Loads three standard SNOMED codes
    /// relevant to respiratory dysfunction (supporting SOFA-respiratory operator tests).
    #[cfg(test)]
    pub fn test_adapter() -> Self {
        let mut snapshot = std::collections::HashMap::new();
        snapshot.insert(
            "67822003".to_string(),
            Atom {
                system: OntologySystem::SNOMED,
                code: "67822003".to_string(),
                preferred_term: "Hypoxemia".to_string(),
                version: "2026-01-31".to_string(),
            },
        );
        snapshot.insert(
            "3723001".to_string(),
            Atom {
                system: OntologySystem::SNOMED,
                code: "3723001".to_string(),
                preferred_term: "Acute respiratory distress syndrome".to_string(),
                version: "2026-01-31".to_string(),
            },
        );
        snapshot.insert(
            "29303001".to_string(),
            Atom {
                system: OntologySystem::SNOMED,
                code: "29303001".to_string(),
                preferred_term: "Respiratory distress".to_string(),
                version: "2026-01-31".to_string(),
            },
        );

        SNOMEDAdapter::new(snapshot, 100, "2026-01-31", CacheMode::CacheOnly)
    }
}

#[async_trait::async_trait]
impl OntologyAdapter for SNOMEDAdapter {
    async fn resolve_atom(&self, code: &str) -> Result<Atom, OntologyError> {
        // Check L1 cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(atom) = cache.get(code) {
                return Ok(atom.clone());
            }
        }

        // Fallback to offline snapshot
        // Note: Lock is released between cache-miss and cache-update, allowing concurrent misses
        // on the same code to both look up the snapshot. This creates rare redundant inserts, but:
        // 1) LruCache::put is idempotent for equal values (safe)
        // 2) Redundant inserts only evict other entries on subsequent misses, which is acceptable
        // 3) Avoiding lock-held-across-await patterns improves async safety
        if let Some(atom) = self.snapshot.get(code).cloned() {
            // Update cache on hit
            let mut cache = self.cache.lock().await;
            cache.put(code.to_string(), atom.clone());
            return Ok(atom);
        }

        // Not found
        Err(OntologyError::CodeNotFound {
            code: code.to_string(),
            system: OntologySystem::SNOMED,
        })
    }

    fn ontology_version(&self) -> &str {
        &self.version
    }

    fn cache_mode(&self) -> CacheMode {
        self.mode
    }
}

/// RxNorm drug ontology adapter with in-memory LRU cache and offline snapshot fallback.
///
/// Per [DESIGN-D2][1] and [SPEC.md DEF-PS-03][2]:
/// RxNormAdapter resolves RxNorm drug codes to atoms using an in-memory LRU cache backed by
/// an offline snapshot. RxNorm codes (RXCUIs) identify drugs with specific strengths and forms
/// (e.g., "855288" for Lisinopril 10 mg tablet). In M1, snapshots are hand-coded test data.
/// Real RxNorm API integration (if needed) defers to M5+.
///
/// [1]: https://github.com/SHA888/SFClinAI/blob/main/DESIGN-D2-ontology.md
/// [2]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#def-ps-03
pub struct RxNormAdapter {
    /// In-memory LRU cache for frequently accessed drug codes.
    cache: std::sync::Arc<tokio::sync::Mutex<lru::LruCache<String, Atom>>>,
    /// Offline snapshot: pre-loaded RxNorm drug codes.
    snapshot: std::sync::Arc<std::collections::HashMap<String, Atom>>,
    /// RxNorm release version (e.g., "2026-01-06").
    version: String,
    /// Caching mode for this adapter.
    mode: CacheMode,
}

impl RxNormAdapter {
    /// Create a new RxNormAdapter.
    ///
    /// # Arguments
    /// * `snapshot` - Pre-loaded RxNorm codes as a HashMap.
    /// * `cache_size` - LRU cache capacity (e.g., 10000 for typical pilot).
    /// * `version` - RxNorm release version (e.g., "2026-01-06").
    /// * `mode` - Caching mode (CacheOnly for M1 with in-memory snapshots).
    pub fn new(
        snapshot: std::collections::HashMap<String, Atom>,
        cache_size: usize,
        version: impl Into<String>,
        mode: CacheMode,
    ) -> Self {
        let cache_size = if cache_size == 0 { 1024 } else { cache_size };
        RxNormAdapter {
            cache: std::sync::Arc::new(tokio::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(cache_size).unwrap(),
            ))),
            snapshot: std::sync::Arc::new(snapshot),
            version: version.into(),
            mode,
        }
    }

    /// Create a test adapter with standard RxNorm examples.
    ///
    /// This is a convenience constructor for testing. Loads three standard RxNorm codes
    /// representing drugs commonly used in critical care (supporting clinical decision operators).
    #[cfg(test)]
    pub fn test_adapter() -> Self {
        let mut snapshot = std::collections::HashMap::new();
        snapshot.insert(
            "855288".to_string(),
            Atom {
                system: OntologySystem::RxNorm,
                code: "855288".to_string(),
                preferred_term: "Lisinopril 10 MG Oral Tablet".to_string(),
                version: "2026-01-06".to_string(),
            },
        );
        snapshot.insert(
            "312547".to_string(),
            Atom {
                system: OntologySystem::RxNorm,
                code: "312547".to_string(),
                preferred_term: "Metformin 500 MG Oral Tablet".to_string(),
                version: "2026-01-06".to_string(),
            },
        );
        snapshot.insert(
            "1049612".to_string(),
            Atom {
                system: OntologySystem::RxNorm,
                code: "1049612".to_string(),
                preferred_term: "Levothyroxine 50 MCG Oral Tablet".to_string(),
                version: "2026-01-06".to_string(),
            },
        );

        RxNormAdapter::new(snapshot, 100, "2026-01-06", CacheMode::CacheOnly)
    }
}

#[async_trait::async_trait]
impl OntologyAdapter for RxNormAdapter {
    async fn resolve_atom(&self, code: &str) -> Result<Atom, OntologyError> {
        // Check L1 cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(atom) = cache.get(code) {
                return Ok(atom.clone());
            }
        }

        // Fallback to offline snapshot
        // Note: Lock is released between cache-miss and cache-update, allowing concurrent misses
        // on the same code to both look up the snapshot. This creates rare redundant inserts, but:
        // 1) LruCache::put is idempotent for equal values (safe)
        // 2) Redundant inserts only evict other entries on subsequent misses, which is acceptable
        // 3) Avoiding lock-held-across-await patterns improves async safety
        if let Some(atom) = self.snapshot.get(code).cloned() {
            // Update cache on hit
            let mut cache = self.cache.lock().await;
            cache.put(code.to_string(), atom.clone());
            return Ok(atom);
        }

        // Not found
        Err(OntologyError::CodeNotFound {
            code: code.to_string(),
            system: OntologySystem::RxNorm,
        })
    }

    fn ontology_version(&self) -> &str {
        &self.version
    }

    fn cache_mode(&self) -> CacheMode {
        self.mode
    }
}

/// LOINC laboratory test ontology adapter with in-memory LRU cache and offline snapshot fallback.
///
/// Per [DESIGN-D2][1] and [SPEC.md DEF-PS-03][2]:
/// LoincAdapter resolves LOINC codes to atoms using an in-memory LRU cache backed by
/// an offline snapshot. LOINC codes (e.g., "2160-0" for creatinine serum) identify laboratory
/// tests, vital signs, and clinical observations. In M1, snapshots are hand-coded test data.
/// Real LOINC API integration (if needed) defers to M5+.
///
/// [1]: https://github.com/SHA888/SFClinAI/blob/main/DESIGN-D2-ontology.md
/// [2]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#def-ps-03
pub struct LoincAdapter {
    /// In-memory LRU cache for frequently accessed lab codes.
    cache: std::sync::Arc<tokio::sync::Mutex<lru::LruCache<String, Atom>>>,
    /// Offline snapshot: pre-loaded LOINC codes.
    snapshot: std::sync::Arc<std::collections::HashMap<String, Atom>>,
    /// LOINC release version (e.g., "2.76").
    version: String,
    /// Caching mode for this adapter.
    mode: CacheMode,
}

impl LoincAdapter {
    /// Create a new LoincAdapter.
    ///
    /// # Arguments
    /// * `snapshot` - Pre-loaded LOINC codes as a HashMap.
    /// * `cache_size` - LRU cache capacity (e.g., 10000 for typical pilot).
    /// * `version` - LOINC release version (e.g., "2.76").
    /// * `mode` - Caching mode (CacheOnly for M1 with in-memory snapshots).
    pub fn new(
        snapshot: std::collections::HashMap<String, Atom>,
        cache_size: usize,
        version: impl Into<String>,
        mode: CacheMode,
    ) -> Self {
        let cache_size = if cache_size == 0 { 1024 } else { cache_size };
        LoincAdapter {
            cache: std::sync::Arc::new(tokio::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(cache_size).unwrap(),
            ))),
            snapshot: std::sync::Arc::new(snapshot),
            version: version.into(),
            mode,
        }
    }

    /// Create a test adapter with standard LOINC examples.
    ///
    /// This is a convenience constructor for testing. Loads three standard LOINC codes
    /// representing laboratory tests and vital signs relevant to clinical decision-making.
    #[cfg(test)]
    pub fn test_adapter() -> Self {
        let mut snapshot = std::collections::HashMap::new();
        snapshot.insert(
            "2160-0".to_string(),
            Atom {
                system: OntologySystem::LOINC,
                code: "2160-0".to_string(),
                preferred_term: "Creatinine [Mass/volume] in Serum or Plasma".to_string(),
                version: "2.76".to_string(),
            },
        );
        snapshot.insert(
            "3024-7".to_string(),
            Atom {
                system: OntologySystem::LOINC,
                code: "3024-7".to_string(),
                preferred_term: "Urea nitrogen [Mass/volume] in Serum or Plasma".to_string(),
                version: "2.76".to_string(),
            },
        );
        snapshot.insert(
            "8867-4".to_string(),
            Atom {
                system: OntologySystem::LOINC,
                code: "8867-4".to_string(),
                preferred_term: "Heart rate".to_string(),
                version: "2.76".to_string(),
            },
        );

        LoincAdapter::new(snapshot, 100, "2.76", CacheMode::CacheOnly)
    }
}

#[async_trait::async_trait]
impl OntologyAdapter for LoincAdapter {
    async fn resolve_atom(&self, code: &str) -> Result<Atom, OntologyError> {
        // Check L1 cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(atom) = cache.get(code) {
                return Ok(atom.clone());
            }
        }

        // Fallback to offline snapshot
        // Note: Lock is released between cache-miss and cache-update, allowing concurrent misses
        // on the same code to both look up the snapshot. This creates rare redundant inserts, but:
        // 1) LruCache::put is idempotent for equal values (safe)
        // 2) Redundant inserts only evict other entries on subsequent misses, which is acceptable
        // 3) Avoiding lock-held-across-await patterns improves async safety
        if let Some(atom) = self.snapshot.get(code).cloned() {
            // Update cache on hit
            let mut cache = self.cache.lock().await;
            cache.put(code.to_string(), atom.clone());
            return Ok(atom);
        }

        // Not found
        Err(OntologyError::CodeNotFound {
            code: code.to_string(),
            system: OntologySystem::LOINC,
        })
    }

    fn ontology_version(&self) -> &str {
        &self.version
    }

    fn cache_mode(&self) -> CacheMode {
        self.mode
    }
}

/// ICD-11 diagnostic classification ontology adapter with in-memory LRU cache and offline snapshot fallback.
///
/// Per [DESIGN-D2][1] and [SPEC.md DEF-PS-03][2]:
/// Icd11Adapter resolves ICD-11 codes to atoms using an in-memory LRU cache backed by
/// an offline snapshot. ICD-11 codes (e.g., "BB81.1" for acute kidney injury) identify
/// diagnoses and conditions per WHO's International Classification of Diseases, 11th Revision.
/// In M1, snapshots are hand-coded test data. Real ICD-11 API integration (if needed) defers to M5+.
///
/// [1]: https://github.com/SHA888/SFClinAI/blob/main/DESIGN-D2-ontology.md
/// [2]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#def-ps-03
pub struct Icd11Adapter {
    /// In-memory LRU cache for frequently accessed diagnosis codes.
    cache: std::sync::Arc<tokio::sync::Mutex<lru::LruCache<String, Atom>>>,
    /// Offline snapshot: pre-loaded ICD-11 codes.
    snapshot: std::sync::Arc<std::collections::HashMap<String, Atom>>,
    /// ICD-11 release version (e.g., "2024-11").
    version: String,
    /// Caching mode for this adapter.
    mode: CacheMode,
}

impl Icd11Adapter {
    /// Create a new Icd11Adapter.
    ///
    /// # Arguments
    /// * `snapshot` - Pre-loaded ICD-11 codes as a HashMap.
    /// * `cache_size` - LRU cache capacity (e.g., 10000 for typical pilot).
    /// * `version` - ICD-11 release version (e.g., "2024-11").
    /// * `mode` - Caching mode (CacheOnly for M1 with in-memory snapshots).
    pub fn new(
        snapshot: std::collections::HashMap<String, Atom>,
        cache_size: usize,
        version: impl Into<String>,
        mode: CacheMode,
    ) -> Self {
        let cache_size = if cache_size == 0 { 1024 } else { cache_size };
        Icd11Adapter {
            cache: std::sync::Arc::new(tokio::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(cache_size).unwrap(),
            ))),
            snapshot: std::sync::Arc::new(snapshot),
            version: version.into(),
            mode,
        }
    }

    /// Create a test adapter with standard ICD-11 examples.
    ///
    /// This is a convenience constructor for testing. Loads three standard ICD-11 codes
    /// representing diagnoses relevant to critical care and the worked examples in NOTE.md.
    #[cfg(test)]
    pub fn test_adapter() -> Self {
        let mut snapshot = std::collections::HashMap::new();
        snapshot.insert(
            "BA47".to_string(),
            Atom {
                system: OntologySystem::ICD11,
                code: "BA47".to_string(),
                preferred_term: "Essential (primary) hypertension".to_string(),
                version: "2024-11".to_string(),
            },
        );
        snapshot.insert(
            "BB81.1".to_string(),
            Atom {
                system: OntologySystem::ICD11,
                code: "BB81.1".to_string(),
                preferred_term: "Acute kidney injury".to_string(),
                version: "2024-11".to_string(),
            },
        );
        snapshot.insert(
            "BA80.31".to_string(),
            Atom {
                system: OntologySystem::ICD11,
                code: "BA80.31".to_string(),
                preferred_term: "Type 2 diabetes mellitus".to_string(),
                version: "2024-11".to_string(),
            },
        );

        Icd11Adapter::new(snapshot, 100, "2024-11", CacheMode::CacheOnly)
    }
}

#[async_trait::async_trait]
impl OntologyAdapter for Icd11Adapter {
    async fn resolve_atom(&self, code: &str) -> Result<Atom, OntologyError> {
        // Check L1 cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(atom) = cache.get(code) {
                return Ok(atom.clone());
            }
        }

        // Fallback to offline snapshot
        // Note: Lock is released between cache-miss and cache-update, allowing concurrent misses
        // on the same code to both look up the snapshot. This creates rare redundant inserts, but:
        // 1) LruCache::put is idempotent for equal values (safe)
        // 2) Redundant inserts only evict other entries on subsequent misses, which is acceptable
        // 3) Avoiding lock-held-across-await patterns improves async safety
        if let Some(atom) = self.snapshot.get(code).cloned() {
            // Update cache on hit
            let mut cache = self.cache.lock().await;
            cache.put(code.to_string(), atom.clone());
            return Ok(atom);
        }

        // Not found
        Err(OntologyError::CodeNotFound {
            code: code.to_string(),
            system: OntologySystem::ICD11,
        })
    }

    fn ontology_version(&self) -> &str {
        &self.version
    }

    fn cache_mode(&self) -> CacheMode {
        self.mode
    }
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

    // SNOMEDAdapter tests
    #[tokio::test]
    async fn test_snomed_adapter_resolve_success() {
        let adapter = SNOMEDAdapter::test_adapter();
        let atom = adapter.resolve_atom("67822003").await;
        assert!(atom.is_ok());
        let atom = atom.unwrap();
        assert_eq!(atom.code, "67822003");
        assert_eq!(atom.preferred_term, "Hypoxemia");
        assert_eq!(atom.system, OntologySystem::SNOMED);
    }

    #[tokio::test]
    async fn test_snomed_adapter_resolve_not_found() {
        let adapter = SNOMEDAdapter::test_adapter();
        let result = adapter.resolve_atom("999999999").await;
        assert!(result.is_err());
        match result {
            Err(OntologyError::CodeNotFound { code, system }) => {
                assert_eq!(code, "999999999");
                assert_eq!(system, OntologySystem::SNOMED);
            }
            _ => panic!("Expected CodeNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_snomed_adapter_cache_behavior() {
        let adapter = SNOMEDAdapter::test_adapter();

        // First call loads from snapshot
        let atom1 = adapter.resolve_atom("67822003").await.unwrap();

        // Second call should come from cache (no snapshot access, but same result)
        let atom2 = adapter.resolve_atom("67822003").await.unwrap();

        assert_eq!(atom1, atom2);
        assert_eq!(atom1.code, "67822003");
    }

    #[tokio::test]
    async fn test_snomed_adapter_version() {
        let adapter = SNOMEDAdapter::test_adapter();
        assert_eq!(adapter.ontology_version(), "2026-01-31");
    }

    #[tokio::test]
    async fn test_snomed_adapter_cache_mode() {
        let adapter = SNOMEDAdapter::test_adapter();
        assert_eq!(adapter.cache_mode(), CacheMode::CacheOnly);
    }

    #[tokio::test]
    async fn test_snomed_adapter_multiple_codes() {
        let adapter = SNOMEDAdapter::test_adapter();

        // Test three different SNOMED codes from the fixture
        let hypoxemia = adapter.resolve_atom("67822003").await.unwrap();
        assert_eq!(hypoxemia.preferred_term, "Hypoxemia");

        let ards = adapter.resolve_atom("3723001").await.unwrap();
        assert_eq!(ards.preferred_term, "Acute respiratory distress syndrome");

        let resp_distress = adapter.resolve_atom("29303001").await.unwrap();
        assert_eq!(resp_distress.preferred_term, "Respiratory distress");

        // All should have the same version
        assert_eq!(hypoxemia.version, "2026-01-31");
        assert_eq!(ards.version, "2026-01-31");
        assert_eq!(resp_distress.version, "2026-01-31");
    }

    #[test]
    fn test_snomed_adapter_compatibility() {
        let adapter = SNOMEDAdapter::test_adapter();
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
        assert!(adapter.validate_compatibility(&atom1, &atom2));
    }

    // RxNormAdapter tests
    #[tokio::test]
    async fn test_rxnorm_adapter_resolve_success() {
        let adapter = RxNormAdapter::test_adapter();
        let atom = adapter.resolve_atom("855288").await;
        assert!(atom.is_ok());
        let atom = atom.unwrap();
        assert_eq!(atom.code, "855288");
        assert_eq!(atom.preferred_term, "Lisinopril 10 MG Oral Tablet");
        assert_eq!(atom.system, OntologySystem::RxNorm);
    }

    #[tokio::test]
    async fn test_rxnorm_adapter_resolve_not_found() {
        let adapter = RxNormAdapter::test_adapter();
        let result = adapter.resolve_atom("999999999").await;
        assert!(result.is_err());
        match result {
            Err(OntologyError::CodeNotFound { code, system }) => {
                assert_eq!(code, "999999999");
                assert_eq!(system, OntologySystem::RxNorm);
            }
            _ => panic!("Expected CodeNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_rxnorm_adapter_cache_behavior() {
        let adapter = RxNormAdapter::test_adapter();

        // First call loads from snapshot
        let atom1 = adapter.resolve_atom("855288").await.unwrap();

        // Second call should come from cache
        let atom2 = adapter.resolve_atom("855288").await.unwrap();

        assert_eq!(atom1, atom2);
        assert_eq!(atom1.code, "855288");
    }

    #[tokio::test]
    async fn test_rxnorm_adapter_version() {
        let adapter = RxNormAdapter::test_adapter();
        assert_eq!(adapter.ontology_version(), "2026-01-06");
    }

    #[tokio::test]
    async fn test_rxnorm_adapter_cache_mode() {
        let adapter = RxNormAdapter::test_adapter();
        assert_eq!(adapter.cache_mode(), CacheMode::CacheOnly);
    }

    #[tokio::test]
    async fn test_rxnorm_adapter_multiple_codes() {
        let adapter = RxNormAdapter::test_adapter();

        // Test three different RxNorm codes from the fixture
        let lisinopril = adapter.resolve_atom("855288").await.unwrap();
        assert_eq!(lisinopril.preferred_term, "Lisinopril 10 MG Oral Tablet");

        let metformin = adapter.resolve_atom("312547").await.unwrap();
        assert_eq!(metformin.preferred_term, "Metformin 500 MG Oral Tablet");

        let levothyroxine = adapter.resolve_atom("1049612").await.unwrap();
        assert_eq!(
            levothyroxine.preferred_term,
            "Levothyroxine 50 MCG Oral Tablet"
        );

        // All should have the same version
        assert_eq!(lisinopril.version, "2026-01-06");
        assert_eq!(metformin.version, "2026-01-06");
        assert_eq!(levothyroxine.version, "2026-01-06");
    }

    #[test]
    fn test_rxnorm_adapter_compatibility() {
        let adapter = RxNormAdapter::test_adapter();
        let atom1 = Atom {
            system: OntologySystem::RxNorm,
            code: "855288".to_string(),
            preferred_term: "Lisinopril 10 MG Oral Tablet".to_string(),
            version: "2026-01-06".to_string(),
        };
        let atom2 = Atom {
            system: OntologySystem::RxNorm,
            code: "855288".to_string(),
            preferred_term: "Lisinopril 10 MG Oral Tablet".to_string(),
            version: "2026-01-06".to_string(),
        };
        assert!(adapter.validate_compatibility(&atom1, &atom2));
    }

    // LoincAdapter tests
    #[tokio::test]
    async fn test_loinc_adapter_resolve_success() {
        let adapter = LoincAdapter::test_adapter();
        let atom = adapter.resolve_atom("2160-0").await;
        assert!(atom.is_ok());
        let atom = atom.unwrap();
        assert_eq!(atom.code, "2160-0");
        assert_eq!(
            atom.preferred_term,
            "Creatinine [Mass/volume] in Serum or Plasma"
        );
        assert_eq!(atom.system, OntologySystem::LOINC);
    }

    #[tokio::test]
    async fn test_loinc_adapter_resolve_not_found() {
        let adapter = LoincAdapter::test_adapter();
        let result = adapter.resolve_atom("9999-9").await;
        assert!(result.is_err());
        match result {
            Err(OntologyError::CodeNotFound { code, system }) => {
                assert_eq!(code, "9999-9");
                assert_eq!(system, OntologySystem::LOINC);
            }
            _ => panic!("Expected CodeNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_loinc_adapter_cache_behavior() {
        let adapter = LoincAdapter::test_adapter();

        // First call loads from snapshot
        let atom1 = adapter.resolve_atom("2160-0").await.unwrap();

        // Second call should come from cache
        let atom2 = adapter.resolve_atom("2160-0").await.unwrap();

        assert_eq!(atom1, atom2);
        assert_eq!(atom1.code, "2160-0");
    }

    #[tokio::test]
    async fn test_loinc_adapter_version() {
        let adapter = LoincAdapter::test_adapter();
        assert_eq!(adapter.ontology_version(), "2.76");
    }

    #[tokio::test]
    async fn test_loinc_adapter_cache_mode() {
        let adapter = LoincAdapter::test_adapter();
        assert_eq!(adapter.cache_mode(), CacheMode::CacheOnly);
    }

    #[tokio::test]
    async fn test_loinc_adapter_multiple_codes() {
        let adapter = LoincAdapter::test_adapter();

        // Test three different LOINC codes from the fixture
        let creatinine = adapter.resolve_atom("2160-0").await.unwrap();
        assert_eq!(
            creatinine.preferred_term,
            "Creatinine [Mass/volume] in Serum or Plasma"
        );

        let urea = adapter.resolve_atom("3024-7").await.unwrap();
        assert_eq!(
            urea.preferred_term,
            "Urea nitrogen [Mass/volume] in Serum or Plasma"
        );

        let heart_rate = adapter.resolve_atom("8867-4").await.unwrap();
        assert_eq!(heart_rate.preferred_term, "Heart rate");

        // All should have the same version
        assert_eq!(creatinine.version, "2.76");
        assert_eq!(urea.version, "2.76");
        assert_eq!(heart_rate.version, "2.76");
    }

    #[test]
    fn test_loinc_adapter_compatibility() {
        let adapter = LoincAdapter::test_adapter();
        let atom1 = Atom {
            system: OntologySystem::LOINC,
            code: "2160-0".to_string(),
            preferred_term: "Creatinine [Mass/volume] in Serum or Plasma".to_string(),
            version: "2.76".to_string(),
        };
        let atom2 = Atom {
            system: OntologySystem::LOINC,
            code: "2160-0".to_string(),
            preferred_term: "Creatinine [Mass/volume] in Serum or Plasma".to_string(),
            version: "2.76".to_string(),
        };
        assert!(adapter.validate_compatibility(&atom1, &atom2));
    }

    // Icd11Adapter tests
    #[tokio::test]
    async fn test_icd11_adapter_resolve_success() {
        let adapter = Icd11Adapter::test_adapter();
        let atom = adapter.resolve_atom("BA47").await;
        assert!(atom.is_ok());
        let atom = atom.unwrap();
        assert_eq!(atom.code, "BA47");
        assert_eq!(atom.preferred_term, "Essential (primary) hypertension");
        assert_eq!(atom.system, OntologySystem::ICD11);
    }

    #[tokio::test]
    async fn test_icd11_adapter_resolve_not_found() {
        let adapter = Icd11Adapter::test_adapter();
        let result = adapter.resolve_atom("ZZ99").await;
        assert!(result.is_err());
        match result {
            Err(OntologyError::CodeNotFound { code, system }) => {
                assert_eq!(code, "ZZ99");
                assert_eq!(system, OntologySystem::ICD11);
            }
            _ => panic!("Expected CodeNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_icd11_adapter_cache_behavior() {
        let adapter = Icd11Adapter::test_adapter();

        // First call loads from snapshot
        let atom1 = adapter.resolve_atom("BA47").await.unwrap();

        // Second call should come from cache
        let atom2 = adapter.resolve_atom("BA47").await.unwrap();

        assert_eq!(atom1, atom2);
        assert_eq!(atom1.code, "BA47");
    }

    #[tokio::test]
    async fn test_icd11_adapter_version() {
        let adapter = Icd11Adapter::test_adapter();
        assert_eq!(adapter.ontology_version(), "2024-11");
    }

    #[tokio::test]
    async fn test_icd11_adapter_cache_mode() {
        let adapter = Icd11Adapter::test_adapter();
        assert_eq!(adapter.cache_mode(), CacheMode::CacheOnly);
    }

    #[tokio::test]
    async fn test_icd11_adapter_multiple_codes() {
        let adapter = Icd11Adapter::test_adapter();

        // Test three different ICD-11 codes from the fixture
        let hypertension = adapter.resolve_atom("BA47").await.unwrap();
        assert_eq!(
            hypertension.preferred_term,
            "Essential (primary) hypertension"
        );

        let aki = adapter.resolve_atom("BB81.1").await.unwrap();
        assert_eq!(aki.preferred_term, "Acute kidney injury");

        let diabetes = adapter.resolve_atom("BA80.31").await.unwrap();
        assert_eq!(diabetes.preferred_term, "Type 2 diabetes mellitus");

        // All should have the same version
        assert_eq!(hypertension.version, "2024-11");
        assert_eq!(aki.version, "2024-11");
        assert_eq!(diabetes.version, "2024-11");
    }

    #[test]
    fn test_icd11_adapter_compatibility() {
        let adapter = Icd11Adapter::test_adapter();
        let atom1 = Atom {
            system: OntologySystem::ICD11,
            code: "BA47".to_string(),
            preferred_term: "Essential (primary) hypertension".to_string(),
            version: "2024-11".to_string(),
        };
        let atom2 = Atom {
            system: OntologySystem::ICD11,
            code: "BA47".to_string(),
            preferred_term: "Essential (primary) hypertension".to_string(),
            version: "2024-11".to_string(),
        };
        assert!(adapter.validate_compatibility(&atom1, &atom2));
    }

    #[test]
    fn test_atom_identity_excludes_preferred_term() {
        // Two atoms with same semantic identity (system, code, version) but different
        // preferred_term should be equal. This fixes bug #4: atom identity mismatch.
        let atom_hypoxemia_label1 = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        };
        let atom_hypoxemia_label2 = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Low blood oxygen level".to_string(),
            version: "2026-01-31".to_string(),
        };

        // Same semantic identity
        assert_eq!(atom_hypoxemia_label1, atom_hypoxemia_label2);

        // HashSet membership should work correctly
        let mut set = std::collections::HashSet::new();
        set.insert(atom_hypoxemia_label1.clone());
        assert!(set.contains(&atom_hypoxemia_label2));

        // Different version or code should not be equal
        let atom_different_version = Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2025-01-31".to_string(),
        };
        assert_ne!(atom_hypoxemia_label1, atom_different_version);
    }
}
