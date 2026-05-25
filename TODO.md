# SFClinAI — TODO

**Scope:** substrate kernel v0.1.0 milestone only. Out-of-scope items live in `NOTE.md` §7 and `SPEC.md` §7; do not add long-horizon planning here.

**Milestone definition (v0.1.0):**

- A `clinlat` Rust crate lives at `clinlat/` in this repo.
- The crate exposes a `Hyp` poset type, a `Result⟨H, A⟩` sum, and one deduction operator: Sepsis-3 SOFA respiratory-component (PaO₂/FiO₂).
- The operator's soundness obligation (OBL-PS-03 per `SPEC.md` §6) is discharged at the "informal argument" tier — a markdown paragraph attached to the operator, named, dated.
- `cargo test` passes for the operator on a hand-authored test fixture (one accepting case, one abstaining case, one refusing case).
- `cargo doc` renders without warnings.
- README in `clinlat/` documents the operator end-to-end.

**Status tags:** `[ ]` not started · `[~]` in progress · `[x]` done · `[!]` blocked/needs decision

---

## Phase 0 — Repo prerequisites (~half-day)

- [ ] **0.1 Add `clinlat/` directory to SFClinAI repo root.**
- [ ] **0.2 Initialize Rust crate.**
  - [ ] 0.2.1 `cargo new --lib clinlat` from repo root.
  - [ ] 0.2.2 Set `edition = "2024"` in `Cargo.toml`.
  - [ ] 0.2.3 Set `rust-version = "1.86.0"` to match your standard MSRV.
  - [ ] 0.2.4 Set `license = "MIT OR Apache-2.0"`.
  - [ ] 0.2.5 Set `repository = "https://github.com/SHA888/SFClinAI"` and `documentation = "https://docs.rs/clinlat"`.
- [ ] **0.3 Add license files at repo root.**
  - [ ] 0.3.1 Add `LICENSE-MIT` (verbatim MIT text, copyright "Kresna Sucandra").
  - [ ] 0.3.2 Add `LICENSE-APACHE` (verbatim Apache-2.0 text).
  - [ ] 0.3.3 Update existing `LICENSE` to clarify: applies to prose files (NOTE, SPEC, ARCHITECTURE, README); code under MIT OR Apache-2.0.
- [ ] **0.4 Surgical update to README.md License section.** Single-line change: "future code = Apache-2.0" → "code = MIT OR Apache-2.0 dual".
- [ ] **0.5 Add `cargo-skill` per userPreferences standard.** `cargo install cargo-skill` documented in a CONTRIBUTING.md stub.
- [ ] **0.6 CI scaffolding.**
  - [ ] 0.6.1 `.github/workflows/ci.yml` with: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo doc --no-deps`.
  - [ ] 0.6.2 `cargo-semver-checks` job (runs on push to main) per your meta-rule that SemVer needs CI enforcement.
  - [ ] 0.6.3 MSRV check job pinning to 1.86.0.
- [ ] **0.7 `.gitignore` and `rust-toolchain.toml`.**

## Phase 1 — Core type scaffolding (~1–2 days)

- [ ] **1.1 `Hyp` type stub.**
  - [ ] 1.1.1 Define `pub struct Hyp` as a placeholder newtype around `Vec<AtomId>` for v0.1.0.
  - [ ] 1.1.2 Implement `PartialOrd` reflecting refinement order from `SPEC.md` DEF-PS-01.
  - [ ] 1.1.3 Implement compatibility predicate `compat(h1, h2) -> bool` per DEF-PS-01.
  - [ ] 1.1.4 Implement partial meet `meet(h1, h2) -> Option<Hyp>` per DEF-MP-05.
  - [ ] 1.1.5 Document the v0.1.0 simplification: `AtomId` is `&'static str` for now; ontology binding (DEF-PS-03) is deferred to v0.2.
- [ ] **1.2 `AtomId` placeholder.** `pub type AtomId = &'static str;` with doc comment naming the deferred ontology binding.
- [ ] **1.3 `Result<H, A>` operator output type.**
  - [ ] 1.3.1 `pub enum Outcome<H, A> { Refined(H), Abstain(A) }` per DEF-MP-13 (renamed from `Result` to avoid Rust collision; matches SPEC.md OQ-MP-02 disposition).
  - [ ] 1.3.2 Implement `Outcome::map`, `Outcome::and_then` for ergonomic composition.
- [ ] **1.4 `AbstainReason` enum stub.**
  - [ ] 1.4.1 Define five variants from DEF-PS-10: `InsufficientEvidence`, `OutOfDistribution`, `AmbiguousRefinement`, `OperatorPreconditionUnmet`, `OntologyOutOfScope`.
  - [ ] 1.4.2 v0.1.0 simplification: each variant carries a `&'static str` rather than structured detail. Structured detail deferred to v0.2.
- [ ] **1.5 Evidence type stub.**
  - [ ] 1.5.1 `pub struct Evidence` carrying a typed observation packet.
  - [ ] 1.5.2 v0.1.0 simplification: provenance carrier (`Prov` per DEF-MP-14) is unit type `()`. Real provenance deferred to v0.2.
- [ ] **1.6 Operator signature trait.**
  - [ ] 1.6.1 `pub trait Operator { fn apply(&self, h: &Hyp, e: &Evidence) -> Outcome<Hyp, AbstainReason>; }` matching DEF-PS-07.

## Phase 2 — The SOFA-respiratory operator (~2–3 days)

- [ ] **2.1 SOFA respiratory thresholds.** Encode the PaO₂/FiO₂ ratio thresholds per Sepsis-3: ≥400, 300–399, 200–299, 100–199, <100 (SOFA scores 0–4).
- [ ] **2.2 Evidence type for SOFA respiratory.** `struct SofaRespEvidence { pao2: f64, fio2: f64, on_mech_vent: bool }` — last field needed because SOFA scores 3–4 require mechanical ventilation.
- [ ] **2.3 Hypothesis space.** `Hyp` variants for SOFA respiratory: `Unknown`, `Score0`, `Score1`, `Score2`, `Score3`, `Score4`. Refinement order: `Unknown` ⊐ each `Score{N}`. Compatibility: each `Score{N}` is compatible only with itself and with `Unknown`.
- [ ] **2.4 Implement the operator.**
  - [ ] 2.4.1 `pub struct SofaRespOperator;`
  - [ ] 2.4.2 `impl Operator for SofaRespOperator` with the body:
    - Compute ratio = pao2 / fio2.
    - Map ratio + ventilation to SOFA score.
    - Abstain `InsufficientEvidence` if fio2 is zero or negative.
    - Abstain `OperatorPreconditionUnmet` if score ≥3 is computed but `on_mech_vent == false` (Sepsis-3 requires ventilation for scores 3–4).
    - Otherwise return `Refined(Score{N})`.
- [ ] **2.5 Soundness argument.**
  - [ ] 2.5.1 Markdown file `clinlat/docs/operators/sofa_resp_soundness.md`.
  - [ ] 2.5.2 State the three soundness clauses from DEF-PS-08 and argue each holds for this operator. Informal-argument tier per OBL-PS-03's discharge plan.
  - [ ] 2.5.3 Cross-reference: SOFA reference (Vincent et al. 1996), Sepsis-3 (Singer et al. 2016, JAMA).

## Phase 3 — Tests and docs (~1–2 days)

- [ ] **3.1 Unit tests for `Hyp`.** Order properties, compat predicate, meet behavior.
- [ ] **3.2 Unit tests for `SofaRespOperator`.**
  - [ ] 3.2.1 Accepting case: ratio 350, returns `Refined(Score1)`.
  - [ ] 3.2.2 Abstaining case: fio2 = 0, returns `Abstain(InsufficientEvidence)`.
  - [ ] 3.2.3 Refusing case: ratio 80 without ventilation, returns `Abstain(OperatorPreconditionUnmet)`.
  - [ ] 3.2.4 Refinement-direction property test: operator output is always a refinement of input.
- [ ] **3.3 `clinlat/README.md`.** End-to-end example: construct evidence, apply operator, inspect outcome. Cross-references to `NOTE.md` §4A and `SPEC.md` §2.
- [ ] **3.4 Crate-level docs.** Module-level rustdoc for `clinlat`, `clinlat::hyp`, `clinlat::operator`, `clinlat::sofa`.
- [ ] **3.5 `cargo doc --no-deps` renders without warnings.**

## Phase 4 — Release prep (~half-day)

- [ ] **4.1 Verify CI green on a fresh clone.**
- [ ] **4.2 Verify `cargo publish --dry-run` succeeds.**
- [ ] **4.3 Surgical NOTE.md update.** Bump to v0.13.0-draft if the kernel surfaces any prose revisions (per the "kernel-first revisions" pattern). Otherwise leave at v0.12.0.
- [ ] **4.4 Surgical SPEC.md update.** Close OQ-MP-02 (Result naming → renamed to Outcome, decision documented). Close OQ-X-03 partially (one operator now has informal-argument discharge as worked example).
- [ ] **4.5 Tag `clinlat-v0.1.0` in git.**
- [ ] **4.6 Decide on Zenodo DOI minting per the prior decision (kernel-first ordering).** If yes: GitHub release → Zenodo automatic mint → stamp DOI in NOTE.md header and clinlat README.

---

## Explicitly out of scope for v0.1.0

These belong to v0.2.0+ and live as backlog:

- Real ontology binding (SNOMED CT, RxNorm, LOINC, ICD-11) — DEF-PS-03.
- Real provenance carriers — DEF-MP-14 beyond the `()` stub.
- Galois connection (`α_PS`, `γ_PS`) — DEF-PS-06.
- Operator-set type `Δ_PS` — DEF-PS-09.
- Proposer (learned component) — DEF-PS-14.
- Any institutional substrate (§3) work.
- Any interaction layer (§4) work.
- Any temporal evolution (§5) work.
- ARCHITECTURE.md revision against SPEC.md v0.3.0.
- Additional operators beyond SOFA-respiratory.

## Notes on discipline

- Items move from `[ ]` → `[~]` when work starts; `[~]` → `[x]` when done; `[~]` → `[!]` if blocked >24h.
- An item stuck at `[!]` for >1 week gets either decomposed further or moved to a `BACKLOG` section.
- TODO.md never grows past ~200 lines; when v0.1.0 ships, this file resets to track v0.2.0.
- This file is not the source of truth for vision (`NOTE.md`), formalization (`SPEC.md`), or architecture (`ARCHITECTURE.md` once revised). It is the source of truth only for: "what is the next concrete action toward `clinlat` v0.1.0."
-
