# Worked Example: Substrate-First Claim on Sepsis-3 (Proposer Swap)

**Task**: Phase 11.3 of M2 (OBL-PS-05 discharge + substrate-invariance demonstration)
**Date**: 2026-07-03
**Reference**: SPEC.md §2.7 (DEF-PS-14, DEF-PS-15, INV-PS-06, OBL-PS-05), NOTE.md §3, §5, §4A.5, ARCHITECTURE.md Diagram 3 and 5

## Overview

This worked example is the human-readable companion to:

- The machine-checked property tests in `clinlat/src/proposer.rs` ("Substrate-Invariance Tests", task 11.2).
- The obligation discharge argument in `clinlat/docs/obligations/obl-ps-05-proposer-constraint.md` (task 11.1).

It closes the M2 Definition of Done by demonstrating, on the same sepsis-3 scenario used throughout Phases 9 and 10, the **substrate-first claim**: the substrate's *licensed* outcome is identical across proposer architectures, even when the proposers' *raw* candidate sets diverge.

> **"The substrate is what determines the accepted refinement, not the proposer that suggested it."**
> — NOTE.md §3 (Problem framing), §5 (Substrate-first commitment)

The example is self-contained and runnable:

```bash
cd clinlat && cargo run --example substrate_invariance_sepsis
```

## Clinical Scenario

Same patient as tasks 9.3 and 10.4:

**Patient A** (64-year-old):
- **Presentation**: Community-acquired pneumonia (CAP) with fever, cough, SOB × 2 days
- **Current status**: Intubated on mechanical ventilation in ICU

**Laboratory findings**:
- **Arterial blood gas**: PaO₂ = 150 mmHg, FiO₂ = 0.60 (on ventilator) → PaO₂/FiO₂ ratio = 250 mmHg → SOFA respiratory score 2

Only the SOFA respiratory operator is registered for this example (a single operator keeps the divergence-then-convergence story legible; the same argument holds with additional operators, as tasks 9.3 and 10.4 demonstrate separately).

## The Substrate-Invariance Argument

### Step 1: Identical evidence, two proposers

Both `LatticeSearchProposer` (task 9.1) and `LlmProposer` (task 10.2, mock mode) see the **same** `Evidence` and the **same** input hypothesis (`Hyp::unknown()`).

### Step 2: Ground truth, derived not hardcoded

The example calls `SofaRespOperator::apply()` directly once, up front, to obtain the atom this evidence actually licenses. The LLM's "correct" mock response line is built from that atom programmatically:

```rust
let ground_truth_atom = match ground_truth_op.apply(&Hyp::unknown(), &evidence) {
    Outcome::Refined(refined) => refined.atoms().first().cloned().unwrap(),
    other => panic!("expected Outcome::Refined, got {:?}", other),
};
let correct_line = format!(
    "{}:{}@{}",
    ground_truth_atom.system, ground_truth_atom.code, ground_truth_atom.version
);
```

This keeps the example correct even if `SofaRespOperator`'s internal atom encoding changes later — it is not a fragile hardcoded string.

### Step 3: Raw candidates diverge

**`LatticeSearchProposer`** exhaustively searches the (single-operator) lattice and returns exactly one raw candidate: the operator-reachable one.

**`LlmProposer`** (mock) is configured with a 4-line response:

| Line | Content | Outcome |
|------|---------|---------|
| 1 | `correct_line` (matches ground truth) | ✓ Parses, later licensed |
| 2 | `correct_line` with the trailing digit changed (`resp-2` → `resp-4`) | ✓ Parses (ontology-valid), **not** licensed — wrong severity |
| 3 | `NOT_AN_ATOM_AT_ALL` | ✗ Fails `parse_atom` (no `:`) — dropped silently at **parse** stage |
| 4 | `FHIR:99999@0.2.0` | ✗ Unknown ontology system — dropped silently at **parse** stage |

**Result**: `LatticeSearchProposer` produces **1** raw candidate; `LlmProposer` produces **2** raw candidates (lines 3–4 never survive parsing, so they never even reach the candidate set). The raw candidate sets already disagree in size and content — this is the "divergent candidate sets" half of the DoD.

### Step 4: Both pass through the identical soundness gate

Each proposer's output is routed through `propose_verify` (INV-PS-06 enforcement). Per this repo's test-independence discipline (see `CLAUDE.md`, "Structuring tests for safety verification"), each `propose_verify` call is given its **own independently-constructed** `OperatorSet` instance — three separate `OperatorSet::new().register(...)` calls in total (one for the lattice search itself, one for the lattice-search licensing check, one for the LLM licensing check) — so the demonstration validates the invariant across independently-built objects, not merely that one shared instance is self-consistent.

The wrong-severity hallucination (line 2) is **not** rejected by the same mechanism as lines 3–4: it survives ontology parsing (it is a well-formed SNOMED atom) and even survives `ProposerConstraint::validate()` (Stage 1), because Stage 1 only checks ontology-boundedness and one-step-refinement, not clinical correctness. It is rejected at **Stage 2 (licensing)**, inside `propose_verify`, because its atom is not a member of `OperatorSet::apply_set()`'s result for this evidence. This distinguishes the two silent-drop mechanisms the codebase asks callers to name precisely: parse-stage drops (lines 3–4, no audit trail) vs. licensing-stage drops (line 2, recorded in `VerifyResult.licensing_verdicts`).

### Step 5: Licensed outcome converges — the substrate-first claim, mechanically checked

```rust
let identical = lattice_verify.licensed_candidates == llm_verify.licensed_candidates;
assert!(identical, "substrate-invariance violated: licensed_candidates differ across proposer architectures");
```

Both `licensed_candidates` sets contain exactly one entry: `Hyp` wrapping the ground-truth SOFA-2 atom. The assertion passes — despite the raw candidate sets differing in size (1 vs. 2) and content (the LLM path included a plausible-but-wrong hallucination the lattice-search path never generated), the **post-gate** outcome is byte-for-byte identical.

## Why This Matters

This is the empirical demonstration referenced in the M2 Definition of Done: **substrate behavior is independent of proposer architecture**. Safety and correctness are enforced by the deduction substrate (operators + `propose_verify`'s soundness gate) — not by which proposer generated the candidates, and not by how "smart" or "dumb" that proposer happens to be on a given call.

This complements task 11.2's property tests (which check the same claim across ≥10 paired cases, generalized over abstention, mixed hallucinations, and operator-licensing edge cases) with a single, narratively-explained instance a reviewer can read top-to-bottom without running a test harness.

## References

- **SPEC.md §2.7**: DEF-PS-14 (Refinement proposer signature), DEF-PS-15 (Proposer codomain constraint), INV-PS-06 (Proposer cannot bypass soundness), OBL-PS-05 (Proposer-operator separation).
- **NOTE.md §3**: Problem framing (why proposer trust cannot be the safety mechanism).
- **NOTE.md §5**: Substrate-first commitment.
- **NOTE.md §4A.5**: Learned components as refinement proposers.
- **ARCHITECTURE.md Diagram 3**: Substrate-learned-component boundary (soundness-verification node `SV`).
- **ARCHITECTURE.md Diagram 5**: Learned-component composition (proposer slot `RP` filled by both `LatticeSearchProposer` and `LlmProposer`).
- **`clinlat/docs/obligations/obl-ps-05-proposer-constraint.md`** (task 11.1): Formal discharge argument this example instantiates.
- **`clinlat/src/proposer.rs`**, "Substrate-Invariance Tests" (task 11.2): Machine-checked property tests over ≥10 paired cases.

## Running the Example

```bash
cd clinlat
cargo run --example substrate_invariance_sepsis
```

**Expected output**:
- Patient scenario setup and shared evidence
- Ground-truth operator output (derived, not hardcoded)
- `LatticeSearchProposer` raw candidates (1)
- `LlmProposer` raw candidates (2, after two hallucinated lines are silently dropped at parsing)
- Explicit note that raw candidate sets already diverge
- Both proposers' `propose_verify` results (licensed / unlicensed counts)
- Final assertion: `licensed_candidates` identical across proposers (substrate-first claim holds)
