# Soundness Argument: SOFA-3 Respiratory Component (PaO₂/FiO₂)

**Operator:** `SofaRespOperator` (SOFA-3 respiratory scoring)
**Scope:** Informal-argument tier per OBL-PS-03
**Date:** 2026-05-25
**References:**

- Vincent JL, et al. The SOFA (Sepsis-related Organ Failure Assessment) score to describe organ dysfunction/failure. Intensive Care Med. 1996;22(7):707–710.
- Singer M, et al. The Third International Consensus Definitions for Sepsis and Septic Shock (Sepsis-3). JAMA. 2016;315(8):801–810.

---

## Overview

This operator maps arterial oxygen partial pressure (PaO₂) and fraction of inspired oxygen (FiO₂) to a SOFA respiratory score (0–4), indicating severity of respiratory dysfunction in sepsis. The three soundness clauses from DEF-PS-08 are addressed below.

## Soundness Clause 1: Refinement Monotonicity

**Statement:** If hypothesis h₁ refines h₂ (h₁ ⊑ h₂), then the operator output refines the original output under h₂.

**Argument:**

In v0.1.0, the operator takes a unit `Evidence` type and does not use the input hypothesis h. The operator's output is determined entirely by the evidence (PaO₂, FiO₂, mechanical ventilation status) and the SOFA threshold table, which is fixed.

For any hypotheses h₁, h₂, and evidence e:

- If the operator produces `Refined(score)` on (h₁, e), it produces the same `Refined(score)` on (h₂, e).
- If the operator produces `Abstain(reason)` on (h₁, e), it produces the same `Abstain(reason)` on (h₂, e).

Therefore, the operator is **hypothesis-independent** in v0.1.0 and trivially satisfies monotonicity. In v0.2+, when the operator uses the input hypothesis to constrain candidate scores, monotonicity will be enforced by the lattice structure: if h₁ ⊑ h₂, then the candidate scores of h₁ form a subset of those of h₂, so refinement is monotonic.

---

## Soundness Clause 2: No Spurious Refinement

**Statement:** The operator never produces a refined hypothesis that is not justified by the evidence.

**Argument:**

The SOFA-3 respiratory component uses a deterministic threshold table:

- PaO₂/FiO₂ ≥ 400 mmHg → score 0 (no respiratory dysfunction)
- 300–399 mmHg → score 1
- 200–299 mmHg → score 2
- 100–199 mmHg → score 3 (requires mechanical ventilation per Sepsis-3)
- < 100 mmHg → score 4 (requires mechanical ventilation)

Each score is the direct, deterministic consequence of measured PaO₂ and FiO₂ values. The operator computes the ratio and applies the thresholds without interpolation, rounding, or additional reasoning.

**Special case: Mechanical ventilation precondition.**
Sepsis-3 defines scores 3 and 4 *only* for intubated patients. The operator enforces this by returning `Abstain(OperatorPreconditionUnmet)` if the computed score is ≥3 but the patient is not on mechanical ventilation. This prevents spurious refinement to a score that is invalid in the clinical context.

Thus, the operator never produces a score that is not justified by the evidence and clinical context (ventilation requirement).

---

## Soundness Clause 3: Abstention Purity

**Statement:** Abstention is a structural decision, not an implementation error or undefined behavior.

**Argument:**

The operator abstains in two documented cases:

1. **Insufficient evidence:** FiO₂ ≤ 0 (cannot compute ratio).
   - This is a well-defined condition: missing or invalid FiO₂ makes the PaO₂/FiO₂ ratio undefined.
   - Abstention is the correct response; no refinement is attempted.

2. **Precondition unmet:** Computed score ≥ 3, but patient not on mechanical ventilation.
   - This is a well-defined condition derived from Sepsis-3 clinical definitions.
   - The operator does not attempt to force a score; it abstains and signals the conflict to the clinician.

Both cases are explicit, documented, and deterministic. There is no silent error handling, no graceful degradation, and no undefined behavior. Abstention is the intended output, not a fallback.

---

## Conclusion

The SOFA-3 respiratory operator satisfies all three soundness clauses:

1. **Monotonicity:** Hypothesis-independent in v0.1.0; will be enforced by lattice structure in v0.2+.
2. **No spurious refinement:** Deterministic thresholds + mechanical ventilation precondition prevent unjustified scores.
3. **Abstention purity:** Both abstention cases (FiO₂ invalid, ventilation precondition) are structural and documented.

The operator is **sound** under the informal-argument tier. A formal proof would require formalization of the Sepsis-3 clinical definitions and mechanized verification in a proof assistant; that is deferred to v0.2.0+.

---

## Future Work (v0.2.0+)

- Integrate the input hypothesis: candidate scores will be constrained by the hypothesis lattice.
- Real `Evidence` type: carry multiple measurements, timestamps, and provenance.
- Formal soundness proof: mechanized in Coq, Isabelle, or Lean.
- Extended SOFA scoring: integrate other organ components (cardiovascular, renal, etc.).
