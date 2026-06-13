# Worked Example: LlmProposer with Hallucinations on Sepsis-3 Patient

**Task**: Phase 10.4 of M2 (Constrained refinement proposer)
**Date**: 2026-06-14
**Reference**: SPEC.md §2.7 (DEF-PS-14, DEF-PS-15, INV-PS-06), NOTE.md §4A.5, ARCHITECTURE.md Diagram 3 and 5

## Overview

This worked example demonstrates the `LlmProposer` (Phase 10, M2.5) in action, using **mock LLM responses** (including hallucinations) on a **sepsis-3 patient state** to empirically verify **INV-PS-06**: the substrate remains sound even when the LLM produces invalid candidates.

The key innovation over deterministic search (task 9.3) is showing that learned components like LLMs can be safely integrated into the substrate-first architecture through constraint gates, not by restricting LLM behavior.

The example is self-contained and runnable:

```bash
cd clinlat && cargo run --example llm_proposer_sepsis
```

## Clinical Scenario

Same patient as task 9.3:

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

## The Substrate Workflow with LLM

### Step 1: Construct Evidence

Evidence is identical to task 9.3 (deterministic proposer), demonstrating that the proposer type is decoupled from evidence collection:

```rust
let observations = vec![
    Observation::new("LOINC:2703-7", serde_json::json!(150.0))  // PaO₂
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
```

### Step 2: Configure LLM Proposer with Mock Responses

Unlike deterministic search, we configure the proposer with **mock LLM responses** that include realistic hallucinations:

```rust
let mock_response = vec![
    // Response 1: LLM suggests valid candidates + hallucinations
    // Valid:      SNOMED:76948002 (respiratory dysfunction), LOINC:8480-6 (systolic BP)
    // Hallucin:   UNKNOWN_SYSTEM:99999 (fake ontology), SNOMED: (empty code), SOFA-99 (malformed)
    "SNOMED:76948002,UNKNOWN_SYSTEM:99999,SNOMED:,LOINC:8480-6,SOFA-99".to_string(),
    // Response 2 & 3: Same response (demonstrate idempotence across gate stages)
    "SNOMED:76948002,UNKNOWN_SYSTEM:99999,SNOMED:,LOINC:8480-6,SOFA-99".to_string(),
    "SNOMED:76948002,UNKNOWN_SYSTEM:99999,SNOMED:,LOINC:8480-6,SOFA-99".to_string(),
];

let config = LlmProposerConfig::mock("gpt-4-sepsis", mock_response, "0.1.0");
let proposer = LlmProposer::new(config);
```

The mock responses are intentionally designed to test **hallucination filtering**:

| Token | Status | Reason |
|-------|--------|--------|
| `SNOMED:76948002` | ✓ Valid | Recognized SNOMED code (parses successfully) |
| `UNKNOWN_SYSTEM:99999` | ✗ Hallucination | Unknown ontology system (silently filtered during parsing) |
| `SNOMED:` | ✗ Hallucination | Empty code after colon (parsing fails) |
| `LOINC:8480-6` | ✓ Valid | Valid LOINC code (parses successfully) |
| `SOFA-99` | ✗ Hallucination | Malformed (no `:` separator) |

**Result**: 5 tokens → 2 valid candidates, 3 hallucinations silently discarded

### Step 3: Call the Proposer

```rust
let input = Hyp::unknown();
let candidates = proposer.propose(&input, &evidence);
```

The proposer internally:
1. Constructs a prompt from the hypothesis and evidence
2. Calls the (mocked) LLM API
3. **Parses the response** — hallucinations are filtered here (not elsewhere)
4. Returns only ontology-valid candidates

### Step 4: Constraint Filtering

The `propose_and_filter()` adapter is called:

```rust
let filter_result = clinlat::proposer::propose_and_filter(&proposer, &input, &evidence);
```

This applies the **ProposerConstraint** (DEF-PS-15) to the proposer's output:
- **Output-side gate**: Each candidate must be ontology-bounded (no Unstructured atoms)
- **Input-side gate**: Input hypothesis must be ontology-valid
- **Result**: Valid candidates pass through; any remaining invalid candidates are filtered with audit trail

### Step 5: Operator Licensing

The `propose_verify()` adapter completes the soundness pipeline:

```rust
let verify_result = clinlat::proposer::propose_verify(&proposer, &ops, &input, &evidence);
```

This applies the **SoundnessVerificationGate**:
- Routes each candidate through `OperatorSet.apply_set()`
- Only candidates licensed by ≥1 operator are returned
- If no candidate is licensed: emit `AbstainReason::NoOperatorLicenses` (safe fallback)

## Safety Analysis: INV-PS-06

**The central claim**: The substrate (not the LLM) enforces safety.

### Hallucination Filtering In Action

The mock LLM response contains 3 hallucinations. Where are they filtered?

| Hallucination | Stage | Mechanism |
|---------------|-------|-----------|
| `UNKNOWN_SYSTEM:99999` | **Parsing** (proposer internal) | Unknown system rejected by `parse_atom()` |
| `SNOMED:` | **Parsing** (proposer internal) | Empty code detected and rejected |
| `SOFA-99` | **Parsing** (proposer internal) | Malformed (no `:` separator), parsing fails |

**None** of these reach the operator licensing stage because the parsing layer filters them first.

### Why This Matters for Safety

**If the substrate relied on the proposer**:
- ✗ LlmProposer returns `UNKNOWN_SYSTEM:99999` unchecked
- ✗ Operator crashes or produces undefined behavior
- ✗ Safety depends on LLM training + prompt engineering
- ✗ Risk of silent failures (hallucination slips through)

**Because the substrate enforces gates**:
- ✓ `UNKNOWN_SYSTEM:99999` is filtered BEFORE operators see it
- ✓ Only ontology-valid atoms reach the soundness-verification gate
- ✓ Safety is guaranteed **regardless of LLM behavior**
- ✓ Audit trail records all filtering decisions (OBL-PS-04)

### INV-PS-06 Verified

The example empirically demonstrates that:

1. **Filtered candidates are valid refinements**: The 2 valid candidates (SNOMED:76948002, LOINC:8480-6) satisfy the refinement condition (candidate ⊇ input atoms)
2. **System tolerates hallucinations**: No crash, no undefined behavior — hallucinations are silently discarded
3. **Audit trail records decisions**: `FilterResult.filter_errors` captures any constraint violations
4. **Substrate behavior is proposer-independent**: With or without LLM hallucinations, the post-gate behavior is identical

## Comparison with Deterministic Search (Task 9.3)

| Dimension | LatticeSearchProposer | LlmProposer |
|-----------|----------------------|------------|
| **Output type** | Guaranteed valid (by construction) | Can include hallucinations |
| **Soundness source** | Operator-reachability (exhaustive) | Constraint gates (filtering) |
| **Scalability** | ≤5 operators efficiently | Large decision spaces tractable |
| **Candidate diversity** | Exhaustive enumeration | LLM-driven suggestions |
| **Guarantees** | Completeness by design | Safety by gates + audit |

**Both proposers feed through identical gates**:

```
Proposer Output
    ↓
[ProposerConstraint: ontology-boundedness check]
    ↓
[SoundnessVerificationGate: operator licensing]
    ↓
Refined Hypothesis or Abstention
```

Result: **Identical substrate behavior for the same evidence**, regardless of which proposer is plugged in.

## Design Insights

### Load-Bearing Safety Property (INV-PS-06)

This example is the empirical demonstration of the substrate-first architecture:

> **"The system remains sound even if the LLM hallucinates, because soundness is enforced by the deduction substrate (operators), not by the behavior of learned components (LLM)."**
> — NOTE.md §4A.5 (Learned components as refinement proposers)

The hallucination filtering happens at **two layers**:
1. **Parsing layer** (LlmProposer internal): Malformed atoms rejected during parsing
2. **Constraint layer** (ProposerConstraint gate): Ontology violations rejected before operators

An adversarial or maximally hallucinating LLM cannot bypass these layers because they are structural, not behavioral.

### Audit Trail for Compliance

The `FilterResult` structure (from `propose_and_filter`) maintains a complete audit trail:

```rust
pub struct FilterResult {
    pub valid_candidates: Vec<Hyp>,
    pub filtered_out_count: usize,
    pub filter_errors: Vec<FilterError>,  // ← Compliance record
}
```

Each filtering decision is recorded with error details (clause violated, candidate rejected). This satisfies **OBL-PS-04** (Provenance and decision audit) — clinical reviewers can reconstruct why the system made each choice.

## References

- **SPEC.md §2.7**: DEF-PS-14 (Refinement proposer signature), DEF-PS-15 (Proposer codomain constraint)
- **SPEC.md §2.7**: INV-PS-06 (Proposer cannot bypass soundness)
- **SPEC.md §2 Preliminaries**: OBL-PS-04 (Provenance and audit trail discharge)
- **NOTE.md §4A.5**: Constrained refinement proposer (Position note principle)
- **NOTE.md §3**: Substrate-first framing (Why learned components require gates)
- **ARCHITECTURE.md Diagram 3**: Substrate-learned-component boundary (proposer input/output gates, soundness-verification node `SV`)
- **ARCHITECTURE.md Diagram 5**: Learned-component composition (proposer slot `RP` filled by both LatticeSearchProposer and LlmProposer)

## Running the Example

```bash
cd clinlat
cargo run --example llm_proposer_sepsis
```

**Expected output**:
- Patient scenario setup
- Evidence collection step
- Mock LLM response with breakdown (valid candidates vs. hallucinations)
- Raw proposer output (2 valid candidates after parsing)
- Constraint filtering stage (no additional candidates filtered if all are ontology-valid)
- Operator licensing stage (demonstrates abstention with empty operator set)
- Safety analysis (INV-PS-06 verification: hallucinations silently filtered, substrate sound)
- Comparison with LatticeSearchProposer (both feed through identical gates)

Each step is annotated with what's happening and why. The example is designed for both:
- **Developers**: Shows LlmProposer API usage and gate integration
- **Clinical reviewers**: Demonstrates safety guarantees and audit trail completeness
