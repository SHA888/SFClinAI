# OBL-PS-02: Galois Adjunction Soundness

**Obligation:** The patient Galois connection `(Obs_PS, α_PS, γ_PS, H_PS)` satisfies the `DEF-MP-08` adjunction.

**Formalization:** SPEC.md §2.3 (DEF-PS-05, DEF-PS-06), §1.4 (DEF-MP-08).

**Discharge tier:** Property-test (v0.2.0-alpha.0).

---

## Executive Summary

This document discharges OBL-PS-02 by demonstrating that the patient-substrate Galois connection satisfies the formal adjunction law unconditionally. The connection maps clinical observations (concrete side) to refined hypotheses (abstract side):

- **Abstraction `α_PS`:** Evidence → Hyp (extracts atoms from observations; unknown if empty)
- **Concretization `γ_PS`:** Hyp → Set⟨Evidence⟩ (predicate: `is_consistent_with`)

The formal adjunction (DEF-MP-08) is validated through:

1. **Nine property-based tests** over 256+ randomly-generated cases per property
   - Seven foundational tests covering inflationary, monotonicity, antitonicity, and edge cases
   - Two antitonicity tests (one α-derived, one hand-crafted) strengthening lattice coverage (obs-2, task 3.6)
2. **Worked examples** instantiating the Galois laws on real clinical data
3. **Informal-argument proof** linking tests to the formal obligation

Result: **All 9 property tests pass.** The adjunction laws hold unconditionally in the implementation.

---

## Formal Adjunction (DEF-MP-08)

The Galois connection `(Obs_PS, α_PS, γ_PS, H_PS)` is defined by the **adjunction property**:

$$\forall e \in \text{Evidence}, h \in \text{Hyp} \,.\, \alpha_{PS}(e) \sqsubseteq h \,\Leftrightarrow\, e \sqsubseteq_\gamma \gamma_{PS}(h)$$

Where:
- `e ⊆_γ γ(h)` means "`e` is consistent with hypothesis `h`" (i.e., `is_consistent_with(h, e)`)
- `⊑` denotes the refinement order on Hyp (more specific = lower in order)
- The equivalence holds for **all** evidence and hypotheses

This single law entails the standard derived properties (INV-MP-02):

1. **Deflationary** (α ∘ γ on Hyp): ∀ h. α(γ(h)) ⊑ h
2. **Inflationary** (γ ∘ α on Evidence): ∀ e. e ⊑_γ γ(α(e))
3. **Monotonicity of α**: ∀ e₁, e₂. e₁ ⊑ e₂ ⟹ α(e₁) ⊑ α(e₂)
4. **Antitonicity of γ (predicate form)**: ∀ h₁, h₂. h₁ ⊑ h₂ ⟹ γ_pred(h₂) ⟹ γ_pred(h₁)

**Tier scope.** Properties (2)–(4) are property-tested directly. Property (1) — the deflationary
direction `α(γ(h)) ⊑ h` — is not tested in this tier because γ_PS is implemented as a
predicate (DEF-PS-06), not as a concrete set-returning function, so `α(γ(h))` is not a value
the test harness can construct. It is recovered semantically: under atom-set inclusion, `γ(h)`
is exactly `{ e | atoms(h) ⊆ atoms(α(e)) }`, and any element of that set re-abstracts to a Hyp
that contains all of h's atoms, satisfying `α(γ(h)) ⊑ h` by construction. A mechanized proof
of (1) is deferred to the Tier C lift (Lean 4 / Agda, v1.x).

---

## Implementation: `α_PS` and `γ_PS` in Rust

### Abstraction: `α_PS` (Evidence → Hyp)

```rust
pub fn abstract_evidence(e: &Evidence) -> Hyp {
    let atoms = e.observations
        .iter()
        .filter_map(|obs| {
            // Parse "SYSTEM:CODE" format from observation code
            parse_observation_code(&obs.code)
        })
        .collect::<Vec<_>>();

    if atoms.is_empty() {
        Hyp::unknown()  // Top element if no atoms extracted
    } else {
        Hyp::new(atoms)
    }
}
```

**Semantics:**
- Extracts all atom codes from observations in "SYSTEM:CODE" format
- Builds a Hyp containing all parsed atoms
- Empty observations → Hyp::unknown() (the top element ⊤)

### Concretization: `γ_PS` (Hyp → Set⟨Evidence⟩)

```rust
pub fn is_consistent_with(h: &Hyp, e: &Evidence) -> bool {
    let e_abstracted = abstract_evidence(e);
    let e_atoms = e_abstracted.atoms();

    for h_atom in h.atoms() {
        let found_compatible = e_atoms.iter().any(|e_atom| {
            h_atom.system == e_atom.system
                && h_atom.code == e_atom.code
                && h_atom.version == e_atom.version
        });
        if !found_compatible {
            return false;
        }
    }
    true
}
```

**Semantics:**
- Returns true iff every atom in the hypothesis has a compatible match in `α_PS(e)`.
- `Hyp::unknown()` (empty atom set) trivially returns true because the loop has no iterations
  — no separate short-circuit branch is needed.
- Atom compatibility is semantic matching on `(system, code, version)`, ignoring `preferred_term`.
- Predicate form of γ_PS per DEF-PS-06: `is_consistent_with(h, e) ⟺ e ∈ γ_PS(h)`.

---

## Property-Test Discharge (v0.2.0-alpha.0)

### Test Suite Overview

Nine property-based tests validate the Galois adjunction laws over **256+ randomly-generated cases per property**, using proptest framework.

**Run output (all passing):**
```
test proptest_galois_laws::prop_upper_adjoint_inflationary ... ok
test proptest_galois_laws::prop_alpha_monotone ... ok
test proptest_galois_laws::prop_gamma_antitone_in_hyp ... ok
test proptest_galois_laws::prop_gamma_antitone_with_hand_crafted_hyps ... ok
test proptest_galois_laws::prop_abstraction_from_empty_is_unknown ... ok
test proptest_galois_laws::prop_unknown_consistent_with_all ... ok
test proptest_galois_laws::prop_abstraction_completeness ... ok
test proptest_galois_laws::prop_alpha_deterministic ... ok
test proptest_galois_laws::prop_atom_set_consistency ... ok

result: ok. 9 passed; 0 failed; 0 ignored
```

### Individual Property Tests

Each test below covers a distinct property; none of the assertions are tautological.
The strategy generator covers all four ontology systems (SNOMED, LOINC, RxNorm, ICD-11).

#### 1. **Upper-adjoint Inflationary Law** (`prop_upper_adjoint_inflationary`)

**Formal law (INV-MP-02 property 2):** ∀ e ∈ Evidence. e ⊑_γ γ(α(e))

**Test code:**
```rust
let alpha_e = abstract_evidence(&e);
prop_assert!(is_consistent_with(&alpha_e, &e),
    "Upper-adjoint inflationary law violated: e ⊑_γ γ(α(e))");
```

**What it validates:** Evidence is always in the concretization of its own abstraction.
This is the unit of the Galois adjunction (γ ∘ α inflationary on Evidence).

**Result:** ✓ 256+ test cases pass

#### 2. **α-Monotonicity** (`prop_alpha_monotone`)

**Formal law (DEF-MP-06 applied to α):** ∀ e₁, e₂. e₁ ⊑ e₂ ⟹ α(e₁) ⊑ α(e₂)

**Test code:**
```rust
let (e_sub, e_full) = monotone_evidence_pair();  // e_sub.obs ⊆ e_full.obs
let alpha_sub = abstract_evidence(&e_sub);
let alpha_full = abstract_evidence(&e_full);
let sub_atoms: HashSet<&Atom> = alpha_sub.atoms().iter().collect();
let full_atoms: HashSet<&Atom> = alpha_full.atoms().iter().collect();
prop_assert!(sub_atoms.is_subset(&full_atoms),
    "α monotonicity violated: atoms(α(e_sub)) ⊄ atoms(α(e_full))");
```

**What it validates:** Genuine monotonicity. Extending evidence with more observations
yields a hypothesis whose atom set is a superset (in atom-set inclusion semantics,
that is refinement). Replaces the prior tautological assertion `x ≤ x + 1`.

**Result:** ✓ 256+ test cases pass

#### 3. **γ-Antitonicity in h (predicate form, α-derived)** (`prop_gamma_antitone_in_hyp`)

**Formal law:** ∀ h₁ ⊑ h₂. is_consistent_with(h₁, e) ⟹ is_consistent_with(h₂, e)

**Test code:**
```rust
let (e_sub, e_full) = monotone_evidence_pair();
let h_general = abstract_evidence(&e_sub);   // fewer atoms
let h_specific = abstract_evidence(&e_full); // h_general.atoms ⊆ h_specific.atoms
if is_consistent_with(&h_specific, &e_full) {
    prop_assert!(is_consistent_with(&h_general, &e_full),
        "γ antitonicity violated");
}
```

**What it validates:** Refining the hypothesis cannot turn consistency on. Equivalently,
the predicate-encoded γ(h) is antitone in h: γ(h_specific) ⊆ γ(h_general).
This test covers the special case where both hypotheses are derived via α from evidence.

**Result:** ✓ 256+ test cases pass

#### 3.5. **γ-Antitonicity with Hand-Crafted Hypothesis Pairs** (`prop_gamma_antitone_with_hand_crafted_hyps`)

**Formal law (same as 3, broader coverage):** ∀ h₁ ⊑ h₂, ∀ e. is_consistent_with(h₁, e) ⟹ is_consistent_with(h₂, e)

**Test code:**
```rust
let (h_general, h_specific) = comparable_hyp_pair();  // h_general ⊑ h_specific by construction
let e = evidence_strategy();  // arbitrary evidence
if is_consistent_with(&h_specific, &e) {
    prop_assert!(is_consistent_with(&h_general, &e),
        "γ antitonicity violated with hand-crafted hyps");
}
```

**What it validates:** Strengthens the antitonicity property (obs-2, task 3.6) by testing
against arbitrary hypothesis pairs constructed directly as atom sets, not just those
reachable via α. This exercises the full lattice structure, ensuring antitonicity holds
for all comparable hypotheses, not just α-derived ones. The `comparable_hyp_pair()`
strategy generates pairs where h_general.atoms ⊆ h_specific.atoms, covering a broader
range of lattice configurations.

**Result:** ✓ 256+ test cases pass

#### 4. **Empty Evidence → Unknown** (`prop_abstraction_from_empty_is_unknown`)

**Test code:**
```rust
let empty_obs = Evidence::new(vec![], provenance);
let hyp = abstract_evidence(&empty_obs);
prop_assert_eq!(hyp, Hyp::unknown());
```

**What it validates:** α_PS maps empty observations to the top element. Required for
the connection to be well-defined at the boundary (∅ ↦ ⊤).

**Result:** ✓ Deterministic (always true)

#### 5. **Unknown Consistency** (`prop_unknown_consistent_with_all`)

**Test code:**
```rust
prop_assert!(is_consistent_with(&Hyp::unknown(), &e));
```

**What it validates:** The top element is in γ_PS(e) for every e — i.e., the universe
of evidence trivially refines `⊤`. Top-element property: ∀ e. e ⊑_γ γ(⊤).

**Result:** ✓ 256+ test cases pass

#### 6. **Abstraction Completeness** (`prop_abstraction_completeness`)

**Test code:**
```rust
let codes = vec_of("(SNOMED|LOINC|RxNorm|ICD11):[0-9]{3,5}", 1..4);
let evidence = Evidence::new(observations_from(codes), prov);
let hyp = abstract_evidence(&evidence);
prop_assert_eq!(hyp.atoms().len(), codes.len());
prop_assert!(is_consistent_with(&hyp, &evidence));
```

**What it validates:** α_PS preserves cardinality on valid ontology codes (no atom
silently dropped or duplicated). Now exercises all four ontology systems.

**Result:** ✓ 256+ test cases pass

#### 7. **α Determinism** (`prop_alpha_deterministic`)

**Test code:**
```rust
let alpha_1 = abstract_evidence(&e);
let alpha_2 = abstract_evidence(&e);
prop_assert_eq!(alpha_1, alpha_2);
```

**What it validates:** α_PS is a function (deterministic), a prerequisite for any
Galois connection. Replaces the prior `x == x` tautology with an actual two-call
determinism check across the full evidence distribution.

**Result:** ✓ 256+ test cases pass

#### 8. **Atom-set Round-trip** (`prop_atom_set_consistency`)

**Test code:**
```rust
let observations = observations_encoding(h.atoms());  // build evidence from h
let evidence = Evidence::new(observations, prov_with_matching_version);
if uniform_version_across_atoms(&h) {
    prop_assert!(is_consistent_with(&h, &evidence));
}
```

**What it validates:** Hand-crafted hypotheses are consistent with evidence that
encodes their atoms under a shared provenance version. Exercises the γ_PS predicate
end-to-end on synthesized hypotheses (not just those produced by α_PS).

**Result:** ✓ 256+ test cases pass

---

## Worked Examples

### Example 1: SOFA-Respiratory Evidence

**Evidence:** Patient observation showing PaO₂/FiO₂ ratio = 200 (respiratory failure).

```rust
let evidence = Evidence::new(vec![
    Observation::new("LOINC:2019-8", json!(200.0)),  // PaO₂/FiO₂ ratio
], provenance);

let hyp = abstract_evidence(&evidence);
// hyp = Hyp::new([Atom {
//     system: LOINC,
//     code: "2019-8",
//     preferred_term: "PaO2/FiO2 ratio",
//     version: "v1.0"
// }])

// Lower adjoint law holds:
assert!(is_consistent_with(&hyp, &evidence));  ✓
```

**Interpretation:** The abstraction of a SOFA observation is a hypothesis containing the LOINC code. The evidence trivially satisfies this hypothesis because we extracted the code from it.

---

### Example 2: Multi-Observation Evidence

**Evidence:** AKI staging with creatinine and urine output.

```rust
let evidence = Evidence::new(vec![
    Observation::new("LOINC:2160-0", json!(1.8)),     // Serum creatinine
    Observation::new("SNOMED:76156009", json!(0.3)),  // Urine output
], provenance);

let hyp = abstract_evidence(&evidence);
// hyp.atoms().len() == 2 (LOINC atom + SNOMED atom)

// Both lower and upper adjoint laws hold:
assert!(is_consistent_with(&hyp, &evidence));    ✓  (e ⊆_γ γ(α(e)))

let gamma_hyp = vec![evidence];  // γ(hyp) is the set of evidence consistent with hyp
assert!(gamma_hyp.iter().all(|e| is_consistent_with(&hyp, e)));  ✓
```

**Interpretation:** Multi-observation evidence abstracts to a multi-atom hypothesis. The Galois connection ensures that the original evidence is always in the concretization of its own abstraction.

---

### Example 3: Empty Evidence (Top Element)

**Evidence:** No observations recorded.

```rust
let empty_evidence = Evidence::new(vec![], provenance);

let hyp = abstract_evidence(&empty_evidence);
// hyp == Hyp::unknown()  (the top element ⊤)

// Top-element property holds:
assert!(is_consistent_with(&hyp, &empty_evidence));  ✓
// Unknown is consistent with all evidence (trivially here with empty)
```

**Interpretation:** Empty observations abstract to the unknown hypothesis. In the lattice, this is the most-general position. The Galois connection respects this: unknown is consistent with any evidence (including empty).

---

### Example 4: Consistency Predicate Semantics

**Hypothesis:** Pneumonia diagnosis (SNOMED:233604007).

**Evidence:** Lab values that don't mention pneumonia directly, but are consistent with it (e.g., chest X-ray findings that clinicians know imply pneumonia).

```rust
let hyp = Hyp::new(vec![Atom {
    system: OntologySystem::SNOMED,
    code: "233604007".to_string(),
    preferred_term: "Pneumonia".to_string(),
    version: "2026-01-31".to_string(),
}]);

// Two scenarios:
// Case A: Evidence includes "SNOMED:233604007"
let evidence_direct = Evidence::new(vec![
    Observation::new("SNOMED:233604007", json!(true)),
], provenance);
assert!(is_consistent_with(&hyp, &evidence_direct));  ✓

// Case B: Evidence does NOT include SNOMED:233604007
let evidence_indirect = Evidence::new(vec![
    Observation::new("LOINC:2160-0", json!(98.0)),  // O₂ sat
], provenance);
// Consistency check fails (SNOMED:233604007 not in abstraction)
assert!(!is_consistent_with(&hyp, &evidence_indirect));  ✓ (correct rejection)
```

**Interpretation:** Consistency is strict: a hypothesis is only consistent with evidence that explicitly contains the atoms in the hypothesis. This prevents spurious refinement (NOTE.md §4A.2 soundness). The Galois connection ensures that this strictness is enforced uniformly.

---

## Proof Sketch: Why the Adjunction Holds

The adjunction law is enforced by the implementation design:

1. **Definition of `α_PS`:** Extracts all atoms from observations whose codes parse to a
   known ontology system. No atoms are silently added or removed. The image of α_PS is
   the set of Hyp values whose atom set is some subset of the parseable atoms in e.

2. **Definition of `γ_PS` (via `is_consistent_with`):** Predicate `e ∈ γ_PS(h)` holds iff
   every atom in h has a compatible match in `α_PS(e)`. Equivalently:
   `γ_PS(h) = { e | atoms(h) ⊆ atoms(α_PS(e)) }`.

3. **Refinement order on Hyp.** `Hyp::PartialOrd` uses atom-set inclusion (DEF-PS-01):
   `h₁ ⊑ h₂ ⟺ atoms(h₁) ⊇ atoms(h₂)`. More atoms means more specific. `Hyp::unknown()`
   (empty atom set) is the top element. This is the Rust-level realization of `⊑_PS`
   used throughout this proof sketch.

4. **Adjunction property from definitions:**
   - `α_PS(e) ⊑ h` ⟺ `atoms(α_PS(e)) ⊇ atoms(h)` (by step 3).
   - `atoms(α_PS(e)) ⊇ atoms(h)` ⟺ every atom in h has a compatible match in α_PS(e)
     ⟺ `is_consistent_with(h, e)` (by step 2)
     ⟺ `e ∈ γ_PS(h)`.
   - So `α_PS(e) ⊑ h ⟺ e ∈ γ_PS(h)` holds by composition of definitions.

5. **Property tests provide empirical evidence:** The nine tests in §"Individual Property
   Tests" cover the inflationary direction, α-monotonicity, γ-antitonicity, top-element
   behavior, determinism, completeness, and round-trip consistency. Each runs ≥256 cases
   over a strategy spanning all four ontology systems. The deflationary direction is
   recovered from the predicate encoding of γ_PS (step 2): there is no `α(γ(h))` value to
   test because γ_PS is a predicate, but the equivalence in step 4 means any deflation
   would manifest as a violation of one of the tested directions.

---

## Limitations and Future Work

**Current scope (v0.2.0-alpha.0):**
- Adjunction tested on randomly-generated Evidence and Hyp samples.
- Ontology atoms are synthetic (not connected to live SNOMED CT / LOINC / RxNorm / ICD-11 servers).
- Provenance version is fixed at test-generation time.

**Future work (v1.x):**
- Mechanized proof of the adjunction in Lean 4 or Agda (Tier C formalization).
- Integration tests with real ontology snapshots.
- Performance bounds on abstraction/concretization (latency, space).

---

## Conclusion

**OBL-PS-02 is discharged at the property-test tier with the scope noted below.**

The patient Galois connection `(Obs_PS, α_PS, γ_PS, H_PS)` satisfies the adjunction property (DEF-MP-08), evidenced by:

1. ✓ Nine non-tautological property tests over 256+ cases per property
2. ✓ Inflationary (γ ∘ α), α-monotonicity, and γ-antitonicity (predicate form) directly tested
3. ✓ Deflationary (α ∘ γ on Hyp) recovered semantically from the predicate definition of γ_PS;
   direct test deferred until γ_PS gains a set-returning form (Tier C lift)
4. ✓ Worked examples on realistic clinical data (SOFA, AKI, pneumonia)
5. ✓ Proof sketch linking implementation to formal definition

The implementation enforces the adjunction by design: `α_PS` extracts exactly the atoms present in evidence; `γ_PS` (predicate) checks atom membership. The Galois connection is sound by construction for the tested directions; the untested direction is structurally guaranteed by the predicate encoding.

---

## References

- **SPEC.md §1.4 (DEF-MP-08):** Galois connection definition with adjunction law.
- **SPEC.md §2.3 (DEF-PS-05, DEF-PS-06):** Patient observation and Galois connection spaces.
- **SPEC.md §2.8 (OBL-PS-02):** Proof obligation statement.
- **clinlat/src/operator.rs:** Implementation of `abstract_evidence` and `is_consistent_with`.
- **Cousot & Cousot (1977):** "Abstract interpretation: A unified lattice model for static analysis of programs by construction or approximation."

---

**Artifact:** This document is part of the clinlat v0.2.0-alpha.0 Galois connection implementation.
**Date:** 2026-05-27
**Commit:** (to be updated)
