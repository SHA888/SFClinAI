//! Clinical hypotheses as a refinable poset.
//!
//! A `Hyp` represents a set of clinical hypotheses at some level of refinement.
//! Hypotheses are ordered by refinement: a hypothesis h1 refines h2 (h1 ≤ h2) if h1 is more specific.
//!
//! Per [SPEC.md § 2] (DEF-PS-01, INV-PS-02):
//! Hypotheses are formalized as sets of resolved atoms from external ontologies (SNOMED CT, RxNorm, LOINC, ICD-11).
//! Atoms replace the v0.1.0 `&'static str` placeholder, enabling real ontology binding and audit trail support.

use crate::ontology::Atom;
use std::cmp::Ordering;
use std::collections::HashSet;

/// A clinical hypothesis: a refinable set of resolved atoms.
///
/// Hypotheses are ordered by refinement: `h1 ≤ h2` means h1 is more specific (refines) h2.
/// This ordering forms a partially ordered set (poset) where:
/// - `Unknown` is the top element (least specific).
/// - Each concrete hypothesis carries resolved `Atom` payloads from external ontologies.
///
/// Implements DEF-PS-01 (refinement order) and INV-PS-02 (partial meet).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hyp(Vec<Atom>);

impl Hyp {
    /// Creates a new hypothesis from a list of atoms.
    ///
    /// Atoms are deduplicated and sorted by (system, code) to ensure consistency
    /// between `PartialEq` (Vec comparison) and `PartialOrd` (HashSet comparison).
    /// This enforces that duplicate atoms are collapsed to a single logical entry,
    /// preventing INV-PS-02 violations where `meet()` would fail on semantically
    /// identical hypotheses that differ only by atom duplication.
    pub fn new(atoms: Vec<Atom>) -> Self {
        let mut deduplicated: Vec<Atom> = atoms;
        deduplicated.sort_by(|a, b| {
            // Sort by (system, code) first, then by version for determinism
            match a.system.cmp(&b.system) {
                Ordering::Equal => match a.code.cmp(&b.code) {
                    Ordering::Equal => a.version.cmp(&b.version),
                    other => other,
                },
                other => other,
            }
        });
        deduplicated.dedup();
        Hyp(deduplicated)
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
    /// Two hypotheses are compatible if they are equal or if at least one is `Unknown`.
    /// In future versions, compatibility may be determined by ontological constraints.
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
    /// - If h1 and h2 are equal, their meet is h1.
    /// - If one is `Unknown`, the meet is the other.
    /// - Otherwise, no meet exists (returns `None`).
    ///
    /// In future versions, meet will be determined by lattice structure over the ontology.
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
    pub fn atoms(&self) -> &[Atom] {
        &self.0
    }
}

impl PartialOrd for Hyp {
    /// Compares two hypotheses by refinement order via atom-set inclusion.
    ///
    /// `self ⊑ other` (self refines other) iff `self.atoms() ⊇ other.atoms()` as sets:
    /// more atoms = more specific (more refined). The empty atom set (`Unknown`) is
    /// the top element; every hypothesis refines it.
    ///
    /// - `Some(Less)`: `self` refines `other` (self has strictly more atoms).
    /// - `Some(Greater)`: `other` refines `self` (self has strictly fewer atoms).
    /// - `Some(Equal)`: atom sets coincide.
    /// - `None`: atom sets are incomparable (neither is a subset of the other).
    ///
    /// This implementation aligns `Hyp::PartialOrd` with the formal `⊑_PS` semantics
    /// used in the OBL-PS-02 proof sketch and required for the Galois adjunction
    /// `α_PS(e) ⊑ h ⟺ atoms(α_PS(e)) ⊇ atoms(h)` to hold beyond Unknown.
    ///
    /// Implements DEF-PS-01 refinement order.
    fn partial_cmp(&self, other: &Hyp) -> Option<Ordering> {
        let self_atoms: HashSet<&Atom> = self.atoms().iter().collect();
        let other_atoms: HashSet<&Atom> = other.atoms().iter().collect();
        let self_subset_of_other = self_atoms.is_subset(&other_atoms);
        let other_subset_of_self = other_atoms.is_subset(&self_atoms);
        match (self_subset_of_other, other_subset_of_self) {
            (true, true) => Some(Ordering::Equal),
            (false, true) => Some(Ordering::Less), // self has strictly more atoms => more refined
            (true, false) => Some(Ordering::Greater), // self has strictly fewer atoms => more general
            (false, false) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::OntologySystem;

    fn atom_diagnosis_a() -> Atom {
        Atom {
            system: OntologySystem::SNOMED,
            code: "67822003".to_string(),
            preferred_term: "Hypoxemia".to_string(),
            version: "2026-01-31".to_string(),
        }
    }

    fn atom_diagnosis_b() -> Atom {
        Atom {
            system: OntologySystem::SNOMED,
            code: "3723001".to_string(),
            preferred_term: "Acute respiratory distress syndrome".to_string(),
            version: "2026-01-31".to_string(),
        }
    }

    fn atom_severity_high() -> Atom {
        Atom {
            system: OntologySystem::SNOMED,
            code: "24484000".to_string(),
            preferred_term: "Severe".to_string(),
            version: "2026-01-31".to_string(),
        }
    }

    #[test]
    fn test_unknown_top_element() {
        let unknown = Hyp::unknown();
        let hyp = Hyp::new(vec![atom_diagnosis_a()]);

        // Any hypothesis refines (is more specific than) Unknown.
        assert_eq!(hyp.partial_cmp(&unknown), Some(Ordering::Less));
        // Unknown refines any hypothesis (is less specific).
        assert_eq!(unknown.partial_cmp(&hyp), Some(Ordering::Greater));
    }

    #[test]
    fn test_equal_hypotheses() {
        let atom_a = atom_diagnosis_a();
        let atom_s = atom_severity_high();
        let h1 = Hyp::new(vec![atom_a.clone(), atom_s.clone()]);
        let h2 = Hyp::new(vec![atom_a, atom_s]);

        assert_eq!(h1.partial_cmp(&h2), Some(Ordering::Equal));
        assert!(h1.compat(&h2));
    }

    #[test]
    fn test_incomparable_hypotheses() {
        let h1 = Hyp::new(vec![atom_diagnosis_a()]);
        let h2 = Hyp::new(vec![atom_diagnosis_b()]);

        // Incomparable hypotheses have no ordering relationship.
        assert_eq!(h1.partial_cmp(&h2), None);
        // Incomparable hypotheses are not compatible (unless one is Unknown).
        assert!(!h1.compat(&h2));
    }

    #[test]
    fn test_subset_refinement() {
        // Atom-set inclusion: a superset hypothesis refines the subset one.
        let h_general = Hyp::new(vec![atom_diagnosis_a()]);
        let h_specific = Hyp::new(vec![atom_diagnosis_a(), atom_severity_high()]);

        // h_specific refines h_general (more atoms => more specific).
        assert_eq!(h_specific.partial_cmp(&h_general), Some(Ordering::Less));
        assert_eq!(h_general.partial_cmp(&h_specific), Some(Ordering::Greater));
    }

    #[test]
    fn test_compat_with_unknown() {
        let unknown = Hyp::unknown();
        let hyp = Hyp::new(vec![atom_diagnosis_a()]);

        // Any hypothesis is compatible with Unknown.
        assert!(hyp.compat(&unknown));
        assert!(unknown.compat(&hyp));
    }

    #[test]
    fn test_meet_equal_hypotheses() {
        let atom = atom_diagnosis_a();
        let h1 = Hyp::new(vec![atom.clone()]);
        let h2 = Hyp::new(vec![atom]);

        assert_eq!(h1.meet(&h2), Some(h1.clone()));
    }

    #[test]
    fn test_meet_with_unknown() {
        let unknown = Hyp::unknown();
        let hyp = Hyp::new(vec![atom_diagnosis_a()]);

        // Meet of any hypothesis with Unknown is the hypothesis itself.
        assert_eq!(hyp.meet(&unknown), Some(hyp.clone()));
        assert_eq!(unknown.meet(&hyp), Some(hyp.clone()));
    }

    #[test]
    fn test_meet_incomparable_none() {
        let h1 = Hyp::new(vec![atom_diagnosis_a()]);
        let h2 = Hyp::new(vec![atom_diagnosis_b()]);

        // Incomparable hypotheses have no meet.
        assert_eq!(h1.meet(&h2), None);
    }

    #[test]
    fn test_hyp_duplicate_atoms_deduplicated() {
        let atom = atom_diagnosis_a();
        // Create a Hyp with the same atom twice.
        let h_with_dup = Hyp::new(vec![atom.clone(), atom.clone()]);
        let h_deduped = Hyp::new(vec![atom]);

        // After deduplication, they should be equal.
        assert_eq!(h_with_dup, h_deduped);
        // And they should have the same ordering comparison.
        assert_eq!(h_with_dup.partial_cmp(&h_deduped), Some(Ordering::Equal));
        // And meet should work (not return None).
        assert_eq!(h_with_dup.meet(&h_deduped), Some(h_deduped));
    }

    #[test]
    fn test_hyp_atoms_sorted_deterministically() {
        let atom_a = atom_diagnosis_a();
        let atom_b = atom_diagnosis_b();
        let atom_s = atom_severity_high();

        // Create hypotheses with atoms in different orders.
        let h1 = Hyp::new(vec![atom_s.clone(), atom_a.clone(), atom_b.clone()]);
        let h2 = Hyp::new(vec![atom_b.clone(), atom_s.clone(), atom_a.clone()]);

        // After normalization, they should be equal (same sorted order).
        assert_eq!(h1, h2);
        // And atoms() should return them in sorted order.
        let atoms = h1.atoms();
        assert_eq!(atoms.len(), 3);
        // Atoms should be sorted by (system, code).
        for i in 1..atoms.len() {
            assert!(
                (atoms[i - 1].system, &atoms[i - 1].code) <= (atoms[i].system, &atoms[i].code),
                "atoms not sorted: {:?} > {:?}",
                atoms[i - 1],
                atoms[i]
            );
        }
    }
}
