# `clinlat` — Clinical Lattice Types

A Rust substrate for symbolic clinical decision-making based on refinable hypothesis lattices and sound deduction operators.

**Version:** 0.1.0  
**Status:** Early implementation (kernel milestone)

## Overview

The `clinlat` crate provides:

- **`Hyp`**: A poset (partially ordered set) of clinical hypotheses, ordered by refinement (specificity).
- **`Outcome<H, A>`**: A result type for operator outputs (refined hypothesis or abstention).
- **Deduction operators**: Sound functions that refine hypotheses using clinical evidence (e.g., SOFA-3 respiratory scoring).

This implements the **patient-state substrate** from [Substrate-First Clinical AI](https://github.com/SHA888/SFClinAI), a position arguing that clinical AI safety must be enforced by symbolic reasoning, not learned components alone.

## Quick Start

### Creating a Hypothesis

```rust
use clinlat::Hyp;

// The top element (most general hypothesis).
let unknown = Hyp::unknown();

// A specific hypothesis (in v0.1.0, atoms are static strings).
let diagnosis = Hyp::new(vec!["sepsis_3", "respiratory_dysfunction"]);
```

### Refinement Ordering

```rust
use clinlat::Hyp;
use std::cmp::Ordering;

let unknown = Hyp::unknown();
let specific = Hyp::new(vec!["sofa_score_3"]);

// `specific` refines (is more specific than) `unknown`.
assert_eq!(specific.partial_cmp(&unknown), Some(Ordering::Less));
```

### Compatibility and Meet

```rust
use clinlat::Hyp;

let h1 = Hyp::new(vec!["diagnosis_a"]);
let h2 = Hyp::new(vec!["diagnosis_b"]);
let unknown = Hyp::unknown();

// h1 and h2 are incompatible (orthogonal diagnoses).
assert!(!h1.compat(&h2));

// Any hypothesis is compatible with unknown.
assert!(h1.compat(&unknown));

// Meet of h1 and unknown is h1 itself.
assert_eq!(h1.meet(&unknown), Some(h1.clone()));

// Meet of incomparable hypotheses is None.
assert_eq!(h1.meet(&h2), None);
```

### SOFA-3 Respiratory Scoring

```rust
use clinlat::sofa::{SofaRespEvidence, score_from_ratio};

// Evidence: PaO₂ = 350 mmHg, FiO₂ = 1.0, not on ventilator.
let evidence = SofaRespEvidence::new(350.0, 1.0, false);
let ratio = evidence.pao2_fio2_ratio().unwrap();  // 350.0

// Map to SOFA score (300–399 → score 1).
let score = score_from_ratio(ratio, false);
assert_eq!(score, Some(1));

// If score ≥3, mechanical ventilation is required.
let severe = score_from_ratio(80.0, false);  // <100 → score 4
assert_eq!(severe, None);  // Precondition unmet (not ventilated).

let severe_ventilated = score_from_ratio(80.0, true);
assert_eq!(severe_ventilated, Some(4));
```

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

## v0.1.0 Simplifications

- **AtomId**: Static string reference (`&'static str`). Real ontology binding (SNOMED CT, RxNorm, LOINC, ICD-11) deferred to v0.2.
- **Evidence**: Unit type in the trait interface. Structured evidence (lab values, timestamps, provenance) deferred to v0.2.
- **Operator trait**: Generic interface; actual operators (like SOFA-3) use concrete evidence types.
- **Provenance**: Unit stub `()`; real lineage and timestamping deferred to v0.2.

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

For questions or issues, see the main repository: https://github.com/SHA888/SFClinAI
