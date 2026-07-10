# `clinlat` — Clinical Lattice Types

A Rust substrate for symbolic clinical decision-making based on refinable hypothesis lattices and sound deduction operators.

**Version:** 0.3.0
**Status:** M2 milestone complete (Constrained refinement proposer)

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

### KDIGO AKI Staging Operator

```rust
use clinlat::{Hyp, Operator, Outcome, KdigoAkiOperator};
# use std::collections::BTreeMap;
# use chrono::Utc;
# use clinlat::{Evidence, Observation, Provenance, ProvenanceOrigin, Ver};
# let evidence = Evidence::new(
#     vec![
#         Observation::new("LOINC:2160-0-baseline", serde_json::json!(1.2)),  // baseline Cr
#         Observation::new("LOINC:2160-0-current", serde_json::json!(2.8)),   // current Cr
#     ],
#     Provenance::new(
#         ProvenanceOrigin::new("lab_api", "LOINC", "2160-0"),
#         Utc::now(),
#         Ver::new("clinlat", "lab_ingest", "0.1.0"),
#         BTreeMap::new(),
#     ),
# );

let operator = KdigoAkiOperator::new("0.2.0");
let outcome = operator.apply(&Hyp::unknown(), &evidence);

match outcome {
    Outcome::Refined(h) => {
        // Refined to KDIGO AKI Stage 2 (creatinine 2.3x baseline)
        println!("AKI staging: {:?}", h);
    }
    Outcome::Abstain(reason) => println!("Cannot stage: {:?}", reason),
}
```

The KDIGO AKI operator stratifies kidney injury severity by serum creatinine fold-change and urine output decline, per the Kidney Disease: Improving Global Outcomes 2021 guideline. Stage assignments (0–3) drive admission and dialysis decisions in critical care.

See [`clinlat/docs/operators/kdigo_aki_soundness.md`](docs/operators/kdigo_aki_soundness.md) for soundness argument.

### Wells PE Risk Stratification Operator

```rust
use clinlat::{Hyp, Operator, Outcome, WellsPeOperator};
# use std::collections::BTreeMap;
# use chrono::Utc;
# use clinlat::{Evidence, Observation, Provenance, ProvenanceOrigin, Ver};
# let evidence = Evidence::new(
#     vec![
#         Observation::new("PE-LIKELY", serde_json::json!(true)),      // gestalt
#         Observation::new("DVT-SIGNS", serde_json::json!(true)),      // +3 points
#         Observation::new("HEART-RATE", serde_json::json!(115.0)),    // +1.5 points
#     ],
#     Provenance::new(
#         ProvenanceOrigin::new("manual_entry", "clinician", "wells"),
#         Utc::now(),
#         Ver::new("clinlat", "clinical_assessment", "0.1.0"),
#         BTreeMap::new(),
#     ),
# );

let operator = WellsPeOperator::new("0.2.0");
let outcome = operator.apply(&Hyp::unknown(), &evidence);

match outcome {
    Outcome::Refined(h) => {
        // Refined to PE-LIKELY (score 4.5 > 4.0 threshold)
        // Next step: CTPA imaging indicated
        println!("PE risk: {:?}", h);
    }
    Outcome::Abstain(reason) => println!("Cannot score: {:?}", reason),
}
```

The Wells PE operator implements cumulative clinical scoring for pulmonary embolism risk, per Wells et al. (1997/2006). The gestalt assessment ("is PE your leading diagnosis?") is mandatory—the operator enforces clinician judgment as a core input, not an optional feature. Scores ≤4 → D-dimer testing (rule-out); scores >4 → CTPA imaging (rule-in).

See [`clinlat/docs/operators/wells_pe_soundness.md`](docs/operators/wells_pe_soundness.md) for soundness argument.

### CURB-65 CAP Disposition Operator

```rust
use clinlat::{Hyp, Operator, Outcome, Curb65Operator};
# use std::collections::BTreeMap;
# use chrono::Utc;
# use clinlat::{Evidence, Observation, Provenance, ProvenanceOrigin, Ver};
# let evidence = Evidence::new(
#     vec![
#         Observation::new("CONFUSION", serde_json::json!(false)),      // no confusion
#         Observation::new("UREA", serde_json::json!(8.5)),             // >7 mmol/L → +1
#         Observation::new("RESP-RATE", serde_json::json!(32.0)),       // ≥30 → +1
#         Observation::new("SBP", serde_json::json!(92.0)),             // not <90
#         Observation::new("AGE", serde_json::json!(71.0)),             // ≥65 → +1
#     ],
#     Provenance::new(
#         ProvenanceOrigin::new("ward_vitals", "manual", "cap_assessment"),
#         Utc::now(),
#         Ver::new("clinlat", "cap_scoring", "0.1.0"),
#         BTreeMap::new(),
#     ),
# );

let operator = Curb65Operator::new("0.2.0");
let outcome = operator.apply(&Hyp::unknown(), &evidence);

match outcome {
    Outcome::Refined(h) => {
        // Refined to WARD-ADMISSION (score 3)
        println!("CAP disposition: {:?}", h);
    }
    Outcome::Abstain(reason) => println!("Cannot score: {:?}", reason),
}
```

The CURB-65 operator stratifies community-acquired pneumonia (CAP) severity for disposition decisions, per the British Thoracic Society and IDSA/ATS guidelines. Scores 0–1 → outpatient; score 2 → ward admission; scores 3–5 → ICU evaluation with possible critical care.

See [`clinlat/docs/operators/curb65_soundness.md`](docs/operators/curb65_soundness.md) for soundness argument.

### Constrained Refinement Proposer (M2)

A [`RefinementProposer`] is a black-box that suggests candidate refinements —
it never decides on its own; every candidate must still pass the ontology
gates ([`ProposerConstraint`], DEF-PS-15) and the soundness-verification gate
(`propose_verify`, the Diagram 3 `SV` node) before it can be used. This is
what lets `clinlat` plug in an untrusted or even hallucinating proposer (see
the [`LlmProposer`](#llm-class-proposer-m2) below) without weakening the
substrate's safety guarantee.

```rust
use clinlat::{Hyp, LatticeSearchProposer, OperatorMetadata, OperatorSet, SofaRespOperator, propose_verify};
# use std::collections::BTreeMap;
# use chrono::Utc;
# use clinlat::{Evidence, Observation, Provenance, ProvenanceOrigin, Ver};
# let evidence = Evidence::new(
#     vec![
#         Observation::new("LOINC:2703-7", serde_json::json!(180.0)).with_unit("mmHg"),
#         Observation::new("LOINC:3150-0", serde_json::json!(1.0)),
#     ],
#     Provenance::new(
#         ProvenanceOrigin::new("external_lab_api", "LOINC", "2703-7"),
#         Utc::now(),
#         Ver::new("clinlat", "lab_ingest", "0.1.0"),
#         BTreeMap::new(),
#     ),
# );

// Independent OperatorSet instances — one drives the proposer's search space,
// one drives the soundness gate — per this repo's test/usage discipline that
// shared-instance construction can mask non-determinism (see CLAUDE.md).
let search_operators = OperatorSet::new().register(
    Box::new(SofaRespOperator::default_v0_2()),
    OperatorMetadata { name: "SofaRespOperator".to_string(), version: "0.2.0".to_string() },
);
let gate_operators = OperatorSet::new().register(
    Box::new(SofaRespOperator::default_v0_2()),
    OperatorMetadata { name: "SofaRespOperator".to_string(), version: "0.2.0".to_string() },
);

let proposer = LatticeSearchProposer::new(search_operators);

match propose_verify(&proposer, &gate_operators, &Hyp::unknown(), &evidence) {
    Ok(result) => {
        // Every candidate here is provably reachable by an operator (INV-PS-06).
        println!("Licensed candidates: {:?}", result.licensed_candidates);
    }
    Err(reason) => println!("Abstained: {:?}", reason),
}
```

`LatticeSearchProposer` is trivially sound by construction (exhaustive search
over one-operator-step reachable hypotheses). See
[`docs/examples/example_sofa_kdigo_proposer.md`](docs/examples/example_sofa_kdigo_proposer.md)
for a worked SOFA + KDIGO example.

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

Each operator carries a soundness argument establishing three properties per **DEF-PS-08** (Soundness of a deduction operator):

1. **Refinement monotonicity (INV-PS-03)**: If h₁ ⊑ h₂, operator output on h₁ refines that on h₂.
2. **No spurious refinement**: Output never exceeds what the evidence justifies.
3. **Abstention purity (INV-PS-04)**: Abstention is structural, not error handling.

Soundness arguments are provided for all M1 operators:

- **SOFA-3 respiratory:** [`docs/operators/sofa_resp_soundness.md`](docs/operators/sofa_resp_soundness.md) (property-test tier: 46 tests)
- **KDIGO AKI:** [`docs/operators/kdigo_aki_soundness.md`](docs/operators/kdigo_aki_soundness.md) (informal-argument tier)
- **Wells PE:** [`docs/operators/wells_pe_soundness.md`](docs/operators/wells_pe_soundness.md) (informal-argument tier)
- **CURB-65 CAP:** [`docs/operators/curb65_soundness.md`](docs/operators/curb65_soundness.md) (informal-argument tier)

All soundness arguments discharge **OBL-PS-03** (Operator set soundness) and satisfy **INV-PS-01**–**INV-PS-06** (patient-substrate invariants).

**Formal reference:** See [`SPEC.md` §2 (Patient-state substrate)](../SPEC.md) and [`SPEC.md` §8 (Bidirectional traceability)](../SPEC.md) for the complete formalization of operator soundness and the mapping from principles (NOTE.md §4A) to formal definitions.

## Status

**M1 (Patient substrate completion, `v0.2.0`, shipped 2026-05-31):** all eleven
4A-anchored SPEC.md elements (DEF-PS-01..15, INV-PS-01..06, OBL-PS-01..05)
reachable from running code; four operators discharged (SOFA at
property-test tier, KDIGO/Wells/CURB-65 at informal-argument tier).

**M2 (Constrained refinement proposer, `v0.3.0`) — complete:**

- **`RefinementProposer` / `ProposerConstraint`** (Phase 8): black-box
  proposer interface (DEF-PS-14) with input- and output-side ontology gates
  (DEF-PS-15); `propose_and_filter` and `propose_verify` adapters wire
  proposer output through the soundness-verification gate with structured
  abstention (`AbstainReason::NoOperatorLicenses`); INV-PS-06 enforced by a
  dedicated structural test, not argument alone.
- **`LatticeSearchProposer`** (Phase 9): exhaustive one-operator-step search,
  trivially sound by construction; ~27 property cases for completeness,
  minimality, and monotonicity.
- **`LlmProposer`** (Phase 10): foundation-model adapter (with offline mock
  mode for CI) demonstrating the substrate-first claim — hallucinated
  candidates are filtered, valid candidates pass through, and system
  behavior stays sound either way.
- **OBL-PS-05 discharge + substrate-invariance** (Phase 11): property-test
  tier discharge across both proposers, plus a ≥10-case paired test proving
  identical post-soundness-gate refinement across a proposer swap for the
  same evidence — see [`docs/obligations/obl-ps-05-proposer-constraint.md`](docs/obligations/obl-ps-05-proposer-constraint.md).

See the [Constrained Refinement Proposer](#constrained-refinement-proposer-m2)
section above for usage, and `CHANGELOG.md` for the full `[0.3.0]` entry.

Test coverage: **299 tests passing** (all operators, both proposers, all phases).

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
