# SPEC.md — Substrate-First Clinical AI: Formalization

**Version:** v0.3.0-draft
**Status:** Working draft for scrutiny. Not for citation.
**License:** CC BY 4.0
**Companion to:** `NOTE.md` v0.12.0-draft

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

_Rationale._ Abstention is a first-class output in `NOTE.md` §4A.3, §4B.3, §4C.3. Encoding it as ⊥ ∈ P collapses two distinct epistemic states ("the data refines to no further hypothesis" vs. "the operator declines to commit") into one. The separate-type encoding makes the distinction enforceable by type-checking and makes the abstention reason part of the audit trail.

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

### §0.9. Criticality scheme

SPEC.md inherits the per-principle criticality scheme introduced in `NOTE.md` v0.11.0 §4. The scheme is summarized here for self-containment; the authoritative definitions live in `NOTE.md`.

- **P (Position-critical)** — violation falsifies a substrate, layer, or coupling claim the position itself depends on.
- **S (Safety-property)** — violation breaks a stated safety property within an existing substrate.
- **F (Foundation)** — violation breaks supporting infrastructure other principles depend on.

Each obligation (`OBL-*` in §6) inherits the criticality of the `NOTE.md` §4 principle it most directly supports via its `[formalizes:]` annotation. Where an obligation supports multiple principles, the higher tier (P > S > F) governs. The inheritance is intended to be mechanical: if a `NOTE.md` principle's tier is revised, every SPEC.md obligation traceable to that principle re-inherits the new tier without further commentary.

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

**DEF-PS-03 (Ontology-bounded set).** [formalizes: `NOTE.md` §4A.2]

An _ontology-bounded set_ `O` is a finite or recursively enumerable set equipped with:

- A _membership predicate_ `is_member : T → Bool` for the carrier type `T` (decidable).
- A _version identifier_ `ver(O) : Ver` (per DEF-MP-16).
- A _source attribution_ `source(O) : OntologyId` naming the underlying terminology (SNOMED CT, RxNorm, LOINC, ICD-11, or other).

Concrete ontology bindings — which terminologies, at which versions, with what mappings between them — are out of scope per §0.5. SPEC.md depends only on the abstract `OntologyBoundedSet` interface. ∎

**DEF-PS-04 (Hypothesis candidate constraint).** [formalizes: `NOTE.md` §4A.2]

Let `Atom` be an ontology-bounded set of _clinical atoms_ (concept identifiers — diseases, findings, medications, lab observations, anatomical sites). A hypothesis `h ∈ Hyp` is _ontology-bounded_ iff every atomic concept appearing in `h` satisfies `Atom.is_member`.

`Hyp` is constrained so that every `h ∈ Hyp` is ontology-bounded. Hypotheses referencing non-member atoms are not representable in the substrate. ∎

**OBL-PS-01 (Ontology decidability).** [formalizes: `NOTE.md` §4A.2, "no free-form atoms"]

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

**DEF-PS-10 (Abstention reason).** [formalizes: `NOTE.md` §4A.3]

The patient-substrate abstention type is a sum:

`AbstainReason_PS = `
`InsufficientEvidence(missing: Set⟨RequiredObservation⟩)`
`| OutOfDistribution(detail: OodReport)`
`| AmbiguousRefinement(candidates: Set⟨Hyp⟩, rationale: Prov)`
`| OperatorPreconditionUnmet(operator: OperatorName, condition: PreconditionId)`
`| OntologyOutOfScope(atoms: Set⟨AtomId⟩)`

Each variant carries structured information about _why_ the operator declined to commit. Free-text abstention is not permitted: every abstention is machine-classifiable. ∎

**INV-PS-04 (Abstention is sound).** [formalizes: `NOTE.md` §4A.3]

Abstention never violates DEF-PS-08. An operator returning `Abstain(r)` makes no claim about the patient's state, so soundness is trivially preserved. The only soundness-relevant property of abstention is that the reason `r` is well-formed (`r : AbstainReason_PS` and all carried data satisfies its substructure invariants). ∎

**DEF-PS-11 (Abstention is not bottom).** [formalizes: `NOTE.md` §4A.3, "first-class output"]

`Abstain(r)` is _not_ equivalent to any `Refined(h)` for any `h ∈ Hyp`, including a hypothetical bottom `⊥_PS`. The two epistemic states — "no further refinement is supported" and "I decline to refine" — are encoded by distinct constructors of `Result⟨Hyp, AbstainReason_PS⟩` and cannot be conflated.

(SPEC.md does not commit to whether `Hyp` has a bottom element; if one exists, it represents "a maximally-specific patient state consistent with all observations," which is a different concept from abstention. See §7 for open questions.) ∎

### §2.6. Provenance integration

**DEF-PS-12 (Patient-substrate evidence packet).** [formalizes: `NOTE.md` §4A.4, "auditable provenance"]

An _evidence packet_ is `Evidence = Obs^P` (per DEF-MP-15). Every observation entering a deduction operator carries a provenance carrier identifying its source (device, lab system, clinician input, prior operator output). ∎

**DEF-PS-13 (Operator output with provenance).** [formalizes: `NOTE.md` §4A.4]

A deduction operator's signature is refined from DEF-PS-07 to:

`δ : Hyp^P × Evidence → Result⟨Hyp^P, AbstainReason_PS⟩^P`

That is, the input hypothesis carries provenance, the evidence carries provenance, and the output (whether refined hypothesis or abstention) carries provenance derived from both inputs via `·` (DEF-MP-14).

The refined hypothesis's provenance is `(prov_h · prov_e · op_marker)` where `op_marker : Prov` identifies which operator and operator-version produced this refinement. ∎

**INV-PS-05 (Provenance closure).** [formalizes: `NOTE.md` §4A.4]

Every value in the patient substrate that derives from any operator application carries a provenance composed (via `·`) from the provenances of all inputs and the operator's marker. There is no path by which a value reaches the substrate without provenance: every constructor of `Hyp^P`, `Evidence`, and `Result⟨...⟩^P` requires a `Prov` argument. ∎

**OBL-PS-04 (Provenance auditability).** [formalizes: `NOTE.md` §4A.4]

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

## §3. Institutional-state substrate

Formalizes `NOTE.md` §4B. This section defines the structure within which an institution's resource-allocation state is represented, updated under operational events, and operated on. The structure parallels §2 by design — `NOTE.md` §4B mirrors §4A in form — and shared abstractions from §1 are reused. The mirror is in form, not in substance: `NOTE.md` v0.12.0 §4B intro names the structural asymmetry explicitly (institutional refinement is additionally bounded by physical resource availability, with no patient analog), and that asymmetry surfaces at the formal level in DEF-IS-04 (physical capacity bound), OBL-IS-01 (physical-validity preservation), and the `PhysicalValidityWouldBeViolated` variant of DEF-IS-10 (institutional abstention reason).

Three structural differences from §2 are load-bearing and surfaced as they arise:

- The institutional substrate is **one per institution**, shared across patients (versus per-patient for §2).
- Its state is subject to **hard physical constraints** (bed counts, formulary supply, lab cycle times) that are not negotiable by refinement.
- Its observation space carries **resource events** (admissions, discharges, deliveries, shift changes) rather than per-patient clinical observations.

### §3.1. Institutional state space

**DEF-IS-01 (Institutional state space).** [formalizes: `NOTE.md` §4B.1]

An _institutional state space_ is a triple `H_IS = (Cap, ⊑_IS, compat_IS)` where:

- `Cap` is a set of _capacity hypotheses_. A capacity hypothesis is a constrained assignment of resources to potential commitments: bed allocations, formulary holds, lab-queue positions, on-call assignments. Concrete representation in §3.2.
- `⊑_IS` is a partial order on `Cap`. `c₁ ⊑_IS c₂` reads "c₁ is at least as committed as c₂" — more allocations made, fewer degrees of freedom remaining.
- `compat_IS` is a compatibility predicate. Two capacity hypotheses are compatible iff their union does not exceed any physical capacity bound (DEF-IS-04).

`H_IS` is a partial-meet poset (MC-1). Meets correspond to consolidating two compatible allocation views. ∎

**DEF-IS-02 (Top institutional hypothesis).** [formalizes: `NOTE.md` §4B.1]

`Cap` contains a distinguished element `⊤_IS` (read: "fully uncommitted") such that ∀ c ∈ Cap. c ⊑_IS ⊤_IS. This represents a hypothetical state with all resources available; it is the upper reference point, never the actual institutional state in operation. ∎

**INV-IS-01 (Compatibility under refinement).** [formalizes: `NOTE.md` §4B.2]

If `c₁ ⊑_IS c₂` and `compat_IS(c₂, c₃)`, then `compat_IS(c₁, c₃)`. Mirrors INV-PS-01. ∎

**INV-IS-02 (Meet via refinement).** For compatible `c₁, c₂`, the meet `c₁ ⊓_IS c₂` is the unique most-uncommitted capacity hypothesis that is at least as committed as both. Mirrors INV-PS-02. ∎

### §3.2. Scope-bounded resources and physical capacity bounds

**DEF-IS-03 (Resource-bounded set).** [formalizes: `NOTE.md` §4B.2]

A _resource-bounded set_ `R` is the institutional analog of the ontology-bounded set (DEF-PS-03). It is an `OntologyBoundedSet`-typed enumeration of:

- Physical resource units (bed identifiers, ICU bays, OR rooms, ventilators).
- Consumable resource classes (formulary items, lab reagents).
- Time-divisible resource slots (lab queue positions, OR block hours, on-call shifts).
- Personnel role assignments (specialist coverage, nursing ratios).

Each resource type carries a version identifier and source attribution. Concrete bindings to specific hospital information systems are out of scope per §0.5. ∎

**DEF-IS-04 (Physical capacity bound).** [formalizes: `NOTE.md` §4B.1, "hard physical limits"]

A _physical capacity bound_ is a function `cap : R → ℕ` (or `ℕ ∪ {∞}` for unbounded resources) giving the maximum simultaneous instances of each resource available.

`cap` is itself versioned (`ver(cap) : Ver`) and changes only through operator-mediated updates (§3.4) — for example, when a bay opens, a unit closes, or formulary supply is delivered.

A capacity hypothesis `c ∈ Cap` is _physically valid_ iff its committed-resource count for each `r ∈ R` does not exceed `cap(r)`. ∎

**OBL-IS-01 (Physical validity preservation).** [formalizes: `NOTE.md` §4B.1, §4B.2]

Every operator in §3.4 must preserve physical validity: applying an operator to a physically-valid input cannot produce a physically-invalid output. Operators that would violate `cap` must abstain (DEF-IS-10). ∎

**OBL-IS-02 (Resource decidability).** [formalizes: `NOTE.md` §4B.2]

Membership in `R` is decidable. Free-form resource identifiers cannot enter `Cap`. ∎

### §3.3. Institutional event space and Galois connection

**DEF-IS-05 (Institutional event space).** [formalizes: `NOTE.md` §4B.1, concrete side]

The _institutional event space_ `Evt_IS = (Evt, ⊑_Evt)` is a poset where `Evt` is the set of timestamped, provenance-tagged _operational events_ — admissions, discharges, transfers, supply deliveries, shift changes, allocation requests, allocation releases.

`e₁ ⊑_Evt e₂` iff `e₁`'s event multiset is a superset of `e₂`'s: more events observed implies a more-refined event history. ∎

**DEF-IS-06 (Institutional Galois connection).** [formalizes: `NOTE.md` §4B.1]

The _institutional Galois connection_ is `(Evt_IS, α_IS, γ_IS, H_IS)` where:

- `α_IS : Evt → Cap` maps an event history to the most-committed capacity hypothesis it entails.
- `γ_IS : Cap → Evt` maps a capacity hypothesis to the set of event histories consistent with it.

Satisfies DEF-MP-08. ∎

**OBL-IS-03 (Institutional adjunction soundness).** Mirrors OBL-PS-02.

∀ e ∈ Evt, c ∈ Cap. α_IS(e) ⊑_IS c ⟺ e ⊑_Evt γ_IS(c) ∎

### §3.4. Capacity-update operators

**DEF-IS-07 (Capacity-update operator signature).** [formalizes: `NOTE.md` §4B.2]

A _capacity-update operator_ is a function

`δ_IS : Cap × InstEvidence → Result⟨Cap, AbstainReason_IS⟩`

where `InstEvidence` is a typed evidence packet over the institutional event space (an element of `Evt` with provenance), and `AbstainReason_IS` is defined in §3.5. ∎

**DEF-IS-08 (Soundness of a capacity-update operator).** [formalizes: `NOTE.md` §4B.2, "sound"]

A capacity-update operator `δ_IS` is _sound_ iff, for all `c ∈ Cap` and evidence packets `e_inst` carrying event multiset `evt ∈ Evt`:

If `δ_IS(c, e_inst) = Refined(c')`, then:

1. `c' ⊑_IS c` (operator only refines — only commits, never un-commits without a corresponding event).
2. `c' ⊑_IS α_IS(evt)` (refinement is at most as committed as the events justify).
3. `c'` is physically valid (OBL-IS-01). ∎

**INV-IS-03 (Capacity-update monotonicity).** [formalizes: `NOTE.md` §4B.2]

For any sound `δ_IS`: `δ_IS(c, e) = Refined(c') ⟹ c' ⊑_IS c`. Half of DEF-IS-08, called out separately. ∎

**DEF-IS-09 (Institutional operator set).** [formalizes: `NOTE.md` §4B.2]

The _institutional operator set_ is a finite, named, versioned set `Δ_IS = {(name_i, δ_IS_i, ver_i)}` where each operator satisfies DEF-IS-08.

`Δ_IS` is itself versioned. Changes follow §5. ∎

**OBL-IS-04 (Institutional operator-set soundness).** Mirrors OBL-PS-03. Every `δ_IS_i ∈ Δ_IS` satisfies DEF-IS-08. ∎

### §3.5. Allocation abstention

**DEF-IS-10 (Institutional abstention reason).** [formalizes: `NOTE.md` §4B.3]

`AbstainReason_IS = `
`CapacityExceeded(resource: R, demand: ℕ, available: ℕ)`
`| DemandUncertain(forecast_ci: ConfidenceInterval, threshold_breached: ThresholdId)`
`| AllocationContested(stakeholders: Set⟨StakeholderId⟩, rationale: Prov)`
`| EventOutOfScope(event_ids: Set⟨EventId⟩)`
`| OperatorPreconditionUnmet(operator: OperatorName, condition: PreconditionId)`
`| PhysicalValidityWouldBeViolated(violations: Set⟨ResourceBreach⟩)`

Mirrors DEF-PS-10 in structure: every abstention is machine-classifiable, no free-text reasons. The `PhysicalValidityWouldBeViolated` variant is institution-specific — it has no counterpart in `AbstainReason_PS` because the patient substrate has no equivalent hard physical bound. ∎

**INV-IS-04 (Institutional abstention is sound).** Mirrors INV-PS-04. ∎

**DEF-IS-11 (Allocation abstention is not stalling).** [formalizes: `NOTE.md` §4B.3, "first-class output"]

`Abstain(r)` for an allocation decision means the substrate has explicitly produced a non-decision, with a machine-readable reason and a provenance trail. It is _not_ the same as the operator timing out, crashing, or silently producing a default allocation. The substrate guarantees that every allocation request yields either `Refined(c')` or `Abstain(r)` in bounded steps — never silent failure. (Bounded-time guarantees themselves are an architectural concern; here we require only that the abstention path is structurally reachable.) ∎

### §3.6. Provenance integration

The provenance machinery is shared with the patient substrate. Section §2.6 definitions apply with substrate-appropriate type substitutions; only delta-relevant statements appear here.

**DEF-IS-12 (Institutional evidence packet).** [formalizes: `NOTE.md` §4B.4] Mirrors DEF-PS-12: `InstEvidence = Evt^P`. Every event entering an operator carries provenance. ∎

**DEF-IS-13 (Institutional operator output with provenance).** [formalizes: `NOTE.md` §4B.4] Mirrors DEF-PS-13:

`δ_IS : Cap^P × InstEvidence → Result⟨Cap^P, AbstainReason_IS⟩^P` ∎

**INV-IS-05 (Institutional provenance closure).** [formalizes: `NOTE.md` §4B.4] Mirrors INV-PS-05. ∎

**OBL-IS-05 (Institutional provenance auditability).** [formalizes: `NOTE.md` §4B.4] Mirrors OBL-PS-04. ∎

### §3.7. The capacity-learned proposer

**DEF-IS-14 (Capacity-learned proposer signature).** [formalizes: `NOTE.md` §4B.5, "demand forecasters, queue dynamics predictors"]

A _capacity-learned proposer_ is a function

`π_IS : Cap^P × InstEvidence → Set⟨Cap⟩`

returning a finite set of candidate capacity refinements. The proposer integrates learned components: demand forecasters, queue-dynamics predictors, length-of-stay regressors, no-show classifiers.

As in DEF-PS-14, the proposer does **not** decide — it produces candidates for the capacity-update operators in §3.4 to evaluate. ∎

**DEF-IS-15 (Institutional proposer codomain constraint).** [formalizes: `NOTE.md` §4B.5, "constrained"]

Every candidate `c ∈ π_IS(...)` must satisfy:

1. Resource-boundedness (DEF-IS-03): all referenced resources are members of `R`.
2. Physical validity under current `cap` (DEF-IS-04).
3. At-most-one-step refinement under `⊑_IS` (defined analogously to DEF-PS-15.2: there exists `δ_IS ∈ Δ_IS` that could plausibly produce this refinement from the input evidence).

Candidates failing any constraint are filtered before reaching the operators. ∎

**INV-IS-06 (Institutional proposer cannot bypass soundness).** [formalizes: `NOTE.md` §4B.5, "constrained"]

The active institutional state cannot be modified except through a sound capacity-update operator. Even if `π_IS` is adversarial, the soundness and physical validity of `Cap^P` depend only on `Δ_IS`, not on `π_IS`. Mirrors INV-PS-06. ∎

**OBL-IS-06 (Institutional proposer-operator separation).** Mirrors OBL-PS-05. ∎

### §3.8. Summary of institutional-substrate proof obligations

Consolidated from §3.1–§3.7; cross-listed in §6:

- **OBL-IS-01** — Physical validity preservation across operator application.
- **OBL-IS-02** — Resource decidability.
- **OBL-IS-03** — Adjunction soundness of `(α_IS, γ_IS)`.
- **OBL-IS-04** — Every operator in `Δ_IS` satisfies DEF-IS-08.
- **OBL-IS-05** — Institutional provenance auditability.
- **OBL-IS-06** — Institutional proposer-operator separation enforced structurally.

The institutional substrate inherits the patient substrate's separation discipline (proposer vs. operator) and adds one institution-specific obligation (OBL-IS-01, physical validity). All six are stated, not discharged.

---

_End of §3._

## §4. Interaction semantics

Formalizes `NOTE.md` §4C. This section defines how the patient substrate (§2) and the institutional substrate (§3) communicate without one being subordinated to the other. Three principles from §4C are formalized: a typed cross-layer event interface, joint licensing of recommendations, and joint abstention as a first-class output.

This is the section where the residual originality claim from `NOTE.md` §6 — coupling of patient and institutional substrates via an explicit event interface with joint licensing and joint abstention — lives. The formal commitments below are accordingly more committal than in §2 and §3.

### §4.1. Cross-layer event interface

**DEF-IX-01 (Cross-layer event).** [formalizes: `NOTE.md` §4C.2]

A _cross-layer event_ is a tagged value:

`CrossLayerEvent = `
`PatientToInstitutional(p_evidence: Evidence, derived: AllocationImpact)`
`| InstitutionalToPatient(i_evidence: InstEvidence, derived: PatientImpact)`
`| Coupled(p_evidence: Evidence, i_evidence: InstEvidence, link: CouplingId)`

Each variant carries the originating substrate's evidence and a _derivation_ into the receiving substrate's evidence space.

`PatientToInstitutional` is produced when a refinement in `H_PS` implies an allocation impact (e.g., refining to "needs mechanical ventilation" entails a ventilator allocation request).

`InstitutionalToPatient` is produced when a capacity update implies a patient-level reevaluation (e.g., a closing ICU bay triggers reassessment of patient-level disposition recommendations).

`Coupled` carries simultaneously-occurring events that affect both substrates (e.g., an admission event refines both the patient hypothesis and the bed allocation). ∎

**DEF-IX-02 (Allocation impact derivation).** [formalizes: `NOTE.md` §4C.2]

For a `PatientToInstitutional` event, the derivation function

`derive_alloc : Evidence × Hyp^P → InstEvidence`

maps patient evidence and the current patient hypothesis to the institutional evidence it implies. `derive_alloc` is total, deterministic, and ontology-bounded on both sides.

The dual `derive_patient : InstEvidence × Cap^P → Evidence` is similarly required for `InstitutionalToPatient` events. ∎

**INV-IX-01 (Derivation respects bounding).** [formalizes: `NOTE.md` §4C.2]

`derive_alloc` produces only resource-bounded `InstEvidence` (per DEF-IS-03). `derive_patient` produces only ontology-bounded `Evidence` (per DEF-PS-04). Cross-layer derivations cannot smuggle un-bounded atoms or resources into either substrate. ∎

**OBL-IX-01 (Derivation soundness).** [formalizes: `NOTE.md` §4C.2]

The derivation functions must be consistent with the substrate Galois connections:

∀ e ∈ Evidence, h ∈ Hyp. α_IS(derive_alloc(e, h)) ⊑_IS some_explicit_bound(α_PS(e), h)

where the right-hand side names what allocation impact the patient evidence can entail. Concrete form of `some_explicit_bound` is part of the operator-set design; the obligation here is that _some_ explicit bound is stated, not implicit. ∎

### §4.2. Composite state and joint operator signature

**DEF-IX-03 (Composite substrate state).** [formalizes: `NOTE.md` §4C, "coupled two-layer"]

The _composite substrate state_ is the product

`S = Hyp^P × Cap^P`

with componentwise refinement: `(h₁, c₁) ⊑_S (h₂, c₂)` iff `h₁ ⊑_PS h₂` and `c₁ ⊑_IS c₂`.

Both components carry provenance independently. The composite is not itself a poset with new structure — it is the product poset — but it is the structural unit on which joint operators act. ∎

**DEF-IX-04 (Joint operator signature).** [formalizes: `NOTE.md` §4C.1]

A _joint operator_ is a function

`δ_J : S × CrossLayerEvent → Result⟨S, AbstainReason_J⟩`

where `AbstainReason_J` is defined in §4.4. ∎

**DEF-IX-05 (Joint operator decomposition).** [formalizes: `NOTE.md` §4C.1, "explicit"]

Every joint operator decomposes into a triple `(δ_PS', δ_IS', coupling_check)` where:

- `δ_PS' ∈ Δ_PS` is the patient-substrate operator that produces the patient-component output.
- `δ_IS' ∈ Δ_IS` is the institutional-substrate operator that produces the institutional-component output.
- `coupling_check : (Hyp × Cap) → Bool` is a decidable predicate verifying that the joint output satisfies cross-layer consistency.

A joint operator `δ_J` is not an opaque function but a structured composition of substrate operators with an explicit coupling check. ∎

### §4.3. Joint licensing

**DEF-IX-06 (Licensed refinement).** [formalizes: `NOTE.md` §4C.1, "joint licensing"]

A refinement of composite state `(h, c) ⟶ (h', c')` is _jointly licensed_ iff:

1. `δ_PS'(h, e_PS) = Refined(h')` for some `δ_PS' ∈ Δ_PS` and some patient evidence `e_PS` extracted from the cross-layer event (DEF-PS-08 satisfied).
2. `δ_IS'(c, e_IS) = Refined(c')` for some `δ_IS' ∈ Δ_IS` and some institutional evidence `e_IS` extracted from the same cross-layer event (DEF-IS-08 satisfied).
3. `coupling_check(h', c') = true`.

All three are required. Failing any one produces an abstention (§4.4), not a partial refinement. ∎

**INV-IX-02 (Licensing is monotone).** [formalizes: `NOTE.md` §4C.1]

If `(h, c) ⟶ (h', c')` is licensed and `(h', c') ⊑_S (h'', c'')` is also licensed, then `(h, c) ⟶ (h'', c'')` is licensed. Composition of licensed refinements is licensed. ∎

**DEF-IX-07 (Coupling-check soundness).** [formalizes: `NOTE.md` §4C.1]

A coupling check `coupling_check` is _sound_ iff its `true` outputs correspond to states in which both substrates can simultaneously satisfy their respective soundness obligations under the same evidence interpretation. Equivalent statement: a sound coupling check never returns `true` on a state pair that one substrate's view considers inconsistent with the cross-layer event.

Concrete coupling checks (e.g., "the patient hypothesis 'requires ICU' implies the institutional state has at least one allocated ICU bed for this patient") instantiate this; SPEC.md v0.1.0 requires only the structural property. ∎

**OBL-IX-02 (Coupling-check soundness obligation).** Every coupling check used in a joint operator must be shown sound per DEF-IX-07. ∎

### §4.4. Joint abstention with structured diff

The most committal section of SPEC.md v0.1.0-draft. This formalizes the property that, when patient-locally-optimal diverges from institutionally-feasible, the substrate produces _both_ with explicit diff rather than silently downgrading to the feasible one.

**DEF-IX-08 (Joint abstention reason).** [formalizes: `NOTE.md` §4C.3]

`AbstainReason_J = `
`PatientOnly(r_PS: AbstainReason_PS)`
`| InstitutionalOnly(r_IS: AbstainReason_IS)`
`| Divergent(diff: SubstrateDiff)`
`| CouplingViolated(check: CouplingId, witness: Prov)`

The `PatientOnly` and `InstitutionalOnly` variants are lifts of the substrate-local abstention reasons — one substrate would have refined, the other declined. The novel variants are `Divergent` and `CouplingViolated`. ∎

**DEF-IX-09 (Substrate diff).** [formalizes: `NOTE.md` §4C.3, "explicit diff"]

A _substrate diff_ is a structured record:

`SubstrateDiff = {`
`patient_locally_optimal: Hyp^P,`
`institutional_constraint: Cap^P,`
`infeasibility_reason: AbstainReason_IS,`
`alternative_feasible_option: Option⟨(Hyp^P, Cap^P)⟩,`
`divergence_provenance: Prov`
`}`

When the patient substrate would refine to `patient_locally_optimal` but no jointly-licensed refinement exists at that hypothesis, the `Divergent` variant carries:

- The hypothesis the patient substrate considers most refined under the evidence.
- The institutional constraint that prevents joint licensing.
- The machine-classifiable reason for the institutional refusal.
- A (possibly absent) alternative composite state that _would_ be jointly licensed under the institutional constraint — this is the institutionally-actionable plan, not a substrate-selected choice; it is the input the clinician sees alongside the patient-locally-optimal hypothesis.
- A provenance carrier reconstructing how the divergence arose. ∎

**INV-IX-03 (Joint abstention is not downgrade).** [formalizes: `NOTE.md` §4C.3, "rather than silently downgrading"]

When `δ_J((h, c), evt) = Abstain(Divergent(diff))`:

1. The composite state is _not_ updated. Neither the patient component nor the institutional component changes as a side effect of computing the diff.
2. The diff is the substrate's output, exposed to the clinician (or the human decision-making interface) without selection or ranking by the substrate itself.
3. If the substrate proceeds to apply any subsequent operator, it does so against the unchanged `(h, c)`, not against `alternative_feasible_option` (if present).

Equivalent statement: divergent joint abstention is observably distinct from picking the locally-feasible option. A black-box test cannot conflate the two. ∎

**DEF-IX-10 (Joint abstention is not stalling).** Mirrors DEF-IS-11 for joint operators. `Abstain(r)` is structurally distinct from timeout, crash, or silent defaulting. ∎

**OBL-IX-03 (No silent downgrade).** [formalizes: `NOTE.md` §4C.3]

There is no code path through any joint operator whose net effect is "the patient substrate's preferred refinement is suppressed and an institutionally-feasible refinement is applied" without an explicit `Divergent(diff)` abstention being produced and the composite state remaining unchanged through the abstention.

This is the load-bearing safety property of §4. It is the property that operationalizes the `NOTE.md` §4C.3 claim. ∎

### §4.5. Substrate independence preserved

A property worth stating explicitly because it's load-bearing for the architecture's modular review.

**INV-IX-04 (Substrate-local soundness is independent).** [formalizes: `NOTE.md` §4C, structural independence]

Removing all `δ_J` joint operators from the system leaves §2 and §3 still sound as independent substrates. The interaction layer adds licensing and joint-abstention discipline; it does not modify the soundness conditions of `Δ_PS` or `Δ_IS`.

_Reading._ A reviewer can audit the patient substrate independently of the institutional substrate and vice versa. Cross-layer coupling is structurally additive — it constrains which composite refinements are licensed but cannot make a substrate-local operator unsound. ∎

### §4.6. Summary of interaction-substrate proof obligations

Consolidated from §4.1–§4.5; cross-listed in §6:

- **OBL-IX-01** — Cross-layer derivation soundness against substrate Galois connections.
- **OBL-IX-02** — Coupling-check soundness for every coupling used in a joint operator.
- **OBL-IX-03** — No silent downgrade: divergent abstention is observably distinct from feasibility-selection.

Three obligations, matching the three principles of `NOTE.md` §4C. The load-bearing one is OBL-IX-03 — it is the obligation that the joint-abstention claim from `NOTE.md` §4C.3 ultimately rests on.

---

_End of §4._

## §5. Temporal evolution

Formalizes `NOTE.md` §4D. This section defines how substrate components change over time without violating soundness, and how active substrate state relates to operator versions under which it was derived. Five principles from §4D are formalized: substrate-component versioning, evidence currency as a first-class signal, operator-set changes that are sound by construction, clinician-mediated propagation, and evolution-aware provenance.

The interesting formal work in §5 is in §5.3 (operator-set evolution) and §5.4 (clinician-mediated propagation). Both are load-bearing for the `NOTE.md` §4D claim that the substrate forbids silent drift structurally.

### §5.1. Substrate component versioning

**DEF-TE-01 (Versioned substrate component).** [formalizes: `NOTE.md` §4D.1]

A _versioned substrate component_ is any of:

- An operator set: `Δ_PS^V`, `Δ_IS^V`, lifted via DEF-MP-17.
- An individual operator: `(δ, ver(δ))`.
- An ontology-bounded set: `Atom^V`, `R^V` (per DEF-PS-03, DEF-IS-03, both already carrying `ver`).
- A capacity bound: `cap^V` (per DEF-IS-04).
- A coupling check: `coupling_check^V`.
- A derivation function: `derive_alloc^V`, `derive_patient^V`.

Every component identifier above is required to carry a `Ver`. There is no path through which a substrate operation depends on an un-versioned component. ∎

**INV-TE-01 (Version closure).** [formalizes: `NOTE.md` §4D.1]

For any operator application that produces a value `v : T^P`, the resulting provenance `prov(v)` records the versions of every substrate component consulted in producing `v`: operator version, operator-set version, ontology version, and (for institutional operators) capacity-bound version.

Reconstruction of `v`'s derivation from `prov(v)` yields the exact component versions under which `v` was produced, not merely the components by identity. ∎

### §5.2. Evidence currency

**DEF-TE-02 (Evidence currency carrier).** [formalizes: `NOTE.md` §4D.2, "currency as first-class signal"]

Every evidence packet `e : Evidence` (and `e : InstEvidence`) carries a _currency carrier_:

`Currency = { captured_at: Timestamp, expires_at: Option⟨Timestamp⟩, freshness_class: FreshnessClass }`

where `FreshnessClass` is an enumeration (`Realtime`, `Recent`, `Stale`, `Historical`) defined per evidence type. A vital-sign reading classified `Realtime` 30 seconds ago is `Recent` 5 minutes later; a chest X-ray classified `Recent` is `Historical` 48 hours later. The transition thresholds are part of the substrate's configuration. ∎

**DEF-TE-03 (Currency-aware operator signature).** [formalizes: `NOTE.md` §4D.2]

Operator signatures are refined: every operator receives currency-annotated evidence and is required to make currency a decidable input to its abstention logic.

`δ : Hyp^P × (Evidence × Currency) → Result⟨Hyp^P, AbstainReason_PS⟩^P`

Operators may abstain with reason `InsufficientEvidence` (lifted to include a `currency_inadequate` subvariant) when the evidence's freshness class is below the operator's threshold. ∎

**INV-TE-02 (Currency is not silent).** [formalizes: `NOTE.md` §4D.2]

If an operator refines under evidence whose currency would not have permitted refinement under stricter freshness thresholds, the operator's output provenance records the currency at refinement time. A subsequent audit can identify refinements that were licensed only by lenient currency thresholds. ∎

### §5.3. Sound operator-set evolution

This is the formal content of "operator-set changes are sound by construction" (`NOTE.md` §4D.3). The structure draws on the Galois machinery in §1.4: operator-set changes are framed as transitions between abstract interpretations of the same underlying concrete semantics.

**DEF-TE-04 (Operator-set transition).** [formalizes: `NOTE.md` §4D.3]

An _operator-set transition_ is a pair `(Δ_PS_old, Δ_PS_new)` (or the institutional analog) related by a version successor: `ver(Δ_PS_new) > ver(Δ_PS_old)`.

A transition is _admissible_ iff there exists a _transition justification_ — a structured record specifying, for each operator `δ_new ∈ Δ_PS_new`, exactly one of:

1. **Carried forward:** `δ_new = δ_old` for some `δ_old ∈ Δ_PS_old`. No re-justification required.
2. **Newly introduced:** `δ_new ∉ Δ_PS_old`. An explicit soundness argument (DEF-PS-08) is included.
3. **Replacing:** `δ_new` replaces a specific `δ_old ∈ Δ_PS_old`. The justification includes both an independent soundness argument for `δ_new` and a _comparability statement_ relating `δ_new`'s output behavior to `δ_old`'s on the shared input domain. ∎

**DEF-TE-05 (Comparability statement).** [formalizes: `NOTE.md` §4D.3]

A comparability statement for a replacing operator is one of:

- **Strict refinement:** `∀ h, e. δ_new(h, e) refines-or-equals δ_old(h, e)` under a stated order on `Result⟨...⟩`.
- **Strict generalization:** the dual.
- **Incomparable:** the new operator's refinements are not order-related to the old one's, with a stated rationale for why the change is nevertheless clinically motivated. **Incomparable transitions carry an additional justification burden** (per `NOTE.md` §4D.3): the rationale must be explicit, reviewable, and not absorbed into the operator-version diff. Specifically, an `Incomparable` transition justification must state (a) which clinical judgment the new operator embodies that the old did not, (b) why that judgment supersedes the prior one rather than coexists with it, and (c) the evidence base anchoring the new judgment. A `Refinement` or `Generalization` transition does not require these three elements (the order-theoretic relationship to the prior operator carries part of the justification structurally); an `Incomparable` transition does, because the order-theoretic relationship is absent.

`Incomparable` transitions are permitted but require explicit acknowledgement. They are the case where the new operator embodies a genuinely different clinical judgment, not a sharpening or softening of the old one (worked example in `NOTE.md` §7E.6, the SSC 2021 → 2026 transition). ∎

**OBL-TE-01 (Transition admissibility).** [formalizes: `NOTE.md` §4D.3]

No operator-set version `Δ_PS_new` may be activated without a stored, signed transition justification (DEF-TE-04) relative to its predecessor `Δ_PS_old`. The substrate refuses to load an operator-set version lacking a transition justification. ∎

**INV-TE-03 (Transition does not retroactively change active state).** [formalizes: `NOTE.md` §4D.3]

The activation of `Δ_PS_new` does not modify any active patient hypothesis whose provenance points to `Δ_PS_old`. Active hypotheses retain their provenance pointing to the operator-set version under which they were derived; the new operator set governs only future deductions (subject to §5.4). ∎

### §5.4. Clinician-mediated propagation

**DEF-TE-06 (Re-review event).** [formalizes: `NOTE.md` §4D.4]

When an operator-set transition replaces an operator `δ_old`, the substrate emits a _re-review event_ for every active hypothesis `h^P` whose provenance includes a deduction step through `δ_old`:

`ReReviewEvent = { active_hypothesis: Hyp^P, old_operator: OperatorName × Ver, new_operator: OperatorName × Ver, comparability: ComparabilityStatement, status: ReReviewStatus }`

with `ReReviewStatus = Pending | ResolvedKeep | ResolvedReplace(new_hyp: Hyp^P, clinician: PrincipalId)`. ∎

**DEF-TE-06b (Institutional re-review event).** [formalizes: `NOTE.md` §4D.4, institutional symmetry paragraph]

When an operator-set transition replaces an institutional capacity-update operator `δ_IS_old`, the substrate emits an _institutional re-review event_ for every active capacity hypothesis `c^P` whose provenance includes an allocation through `δ_IS_old`:

`InstReReviewEvent = { active_capacity: Cap^P, old_operator: OperatorName × Ver, new_operator: OperatorName × Ver, comparability: ComparabilityStatement, authority_class: AuthorityClass, status: ReReviewStatus }`

where `AuthorityClass ∈ { CapacityManager, EthicsCommittee, FormularyCommittee, ... }` names the institutional authority empowered to resolve this class of re-review (capacity-policy revisions resolve via capacity managers; scarcity-allocation framework revisions via ethics committee; formulary-restriction revisions via formulary committee; the enumeration is per-institution).

The institutional re-review path is structurally identical to DEF-TE-06's patient path: the same `ReReviewStatus` lifecycle, the same provenance discipline, the same prohibition on automatic replacement. The only structural difference is `authority_class`, which routes the event to the appropriate institutional resolver rather than to a bedside clinician. ∎

**INV-TE-04 (No automatic replacement).** [formalizes: `NOTE.md` §4D.4, "silent drift is structurally forbidden" and "unconditional on transition type"]

For an active patient hypothesis `h^P` with a pending re-review event (DEF-TE-06), _and_ for an active institutional capacity hypothesis `c^P` with a pending institutional re-review event (DEF-TE-06b):

1. The substrate does not automatically apply the new operator to derive a replacement hypothesis. The active value remains as-is with its original provenance until the re-review event is resolved.
2. The substrate may compute _what the new operator would produce_ and surface that as part of the re-review event's payload, but the surfaced candidate is not the active value.
3. Transition from `Pending` to `Resolved*` requires a principal identifier (`PrincipalId` for patient re-review; an authority within the appropriate `AuthorityClass` for institutional re-review).
4. This invariant holds **unconditionally on transition type** (DEF-TE-05): even when the new operator strictly refines the old (the new output refines the old output for every input), automatic propagation is forbidden. The "obviously safer" judgment is reserved for human authority. ∎

**OBL-TE-02 (No silent drift).** [formalizes: `NOTE.md` §4D.4]

There is no code path that updates an active patient or institutional hypothesis as a side effect of activating a new operator-set version. The only paths from a value `v_old` (with provenance under `Δ_old`) to `v_new` (with provenance under `Δ_new`) are, for both substrates:

1. A re-review event resolved as `ResolvedReplace`, with the new value carrying provenance to both the resolving authority (clinician for patient re-review; institutional authority for institutional re-review) and `Δ_new`.
2. Receipt of new evidence that triggers a fresh deduction under `Δ_new` (in which case the active value updates through the normal §2 or §3 path).

Path (1) is the only path that exists _because of_ the operator-set transition. Path (2) is a normal deduction whose currency happens to be after the transition. The two paths are observably distinct in provenance. This obligation applies symmetrically to `Δ_PS` and `Δ_IS`. ∎

### §5.5. Evolution-aware provenance

**DEF-TE-07 (Evolution-aware provenance carrier).** [formalizes: `NOTE.md` §4D.5]

The provenance carrier from DEF-MP-14 is refined for §5: every elementary provenance event includes a `(component_identifier, version_identifier)` pair. The `derives_from` relation respects versions — two provenance carriers that record derivation through the "same" operator at different versions are not equivalent.

In particular, queries of the form

"show all currently-active hypotheses whose derivation chain includes any operator at version V or earlier"

are required to be answerable from provenance alone, without external bookkeeping. ∎

**INV-TE-05 (Provenance pins versions, not just identities).** [formalizes: `NOTE.md` §4D.5]

For any value `v` in the substrate, `prov(v)` records the version (not merely the identity) of every substrate component consulted in producing `v`. This is the conjunction of INV-TE-01 with the version-respecting `derives_from` from DEF-TE-07. ∎

**OBL-TE-03 (Auditability across evolution).** [formalizes: `NOTE.md` §4D.5]

The provenance representation must support audit queries that filter on component versions, not just component identities. Specifically:

- "Which active hypotheses were derived (in whole or in part) under operator-set version `V_old`?"
- "Which re-review events resolved as `ResolvedReplace` between version transitions `V_old → V_new`?"
- "For hypothesis `h^P`, what was every operator version consulted in its derivation chain?"

All three must be answerable from stored provenance without reconstruction of operator state. ∎

### §5.6. Summary of temporal-evolution proof obligations

Consolidated from §5.1–§5.5; cross-listed in §6:

- **OBL-TE-01** — Operator-set transitions require stored transition justifications (DEF-TE-04).
- **OBL-TE-02** — No silent drift across operator-set versions; the two observably-distinct paths from old to new active hypothesis are exhaustive.
- **OBL-TE-03** — Version-aware audit queries are answerable from provenance alone.

Three obligations. The load-bearing one is OBL-TE-02 — it is the structural operationalization of `NOTE.md` §4D.4 ("silent drift is structurally forbidden") and the property that distinguishes this substrate from systems where guideline updates propagate by default.

---

_End of §5._

## §6. Consolidated proof obligations

Every `OBL-*` stated in §2–§5, grouped by substrate, with reading and origin pointer. No new content; this is a single-source-of-truth view for downstream artifacts.

In v0.1.0-draft, every obligation is **stated, not discharged**. Discharge mechanism — mechanized proof, property-based test, runtime assertion, type-system enforcement — is an architectural concern. Each obligation's expected discharge tier is annotated.

### §6.1. Patient substrate (§2)

| Id            | Criticality | Reading                                                                                                                                                                           | Origin | Expected discharge tier                                                                         |
| ------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ | ----------------------------------------------------------------------------------------------- |
| **OBL-PS-01** | S           | Ontology membership (`Atom.is_member`) is decidable in bounded time; non-member atoms cannot enter `Hyp` through any path.                                                        | §2.2   | Type system + parsing-stage validation                                                          |
| **OBL-PS-02** | P           | The patient Galois connection `(Obs_PS, α_PS, γ_PS, H_PS)` satisfies the DEF-MP-08 adjunction.                                                                                    | §2.3   | Property-based test in v0.x; mechanized proof candidate for v1.x                                |
| **OBL-PS-03** | S           | Every operator in `Δ_PS` satisfies DEF-PS-08 (sound deduction).                                                                                                                   | §2.4   | Per-operator informal argument in v0.1; mechanized proof candidate later                        |
| **OBL-PS-04** | S           | For any value `v : T^P` in the substrate, the full derivation chain back to source observations is reconstructible from `prov(v)` via `derives_from`.                             | §2.6   | Runtime assertion + audit-log verification                                                      |
| **OBL-PS-05** | S           | No code path inserts a value into the active-hypothesis position without it being the `Refined(_)` branch of a sound operator's output; proposer outputs cannot bypass operators. | §2.7   | Type-system enforcement (distinct types for proposer-candidate and active-hypothesis positions) |

### §6.2. Institutional substrate (§3)

| Id            | Criticality | Reading                                                                                                                                                          | Origin | Expected discharge tier                                                                 |
| ------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ | --------------------------------------------------------------------------------------- |
| **OBL-IS-01** | S           | Every capacity-update operator preserves physical validity: physically-valid input cannot yield physically-invalid output; would-be violations force abstention. | §3.2   | Type system (refinement type on `Cap` carrying physical-validity proof) + runtime check |
| **OBL-IS-02** | S           | Resource membership (`R.is_member`) is decidable; free-form resource identifiers cannot enter `Cap`.                                                             | §3.2   | Type system + parsing-stage validation                                                  |
| **OBL-IS-03** | P           | The institutional Galois connection `(Evt_IS, α_IS, γ_IS, H_IS)` satisfies DEF-MP-08.                                                                            | §3.3   | Property-based test in v0.x; mechanized proof candidate for v1.x                        |
| **OBL-IS-04** | S           | Every operator in `Δ_IS` satisfies DEF-IS-08 (sound capacity update, including physical validity).                                                               | §3.4   | Per-operator informal argument in v0.1; mechanized proof candidate later                |
| **OBL-IS-05** | S           | Provenance auditability holds for the institutional substrate (mirrors OBL-PS-04).                                                                               | §3.6   | Runtime assertion + audit-log verification                                              |
| **OBL-IS-06** | S           | Proposer-operator separation enforced structurally in the institutional substrate (mirrors OBL-PS-05).                                                           | §3.7   | Type-system enforcement                                                                 |

### §6.3. Interaction layer (§4)

| Id            | Criticality | Reading                                                                                                                                                                                                                                           | Origin | Expected discharge tier                                                                                       |
| ------------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------- |
| **OBL-IX-01** | F           | Cross-layer derivation functions (`derive_alloc`, `derive_patient`) are consistent with both substrate Galois connections; every derivation states an explicit bound on what the other substrate must accept.                                     | §4.1   | Per-derivation informal argument + property-based test                                                        |
| **OBL-IX-02** | P           | Every coupling check used in a joint operator is sound per DEF-IX-07.                                                                                                                                                                             | §4.3   | Per-coupling-check informal argument; mechanized proof candidate later                                        |
| **OBL-IX-03** | P           | No code path produces the net effect "patient-substrate preference suppressed, institutionally-feasible refinement silently applied" without an explicit `Divergent(diff)` abstention being produced and the composite state remaining unchanged. | §4.4   | Type-system enforcement (composite-state updates only via licensed-refinement constructor) + integration test |

### §6.4. Temporal evolution (§5)

| Id            | Criticality | Reading                                                                                                                                                                                                                                                                                                          | Origin | Expected discharge tier                                             |
| ------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ | ------------------------------------------------------------------- |
| **OBL-TE-01** | F           | No operator-set version may be activated without a stored transition justification (DEF-TE-04) relative to its predecessor; the substrate refuses to load operator-set versions lacking one.                                                                                                                     | §5.3   | Type-system enforcement + load-time validation                      |
| **OBL-TE-02** | P           | The two paths from an active value under `Δ_old` to an active value under `Δ_new` (re-review resolution, fresh deduction with post-transition currency) are exhaustive and observably distinct in provenance; no third silent path exists. Applies symmetrically to `Δ_PS` (patient) and `Δ_IS` (institutional). | §5.4   | Type-system enforcement + integration test + audit-log verification |
| **OBL-TE-03** | S           | Version-aware audit queries are answerable from stored provenance alone, without reconstruction of operator state.                                                                                                                                                                                               | §5.5   | Provenance-schema validation + query-engine test                    |

### §6.5. Tally and criticality distribution

- 17 obligations total: 5 patient, 6 institutional, 3 interaction, 3 temporal.
- 6 expected to discharge via type-system enforcement (or type-system + complementary mechanism).
- 4 expected to discharge via property-based testing in v0.x with mechanization as a future candidate.
- 5 expected to discharge via runtime assertion or audit-log verification.
- 2 are per-instance informal arguments at v0.1, accumulating as `Δ_PS` and `Δ_IS` grow.

The distribution matters for architectural planning: roughly a third of obligations live in the type system (so the build system can enforce them), roughly a third live in test infrastructure (so CI can enforce them per the meta-rule that principles without enforcement are decoration), and roughly a third live in runtime/audit (so they are observable in operation rather than at build time).

**Criticality distribution (derived from NOTE.md v0.12.0 §4 tier assignments):**

- **P (Position-critical):** 5 obligations — OBL-PS-02, OBL-IS-03, OBL-IX-02, OBL-IX-03, OBL-TE-02.
- **S (Safety-property):** 10 obligations — OBL-PS-01, OBL-PS-03, OBL-PS-04, OBL-PS-05, OBL-IS-01, OBL-IS-02, OBL-IS-04, OBL-IS-05, OBL-IS-06, OBL-TE-03.
- **F (Foundation):** 2 obligations — OBL-IX-01, OBL-TE-01.

Total: P=5, S=10, F=2. Sum = 17 obligations. ✓

(Note: the 18 principles in NOTE.md §4 yield 5 P, 10 S, 3 F by direct tier assignment; the 17 obligations in SPEC.md §6 yield 5 P, 10 S, 2 F. The slight discrepancy in F-count reflects that OBL-IX-01 and OBL-TE-01 are the two obligations corresponding to the three F-tier principles in NOTE.md — 4C.2, 4D.1, 4D.2 — because some F-tier principles aggregate into a single obligation while others split.)

### §6.6. Criticality inheritance from NOTE.md

Each obligation inherits its criticality from the NOTE.md §4 principle it most directly supports, via the `[formalizes:]` annotations in §2–§5. Where an obligation supports multiple principles, the higher tier (P > S > F) governs.

This inheritance is the contract between NOTE.md and SPEC.md on criticality: if a NOTE.md principle's tier changes in a future revision, every SPEC.md obligation traceable to that principle re-inherits the new tier in lockstep. The audit query "for principle 4X.Y at tier T, which obligations does it carry?" is answerable from the §8 traceability table (pending in v0.2.0-draft).

The inverse query — "for obligation OBL-XX-NN, which principles license it, and what is its inherited criticality?" — is the per-obligation accountability check that links architectural review back to the position note's load-bearing claims.

---

_End of §6._

## §7. Open formal questions and deferred commitments

This section catalogs questions surfaced during §1–§6 drafting that v0.2.0-draft does not resolve. Three categories:

- **Deferred resolution** — the question is real, an answer is needed, but v0.2 is the wrong revision to commit. Each entry names the disposition (which future revision, what would trigger commitment).
- **Pending NOTE.md confirmation** — SPEC.md makes a claim stronger than the corresponding NOTE.md principle plainly licenses. Each entry names what would need to change in `NOTE.md` (or in SPEC.md) to resolve the asymmetry.
- **Structural choice under uncertainty** — a commitment was made under absent or weak evidence; revision is plausible.

Numbering: open questions are tagged `OQ-{section}-{NN}` where section ∈ {MP, PS, IS, IX, TE, X} (X = cross-cutting). The naming mirrors `OBL-*` so a reader can scan obligations and open questions side by side.

### §7.1. Mathematical preliminaries (§1)

**OQ-MP-01 — INV-MP-02 deflationary/inflationary orientation.** SPEC.md §1.4 stated that `α ∘ γ` is deflationary on `A` and `γ ∘ α` is inflationary on `C`, with a parenthetical noting that Cousot & Cousot (1977) use the dual convention depending on which side is treated as "abstract." The orientation is correct for the use SPEC.md makes of it in §2.3 (`A = Hyp`, `C = Obs`), but the convention should be explicitly verified at each substrate instantiation. _Disposition:_ resolved when §8 traceability adds per-substrate orientation checks.

**OQ-MP-02 — `Result⟨H, R⟩` name collision with Rust's `Result<T, E>`.** DEF-MP-13 reserves the name `Result` for the operator-output sum type. Rust's standard `Result<T, E>` has `Ok` and `Err` constructors with error-channel semantics; SPEC.md's `Result⟨H, R⟩` has `Refined` and `Abstain` constructors with epistemic-state semantics. The collision is documented in DEF-MP-13's text, not resolved. _Disposition:_ candidate rename to `Outcome⟨H, R⟩` in v0.3.x if downstream confusion is observed during ARCHITECTURE.md drafting; no commitment yet.

**OQ-MP-03 — Provenance carrier algebra.** DEF-MP-14 specifies `Prov` as an associative-identity monoid with a `derives_from` predicate. This is the minimum interface SPEC.md depends on. Whether the concrete provenance encoding (Merkle DAG, signed event chain, content-addressed graph) preserves additional properties — for example, commutativity in cases where derivation order is irrelevant — is left open. _Disposition:_ deferred to ARCHITECTURE.md; SPEC.md should not commit further.

### §7.2. Patient substrate (§2)

**OQ-PS-01 — Existence of a bottom element in `Hyp`.** DEF-PS-11's text noted that SPEC.md does not commit to whether `Hyp` has a bottom element `⊥_PS`, and that if one exists it represents "a maximally-specific patient state consistent with all observations" — a different concept from abstention. The question is open. _Disposition:_ defer to v0.x revision when a concrete clinical scenario forces a commitment. Plausible answer: `⊥_PS` does not exist as a substrate-level element, because no maximally-specific patient state is generally constructible from finite observations; the partial-meet structure handles this without needing a bottom.

**OQ-PS-02 — DEF-PS-15.2 "at most one refinement step." [RESOLVED in v0.3.0 — NOTE.md v0.12.0 §4A.5]** The proposer codomain is constrained to "candidates that could plausibly come out of some `δ ∈ Δ_PS` from this evidence." `NOTE.md` v0.12.0 §4A.5 now explicitly states "The proposer's output space is bounded to candidates the deduction operator set could in principle produce from the available evidence — multi-step refinements that would require a chain of operator applications are not proposer-level outputs." SPEC.md DEF-PS-15.2 stands as the correct formalization.

**OQ-PS-03 — DEF-PS-08 soundness condition: conjunction of two clauses. [RESOLVED in v0.3.0 — NOTE.md v0.12.0 §4A.2]** SPEC.md defined operator soundness as: refined output must (a) refine the input hypothesis _and_ (b) refine `α_PS(o_e)`. `NOTE.md` v0.12.0 §4A.2 now explicitly states "A licensed refinement is sound only if it is both (a) at least as specific as the prior state — refinement does not generalize — and (b) supported by the cited evidence: a hypothesis refinement that the evidence does not entail is unsound regardless of operator licensing." SPEC.md DEF-PS-08's conjunction reading stands.

### §7.3. Institutional substrate (§3)

**OQ-IS-01 — Physical validity inside DEF-IS-08 versus as a separate invariant.** SPEC.md folded physical-validity preservation into DEF-IS-08's third clause, making "sound capacity-update operator" mean physical-validity-preserving by definition. The alternative is to leave physical validity as INV-IS-only and treat it as an invariant a separately-defined-sound operator must additionally satisfy. The current formulation is more committal but the right one if physical validity is conceptually inseparable from soundness in the institutional substrate. _Structural choice under uncertainty._ Revisit if a clinical scenario demonstrates a useful "physically-invalid but operator-sound" intermediate state.

**OQ-IS-02 — DEF-IS-11 (allocation abstention is not stalling). [RESOLVED in v0.3.0 — NOTE.md v0.12.0 §4B.3]** SPEC.md committed that `Abstain(r)` is structurally distinct from timeout, crash, or silent-default. `NOTE.md` v0.12.0 §4B.3 now states "This abstention is a structurally distinct output — observably different in the audit trail from a timeout, a crash, or a default-fallthrough; the substrate guarantees that every allocation decision yields either a licensed allocation or an explicit abstention in bounded steps, never silent failure." DEF-IS-11 stands as the correct formalization.

**OQ-IS-03 — Institutional re-review event under §5.4. [RESOLVED in v0.3.0 — DEF-TE-06b added; NOTE.md v0.12.0 §4D.4 institutional symmetry paragraph]** §5.4 now formalizes the institutional analog via `DEF-TE-06b` (`InstReReviewEvent`) and OBL-TE-02 expands to span both substrates. The architecture's silent-drift prohibition now applies symmetrically to clinical recommendations and to allocation decisions.

### §7.4. Interaction layer (§4)

**OQ-IX-01 — SubstrateDiff.alternative_feasible_option. [RESOLVED in v0.3.0 — SPEC.md weakened]** DEF-IX-09 originally specified `alternative_feasible_options: Set⟨...⟩`. `NOTE.md` v0.12.0 §4C.1 retained its singular-pair reading ("the unconstrained-optimal recommendation and the institutionally-actionable plan"). Resolution: SPEC.md weakens to `alternative_feasible_option: Option⟨(Hyp^P, Cap^P)⟩`. The set-of-alternatives reading was an SPEC.md overreach; the substrate produces at most one alternative — the institutionally-actionable plan — alongside the patient-locally-optimal hypothesis.

**OQ-IX-02 — DEF-IX-05 joint operator decomposition.** SPEC.md committed that every joint operator decomposes as `(δ_PS', δ_IS', coupling_check)`. This forbids monolithic joint operators not traceable to substrate-local ones. The decomposition discipline is what makes INV-IX-04 (substrate independence) provable, but it reduces expressiveness. _Structural choice under uncertainty._ Revisit if a clinical scenario requires a joint operator that resists clean decomposition. The patient/institutional pair was designed to be modular precisely to enable this discipline; if the modularity holds in practice, the choice is sound.

**OQ-IX-03 — INV-IX-03 strength (no silent downgrade). [RESOLVED in v0.3.0 — NOTE.md v0.12.0 §4C.1]** SPEC.md's strongest claim: the substrate produces a `Divergent(diff)` abstention rather than silently picking the institutionally-feasible refinement. `NOTE.md` v0.12.0 §4C.1 now explicitly states "This is a structural property of the substrate, not a behavioral preference: silent downgrade is observably distinct in the audit trail from divergent licensing, and the architecture forbids the path that would produce silent downgrade rather than rely on the learned components to avoid it." INV-IX-03 stands as the correct formalization.

### §7.5. Temporal evolution (§5)

**OQ-TE-01 — DEF-TE-05 `Incomparable` category licensing. [RESOLVED in v0.3.0 — NOTE.md v0.12.0 §4D.3 + DEF-TE-05 updated]** SPEC.md committed to three comparability-statement categories: `Refinement`, `Generalization`, `Incomparable`. The escape-hatch risk (every transition becoming `Incomparable`) is now closed by `NOTE.md` v0.12.0 §4D.3's explicit asymmetric justification burden and the corresponding tightening in DEF-TE-05, which now requires `Incomparable` transitions to state (a) which clinical judgment the new operator embodies, (b) why that judgment supersedes the prior one, and (c) the evidence base anchoring the new judgment. `Refinement` and `Generalization` transitions carry the order-theoretic relationship as part of their structural justification; `Incomparable` transitions must provide it explicitly.

**OQ-TE-02 — INV-TE-04 strength (no automatic replacement under any transition). [RESOLVED in v0.3.0 — NOTE.md v0.12.0 §4D.4]** SPEC.md committed that active hypotheses with pending re-reviews stay as-is even when the new operator is strictly better. `NOTE.md` v0.12.0 §4D.4 now states this unconditionally: "This commitment is unconditional on transition type — even when the new operator strictly refines the old (the new recommendation is at least as specific and at least as cautious as the old), automatic propagation is forbidden." INV-TE-04 has been updated to reflect this explicitly (clause 4 of the invariant).

**OQ-TE-03 — Currency thresholds and freshness-class enumeration.** DEF-TE-02 named four freshness classes (`Realtime`, `Recent`, `Stale`, `Historical`) without committing to threshold values or to whether the enumeration is exhaustive. The thresholds are correctly an architectural choice (per §0.5 exclusions). _Disposition:_ deferred to ARCHITECTURE.md; the enumeration set itself may need extension when concrete clinical evidence-currency vocabularies are surveyed.

### §7.6. Cross-cutting and structural

**OQ-X-01 — F-count asymmetry between NOTE.md and SPEC.md.** `NOTE.md` v0.11.0 §4 yields 3 F-tier principles (4C.2, 4D.1, 4D.2). SPEC.md §6 yields 2 F-tier obligations (OBL-IX-01, OBL-TE-01). The difference is structural: §4D.2 (currency tracking) and parts of §4D.1 (versioning) are formalized through definitions (DEF-TE-01, DEF-TE-02, DEF-TE-03, INV-TE-01, INV-TE-02) without producing a discharge-bearing obligation. _Disposition:_ either add a missing obligation (e.g., "OBL-TE-04: currency-aware operator behavior is enforceable") or document the asymmetry as expected. The latter is the v0.2 choice; revisit in v0.3.x if a discharge mechanism for these foundational properties surfaces during ARCHITECTURE.md drafting.

**OQ-X-02 — Bidirectional traceability formalization (§8).** §8 (NOTE.md ↔ SPEC.md bidirectional table) is pending in v0.2.0-draft. The §0.1 commitment that "clinicians scrutinize SPEC.md via §8" is unmet until §8 lands. _Disposition:_ drafted in the v0.2.x cycle; §8 is required-before-v1.0 but the format (markdown table, separate file, generated artifact) is open.

**OQ-X-03 — Discharge mechanism for "informal argument" obligations.** OBL-PS-03 and OBL-IS-04 expect discharge via "per-operator informal argument in v0.1; mechanized proof candidate later." The informal-argument form is an obvious soft spot: a one-paragraph English justification for each operator's soundness, attached to that operator's definition, with no machine-checkable property tying the argument to the operator's actual behavior. _Disposition:_ unresolved by design. Mechanization is a v1.x or v2.x destination per §0.2 Tier C path; in v0.x, the informal arguments accumulate as text and the gap between argument and behavior is named, not closed.

**OQ-X-04 — Composite-substrate-state invariants beyond product structure.** DEF-IX-03 stated that the composite state `S = Hyp^P × Cap^P` is "not itself a poset with new structure — it is the product poset." This may be too modest. A composite refinement that is licensed under §4C may carry invariants that neither component does alone — for example, "every active institutional bed allocation has a corresponding active patient hypothesis." _Disposition:_ defer to v0.3.x. Surfacing concrete composite invariants requires worked examples that v0.2 does not include.

**OQ-X-05 — Concurrency and observation atomicity.** SPEC.md is silent on concurrent operator application. Two operators applied simultaneously to the same composite state may produce a result no sequential ordering produces. The patient substrate (one per patient) bounds this to within-patient races; the institutional substrate is genuinely concurrent across patients. _Disposition:_ deferred to ARCHITECTURE.md and beyond. A formal treatment requires committing to a concurrency model (linearizability, serializability, CRDT-style commutative composition); SPEC.md should not pre-commit at v0.2.

### §7.7. Resolution path

Each open question above carries a disposition. Summarizing across categories (status as of v0.3.0-draft):

- **Resolved in v0.3.0** (closed by NOTE.md v0.12.0 strengthening or by corresponding SPEC.md weakening): OQ-PS-02, OQ-PS-03, OQ-IS-02, OQ-IS-03, OQ-IX-01, OQ-IX-03, OQ-TE-01, OQ-TE-02. **Eight items closed.**
- **Deferred to v0.4.x or later** (revisit when ARCHITECTURE.md drafting or worked examples force commitment): OQ-MP-02, OQ-MP-03, OQ-IX-02, OQ-TE-03, OQ-X-01, OQ-X-04, OQ-X-05. Seven items.
- **Structural choice under uncertainty** (commitment made; revisit if downstream surface area reveals the choice was wrong): OQ-IS-01, OQ-IX-02 (also listed above). Two items, one overlapping.
- **Required before v1.0** (cannot ship a stable v1 with these open): OQ-X-02 (§8 bidirectional traceability), OQ-X-03 (informal-argument discharge). Two items.
- **Confined to within-section disposition** (resolved per concrete clinical instantiation): OQ-MP-01, OQ-PS-01. Two items.

Total: 20 open questions originally; 8 resolved in v0.3.0; **12 remain open**.

The **Pending NOTE.md confirmation** bucket — six items at v0.2.0 — is now empty. Five of the six were resolved by NOTE.md v0.12.0 strengthening; one (OQ-IX-01) was resolved by SPEC.md weakening to match NOTE.md's singular-pair reading. This is the cleanest possible disposition of that bucket: every formalization-vs-prose ambiguity was decided explicitly rather than left unresolved.

The largest remaining bucket is **Deferred to v0.4.x or later** — these will surface again during ARCHITECTURE.md drafting and worked-example construction, where downstream commitments may force resolution.

---

_End of §7._
