# `clinlat` — Clinical Lattice Types

A Rust substrate for symbolic clinical decision-making based on refinable hypothesis lattices and sound deduction operators.

**Version:** 0.3.0 — see [`CHANGELOG.md`](CHANGELOG.md) for full version history.

## Overview

The `clinlat` crate provides:

- **`Hyp`**: A poset (partially ordered set) of clinical hypotheses, ordered by refinement (specificity).
- **`Outcome<H, A>`**: A result type for operator outputs (refined hypothesis or abstention).
- **Deduction operators**: Sound functions that refine hypotheses using clinical evidence (e.g., SOFA-3 respiratory scoring).
- **Refinement proposers**: Black-box candidate generators (deterministic search, LLM-class) gated by ontology and soundness constraints — see [Constrained Refinement Proposer](#constrained-refinement-proposer-m2) below.

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

### Other operators

Every operator follows the same call pattern as SOFA-3 above — construct `Evidence`, call
`operator.apply(&Hyp::unknown(), &evidence)`, match `Outcome::Refined`/`Outcome::Abstain`:

| Operator | Type | Purpose | Soundness argument |
|----------|------|---------|---------------------|
| KDIGO AKI staging | `KdigoAkiOperator` | Kidney injury severity by creatinine fold-change / urine output, per KDIGO 2021 | [`docs/operators/kdigo_aki_soundness.md`](docs/operators/kdigo_aki_soundness.md) (informal-argument tier) |
| Wells PE risk | `WellsPeOperator` | Cumulative PE risk scoring per Wells et al. 1997/2006; mandatory gestalt input | [`docs/operators/wells_pe_soundness.md`](docs/operators/wells_pe_soundness.md) (informal-argument tier) |
| CURB-65 CAP disposition | `Curb65Operator` | Pneumonia severity/disposition per BTS and IDSA/ATS guidelines | [`docs/operators/curb65_soundness.md`](docs/operators/curb65_soundness.md) (informal-argument tier) |

### Constrained Refinement Proposer (M2)

A [`RefinementProposer`] is a black-box that suggests candidate refinements —
it never decides on its own; every candidate must still pass the ontology
gates ([`ProposerConstraint`], DEF-PS-15) and the soundness-verification gate
(`propose_verify`, the Diagram 3 `SV` node) before it can be used. This is
what lets `clinlat` plug in an untrusted or even hallucinating proposer
without weakening the substrate's safety guarantee.

```rust,ignore
use clinlat::{LatticeSearchProposer, OperatorSet, propose_verify};

// search_operators / gate_operators are independent OperatorSet instances (same
// registrations); hypothesis / evidence as constructed above. Full setup in the
// worked example linked below.
let proposer = LatticeSearchProposer::new(search_operators);
match propose_verify(&proposer, &gate_operators, &hypothesis, &evidence) {
    Ok(result) => println!("Licensed candidates: {:?}", result.licensed_candidates),
    Err(reason) => println!("Abstained: {:?}", reason),
}
```

`LatticeSearchProposer` is trivially sound by construction (exhaustive search
over one-operator-step reachable hypotheses). See
[`docs/examples/example_sofa_kdigo_proposer.md`](docs/examples/example_sofa_kdigo_proposer.md)
for the full runnable SOFA + KDIGO example (including `OperatorSet` setup).

#### LLM-Class Proposer (M2)

`LlmProposer` wraps a foundation-model API call (or an offline mock, for
CI/testing) behind the same `RefinementProposer` interface. The LLM can
hallucinate freely — invalid responses are filtered by `ProposerConstraint`
and logged as filtered candidates — while the substrate's refinement
behavior stays identical to a run through `LatticeSearchProposer` on the
same evidence. See
[`docs/examples/example_llm_proposer_sepsis.md`](docs/examples/example_llm_proposer_sepsis.md)
(hallucination filtered, valid candidate accepted) and
[`docs/examples/example_substrate_invariance_sepsis.md`](docs/examples/example_substrate_invariance_sepsis.md)
(side-by-side proposer swap showing identical substrate output).

Proposer safety is discharged in two documents:

- [`docs/invariants/inv-ps-06-proposer-safety.md`](docs/invariants/inv-ps-06-proposer-safety.md) — proves no proposer output can bypass `OperatorSet::apply_set()`.
- [`docs/obligations/obl-ps-05-proposer-constraint.md`](docs/obligations/obl-ps-05-proposer-constraint.md) — OBL-PS-05 discharge at property-test tier across both reference proposers.

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

## Soundness

Each operator carries a soundness argument establishing three properties per **DEF-PS-08** (Soundness of a deduction operator):

1. **Refinement monotonicity (INV-PS-03)**: If h₁ ⊑ h₂, operator output on h₁ refines that on h₂.
2. **No spurious refinement**: Output never exceeds what the evidence justifies.
3. **Abstention purity (INV-PS-04)**: Abstention is structural, not error handling.

All soundness arguments discharge **OBL-PS-03** (Operator set soundness) and satisfy **INV-PS-01**–**INV-PS-06** (patient-substrate invariants). See the [Other operators](#other-operators) table above for per-operator links; **SOFA-3 respiratory** is at property-test tier ([`docs/operators/sofa_resp_soundness.md`](docs/operators/sofa_resp_soundness.md), 46 tests), the rest at informal-argument tier.

**Formal reference:** See [`SPEC.md` §2 (Patient-state substrate)](../SPEC.md) and [`SPEC.md` §8 (Bidirectional traceability)](../SPEC.md) for the complete formalization of operator soundness and the mapping from principles (NOTE.md §4A) to formal definitions.

## Status

Current release: **`v0.3.0`** — M2 (constrained refinement proposer) complete. Test coverage: 299 tests passing.

For milestone-by-milestone history (M1 patient-substrate completion, M2 proposer interface + reference proposers), see [`CHANGELOG.md`](CHANGELOG.md). For the in-progress milestone (M3, institutional substrate), see the repository root [`Plans.md`](../Plans.md) and [`TODO.md`](../TODO.md).

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
