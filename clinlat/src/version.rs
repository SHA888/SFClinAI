//! Version tuple for tracking operator and build versions in provenance chains.

use serde::{Deserialize, Serialize};

/// A version tuple identifying the system, operator, and build version.
///
/// Per SPEC.md §2.4 (DEF-PS-13), this type enables version-respecting derivation chains
/// (INV-PS-05) and supports audit queries that filter hypotheses by operator version.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ver {
    /// System identifier (e.g., "clinlat").
    pub system: String,

    /// Operator identifier (e.g., "sofa_resp", "kdigo_aki").
    pub operator: String,

    /// Build version as SemVer string (e.g., "0.2.0", "0.3.1").
    pub build: String,
}

impl Ver {
    /// Create a new version tuple.
    pub fn new(
        system: impl Into<String>,
        operator: impl Into<String>,
        build: impl Into<String>,
    ) -> Self {
        Self {
            system: system.into(),
            operator: operator.into(),
            build: build.into(),
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ver_construction() {
        let v = Ver::new("clinlat", "sofa_resp", "0.2.0");
        assert_eq!(v.system, "clinlat");
        assert_eq!(v.operator, "sofa_resp");
        assert_eq!(v.build, "0.2.0");
    }

    #[test]
    fn test_ver_equality() {
        let v1 = Ver::new("clinlat", "sofa_resp", "0.2.0");
        let v2 = Ver::new("clinlat", "sofa_resp", "0.2.0");
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_ver_inequality() {
        let v1 = Ver::new("clinlat", "sofa_resp", "0.2.0");
        let v2 = Ver::new("clinlat", "sofa_resp", "0.3.0");
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_ver_to_json() {
        let v = Ver::new("clinlat", "sofa_resp", "0.2.0");
        let json = v.to_json().expect("serialization failed");
        assert!(json.contains("\"system\":\"clinlat\""));
        assert!(json.contains("\"operator\":\"sofa_resp\""));
        assert!(json.contains("\"build\":\"0.2.0\""));
    }

    #[test]
    fn test_ver_from_json() {
        let json = r#"{"system":"clinlat","operator":"sofa_resp","build":"0.2.0"}"#;
        let v = Ver::from_json(json).expect("deserialization failed");
        assert_eq!(v.system, "clinlat");
        assert_eq!(v.operator, "sofa_resp");
        assert_eq!(v.build, "0.2.0");
    }

    #[test]
    fn test_ver_json_round_trip() {
        let original = Ver::new("clinlat", "kdigo_aki", "0.2.1");
        let json = original.to_json().expect("serialization failed");
        let restored = Ver::from_json(&json).expect("deserialization failed");
        assert_eq!(original, restored);
    }

    #[test]
    fn test_ver_hash() {
        let v1 = Ver::new("clinlat", "sofa_resp", "0.2.0");
        let v2 = Ver::new("clinlat", "sofa_resp", "0.2.0");
        // Ensure both can be hashed and produce the same hash
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(v1);
        set.insert(v2);
        assert_eq!(set.len(), 1); // Both should be the same hash
    }
}
