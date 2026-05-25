# SPEC.md — Substrate-First Clinical AI: Formalization

**Version:** v0.1.0-draft
**Status:** Working draft for scrutiny. Not for citation.
**License:** CC BY 4.0
**Companion to:** `NOTE.md` v0.10.1-draft

---

## §0. Preliminaries

### §0.1. Purpose and audience

This document is the formalization layer between `NOTE.md` (clinical-prose position note) and `ARCHITECTURE.md` (visual architectural overview, to be drafted against this document, not against `NOTE.md` directly).

The audience is engineering and formal-methods readers. Clinical scrutiny of the eighteen load-bearing principles lives in `NOTE.md`. Clinical scrutiny of _whether SPEC.md faithfully formalizes those principles_ depends on the bidirectional traceability in §8 — clinicians are not expected to read this document end-to-end.

SPEC.md exists to:

1. **Disambiguate.** Convert prose-level terms (_lattice_, _refinement_, _soundness_, _abstention_, _joint licensing_, _currency_, _sound by construction_) into named definitions with typed signatures and stated invariants.
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

**MC-1. Hypothesis space as a poset, not a lattice.** Each substrate's hypothesis space is a partially ordered set (P, ⊑). The meet operation ⊓ is **partial**: it is defined for _compatible_ pairs of hypotheses (defined per substrate in §2 and §3) and undefined otherwise. Full lattice structure — total meet and total join — is deferred to a future revision; declaring a total lattice now would require committing to a join operation whose clinical and institutional semantics are not yet clear.

_Rationale._ Real clinical reasoning has hypotheses that do not have a meaningful greatest-lower-bound (e.g., "patient has pneumonia" and "patient has pulmonary embolism" — both can be true, but their meet is not their conjunction in the order-theoretic sense). Forcing a total meet now would either misrepresent that or trivialize it.

**MC-2. Refinement via Galois connection.** The relationship between an abstract hypothesis space `A` and a concrete observation space `C` is given by a Galois connection (α, γ) where α : C → A is the abstraction map and γ : A → C is the concretization map, satisfying the standard adjunction:

∀ c ∈ C, a ∈ A: α(c) ⊑ a ⟺ c ⊑ γ(a)

_Rationale._ The Galois-connection structure makes "operator-set changes are sound by construction" (`NOTE.md` §4D.3) a stateable proof obligation rather than an assertion. Operators defined as abstractions of concrete transitions inherit soundness from the connection, provided the connection itself is preserved.

**MC-3. Abstention as a separate type.** Operators that may abstain do not return a distinguished bottom element of the hypothesis space. They return a value of a separate type:

`Result⟨H, A⟩ = Refined(H) | Abstain(A)`

where `H` is the hypothesis-space type and `A` is the abstention-reason type (defined per substrate).

_Rationale._ Abstention is a first-class output in `NOTE.md` §4A.4, §4B.4, §4C.3. Encoding it as ⊥ ∈ P collapses two distinct epistemic states ("the data refines to no further hypothesis" vs. "the operator declines to commit") into one. The separate-type encoding makes the distinction enforceable by type-checking and makes the abstention reason part of the audit trail.

### §0.4. Notation conventions

The following symbols are reserved throughout SPEC.md.

| Symbol         | Meaning                                    | Notes                                                 |
| -------------- | ------------------------------------------ | ----------------------------------------------------- |
| `⊑`            | Refinement order on a hypothesis space     | `h₁ ⊑ h₂` reads "h₁ refines h₂" (h₁ is more specific) |
| `⊓`            | Partial meet on a hypothesis space         | Defined for compatible pairs; see MC-1                |
| `α`            | Abstraction map of a Galois connection     | `α : C → A`; concrete to abstract                     |
| `γ`            | Concretization map of a Galois connection  | `γ : A → C`; abstract to concrete                     |
| `δ`            | Deduction operator                         | Reserved per substrate in §2, §3                      |
| `Result⟨H, A⟩` | Sum type for possibly-abstaining operators | See MC-3                                              |
| `∎`            | End of a named definition or obligation    |                                                       |

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

| Section | Content                                                       | Status in v0.1.0-draft |
| ------- | ------------------------------------------------------------- | ---------------------- |
| §0      | Preliminaries (this section)                                  | Initial draft          |
| §1      | Mathematical preliminaries (order theory, Galois connections) | Pending                |
| §2      | Patient-state substrate (formalizes `NOTE.md` §4A)            | Pending                |
| §3      | Institutional-state substrate (formalizes `NOTE.md` §4B)      | Pending                |
| §4      | Interaction semantics (formalizes `NOTE.md` §4C)              | Pending                |
| §5      | Temporal evolution (formalizes `NOTE.md` §4D)                 | Pending                |
| §6      | Consolidated proof obligations                                | Pending                |
| §7      | Open formal questions and deferred commitments                | Pending                |
| §8      | Bidirectional traceability `NOTE.md` ↔ SPEC.md                | Pending                |

Sections are drafted in order. Each substrate section (§2–§5) is iterated to stability before the next begins. §6 and §8 are constructed incrementally as §2–§5 grow.

---

_End of §0._

## §1. Mathematical preliminaries

This section fixes the order-theoretic and category-theoretic machinery used throughout §2–§5. Readers familiar with order theory and abstract interpretation can skim and reference back. Standard references: Davey & Priestley, _Introduction to Lattices and Order_ (2002); Cousot & Cousot, "Abstract Interpretation" (POPL 1977).

### §1.1. Partial orders and posets

**DEF-MP-01 (Partial order).** A _partial order_ on a set `P` is a binary relation `⊑ ⊆ P × P` satisfying:

- **Reflexivity:** ∀ x ∈ P. x ⊑ x
- **Antisymmetry:** ∀ x, y ∈ P. (x ⊑ y ∧ y ⊑ x) ⟹ x = y
- **Transitivity:** ∀ x, y, z ∈ P. (x ⊑ y ∧ y ⊑ z) ⟹ x ⊑ z

The pair `(P, ⊑)` is a _poset_. ∎

**DEF-MP-02 (Reading of ⊑).** Throughout SPEC.md, `h₁ ⊑ h₂` reads "h₁ refines h₂" — i.e., `h₁` is at least as specific as `h₂`. The most general hypothesis is the maximum; refinements move downward in the order. This convention matches abstract-interpretation usage (more concrete ⊑ more abstract) and inverts some clinical-reasoning prose where "refining a diagnosis" intuitively moves "up" in specificity. ∎

**DEF-MP-03 (Partial binary operation).** A _partial binary operation_ on `P` is a function `f : D → P` where `D ⊆ P × P` is the _domain of definition_ of `f`. We write `f(x, y) = ⊥_def` (read: "undefined") when `(x, y) ∉ D`. ∎

### §1.2. Partial meets

**DEF-MP-04 (Compatibility predicate).** A _compatibility predicate_ on a poset `(P, ⊑)` is a symmetric reflexive relation `compat ⊆ P × P`. Two elements `x, y ∈ P` are _compatible_ when `compat(x, y)` holds.

The interpretation, per substrate, will be that compatible hypotheses can be meaningfully conjoined; incompatible ones cannot. ∎

**DEF-MP-05 (Partial meet).** A _partial meet_ on `(P, ⊑)` with respect to a compatibility predicate `compat` is a partial binary operation `⊓ : compat → P` such that, for all `x, y ∈ P` with `compat(x, y)`:

- `x ⊓ y ⊑ x`
- `x ⊓ y ⊑ y`
- ∀ z ∈ P. (z ⊑ x ∧ z ⊑ y) ⟹ z ⊑ (x ⊓ y)

That is, `x ⊓ y` is the greatest lower bound of `x` and `y` whenever they are compatible.

**INV-MP-01 (Compatibility necessary for meet).** `⊓` is defined on a pair `(x, y)` if and only if `compat(x, y)`. ∎

### §1.3. Monotone functions

**DEF-MP-06 (Monotone function).** Given posets `(P, ⊑_P)` and `(Q, ⊑_Q)`, a function `f : P → Q` is _monotone_ (or _order-preserving_) iff:

∀ x, y ∈ P. x ⊑_P y ⟹ f(x) ⊑_Q f(y) ∎

**DEF-MP-07 (Antitone function).** A function `f : P → Q` is _antitone_ (or _order-reversing_) iff:

∀ x, y ∈ P. x ⊑_P y ⟹ f(y) ⊑_Q f(x) ∎

### §1.4. Galois connections

**DEF-MP-08 (Galois connection).** Given posets `(C, ⊑_C)` and `(A, ⊑_A)`, a _Galois connection_ between `C` and `A` is a pair of monotone functions `α : C → A` (the _abstraction_) and `γ : A → C` (the _concretization_) satisfying the adjunction:

∀ c ∈ C, a ∈ A. α(c) ⊑_A a ⟺ c ⊑_C γ(a)

We write `(C, α, γ, A)` for the connection. ∎

**INV-MP-02 (Galois connection properties).** Every Galois connection `(C, α, γ, A)` satisfies:

1. `α ∘ γ` is _deflationary_ on `A`: ∀ a ∈ A. α(γ(a)) ⊑_A a.

   _(Standard reference: Cousot & Cousot 1977 use the dual convention where this is inflationary. The choice depends on which side is "abstract." SPEC.md treats `A` as the abstract side; concretizations may lose precision when re-abstracted, hence deflationary on `A`. Verify against application before relying on this orientation.)_

2. `γ ∘ α` is _inflationary_ on `C`: ∀ c ∈ C. c ⊑_C γ(α(c)).
3. `α` preserves all existing joins in `C`; `γ` preserves all existing meets in `A`. ∎

**DEF-MP-09 (Sound abstraction of an operator).** Let `(C, α, γ, A)` be a Galois connection. A function `f_A : A → A` is a _sound abstraction_ of `f_C : C → C` iff:

∀ c ∈ C. α(f_C(c)) ⊑_A f_A(α(c))

Equivalently (and more usefully in practice):

∀ a ∈ A. f_C(γ(a)) ⊑_C γ(f_A(a))

This is the property that lets us reason in `A` and trust that conclusions transfer to `C`. ∎

**INV-MP-03 (Best abstraction).** Given `f_C : C → C` and a Galois connection `(C, α, γ, A)`, the _best abstraction_ of `f_C` is `f_A♯ = α ∘ f_C ∘ γ`. Any sound `f_A` satisfies `f_A♯ ⊑_{A→A} f_A` pointwise. The best abstraction is computable when `α`, `γ`, and `f_C` are; in practice, substrate operators may use sound-but-not-best abstractions for tractability. ∎

### §1.5. Sum and product types

The following type constructors are used throughout. Standard ML/Haskell semantics apply.

**DEF-MP-10 (Product type).** Given types `A` and `B`, the _product type_ `A × B` is the set of pairs `(a, b)` with `a ∈ A`, `b ∈ B`, with projections `π₁ : A × B → A` and `π₂ : A × B → B`. ∎

**DEF-MP-11 (Sum type).** Given types `A` and `B`, the _sum type_ `A | B` (also written `A + B`) is the disjoint union, with injections `inl : A → A | B` and `inr : B → A | B`. We use named constructors throughout: rather than `inl(x)`, we write `Refined(x)`, `Abstain(r)`, etc., with the type definition naming each constructor. ∎

**DEF-MP-12 (Option type).** `Option⟨A⟩ = Some(A) | None`. Standard. ∎

**DEF-MP-13 (Result type for operators).** The operator-result type used throughout SPEC.md:

`Result⟨H, R⟩ = Refined(H) | Abstain(R)`

where `H` is a hypothesis-space type and `R` is an abstention-reason type. This is the encoding from MC-3. The name `Result` is chosen for legibility despite potential collision with Rust's `Result<T, E>`; the semantics differ in that `Abstain` is not an error condition but a valid epistemic state. ∎

### §1.6. Provenance carrier

A construct used by every substrate operator. Provenance is treated abstractly here; concrete encoding belongs in `ARCHITECTURE.md` (excluded per §0.5).

**DEF-MP-14 (Provenance carrier).** A _provenance carrier_ `Prov` is an opaque type satisfying:

- An associative, identity-having composition `· : Prov × Prov → Prov` with identity `ε : Prov`.
- A predicate `derives_from : Prov × Prov → Bool` such that `p₁ derives_from (p₁ · p₂)` and `p₂ derives_from (p₁ · p₂)`.

Concrete provenance encodings (signed event chains, Merkle DAGs, etc.) instantiate `Prov`; SPEC.md depends only on the algebraic interface. ∎

**DEF-MP-15 (Provenance-carrying value).** For any type `T`, the _provenance-carrying_ lifting is `T^P = T × Prov`. Operators that produce values with provenance return `T^P` rather than `T`. ∎

### §1.7. Versioning carrier

A construct used by every substrate component subject to evolution (§5).

**DEF-MP-16 (Version identifier).** A _version identifier_ `Ver` is a totally ordered type with a distinguished initial element `ver₀` and a successor relation. Concrete instantiations (SemVer triples, content hashes, monotonic counters) are out of scope for v0.1.0-draft. ∎

**DEF-MP-17 (Versioned value).** For any type `T`, the _versioned_ lifting is `T^V = T × Ver`. ∎

---

_End of §1._

## §2. Patient-state substrate

Formalizes `NOTE.md` §4A. This section defines the structure within which a single patient's clinical state is represented, refined under observations, and operated on.

### §2.1. Clinical hypothesis space

**DEF-PS-01 (Clinical hypothesis space).** [formalizes: `NOTE.md` §4A.1]

A _clinical hypothesis space_ for a patient is a triple `H_PS = (Hyp, ⊑_PS, compat_PS)` where:

- `Hyp` is a set of _clinical hypotheses_. Concrete instantiation (e.g., as ontology-bounded propositional combinations) is given in §2.2.
- `⊑_PS ⊆ Hyp × Hyp` is a partial order (per DEF-MP-01), the _refinement order_. `h₁ ⊑_PS h₂` reads "h₁ is at least as specific as h₂."
- `compat_PS ⊆ Hyp × Hyp` is a compatibility predicate (per DEF-MP-04). Two hypotheses are compatible when they can be coherently held simultaneously about the same patient at the same time.

`H_PS` is a _partial-meet poset_ in the sense of MC-1: meets are defined only on `compat_PS`-related pairs. ∎

**DEF-PS-02 (Top hypothesis).** [formalizes: `NOTE.md` §4A.1, initial state]

`Hyp` contains a distinguished element `⊤_PS` (read: "any patient state") such that ∀ h ∈ Hyp. h ⊑_PS ⊤_PS. This is the initial hypothesis before any observation. ∎

**INV-PS-01 (Compatibility under refinement).** [formalizes: `NOTE.md` §4A.2, monotonicity of clinical reasoning]

If `h₁ ⊑_PS h₂` and `compat_PS(h₂, h₃)`, then `compat_PS(h₁, h₃)`.

_Reading._ If a hypothesis is compatible with another, every refinement of it is also compatible with that other. Refining toward greater specificity cannot create new incompatibilities — it can only inherit them. ∎

**INV-PS-02 (Meet via refinement).** [formalizes: `NOTE.md` §4A.2]

For compatible `h₁, h₂`, the meet `h₁ ⊓_PS h₂` is the unique (by antisymmetry) most general hypothesis that refines both. Existence is required by DEF-MP-05; uniqueness follows from antisymmetry of `⊑_PS`. ∎

### §2.2. Ontology-bounded hypothesis candidates

**DEF-PS-03 (Ontology-bounded set).** [formalizes: `NOTE.md` §4A.3]

An _ontology-bounded set_ `O` is a finite or recursively enumerable set equipped with:

- A _membership predicate_ `is_member : T → Bool` for the carrier type `T` (decidable).
- A _version identifier_ `ver(O) : Ver` (per DEF-MP-16).
- A _source attribution_ `source(O) : OntologyId` naming the underlying terminology (SNOMED CT, RxNorm, LOINC, ICD-11, or other).

Concrete ontology bindings — which terminologies, at which versions, with what mappings between them — are out of scope per §0.5. SPEC.md depends only on the abstract `OntologyBoundedSet` interface. ∎

**DEF-PS-04 (Hypothesis candidate constraint).** [formalizes: `NOTE.md` §4A.3]

Let `Atom` be an ontology-bounded set of _clinical atoms_ (concept identifiers — diseases, findings, medications, lab observations, anatomical sites). A hypothesis `h ∈ Hyp` is _ontology-bounded_ iff every atomic concept appearing in `h` satisfies `Atom.is_member`.

`Hyp` is constrained so that every `h ∈ Hyp` is ontology-bounded. Hypotheses referencing non-member atoms are not representable in the substrate. ∎

**OBL-PS-01 (Ontology decidability).** [formalizes: `NOTE.md` §4A.3, "no free-form atoms"]

Membership in `Atom` must be decidable in bounded time. Free-text concepts, ad-hoc strings, and tokens not present in `Atom` cannot enter `Hyp` through any path. This is enforced by parsing-stage validation: any input hypothesis is parsed against `Atom` at construction time and rejected if it fails. ∎

### §2.3. Patient observation space and Galois connection

**DEF-PS-05 (Patient observation space).** [formalizes: `NOTE.md` §4A.1, concrete side]

The _patient observation space_ `Obs_PS = (Obs, ⊑_Obs)` is a poset where `Obs` is the set of multisets of _typed clinical observations_ — vital signs, lab values, imaging findings, history elements, medication administrations — each timestamped and provenance-tagged.

`o₁ ⊑_Obs o₂` iff `o₁` contains all observations in `o₂` (and possibly more): the order refines toward more-informed observation states. ∎

**DEF-PS-06 (Patient Galois connection).** [formalizes: `NOTE.md` §4A.1, refinement structure]

The _patient Galois connection_ is `(Obs_PS, α_PS, γ_PS, H_PS)` where:

- `α_PS : Obs → Hyp` maps an observation multiset to the _most refined hypothesis it entails_.
- `γ_PS : Hyp → Obs` maps a hypothesis to the _set of observations compatible with it_.

`(α_PS, γ_PS)` satisfy DEF-MP-08 (Galois adjunction). Existence is asserted as a structural requirement on any concrete instantiation of the substrate. ∎

**OBL-PS-02 (Adjunction soundness).** Any concrete patient substrate must produce `α_PS`, `γ_PS` satisfying:

∀ o ∈ Obs, h ∈ Hyp. α_PS(o) ⊑_PS h ⟺ o ⊑_Obs γ_PS(h)

Violation means the substrate's refinement semantics are inconsistent with its observation semantics. ∎

### §2.4. Deduction operators

**DEF-PS-07 (Patient-substrate operator signature).** [formalizes: `NOTE.md` §4A.2]

A _patient-substrate operator_ is a function

`δ : Hyp × Evidence → Result⟨Hyp, AbstainReason_PS⟩`

where `Evidence` is a typed evidence packet (an element of `Obs` together with provenance — see §2.6), and `AbstainReason_PS` is defined in §2.5.

Operators take a current hypothesis and incoming evidence, and either return a refined hypothesis or abstain with a reason. ∎

**DEF-PS-08 (Soundness of a deduction operator).** [formalizes: `NOTE.md` §4A.2, "sound deduction"]

A deduction operator `δ` is _sound_ iff, for all `h ∈ Hyp` and evidence packets `e` carrying observation `o_e ∈ Obs`:

If `δ(h, e) = Refined(h')`, then `h' ⊑_PS h` _and_ `h' ⊑_PS α_PS(o_e)`.

_Reading._ A sound operator (1) only refines — never generalizes — the current hypothesis, and (2) only produces a refinement that is itself entailed by the evidence's most-refined abstraction. The operator may abstain instead; abstention is never unsound. ∎

**INV-PS-03 (Operator monotonicity).** [formalizes: `NOTE.md` §4A.2]

For any sound `δ` and any `h ∈ Hyp`, `e ∈ Evidence`: `δ(h, e) = Refined(h') ⟹ h' ⊑_PS h`. (This is half of DEF-PS-08, separated as a named invariant because downstream code will check it independently.) ∎

**DEF-PS-09 (Operator set).** [formalizes: `NOTE.md` §4A.2, "a defined family"]

The _operator set_ of a patient substrate is a finite, named, versioned set `Δ_PS = {(name_i, δ_i, ver_i)}` where each `δ_i` is sound (DEF-PS-08) and `ver_i : Ver` identifies the operator's version.

`Δ_PS` is itself versioned: `ver(Δ_PS) : Ver`. Changes to `Δ_PS` follow the discipline in §5 (temporal evolution). ∎

**OBL-PS-03 (Operator set soundness).** [formalizes: `NOTE.md` §4A.2]

Every `δ_i ∈ Δ_PS` satisfies DEF-PS-08. No operator may enter `Δ_PS` without a stated soundness argument (mechanized proof in a later tier; informal argument in v0.1.0-draft). ∎

### §2.5. Abstention semantics

**DEF-PS-10 (Abstention reason).** [formalizes: `NOTE.md` §4A.4]

The patient-substrate abstention type is a sum:

`AbstainReason_PS = `
`InsufficientEvidence(missing: Set⟨RequiredObservation⟩)`
`| OutOfDistribution(detail: OodReport)`
`| AmbiguousRefinement(candidates: Set⟨Hyp⟩, rationale: Prov)`
`| OperatorPreconditionUnmet(operator: OperatorName, condition: PreconditionId)`
`| OntologyOutOfScope(atoms: Set⟨AtomId⟩)`

Each variant carries structured information about _why_ the operator declined to commit. Free-text abstention is not permitted: every abstention is machine-classifiable. ∎

**INV-PS-04 (Abstention is sound).** [formalizes: `NOTE.md` §4A.4]

Abstention never violates DEF-PS-08. An operator returning `Abstain(r)` makes no claim about the patient's state, so soundness is trivially preserved. The only soundness-relevant property of abstention is that the reason `r` is well-formed (`r : AbstainReason_PS` and all carried data satisfies its substructure invariants). ∎

**DEF-PS-11 (Abstention is not bottom).** [formalizes: `NOTE.md` §4A.4, "first-class output"]

`Abstain(r)` is _not_ equivalent to any `Refined(h)` for any `h ∈ Hyp`, including a hypothetical bottom `⊥_PS`. The two epistemic states — "no further refinement is supported" and "I decline to refine" — are encoded by distinct constructors of `Result⟨Hyp, AbstainReason_PS⟩` and cannot be conflated.

(SPEC.md does not commit to whether `Hyp` has a bottom element; if one exists, it represents "a maximally-specific patient state consistent with all observations," which is a different concept from abstention. See §7 for open questions.) ∎

### §2.6. Provenance integration

**DEF-PS-12 (Patient-substrate evidence packet).** [formalizes: `NOTE.md` §4A.5, "auditable provenance"]

An _evidence packet_ is `Evidence = Obs^P` (per DEF-MP-15). Every observation entering a deduction operator carries a provenance carrier identifying its source (device, lab system, clinician input, prior operator output). ∎

**DEF-PS-13 (Operator output with provenance).** [formalizes: `NOTE.md` §4A.5]

A deduction operator's signature is refined from DEF-PS-07 to:

`δ : Hyp^P × Evidence → Result⟨Hyp^P, AbstainReason_PS⟩^P`

That is, the input hypothesis carries provenance, the evidence carries provenance, and the output (whether refined hypothesis or abstention) carries provenance derived from both inputs via `·` (DEF-MP-14).

The refined hypothesis's provenance is `(prov_h · prov_e · op_marker)` where `op_marker : Prov` identifies which operator and operator-version produced this refinement. ∎

**INV-PS-05 (Provenance closure).** [formalizes: `NOTE.md` §4A.5]

Every value in the patient substrate that derives from any operator application carries a provenance composed (via `·`) from the provenances of all inputs and the operator's marker. There is no path by which a value reaches the substrate without provenance: every constructor of `Hyp^P`, `Evidence`, and `Result⟨...⟩^P` requires a `Prov` argument. ∎

**OBL-PS-04 (Provenance auditability).** [formalizes: `NOTE.md` §4A.5]

For any value `v : T^P` in the substrate, the `derives_from` relation (DEF-MP-14) must allow reconstruction of the full derivation chain back to source observations. The substrate must reject any operator whose output provenance fails to satisfy this property. ∎

### §2.7. The learned proposer as a constrained black box

**DEF-PS-14 (Refinement proposer signature).** [formalizes: `NOTE.md` §4A.5, "constrained refinement proposer"]

A _refinement proposer_ is a function

`π : Hyp^P × Evidence → Set⟨Hyp⟩`

returning a finite set of _candidate refinements_ of the current hypothesis. The proposer is the integration point for learned components (LLMs, classifiers, retrieval systems, etc.).

The proposer does **not** decide. Its output is candidate hypotheses; whether any candidate is accepted is determined by the deduction operators in §2.4. ∎

**DEF-PS-15 (Proposer codomain constraint).** [formalizes: `NOTE.md` §4A.5, "constrained"]

The proposer is constrained at its _codomain_ — every element of its output set must satisfy:

1. Be ontology-bounded (DEF-PS-04).
2. Be at most one refinement step from the input hypothesis under `⊑_PS`, where "one step" is defined by the substrate's operator set (precise definition: there exists `δ ∈ Δ_PS` and an evidence packet derived from the input evidence such that `δ` could plausibly produce this refinement).

Candidates failing either constraint are filtered before reaching the deduction operators. Filtering is not the proposer's responsibility; it is the substrate's enforcement boundary. ∎

**INV-PS-06 (Proposer cannot bypass soundness).** [formalizes: `NOTE.md` §4A.5, "constrained"]

The proposer cannot produce a refined hypothesis that becomes the active hypothesis without passing through a sound deduction operator (DEF-PS-08). Even if the proposer is adversarial, the soundness of the active hypothesis depends only on `Δ_PS`, not on `π`.

This is the load-bearing safety property of the patient substrate: **learned-component behavior cannot violate substrate soundness**. ∎

**OBL-PS-05 (Proposer-operator separation).** [formalizes: `NOTE.md` §4A.5]

No code path may insert a value into `Hyp^P` (as the active patient hypothesis) without that value being the `Refined(_)` branch of some sound operator's output. Enforcement is structural: the active-hypothesis type and the proposer-output type are distinct, and only operator results inhabit the former. ∎

### §2.8. Summary of patient-substrate proof obligations

Consolidated from §2.1–§2.7; cross-listed in §6:

- **OBL-PS-01** — Ontology decidability.
- **OBL-PS-02** — Adjunction soundness of `(α_PS, γ_PS)`.
- **OBL-PS-03** — Every operator in `Δ_PS` satisfies DEF-PS-08.
- **OBL-PS-04** — Provenance auditability (full derivation chain reconstructible).
- **OBL-PS-05** — Proposer-operator separation enforced structurally.

These are stated, not discharged. Discharge mechanism (mechanized proof, property-based test, runtime assertion) is an architectural concern.

---

_End of §2._
