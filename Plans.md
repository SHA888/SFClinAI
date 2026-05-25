# SFClinAI — Plans.md

**Project:** clinlat substrate kernel v0.2.0
**Milestone:** M1 — Patient substrate completion
**Created:** 2026-05-25
**Status:** In progress
**Architectural Scope:** Complete NOTE.md §4A / SPEC.md §2 / ARCHITECTURE.md Diagrams 1–3 patient-substrate side.

---

## Phase 0: Architectural decisions and design docs

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 0.1 | **D1 Decision: Provenance encoding** — decide CBOR vs. JSON vs. Merkle DAG vs. hybrid for `Provenance` carrier type. Criteria: serialization overhead, query-ability, audit trail fidelity per OBL-PS-04. Document trade-offs in `DESIGN-D1-provenance.md` | Decision document with rationale and implementation sketch; consensus in team/advisor review if applicable | - | cc:done [7fa0c69] |
| 0.2 | **D2 Decision: Ontology adapter caching strategy** — decide in-memory cache vs. Redis vs. offline snapshot vs. hybrid for SNOMED CT, RxNorm, LOINC, ICD-11 access. Constraints per M1.1: DEF-PS-03/04, INV-PS-01. Document in `DESIGN-D2-ontology.md` | Decision document with caching topology sketch, API contract for OntologyAdapter trait | 0.1 | cc:done [a621f2f] |
| 0.3 | Write **spec SSOT for M1 provenance contract** (`docs/spec/M1-provenance-spec.md`) — formalizes the Provenance type signature, serialization, deserialization, query interface; anchors to DEF-MP-14, DEF-PS-12, DEF-PS-13, INV-PS-05, OBL-PS-04 | Spec document with type signatures, invariant proofs, example encoded/decoded Provenance values | 0.1 | cc:done [139ecfa] |

---

## Phase 1: Ontology infrastructure (M1.1)

**Goal:** Replace `&'static str` AtomId with adapters for SNOMED CT, RxNorm, LOINC, ICD-11. Discharge DEF-PS-03, DEF-PS-04, INV-PS-01, OBL-PS-01. Diagram 1's `OB` node becomes runnable.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 1.1 | Define `OntologyAdapter` trait signature | `pub trait OntologyAdapter { fn resolve_atom(&self, code: &str, system: OntologySystem) -> Result<Atom, OntologyError>; fn validate_compatibility(&self, atom1: &Atom, atom2: &Atom) -> bool; }` with doc anchoring to DEF-PS-03 | 0.2 | cc:done [97b0304] |
| 1.2 | Implement `SnomedAdapter` — thin client for SNOMED CT API/snapshot (per M1.1 scope) | Adapter impl with ≥3 example codes; `cargo test` passes; doc refs SNOMED CT Edition reference | 1.1 [tdd:required] | cc:todo |
| 1.3 | Implement `RxNormAdapter` — thin client for RxNorm (drugs, strengths) | Adapter impl with ≥3 example drug codes; `cargo test` passes | 1.1 [tdd:required] | cc:done [8e085f0] |
| 1.4 | Implement `LoincAdapter` — thin client for LOINC (lab tests, vital signs) | Adapter impl with ≥3 example LOINC codes; `cargo test` passes | 1.1 [tdd:required] | cc:done [dad0fb0] |
| 1.5 | Implement `Icd11Adapter` — thin client for ICD-11 (diagnoses, procedure codes) | Adapter impl with ≥3 example ICD-11 codes; `cargo test` passes | 1.1 [tdd:required] | cc:done [68f6fc3] |
| 1.6 | Define `Atom` type as replacement for `&'static str` AtomId | `pub struct Atom { system: OntologySystem, code: String, preferred_term: String }` with PartialEq, Hash, Clone; doc names DEF-PS-03 | 1.1 | cc:done [1.2–1.5 impl] |
| 1.7 | Update `Hyp` struct to use `Atom` instead of `&'static str`; preserve refinement order semantics | `Hyp` variants now carry `Atom` payloads; PartialOrd / compatibility / meet logic unchanged; all existing tests pass | 1.2, 1.3, 1.4, 1.5, 1.6 [tdd:required] | cc:done [8d7593c] |
| 1.8 | Write INV-PS-01 proof (ontology closure) — show that all atoms in a Hyp are reachable from resolving the registered OntologyAdapter set | Informal-argument doc `clinlat/docs/invariants/inv-ps-01-closure.md`; cite adapters as correctness premises | 1.7 | cc:done [204f375] |

---

## Phase 2: Provenance carrier (M1.2)

**Goal:** Replace `()` stub with a typed provenance carrier supporting DEF-MP-14, DEF-PS-12, DEF-PS-13, INV-PS-05, OBL-PS-04. Encoding choice (CBOR vs. JSON vs. Merkle DAG) decided in DESIGN-D1.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 2.1 | Implement `Provenance` type per DESIGN-D1 and spec SSOT (0.3) | `pub struct Provenance { origin: DataSource, timestamp: SystemTime, version: Ver, metadata: BTreeMap<String, Value> }` with serialization/deserialization | 0.1, 0.3 | cc:todo |
| 2.2 | Update `Evidence` struct to carry typed `Provenance` instead of `()` | `pub struct Evidence { observations: Vec<Observation>, provenance: Provenance }`; preserve Evidence::new constructor signature | 2.1 | cc:todo |
| 2.3 | Update `SofaRespOperator.apply()` to extract and validate provenance per OBL-PS-04 | Implementation validates `provenance.version` matches operator version; emits abstention if mismatch; property test: version invariant held | 2.1, 2.2 [tdd:required] | cc:todo |
| 2.4 | Write OBL-PS-04 discharge proof (provenance audit-trail fidelity) — show that operator output provenance carries source, timestamp, version; audit queries answerable | Informal-argument doc `clinlat/docs/obligations/obl-ps-04-provenance-audit.md`; worked example: trace SOFA-respiratory evidence back to source | 2.3 | cc:todo |

---

## Phase 3: Galois connection and abstraction (M1.3)

**Goal:** Implement `α_PS`, `γ_PS` per DEF-PS-05/06; discharge OBL-PS-02 (adjunction laws) at property-test tier.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 3.1 | Define abstraction function `α_PS: Evidence → Hyp` | Signature `fn abstract_evidence(e: &Evidence) -> Hyp` mapping observed facts to patient hypotheses; doc anchors to DEF-PS-05 | 0.3 | cc:todo |
| 3.2 | Define concretization function `γ_PS: Hyp → Set<Evidence>` | Signature `fn concretize_hypothesis(h: &Hyp) -> Set<Evidence>` (represented as predicate `fn is_consistent_with(&Hyp, &Evidence) -> bool`); doc anchors to DEF-PS-06 | 3.1 | cc:todo |
| 3.3 | Implement adjunction property tests for α_PS and γ_PS | Property tests verify: (1) `e ∈ γ_PS(α_PS(e))` (lower adjoint), (2) `α_PS(γ_PS(h)) ⊑ h` (upper adjoint), (3) monotonicity; property test framework (proptest); ≥10 generated test cases | 3.1, 3.2 [tdd:required] | cc:todo |
| 3.4 | Write OBL-PS-02 discharge proof (adjunction sound) — show that the adjoint laws hold unconditionally | Property-test tier doc `clinlat/docs/obligations/obl-ps-02-adjunction.md` with test suite output | 3.3 | cc:todo |

---

## Phase 4: Operator-set formalization (M1.4)

**Goal:** Formalize the operator collection per DEF-PS-09; extend soundness obligation OBL-PS-03 across the set.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 4.1 | Define `OperatorSet` type (Δ_PS) | `pub struct OperatorSet { operators: Vec<Box<dyn Operator>>, metadata: BTreeMap<String, OperatorMetadata> }` with DEF-PS-09 semantics | 1.7, 2.2 | cc:todo |
| 4.2 | Implement OperatorSet::apply_set() method | Method applies all registered operators in sequence, collects refined Hyp or first abstention; preserves refinement order from Phase 1 | 4.1 [tdd:required] | cc:todo |
| 4.3 | Property-test OperatorSet soundness (OBL-PS-03 across the set) | Tests verify: (1) operator composition preserves refinement order, (2) no silent contradictions between operator outputs, (3) abstention from one doesn't silence another; ≥15 property cases | 4.2 [tdd:required] | cc:todo |
| 4.4 | Write OBL-PS-03 discharge proof (operator-set soundness) — property-test tier | Doc `clinlat/docs/obligations/obl-ps-03-operator-set-sound.md` with test suite summary | 4.3 | cc:todo |

---

## Phase 5: Additional operators (M1.5)

**Goal:** Three additional operators matching NOTE.md §7E worked examples, each with informal-argument discharge plus property tests (KDIGO AKI, Wells/PE, CURB-65).

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 5.1 | **KDIGO AKI Staging Operator** — encode KDIGO criteria (creatinine fold-change, UO decline) | `KdigoAkiOperator` impl per Kidney Disease: Improving Global Outcomes guideline; handles stages 0–3; `cargo test` passes [tdd:required] | 1.7, 2.2 | cc:todo |
| 5.2 | **Wells Score Operator** — PE risk stratification with sequential testing | `WellsPeOperator` impl: encodes criteria (leg swelling, HR, RV strain, etc.); outputs: low/intermediate/high risk; handles missing evidence via abstention [tdd:required] | 1.7, 2.2 | cc:todo |
| 5.3 | **CURB-65 Operator** — CAP disposition (outpatient vs. admission) | `Curb65Operator` impl per BTS CAP guideline; inputs: confusion, BUN, RR, BP, age ≥65; outputs: recommendation for care setting; handles missing evidence [tdd:required] | 1.7, 2.2 | cc:todo |
| 5.4 | Write soundness argument for KDIGO AKI operator | Doc `clinlat/docs/operators/kdigo_aki_soundness.md`; cite KDIGO 2021 clinical practice guideline; informal-argument tier; state three DEF-PS-08 soundness clauses | 5.1 | cc:todo |
| 5.5 | Write soundness argument for Wells PE operator | Doc `clinlat/docs/operators/wells_pe_soundness.md`; cite Wells et al. 1997/2006; informal-argument tier; note sequential-testing constraint | 5.2 | cc:todo |
| 5.6 | Write soundness argument for CURB-65 operator | Doc `clinlat/docs/operators/curb65_soundness.md`; cite BTS CAP guideline; informal-argument tier | 5.3 | cc:todo |

---

## Phase 6: Discharge-tier upgrade (M1.6)

**Goal:** Upgrade SOFA-respiratory from informal-argument to property-test tier; refresh soundness doc.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 6.1 | Refactor `SofaRespOperator` to use new Atom/Provenance infrastructure (Phase 1–2) | Operator updated to work with Phase 1's Atom and Phase 2's Provenance; all v0.1.0 tests still pass | 1.7, 2.3 | cc:todo |
| 6.2 | Expand SOFA-respiratory test suite to property-test tier | Use proptest to generate arbitrary PaO₂/FiO₂ ratios; verify: (1) monotonicity (lower ratio → same or worse score), (2) threshold boundaries (no gaps), (3) abstention on invalid input; ≥20 property cases | 6.1 [tdd:required] | cc:todo |
| 6.3 | Refresh `clinlat/docs/operators/sofa_resp_soundness.md` | Upgrade from "informal-argument tier" to "property-test tier"; add reference to test suite output; preserve clinical citations (Vincent 1996, Singer 2016 Sepsis-3) | 6.2 | cc:todo |

---

## Phase 7: Integration and release

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 7.1 | Update `clinlat/README.md` with M1 examples | Add worked examples for KDIGO AKI and Wells/PE alongside SOFA-respiratory; link to soundness docs | 5.4, 5.5, 5.6 | cc:todo |
| 7.2 | Update SPEC.md ARCHITECTURE.md cross-references | Ensure all M1 formal definitions (DEF-PS-03, etc.) linked from SPEC §2; all Diagram 1–3 nodes anchored | 1.1 through 6.3 | cc:todo |
| 7.3 | Verify `cargo test`, `cargo doc --no-deps` green; CI matrix passes | All checks pass: fmt, clippy, test, doc, semver, MSRV | 7.1, 7.2 | cc:todo |
| 7.4 | Dry-run publish verification | `cargo publish --dry-run` succeeds without errors | 7.3 | cc:todo |

---

## Definition of Done for M1

✓ All eleven 4A-anchored SPEC.md elements (DEF-PS-01..15, INV-PS-01..06, OBL-PS-01..05) reachable from running code
✓ Four operators discharged at property-test tier minimum (SOFA-respiratory, KDIGO AKI, Wells/PE, CURB-65)
✓ `clinlat` crate compiles, tests pass, docs render
✓ Ontology adapters (SNOMED, RxNorm, LOINC, ICD-11) integrated; Atom replaces `&'static str` throughout
✓ Provenance carrier typed per OBL-PS-04; audit-trail fidelity demonstrated
✓ Galois connection (α_PS, γ_PS) property-tested per OBL-PS-02
✓ Operator-set type and composition formalized per DEF-PS-09, OBL-PS-03

---

## Explicitly out of scope for M1

These belong to M2+ backlog:

- Proposer (constrained refinement suggester) — DEF-PS-14, M2
- Institutional substrate (§3) — M3
- Interaction layer (§4C) — M4
- Temporal evolution (§5) — M5
- Cross-cutting concerns (§6) — M6

---

## Next session startup

**New session command:**
```bash
claude
```

**First input:**
```
/harness-work 1.1
```

**Rationale:** Phase 1 task 1.1 (OntologyAdapter trait) is the critical blocker for all downstream tasks in Phases 1–2. Once the trait and adapters land, ontology work unblocks provenance work, which unblocks the Galois connection and operator-set formalization.

Alternatively, if you prefer to work through independent tracks in parallel:

```bash
ENABLE_PROMPT_CACHING_1H=1 claude
```

**First input:**
```
/breezing all
```

**Rationale:** Decision documents (Phase 0) can be drafted in parallel with adapter implementations (Phase 1) once decisions are outlined. Phases 3–4 (Galois connection, operator-set) can start once Phase 1.7 is complete and are otherwise independent of Phase 2 (provenance). Phase 5 (three operators) runs in parallel once Phase 1 is solid.

---

## Notes on discipline

- **TDD adoption:** All implementation tasks (1.2–1.7, 2.3, 3.3, 4.2–4.3, 5.1–5.3, 6.1–6.2) are marked `[tdd:required]`. Write failing tests first.
- **Soundness discharge:** Six tasks (1.8, 2.4, 3.4, 4.4, 5.4–5.6) are pure documentation. These constitute the informal-argument and property-test tier discharges per SPEC.md §6.
- **Design decisions:** Tasks 0.1–0.3 must complete before Phases 1–2 implementation begins; they are critical gates per NOTE.md §4D (sound-evolution checking framing).
