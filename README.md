# Substrate-First Clinical AI (SFClinAI)

A position note arguing that clinical AI's safety properties must be enforced by a **symbolic substrate**, not by the behavior of learned components — with a coupled two-layer (patient + institutional) architecture, a temporal-evolution axis, six worked examples, and prior-art mapping.

This repository holds the prose of that position, plus the **`clinlat` Rust kernel** (`v0.2.0-alpha.0`; `v0.1.0` on crates.io): a symbolic substrate for clinical hypothesis refinement with sound deduction operators, ontology-bounded atoms, and auditable provenance.

---

## What it is

A working draft (`NOTE.md`, v0.12.0-draft) and accompanying Mermaid diagrams (`ARCHITECTURE.md`, v0.2.0-draft) staking out a falsifiable diagnostic claim: **clinical AI is bottlenecked less by model size than by the absence of explicit, auditable, reversible belief-state architectures that can update medical hypotheses under hard safety constraints** (NOTE.md §3).

The architecture rests on eighteen load-bearing principles — five for the patient-state substrate (§4A), five for the institutional-state substrate (§4B), three for the interaction semantics between them (§4C), and five for temporal evolution and substrate currency (§4D). Each is stated separately so it can be attacked separately.

## What it isn't

Not a product, startup thesis, or published paper. The note is explicitly **a working draft for scrutiny, not for citation** (NOTE.md header). The `clinlat` kernel is an early research substrate — it discharges patient-substrate proof obligations in code, but it is not a clinical tool: no deployed system, no patient data, no regulatory engagement. The work is a research program with a 2–4 year horizon for narrow applications and 5–8+ years for broader diagnostic reasoning (§7). Originality claims carry an explicit obsolescence window and are re-verified quarterly; the current assessment is valid through 2026-08-24 (NOTE.md header; §6 closing).

## Quick start (read this order)

1. **`NOTE.md` §TL;DR through §3** — the problem, why scaling doesn't fix it, the principle stated plainly.
2. **`ARCHITECTURE.md`** — five Mermaid diagrams. Read these _alongside_ §4 of the note.
3. **`NOTE.md` §4** — the eighteen load-bearing principles (4A patient ×5, 4B institutional ×5, 4C interaction ×3, 4D temporal ×5).
4. **`NOTE.md` §7E.1–7E.6** — six worked examples: Sepsis-3 prognostic stratification, KDIGO AKI staging, Wells/PE with sequential testing, CURB-65 disposition, chronic depression as an anti-example, and the live SSC 2021 → 2026 guideline transition through the substrate.
5. **`NOTE.md` §6 and §8** — prior art, originality narrowing after substantial verification, and honest limitations.

Short on time: stopping after step 3 still holds the load-bearing claim.

Formal-methods readers: **`SPEC.md`** sits between the note and the diagrams in the document stack (NOTE → SPEC → ARCHITECTURE). It carries the typed definitions, invariants, and proof obligations behind the eighteen principles, each annotated `[formalizes: NOTE.md §X.Y]`. The `clinlat` kernel is where the §2 patient-substrate obligations are discharged in code.

## Repository layout

```
SFClinAI/
├── NOTE.md             # Position note, v0.12.0-draft
├── ARCHITECTURE.md     # Five Mermaid diagrams, v0.2.0-draft
├── SPEC.md             # Engineering formalization, v0.3.0-draft
├── clinlat/            # Rust substrate kernel, v0.2.0-alpha.0
│   ├── Cargo.toml
│   ├── src/
│   ├── README.md       # Kernel documentation
│   └── docs/           # Soundness arguments
├── README.md           # This file
└── LICENSE             # CC BY 4.0 (verbatim legal code)
```

## Architecture overview

Two substrates, coupled, evolving in time.

The **patient-state substrate** carries a refinable lattice of clinical hypotheses with sound deduction operators, ontology-bounded candidates, first-class abstention, and auditable provenance; the learned component is a constrained refinement proposer, not a decision-maker (§4A.1–4A.5).

The **institutional-state substrate** mirrors this for cross-patient resource allocation — beds, formulary, lab cycles, on-call coverage — with capacity-update operators, allocation abstention, and capacity-learned components as constrained proposers (§4B.1–4B.5).

The two communicate through a defined event interface with **joint licensing** of recommendations and **joint abstention** as a first-class output: when patient-locally-optimal diverges from institutionally-feasible, the system produces both with explicit diff rather than silently downgrading (§4C).

All substrate components are versioned; evidence currency is architectural signal; operator-set changes are sound by construction; guideline updates propagate into active care only through clinician-mediated re-review (silent drift is structurally forbidden); provenance is evolution-aware (§4D.1–4D.5).

The composite is positioned as a route from UNDCS-class to CCS-class under the Tan et al. (2026) regulatory taxonomy, making LLM-class components evaluable under existing FDA-SaMD pathways (§5).

## Status and maturity

- `NOTE.md`: **v0.12.0-draft**. Working draft for scrutiny. Not for citation.
- `SPEC.md`: **v0.3.0-draft**. The formalization layer between the note and the diagrams: 57 named definitions (`DEF-*`), 23 invariants (`INV-*`), 17 proof obligations (`OBL-*`), 3 mathematical commitments (`MC-*`), and 12 open questions (`OQ-*`). Each definition is annotated `[formalizes: NOTE.md §X.Y]`, and §8 bidirectional traceability covers all eighteen principles with no known coverage gaps. The `clinlat` kernel discharges the §2 patient-substrate obligations in running code.
- `ARCHITECTURE.md`: **v0.2.0-draft**. Re-anchored to `SPEC.md` v0.3.0-draft: principle-to-formalization map covering all eighteen principles with criticality tier (P/S/F) and SPEC.md DEF/INV/OBL anchors; Diagram 4 now renders the institutional symmetric re-review path (DEF-TE-06b) alongside the patient-side clinician-mediated path.
- `clinlat`: **v0.2.0-alpha.0** (`v0.1.0` published to crates.io; the v0.2.0 milestone M1 — patient-substrate completion — is in progress). Patient-state substrate with `Hyp` refinement poset, `Outcome<H,A>` result type, and the SOFA-3 respiratory operator. Phases 0–3 of M1 have landed: an `Atom` type backed by four `OntologyAdapter` implementations (SNOMED CT, RxNorm, LOINC, ICD-11) replacing string atom IDs; a typed `Provenance` carrier with version-respecting derivation chains (OBL-PS-04); and the Galois connection α_PS / γ_PS property-tested for the adjunction laws (OBL-PS-02). Remaining for the v0.2.0 cut: `OperatorSet` composition (Phase 4); three further operators — KDIGO AKI, Wells/PE, CURB-65 (Phase 5); and the SOFA-respiratory property-test-tier upgrade (Phase 6). See [`clinlat/README.md`](clinlat/README.md) for quick start, [`clinlat/CHANGELOG.md`](clinlat/CHANGELOG.md) for version history, and [`clinlat/docs/operators/sofa_resp_soundness.md`](clinlat/docs/operators/sofa_resp_soundness.md) for the SOFA soundness argument.
- No deployed system. No regulatory engagement.
- Originality assessment timestamped 2026-05-24, valid through **2026-08-24** (quarterly re-verification cadence, per NOTE.md header and §6 closing).

## Citing this work

Please cite the specific version and date accessed, and treat as a working draft:

> Sucandra, K. (2026). _Substrate-First Clinical AI: A Position Note_ (v0.12.0-draft) [Working draft]. <https://github.com/SHA888/SFClinAI>

Author byline: **Kresna Sucandra, MD**. The note's header asks that the draft not be cited as a settled position; if citation is unavoidable, cite as a working draft.

## Contributing

The note exists to be argued with (§9). In order of preference:

1. A demonstration that the synthesis is **already published** and was missed in the §6 prior-art mapping.
2. A demonstration that **one of the eighteen principles** in §4 is wrong or unnecessary — they are stated separately so they can be attacked separately.
3. A demonstration that the **substrate-first framing fails** on a clinical decision the six worked examples (§7E.1–7E.6) did not cover.

Issues for short critiques; for long-form, open a PR and tag it as a long-form critique.

## License

- **Prose and diagrams** (`NOTE.md`, `ARCHITECTURE.md`, `SPEC.md`, `README.md`) are licensed under **Creative Commons Attribution 4.0 International (CC BY 4.0)**. See `LICENSE` for the preamble and full legal code.
- **Code** (including the `clinlat/` crate and any future code) is licensed under **MIT OR Apache-2.0**. See `LICENSE-MIT` and `LICENSE-APACHE` for the full text of each license.

Attribution: **Kresna Sucandra, MD**.
