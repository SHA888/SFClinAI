# Changelog

All notable changes to the `clinlat` crate are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Pre-1.0 minor bumps (`0.x.0`) carry breaking changes by SemVer convention.

## [Unreleased]

Tracking what is not yet on a published version. See `Plans.md` for the
authoritative phase-by-phase task list.

### Added (landed since `0.2.0-alpha.0`, not yet tagged)

- Galois connection (Phase 3): `abstract_evidence` (α_PS) and the
  `is_consistent_with` predicate (γ_PS), property-tested for the adjunction
  laws — `e ∈ γ_PS(α_PS(e))`, `α_PS(γ_PS(h)) ⊑ h`, and monotonicity —
  discharging OBL-PS-02 at the property-test tier. (DEF-PS-05 / DEF-PS-06)
  See `docs/obligations/obl-ps-02-adjunction.md`. Post-review test-coverage
  cleanups (Plans.md tasks 3.5–3.7) remain before Phase 4.

### Planned for `0.2.0` (M1 — Patient substrate completion)

- `OperatorSet` type and composition per DEF-PS-09 / OBL-PS-03 (Phase 4).
- Three additional operators: KDIGO AKI, Wells/PE, CURB-65 (Phase 5).
- SOFA-respiratory upgrade from informal-argument to property-test tier (Phase 6).
- Release prep: SPEC/ARCHITECTURE cross-references, `cargo publish --dry-run` (Phase 7).

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

[Unreleased]: https://github.com/SHA888/SFClinAI/compare/clinlat-0.2.0-alpha.0...HEAD
[0.2.0-alpha.0]: https://github.com/SHA888/SFClinAI/releases/tag/clinlat-0.2.0-alpha.0
[0.1.0]: https://crates.io/crates/clinlat/0.1.0
