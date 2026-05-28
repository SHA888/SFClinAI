# `clinlat` — Clinical Lattice Types

A Rust substrate for symbolic clinical decision-making based on refinable hypothesis lattices and sound deduction operators.

**Version:** 0.2.0-alpha.0
**Status:** M1 milestone in progress (Patient substrate completion; Phases 0–3 of 7 shipped)

## Overview

The `clinlat` crate provides:

- **`Hyp`**: A poset (partially ordered set) of clinical hypotheses, ordered by refinement (specificity).
- **`Outcome<H, A>`**: A result type for operator outputs (refined hypothesis or abstention).
- **Deduction operators**: Sound functions that refine hypotheses using clinical evidence (e.g., SOFA-3 respiratory scoring).

This implements the **patient-state substrate** from [Substrate-First Clinical AI](https://github.com/SHA888/SFClinAI), a position arguing that clinical AI safety must be enforced by symbolic reasoning, not learned components alone.

## Quick Start

### Creating a Hypothesis

```rust
use clinlat::{Atom, Hyp, OntologySystem};

// The top element (most general hypothesis).
let unknown = Hyp::unknown();

// A specific hypothesis. Atoms are resolved through ontology adapters
// (SNOMED CT, RxNorm, LOINC, ICD-11); see `clinlat::ontology`.
let hypoxemia = Atom {
    system: OntologySystem::SNOMED,
    code: "67822003".to_string(),
    preferred_term: "Hypoxemia".to_string(),
    version: "2026-01-31".to_string(),
};
let diagnosis = Hyp::new(vec![hypoxemia]);
```

### Refinement Ordering

```rust
use clinlat::{Atom, Hyp, OntologySystem};
use std::cmp::Ordering;

let unknown = Hyp::unknown();
let specific = Hyp::new(vec![Atom {
    system: OntologySystem::SNOMED,
    code: "clinlat-sofa-resp-3".to_string(),
    preferred_term: "SOFA respiratory score 3".to_string(),
    version: "0.2.0".to_string(),
}]);

// `specific` refines (is more specific than) `unknown`.
assert_eq!(specific.partial_cmp(&unknown), Some(Ordering::Less));
```

### Evidence with Provenance

```rust
use std::collections::BTreeMap;
use chrono::Utc;
use clinlat::{Evidence, Observation, Provenance, ProvenanceOrigin, Ver};

let observations = vec![
    Observation::new("LOINC:2703-7", serde_json::json!(98.0))
        .with_unit("mmHg")
        .with_source("Epic LIS"),  // PaO₂
    Observation::new("LOINC:3150-0", serde_json::json!(1.0))
        .with_source("Epic LIS"),  // FiO₂
];

let provenance = Provenance::new(
    ProvenanceOrigin::new("external_lab_api", "LOINC", "2703-7"),
    Utc::now(),
    Ver::new("clinlat", "lab_ingest", "0.1.0"),
    BTreeMap::new(),
);

let evidence = Evidence::new(observations, provenance);
```

### SOFA-3 Respiratory Operator

```rust
use clinlat::{Hyp, Operator, Outcome, SofaRespOperator};
# use std::collections::BTreeMap;
# use chrono::Utc;
# use clinlat::{Evidence, Observation, Provenance, ProvenanceOrigin, Ver};
# let evidence = Evidence::new(
#     vec![
#         Observation::new("LOINC:2703-7", serde_json::json!(350.0)).with_unit("mmHg"),
#         Observation::new("LOINC:3150-0", serde_json::json!(1.0)),
#     ],
#     Provenance::new(
#         ProvenanceOrigin::new("external_lab_api", "LOINC", "2703-7"),
#         Utc::now(),
#         Ver::new("clinlat", "lab_ingest", "0.1.0"),
#         BTreeMap::new(),
#     ),
# );

let operator = SofaRespOperator::default_v0_2();
let outcome = operator.apply(&Hyp::unknown(), &evidence);

match outcome {
    Outcome::Refined(h) => println!("Refined to: {:?}", h),
    Outcome::Abstain(reason) => println!("Abstained: {:?}", reason),
}
```

Lower-level `score_from_ratio(ratio, on_mech_vent) -> Option<u8>` is also exposed
as a standalone numeric helper (no Evidence/Provenance plumbing) for callers that
just need the SOFA mapping in isolation.

## Architecture

### Hypothesis Lattice

Hypotheses are ordered by refinement:

```
        Unknown (top)
           / | \
      Score0 Score1 ... Score4 (bottom elements, incomparable to each other)
```

In this lattice:

- **Unknown** is the greatest element (least specific; no information).
- **Score{N}** elements are minimal (each is a specific diagnostic score).
- The **partial meet** of two elements is their greatest lower bound (if it exists).

### Operators

An `Operator` is a function from a hypothesis and evidence to an outcome:

```
apply: (Hyp, Evidence) → Outcome<Hyp, AbstainReason>
```

The operator either:

- **Refines** the hypothesis to a more specific one based on evidence.
- **Abstains** with a reason (e.g., insufficient evidence, precondition unmet).

### SOFA-3 Respiratory Component

The SOFA-3 operator scores respiratory dysfunction severity via the PaO₂/FiO₂ ratio:

| Ratio (mmHg) | Score | Meaning |
|--------------|-------|---------|
| ≥ 400 | 0 | No dysfunction |
| 300–399 | 1 | Mild |
| 200–299 | 2 | Moderate |
| 100–199 | 3 | Severe (requires ventilation) |
| < 100 | 4 | Very severe (requires ventilation) |

Per Sepsis-3, scores 3 and 4 are only valid for intubated patients; the operator abstains if this precondition is unmet.

## Soundness

Each operator carries a soundness argument establishing three properties:

1. **Refinement monotonicity**: If h₁ ⊑ h₂, operator output on h₁ refines that on h₂.
2. **No spurious refinement**: Output never exceeds what the evidence justifies.
3. **Abstention purity**: Abstention is structural, not error handling.

See [`clinlat/docs/operators/sofa_resp_soundness.md`](docs/operators/sofa_resp_soundness.md) for the SOFA-3 argument.

## v0.2.0-alpha Status

What has shipped in this pre-release:

- **`Atom`** (Phase 1): replaces `&'static str` AtomId with `{ system, code, preferred_term, version }`. Resolved through four `OntologyAdapter` implementations: SNOMED CT, RxNorm, LOINC, ICD-11.
- **`Evidence`** (Phase 2): typed `{ observations: Vec<Observation>, provenance: Provenance }` carrying clinical observations and audit-trail provenance (DEF-PS-12, DEF-PS-13).
- **`Provenance`** (Phase 2): typed carrier with origin, ISO 8601 timestamp, operator version, metadata, and optional `derives_from` hashes (DEF-MP-14, OBL-PS-04). JSON serializable with optional gzip compression.
- **`SofaRespOperator`** (Phase 2): full `Operator::apply()` implementation with version-respecting derivation chain enforcement — the operator abstains rather than silently process evidence whose provenance version does not match (INV-PS-05).
- **Galois connection** (Phase 3): abstraction `abstract_evidence` (α_PS) and the concretization predicate `is_consistent_with` (γ_PS) property-tested for the adjunction laws — `e ∈ γ_PS(α_PS(e))`, `α_PS(γ_PS(h)) ⊑ h`, and monotonicity — discharging OBL-PS-02 at the property-test tier. See [`docs/obligations/obl-ps-02-adjunction.md`](docs/obligations/obl-ps-02-adjunction.md). (A few post-review test-coverage cleanups, Plans.md tasks 3.5–3.7, remain before Phase 4.)

What remains for the v0.2.0 release (Phases 4–7 of Plans.md):

- **`OperatorSet`** type and composition formalized per DEF-PS-09 / OBL-PS-03 (Phase 4).
- **Three additional operators**: KDIGO AKI, Wells/PE, CURB-65 (Phase 5).
- **SOFA-respiratory upgrade** from informal-argument tier to property-test tier (Phase 6).
- **Release prep**: README/SPEC cross-references, full CI green, publish dry-run (Phase 7).

Pre-release versioning (`0.2.0-alpha.N`) is used while these phases land; the suffix is dropped on the cut to `0.2.0`.

## References

- **Position note**: [Substrate-First Clinical AI (NOTE.md)](../NOTE.md) § 4A–4D (eighteen load-bearing principles).
- **Formalization**: [SPEC.md](../SPEC.md) § 2 (patient-state substrate definitions and proof obligations).
- **Clinical references**:
  - Vincent JL, et al. The SOFA (Sepsis-related Organ Failure Assessment) score to describe organ dysfunction/failure. *Intensive Care Medicine*. 1996;22(7):707–710.
  - Singer M, et al. The Third International Consensus Definitions for Sepsis and Septic Shock (Sepsis-3). *JAMA*. 2016;315(8):801–810.

## Testing

All public types and functions have unit tests:

```bash
cargo test
```

Documentation tests are included in rustdoc:

```bash
cargo test --doc
```

## Building Documentation

Generate full API documentation with cross-references:

```bash
cargo doc --no-deps --open
```

## License

Code: **MIT OR Apache-2.0** (see `LICENSE-MIT` and `LICENSE-APACHE` at repo root).

## Contributing

Contributions follow the repository contribution order (see [`CONTRIBUTING.md`](../CONTRIBUTING.md)):

1. Show the synthesis is already published (and was missed in prior art).
2. Demonstrate one of the eighteen principles is wrong or unnecessary.
3. Show the substrate framing fails on a clinical decision not covered by the worked examples.

For questions or issues, see the main repository: <https://github.com/SHA888/SFClinAI>
