# INV-PS-06: Proposer Cannot Bypass Soundness

**Invariant:** Even if a refinement proposer is adversarial or hallucinating, the soundness of the active hypothesis depends only on the deduction operators (`Δ_PS`), not on the proposer's behavior.

**Formal statement (verbatim, SPEC.md §2.7, INV-PS-06):**

> The proposer cannot produce a refined hypothesis that becomes the active hypothesis without passing through a sound deduction operator (DEF-PS-08). Even if the proposer is adversarial, the soundness of the active hypothesis depends only on `Δ_PS`, not on `π`.
>
> This is the load-bearing safety property of the patient substrate: **learned-component behavior cannot violate substrate soundness**.

The companion obligation **OBL-PS-05** (SPEC.md §2.7) states the enforcement mechanism that this argument relies on:

> No code path may insert a value into `Hyp^P` (as the active patient hypothesis) without that value being the `Refined(_)` branch of some sound operator's output. Enforcement is structural: the active-hypothesis type and the proposer-output type are distinct, and only operator results inhabit the former.

**Status:** Informal argument (property-test discharge at task 8.6). This document argues the invariant against the substrate's *intended* end-to-end flow. Two pieces of that flow are not yet built or are stubbed at the time of writing; they are disclosed explicitly under [Disclosed gaps](#disclosed-gaps-not-yet-built-or-stubbed) so the argument's scope is not overstated.

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
- Returns candidates a clinically sound operator would never derive from the evidence.

---

## Proof Strategy

The safety property rests on **two independent lines of defence**. The second is the load-bearing one; the first is defence-in-depth.

1. **Stage 1 — Proposer constraint filter (DEF-PS-15), defence-in-depth.**
   - `ProposerConstraint::validate()` filters proposer output before it is used.
   - Rejects candidates that are not ontology-bounded (Clause 1).
   - Rejects candidates that do not refine the input hypothesis (Clause 2, conservative form — see note below).
   - **Effect:** Most malformed candidates are discarded early, improving the audit trail. This stage is *advisory hygiene*, not the soundness guarantee — even if it let a bad candidate through, Stage 2 would still hold.

2. **Stage 2 — Structural operator-origin guarantee (OBL-PS-05), load-bearing.**
   - The committed (active) hypothesis is never *selected from* the proposer's candidate set. It is the `result` field of `OperatorSet::apply_set(h, e)`, which is constructed **only** from the `Outcome::Refined(_)` branch of registered operators (`operator_set.rs:113–127`). `apply_set` does not take the candidate set as an argument and never reads it.
   - Because the proposer-output type (`CandidateSet`) and the active-hypothesis value (`SetOutcome.result: Hyp`) are produced by disjoint code paths, an adversarial proposer **cannot place a value into the active-hypothesis slot at all** — not by matching, not by injection. The proposer can only influence *which refinements are explored*, never *what is committed*.
   - **Effect:** The active hypothesis is, by construction, the output of a sound operator (DEF-PS-08), hence sound — independent of `π`.

> **Note on "operator-reachable" (Clause 2).** SPEC.md DEF-PS-15.2 defines reachability as "there exists `δ ∈ Δ_PS` that could plausibly produce this refinement." The current implementation discharges this conservatively as `candidate.atoms() ⊇ input.atoms()` (candidate refines input). This is sound-but-incomplete: it admits some candidates no operator would actually produce. That gap is harmless for INV-PS-06 because Stage 2 — not Stage 1 — is what guarantees soundness.

---

## The Two Paths

The key structural fact is that the proposer's output and the active hypothesis travel on **separate, non-converging paths**:

```
                    Proposer π(h, e)
                          │
                          ▼
                    CandidateSet                 ← proposer-output type
                          │
                          ▼
         ProposerConstraint::validate()          ← Stage 1 (advisory filter)
                          │
                          ▼
              valid candidates (search hints)
                          ┊
   ( used only to decide WHICH refinements to explore — never committed )
                          ┊
─────────────────────────────────────────────────────────────────────────
                          │
   OperatorSet::apply_set(h, e)                  ← reads h and e ONLY,
        for δ in Δ_PS:                             never the candidate set
            match δ.apply(current_h, e):
                Refined(h') ⟹ current_h = h'     ← active value built here
                Abstain(r)  ⟹ record r
                          │
                          ▼
        SetOutcome { result, abstentions }        ← active-hypothesis type
                          │
                          ▼
                  Active hypothesis = result
```

**Key observations:**

1. **The paths never merge.** `apply_set` (`operator_set.rs:109`) has signature `(&Hyp, &Evidence) -> SetOutcome`. It has no parameter for the candidate set and cannot read it. The proposer therefore has *no channel* through which to place a value into `SetOutcome.result`.

2. **The committed value is operator-built, not candidate-selected.** `result` starts as `h.clone()` and is reassigned only inside the `Outcome::Refined(h')` arm of a registered operator (`operator_set.rs:115–121`). No proposer candidate is ever copied into it.

3. **Independence from the proposer's internals.** The proposer's logic, model weights, and hyperparameters affect only *search ordering* (Stage 1's candidates are hints). They cannot affect `apply_set`'s output, which is a pure function of `h`, `e`, and `Δ_PS`.

4. **Soundness follows directly.** `result` is either the untouched input `h` (all operators abstained) or the `Refined(_)` output of a sound operator. By DEF-PS-08 every operator's `Refined` output refines the input soundly; therefore `result` is sound — for any `π`, adversarial or not.

---

## Worked Example: Adversarial LLM Proposer

### Setup

- **Operators:** `{SofaRespOperator, KdigoAkiOperator}` (sound by construction; shipped and tested in M1).
- **Input hypothesis:** `h₀ = Unknown` (no information).
- **Evidence:** `e = {pao2_fio2 = 150, creatinine = 2.5, urine_output = 200}`.
- **Proposer:** An LLM that hallucinates.

### LLM Output (Unconstrained)

```
"Based on the evidence, I suggest these hypotheses:
  - {Unstructured: 'the patient has severe hypoxemia'}
  - {SNOMED: '99999999'}          (a code the LLM invented)
  - {SNOMED: '67822003'}          (Hypoxemia — a real code)
  - {SNOMED: '3723001', SNOMED: '14669001'}  (ARDS + acute renal failure)
"
```

### Stage 1: Proposer Constraint Filtering (advisory)

Each candidate is validated against DEF-PS-15 by `ProposerConstraint::validate()`. Input is `h₀ = Unknown` (empty atom set), so every candidate trivially satisfies the conservative operator-reachable check (`atoms ⊇ ∅`). The current implementation enforces the ontology-bounded clause as: reject `Unstructured`, reject empty codes (`proposer.rs:143–162`).

| Candidate | Ontology clause (current impl) | Filtered today? | Note |
|-----------|-------------------------------|-----------------|------|
| `{Unstructured: 'hypoxemia'}` | ✗ rejected — `Unstructured` (OBL-PS-01) | **REJECTED** | Enforced now |
| `{SNOMED: '99999999'}` | ✓ passes — code is non-empty | **PASSES** | ⚠️ *Code existence not yet checked* — see below |
| `{SNOMED: '67822003'}` | ✓ passes | passes | Valid refinement |
| `{SNOMED: '3723001', '14669001'}` | ✓ passes | passes | Valid refinement |

> ⚠️ **Honest disclosure:** the fabricated code `99999999` is **not** rejected by the current Stage-1 filter — code-existence validation against the `OntologyAdapter` is an open TODO (`proposer.rs:154`). At the property-test tier this will be closed (task 8.6). **Crucially, INV-PS-06 holds anyway**, because Stage 1 is not what guarantees soundness — Stage 2 does. The next subsection shows why this surviving hallucination is still harmless.

### Stage 2: The candidates never reach the active hypothesis

This is the load-bearing step. The substrate does **not** select the active hypothesis from the (partially filtered) candidate set. It computes it independently:

```rust
// Candidates from Stage 1 are search hints only — NOT passed here.
let outcome = operator_set.apply_set(&h0, &e);
//   apply_set signature: (&Hyp, &Evidence) -> SetOutcome
//   It reads h0 and e. It never sees `candidates`.
//
// Inside apply_set (operator_set.rs:113-127), for each registered operator:
//   match op.apply(&current_h, &e) {
//       Outcome::Refined(h_prime) => current_h = h_prime,  // sound by DEF-PS-08
//       Outcome::Abstain(reason)  => record(reason),       // current_h unchanged
//   }
//
// active_hypothesis = outcome.result;  // built only from Refined(_) outputs
```

The surviving hallucination `{SNOMED: '99999999'}` is in the candidate set, but the candidate set is never an input to `apply_set`. Whatever `SofaRespOperator` and `KdigoAkiOperator` derive from `h₀` and `e` is what gets committed; the invented code has no path to `outcome.result`.

### Output: Sound

```
Soundness of active_hypothesis = outcome.result:
  outcome.result is either:
    (a) h₀ unchanged           — if every operator abstained, or
    (b) Refined(_) output of a registered operator δ ∈ Δ_PS.
  Every δ satisfies DEF-PS-08 (Refined(h') ⟹ h' ⊑_PS h, soundly).
  The candidate set never enters apply_set (no parameter for it).
  ⇒ active_hypothesis is sound, for any π — including one whose
    fabricated code survived Stage 1. ✓
```

**The LLM's hallucination survives Stage 1 but is structurally incapable of becoming the active hypothesis.** That is a *stronger* statement than "it was filtered out": even an unfiltered hallucination cannot compromise soundness.

---

## Why This Matters

This invariant is the **answer to the central safety question** of substrate-first design:

> **Q:** If we feed our clinical AI system an LLM or other learned model, won't its failures/hallucinations compromise patient safety?
>
> **A:** No. The soundness of the diagnosis is guaranteed by the substrate's deduction operators, not the proposer. Learned models propose; the substrate's logic decides.

The substrate **absorbs and neutralizes** the uncertainty of learned components. It's not that we trust the LLM—we don't. It's that we've architected the system so that trust is **not required for soundness**.

---

## Residual Assumptions

This argument is at the **informal-argument tier**. It assumes:

1. **Operator soundness (DEF-PS-08, OBL-PS-03):** Each operator in `Δ_PS` is sound by construction. Discharged per-operator (SOFA-3 at property-test tier; KDIGO AKI / Wells-PE / CURB-65 with soundness arguments, M1).

2. **Operator-origin enforcement (OBL-PS-05):** `OperatorSet::apply_set` is the only constructor of the active-hypothesis value, and it never reads the candidate set. This is currently enforced by the `apply_set` signature `(&Hyp, &Evidence) -> SetOutcome` (no candidate parameter exists). SPEC OBL-PS-05 calls for this to be enforced by *type* distinctness (proposer-output type vs. active-hypothesis type); today the separation is by function signature, not by a newtype barrier. Hardening to a distinct active-hypothesis newtype is future work.

## Disclosed gaps (not yet built or stubbed)

The argument above describes the substrate's intended end-to-end flow. Two parts of that flow are incomplete at the time of writing. Neither weakens INV-PS-06, because the invariant rests on Stage 2, but they are disclosed so the scope is honest:

1. **Ontology code-existence check is a stub.** `ProposerConstraint::validate` currently rejects only `Unstructured` atoms and empty codes; it does not yet verify that a coded atom resolves in its ontology (`proposer.rs:154`, TODO). Fabricated-but-non-empty codes survive Stage 1 today. Closed at task 8.6 (property tier).

2. **`propose_verify` (the candidate-routing soundness-verification adapter) is not yet built.** Task 8.5 (`cc:todo`) will route constraint-passing candidates through `apply_set` and emit `AbstainReason::NoOperatorLicenses` when none is licensed. **Open design question for 8.5:** since `apply_set` returns a single threaded `result` (not a per-operator set of outputs), the mechanism by which a *specific* candidate is judged "licensed by ≥1 operator" must be defined — e.g., re-running each operator individually from `h` and testing candidate equality against its `Refined` output, rather than against the threaded composition. This document deliberately does **not** assume that mechanism; the INV-PS-06 argument here depends only on the operator-origin property of `apply_set`, which holds independently of how 8.5 defines candidate licensing.

---

## References

- **Formal definition:** SPEC.md §2.7 (INV-PS-06)
- **Enforcement obligation:** SPEC.md §2.7 (OBL-PS-05 proposer-operator separation)
- **Proposer semantics:** SPEC.md §2.7 (DEF-PS-14 `RefinementProposer`)
- **Constraint:** SPEC.md §2.7 (DEF-PS-15 proposer codomain)
- **Operator soundness:** SPEC.md §2.4 (DEF-PS-08, INV-PS-03 monotonicity, DEF-PS-09 operator set, OBL-PS-03 set soundness)
- **Implementation:**
  - `clinlat/src/proposer.rs` (`RefinementProposer` trait, `ProposerConstraint`, `propose_and_filter`)
  - `clinlat/src/operator_set.rs` (`OperatorSet::apply_set` — the operator-origin boundary)
  - `clinlat/src/outcome.rs` (`Outcome<Hyp, AbstainReason>` — single `Refined`/`Abstain`, not a set)
- **Position statement:** NOTE.md §4A.5 (constrained refinement proposer), §5 (substrate-first framing)

---

## Next Steps

**Property-test discharge (task 8.6):**
- Generate adversarial proposers with ≥10 hallucination profiles (out-of-ontology atoms, non-refining candidates, `Unstructured` atoms, fabricated-but-non-empty codes).
- Assert the operator-origin property directly: for any such `π`, the value committed by `apply_set(h, e)` is identical whether the proposer is sound or adversarial — i.e. `apply_set`'s output is invariant under proposer substitution, because it never reads the candidate set.
- Close the disclosed Stage-1 gap: extend `ProposerConstraint::validate` to reject coded atoms that do not resolve via the `OntologyAdapter` (currently `proposer.rs:154` TODO), and add ≥10 cases over out-of-bounds candidate generators.
- Property: `∀ adversarial π, ∀ h, ∀ e: apply_set(h, e).result` is sound and equal to the sound-proposer result.
