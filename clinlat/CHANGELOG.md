# Changelog

All notable changes to the `clinlat` crate are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Pre-1.0 minor bumps (`0.x.0`) carry breaking changes by SemVer convention.

## [Unreleased]

Tracking what is not yet on a published version. See `Plans.md` for the
authoritative phase-by-phase task list.

### Planned for `0.3.0` (M2 — Constrained refinement proposer)

- Black-box proposer interface per DEF-PS-14 / DEF-PS-15 (INV-PS-06).
- Soundness verification adapter wiring proposer output through safety gates.
- Two reference proposers: deterministic lattice search + LLM-class adapter.
- OBL-PS-05 discharge (proposer-operator separation enforced structurally).

## [0.2.0] — 2026-05-31

**Milestone M1: Patient substrate completion.** All eleven 4A-anchored SPEC.md
elements (DEF-PS-01..15, INV-PS-01..06, OBL-PS-01..05) reachable from running
code. Four operators discharged (SOFA at property-test tier; KDIGO, Wells,
CURB-65 at informal-argument tier). 193 tests passing.

### Added

- **Galois connection (Phase 3):** `abstract_evidence` (α_PS) and the
  `is_consistent_with` predicate (γ_PS), property-tested for the adjunction
  laws — `e ∈ γ_PS(α_PS(e))`, `α_PS(γ_PS(h)) ⊑ h`, and monotonicity —
  discharging OBL-PS-02 at the property-test tier. (DEF-PS-05 / DEF-PS-06)
  See `docs/obligations/obl-ps-02-adjunction.md`.

- **Operator-set type `Δ_PS` (Phase 4):** `OperatorSet` struct with
  `apply_set()` method implementing propagate-forward abstention semantics
  per DEF-PS-09. Soundness obligation OBL-PS-03 extended across the set via
  6 property tests. See `docs/obligations/obl-ps-03-operator-set-sound.md`.

- **Three additional operators (Phase 5):**
  - `KdigoAkiOperator`: KDIGO 2021 AKI staging by serum creatinine
    fold-change and urine output decline. 9 unit tests. Soundness discharge:
    `docs/operators/kdigo_aki_soundness.md` (informal-argument tier).
  - `WellsPeOperator`: Wells score for PE risk stratification with sequential
    testing (D-dimer vs. CTPA). 8 unit tests. Soundness discharge:
    `docs/operators/wells_pe_soundness.md` (informal-argument tier).
  - `Curb65Operator`: CURB-65 CAP disposition (outpatient vs. ward vs. ICU).
    10 unit tests. Soundness discharge: `docs/operators/curb65_soundness.md`
    (informal-argument tier).

- **SOFA-respiratory discharge-tier upgrade (Phase 6):** 17 new property-test
  cases (46 total: 29 unit + 17 property) validating monotonicity, boundary
  coverage, and abstention invariants. Upgraded from informal-argument to
  property-test tier. See refreshed `docs/operators/sofa_resp_soundness.md`.

- **Integration and release (Phase 7):**
  - `clinlat/README.md` updated with three M1 operator examples (KDIGO, Wells,
    CURB-65) and M1 status section. Cross-linked to soundness docs and SPEC.md
    §2–8 for formal foundations.
  - Soundness discharge documents created for KDIGO AKI and CURB-65 operators,
    completing the set.
  - Bidirectional traceability verified: all 18 NOTE.md §4A principles mapped
    to SPEC.md formalizations in §8.

### Fixed

- **Phase 5 Code Review Bugfixes:** 9 critical operator bugs fixed post-review:
  - KDIGO AKI (6 bugs): Division by zero on baseline Cr=0; missing UO Stage 2;
    missing acute-rise qualifier for absolute Cr ≥4.0; no temporal window
    validation for UO; LOINC code collision; provenance loss on UO upgrade.
  - CURB-65 (2 bugs): DBP criterion lost in else-if when SBP normal; urea flag
    set true even when value unparseable.
  - Wells/PE (1 bug): Missing abstention on PE gestalt assessment (now enforces
    clinician judgment as mandatory input).

### Changed

- **Version status:** Pre-release suffix `0.2.0-alpha.0` dropped. Shipping as
  stable `0.2.0` with M1 DoD satisfied.

### Verified

- ✓ `cargo test --lib`: 193 tests passing (earlier: 190 baseline → +3 bugfix tests)
- ✓ `cargo doc --no-deps`: Full docs render without warnings
- ✓ `cargo fmt --check`: Code formatted
- ✓ `cargo clippy -- -D warnings`: No clippy warnings
- ✓ `cargo publish --dry-run`: Package ready for crates.io

## [0.2.0-alpha.0] — 2026-05-26

Pre-release. Phases 0–2 of the M1 milestone (Patient substrate completion).
**Breaking** changes relative to `0.1.0`; pre-release suffix used while
Phases 3–7 land.

### Added

- `Atom` struct (`{ system, code, preferred_term, version }`) replacing the
  v0.1.0 `&'static str` AtomId. (Phase 1, DEF-PS-03)
- `OntologySystem` enum and `OntologyAdapter` trait with four offline-snapshot
  implementations: `SNOMEDAdapter`, `RxNormAdapter`, `LoincAdapter`,
  `Icd11Adapter`. (Phase 1, DEF-PS-03 / DEF-PS-04)
- `Provenance` carrier with `ProvenanceOrigin`, ISO 8601 timestamp, operator
  `Ver`, metadata, and optional `derives_from` ancestor hashes. JSON
  serializable with optional gzip compression. (Phase 2, DEF-MP-14 /
  DEF-PS-12 / DEF-PS-13)
- `Observation` struct (`{ code, value, unit, source }`) for clinical
  observations carried inside `Evidence`. (Phase 2, DEF-MP-10)
- `Evidence` struct (`{ observations, provenance }`) replacing the v0.1.0
  unit type. Constructor `Evidence::new(observations, provenance)` makes
  provenance mandatory at the type level. (Phase 2, DEF-MP-11)
- `Ver` struct (`{ system, operator, build }`) for version-respecting
  derivation chains. (Phase 2, DEF-MP-16)
- `SofaRespOperator::new(version)` and `default_v0_2()` constructors,
  plus a full `Operator::apply()` implementation that validates evidence
  provenance, extracts observations, computes the SOFA respiratory score,
  and returns a typed `Outcome<Hyp, AbstainReason>`. (Phase 2, DEF-PS-07)
- Discharge proof documents:
  - `docs/invariants/inv-ps-01-closure.md` — ontology closure (Phase 1).
  - `docs/obligations/obl-ps-04-provenance-audit.md` — provenance
    auditability (Phase 2).

### Changed

- **Breaking:** `Hyp::new` now takes `Vec<Atom>` instead of
  `Vec<&'static str>`. All hypotheses are constructed from atoms resolved
  through `OntologyAdapter`.
- **Breaking:** `Operator::apply` now takes `&Evidence` (the new struct
  type) instead of the v0.1.0 unit `Evidence`.
- **Breaking:** `SofaRespOperator` is now a struct `{ version: String }`
  rather than a unit struct, and enforces the version-respecting
  derivation chain invariant (INV-PS-05) by abstaining on version
  mismatch.
- **Breaking:** `Evidence` no longer derives `Eq` (it contains
  `serde_json::Value`, which carries floats and is `PartialEq` only).

### Tooling

- Added `cargo-doc` pre-commit hook with `RUSTDOCFLAGS='-D warnings'` to
  catch rustdoc errors locally before CI.

## [0.1.0] — 2026-04 (initial publication)

Initial crates.io publication. Skeleton substrate kernel with simplified
types (string AtomId, unit Evidence, unit Provenance) and the
SOFA-respiratory worked example.

### Added

- `Hyp` poset of clinical hypotheses (refinement-ordered).
- `Outcome<H, A>` sum type for operator results.
- `Operator` trait (generic, with unit `Evidence`).
- `SofaRespOperator` as a unit struct with `score_from_ratio` numeric
  helper.
- `AbstainReason` enum.

[Unreleased]: https://github.com/SHA888/SFClinAI/compare/clinlat-0.2.0...HEAD
[0.2.0]: https://github.com/SHA888/SFClinAI/compare/clinlat-0.2.0-alpha.0...clinlat-0.2.0
[0.2.0-alpha.0]: https://github.com/SHA888/SFClinAI/releases/tag/clinlat-0.2.0-alpha.0
[0.1.0]: https://crates.io/crates/clinlat/0.1.0
