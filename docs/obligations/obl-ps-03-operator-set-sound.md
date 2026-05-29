# OBL-PS-03: Operator Set Soundness

**Obligation:** Prove that operator composition via `OperatorSet::apply_set` preserves the refinement order.

**Formalizes:** SPEC.md §6 / OBL-PS-03, DEF-PS-09 (Operator Set Δ_PS)

**Status:** Discharged via inductive proof and 6+ property tests, implemented in clinlat v0.2.0

---

## Obligation Statement

Let Δ_PS = {δ₁, δ₂, …, δₖ} be a finite operator set (DEF-PS-09), where each δᵢ satisfies
the refinement monotonicity property:

**INV-PS-03:** For all hypotheses h and evidence e:
- If δᵢ(h, e) = ⊢ h', then h' ⊑ h (i.e., h' refines h or h' = h).

**OBL-PS-03:** The composition apply_set(h, e) := δₖ(…δ₂(δ₁(h, e))…) preserves refinement:

```
For all hypotheses h, evidence e, and operator sets Δ_PS:
    apply_set(h, e) ⊑ h
```

In other words: chaining sound operators always produces refinement or identity.

---

## Proof Strategy: Lifting Lemma

**Lifting Lemma (Induction on Operator Chain Length):**

**Base case** (k = 0, empty set):
- apply_set(h, e) = h (identity).
- h ⊑ h ✓

**Base case** (k = 1, single operator):
- apply_set(h, e) = δ₁(h, e).
- By INV-PS-03 applied to δ₁: δ₁(h, e) ⊑ h ✓

**Inductive case** (k → k+1):
- Assume apply_set_k(h, e) ⊑ h for all hypotheses h and operator chains of length ≤ k.
- Let Δ_PS = {δ₁, …, δₖ₊₁}.
- Define h' = apply_set_k(h, e) (result of first k operators).
- By inductive hypothesis: h' ⊑ h.
- Apply δₖ₊₁ to h':
  - If δₖ₊₁(h', e) = ⊢ h'', then by INV-PS-03: h'' ⊑ h'.
  - If δₖ₊₁(h', e) = Abstain, then h'' = h' (no change, current_h unchanged in apply_set).
- Either way: h'' ⊑ h' ⊑ h ✓

**Conclusion:** apply_set(h, e) ⊑ h for all chains. ∎

---

## Implementation: Propagate-Forward Semantics

The apply_set algorithm in clinlat/src/operator_set.rs implements the proof:

```rust
pub fn apply_set(&self, h: &Hyp, e: &Evidence) -> SetOutcome {
    let mut current_h = h.clone();
    let mut abstentions = Vec::new();

    for (op, (op_name, _)) in self.operators.iter().zip(self.metadata.iter()) {
        match op.apply(&current_h, e) {
            Outcome::Refined(h_prime) => {
                debug_assert!(h_prime <= current_h,
                    "INV-PS-03 violated by {}", op_name);
                current_h = h_prime;
            }
            Outcome::Abstain(reason) => {
                abstentions.push((op_name.clone(), reason));
                // current_h unchanged — propagate forward
            }
        }
    }

    SetOutcome {
        result: current_h,
        abstentions,
    }
}
```

**Key invariant maintained at each loop iteration:**
- current_h ⊑ h (by induction, following the lifting lemma)
- When op.apply(current_h, e) returns Refined(h'), we have h' ⊑ current_h (INV-PS-03 per op).
- Set current_h := h'; loop invariant maintained.
- When op.apply(current_h, e) returns Abstain, current_h unchanged; invariant maintained.

**Proof obligations discharged at each iteration:**
1. **INV-PS-03 enforcement:** The debug_assert fires if an operator violates monotonicity.
   In release builds, we trust individual operator implementations (pre-condition for OBL-PS-03).
2. **Propagate-forward semantics:** Abstentions are recorded but do not silence subsequent operators.
   Each operator sees the best-known (most-refined) hypothesis so far.

---

## Property Test Discharge

Six property tests validate OBL-PS-03 across fixture and composed operators:

### 1. test_apply_set_empty_set_identity
```
Input: empty operator set, h = Unknown
Expected: result = h, no abstentions
Status: ✓ PASS
Validates: Base case (k=0)
```

### 2. test_apply_set_noop_chain_identity
```
Input: two NoopOperator instances (identity refinements)
Expected: result = h, no abstentions
Status: ✓ PASS
Validates: Chain of identity operators preserves h
```

### 3. test_apply_set_monotonicity_const_refine
```
Input: ConstRefineOperator (adds fixed atom)
Expected: result ⊑ h (tighter than input)
Status: ✓ PASS
Validates: Non-trivial monotone composition
```

### 4. test_apply_set_abstention_propagates_forward
```
Input: [AlwaysAbstainOperator, ConstRefineOperator]
Expected: result ⊑ h, abstention recorded, second op still runs
Status: ✓ PASS
Validates: Abstention from op1 does not silence op2
```

### 5. test_apply_set_all_abstain_preserves_input
```
Input: three AlwaysAbstainOperator instances
Expected: result = h, three abstentions recorded
Status: ✓ PASS
Validates: All-abstain reduces to identity (h' = h)
```

### 6. test_apply_set_multiple_const_refine
```
Input: [ConstRefineOperator(atom1), ConstRefineOperator(atom2)]
Expected: result ⊑ h, both atoms added
Status: ✓ PASS
Validates: Successive refinements compose
```

---

## Worked Example: SofaRespOperator Composition

Suppose we compose:
- δ₁ = SofaRespOperator (SOFA-3 respiratory score → atom)
- δ₂ = KdigoAkiOperator (KDIGO AKI stage → atom)
- δ₃ = NoopOperator (identity)

**Input:** h = Unknown (no atoms)

**Evidence:** PaO₂/FiO₂ = 250 (score = 2), serum creatinine = 1.8 (KDIGO stage = 2)

**Step 1: δ₁(h, e)**
- Input: h = Unknown
- Applies SOFA-3 logic; score = 2 → adds SofaRespAtom(score: 2)
- Output: h' = {SofaRespAtom(score: 2)}
- Invariant: h' ⊑ h ✓ (specific ⊑ unknown)

**Step 2: δ₂(h', e)**
- Input: h' = {SofaRespAtom(score: 2)}
- Applies KDIGO logic; creatinine = 1.8 → adds KdigoAkiAtom(stage: 2)
- Output: h'' = {SofaRespAtom(score: 2), KdigoAkiAtom(stage: 2)}
- Invariant: h'' ⊑ h' ⊑ h ✓

**Step 3: δ₃(h'', e)**
- Input: h'' = {…, …}
- Noop: returns h'' unchanged
- Output: h''' = h''
- Invariant: h''' ⊑ h'' ⊑ h ✓

**Final result:** apply_set(Unknown, e) = {SofaRespAtom, KdigoAkiAtom} ⊑ Unknown ✓

---

## Limitations and Assumptions

1. **Debug assertions in debug builds only:**
   The debug_assert in apply_set checks INV-PS-03 but only fires in debug builds.
   In release builds, we rely on each operator's unit tests to enforce monotonicity.

2. **Abstention semantics:**
   The propagate-forward approach means one operator's abstention does not affect
   the input to the next operator. This is by design (SPEC.md §4C.3) and allows
   clinicians to interpret partial refinements when some operators abstain.

3. **No silent contradiction detection:**
   apply_set does not check whether atoms from different operators contradict
   each other (e.g., both "severe hypoxemia" and "normal oxygenation").
   This is a candidate for Phase 5+ (OQ-PS-07: contradiction detection across
   multi-domain compositions).

4. **No operator ordering guarantees:**
   The order of operators in Δ_PS affects the result when operators refine differently
   (e.g., competing hypotheses). Future work (SPEC.md §7) may formalize operator ordering
   (e.g., by confidence, by clinical priority).

---

## Verification Checklist

- [x] Lifting lemma proof (induction on chain length)
- [x] Base cases validated (empty set, single operator)
- [x] Inductive case validated (chained operators)
- [x] Implementation matches proof (propagate-forward semantics)
- [x] debug_assert enforces INV-PS-03 per operator
- [x] 6 property tests cover key scenarios
- [x] Edge cases: all-noop, all-abstain, mixed chains
- [x] Worked example: realistic operator composition
- [x] Limitations documented (debug builds, abstractions, ordering)

**Conclusion:** OBL-PS-03 is discharged. ✓

---

## References

- SPEC.md §2.3–2.4 (DEF-PS-09, DEF-PS-08)
- SPEC.md §6 (OBL-PS-03)
- SPEC.md §7 (OQ-PS-07, OQ-PS-08: future ordering and contradiction detection)
- clinlat v0.2.0 src/operator_set.rs (apply_set implementation)
- clinlat v0.2.0 src/operator.rs (Operator trait, INV-PS-03 enforcement per-operator)
