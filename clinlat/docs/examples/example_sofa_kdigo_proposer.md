# Worked Example: SOFA + KDIGO Proposer on Sepsis-3 Patient

**Task**: Phase 9.3 of M2 (Constrained refinement proposer)
**Date**: 2026-06-06
**Reference**: SPEC.md §2.7 (DEF-PS-14, DEF-PS-15), NOTE.md §4A.5, ARCHITECTURE.md Diagram 3 and 5

## Overview

This worked example demonstrates the `LatticeSearchProposer` (Phase 9, M2.4) in action, using both the **SOFA respiratory** and **KDIGO AKI** operators to generate candidate refinements for a realistic **sepsis-3 patient state**.

The example is self-contained and runnable:

```bash
cd clinlat && cargo run --example sofa_kdigo_proposer
```

## Clinical Scenario

**Patient A** (64-year-old, 70 kg):
- **Presentation**: Community-acquired pneumonia (CAP) with fever, cough, SOB × 2 days
- **Current status**: Intubated on mechanical ventilation in ICU
- **Concern**: Respiratory failure + worsening kidney function

**Laboratory findings** (at 2026-06-06 08:00):
- **Arterial blood gas**: PaO₂ = 150 mmHg, FiO₂ = 0.60 (on ventilator)
  - PaO₂/FiO₂ ratio = 250 mmHg
- **Kidney function**:
  - Baseline serum creatinine (admission, 2 days ago): 1.0 mg/dL
  - Current serum creatinine (today): 2.4 mg/dL
  - Fold-change = 2.4×

## The Substrate Workflow

### Step 1: Construct Evidence

Evidence is collected as observations with provenance (timestamp, source, version):

```rust
let observations = vec![
    Observation::new("LOINC:2703-7", serde_json::json!(150.0)) // PaO₂
        .with_source("ABG from 14:30"),
    Observation::new("LOINC:3150-0", serde_json::json!(0.60))   // FiO₂
        .with_source("Ventilator settings"),
    Observation::new("SNOMED:243144002", serde_json::json!(true)) // On mech vent
        .with_source("Ventilator active"),
    Observation::new("LOINC:2160-0-baseline", serde_json::json!(1.0)) // Baseline Cr
        .with_source("Admission labs"),
    Observation::new("LOINC:2160-0-current", serde_json::json!(2.4))  // Current Cr
        .with_source("Today's labs"),
];

let provenance = Provenance::new(
    ProvenanceOrigin::new("epic_lms", "LOINC", "2703-7"),
    Utc::now(),
    Ver::new("clinlat", "sofa_resp", "0.2.0"),
    BTreeMap::new(),
);

let evidence = Evidence::new(observations, provenance);
```

**Provenance note**: The version must match the operator's expected version (INV-PS-05). In this example, we use SOFA as the primary component; KDIGO will also process the same evidence.

### Step 2: Instantiate Operators

```rust
let sofa_op = SofaRespOperator::default_v0_2();
let kdigo_op = KdigoAkiOperator::new("0.2.0");
```

### Step 3: Build OperatorSet and Proposer

```rust
let op_set = OperatorSet::new()
    .register(
        Box::new(sofa_op),
        OperatorMetadata {
            name: "sofa_resp".to_string(),
            version: "clinlat-v0.2.0".to_string(),
        },
    )
    .register(
        Box::new(kdigo_op),
        OperatorMetadata {
            name: "kdigo_aki".to_string(),
            version: "clinlat-v0.2.0".to_string(),
        },
    );

let proposer = LatticeSearchProposer::new(op_set);
```

### Step 4: Call the Proposer

```rust
let input = Hyp::unknown();  // Most general hypothesis
let candidates = proposer.propose(&input, &evidence);
```

## Results

The proposer generates **two candidates**:

### Candidate 1: SOFA Respiratory Score 2

```
SOFA respiratory score 2 (SNOMED:clinlat-sofa-resp-2)
Reasoning:
  • PaO₂/FiO₂ ratio = 150 / 0.60 = 250 mmHg
  • 250 falls in the 200–299 range (SOFA thresholds)
  • → Score 2 (moderate respiratory dysfunction)
  • Patient is on mechanical ventilation (score 2 allowed without vent requirement)
```

**Refinement**: `Unknown` → `SOFA Score 2` ✓

### Candidate 2: KDIGO AKI Stage 2

```
KDIGO AKI Stage 2 (SNOMED:KDIGO-AKI-STAGE-2)
Reasoning:
  • Current Cr = 2.4 mg/dL, Baseline = 1.0 mg/dL
  • Fold-change = 2.4 / 1.0 = 2.4×
  • 2.4× falls in the 2.0–2.9× range (Stage 2)
  • → Stage 2 (moderate kidney injury)
```

**Refinement**: `Unknown` → `KDIGO Stage 2` ✓

## Why Both Operators Produce Candidates

1. **Exhaustive search**: `LatticeSearchProposer` applies each operator to the input hypothesis
2. **Independent evidence**: Each operator extracts its own relevant observations:
   - SOFA: extracts PaO₂, FiO₂, ventilation status
   - KDIGO: extracts baseline Cr, current Cr
3. **Union of refinements**: The proposer returns the set union of all operator outputs
4. **Candidate diversity**: Each candidate represents a single-operator refinement

## Soundness Properties (DEF-PS-15)

Both candidates are **trivially sound** by construction:

| Property | Evidence |
|----------|----------|
| **Operator-reachable** | Each candidate was produced by exactly one operator (SOFA or KDIGO) |
| **Ontology-bounded** | Both atoms (SNOMED codes) are resolvable through the SNOMED ontology adapter |
| **No spurious candidates** | The proposer only collects operator-produced refinements; it cannot generate candidates from thin air |
| **Soundness gate compatibility** | When passed through `propose_verify` (soundness-verification gate), both candidates will be licensed because `OperatorSet.apply_set()` will recognize them as reachable |

**INV-PS-06 (Proposer cannot bypass soundness)**: Even if a proposer is adversarial or hallucinating, the soundness of the active hypothesis depends only on the deduction operators, not on the proposer's behavior.

## Design Insights

### Conservative Operator Abstention

This example also illustrates **structural abstention** (DEF-PS-11):

- **Urine output data** is clinically available (ICU nursing records) but **excluded** from this example
- **Reason**: KDIGO requires validation of the temporal window (6–24 hours)
- **Operator behavior**: The KDIGO operator abstains if UO rate is present without temporal metadata
- **Lesson**: Operators are conservative; they abstain rather than guess at missing context

This is a feature, not a bug:
- Prevents false confidence from incomplete information
- Allows the substrate to escalate or request additional evidence
- Enables structured audit trails (operator abstentions are recorded)

### Provenance Version Checking

Notice the provenance version `Ver::new("clinlat", "sofa_resp", "0.2.0")`:
- **INV-PS-05** (Version-respecting derivation chains): Operators check that input evidence matches their version
- **Why**: Ensures that changes to operator semantics don't silently process stale evidence
- **Clinical implication**: Evidence must be explicitly re-validated if operators are upgraded

## Multi-Step Refinement

The proposer can be called iteratively:

```rust
let candidates = proposer.propose(&Hyp::unknown(), &evidence);   // Step 1
for candidate in candidates {
    let deeper = proposer.propose(&candidate, &evidence);        // Step 2+
    // ...
}
```

However, in this example, operators typically produce a final diagnostic score and return self-identity when called again on that score. The `LatticeSearchProposer` filters out self-loops (DEF-PS-15), preventing infinite loops.

## Completeness Guarantee

For bounded operator sets (≤5 operators), the proposer exhaustively enumerates all single-step refinements. The **completeness property** (Property 9.2 of M2) guarantees:

- **(1) Completeness**: Every hypothesis reachable by one operator application is in the output
- **(2) Minimality**: No spurious candidates (filtered by definition)
- **(3) Monotonicity**: Output candidates form a refinement chain

## References

- **SPEC.md §2.7**: DEF-PS-14 (Refinement proposer signature), DEF-PS-15 (Proposer codomain constraint)
- **SPEC.md §2.7**: INV-PS-06 (Proposer cannot bypass soundness)
- **NOTE.md §4A.5**: Constrained refinement proposer (Position note principle)
- **ARCHITECTURE.md Diagram 3**: Substrate-learned-component boundary (proposer input/output gates, soundness-verification node)
- **ARCHITECTURE.md Diagram 5**: Learned-component composition (proposer slot `RP` filled by LatticeSearchProposer)

## Running the Example

```bash
cd clinlat
cargo run --example sofa_kdigo_proposer
```

Output shows the step-by-step substrate workflow and the final candidate set.
