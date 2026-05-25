# M1 Provenance Specification

**Document Type:** Specification of Correctness (SSOT) for M1 provenance carrier implementation
**Date:** 2026-05-25
**Scope:** Formalizes the `Provenance` type contract for clinlat v0.2.0
**Depends on:** D1 (JSON encoding decision), SPEC.md §0.2 (mathematical preliminaries)

---

## 1. Overview

The patient-state substrate (clinlat) carries evidence through operators via a typed `Provenance` carrier, replacing the `()` stub from v0.1.0. This document specifies the type contract, serialization format, and invariants that ensure audit-trail fidelity (OBL-PS-04) and version-respecting derivation chains (INV-PS-05, DEF-TE-07).

---

## 2. Type Signature

### 2.1 Core Types

**Provenance (carrier type):**

```
Provenance ≡ {
  origin: ProvenanceOrigin,
  timestamp: DateTime[UTC],
  version: Ver,
  metadata: Map[String, Value],
  derives_from?: Vec[Hash]  // optional, for M5+ evolution
}
```

**ProvenanceOrigin (source tracking):**

```
ProvenanceOrigin ≡ {
  source_type: String,  // "external_lab_api", "clinician_input", "operator_derivation"
  system: OntologySystem,  // "SNOMED", "RxNorm", "LOINC", "ICD11", or "unstructured"
  identifier: String  // code/ID in that system
}
```

**OntologySystem (enumeration):**

```
OntologySystem ∈ { SNOMED, RxNorm, LOINC, ICD11, Unstructured }
```

**Ver (version tuple, from SPEC.md §1.1):**

```
Ver ≡ {
  system: String,    // "clinlat"
  operator: String,  // "sofa_resp", "kdigo_aki", etc.
  build: String      // SemVer: "0.2.0", "0.2.1", etc.
}
```

### 2.2 JSON Serialization

**Wire format (canonical JSON):**

```json
{
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
    "transmission_protocol": "hl7_v2",
    "confidence_score": "0.98"
  },
  "derives_from": null
}
```

**Serialization rules:**

1. `timestamp` must be in ISO 8601 format with explicit UTC timezone (`Z` suffix).
2. `metadata` is a JSON object with arbitrary string keys and JSON-serializable values.
3. `derives_from` is omitted (null) in M1. When present (M5+), contains hash strings referencing ancestor provenance records.
4. All timestamps must be canonical (no fractional seconds beyond milliseconds).

### 2.3 Deserialization Contract

**Parsing safety:**

- Missing fields: `derives_from` defaults to `null`; all other fields are required (deserialization fails if missing).
- Malformed `timestamp`: fails (not ISO 8601 or non-UTC).
- `metadata` with circular structure: fails (JSON does not permit cycles; structural rejection at parse time).
- `version.build` with invalid SemVer: accepted as-is (downgrade to string comparison; no semantic version parsing required in M1).

---

## 3. Invariants (INV-PS-05, INV-TE-04 preview)

### INV-PS-05: Version-Respecting Derivation Chains

**Statement:**

If `H'` is derived from `H` under operator `Op(v)`, then `provenance(H')` carries `version = v`, and any subsequent evidence incorporation into `H'` must not silent-replace `Op(v)` with a newer or older `Op(v')` without explicit re-review (M5 temporal evolution, INV-TE-04).

**Formalization:**

```
Let H ∈ Hyp, Op ∈ Operator, v ∈ Ver, E ∈ Evidence.
If Outcome.Refined(H') = Op.apply(H, E) with Op.version = v,
then provenance(H').version = v.

No code path through Op.apply can silently change provenance version
  without clinician-mediated re-review (M5+).
```

**Implementation check (M1):**

In `clinlat/docs/invariants/inv-ps-05-version-chain.md`:
- Operator.apply() extracts Evidence.provenance.version.
- Operator.apply() attaches operator version to Hyp output provenance.
- Property test: `∀ H ∈ Hyp, E ∈ Evidence: provenance(Op.apply(H, E)).version == Op.version`.

### INV-TE-04 (M5 preview): No Automatic Replacement Without Re-Review

**Foreshadowing (deferred to M5 temporal evolution):**

The provenance carrier is designed to support INV-TE-04, which forbids silent downgrade from patient-locally-optimal to institutionally-feasible refinement (M4) or from one operator version to another without clinician visibility. By carrying explicit version tags, provenance enables audit queries that detect such replacement.

---

## 4. Proof Obligations

### OBL-PS-04: Audit-Trail Fidelity

**Statement (from SPEC.md §6):**

Operator output provenance carries source, timestamp, version; audit queries answerable from provenance alone.

**Discharge proof (informal-argument tier, M1):**

1. **Source tracking:** `ProvenanceOrigin` captures the input evidence source (lab system, clinician, etc.).
2. **Temporal ordering:** `timestamp` (ISO 8601 UTC) enables sorting/filtering by time.
3. **Version explicit:** `version.operator` and `version.build` make the operator version queryable.
4. **Audit query examples:**
   - SQL: `SELECT * FROM substrate WHERE provenance->>'version'->'operator' = 'sofa_resp' AND provenance->>'version'->'build' = '0.2.0'`
   - Informal: "Which SOFA-respiratory v0.2.0 hypotheses were derived from this patient?"
5. **Audit trail preservation:** Provenance is immutable once attached; no post-hoc editing.

**Implementation check (M1):**

In `clinlat/docs/obligations/obl-ps-04-provenance-audit.md`:
- All operator outputs carry non-null provenance with origin, timestamp, version.
- Property test: `∀ Op ∈ OperatorSet: Op.apply(H, E) → Outcome.Refined(H') ⟹ provenance(H') ∉ {null, empty}`.
- Worked example: Trace a SOFA-respiratory hypothesis back to the source lab value through provenance chain.

---

## 5. Examples

### 5.1 Lab Evidence → SOFA-Respiratory Hypothesis

**Input Evidence:**

```json
{
  "observations": [
    {
      "code": "LOINC:2160-0",
      "value": 98,
      "unit": "mg/dL",
      "source": "Epic LIS"
    }
  ],
  "provenance": {
    "origin": {
      "source_type": "external_lab_api",
      "system": "LOINC",
      "identifier": "2160-0"
    },
    "timestamp": "2026-05-25T09:15:00Z",
    "version": {
      "system": "clinlat",
      "operator": "lab_ingest",
      "build": "0.1.0"
    },
    "metadata": {
      "lab_system": "epic_lis",
      "specimen_id": "LAB-2026-05-25-001"
    }
  }
}
```

**Output Hypothesis (after SofaRespOperator.apply):**

```
Hyp {
  condition: Score1,  // PaO2/FiO2 300–399
  provenance: {
    origin: {
      source_type: "operator_derivation",
      system: "SNOMED",
      identifier: "67822003"  // hypoxemia
    },
    timestamp: "2026-05-25T09:15:02Z",
    version: {
      system: "clinlat",
      operator: "sofa_resp",
      build: "0.2.0"
    },
    metadata: {
      prior_hyp: "Unknown",
      pao2_value: 98,
      input_lab_code: "LOINC:2160-0"
    }
  }
}
```

**Audit query** (demonstrate OBL-PS-04):

```sql
-- "Find all hypotheses derived under SOFA-respiratory v0.2.0"
SELECT patient_id, hypothesis, timestamp
FROM substrate
WHERE provenance->>'version'->>'operator' = 'sofa_resp'
  AND provenance->>'version'->>'build' = '0.2.0'
ORDER BY timestamp;

-- Output:
-- patient_id | hypothesis | timestamp
-- 12345      | Score1     | 2026-05-25T09:15:02Z
```

### 5.2 Hypothetical: Clinician Input (Non-Structured)

**Input:**

Clinician types "Pneumonia suspected based on chest X-ray findings."

**Captured Evidence (M4 interaction-layer modeling):**

```json
{
  "observations": [
    {
      "code": "unstructured",
      "text": "Pneumonia suspected; CXR shows infiltrates"
    }
  ],
  "provenance": {
    "origin": {
      "source_type": "clinician_input",
      "system": "Unstructured",
      "identifier": "free_text"
    },
    "timestamp": "2026-05-25T10:30:00Z",
    "version": {
      "system": "clinlat",
      "operator": "clinician_intake",
      "build": "0.1.0"
    },
    "metadata": {
      "clinician_id": "MD-42",
      "institution": "Tertiary Care Hospital",
      "modality": "voice_transcription"
    }
  }
}
```

**Audit trail:** Later queries can trace this unstructured input through the substrate, noting that it was human-entered (source: clinician_input) and transcribed (metadata.modality).

---

## 6. Encoding and Compression

### 6.1 JSON Representation

- **Canonical form:** serde_json with deterministic field ordering.
- **Size:** ~1–2 KB per provenance instance (uncompressed).
- **Compression:** Gzip reduces to ~200–300 bytes per instance (85% reduction).

### 6.2 Example Serialized Bytes

**Uncompressed (pretty-printed):**

```
~1500 bytes (as shown in section 5.1 example)
```

**Gzip compressed:**

```
~250 bytes
```

**Compression commands (Rust):**

```rust
use flate2::Compression;
use flate2::write::GzEncoder;

let json = serde_json::to_string(&provenance)?;
let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
encoder.write_all(json.as_bytes())?;
let compressed = encoder.finish()?;  // ~250 bytes for typical instance
```

---

## 7. Evolution (M5+ Preview)

### 7.1 Optional `derives_from` Field

In M5, the `derives_from` field becomes populated with hash references to ancestor provenance:

```json
{
  "...other fields...",
  "derives_from": [
    "sha256:abc123...",  // hash of prior provenance that this one derives from
    "sha256:def456..."   // optional second ancestor
  ]
}
```

This enables:
- **Derivation chain queries:** Find all hypotheses that transitively derive from evidence E.
- **Audit depth:** Trace a modern hypothesis back through a sequence of operator applications.
- **Impact analysis:** "If we revoke evidence E (e.g., lab error detected), which downstream hypotheses are affected?"

### 7.2 Version Evolution (M5 re-review requirement)

When operator version changes (e.g., SOFA-respiratory from v0.2.0 → v0.3.0), the provenance.version field changes. Per INV-TE-04, this change triggers M5 re-review (clinician sees diff, decides keep/replace). Provenance audit trail makes this re-review trackable.

---

## 8. References

- **SPEC.md §0.2:** Mathematical preliminaries and notation
- **SPEC.md §2:** Patient-state substrate definitions (DEF-PS-01 through DEF-PS-15)
- **SPEC.md §2.4:** Provenance definitions (DEF-MP-14, DEF-PS-12, DEF-PS-13)
- **SPEC.md §5:** Temporal evolution (DEF-TE-07, INV-TE-04)
- **SPEC.md §6.1:** Proof obligations (OBL-PS-04)
- **DESIGN-D1-provenance.md:** Architectural decision on JSON encoding
- **clinlat/docs/obligations/obl-ps-04-provenance-audit.md:** M1 discharge proof (informal-argument tier)

---

## 9. Sign-off

**Specification approved:** Yes, ready for M1.2 implementation.
**Date:** 2026-05-25
**Next:** M1.2 task 2.1 — Implement `Provenance` struct in Rust per this spec.
