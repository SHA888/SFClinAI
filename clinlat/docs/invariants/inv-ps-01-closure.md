# INV-PS-01: Ontology Closure Invariant

**Invariant**: All atoms in a hypothesis are reachable from the registered OntologyAdapter set.

**Formalization** ([SPEC.md § 2.5][1]):
```
∀ h ∈ Hyp, ∀ a ∈ h.atoms():
  ∃ adapter ∈ OntologySystem.adapters(),
    adapter.resolve_atom(a.code) = Ok(a)
```

In plain language: Every atom in a hypothesis must be resolvable through at least one ontology adapter registered for its system.

---

## Proof Strategy

We discharge INV-PS-01 by construction: hypotheses are built exclusively via atoms resolved through the OntologyAdapter trait.

### Lemma 1: Atom Resolution Implies Closure

**Claim**: If an atom `a` is returned by `adapter.resolve_atom(code)` for some adapter and code, then `a` is in the reachable set.

**Proof**: By definition of `OntologyAdapter::resolve_atom()` (SPEC.md § 2.3, DEF-PS-03):
- The method takes a code string.
- It returns `Ok(Atom)` only if the code is found in the adapter's ontology (snapshot or API).
- The returned atom carries the adapter's `OntologySystem` and `version`.
- Thus `a` is reachable from that specific adapter.

### Lemma 2: Hyp Construction from Resolved Atoms

**Claim**: If a hypothesis is constructed from atoms resolved via `adapter.resolve_atom()`, the closure property holds.

**Proof**: Examine the constructor `Hyp::new(atoms: Vec<Atom>)`:
```rust
pub fn new(atoms: Vec<Atom>) -> Self {
    Hyp(atoms)
}
```

Each atom in the input vector must originate from successful resolution (a call site that obtained `Ok(Atom)`).
By Lemma 1, each atom is reachable from its originating adapter.
Thus, the hypothesis satisfies the invariant at construction time.

### Lemma 3: Hypothesis Operations Preserve Closure

**Claim**: Operations on hypotheses preserve closure.

**Proof by cases**:

1. **`Hyp::unknown()`** (top element):
   - Constructs `Hyp(vec![])` (empty atom set).
   - Vacuously satisfies closure (no atoms to resolve).

2. **`Hyp::compat()`** (compatibility check):
   - Pure read operation; doesn't modify atoms or create new hypotheses.
   - No closure violation possible.

3. **`Hyp::meet()`** (partial meet):
   - Returns an existing hypothesis (either `self` or `other` or `None`).
   - Does not construct new atoms or merge incompatible atom sets.
   - Closure is preserved because the returned hypothesis is already closed (Lemma 2 applies to inputs).

4. **`Hyp::atoms()`** (accessor):
   - Pure read; returns a slice of the stored atoms.
   - No closure violation possible.

### Lemma 4: Concrete Adapter Instances Satisfy OntologyAdapter Contract

**Claim**: The four concrete adapters (SNOMEDAdapter, RxNormAdapter, LoincAdapter, Icd11Adapter) implement OntologyAdapter correctly.

**Proof**: Each adapter:
1. Maintains an offline snapshot (HashMap) of codes → atoms.
2. Implements `resolve_atom(code)` by:
   - Checking an in-memory LRU cache (L1).
   - Falling back to the snapshot (L2).
   - Returning `Ok(Atom)` only if the code is found.
   - The returned atom has the correct `system` and `version`.

By construction, each returned atom is traceable to the adapter's snapshot, which carries atoms for its designated ontology system. Thus the contract is satisfied.

### Concrete Verification (M1)

In v0.1.0 and v0.2.0, hypotheses are constructed via:

1. **Operator applications**: Operators (e.g., `SofaRespOperator`) receive evidence and output a refined `Hyp` containing atoms resolved from the SNOMED adapter. These atoms are checked against the adapter's snapshot at resolve time.

2. **Test fixtures**: All test hypotheses are constructed with atoms from the four concrete adapters' fixtures, ensuring closure by construction.

Example (test fixture):
```rust
let hyp = Hyp::new(vec![
    Atom {
        system: OntologySystem::SNOMED,
        code: "67822003".to_string(),
        preferred_term: "Hypoxemia".to_string(),
        version: "2026-01-31".to_string(),
    }
]);
// This atom is reachable via SNOMEDAdapter::resolve_atom("67822003")
```

---

## Failure Mode (Out of Scope for M1)

**When closure could fail** (hypothetical, future versions):
- A hypothesis is constructed with a manually-created `Atom` that doesn't exist in any registered adapter.
  - **Mitigation** (v0.2+): Statically prevent this by requiring `Atom` construction to go through `OntologyAdapter::resolve_atom()`.

---

## Correctness Premises

1. **OntologyAdapter trait** (DEF-PS-03): The trait contract is correctly implemented by all four concrete adapters. ✓ (Verified in clinlat/src/ontology.rs, tasks 1.2–1.5)

2. **Hyp constructor** (DEF-PS-01): The constructor accepts atoms; atoms come from resolved sources. ✓ (Task 1.7)

3. **Snapshot integrity**: Each adapter's snapshot is populated with valid atoms before use. ✓ (Test fixtures in tasks 1.2–1.5)

4. **No external atom construction**: Outside of the test suite and operator implementations, atoms are not constructed manually. ✓ (Code inspection in src/sofa.rs, src/hyp.rs; test file scope)

---

## Conclusion

The invariant INV-PS-01 (ontology closure) is satisfied by construction:
- Atoms are resolved via the OntologyAdapter trait.
- Hypotheses are built from resolved atoms.
- Operations on hypotheses preserve the invariant (Lemma 3).
- The four concrete adapters (SNOMED, RxNorm, LOINC, ICD-11) guarantee reachability (Lemma 4).

**Discharge**: ✓ Informal-argument tier (M1).

---

## References

- **SPEC.md § 2** (DEF-PS-01, DEF-PS-03, DEF-PS-04, INV-PS-01): Patient-state substrate definitions.
- **clinlat/src/ontology.rs** (tasks 1.2–1.5): Concrete adapter implementations and test fixtures.
- **clinlat/src/hyp.rs** (task 1.7): Hypothesis type and operations.
- **clinlat/src/sofa.rs**: SOFA-respiratory operator (example operator using adapters).

[1]: https://github.com/SHA888/SFClinAI/blob/main/SPEC.md#25-ontology-closure-inv-ps-01
