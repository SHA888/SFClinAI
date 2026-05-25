# DESIGN-D1: Provenance Encoding Strategy

**Date:** 2026-05-25
**Milestone:** M1 (v0.2.0)
**Decision:** JSON encoding with optional gzip compression for M1–M4; re-evaluate for DAG-based approach at M5+.
**Status:** Approved for implementation

---

## Problem

Milestone M1.2 requires a typed `Provenance` carrier replacing the `()` stub. The carrier must support:

- **DEF-MP-14**: Typed provenance with `origin` (data source), `timestamp` (ISO 8601), `version` (Ver struct), `metadata` (key-value pairs)
- **OBL-PS-04**: Audit-trail fidelity — operator output provenance carries source, timestamp, version; audit queries must be answerable
- **INV-PS-05**: Version-respecting derivation chains; `derives_from` lineage must be reconstructible
- **DEF-TE-07** (M5+): Evolution-aware provenance with version-respecting `derives_from` over months/years of operator updates

## Options Evaluated

### 1. JSON Encoding

**Structure:**
```json
{
  "origin": { "source_type": "external_lab_api", "system": "LOINC", "identifier": "2160-0" },
  "timestamp": "2026-05-25T10:30:00Z",
  "version": { "system": "clinlat", "operator": "sofa_resp", "build": "0.2.0" },
  "metadata": {
    "lab_system": "epic_lis",
    "transmission_protocol": "hl7_v2",
    "confidence_score": "0.98"
  }
}
```

**Pros:**
- Native queryability: SQL `WHERE json_column->'version'->>'operator' = 'sofa_resp'` in SQLite/PostgreSQL
- Human-readable audit logs and debugging
- Gzip compression reduces transit size to ~15% overhead (vs. ~7 KB uncompressed)
- Version-chain reconstruction: traverse `derives_from` arrays with native SQL/Python json operators
- No external dependencies (serde_json in Rust is stable)

**Cons:**
- Text-based; larger in-memory footprint than binary encodings
- Not content-addressable (no built-in deduplication)

**Audit Fidelity Score:** ✓ (Full: source system, timestamp, version, operator identity all queryable)

### 2. CBOR Encoding (RFC 7049)

**Pros:**
- ~10% overhead (more compact than JSON)
- Deterministic encoding (supports cryptographic signing per future M6 regulatory work)
- Still human-inspectable with `cbor2` tools

**Cons:**
- Query-ability requires deserialization (not native SQL)
- Adds dependency on `serde_cbor` (stable but less ecosystem maturity than JSON)
- Version chains require round-trip decode/encode to traverse

**Audit Fidelity Score:** ✓ (Full: same as JSON, but requires codec step)

### 3. Merkle DAG (Content-Addressed)

**Concept:** Each provenance node is a hash of its content; derivation chains are hash links.

**Pros:**
- Deduplication: identical evidence appearing in multiple derivation chains stored once
- Immutable audit trail: hash link can't be forged
- Excellent for long-running evolutionary systems (M5+)

**Cons:**
- Very high complexity for M1–M4 (not justified by scale)
- Query-ability: "find all hypotheses derived from evidence E" requires full DAG traversal
- Requires content-addressing infrastructure (IPFS-like or custom)
- Operator versioning becomes implicit in hash, less transparent

**Audit Fidelity Score:** ✓✓ (Superior for large-scale evolution, but over-engineered for M1)

### 4. Hybrid (JSON + Merkle)

**Concept:** Inline small provenance as JSON; large lineage graphs as Merkle DAG with hash references.

**Pros:**
- Flexibility: pay complexity cost only when scale demands it

**Cons:**
- Dual serialization logic (maintainability burden)
- Uncertain transition point between inline and DAG
- Deferred optimization (complexity before evidence)

---

## Decision

**Chosen:** **JSON encoding** for M1–M4.

**Rationale:**

1. **Query-ability first:** OBL-PS-04 requires "audit queries answerable". SQL-native JSON querying is the fastest path to compliance. Merkle DAG adds 2–3 weeks of infrastructure we don't need yet.

2. **Version-chain reconstruction:** INV-PS-05 + DEF-TE-07 (M5) require traversing `derives_from` chains. JSON trees map directly to SQL recursion (`WITH RECURSIVE`), requiring no extra tooling.

3. **Clinical-grade auditability:** Clinicians and regulators (FDA post-market monitoring, EU AI Act Article 72) need human-readable provenance logs. JSON audit dumps are transparent; CBOR/DAG require tools.

4. **Pragmatic scope:** M1–M4 are bootstrapping operations (4 operators, single institution simulation). Provenance volume is ~100 KB/day at pilot scale. Compression handles transmission; DAG is premature.

5. **M5+ re-evaluation:** Once M5 temporal evolution lands, we'll have months of real data showing whether DAG deduplication is worth the refactor. Migrate then if audit volume justifies.

**Non-goal:** Cryptographic signing of provenance (Merkle hash chain). This belongs to M6 (regulatory artifacts). For now, timestamp + version is sufficient audit signal.

---

## Implementation Sketch

**Rust type:**
```rust
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Provenance {
    pub origin: ProvenanceOrigin,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: Ver,
    pub metadata: BTreeMap<String, serde_json::Value>,
    /// Optional: derives_from hashes for M5+ temporal evolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derives_from: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProvenanceOrigin {
    pub source_type: String, // "external_lab_api", "clinician_input", "operator_derivation"
    pub system: String,      // ontology system: "LOINC", "SNOMED", etc.
    pub identifier: String,  // code in that system
}

impl Provenance {
    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Gzip-compress JSON for transit
    pub fn to_json_compressed(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let json = self.to_json()?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json.as_bytes())?;
        Ok(encoder.finish()?)
    }

    /// Decompress gzip and deserialize
    pub fn from_json_compressed(compressed: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(compressed);
        let mut json = String::new();
        decoder.read_to_string(&mut json)?;
        Ok(Self::from_json(&json)?)
    }
}
```

**Audit query (SQL example, for M5 audit infrastructure):**
```sql
-- Find all hypotheses derived under SOFA-respiratory v0.2.0
SELECT hypothesis_id, evidence_id, timestamp
FROM patient_substrate
WHERE provenance->>'version'->'operator' = 'sofa_resp'
  AND provenance->>'version'->'build' = '0.2.0'
ORDER BY timestamp;

-- Trace derivation chain (M5 temporal-evolution audit)
WITH RECURSIVE chain AS (
  SELECT id, derives_from, timestamp FROM patient_substrate
  WHERE id = ?
  UNION ALL
  SELECT ps.id, ps.derives_from, ps.timestamp
  FROM patient_substrate ps
  JOIN chain ON ps.id = chain.derives_from[0]
)
SELECT * FROM chain;
```

---

## Trade-offs Accepted

- **Overhead:** ~15% transit size for JSON vs. binary. Acceptable at clinical event scale; re-evaluate at 10M+ events/month.
- **Queryability within Rust:** No native JSON query in serde_json; must deserialize to query fields. Pattern: use `serde_json::Value` for flexible metadata, but keep `origin`, `timestamp`, `version` as typed fields.
- **Not cryptographically signed:** Timestamp + version alone don't prevent tampering. M6 (regulatory) will add signature when required.

---

## Next Steps

1. M1.2 task: Implement `Provenance` struct per this sketch; add `to_json()`, `from_json()`, and gzip methods.
2. M1.2 task: Update `Evidence` to carry `Provenance` instead of `()`.
3. M5 (re-evaluation): If audit volume and clinician feedback justify Merkle DAG, revisit with real data.
4. M6 (regulatory): Add cryptographic signing if FDA/EU requirements demand immutability.

---

## Sign-off

**Decided:** Yes, proceed with JSON encoding for M1–M4.
**Date:** 2026-05-25
**Rationale Owner:** Substrate-first clinical AI architecture
