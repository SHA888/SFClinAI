# SFClinAI — TODO (end-to-end milestone roadmap)

**Goal:** realize the architecture specified in `ARCHITECTURE.md` v0.2.0-draft / formalized in `SPEC.md` v0.3.0-draft / argued in `NOTE.md` v0.12.0-draft as a working `clinlat` Rust system covering all eighteen load-bearing principles.

**Status:** `clinlat-v0.1.0` ships partial 4A patient-substrate kernel with one operator (SOFA-3 respiratory). The remaining seventeen principles, the institutional substrate, the interaction layer, the temporal-evolution lifecycle, and the cross-cutting concerns are scoped below as sequenced milestones.

**Per-milestone task tracking lives in `Plans.md`.** This file is the milestone-level roadmap only.

---

## Completed: `clinlat-v0.1.0` (2026-05-25, archived)

- ✓ Rust crate at `clinlat/` with `Hyp` poset (DEF-PS-01), `Outcome⟨H, A⟩` sum (DEF-MP-13), `AbstainReason` (DEF-PS-10), `Evidence` stub, `Operator` trait (DEF-PS-07).
- ✓ One operator: Sepsis-3 SOFA respiratory-component (PaO₂/FiO₂), discharged at informal-argument tier per OBL-PS-03 in `clinlat/docs/operators/sofa_resp_soundness.md`.
- ✓ `cargo test`, `cargo doc --no-deps`, `cargo publish --dry-run` green; CI matrix (fmt, clippy, test, doc, semver, MSRV) green.
- ✓ Dual licensing (MIT OR Apache-2.0 for code; CC BY 4.0 for prose); published to crates.io.

Per-task detail with commit hashes: see `Plans.md`. Architectural coverage at v0.1.0: 4A.1–4A.5 partial (single-operator state; provenance is `()` stub; no proposer; no ontology binding).

---

## ✓ Completed: Milestone M1 — Patient substrate completion (`clinlat-v0.2.0`)

**Architectural scope:** complete `NOTE.md` §4A / `SPEC.md` §2 / `ARCHITECTURE.md` Diagrams 1–3 patient-substrate side.

**Status:** ✓ Shipped 2026-05-31. All tasks complete; 193 tests passing; four operators discharged; all formal definitions reachable from code.

- ✓ **M1.1 Real ontology binding** — replaced `&'static str` AtomId with `Atom` struct containing adapters for SNOMED CT, RxNorm, LOINC, ICD-11 (DEF-PS-03, DEF-PS-04, INV-PS-01, OBL-PS-01). Diagram 1's `OB` node is runnable. *(Phase 1, commit 97b0304)*
- ✓ **M1.2 Real provenance carrier** — replaced `()` stub with typed `Provenance` carrier supporting DEF-MP-14, DEF-PS-12, DEF-PS-13, INV-PS-05, OBL-PS-04. JSON serialization with optional gzip compression (encoding: JSON per DESIGN-D1). *(Phase 2, commit b2246f8)*
- ✓ **M1.3 Galois connection** — implemented `abstract_evidence` (α_PS), `is_consistent_with` (γ_PS) per DEF-PS-05/06; discharged OBL-PS-02 (adjunction laws) at property-test tier with 10+ property cases. *(Phase 3, commit 50a6d5a)*
- ✓ **M1.4 Operator-set type `Δ_PS`** — formalized `OperatorSet` collection per DEF-PS-09; soundness obligation OBL-PS-03 extended across set with propagate-forward semantics. *(Phase 4, commit 9793260)*
- ✓ **M1.5 Three additional operators** matching `NOTE.md` §7E worked examples, each with informal-argument discharge; 9 code-review bugs fixed:
  - KDIGO AKI staging (§7E.2) — 9 tests, 6 bugs fixed *(Phase 5 + 5-BF, commit 80e0f8f)*
  - Wells score for PE with sequential testing (§7E.3) — 8 tests, 1 bug fixed *(Phase 5 + 5-BF, commit 9261d66)*
  - CURB-65 for CAP disposition (§7E.4) — 10 tests, 2 bugs fixed *(Phase 5 + 5-BF, commit 70ca6d1)*
- ✓ **M1.6 Discharge-tier upgrade for SOFA-respiratory** — upgraded from informal-argument → property-test tier; 17 new property-test cases + refreshed `sofa_resp_soundness.md` documenting 46 total tests (29 unit + 17 property). *(Phase 6, commit 5a6e322)*
- ✓ **M1.7 Phase 7 Integration and release** — README updated with M1 examples; soundness documents created for all three Phase 5 operators; full bidirectional traceability NOTE.md ↔ SPEC.md verified; `cargo test` (193 passing), `cargo doc`, `cargo publish --dry-run` all green. *(Phase 7, commits 21f2e83, 17a6b48)*

**Definition of Done — ✓ All criteria met:**
- ✓ All eleven 4A-anchored SPEC.md elements (DEF-PS-01..15, INV-PS-01..06, OBL-PS-01..05) reachable from running code
- ✓ Four operators discharged: SOFA-resp at property-test tier (46 tests); KDIGO, Wells, CURB-65 at informal-argument tier (9+8+10 tests)
- ✓ `clinlat` crate compiles, 193 tests pass, docs render; cargo publish --dry-run succeeds
- ✓ Ontology adapters (SNOMED, RxNorm, LOINC, ICD-11) integrated; Atom replaces `&'static str` throughout
- ✓ Provenance typed per OBL-PS-04; audit-trail fidelity demonstrated; derivation chain version-respecting (INV-PS-05)
- ✓ Galois connection (α_PS, γ_PS) property-tested; adjunction laws hold
- ✓ Operator-set type and composition formalized; propagate-forward abstention semantics working
- ✓ Bidirectional traceability: all 18 NOTE.md principles mapped to SPEC.md formalizations in §8

---

## ✓ Completed: Milestone M2 — Constrained refinement proposer (`clinlat-v0.3.0`)

**Architectural scope:** `NOTE.md` §4A.5 / `SPEC.md` §2.7 / `ARCHITECTURE.md` Diagrams 3 and 5 patient-substrate slots.

**Status:** ✓ Shipped 2026-07-10. All `Plans.md` tasks (8.1–12.4) complete, including release prep (Phase 12) and the release itself: tagged `clinlat-v0.3.0` (commit `dcd6226`), [GitHub Release published](https://github.com/SHA888/SFClinAI/releases/tag/clinlat-v0.3.0), and live on crates.io.

- ✓ **M2.1 Black-box proposer interface** — `RefinementProposer` trait per DEF-PS-14 (task 8.1); `ProposerConstraint` input/output ontology gates per DEF-PS-15 (task 8.2); `propose_and_filter` adapter (task 8.3). *(Phase 8, commits 102a8e9, 9cfdfd3, 78c01f3)*
- ✓ **M2.2 Soundness verification adapter** — `propose_verify` routes constraint-passing candidates through `OperatorSet.apply_set()` (Diagram 3 `SV` node); emits `AbstainReason::NoOperatorLicenses` when nothing is licensed (task 8.5). *(Phase 8, commit 77c954c)*
- ✓ **M2.3 INV-PS-06 enforcement** — informal-argument proof doc (task 8.4) plus dedicated structural test with ≥10 property cases over adversarial out-of-ontology proposers (task 8.6). *(Phase 8, commits cb83a95, e1e62a8)*
- ✓ **M2.4 Reference proposer #1: deterministic search** — `LatticeSearchProposer` exhaustive BFS (task 9.1); completeness/minimality/monotonicity property tests, refined after code review (tasks 9.2, 9.2-fix); SOFA+KDIGO sepsis-3 worked example (task 9.3). *(Phase 9, commits 7bfdd94, 0ca7ba8, 989883b, 6bf6e96)*
- ✓ **M2.5 Reference proposer #2: LLM-class adapter** — `LlmProposerConfig` with offline mock mode (task 10.1); `LlmProposer` adapter wrapping prompt→LLM→parse→`ProposerConstraint` filter (task 10.2); safety/robustness property tests over hallucinated and valid mock responses (task 10.3); sepsis-3 LLM worked example (task 10.4). *(Phase 10, commits 00aa6b6, aa1ad40, 56abd18, cd25fba)*
- ✓ **M2.6 OBL-PS-05 discharge** — discharge doc at property-test tier across both proposers (task 11.1); substrate-invariance test proving identical post-soundness-gate refinement across a proposer swap, ≥10 paired cases (task 11.2); side-by-side sepsis-3 worked example demonstrating the substrate-first claim (task 11.3). *(Phase 11, commits 190cbe5, bad9c35, d033171, 56cdacb)*

**Definition of Done — ✓ All criteria met:**
- ✓ Diagram 3 boundary contract realized end-to-end: input gate → proposer → output gate → soundness-verification (`SV`) node → abstention path
- ✓ Diagram 5 patient-side proposer slot `RP` filled by two architectures: deterministic lattice search and LLM-class adapter
- ✓ INV-PS-06 enforced by structural test, not argument alone
- ✓ OBL-PS-05 discharged at property-test tier across both reference proposers
- ✓ Substrate behavior identical across proposer swap for the same evidence — substrate-first claim demonstrated empirically

**Release commits:** `1c68ecf` (pre-existing test-bug fix, unrelated to M2 scope), `9f05aae` (CHANGELOG), `edd1aca` (README), `f004918` (version bump + CI matrix), `dcd6226` (release-prep complete). Tag `clinlat-v0.3.0` points at `dcd6226`.

---

## Milestone M3 — Institutional substrate kernel (`clinlat-v0.4.0`)

**Architectural scope:** `NOTE.md` §4B / `SPEC.md` §3 / `ARCHITECTURE.md` Diagrams 1–3 institutional-substrate side.

- [ ] **M3.1 Institutional state types** — `Cap^P` capacity hypothesis space, `InstEvidence` evidence space, physical capacity bounds (DEF-IS-01..06, INV-IS-01/02/03, OBL-IS-03).
- [ ] **M3.2 Capacity-update operators** — `Δ_IS` operator set covering: ICU/HDU bed admit/discharge/transfer, OR/infusion-chair slots, pharmacy inventory dispense/restock, blood-product allocation (DEF-IS-07/08/09, OBL-IS-01/02/04).
- [ ] **M3.3 Allocation abstention** — `AbstainReason_IS` with the institutional-specific variants; INV-IS-04 bounded-steps guarantee per `NOTE.md` §4B.3 (DEF-IS-10/11).
- [ ] **M3.4 Institutional provenance ledger** — provenance for capacity decisions spanning patients, not just patient-local sequences (DEF-IS-12/13, INV-IS-05, OBL-IS-05).
- [ ] **M3.5 Capacity-learned proposer interface** — DEF-IS-14/15, INV-IS-06, OBL-IS-06; one reference proposer (e.g., a stub demand-forecasting model).
- [ ] **M3.6 One capacity-update operator with discharge** — ICU bed admit/discharge as the worked example; informal-argument tier minimum, property-test tier target.

**Definition of Done:** all ten 4B-anchored SPEC.md elements (DEF-IS-01..15, INV-IS-01..06, OBL-IS-01..06) reachable from running code; institutional substrate exists as a peer of the patient substrate per `INV-IX-04` (substrate-local soundness independent of coupling).

---

## Milestone M4 — Interaction layer (`clinlat-v0.5.0`)

**Architectural scope:** `NOTE.md` §4C / `SPEC.md` §4 / `ARCHITECTURE.md` Diagrams 1 and 2 interaction-layer nodes.

- [ ] **M4.1 Cross-layer event bus** — typed `CrossLayerEvent` per DEF-IX-01 with `PatientToInstitutional`, `InstitutionalToPatient`, `Coupled` variants; `derive_alloc`/`derive_patient` derivations (DEF-IX-02, INV-IX-01, OBL-IX-01).
- [ ] **M4.2 Joint operator decomposition** — joint operators as structured `(δ_PS', δ_IS', coupling_check)` triples per DEF-IX-05; not opaque functions.
- [ ] **M4.3 Joint licensing gate** — three-condition licensing (patient-licensed ∧ institutionally-licensed ∧ coupling-check-true) per DEF-IX-06; INV-IX-02 monotonicity; OBL-IX-02 coupling-check soundness.
- [ ] **M4.4 Joint abstention with structured diff** — `AbstainReason_J` with `Divergent(SubstrateDiff)` carrying the unconstrained-optimal hypothesis and the institutionally-actionable alternative side by side (DEF-IX-08/09/10, INV-IX-03).
- [ ] **M4.5 No-silent-downgrade obligation discharge** — OBL-IX-03 (load-bearing safety property of §4): no code path through any joint operator can silently substitute the institutionally-feasible refinement for the patient-locally-optimal one. Property-test tier minimum, mechanized target.
- [ ] **M4.6 Worked example: Sepsis-3 in capacity-constrained tertiary care** — `NOTE.md` §7E.1 end-to-end demonstration through the joint-licensing pipeline.

**Definition of Done:** Diagram 2 data-and-event-flow fully runnable from clinical input to joint output (recommendation pair with diff, or one of three abstention outputs); §4C three principles all formalized in code and exercised by §7E.1.

---

## Milestone M5 — Temporal evolution (`clinlat-v0.6.0`)

**Architectural scope:** `NOTE.md` §4D / `SPEC.md` §5 / `ARCHITECTURE.md` Diagram 4 (now including institutional symmetric re-review path per DEF-TE-06b, new in v0.3.0).

- [ ] **M5.1 Version registry** — every operator, ontology, capacity bound, coupling check, and derivation carries `Ver` per DEF-TE-01; INV-TE-01 version-closure enforced in provenance.
- [ ] **M5.2 Currency monitor** — DEF-TE-02 currency carrier; DEF-TE-03 active/advisory/inactive lifecycle; INV-TE-02 staleness threshold enforcement.
- [ ] **M5.3 Sound-evolution checker** — DEF-TE-04 transition discipline; DEF-TE-05 refines/generalizes/incomparable categorization with the incomparable-change justification burden per `NOTE.md` §4D.3 ¶2; OBL-TE-01 transition justifications stored.
- [ ] **M5.4 Patient re-review event system** — DEF-TE-06 `ReReviewEvent` with `ResolvedKeep`/`ResolvedReplace` lifecycle; clinician resolution required; INV-TE-04 no-automatic-replacement unconditional on transition type.
- [ ] **M5.5 Institutional re-review event system** — DEF-TE-06b `InstReReviewEvent` with `authority_class` routing to capacity manager / ethics committee / formulary committee; structurally identical lifecycle to M5.4. **New in ARCHITECTURE v0.2.0** (Diagram 4 symmetric path).
- [ ] **M5.6 No-silent-drift obligation discharge** — OBL-TE-02 applies symmetrically to `Δ_PS` and `Δ_IS`; property-test tier minimum.
- [ ] **M5.7 Evolution-aware provenance** — DEF-TE-07 with version-respecting `derives_from`; OBL-TE-03 audit queries answerable from provenance alone ("which active hypotheses derived under operator-version V?"); INV-TE-05.
- [ ] **M5.8 Worked example: SSC 2021 → SSC 2026 guideline transition** — `NOTE.md` §7E.6 end-to-end demonstration on the patient side; mirror demonstration on the institutional side using a queue-priority policy revision.

**Definition of Done:** Diagram 4 lifecycle runnable end-to-end on both patient and institutional sides; OQ-TE open questions either closed or moved to v1.x; the §7E.6 worked example produces re-review events that resolve through the substrate-mandated paths and not silently.

---

## Milestone M6 — Cross-cutting concerns (`clinlat-v0.7.0`)

**Architectural scope:** ARCHITECTURE.md Diagram 1's `CC_CONCERNS` subgraph (audit infrastructure, ontology binding, clinician interface, capacity-management interface, evaluation harness, regulatory artifact production).

- [ ] **M6.1 Unified audit infrastructure** — merged audit trail spanning patient provenance, institutional provenance, abstention outputs, re-review resolutions; queryable per OBL-TE-03 patterns.
- [ ] **M6.2 Clinician interface** — review/override capture, diff display, joint-abstention escalation UI. Substrate-side; UI semantics deliberately left informal in SPEC.md §0.5.
- [ ] **M6.3 Capacity-management interface** — administrator-facing surface for allocation abstention escalation, surge protocol activation, supply procurement triggers.
- [ ] **M6.4 Evaluation harness** — patient + institutional + joint metrics; calibration on the abstention outputs (the most safety-relevant signal).
- [ ] **M6.5 Regulatory artifact production** — FDA PCCP submission templates, EU AI Act Article 72 post-market monitoring report templates, both fed by evolution-aware provenance per OBL-TE-03.

**Definition of Done:** all five ARCHITECTURE.md diagrams realized in deployable shape; one institution-shaped demo (single pilot site simulator) runnable end-to-end.

---

## Milestone M7 — Discharge tier upgrade and pilot readiness (`clinlat-v1.0.0`)

**Architectural scope:** `SPEC.md` §6 (all seventeen OBL-\* obligations); `NOTE.md` §8 (deployment readiness); pilot prep.

- [ ] **M7.1 Property-test tier discharge for all P-criticality obligations** — five P-tier obligations (OBL-PS-02 if reclassified, OBL-IX-02, OBL-IX-03, OBL-TE-02, plus institutional symmetry).
- [ ] **M7.2 Mechanization spike for OBL-IX-03 in Lean 4** — load-bearing no-silent-downgrade obligation; per `SPEC.md` §9.3 Tier B → Tier C migration target.
- [ ] **M7.3 Property-test tier discharge for all S-criticality obligations** — ten S-tier obligations.
- [ ] **M7.4 Close remaining open questions** — OQ-X-01 (F-asymmetry on 4D.1/4D.2), OQ-X-03 (informal-argument discharge mechanism), and any new OQs surfaced through M1–M6.
- [ ] **M7.5 Clinical validation protocol document** — bridges from `clinlat` to a pilot study design; addresses the bottlenecks `NOTE.md` §8 names (regulatory engagement, clinical pilot validation, institutional adoption).
- [ ] **M7.6 v1.0 SemVer stability commitment** — public API of `clinlat` frozen; breaking changes from this point follow strict SemVer with `cargo-semver-checks` CI enforcement (already in place since v0.1.0).

**Definition of Done:** the eighteen-principle claim of `NOTE.md` §3 is demonstrably realized in running code; obligations discharged at property-test tier minimum across all criticality classes; at least one P-tier obligation mechanized; ready to engage a pilot site.

---

## Open architectural decisions (cross-milestone)

These need a `DESIGN.md` doc before the milestone that depends on them — flagged here so the dependency is visible:

- **D1 Provenance carrier encoding** (blocks M1.2) — CBOR / JSON / Merkle DAG / hybrid? SPEC.md §0.5 deliberately left opaque.
- **D2 Ontology adapter design** (blocks M1.1) — caching strategy, version pinning, offline mode for institutions without network access.
- **D3 Cross-layer event bus topology** (blocks M4.1) — in-process async vs. message queue (NATS / Kafka) vs. embedded SQLite event log.
- **D4 Joint-operator authoring model** (blocks M4.2) — derive macro vs. explicit constructor vs. config-driven assembly.
- **D5 Re-review event persistence and authority resolution** (blocks M5.4, M5.5) — how the substrate identifies "the responsible clinician" or routes to the right `authority_class`.
- **D6 Mechanization target** (blocks M7.2) — Lean 4 vs. Agda vs. property-based testing (proptest/Bolero) vs. runtime contracts. Per SPEC.md §9.3, Lean 4 is the working assumption but not yet committed.

---

## Notes on discipline

- This file is the **milestone-level** roadmap. Per-task tracking with DoD/Depends/Status lives in `Plans.md`, regenerated per milestone.
- Milestone numbering aligns with `clinlat-v0.X.0` SemVer minor bumps until v1.0.0. PATCH releases inside a milestone are bug fixes, not new scope.
- Architectural coverage is the gating criterion: a milestone ships when the SPEC.md elements it claims are reachable from running code and the OBL-\* obligations it claims are discharged at the stated tier.
- This file is not the source of truth for vision (`NOTE.md`), formalization (`SPEC.md`), or architecture (`ARCHITECTURE.md`). It is the source of truth only for: "what milestones, in what order, get from `clinlat-v0.1.0` to the full ARCHITECTURE.md vision realized in code."
- Horizon estimate per milestone: 2–6 months at a single-author cadence. Total v0.1.0 → v1.0.0: 18–36 months. Consistent with `NOTE.md` §7's 2–4 year horizon for narrow applications.
