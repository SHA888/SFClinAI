# SFClinAI — Plans.md

**Project:** clinlat substrate kernel
**Current Milestone:** M3 — Institutional substrate kernel (`clinlat-v0.4.0`, cc:TODO — scoped 2026-07-22)
**Previous Milestone:** M2 (✓ Complete, shipped 2026-07-10)
**Created:** 2026-05-25
**Status:** M2 shipped (`clinlat-v0.3.0`, tagged `dcd6226`, live on crates.io). M3 task table generated from `TODO.md`'s M3 roadmap entry; SPEC.md §3 (institutional-state substrate) already formalizes the full scope (DEF-IS-01..15, INV-IS-01..06, OBL-IS-01..06) — **Spec skip reason: no `Spec delta` needed, M3 implements existing SPEC.md §3 formalization**, mirroring how M1 implemented §2 and M2 implemented §2.7.
**Architectural Scope:** Realize NOTE.md §4B / SPEC.md §3 / ARCHITECTURE.md institutional-substrate nodes in Diagrams 1, 3, 5 (peer structure to the patient substrate per INV-IX-04 — substrate-local soundness independent of coupling).

---

## Phases 0–7: M1 implementation (archived to docs/archives/ARCHIVE-M1.md)

All M1 implementation phases are complete and shipped (clinlat v0.1.0 / v0.2.0, 2026-05-31). Full task tables and commits are in `docs/archives/ARCHIVE-M1.md`; bugfix detail in `docs/archives/ARCHIVE-6BF.md`.

- **Phase 0 — Architectural decisions:** D1/D2 decisions + M1 provenance spec SSOT. (0.1–0.3)
- **Phase 1 — Ontology infrastructure (M1.1):** `OntologyAdapter` trait + SNOMED/RxNorm/LOINC/ICD-11 adapters, `Atom` type replacing `&'static str`, INV-PS-01 closure proof. (1.1–1.8)
- **Phase 2 — Provenance carrier (M1.2):** typed `Provenance` + `Evidence` carrier, OBL-PS-04 discharge. (2.1–2.4)
- **Phase 3 — Galois connection (M1.3):** `α_PS`/`γ_PS`, OBL-PS-02 adjunction at property-test tier, INV-PS-01 reconciliation. (3.1–3.7)
- **Phase 4 — Operator-set formalization (M1.4):** `OperatorSet` (Δ_PS), propagate-forward `apply_set`, OBL-PS-03 discharge. (4.1–4.4)
- **Phase 5 — Additional operators (M1.5):** KDIGO AKI, Wells/PE, CURB-65 with soundness arguments. (5.1–5.6)
- **Phase 5-BF — Code review bugfixes:** 9 operator bugs fixed (193 tests green). (5-BF.1–5-BF.3)
- **Phase 6 — Discharge-tier upgrade (M1.6):** SOFA-respiratory raised to property-test tier. (6.1–6.3)
- **Phase 6-BF — Bugfix:** 11 bugs fixed (see ARCHIVE-6BF.md).
- **Phase 7 — Integration and release:** README, cross-references, CI green, dry-run publish. (7.1–7.4)

The M1 Definition of Done (met, shipped 2026-05-31) and the M1 out-of-scope backlog note are recorded in `docs/archives/ARCHIVE-M1.md`.

---

## Phases 8–12: M2 implementation (archived to docs/archives/ARCHIVE-M2.md)

All M2 implementation phases are complete and shipped (`clinlat-v0.3.0`, 2026-07-10). Full task tables and commits are in `docs/archives/ARCHIVE-M2.md`.

- **Phase 8 — Proposer interface and soundness gate (M2.1/M2.2/M2.3):** `RefinementProposer` trait, `ProposerConstraint` gates, `propose_and_filter`/`propose_verify` adapters, INV-PS-06 structural enforcement. (8.1–8.6)
- **Phase 9 — Deterministic search proposer (M2.4):** `LatticeSearchProposer`, completeness property tests, sepsis-3 worked example. (9.1–9.3)
- **Phase 10 — LLM-class adapter (M2.5):** `LlmProposer`, safety/robustness property tests, sepsis-3 worked example. (10.1–10.4)
- **Phase 11 — OBL-PS-05 discharge (M2.6):** discharge doc, substrate-invariance test, substrate-first worked example. (11.1–11.3)
- **Phase 12 — Integration and release (M2.7):** CHANGELOG, README, `0.3.0` bump, CI green, dry-run publish. (12.0–12.4)

The M2 Definition of Done (met, shipped 2026-07-10) is recorded in `docs/archives/ARCHIVE-M2.md`.

---

## Phase 13: Institutional state space and capacity infrastructure (M3.1)

**Goal:** Implement the institutional-state poset, resource-bounded sets, physical capacity bounds, the institutional event space, and the institutional Galois connection — the structural foundation everything else in M3 builds on. Mirrors Phases 1 and 3 of M1.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 13.1 | Define `Cap` / `CapacityHypothesis` type and `⊑_IS` poset, `⊤_IS` top element | Per DEF-IS-01/02: `CapacityHypothesis` struct with a partial-order `⊑_IS` ("at least as committed as"); `Cap::top()` constructs `⊤_IS` (fully uncommitted) satisfying `∀c. c ⊑_IS ⊤_IS`. Doc anchors to SPEC.md §3.1. Partial-meet poset structure (MC-1) documented. | 7.4 [tdd:required] | cc:TODO |
| 13.2 | Implement `compat_IS` predicate | Two capacity hypotheses are compatible iff their union does not exceed any physical bound (forward-references DEF-IS-04; stub `cap` lookup acceptable here, wired fully in 13.5). INV-IS-01 (compatibility under refinement) property test, ≥10 cases. | 13.1 [tdd:required] | cc:TODO |
| 13.3 | Implement meet `⊓_IS` | Unique most-uncommitted capacity hypothesis at least as committed as both inputs, for compatible pairs, per INV-IS-02. Property test mirroring Phase 3's Galois-connection meet tests. | 13.2 [tdd:required] | cc:TODO |
| 13.4 | Define `ResourceBoundedSet` / `Resource` enumeration | Per DEF-IS-03: institutional analog of `OntologyBoundedSet` (DEF-PS-03) covering physical resource units (beds, ICU bays, OR rooms, ventilators), consumable classes (formulary, reagents), time-divisible slots (lab queue, OR block hours), personnel role assignments. Each resource carries a version identifier and source attribution. OBL-IS-02 (resource decidability) test: free-form resource identifiers rejected at construction. | 13.1 [tdd:required] | cc:TODO |
| 13.5 | Implement physical capacity bound `cap: R → ℕ ∪ {∞}` | Per DEF-IS-04: versioned function (`ver(cap): Ver`) giving max simultaneous instances per resource; `is_physically_valid(c: &CapacityHypothesis) -> bool` checks committed-resource count per `r ∈ R` does not exceed `cap(r)`. Wire into 13.2's `compat_IS` (remove stub). | 13.4 [tdd:required] | cc:TODO |
| 13.6 | Define institutional event space `Evt_IS` | Per DEF-IS-05: poset of timestamped, provenance-tagged operational events (admissions, discharges, transfers, supply deliveries, shift changes, allocation requests/releases); `e₁ ⊑_Evt e₂` iff `e₁`'s event multiset ⊇ `e₂`'s. Doc anchor to SPEC.md §3.3. | 13.1 | cc:TODO |
| 13.7 | Implement institutional Galois connection `(α_IS, γ_IS)` | Per DEF-IS-06: `α_IS: Evt → Cap` (event history → most-committed hypothesis it entails), `γ_IS: Cap → Evt` (hypothesis → consistent event histories), satisfying DEF-MP-08. OBL-IS-03 (adjunction soundness) property test: `∀e,c. α_IS(e) ⊑_IS c ⟺ e ⊑_Evt γ_IS(c)`, ≥15 cases mirroring Phase 3's OBL-PS-02 discharge. | 13.3, 13.5, 13.6 [tdd:required] | cc:TODO |

---

## Phase 14: Capacity-update operators (M3.2)

**Goal:** Define the capacity-update operator signature and operator-set formalization, then implement the four concrete operators named in `TODO.md`'s M3.2. Mirrors Phase 4 (operator-set formalization) and Phase 5 (concrete operators) of M1.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 14.1 | Define `CapacityUpdateOperator` trait signature | Per DEF-IS-07: `fn apply(&self, c: &Cap, e: &InstEvidence) -> Result<Cap, AbstainReason_IS>` (forward-declares `InstEvidence` per Phase 16, `AbstainReason_IS` per Phase 15 — stub types acceptable, refined in those phases). Doc anchor to SPEC.md §3.4. | 13.7 | cc:TODO |
| 14.2 | Implement `InstitutionalOperatorSet` (`Δ_IS`) | Per DEF-IS-09: finite, named, versioned set of operators, mirroring `OperatorSet` (Phase 4) structure and its `apply_set` propagate-forward semantics. | 14.1 [tdd:required] | cc:TODO |
| 14.3 | ICU/HDU bed admit/discharge/transfer operator | Sound per DEF-IS-08 (refines-only, bounded by `α_IS(evt)`, physically valid per OBL-IS-01); ≥8 unit tests covering admit, discharge, transfer, and a capacity-exceeded case that must abstain rather than violate `cap`. | 14.2 [tdd:required] | cc:TODO |
| 14.4 | OR/infusion-chair slot allocation operator | Same soundness discipline as 14.3, applied to OR block hours / infusion-chair time-divisible slots. ≥8 unit tests. | 14.2 [tdd:required] | cc:TODO |
| 14.5 | Pharmacy inventory dispense/restock operator | Same soundness discipline, applied to formulary consumable classes (dispense decrements, restock increments, bounded by `cap`). ≥8 unit tests. | 14.2 [tdd:required] | cc:TODO |
| 14.6 | Blood-product allocation operator | Same soundness discipline, applied to blood-product units (a consumable resource with expiry — document expiry handling as out-of-scope per §0.5 if not modeled, or model it if trivial). ≥8 unit tests. | 14.2 [tdd:required] | cc:TODO |
| 14.7 | INV-IS-03 monotonicity property test + OBL-IS-04 discharge doc | Property test across all four operators (14.3–14.6): `δ_IS(c,e) = Refined(c') ⟹ c' ⊑_IS c`, ≥10 cases per operator. `clinlat/docs/obligations/obl-is-04-operator-set-soundness.md` discharging OBL-IS-04 at property-test tier, enumerating the four operators as evidence. | 14.3, 14.4, 14.5, 14.6 [tdd:required] | cc:TODO |

---

## Phase 15: Allocation abstention (M3.3)

**Goal:** Implement institution-specific abstention reasons and wire them through the operator-set application path with a bounded-steps guarantee. Mirrors the abstention half of Phase 8 (M2.2) but for the institutional side.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 15.1 | Define `AbstainReason_IS` enum | Per DEF-IS-10, all six variants: `CapacityExceeded`, `DemandUncertain`, `AllocationContested`, `EventOutOfScope`, `OperatorPreconditionUnmet`, `PhysicalValidityWouldBeViolated`. Every variant machine-classifiable (no free-text reason fields). Replaces the stub type used in 14.1. | 13.4, 14.1 | cc:TODO |
| 15.2 | Wire abstention into `InstitutionalOperatorSet::apply_set` | When no operator in `Δ_IS` licenses a refinement, emit the appropriate `AbstainReason_IS` variant (not a silent empty result). INV-IS-04 (institutional abstention is sound) property test mirroring INV-PS-04's discharge, ≥10 cases. | 14.2, 15.1 [tdd:required] | cc:TODO |
| 15.3 | DEF-IS-11 bounded-steps structural test | Structural test asserting every allocation request through `apply_set` yields `Refined(c')` or `Abstain(r)` in bounded steps — never silent failure, timeout, or crash-as-default. Adversarial cases: contested allocation, out-of-scope event, precondition unmet. ≥8 cases. | 15.2 [tdd:required] | cc:TODO |

---

## Phase 16: Institutional provenance ledger (M3.4)

**Goal:** Extend the patient-substrate provenance machinery (Phase 2) to the institutional side, including the auditability requirement that the ledger spans multiple patients (institution-wide), not a per-patient sequence.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 16.1 | Define `InstEvidence` provenance-carrying evidence type | Per DEF-IS-12: `InstEvidence = Evt^P` — mirrors `Evidence`/`Provenance` from Phase 2, every institutional event carries provenance. Replaces the stub type used in 14.1/15.1. | 14.1, 15.1 | cc:TODO |
| 16.2 | Define provenance-carrying operator output `Cap^P` | Per DEF-IS-13: `δ_IS : Cap^P × InstEvidence → Result⟨Cap^P, AbstainReason_IS⟩^P`. Update `InstitutionalOperatorSet::apply_set` signature accordingly. | 16.1, 14.2 [tdd:required] | cc:TODO |
| 16.3 | INV-IS-05 provenance closure test | Property test mirroring INV-PS-05's discharge: every `Cap^P` produced by an operator has a provenance chain closed over its input evidence, ≥10 cases. | 16.2 [tdd:required] | cc:TODO |
| 16.4 | OBL-IS-05 provenance auditability discharge doc | `clinlat/docs/obligations/obl-is-05-provenance-auditability.md`; documents that the institutional ledger records capacity decisions **spanning patients** (institution-wide, versus the per-patient sequence of Phase 2), per `NOTE.md` §4B.4; discharge tier: informal-argument + the 16.3 property tests as evidence. | 16.3 | cc:TODO |

---

## Phase 17: Capacity-learned proposer interface (M3.5)

**Goal:** Institutional analog of Phase 8/9 — a black-box proposer interface with input/output ontology gates, wired through the soundness gate, plus a stub demand-forecasting reference proposer. Mirrors M2's Phase 8 (interface) and Phase 9 (deterministic reference proposer) exactly, substituting `IS` for `PS`.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 17.1 | Define `CapacityProposer` trait signature | Per DEF-IS-14: `pub trait CapacityProposer { fn propose(&self, c: &Cap, e: &InstEvidence) -> Set<Cap>; }`. Doc anchor to SPEC.md §3.7; type signature enforces no decision-making. | 16.2 | cc:TODO |
| 17.2 | Define `ProposerConstraint_IS` validator | Per DEF-IS-15, three clauses: (1) resource-boundedness (DEF-IS-03), (2) physical validity under current `cap` (DEF-IS-04), (3) at-most-one-step refinement under `⊑_IS` (analogous to DEF-PS-15.2). Structured error per failed clause. | 17.1, 13.5 [tdd:required] | cc:TODO |
| 17.3 | Implement `propose_and_filter_is` / `propose_verify_is` adapters | Mirrors Phase 8's `propose_and_filter`/`propose_verify` (tasks 8.3/8.5): filters proposer output through `ProposerConstraint_IS`, then routes surviving candidates through `InstitutionalOperatorSet::apply_set` (the institutional `SV` node); emits `AbstainReason_IS::OperatorPreconditionUnmet` (or equivalent) when nothing is licensed, never silently. | 17.2, 15.2 [tdd:required] | cc:TODO |
| 17.4 | INV-IS-06 structural enforcement test | Per DEF-IS-06 pattern (Phase 8.6 analog): adversarial `CapacityProposer` returning out-of-resource-bounds or physically-invalid candidates is filtered to empty at every path out of `propose_and_filter_is`/`propose_verify_is`. Property tier ≥10 cases. | 17.2, 17.3 [tdd:required] | cc:TODO |
| 17.5 | Stub demand-forecasting reference proposer | `DemandForecastProposer`: a mock/stub implementation (canned or simple heuristic forecast — no real ML model required, matching `LlmProposer`'s offline-mock pattern from Phase 10) satisfying `CapacityProposer`, generating candidate bed/slot reallocations from a demand signal. | 17.1, 17.3 | cc:TODO |
| 17.6 | OBL-IS-06 discharge doc | `clinlat/docs/obligations/obl-is-06-proposer-operator-separation.md`; mirrors OBL-PS-05's discharge (Phase 11.1); enumerates the 17.4 property tests as evidence; states discharge tier. | 17.4, 17.5 | cc:TODO |

---

## Phase 18: Worked example and discharge-tier upgrade (M3.6)

**Goal:** One capacity-update operator (ICU bed admit/discharge, per `TODO.md` M3.6) taken to a worked example at informal-argument tier minimum, property-test tier as the target — mirroring M1's Phase 5→6 progression (operator, then discharge-tier upgrade).

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 18.1 | ICU bed admit/discharge worked example | Worked example (mirrors 9.3/10.4/11.3 shape) using the 14.3 operator on a concrete institutional scenario: patient admission request against a near-capacity ICU, showing `Refined`/`Abstain(CapacityExceeded)` paths. Informal-argument soundness doc `clinlat/docs/operators/icu_bed_soundness.md` (mirrors `sofa_resp_soundness.md`'s original informal-argument tier). | 14.7, 15.3 | cc:TODO |
| 18.2 | Property-test tier upgrade for ICU bed operator | Mirrors Phase 6 (M1.6): upgrade the 14.3 operator's discharge from informal-argument → property-test tier; ≥15 new property-test cases; refresh `icu_bed_soundness.md` documenting total test count. | 18.1 [tdd:required] | cc:TODO |

---

## Phase 19: Integration and release (M3.7)

**Goal:** Same shape as M1's Phase 7 and M2's Phase 12 — promote Phases 13–18 into a documented, version-bumped, publish-verified crate state ready for the `clinlat-v0.4.0` tag. `/harness-release` performs the actual tag/PR/GitHub-Release step once this phase is `cc:done`.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 19.1 | Write real `[0.4.0]` CHANGELOG entry | `clinlat/CHANGELOG.md` `## [0.4.0]` section in Keep-a-Changelog style, covering: institutional state space (DEF-IS-01/02), resource bounds (DEF-IS-03/04), Galois connection (DEF-IS-06), four capacity-update operators (DEF-IS-07/08/09), allocation abstention (DEF-IS-10/11), provenance (DEF-IS-12/13), capacity-learned proposer (DEF-IS-14/15), and the ICU bed worked example; links to `docs/obligations/` and `docs/operators/`. | 16.4, 17.6, 18.2 | cc:TODO |
| 19.2 | Update `clinlat/README.md` with M3 examples | Add institutional-substrate usage example (capacity-update operator + `propose_verify_is`) alongside existing M1/M2 examples; update crate status/milestone section to reflect M3; cross-links to the new obligation docs; doc tests pass. | 19.1 | cc:TODO |
| 19.3 | Bump `Cargo.toml` to `0.4.0`; verify CI matrix green | `clinlat/Cargo.toml` version = `0.4.0`; `cargo test`, `cargo doc --no-deps`, `cargo fmt --check`, `cargo clippy` (no warnings), `cargo check` all green. | 19.1, 19.2 | cc:TODO |
| 19.4 | Dry-run publish verification | `cargo publish --dry-run` succeeds without errors from within `clinlat/`; crate package contents verified ready for crates.io. | 19.3 | cc:TODO |

---

## Definition of Done for M3

- [ ] All ten M3-anchored SPEC.md elements reachable from running code: DEF-IS-01..15, INV-IS-01..06, OBL-IS-01..06
- [ ] Institutional substrate exists as a peer structure to the patient substrate per INV-IX-04 (substrate-local soundness independent of coupling) — no institutional definition silently depends on patient-substrate internals
- [ ] Four capacity-update operators implemented and OBL-IS-04-discharged: ICU/HDU bed, OR/infusion-chair slots, pharmacy inventory, blood-product allocation (Phase 14)
- [ ] Allocation abstention (`AbstainReason_IS`, 6 variants) wired through the operator-set application path with INV-IS-04 soundness and DEF-IS-11 bounded-steps guarantees (Phase 15)
- [ ] Institutional provenance ledger spans patients (institution-wide), OBL-IS-05 discharged (Phase 16)
- [ ] Capacity-learned proposer interface (Diagram 5 institutional-side `RP` slot) filled by ≥1 reference proposer (stub demand-forecaster), INV-IS-06 enforced structurally, OBL-IS-06 discharged (Phase 17)
- [ ] ICU bed operator taken to property-test tier via worked example (Phase 18)
- [ ] `clinlat-v0.4.0` released: CHANGELOG promoted, `Cargo.toml` bumped, CI matrix green, `cargo publish --dry-run` clean (Phase 19)

---

## Definition of Done for M2 (met — see `docs/archives/ARCHIVE-M2.md`)

✓ `clinlat-v0.3.0` released 2026-07-10: tagged (`dcd6226`), GitHub Release published, live on crates.io. Full DoD checklist archived.

---

## Backlog: Wells PE operator hardening (non-blocking, surfaced 2026-07-22)

**Context:** Found while writing `clinlat/docs/operators/wells_pe_soundness.md` (M1 Phase 5 operator, already shipped). None block M3; pick up whenever, or fold into a future M1-hardening pass.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| WP.1 | Resolve undocumented Wells PE abstention paths | `WellsPeOperator`'s doc comment (`wells_pe.rs`) claims abstention on "D-dimer unavailable" and "CTPA contraindicated," but `apply()` only implements the missing-gestalt abstention. Either (a) implement both as explicit preconditions with dedicated `AbstainReason` variants and tests, or (b) correct the doc comment to drop the unimplemented claims and update `wells_pe_soundness.md` Limitation 2 accordingly. | none | cc:TODO |
| WP.2 | Carry Wells PE sequential-testing next-step into operator output | `apply()` computes a next-step signal (`"consider-d-dimer"` / `"ctpa-indicated"`) and discards it (`_next_step`). Encode it in the output (second Atom, structured field, or provenance annotation) so NOTE.md §7E.3's sequential-testing recommendation is recoverable from the Evidence/Hyp chain, not only from the category-to-next-step convention documented in prose. Existing 8 unit tests stay green; ≥1 new test asserts the next-step value is present in the output. | none | cc:TODO |
| WP.3 | Resolve `PE-LIKELY` naming overlap (gestalt input vs. category output) | The `"PE-LIKELY"` observation code (mandatory gestalt input) and the `PE-LIKELY` output category label (score >4.0) share a name but denote different things. Either rename the output atom codes to unambiguous identifiers (e.g. `WELLS-PE-LOW-RISK`/`WELLS-PE-HIGH-RISK`, updating existing tests) or add an explicit doc-comment cross-reference at both call sites in `wells_pe.rs`; update `wells_pe_soundness.md` Limitation 3 to match whichever is chosen. | none | cc:TODO |
| WP.4 | Fix Wells PE test-comment arithmetic and name precision | `test_wells_pe_all_criteria`'s comment says the all-criteria sum is "12.0"; actual sum is 12.5 (3+3+1.5+1.5+1.5+1+1) — fix the comment. `test_wells_pe_unlikely_with_dvt_signs` asserts a `PE-LIKELY` outcome (score 4.5), not "unlikely" — rename or fix its comment per CLAUDE.md's "Test comment precision" discipline. `cargo test -p clinlat` green after the rename. | none | cc:TODO |

---

## Next session startup

**M3 task table generated 2026-07-22** (this session), scoped from `TODO.md`'s M3 entry against SPEC.md §3 (already fully formalized — no `Spec delta` needed). All 30 tasks across Phases 13–19 are `cc:TODO`; none started.

**New session command:**
```bash
claude
```

**First input:** `/harness-work 13.1` to start the dependency root (institutional-state poset), or `/harness-work all` to let the harness auto-select Breezing mode across the full M3 backlog. Phase 13 (state-space infrastructure) must complete before Phases 14–17 can begin — see each phase's `Depends` column.

---

## Notes on discipline

- **TDD adoption (M3):** Nearly all M3 implementation tasks are marked `[tdd:required]` — write failing tests first. Pure-doc tasks (13.6, 16.1, 16.4, 17.6, 19.1, 19.2) and worked-example tasks (18.1) are exempt by nature.
- **Critical gate (M3):** Task 13.1 (`Cap`/`⊑_IS` poset) is the dependency root for all of Phases 13–19; nothing else in M3 can start before it.
- **Structural asymmetry (M3):** Unlike the patient substrate, the institutional substrate is subject to hard physical capacity bounds (DEF-IS-04) with no patient-side analog — this surfaces as the `PhysicalValidityWouldBeViolated` abstention variant (15.1) and `OBL-IS-01` (physical-validity preservation), which have no `PS`-side counterpart.
- **Peer-substrate invariant:** Per `INV-IX-04`, the institutional substrate must remain sound independent of coupling to the patient substrate — M3 tasks should not introduce silent dependencies on `PS`-side internals beyond the shared `§1` abstractions (`Hyp`-poset machinery reused structurally, not by reference).
- **Prior milestones:** M1 discipline notes are in `docs/archives/ARCHIVE-M1.md`; M2 discipline notes are in `docs/archives/ARCHIVE-M2.md`.
