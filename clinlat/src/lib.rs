//! # `clinlat` — Clinical Lattice Types
//!
//! A Rust substrate for symbolic clinical decision-making based on refinable hypothesis lattices
//! and sound deduction operators.
//!
//! ## Overview
//!
//! The `clinlat` crate provides:
//!
//! - [`Hyp`]: A partially ordered set (poset) of clinical hypotheses, ordered by refinement.
//! - [`Outcome<H, A>`]: A sum type for operator results (refined hypothesis or abstention).
//! - **Deduction operators**: Sound functions that refine hypotheses using clinical evidence
//!   (e.g., [`sofa::SofaRespOperator`] for respiratory dysfunction severity).
//!
//! ## Architecture
//!
//! ### Hypotheses and Refinement
//!
//! A [`Hyp`] represents a set of clinical atoms at some level of specificity.
//! Hypotheses are ordered by **refinement**: `h1 ⊑ h2` (h1 refines h2) means h1 is more specific.
//!
//! ```
//! # use clinlat::{Hyp, Atom, OntologySystem};
//! let unknown = Hyp::unknown();  // Top element: no information.
//! let atom = Atom {
//!     system: OntologySystem::SNOMED,
//!     code: "67822003".to_string(),
//!     preferred_term: "Hypoxemia".to_string(),
//!     version: "2026-01-31".to_string(),
//! };
//! let specific = Hyp::new(vec![atom]);  // More specific.
//! assert!(specific < unknown);  // Refinement ordering.
//! ```
//!
//! ### Operators
//!
//! An [`Operator`] takes a hypothesis and evidence, producing an [`Outcome`]:
//!
//! ```ignore
//! impl Operator for MyOperator {
//!     fn apply(&self, h: &Hyp, e: &Evidence) -> Outcome<Hyp, AbstainReason> {
//!         // Refine h using evidence e, or abstain with a reason.
//!     }
//! }
//! ```
//!
//! Operators are **sound** by construction: they satisfy three properties (DEF-PS-08):
//! 1. **Refinement monotonicity**: Hypothesis specificity never decreases.
//! 2. **No spurious refinement**: Output never exceeds evidence justification.
//! 3. **Abstention purity**: Abstention is structural, not error handling.
//!
//! ### SOFA-3 Respiratory Scoring
//!
//! The [`sofa`] module implements one operator: SOFA-3 respiratory severity (PaO₂/FiO₂ ratio).
//!
//! ```
//! # use clinlat::sofa::{SofaRespEvidence, score_from_ratio};
//! let evidence = SofaRespEvidence::new(350.0, 1.0, false);
//! let ratio = evidence.pao2_fio2_ratio().unwrap();
//! let score = score_from_ratio(ratio, false);  // Maps to SOFA score.
//! assert_eq!(score, Some(1));
//! ```
//!
//! ## v0.1.0 Simplifications
//!
//! - [`AtomId`] is `&'static str`; real ontology binding (SNOMED CT, RxNorm, LOINC, ICD-11)
//!   is deferred to v0.2.
//! - [`operator::Evidence`] is a unit type in the generic trait; concrete operators
//!   (like [`sofa::SofaRespOperator`]) define richer evidence types.
//! - Provenance is unit `()`; timestamped lineage is deferred to v0.2.
//!
//! ## References
//!
//! - **Position**: [Substrate-First Clinical AI](https://github.com/SHA888/SFClinAI)
//!   (NOTE.md § 4A–4D).
//! - **Formalization**: SPEC.md § 2 (patient-state substrate).
//! - **SOFA-3**: Vincent et al. (1996), Singer et al. (2016, Sepsis-3).

pub mod abstain;
pub mod hyp;
pub mod ontology;
pub mod operator;
pub mod outcome;
pub mod sofa;

pub use abstain::AbstainReason;
pub use hyp::Hyp;
pub use ontology::{
    Atom, CacheMode, Icd11Adapter, LoincAdapter, OntologyAdapter, OntologyError, OntologySystem,
    RxNormAdapter, SNOMEDAdapter,
};
pub use operator::Operator;
pub use outcome::Outcome;
pub use sofa::{SofaRespEvidence, SofaRespHypothesis, SofaRespOperator};
