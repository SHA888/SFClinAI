# DESIGN-D2: Ontology Adapter Caching Strategy

**Date:** 2026-05-25
**Milestone:** M1 (v0.2.0)
**Depends on:** D1 (Provenance encoding — JSON)
**Decision:** In-memory LRU cache with offline snapshot fallback. Single-instance for M1–M4; Redis backing for M5+ multi-instance.
**Status:** Approved for implementation

---

## Problem

Milestone M1.1 requires real ontology binding through adapters for SNOMED CT, RxNorm, LOINC, ICD-11 (DEF-PS-03, DEF-PS-04, INV-PS-01, OBL-PS-01). These systems are:

- **Large:** SNOMED CT has 300K+ active concepts; RxNorm ~120K drugs; LOINC ~90K codes
- **External:** Require network access to authoritative APIs or local snapshots
- **Versioned:** Each ontology has release dates; code validity depends on version
- **Institutional constraint:** Some pilot sites lack reliable internet (NOTE.md §5 deployment consideration)

The adapter caching strategy must:

1. Support **fast lookups** (sub-10ms for cached misses; sub-100ms for API hits)
2. Support **offline mode** (institutions without network access can use pre-downloaded snapshots)
3. Support **version pinning** (snapshots are locked to a specific ontology release)
4. Scale **from single-instance (M1–M4) to multi-instance (M5+)** without redesign
5. Maintain **audit trail** (which adapter version was used for each atom resolution?)

## Options Evaluated

### 1. In-Memory LRU Cache (Single-Instance)

**Architecture:**
- On startup: load SNOMED/RxNorm/LOINC/ICD-11 snapshots into memory (or lazy-load on first access)
- LRU cache (e.g., `lru::LruCache<String, Atom>`) for frequently accessed codes
- Cache miss → network API call or filesystem snapshot (fallback)
- No persistence between restarts (acceptable for pilot; re-evaluated post-M4)

**Pros:**
- Simplest implementation (no external dependencies like Redis)
- Fast lookups (sub-1ms for cache hits)
- Offline snapshot support via filesystem
- Deterministic for testing (no distributed state)
- Sufficient for single-instance pilot simulator

**Cons:**
- No inter-process sharing (unsuitable for multi-instance deployments)
- Memory footprint: SNOMED snapshot ~50 MB; RxNorm ~30 MB (total ~150 MB for 4 ontologies) — acceptable for modern laptops/servers

**Suitable for:** M1–M4 bootstrapping, pilot sites with single clinician workstation

### 2. Redis Backing Store

**Architecture:**
- Shared Redis instance (e.g., redis:6379) stores atom resolutions as key-value pairs
- Each instance queries Redis for cache hits; Redis queries upstream API/snapshot on miss
- Thread-safe, multi-instance capable out of the box

**Pros:**
- Multi-instance capable (scales for M5+)
- Persistent caching across restarts
- Built-in TTL support (auto-refresh stale codes)
- Easy monitoring (Redis CLI introspection)

**Cons:**
- Extra dependency (Redis server, `redis` Rust crate)
- Network latency (Redis round-trip vs. in-memory lookup)
- Operational overhead: run, monitor, fail-over Redis
- Not suitable for offline deployments (requires network to Redis)

**Suitable for:** M5+ multi-instance institutional deployments

### 3. Offline Snapshots Only (No Network)

**Architecture:**
- Pre-download SNOMED/RxNorm/LOINC/ICD-11 snapshots once (per ontology release)
- Ship snapshots in the crate or as data files
- No network calls; purely filesystem-based

**Pros:**
- Maximum portability (works in air-gapped environments)
- Predictable behavior (no latency variation from network)
- No infrastructure dependencies

**Cons:**
- Snapshots become stale (if new ICD-11 codes appear, pilot doesn't know about them)
- Large crate binary (shipping 150 MB+ of data)
- Manual snapshot updates (operational burden)

**Suitable for:** Narrow clinical use cases with frozen code sets (e.g., hospital protocols never change)

### 4. Hybrid: LRU Cache + API + Offline Fallback

**Architecture:**
- Try in-memory LRU cache (fast path)
- If miss: try network API call to authoritative system (freshness path)
- If API fails: fallback to offline snapshot (graceful degradation path)
- Configurable mode: `CacheMode::Online` (prefer API), `CacheMode::Offline` (snapshot only), `CacheMode::CacheOnly` (in-memory only)

**Pros:**
- Best of all worlds: fast caching + fresh data + offline robustness
- Graceful degradation (pilot works even if SNOMED API is down)
- Configurable per deployment (online sites use API, offline use snapshots)

**Cons:**
- Complexity: three fallback paths to test and debug
- Uncertain tie-breaking (if cache, API, and snapshot disagree on a code's validity, which wins?)

---

## Decision

**Chosen:** **In-memory LRU cache with offline snapshot fallback** for M1–M4.

**Rationale:**

1. **Pilot bootstrap:** M1–M4 are single-instance simulators. In-memory caching is sufficient and deterministic.

2. **Offline support:** Snapshot fallback satisfies institutional constraint (NOTE.md §5). If API is unavailable, pilot can still use pre-downloaded snapshot.

3. **Deferred complexity:** Redis adds 2–3 weeks of infrastructure. Defer to M5 when multi-instance deployments are actually needed.

4. **Clear API contract:** Define `OntologyAdapter` trait with cache-agnostic interface. Implementations (SNOMEDAdapter, RxNormAdapter, etc.) own their caching strategy.

5. **Version pinning:** Snapshots are tagged with ontology release date/version. Pilot explicitly knows "I'm using SNOMED CT Edition 2026-01-31".

**Layered approach:**

- **M1–M4:** In-memory LRU + offline snapshots. Single-instance.
- **M5–M6:** Add Redis backing (multi-instance support). Keep in-memory LRU as L1 cache; Redis as L2.
- **M7+:** Evaluate based on pilot feedback. If offline sites outnumber online, strengthen snapshot story. If multi-instance is critical, formalize Redis contract.

---

## Implementation Sketch

### Core Adapter Trait

```rust
use async_trait::async_trait;
use thiserror::Error;

/// OntologySystem identifies which ontology a code belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OntologySystem {
    SNOMED,   // SNOMED CT (SNOMED International)
    RxNorm,   // RxNorm (NIH NLM)
    LOINC,    // LOINC (Regenstrief Institute)
    ICD11,    // ICD-11 (WHO)
}

/// Atom is a resolved ontology code (see M1.1 task 1.6)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Atom {
    pub system: OntologySystem,
    pub code: String,                // "8867-4" for LOINC
    pub preferred_term: String,      // "Heart rate"
    pub version: String,             // ontology version: "2026-01-31"
}

#[derive(Debug, Error)]
pub enum OntologyError {
    #[error("Code not found: {code} in {system:?}")]
    CodeNotFound { code: String, system: OntologySystem },

    #[error("Network error resolving {system:?}: {source}")]
    NetworkError { system: OntologySystem, #[from] source: Box<dyn std::error::Error> },

    #[error("Invalid code format: {code}")]
    InvalidCodeFormat { code: String },

    #[error("Offline snapshot unavailable for {system:?}")]
    OfflineSnapshotUnavailable { system: OntologySystem },
}

/// OntologyAdapter resolves codes to Atoms; implementations own caching
#[async_trait]
pub trait OntologyAdapter: Send + Sync {
    /// Resolve a code in this ontology to an Atom
    /// Returns error if code not found, network unavailable, or format invalid
    async fn resolve_atom(&self, code: &str) -> Result<Atom, OntologyError>;

    /// Check if two atoms are compatible (same code, same version)
    /// Used for DEF-PS-04 compatibility checking
    fn validate_compatibility(&self, atom1: &Atom, atom2: &Atom) -> bool {
        atom1.system == atom2.system
            && atom1.code == atom2.code
            && atom1.version == atom2.version
    }

    /// Return the ontology version this adapter uses
    fn ontology_version(&self) -> &str;

    /// Return the cache mode: Online (prefer API), Offline (snapshot only), or CacheOnly
    fn cache_mode(&self) -> CacheMode;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    /// Prefer API calls; fallback to offline snapshot on network failure
    Online,
    /// Use offline snapshots only; error if snapshot unavailable
    Offline,
    /// Use in-memory cache only; no API or snapshot fallback
    CacheOnly,
}
```

### SNOMED CT Adapter (Example Implementation)

```rust
use std::sync::Arc;
use lru::LruCache;
use std::num::NonZeroUsize;
use tokio::sync::Mutex;

pub struct SNOMEDAdapter {
    /// L1 cache: in-memory LRU (e.g., 10K most-accessed SNOMED codes)
    cache: Arc<Mutex<LruCache<String, Atom>>>,
    /// Offline snapshot: pre-downloaded SNOMED CT codes (loaded at startup)
    snapshot: Arc<std::collections::HashMap<String, Atom>>,
    /// Ontology version this adapter was built with
    version: String,
    /// Cache mode (Online, Offline, CacheOnly)
    mode: CacheMode,
}

impl SNOMEDAdapter {
    pub fn new(
        snapshot_path: &str, // e.g., "data/snomed-2026-01-31.json.gz"
        cache_size: usize,   // e.g., 10000
        mode: CacheMode,
    ) -> Result<Self, OntologyError> {
        // Load offline snapshot from gzip JSON file
        let snapshot = Self::load_snapshot(snapshot_path)?;
        let version = snapshot
            .values()
            .next()
            .map(|a| a.version.clone())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(SNOMEDAdapter {
            cache: Arc::new(Mutex::new(
                LruCache::new(NonZeroUsize::new(cache_size).unwrap())
            )),
            snapshot: Arc::new(snapshot),
            version,
            mode,
        })
    }

    async fn load_snapshot(path: &str) -> Result<std::collections::HashMap<String, Atom>, OntologyError> {
        // Load from gzip JSON file
        use flate2::read::GzDecoder;
        use std::io::Read;
        use std::fs::File;

        let file = File::open(path)
            .map_err(|e| OntologyError::OfflineSnapshotUnavailable { system: OntologySystem::SNOMED })?;
        let mut decoder = GzDecoder::new(file);
        let mut json = String::new();
        decoder.read_to_string(&mut json)
            .map_err(|e| OntologyError::OfflineSnapshotUnavailable { system: OntologySystem::SNOMED })?;

        serde_json::from_str(&json)
            .map_err(|e| OntologyError::CodeNotFound {
                code: "snapshot".to_string(),
                system: OntologySystem::SNOMED,
            })
    }
}

#[async_trait]
impl OntologyAdapter for SNOMEDAdapter {
    async fn resolve_atom(&self, code: &str) -> Result<Atom, OntologyError> {
        // Validate code format (SNOMED codes are numeric)
        if code.chars().all(|c| c.is_numeric()) {
            // Check L1 cache
            {
                let mut cache = self.cache.lock().await;
                if let Some(atom) = cache.get(code) {
                    return Ok(atom.clone());
                }
            }

            // Try online API (if mode permits)
            if self.mode == CacheMode::Online {
                // TODO: Implement real API call to SNOMED browser API
                // For M1, this is a stub that queries snapshot
                if let Some(atom) = self.snapshot.get(code).cloned() {
                    self.cache.lock().await.put(code.to_string(), atom.clone());
                    return Ok(atom);
                }
                // API call would go here; for now, fall through to snapshot
            }

            // Fallback to offline snapshot
            if let Some(atom) = self.snapshot.get(code).cloned() {
                self.cache.lock().await.put(code.to_string(), atom.clone());
                return Ok(atom);
            }
        }

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
```

### OntologyAdapterSet (Registry)

```rust
pub struct OntologyAdapterSet {
    snomed: Arc<dyn OntologyAdapter>,
    rxnorm: Arc<dyn OntologyAdapter>,
    loinc: Arc<dyn OntologyAdapter>,
    icd11: Arc<dyn OntologyAdapter>,
}

impl OntologyAdapterSet {
    pub fn new(
        snomed: Arc<dyn OntologyAdapter>,
        rxnorm: Arc<dyn OntologyAdapter>,
        loinc: Arc<dyn OntologyAdapter>,
        icd11: Arc<dyn OntologyAdapter>,
    ) -> Self {
        OntologyAdapterSet { snomed, rxnorm, loinc, icd11 }
    }

    pub async fn resolve(&self, system: OntologySystem, code: &str) -> Result<Atom, OntologyError> {
        match system {
            OntologySystem::SNOMED => self.snomed.resolve_atom(code).await,
            OntologySystem::RxNorm => self.rxnorm.resolve_atom(code).await,
            OntologySystem::LOINC => self.loinc.resolve_atom(code).await,
            OntologySystem::ICD11 => self.icd11.resolve_atom(code).await,
        }
    }
}
```

### Snapshot Generation (Off-Path Tool)

For M1, we'll provide a small utility to generate offline snapshots from authoritative sources:

```rust
// clinlat/src/bin/generate-ontology-snapshots.rs
// Run: cargo run --bin generate-ontology-snapshots -- --out data/

// For M1, we can hand-code ~10-20 test codes per ontology:
// SNOMED: ["8867-4" → "Heart rate", "3717-0" → "Systolic BP", ...]
// RxNorm: ["1049589" → "Lisinopril 10 mg", ...]
// LOINC: ["2160-0" → "Creatinine", ...]
// ICD-11: ["BA47" → "Hypertension", ...]
```

---

## API Contract Summary

**For task 1.1 (OntologyAdapter trait signature):**

```rust
// The trait all adapters implement
pub trait OntologyAdapter: Send + Sync {
    async fn resolve_atom(&self, code: &str) -> Result<Atom, OntologyError>;
    fn validate_compatibility(&self, atom1: &Atom, atom2: &Atom) -> bool;
    fn ontology_version(&self) -> &str;
    fn cache_mode(&self) -> CacheMode;
}

// Concrete types: SNOMEDAdapter, RxNormAdapter, LoincAdapter, Icd11Adapter
// Each impl OntologyAdapter with in-memory LRU + offline snapshot fallback
```

**Dependencies for M1 implementation:**

```toml
# Cargo.toml additions
[dependencies]
lru = "0.12"                    # In-memory LRU cache
chrono = { version = "0.4", features = ["serde"] }  # Timestamps
async-trait = "0.1"             # Async trait syntax
flate2 = "1.0"                  # Gzip for snapshots
```

---

## Trade-offs Accepted

- **No multi-instance in M1–M4:** Defer Redis to M5. Single-instance caching is deterministic and testable.
- **API calls are stubbed in M1:** Real SNOMED CT API requires NLM API key and authentication. For M1, we use offline snapshots only. Real API integration comes when pilots require fresh codes.
- **Large snapshot files:** 150 MB total for 4 ontologies. Acceptable for laptops/servers; consider compression or pagination for embedded systems (deferred to M7+).
- **No real-time code validation:** Snapshots are static. If new SNOMED codes appear between releases, pilot doesn't know. Acceptable because clinical codes are stable; emergency updates require manual intervention.

---

## Next Steps

1. **M1.1 task 1.1:** Implement `OntologyAdapter` trait and `Atom` type per this API contract.
2. **M1.1 tasks 1.2–1.5:** Implement `SNOMEDAdapter`, `RxNormAdapter`, `LoincAdapter`, `Icd11Adapter` with offline snapshot support.
3. **M1.1 task 1.6:** Define `Atom` type as replacement for `&'static str` AtomId.
4. **M5 re-evaluation:** Multi-instance pilot sites → Redis backing store + L1/L2 cache strategy.

---

## Sign-off

**Decided:** Yes, proceed with in-memory LRU + offline snapshots for M1–M4.
**Date:** 2026-05-25
**Rationale Owner:** Substrate-first clinical AI architecture
