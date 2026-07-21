//! Institutional capacity hypotheses as a refinable poset.
//!
//! A `Cap` represents an institution's resource-commitment state at some level of
//! specificity. Capacity hypotheses are ordered by commitment: `c1 ⊑_IS c2` reads
//! "c1 is at least as committed as c2" — more resources allocated, fewer degrees of
//! freedom remaining.
//!
//! Per [SPEC.md § 3.1] (DEF-IS-01, DEF-IS-02):
//! The institutional state space mirrors the patient-state space (`Hyp`, § 2) in
//! *form* — both are posets ordered by specificity, both carry a distinguished top
//! element — but differs in *substance*: institutional commitments are inherently
//! quantitative (how many beds, how many slots), so `⊑_IS` is a pointwise-dominance
//! order over resource commitment counts, not a set-inclusion order over discrete
//! facts as `Hyp`'s `⊑` is over atoms.
//!
//! Per MC-1 (§0.3), `H_IS` is a poset, not a lattice: meet is partial (defined only
//! for `compat_IS`-related pairs), not total. `compat_IS` and `⊓_IS` are out of
//! scope for this module — see DEF-IS-01's compatibility predicate (depends on the
//! physical capacity bound, DEF-IS-04, not yet introduced) and INV-IS-02 (meet).
//!
//! The concrete representation of *which* resources exist (`Resource`,
//! `ResourceBoundedSet`, DEF-IS-03) is deferred to § 3.2, per DEF-IS-01's own text
//! ("concrete representation in §3.2"). This module uses a resource identifier
//! (`String`) as a forward-compatible placeholder; a later revision may narrow
//! `Commitment::resource_id` to a validated `Resource` type without changing the
//! poset structure defined here.

use std::cmp::Ordering;
use std::collections::BTreeMap;

/// A single committed unit of a resource within a capacity hypothesis.
///
/// `resource_id` is an opaque identifier for now (see module docs); `committed_units`
/// is the quantity of that resource currently committed under this hypothesis.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Commitment {
    pub resource_id: String,
    pub committed_units: u32,
}

/// An institutional capacity hypothesis: a refinable set of resource commitments.
///
/// Capacity hypotheses are ordered by commitment: `c1 ⊑_IS c2` means c1 is at least
/// as committed as c2 (c1 commits at least as many units of every resource c2
/// commits, treating an absent resource as 0 committed units). This ordering forms
/// a partially ordered set (poset) where:
/// - [`Cap::top`] (⊤_IS, "fully uncommitted") is the top element (least committed).
/// - Each concrete capacity hypothesis carries a set of [`Commitment`]s.
///
/// Implements DEF-IS-01 (institutional state space) and DEF-IS-02 (top institutional
/// hypothesis).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Cap(Vec<Commitment>);

impl Cap {
    /// Creates a new capacity hypothesis from a list of commitments.
    ///
    /// Commitments are merged by `resource_id` (duplicate entries for the same
    /// resource sum their `committed_units`, since committing 2 units and then 1
    /// more unit of the same resource is 3 units committed, not two independent
    /// facts). Zero-unit entries are dropped after merging, so a `Cap` explicitly
    /// committing 0 units of a resource is indistinguishable from one that never
    /// mentions it — this keeps `PartialEq`/`Hash` consistent with `PartialOrd`
    /// (both treat "absent" and "0 committed" identically), mirroring `Hyp::new`'s
    /// deduplication discipline.
    pub fn new(commitments: Vec<Commitment>) -> Self {
        let mut merged: BTreeMap<String, u32> = BTreeMap::new();
        for c in commitments {
            let entry = merged.entry(c.resource_id).or_insert(0);
            *entry = entry.saturating_add(c.committed_units);
        }
        let normalized: Vec<Commitment> = merged
            .into_iter()
            .filter(|(_, units)| *units > 0)
            .map(|(resource_id, committed_units)| Commitment {
                resource_id,
                committed_units,
            })
            .collect();
        Cap(normalized)
    }

    /// Creates the top element (⊤_IS, "fully uncommitted"): all resources available.
    ///
    /// Per DEF-IS-02: `∀c ∈ Cap. c ⊑_IS ⊤_IS`. This is the upper reference point,
    /// never the actual institutional state in operation.
    pub fn top() -> Self {
        Cap(vec![])
    }

    /// Checks if this is the `top` capacity hypothesis (⊤_IS, fully uncommitted).
    pub fn is_top(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the commitments in this capacity hypothesis.
    pub fn commitments(&self) -> &[Commitment] {
        &self.0
    }

    /// Returns this capacity hypothesis's commitments as a `resource_id -> units`
    /// map, for pointwise comparison against another `Cap`'s map (absent = 0).
    ///
    /// Shared by [`PartialOrd`] here; intended for reuse by `compat_IS` (task 13.2)
    /// and `⊓_IS` meet (task 13.3), which need the same "absence = 0" lookup — see
    /// module docs.
    pub(crate) fn commitment_map(&self) -> BTreeMap<&str, u32> {
        self.0
            .iter()
            .map(|c| (c.resource_id.as_str(), c.committed_units))
            .collect()
    }

    /// Units of `resource_id` committed under this hypothesis (0 if absent).
    pub fn units_for(&self, resource_id: &str) -> u32 {
        self.0
            .iter()
            .find(|c| c.resource_id == resource_id)
            .map(|c| c.committed_units)
            .unwrap_or(0)
    }
}

impl PartialOrd for Cap {
    /// Compares two capacity hypotheses by commitment order via pointwise dominance.
    ///
    /// `self ⊑_IS other` (self is at least as committed as other) iff, for every
    /// resource `other` commits units to, `self` commits at least as many units to
    /// that same resource (absent resources count as 0). This generalizes `Hyp`'s
    /// atom-set-inclusion order from presence/absence facts to quantities.
    ///
    /// - `Some(Less)`: `self` is strictly more committed than `other` (dominates
    ///   pointwise, differs somewhere) — mirrors `Hyp`'s "more atoms = more
    ///   refined = Less" convention.
    /// - `Some(Greater)`: `other` is strictly more committed than `self`.
    /// - `Some(Equal)`: commitment counts coincide (after the union of both
    ///   resource sets, treating absence as 0).
    /// - `None`: neither dominates the other (e.g., each commits to a resource the
    ///   other does not).
    ///
    /// Implements DEF-IS-01's `⊑_IS` order.
    fn partial_cmp(&self, other: &Cap) -> Option<Ordering> {
        let self_map = self.commitment_map();
        let other_map = other.commitment_map();

        let mut self_dominates = true; // self[r] >= other[r] for every r
        let mut other_dominates = true; // other[r] >= self[r] for every r

        // Union of both key sets, each visited once (BTreeMap keys are already sorted).
        let all_keys = self_map.keys().copied().chain(
            other_map
                .keys()
                .copied()
                .filter(|k| !self_map.contains_key(k)),
        );
        for key in all_keys {
            let self_units = *self_map.get(key).unwrap_or(&0);
            let other_units = *other_map.get(key).unwrap_or(&0);
            if self_units < other_units {
                self_dominates = false;
            }
            if other_units < self_units {
                other_dominates = false;
            }
        }

        match (self_dominates, other_dominates) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            (false, false) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(resource_id: &str, units: u32) -> Commitment {
        Commitment {
            resource_id: resource_id.to_string(),
            committed_units: units,
        }
    }

    #[test]
    fn test_top_is_fully_uncommitted() {
        let top = Cap::top();
        assert!(top.is_top());
        assert!(top.commitments().is_empty());
    }

    #[test]
    fn test_any_cap_refines_top_def_is_02() {
        // DEF-IS-02: ∀c ∈ Cap. c ⊑_IS ⊤_IS.
        let top = Cap::top();
        let committed = Cap::new(vec![commit("ICU_BED", 3)]);

        // committed is at least as committed as top (strictly more) => Less.
        assert_eq!(committed.partial_cmp(&top), Some(Ordering::Less));
        // top is at least as committed as committed only if committed == top; it is
        // not, so top ⊑_IS committed does not hold => Greater (top is less committed).
        assert_eq!(top.partial_cmp(&committed), Some(Ordering::Greater));
    }

    #[test]
    fn test_top_equals_itself() {
        assert_eq!(Cap::top().partial_cmp(&Cap::top()), Some(Ordering::Equal));
    }

    #[test]
    fn test_equal_capacity_hypotheses() {
        let c1 = Cap::new(vec![commit("ICU_BED", 2), commit("OR_SLOT", 1)]);
        let c2 = Cap::new(vec![commit("OR_SLOT", 1), commit("ICU_BED", 2)]);

        assert_eq!(c1, c2);
        assert_eq!(c1.partial_cmp(&c2), Some(Ordering::Equal));
    }

    #[test]
    fn test_incomparable_disjoint_commitments() {
        // c1 commits only to ICU_BED, c2 commits only to OR_SLOT: neither
        // dominates the other pointwise.
        let c1 = Cap::new(vec![commit("ICU_BED", 1)]);
        let c2 = Cap::new(vec![commit("OR_SLOT", 1)]);

        assert_eq!(c1.partial_cmp(&c2), None);
        assert_eq!(c2.partial_cmp(&c1), None);
    }

    #[test]
    fn test_incomparable_conflicting_dominance_on_shared_resources() {
        // c1 dominates on A (3 >= 1) but c2 dominates on B (2 >= 1): neither fully
        // dominates. Distinct code path from the fully-disjoint-resources case
        // above (this one shares keys, just conflicts on direction per key).
        let c1 = Cap::new(vec![commit("A", 3), commit("B", 1)]);
        let c2 = Cap::new(vec![commit("A", 1), commit("B", 2)]);

        assert_eq!(c1.partial_cmp(&c2), None);
        assert_eq!(c2.partial_cmp(&c1), None);
    }

    #[test]
    fn test_units_for_and_commitment_map() {
        let c = Cap::new(vec![commit("ICU_BED", 2)]);

        assert_eq!(c.units_for("ICU_BED"), 2);
        assert_eq!(c.units_for("OR_SLOT"), 0); // absent = 0
        assert_eq!(c.commitment_map().get("ICU_BED"), Some(&2));
    }

    #[test]
    fn test_more_units_same_resource_is_more_committed() {
        // Quantity-aware ordering: committing MORE units of the SAME resource is
        // strictly more committed, unlike Hyp's presence/absence atom sets which
        // cannot distinguish this.
        let c_more = Cap::new(vec![commit("ICU_BED", 3)]);
        let c_less = Cap::new(vec![commit("ICU_BED", 1)]);

        assert_eq!(c_more.partial_cmp(&c_less), Some(Ordering::Less));
        assert_eq!(c_less.partial_cmp(&c_more), Some(Ordering::Greater));
    }

    #[test]
    fn test_additional_resource_commitment_is_more_committed() {
        // c1 commits to everything c2 does, plus one more resource: c1 dominates.
        let c2 = Cap::new(vec![commit("ICU_BED", 1)]);
        let c1 = Cap::new(vec![commit("ICU_BED", 1), commit("OR_SLOT", 1)]);

        assert_eq!(c1.partial_cmp(&c2), Some(Ordering::Less));
        assert_eq!(c2.partial_cmp(&c1), Some(Ordering::Greater));
    }

    #[test]
    fn test_construction_merges_duplicate_resource_entries() {
        let from_duplicates = Cap::new(vec![commit("ICU_BED", 1), commit("ICU_BED", 2)]);
        let from_merged = Cap::new(vec![commit("ICU_BED", 3)]);

        assert_eq!(from_duplicates, from_merged);
    }

    #[test]
    fn test_construction_drops_zero_unit_entries() {
        let with_zero = Cap::new(vec![commit("ICU_BED", 0)]);
        let top = Cap::top();

        // An explicit 0-unit commitment is indistinguishable from no commitment,
        // keeping PartialEq/Hash consistent with the "absence = 0" PartialOrd rule.
        assert_eq!(with_zero, top);
        assert!(with_zero.commitments().is_empty());
    }

    #[test]
    fn test_commitments_sorted_deterministically() {
        let c1 = Cap::new(vec![commit("OR_SLOT", 1), commit("ICU_BED", 2)]);
        let c2 = Cap::new(vec![commit("ICU_BED", 2), commit("OR_SLOT", 1)]);

        assert_eq!(c1, c2);
        let commitments = c1.commitments();
        assert_eq!(commitments.len(), 2);
        for i in 1..commitments.len() {
            assert!(
                commitments[i - 1].resource_id <= commitments[i].resource_id,
                "commitments not sorted: {:?} > {:?}",
                commitments[i - 1],
                commitments[i]
            );
        }
    }
}
