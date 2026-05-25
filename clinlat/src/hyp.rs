//! Clinical hypotheses as a refinable poset.
//!
//! A `Hyp` represents a set of clinical hypotheses at some level of refinement.
//! Hypotheses are ordered by refinement: a hypothesis h1 refines h2 (h1 ≤ h2) if h1 is more specific.
//!
//! # v0.1.0 Simplification
//!
//! `AtomId` is `&'static str` (static string reference).
//! Real ontology binding (SNOMED CT, RxNorm, LOINC, ICD-11) is deferred to v0.2 (DEF-PS-03).

use std::cmp::Ordering;

/// Atom identifier: a clinical concept or observation label.
///
/// In v0.1.0, atoms are static string references.
/// In v0.2.0+, atoms will be bound to external ontologies (SNOMED CT, RxNorm, LOINC, ICD-11).
pub type AtomId = &'static str;

/// A clinical hypothesis: a refinable set of atoms.
///
/// Hypotheses are ordered by refinement: `h1 ≤ h2` means h1 is more specific (refines) h2.
/// This ordering forms a partially ordered set (poset) where:
/// - `Unknown` is the top element (least specific).
/// - Each concrete hypothesis (e.g., `SOFA_Score3`) is more specific.
///
/// Implements DEF-PS-01 (refinement order) and INV-PS-02 (partial meet).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hyp(Vec<AtomId>);

impl Hyp {
    /// Creates a new hypothesis from a list of atoms.
    pub fn new(atoms: Vec<AtomId>) -> Self {
        Hyp(atoms)
    }

    /// Creates the top element (most general hypothesis, `Unknown`).
    pub fn unknown() -> Self {
        Hyp(vec![])
    }

    /// Checks if this hypothesis is compatible with another.
    ///
    /// Two hypotheses are compatible if they can coexist (neither contradicts the other).
    /// Implements DEF-PS-01 compatibility predicate.
    ///
    /// # v0.1.0 Definition
    ///
    /// Two hypotheses are compatible if they are equal or if at least one is `Unknown`.
    /// In v0.2.0+, compatibility will be determined by ontological constraints.
    pub fn compat(&self, other: &Hyp) -> bool {
        self == other || self.is_unknown() || other.is_unknown()
    }

    /// Computes the partial meet of two hypotheses.
    ///
    /// The meet of h1 and h2 is the greatest lower bound (most general hypothesis that refines both).
    /// Returns `None` if no meet exists (hypotheses are incompatible).
    ///
    /// Implements INV-PS-02 (meet operation).
    ///
    /// # v0.1.0 Definition
    ///
    /// - If h1 and h2 are equal, their meet is h1.
    /// - If one is `Unknown`, the meet is the other.
    /// - Otherwise, no meet exists (returns `None`).
    ///
    /// In v0.2.0+, meet will be determined by lattice structure over the ontology.
    pub fn meet(&self, other: &Hyp) -> Option<Hyp> {
        if self == other {
            return Some(self.clone());
        }
        if self.is_unknown() {
            return Some(other.clone());
        }
        if other.is_unknown() {
            return Some(self.clone());
        }
        None
    }

    /// Checks if this is the `Unknown` hypothesis (top element).
    fn is_unknown(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the atoms in this hypothesis.
    pub fn atoms(&self) -> &[AtomId] {
        &self.0
    }
}

impl PartialOrd for Hyp {
    /// Compares two hypotheses by refinement order.
    ///
    /// Returns `Some(Ordering::Less)` if `self` refines `other` (is more specific).
    /// Returns `Some(Ordering::Greater)` if `self` is refined by `other` (is more general).
    /// Returns `Some(Ordering::Equal)` if `self == other`.
    ///
    /// Returns `None` if `self` and `other` are incomparable.
    ///
    /// Implements DEF-PS-01 refinement order.
    fn partial_cmp(&self, other: &Hyp) -> Option<Ordering> {
        match (self, other) {
            // Equal hypotheses.
            _ if self == other => Some(Ordering::Equal),
            // Unknown is the top element; any hypothesis refines Unknown.
            (_, _) if other.is_unknown() => Some(Ordering::Less),
            // Any hypothesis is refined by Unknown.
            (_, _) if self.is_unknown() => Some(Ordering::Greater),
            // Incomparable hypotheses (no ordering relationship).
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_top_element() {
        let unknown = Hyp::unknown();
        let hyp = Hyp::new(vec!["diagnosis_a"]);

        // Any hypothesis refines (is more specific than) Unknown.
        assert_eq!(hyp.partial_cmp(&unknown), Some(Ordering::Less));
        // Unknown refines any hypothesis (is less specific).
        assert_eq!(unknown.partial_cmp(&hyp), Some(Ordering::Greater));
    }

    #[test]
    fn test_equal_hypotheses() {
        let h1 = Hyp::new(vec!["diagnosis_a", "severity_high"]);
        let h2 = Hyp::new(vec!["diagnosis_a", "severity_high"]);

        assert_eq!(h1.partial_cmp(&h2), Some(Ordering::Equal));
        assert!(h1.compat(&h2));
    }

    #[test]
    fn test_incomparable_hypotheses() {
        let h1 = Hyp::new(vec!["diagnosis_a"]);
        let h2 = Hyp::new(vec!["diagnosis_b"]);

        // Incomparable hypotheses have no ordering relationship.
        assert_eq!(h1.partial_cmp(&h2), None);
        // Incomparable hypotheses are not compatible (unless one is Unknown).
        assert!(!h1.compat(&h2));
    }

    #[test]
    fn test_compat_with_unknown() {
        let unknown = Hyp::unknown();
        let hyp = Hyp::new(vec!["diagnosis_a"]);

        // Any hypothesis is compatible with Unknown.
        assert!(hyp.compat(&unknown));
        assert!(unknown.compat(&hyp));
    }

    #[test]
    fn test_meet_equal_hypotheses() {
        let h1 = Hyp::new(vec!["diagnosis_a"]);
        let h2 = Hyp::new(vec!["diagnosis_a"]);

        assert_eq!(h1.meet(&h2), Some(h1.clone()));
    }

    #[test]
    fn test_meet_with_unknown() {
        let unknown = Hyp::unknown();
        let hyp = Hyp::new(vec!["diagnosis_a"]);

        // Meet of any hypothesis with Unknown is the hypothesis itself.
        assert_eq!(hyp.meet(&unknown), Some(hyp.clone()));
        assert_eq!(unknown.meet(&hyp), Some(hyp.clone()));
    }

    #[test]
    fn test_meet_incomparable_none() {
        let h1 = Hyp::new(vec!["diagnosis_a"]);
        let h2 = Hyp::new(vec!["diagnosis_b"]);

        // Incomparable hypotheses have no meet.
        assert_eq!(h1.meet(&h2), None);
    }
}
