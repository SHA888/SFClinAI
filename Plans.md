# SFClinAI — Plans.md

**Project:** clinlat substrate kernel
**Current Milestone:** M2 — Constrained refinement proposer (✓ Complete, shipped 2026-07-10)
**Previous Milestone:** M1 (✓ Complete, shipped 2026-05-31)
**Created:** 2026-05-25
**Status:** M2 shipped. `clinlat-v0.3.0` tagged (`dcd6226`), GitHub Release published (https://github.com/SHA888/SFClinAI/releases/tag/clinlat-v0.3.0), and published to crates.io.
**Architectural Scope:** Complete NOTE.md §4A.5 / SPEC.md §2.7 / ARCHITECTURE.md Diagram 3 and 5 patient-substrate proposer slots.

---

## Phase 0: Architectural decisions and design docs (archived to docs/archives/ARCHIVE-M1.md)

**Status:** All 3 tasks complete (0.1–0.3) — D1/D2 decisions + M1 provenance spec SSOT.
**Archive:** See `docs/archives/ARCHIVE-M1.md` for full task table and commits.

---

## Phase 1: Ontology infrastructure (M1.1) (archived to docs/archives/ARCHIVE-M1.md)

**Status:** All 8 tasks complete (1.1–1.8) — `OntologyAdapter` trait + SNOMED/RxNorm/LOINC/ICD-11 adapters, `Atom` type replacing `&'static str`, INV-PS-01 closure proof.
**Archive:** See `docs/archives/ARCHIVE-M1.md` for full task table and commits.

---

## Phases 2–7: M1 implementation (archived to docs/archives/ARCHIVE-M1.md)

All M1 implementation phases are complete and shipped (clinlat v0.1.0 / v0.2.0, 2026-05-31). Full task tables and commits are in `docs/archives/ARCHIVE-M1.md`; bugfix detail in `docs/archives/ARCHIVE-6BF.md`.

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

## Phase 8: Proposer interface and soundness gate (M2.1 / M2.2 / M2.3)

**Goal:** Implement black-box proposer interface per DEF-PS-14 / DEF-PS-15 with input- and output-side ontology gates (M2.1); wire proposer output through the soundness-verification gate with abstention (M2.2); enforce codomain constraint INV-PS-06 by structural test (M2.3).

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 8.1 | Define `RefinementProposer` trait signature | `pub trait RefinementProposer { fn propose(&self, h: &Hyp, e: &Evidence) -> Set<Hyp>; }` per DEF-PS-14; doc anchors to SPEC.md §2.7; type signature enforces no decision-making (returns candidates only) | 7.4 | cc:done [102a8e9] |
| 8.2 | Define `ProposerConstraint` validator (input + output gates) | Validates two clauses per DEF-PS-15: (1) candidate must be ontology-bounded (output-side gate); (2) candidate must be at most one operator step from input. Also gates the input side: rejects evidence/hypotheses outside ontology bounds before proposal per M2.1 (Diagram 3 input-side gate). Returns structured error per failed clause for debugging. | 8.1 [tdd:required] | cc:done [9cfdfd3] |
| 8.3 | Implement `propose_and_filter` adapter | Wrapper that calls proposer and filters output through `ProposerConstraint`. Returns (valid_candidates, filtered_out_count, filter_errors). Logs filtering decisions for audit trail. | 8.2 [tdd:required] | cc:done [78c01f3] |
| 8.4 | Write INV-PS-06 proof (proposer cannot bypass soundness) | Informal-argument doc `clinlat/docs/invariants/inv-ps-06-proposer-safety.md`; proves that no proposer output can bypass `OperatorSet.apply_set()` gate; worked example: adversarial proposer vs. sound operator | 8.3 | cc:done [cb83a95] |
| 8.5 | Implement soundness-verification adapter with abstention (M2.2) | `propose_verify` adapter routes constraint-passing candidates through the soundness gate (`OperatorSet.apply_set()`, the Diagram 3 `SV` node); each surviving candidate must be licensed by ≥1 operator. When no candidate is licensed, emits `AbstainReason::NoOperatorLicenses` per DEF-PS-12/13 rather than returning an empty set silently. Audit trail records SV verdicts. | 8.3 [tdd:required] | cc:done [77c954c] |
| 8.6 | INV-PS-06 structural enforcement test | Dedicated structural test (not the 8.4 argument doc) asserting every path out of `propose_and_filter`/`propose_verify` yields only ontology-bounded candidates: adversarial proposers returning out-of-ontology Hyps are filtered to empty; property tier ≥10 cases over out-of-bounds candidate generators. | 8.2, 8.5 [tdd:required] | cc:done [e1e62a8] |

---

## Phase 9: Reference proposer #1 — Deterministic search (M2.4)

**Goal:** Implement exhaustive lattice-search proposer; trivially sound by construction (every candidate is demonstrably reachable by an operator).

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 9.1 | Implement `LatticeSearchProposer` | Exhaustive breadth-first search of all hypotheses reachable from input via single operator application. Returns all valid candidates per DEF-PS-15. For small operator sets (≤5 operators), search terminates quickly; for larger sets, implement pruning heuristics (e.g., halt at depth N or candidate count threshold). | 8.1, 4.2 [tdd:required] | cc:done [7bfdd94] |
| 9.2 | Property-test `LatticeSearchProposer` completeness | Verify: (1) every hypothesis reachable by one operator application is in the output set; (2) output set is minimal (no spurious candidates); (3) monotonicity of refinement within result set. ≥10 property cases per (1), (2), (3). | 9.1 [tdd:required] | cc:done [0ca7ba8] |
| 9.2-fix | Code review fixes: refactor property-tier tests | Medium-effort code review identified 6 findings (4 CONFIRMED, 2 PLAUSIBLE): (1) P30 doesn't test identity operator self-loop claim; (2) P23/P27 duplicate (0-refining case); (3) 7 Completeness tests duplicate foundation tier; (4) all 10 Monotonicity tests trivial on Hyp::unknown(); (5) P35 allows Equal but untested; (6) P38 O(n²) loop. Fix: remove 9 duplicates, add 6 new tests for actual gaps (non-unknown input, identity ops), rewrite Monotonicity, fix loop structures. Result: ~27 distinct property cases with genuine coverage. | 9.2 [tdd:required] | cc:done [989883b] |
| 9.3 | Worked example: SOFA + KDIGO proposer | Use `LatticeSearchProposer` with {SofaRespOperator, KdigoAkiOperator} on a sepsis-3 patient state. Show how lattice search generates candidate refinements (SOFA stage 2 + KDIGO Stage 1, etc.). | 9.2 | cc:done [6bf6e96] |

---

## Phase 10: Reference proposer #2 — LLM-class adapter (M2.5)

**Goal:** Wrapper around a foundation-model API call with input/output ontology gates. Demonstrates substrate-first safety: LLM can hallucinate, but system remains sound.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 10.1 | Define `LlmProposerConfig` struct | Configuration struct holding: LLM endpoint (OpenAI / Anthropic / local), model name, prompt template, max tokens, temperature, seed for reproducibility. Supports offline mock mode (returns fixed canned responses) for CI/testing. | 8.1 | cc:done [00aa6b6] |
| 10.2 | Implement `LlmProposer` adapter | Wrapper that (1) constructs a prompt from current hypothesis + evidence, (2) calls LLM API, (3) parses response into candidate hypotheses, (4) runs through `ProposerConstraint` filter. Failed parses or out-of-constraint responses logged as "LLM hallucinations"; ontology gate filters them silently. | 10.1, 8.3 [tdd:required] | cc:done [aa1ad40] |
| 10.3 | Property-test `LlmProposer` safety invariant | Verify: (1) every LLM-generated candidate that passes `ProposerConstraint` is usable by an operator (safety); (2) LLM can hallucinate freely and the system still refines correctly (robustness); (3) audit trail records LLM responses and filtering decisions. Use mock LLM responses (canned hallucinations, valid-but-non-obvious candidates). | 10.2 [tdd:required] | cc:done [56abd18] |
| 10.4 | Worked example: Sepsis-3 with LLM proposer | Use `LlmProposer` with mock LLM (pre-recorded responses including hallucinations) on sepsis-3 patient state. Show: (1) LLM suggests a non-existent SOFA band (hallucination filtered), (2) LLM suggests clinically valid KDIGO Stage that passes constraint, (3) substrate refines correctly regardless of LLM behavior. | 10.3 | cc:done [cd25fba] |

---

## Phase 11: OBL-PS-05 discharge and substrate-invariance demonstration (M2.6)

**Goal:** Discharge the proposer-constraint obligation OBL-PS-05 at property-test tier across both reference proposers, and demonstrate the substrate-first claim empirically: identical substrate behavior under a proposer swap for the same evidence.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 11.1 | OBL-PS-05 discharge doc | `clinlat/docs/obligations/obl-ps-05-proposer-constraint.md`; discharges OBL-PS-05 at property-test tier; enumerates the property tests from 9.2 and 10.3 as the discharge evidence; states tier (property-test) and residual informal-argument gaps; links DEF-PS-14/15, INV-PS-06 | 9.2, 10.3 | cc:done [190cbe5] |
| 11.2 | Substrate-invariance test (proposer swap) | Property/integration test feeding the **same** evidence + hypothesis through `LatticeSearchProposer` and `LlmProposer` (mock, returning a superset incl. hallucinations); assert the post-soundness-gate refinement applied by the substrate is **identical** across the swap (substrate behavior independent of proposer architecture per NOTE.md §3, §5). ≥10 paired cases. | 8.5, 9.1, 10.2 [tdd:required] | cc:done [bad9c35] |
| 11.3 | Worked example: substrate-first claim | Side-by-side worked example (sepsis-3 state) showing both proposers yield the same substrate outcome despite divergent candidate sets; documents the empirical demonstration referenced in the M2 DoD. | 11.2 | cc:done [d033171] |

---

## Phase 12: Integration and release (M2.7)

**Goal:** Same shape as M1's Phase 7 — promote the M2 implementation (Phases 8–11) into a documented, version-bumped, publish-verified crate state ready for the `clinlat-v0.3.0` tag. `/harness-release` performs the actual tag/PR/GitHub-Release step once this phase is `cc:done`; this phase only prepares the artifact.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 12.0 | Fix pre-existing `prop_abstraction_completeness` test bug (blocks 12.3's "cargo test green" DoD) | `operator.rs`'s `prop_abstraction_completeness` asserted `hyp.atoms().len() == codes.len()`, but `Hyp::new` deduplicates atoms by full equality (poset invariant); duplicate generated observation codes (e.g. `["ICD11:498","ICD11:498"]`) parse to identical atoms that collapse to one, so the naive count assertion is false whenever the proptest strategy generates a duplicate. Fixed by comparing against the distinct-code count instead. Unrelated to M2 scope — pre-existing, confirmed via `git stash` isolation 2026-07-03 (see harness-mem M2 status note). All 299 tests pass after fix. | none | cc:done [1c68ecf] |
| 12.1 | Write real `[0.3.0]` CHANGELOG entry | Replace `clinlat/CHANGELOG.md`'s `[Unreleased] → Planned for 0.3.0` bullets with a dated `## [0.3.0]` section in Keep-a-Changelog `### Added` style (matching the `[0.2.0]` entry's structure), covering: `RefinementProposer` trait (DEF-PS-14), `ProposerConstraint` gates (DEF-PS-15), `propose_and_filter`/`propose_verify` adapters, INV-PS-06 structural enforcement, `LatticeSearchProposer`, `LlmProposer`, OBL-PS-05 discharge, and the three worked examples (9.3, 10.4, 11.3); links to the relevant `docs/obligations/` and `docs/invariants/` files | 11.1, 11.2, 11.3 | cc:done [9f05aae] |
| 12.2 | Update `clinlat/README.md` with M2 examples | Add a proposer usage example (`LatticeSearchProposer` and/or `LlmProposer` against `propose_verify`) alongside the existing M1 operator examples; update crate status/milestone section to reflect M2; cross-links to `docs/obligations/obl-ps-05-proposer-constraint.md` and `docs/invariants/inv-ps-06-proposer-safety.md`; doc tests pass | 12.1 | cc:done [edd1aca] |
| 12.3 | Bump `Cargo.toml` to `0.3.0`; verify CI matrix green | `clinlat/Cargo.toml` version = `0.3.0`; `cargo test`, `cargo doc --no-deps`, `cargo fmt --check`, `cargo clippy` (no warnings), `cargo check` all green | 12.1, 12.2 | cc:done [f004918] |
| 12.4 | Dry-run publish verification | `cargo publish --dry-run` succeeds without errors from within `clinlat/`; crate package contents verified ready for crates.io | 12.3 | cc:done [verified] |

---

## Definition of Done for M2

✓ All M2-anchored SPEC.md elements (DEF-PS-14/15, INV-PS-06, OBL-PS-05) reachable from running code
✓ Diagram 3 boundary contract realized end-to-end: input gate → proposer → output gate → soundness-verification (`SV`) node → abstention path (8.1–8.6)
✓ Diagram 5 patient-side proposer slot `RP` filled by ≥2 architectures: deterministic lattice search (Phase 9) and LLM-class adapter (Phase 10)
✓ INV-PS-06 enforced by structural test (8.6), not argument alone
✓ OBL-PS-05 discharged at property-test tier across both reference proposers (11.1)
✓ Substrate behavior identical across proposer swap for the same evidence — substrate-first claim demonstrated empirically (11.2, 11.3)
✓ `clinlat-v0.3.0` released: CHANGELOG promoted, `Cargo.toml` bumped, CI matrix green (299 tests, fmt, clippy, doc), `cargo publish --dry-run` clean (Phase 12) — then tagged (`dcd6226`), GitHub Release published, and published to crates.io (2026-07-10)

---

## Next session startup

**M2 is fully shipped** (`clinlat-v0.3.0`: tagged, GitHub Release published, live on crates.io as of 2026-07-10). There is no open task in this file — Phases 8–12 are all `cc:done`.

**New session command:**
```bash
claude
```

**First input:** none required yet. The next unit of work is scoping **M3 — Institutional substrate kernel** (`clinlat-v0.4.0` per `TODO.md`), which has no `Plans.md` task table yet. Start a new session with something like `/harness-plan M3` or discuss scope first — NOTE.md §4B (institutional-state substrate) and SPEC.md §3 are the source material, mirroring how M2 mirrored §4A/§2.

---

## Notes on discipline

- **TDD adoption:** All M2 implementation tasks (8.2, 8.3, 8.5, 8.6, 9.1, 9.2, 10.2, 10.3, 11.2) are marked `[tdd:required]`. Write failing tests first.
- **Soundness discharge:** Tasks 8.4 (INV-PS-06 proof) and 11.1 (OBL-PS-05 discharge) are pure documentation; the three worked examples (9.3, 10.4, 11.3) are demonstrative. These constitute the informal-argument and property-test tier discharges per SPEC.md §6.
- **Critical gate:** Task 8.1 (`RefinementProposer` trait) must complete before any other M2 phase; it is the dependency root for both reference proposers and the soundness-verification adapter.
