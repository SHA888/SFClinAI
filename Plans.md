# SFClinAI — Plans.md

**Project:** clinlat substrate kernel v0.1.0  
**Created:** 2026-05-25  
**Scope:** Rust implementation; out-of-scope items tracked in NOTE.md §7 and SPEC.md §7.

---

## Phase 0: Repo prerequisites

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 0.1 | Add `clinlat/` directory to repo root | Directory exists, `.gitignore` updated | - | cc:完了 [15f5230] |
| 0.2 | Initialize Rust crate with correct metadata | `Cargo.toml` has edition=2024, MSRV=1.86.0, license="MIT OR Apache-2.0", repository and documentation fields | 0.1 | cc:完了 [0f2136c] |
| 0.3 | Add dual license files (MIT, Apache-2.0) at repo root | `LICENSE-MIT` and `LICENSE-APACHE` present and verbatim; existing `LICENSE` clarified for prose vs code | 0.2 | cc:完了 [478cad4] |
| 0.4 | Update README.md License section for code dual-licensing | README License section reflects "MIT OR Apache-2.0" for code | 0.3 | cc:完了 [92e80ac] |
| 0.5 | Add CONTRIBUTING.md stub with cargo-skill docs | CONTRIBUTING.md exists, mentions `cargo install cargo-skill`, links to tooling docs | 0.2 | cc:完了 [8e86d36] |
| 0.6 | CI scaffolding: fmt, clippy, test, doc, semver, MSRV checks | `.github/workflows/ci.yml` runs all 6 checks; all checks pass on fresh clone | 0.2 | cc:完了 [5898727] |
| 0.7 | rust-toolchain.toml and .gitignore for Rust | rust-toolchain.toml pins 1.86.0; .gitignore covers target/, Cargo.lock, IDE artifacts | 0.2 | cc:TODO |

---

## Phase 1: Core type scaffolding

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 1.1 | Define `Hyp` struct with PartialOrd, compat, meet | `Hyp` implements refinement order per DEF-PS-01, compatibility per DEF-PS-01, meet per INV-PS-02; doc comment names v0.1.0 simplification (AtomId is `&'static str`) | 0.7 | cc:TODO |
| 1.2 | Define `AtomId` placeholder type | `pub type AtomId = &'static str` with doc naming ontology-binding deferral | 1.1 | cc:TODO |
| 1.3 | Implement `Outcome<H, A>` operator result type | `Outcome` enum with `Refined(H)` and `Abstain(A)` variants; `map` and `and_then` methods; reasoning: avoids Rust `Result` collision; aligns with SPEC.md OQ-MP-02 | 0.7 | cc:TODO |
| 1.4 | Define `AbstainReason` enum with five variants | `AbstainReason` carries five variants from DEF-PS-10 (`InsufficientEvidence`, `OutOfDistribution`, `AmbiguousRefinement`, `OperatorPreconditionUnmet`, `OntologyOutOfScope`), each with `&'static str` payload | 1.3 | cc:TODO |
| 1.5 | Define `Evidence` stub type | `Evidence` struct typed observation packet; provenance carrier is `()` stub (real provenance deferred to v0.2) | 1.4 | cc:TODO |
| 1.6 | Define `Operator` trait signature | `pub trait Operator { fn apply(&self, h: &Hyp, e: &Evidence) -> Outcome<Hyp, AbstainReason>; }` matching DEF-PS-07 | 1.5 | cc:TODO |

---

## Phase 2: SOFA respiratory operator

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 2.1 | Encode SOFA respiratory thresholds (PaO₂/FiO₂ ratios) | Thresholds for SOFA scores 0–4 per Sepsis-3 (≥400, 300–399, 200–299, 100–199, <100) hardcoded and documented | 1.6 | cc:TODO |
| 2.2 | Define `SofaRespEvidence` type | `SofaRespEvidence` struct with fields `pao2: f64`, `fio2: f64`, `on_mech_vent: bool` per Sepsis-3 requirement | 2.1 | cc:TODO |
| 2.3 | Define SOFA respiratory hypothesis variants | `Hyp` variants `Unknown`, `Score0` through `Score4`; refinement order: `Unknown` ⊐ each `Score{N}`; compatibility: each `Score{N}` compat only with self and `Unknown` | 2.2 | cc:TODO |
| 2.4 | Implement `SofaRespOperator` | `impl Operator for SofaRespOperator`; body: compute pao2/fio2, map to SOFA score, abstain if fio2 ≤ 0 or score ≥3 without ventilation, otherwise refine to `Score{N}`; test-passing implementation | 2.3 [tdd:required] | cc:TODO |
| 2.5 | Write soundness argument for operator | `clinlat/docs/operators/sofa_resp_soundness.md`; state three DEF-PS-08 soundness clauses and argue each; cite Vincent et al. 1996 and Singer et al. 2016 (Sepsis-3); informal-argument tier per OBL-PS-03 | 2.4 | cc:TODO |

---

## Phase 3: Tests and docs

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 3.1 | Unit tests for `Hyp` type | Tests for PartialOrd properties, compat predicate, meet behavior; all passing | 1.6 [tdd:required] | cc:TODO |
| 3.2 | Unit tests for `SofaRespOperator` | Three cases: (a) accepting (ratio 350 → `Refined(Score1)`), (b) abstaining on insufficient evidence (fio2=0 → `Abstain(InsufficientEvidence)`), (c) refusing on precondition (ratio 80 without vent → `Abstain(OperatorPreconditionUnmet)`); property test: operator output always refines input; all passing | 2.4 [tdd:required] | cc:TODO |
| 3.3 | Write `clinlat/README.md` | End-to-end example: construct `Evidence`, apply `SofaRespOperator`, inspect `Outcome`; cross-refs to NOTE.md §4A and SPEC.md §2; rustdoc links | 2.5 | cc:TODO |
| 3.4 | Crate-level and module-level rustdoc | Rustdoc for `clinlat`, `clinlat::hyp`, `clinlat::operator`, `clinlat::sofa` modules; all public items documented | 3.3 | cc:TODO |
| 3.5 | Verify `cargo doc --no-deps` renders without warnings | `cargo doc --no-deps` succeeds, no warnings or errors | 3.4 | cc:TODO |

---

## Phase 4: Release prep

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 4.1 | Verify CI green on fresh clone | Clone repo, `cargo test` and `cargo doc --no-deps` pass; all CI jobs succeed | 3.5 | cc:TODO |
| 4.2 | Dry-run publish verification | `cargo publish --dry-run` succeeds without errors | 4.1 | cc:TODO |
| 4.3 | Surgical NOTE.md update if needed | If kernel surfaces prose revisions, bump to v0.13.0-draft and document in CHANGELOG; otherwise leave at v0.12.0 | 4.2 | cc:TODO |
| 4.4 | Surgical SPEC.md update if needed | Close OQ-MP-02 (Result → Outcome rename decision documented); close OQ-X-03 partially (one operator informal-argument discharge as worked example) | 4.3 | cc:TODO |
| 4.5 | Tag and release | Tag `clinlat-v0.1.0` in git; GitHub release with changeset summary | 4.4 | cc:TODO |
| 4.6 | DOI minting decision | Decide per kernel-first ordering: if yes, GitHub release → Zenodo auto-mint → stamp DOI in NOTE.md header and clinlat README | 4.5 | cc:TODO |

---

## Explicitly out of scope for v0.1.0

These belong to v0.2.0+ backlog:

- Real ontology binding (SNOMED CT, RxNorm, LOINC, ICD-11) — DEF-PS-03
- Real provenance carriers beyond `()` — DEF-MP-14
- Galois connection (`α_PS`, `γ_PS`) — DEF-PS-06
- Operator-set type `Δ_PS` — DEF-PS-09
- Proposer (learned component) — DEF-PS-14
- Any institutional substrate (§3) work
- Any interaction layer (§4) work
- Any temporal evolution (§5) work
- ARCHITECTURE.md revision against SPEC.md v0.3.0
- Additional operators beyond SOFA-respiratory

---

## Next steps

**New session startup:**

```
claude
```

**After startup, use:**

```
/harness-work 0.1
```

**Suitable for:**  
Phase 0 is a half-day of setup; starting with task 0.1 gives repo structure before diving into core types (Phase 1).

Alternatively, to batch all four phases after repo setup:

```
/breezing all
```

This will execute phases in dependency order (0 → 1 → 2 → 3 → 4).
