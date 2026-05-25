# SPEC.md — Substrate-First Clinical AI: Formalization

**Version:** v0.1.0-draft
**Status:** Working draft for scrutiny. Not for citation.
**License:** CC BY 4.0
**Companion to:** `NOTE.md` v0.10.1-draft

---

## §0. Preliminaries

### §0.1. Purpose and audience

This document is the formalization layer between `NOTE.md` (clinical-prose position note) and `ARCHITECTURE.md` (visual architectural overview, to be drafted against this document, not against `NOTE.md` directly).

The audience is engineering and formal-methods readers. Clinical scrutiny of the eighteen load-bearing principles lives in `NOTE.md`. Clinical scrutiny of *whether SPEC.md faithfully formalizes those principles* depends on the bidirectional traceability in §8 — clinicians are not expected to read this document end-to-end.

SPEC.md exists to:

1. **Disambiguate.** Convert prose-level terms (*lattice*, *refinement*, *soundness*, *abstention*, *joint licensing*, *currency*, *sound by construction*) into named definitions with typed signatures and stated invariants.
2. **Make principles individually attackable.** Each of the eighteen principles in `NOTE.md` §4 gets at least one named definition, invariant, or proof obligation it can be challenged on. A reviewer who disputes principle §4A.3 should be able to point at the specific definition or invariant that operationalizes it.
3. **Surface gaps.** Where prose left a structural choice open, this document either makes the choice (with rationale) or explicitly defers it in §7.

SPEC.md does **not** exist to:

- Replace `NOTE.md`. The note remains the human-readable source of truth and the primary citation target.
- Provide mechanized proofs. Proof obligations are stated; discharge is out of scope for v0.1.
- Specify implementation. Concrete data structures, storage layouts, wire formats, and APIs belong in `ARCHITECTURE.md` and downstream design documents.

### §0.2. Notation tier

This document is **Tier B** in the formalization-depth taxonomy: typed signatures, named invariants, stated proof obligations, no mechanization. The tier was chosen deliberately:

- **Tier A** (pseudo-formal prose math) would not disambiguate enough to make principles individually attackable.
- **Tier C** (mechanized in Lean 4 or Agda) would lock in structural choices before the formalization itself has stabilized, and would narrow the reviewer pool below the threshold of useful scrutiny.

Migration to Tier C is a candidate v1.x or v2.x destination, conditional on Tier B stabilizing through at least one full revision cycle without breaking changes to load-bearing definitions.


### §0.3. Mathematical commitments

The following structural choices are made once here and used throughout. Each is one notch less specific than feels comfortable, on the principle that premature commitment to richer structure causes downstream re-versioning.

**MC-1. Hypothesis space as a poset, not a lattice.** Each substrate's hypothesis space is a partially ordered set (P, ⊑). The meet operation ⊓ is **partial**: it is defined for *compatible* pairs of hypotheses (defined per substrate in §2 and §3) and undefined otherwise. Full lattice structure — total meet and total join — is deferred to a future revision; declaring a total lattice now would require committing to a join operation whose clinical and institutional semantics are not yet clear.

  *Rationale.* Real clinical reasoning has hypotheses that do not have a meaningful greatest-lower-bound (e.g., "patient has pneumonia" and "patient has pulmonary embolism" — both can be true, but their meet is not their conjunction in the order-theoretic sense). Forcing a total meet now would either misrepresent that or trivialize it.

**MC-2. Refinement via Galois connection.** The relationship between an abstract hypothesis space `A` and a concrete observation space `C` is given by a Galois connection (α, γ) where α : C → A is the abstraction map and γ : A → C is the concretization map, satisfying the standard adjunction:

  ∀ c ∈ C, a ∈ A:  α(c) ⊑ a  ⟺  c ⊑ γ(a)

  *Rationale.* The Galois-connection structure makes "operator-set changes are sound by construction" (`NOTE.md` §4D.3) a stateable proof obligation rather than an assertion. Operators defined as abstractions of concrete transitions inherit soundness from the connection, provided the connection itself is preserved.

**MC-3. Abstention as a separate type.** Operators that may abstain do not return a distinguished bottom element of the hypothesis space. They return a value of a separate type:

  `Result⟨H, A⟩ = Refined(H) | Abstain(A)`

  where `H` is the hypothesis-space type and `A` is the abstention-reason type (defined per substrate).

  *Rationale.* Abstention is a first-class output in `NOTE.md` §4A.4, §4B.4, §4C.3. Encoding it as ⊥ ∈ P collapses two distinct epistemic states ("the data refines to no further hypothesis" vs. "the operator declines to commit") into one. The separate-type encoding makes the distinction enforceable by type-checking and makes the abstention reason part of the audit trail.


### §0.4. Notation conventions

The following symbols are reserved throughout SPEC.md.

| Symbol | Meaning | Notes |
|---|---|---|
| `⊑` | Refinement order on a hypothesis space | `h₁ ⊑ h₂` reads "h₁ refines h₂" (h₁ is more specific) |
| `⊓` | Partial meet on a hypothesis space | Defined for compatible pairs; see MC-1 |
| `α` | Abstraction map of a Galois connection | `α : C → A`; concrete to abstract |
| `γ` | Concretization map of a Galois connection | `γ : A → C`; abstract to concrete |
| `δ` | Deduction operator | Reserved per substrate in §2, §3 |
| `Result⟨H, A⟩` | Sum type for possibly-abstaining operators | See MC-3 |
| `∎` | End of a named definition or obligation | |

Type signatures use ML-style notation (`f : A → B`). Product types use `(A × B)`. Sum types use `A | B` or named constructors. Universally quantified type variables are lowercase Greek (`α`, `β`); these are distinct from the abstraction map `α`, which always appears in a Galois-connection context.

Named structural elements use prefixes:

- `DEF-{SUBSTRATE}-{NN}` — a definition (e.g. `DEF-PS-01` for the first patient-substrate definition).
- `INV-{SUBSTRATE}-{NN}` — a named invariant.
- `OBL-{SUBSTRATE}-{NN}` — a stated proof obligation (consolidated in §6).

Substrate prefixes: `PS` (patient-state), `IS` (institutional-state), `IX` (interaction), `TE` (temporal evolution), `MC` (mathematical commitment, used in §0).

### §0.5. Scope and exclusions

In scope for v0.1.0-draft:

- Formal definitions of the structures named in `NOTE.md` §4A–§4D.
- Stated proof obligations corresponding to soundness claims in `NOTE.md`.
- Bidirectional mapping between SPEC.md definitions and `NOTE.md` sentences (§8).

Out of scope for v0.1.0-draft (explicitly):

- **UI semantics.** How the system presents abstention, joint licensing diffs, or evidence-currency signals to a clinician.
- **Audit log format.** Whether provenance is encoded as JSON, CBOR, a Merkle DAG, or otherwise.
- **Deployment topology.** Process boundaries, network protocols, persistence layout.
- **Learning-component internals.** The proposer mechanism is treated as a black-box function with a typed signature; its training, architecture, and inference details are not formalized here.
- **Mechanized proofs.** Obligations in §6 are stated; their discharge is deferred.
- **Performance characteristics.** Latency, throughput, and complexity bounds are architectural concerns.
- **Specific clinical ontologies.** The integration of SNOMED CT, RxNorm, LOINC, ICD-11 is referenced via an abstract `OntologyBoundedSet` type defined in §2; concrete bindings belong in `ARCHITECTURE.md`.


### §0.6. Versioning and change discipline

SPEC.md follows SemVer applied to its public definitions and invariants. The rules:

- **MAJOR** bump: any backward-incompatible change to a `DEF-*`, `INV-*`, or `OBL-*` that downstream documents (`ARCHITECTURE.md`, code) may depend on. Examples: changing a type signature, removing a definition, weakening an invariant.
- **MINOR** bump: additive changes — new definitions, new invariants, new proof obligations, new substrates. Existing identifiers retain their meaning.
- **PATCH** bump: clarifications, typo fixes, rephrasings that preserve meaning, additions of cross-references or rationale text.

Each version is accompanied by an entry in a forthcoming `CHANGELOG-SPEC.md` (to be created when SPEC.md leaves draft status).

While `v0.x.y`, all bumps are permitted to be more aggressive than strict SemVer requires; the discipline above applies fully from `v1.0.0` onward. The current `-draft` suffix indicates that even MINOR-numbered identifiers may move before the document stabilizes.

### §0.7. Relationship to `NOTE.md`

SPEC.md is downstream of `NOTE.md`. The note is the authoritative source of clinical reasoning and rationale; SPEC.md formalizes a specific reading of that reasoning.

Two consequences:

1. **Discrepancy resolution.** If SPEC.md and `NOTE.md` disagree on a substantive claim, the note wins by default, and SPEC.md is revised. The exception is when SPEC.md, in attempting to formalize a `NOTE.md` claim, surfaces an ambiguity or contradiction in the prose. In that case, the resolution requires a surgical edit to `NOTE.md` (with corresponding version bump and changelog entry) before SPEC.md proceeds.
2. **Section pinning.** Every formal definition in §2–§5 carries an annotation `[formalizes: NOTE.md §X.Y]` pinning it to a specific section. Reverse mapping is consolidated in §8.

### §0.8. Document structure

| Section | Content | Status in v0.1.0-draft |
|---|---|---|
| §0 | Preliminaries (this section) | Initial draft |
| §1 | Mathematical preliminaries (order theory, Galois connections) | Pending |
| §2 | Patient-state substrate (formalizes `NOTE.md` §4A) | Pending |
| §3 | Institutional-state substrate (formalizes `NOTE.md` §4B) | Pending |
| §4 | Interaction semantics (formalizes `NOTE.md` §4C) | Pending |
| §5 | Temporal evolution (formalizes `NOTE.md` §4D) | Pending |
| §6 | Consolidated proof obligations | Pending |
| §7 | Open formal questions and deferred commitments | Pending |
| §8 | Bidirectional traceability `NOTE.md` ↔ SPEC.md | Pending |

Sections are drafted in order. Each substrate section (§2–§5) is iterated to stability before the next begins. §6 and §8 are constructed incrementally as §2–§5 grow.

---

*End of §0.*
