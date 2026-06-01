# INV-PS-06: Proposer Cannot Bypass Soundness

**Invariant:** Even if a refinement proposer is adversarial or hallucinating, the soundness of the active hypothesis depends only on the deduction operators (`Δ_PS`), not on the proposer's behavior.

**Formal statement (from SPEC.md §2.7):**

> For any proposer `π` (including adversarial ones) and any hypothesis `h` and evidence `e`:
> - If `ω ∈ OperatorSet.apply_set(h, e)` and `ω` is the active hypothesis, then `ω` is sound.
> - The soundness property is structural: it holds **regardless of the candidates proposed by `π`**.

**Status:** Informal argument (property-test discharge at task 8.6).

---

## Threat Model

An **adversarial proposer** is one that:
1. Returns invalid candidates (outside ontologies, not refinements of input, unreachable by operators).
2. Hallucinates hypotheses with atoms that don't exist.
3. Proposes candidates that would violate the substrate invariants if accepted without filtering.
4. Is not constrained by the substrate's design or semantics.

**Example:** An LLM-based proposer that:
- Suggests SNOMED codes it invented (e.g., "99999999" as a fictional concept).
- Proposes hypotheses with Unstructured atoms (free text).
- Returns candidates that have fewer atoms than the input (non-refining).
- Suggests operators that don't exist in the operator set.

---

## Proof Strategy

The proof is a **two-stage firewall**:

1. **Stage 1: Proposer Constraint (DEF-PS-15)**
   - `ProposerConstraint.validate()` filters proposer output before it reaches the soundness gate.
   - Rejects candidates outside ontology bounds (Clause 1).
   - Rejects candidates not reachable by one operator step (Clause 2).
   - **Effect:** Invalid candidates are eliminated before the soundness gate sees them.

2. **Stage 2: Soundness Gate (OperatorSet.apply_set())**
   - `OperatorSet.apply_set()` is the **only mechanism that licenses a candidate to become the active hypothesis** (DEF-PS-08, Diagram 3 node `SV`).
   - The gate applies each operator in `Δ_PS` to the input hypothesis `h`.
   - Only candidates that **exactly match** the output of at least one operator are licensed.
   - **Effect:** The active hypothesis is produced by a sound operator, hence sound.

---

## The Waterfall

```
Proposer π(h, e)
     ↓ [outputs candidates]
Proposer Constraint (DEF-PS-15)
     ↓ [filters invalid]
Valid candidates
     ↓ [fed to soundness gate]
OperatorSet.apply_set(h, e)
     ↓ [applies each operator in Δ_PS]
Licensed hypotheses (↥_PS)
     ↓ [one becomes active via external policy]
Active hypothesis ω_active
```

**Key observations:**

1. **Constraint filter is defensive:** Even if the proposer is adversarial, the constraint filter rejects candidates that violate DEF-PS-15. Unstructured atoms, fabricated codes, non-refining hypotheses — all caught here.

2. **Soundness gate is deterministic:** The gate does not trust the proposer. It **recomputes the candidates itself** by applying each operator independently. The proposer's output is only a **suggestion**. The gate decides.

3. **Independence:** The proposer and operators are decoupled. The proposer's internal logic, ML model weights, hyperparameters — none of this affects the gate's output. Only the operators matter.

4. **Closed-loop:** The active hypothesis is always a member of `OperatorSet.apply_set(h, e)`. By DEF-PS-08 (operator soundness), all members of this set are sound. Therefore, the active hypothesis is sound.

---

## Worked Example: Adversarial LLM Proposer

### Setup

- **Operators:** `{SofaRespOperator, KdigoAkiOperator}` (sound by construction, task 8.1–8.5).
- **Input hypothesis:** `h₀ = Unknown` (no information).
- **Evidence:** `e = {pao2_fio2 = 150, creatinine = 2.5, urine_output = 200}`.
- **Proposer:** An LLM that hallucinates.

### LLM Output (Unconstrained)

```
"Based on the evidence, I suggest these hypotheses:
  - {Unstructured: 'the patient has severe hypoxemia'}
  - {SNOMED: '99999999'}
  - {SNOMED: '67822003'}
  - {SNOMED: '3723001', KDIGO: 'Stage 3'}
"
```

### Stage 1: Proposer Constraint Filtering

Each candidate is validated against DEF-PS-15:

| Candidate | Ontology-bounded? | Operator-reachable? | Filtered? | Reason |
|-----------|------------------|-------------------|-----------|--------|
| `{Unstructured: 'hypoxemia'}` | ✗ (free text) | ✓ | **REJECTED** | OBL-PS-01: Unstructured prohibited |
| `{SNOMED: '99999999'}` | ✗ (code unknown) | ✓ (from Unknown) | **REJECTED** | Code not in SNOMED ontology |
| `{SNOMED: '67822003'}` | ✓ | ✓ (from Unknown) | ✓ | Valid refinement |
| `{SNOMED: '3723001', KDIGO: 'Stage 3'}` | ✓ | ? | Depends on input | See below |

**Result:** 1 invalid, 1 ambiguous, 2 candidates pass constraint.

### Stage 2: Soundness Gate

The gate applies each operator to `h₀ = Unknown`:

```rust
let sofas = SofaRespOperator.apply(h₀, e);
// Returns: {Hyp with SOFA-3 respiratory band}
let kdigas = KdigoAkiOperator.apply(h₀, e);
// Returns: {Hyp with KDIGO AKI stage}
let licensed = sofas ∪ kdigas;
```

The soundness gate returns the union of all operator results. **Only these are licensed hypotheses.** The LLM's hallucinations don't make it here—they were filtered at stage 1.

### Output: Sound

```
Soundness proof for ω_active:
  ω_active ∈ licensed
  licensed ⊆ apply_set(h₀, e)
  apply_set(h₀, e) = {SofaRespOperator.apply(...) ∪ KdigoAkiOperator.apply(...)}
  All operators are sound (DEF-PS-08)
  ⇒ ω_active is sound ✓
```

**The LLM's hallucinations never reach the active hypothesis.**

---

## Why This Matters

This invariant is the **answer to the central safety question** of substrate-first design:

> **Q:** If we feed our clinical AI system an LLM or other learned model, won't its failures/hallucinations compromise patient safety?
>
> **A:** No. The soundness of the diagnosis is guaranteed by the substrate's deduction operators, not the proposer. Learned models propose; the substrate's logic decides.

The substrate **absorbs and neutralizes** the uncertainty of learned components. It's not that we trust the LLM—we don't. It's that we've architected the system so that trust is **not required for soundness**.

---

## Residual Assumptions

This proof assumes:

1. **Operator soundness (DEF-PS-08):** Each operator in `Δ_PS` is sound by construction. This is discharged per-operator (e.g., task 8.1 for SOFA-3, task 8.5 for additional operators).

2. **Proposer constraint correctness (DEF-PS-15, tasks 8.1–8.3):** The constraint validation and `propose_and_filter` adapter are correctly implemented. Discharged by property-test tier (task 8.6).

3. **Operator set integrity:** `OperatorSet.apply_set()` is the **only** mechanism that can license hypotheses to become active. Enforced structurally by the module boundary (Diagram 3, `SV` node).

4. **No side channels:** No learned component can modify the active hypothesis state except through `OperatorSet.apply_set()`. Enforced by access control and module isolation.

---

## References

- **Formal definition:** SPEC.md §2.7 (INV-PS-06)
- **Proposer semantics:** SPEC.md §2.7 (DEF-PS-14 `RefinementProposer`)
- **Constraint:** SPEC.md §2.7 (DEF-PS-15 proposer codomain)
- **Soundness gate:** SPEC.md §2.4 (DEF-PS-08 operator soundness)
- **Implementation:**
  - clinlat/src/proposer.rs (RefinementProposer trait, ProposerConstraint, propose_and_filter)
  - clinlat/src/operator_set.rs (OperatorSet, apply_set)
- **Position statement:** NOTE.md §4A.5 (constrained refinement proposer), §5 (substrate-first framing)

---

## Next Steps

**Property-test discharge (task 8.6):**
- Generate adversarial proposers with ≥10 hallucination profiles (out-of-ontology atoms, non-refining, Unstructured, non-existent codes).
- Verify: every path through `propose_and_filter` → `OperatorSet.apply_set()` yields sound results.
- Property: `∀ adversarial π, ∀ h, ∀ e: soundness(active_hyp) = soundness(operators)`.
