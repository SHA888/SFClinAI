//! Clinical lattice types for substrate-first clinical AI.
//!
//! The `clinlat` crate provides a symbolic substrate for clinical decision-making,
//! based on refinable lattices of hypotheses with sound deduction operators.
//!
//! # v0.1.0 Scope
//!
//! This version implements:
//! - A `Hyp` poset type with refinement ordering and partial meet.
//! - An `Outcome<H, A>` sum type for operator results (refined hypothesis or abstention).
//! - A single deduction operator: SOFA-3 respiratory component (PaO₂/FiO₂).
//!
//! # Simplifications for v0.1.0
//!
//! - `AtomId` is a static string reference (`&'static str`); real ontology binding
//!   (SNOMED CT, RxNorm, LOINC, ICD-11) is deferred to v0.2.
//! - Provenance carriers are unit type `()`; structured provenance is deferred to v0.2.

pub mod hyp;
pub mod outcome;
pub mod abstain;
pub mod operator;

pub use hyp::{Hyp, AtomId};
pub use outcome::Outcome;
pub use abstain::AbstainReason;
pub use operator::Operator;
