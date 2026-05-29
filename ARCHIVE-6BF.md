# Phase 6-BF: Bugfix Archive (M1.6-BF) — Implementation Correctness

**Status:** Archived 2026-05-29 — All 11 bugs fixed and verified
**Goal:** Fix 11 semantic bugs found in clinlat implementation review. Prioritized by severity: 5 HIGH (blocking v0.2.0), 3 MEDIUM (v0.2.0–v0.3.0), 3 LOW (maintenance).

## HIGH severity (blocking v0.2.0)

| Task | Content | Commit |
|------|---------|--------|
| 6b.1 | **Bug #1: Hyp::new allows duplicate atoms, breaking PartialOrd consistency** — Normalize atoms on construction. Deduplicate Vec and sort by (system, code) | [743e1d7](https://github.com/SHA888/SFClinAI/commit/743e1d7) |
| 6b.2 | **Bug #5: SofaRespOperator::apply ignores input hypothesis, violates monotonicity** — Operator must refine input (δ(h,e) ⊑ h per DEF-PS-08). Update apply() to chain refined_atoms = h.atoms() + sofa_atom; add assertion | [3c3b426](https://github.com/SHA888/SFClinAI/commit/3c3b426) |
| 6b.3 | **Bug #3 + #6: Version string drift — SofaRespOperator emits hard-coded "clinlat-v0.2.0" instead of self.version** — Refactor score_to_atom() from fn to &self method; use self.version in atoms. Align with is_consistent_with() version semantics | [3c3b426](https://github.com/SHA888/SFClinAI/commit/3c3b426) |
| 6b.4 | **Bug #4: Atom identity mismatch — Hash/Eq includes preferred_term, but validation (validate_compatibility) excludes it** — Implement custom Hash/Eq that exclude preferred_term (or move it outside the identity). Update PartialOrd to use (system, code, version) for identity | [79f2645](https://github.com/SHA888/SFClinAI/commit/79f2645) |
| 6b.5 | **Bug #2: Hyp::compat violates INV-PS-01 (compatibility under refinement)** — Choose reconciliation from SPEC.md §7 open question (3.7 decision outcome). Implement compat_refined() or widen compat definition | [79f2645](https://github.com/SHA888/SFClinAI/commit/79f2645) |

## MEDIUM severity (v0.2.0–v0.3.0)

| Task | Content | Commit |
|------|---------|--------|
| 6b.6 | **Bug #7: Wrong abstain variant — SofaRespOperator uses InsufficientEvidence for precondition failure** — Replace with OperatorPreconditionUnmet; update abstain handling to distinguish data absence from precondition inapplicability | [31411e7](https://github.com/SHA888/SFClinAI/commit/31411e7) |
| 6b.7 | **Bug #8: Panic on zero cache — NonZeroUsize::new(cache_size).unwrap() panics in all four OntologyAdapter constructors** — Add bounds check; either panic early with descriptive message or use default (e.g., 1024) | [31411e7](https://github.com/SHA888/SFClinAI/commit/31411e7) |
| 6b.8 | **Bug #9: SOFA boundary off-by-one — Threshold 300.0 vs Sepsis-3 table unclear** — Verify exact boundary against Sepsis-3 table (Singer et al. 2016); add test case for exactly 300.0 and document rationale in comment | [31411e7](https://github.com/SHA888/SFClinAI/commit/31411e7) |

## LOW severity (maintenance)

| Task | Content | Commit |
|------|---------|--------|
| 6b.9 | **Bug #10: Lock release in cache — concurrent cache misses cause redundant inserts** — Hold lock across get+put or use entry API; document rationale for chosen pattern | [a9aa92d](https://github.com/SHA888/SFClinAI/commit/a9aa92d) |
| 6b.10 | **Bug #11: AbstainReason too simple — carries &'static str instead of structured payloads per SPEC.md DEF-PS-10** — Document as v0.1.0 simplification; add TODO or create v0.2.0+ card in backlog | [a9aa92d](https://github.com/SHA888/SFClinAI/commit/a9aa92d) |
| 6b.11 | **Property-test verification of all fixes** — Run full test suite; verify OBL-PS-02 adjunction, OBL-PS-03 operator composition, INV-PS-01 through INV-PS-06 | [a9aa92d](https://github.com/SHA888/SFClinAI/commit/a9aa92d) |

## Summary

✅ All 11 bugs fixed and verified
✅ 129 tests passing
✅ All invariants (INV-PS-01 through INV-PS-06) satisfied
✅ All obligations (OBL-PS-02, OBL-PS-03) met
✅ Clippy clean, docs generated

**Next: Phase 4 (OperatorSet formalization) ready to begin.**
