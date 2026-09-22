// Spec: specs/106-obligations-are-declared-constraints/spec.md
//! Obligations (spec 106): a spec's requirements, invariants and verifications
//! as addressable, declared constraints.
//!
//! The authored shape and the compiled shape are the same type: an obligation
//! is carried into the registry record verbatim, and the section it points at
//! is identified by the record's `sectionDigests`, not by a copy here.

use serde::{Deserialize, Serialize};

/// The three constraint kinds (spec 106 §3.2). There is no fourth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObligationKind {
    /// Something the implementation MUST do.
    Requirement,
    /// Something that MUST remain true across every state.
    Invariant,
    /// Something an executable check MUST establish.
    Verification,
}

/// One declared obligation (spec 106 §3.1).
///
/// `deny_unknown_fields`: a misspelt member is refused as malformed
/// frontmatter (`V-002`) rather than silently dropped, because a dropped
/// `withdrawn` would resurrect an obligation its author retired.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    /// Unique within its spec; never contains `#` (§3.3).
    pub id: String,
    pub kind: ObligationKind,
    pub text: String,
    /// A heading slug in the declaring spec's body (§3.4). Required.
    pub anchor: String,
    /// Declared evidence for a `verification`; empty for every other kind
    /// (§3.7). Never inferred.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<String>,
    /// Withdrawn in place: the id stays occupied so it cannot be reused
    /// (§3.3). Omitted when false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub withdrawn: bool,
}

/// The id grammar (§3.3): `^[A-Za-z][A-Za-z0-9]*(-[A-Za-z0-9]+)*$`. It can
/// never contain the `#` a qualified reference splits on.
pub fn valid_obligation_id(id: &str) -> bool {
    let mut parts = id.split('-');
    let Some(first) = parts.next() else {
        return false;
    };
    let mut chars = first.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    parts.all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// A qualified obligation reference, `<spec>#<obligation-id>` (§3.6), split.
///
/// `None` when the reference is unqualified: no `#`, more than one, or an
/// empty half. The caller refuses that; it never resolves it locally.
pub fn split_obligation_ref(reference: &str) -> Option<(&str, &str)> {
    let (spec, id) = reference.split_once('#')?;
    if spec.trim().is_empty() || id.trim().is_empty() || id.contains('#') {
        return None;
    }
    Some((spec, id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_grammar() {
        for ok in ["R-1", "I-1", "V-12", "REQ-auth-3", "R1", "a"] {
            assert!(valid_obligation_id(ok), "{ok}");
        }
        for bad in ["", "1-R", "R-", "-R", "R--1", "R#1", "R 1", "R_1", "R-1-"] {
            assert!(!valid_obligation_id(bad), "{bad}");
        }
    }

    #[test]
    fn a_reference_is_qualified_or_refused() {
        assert_eq!(split_obligation_ref("106#R-1"), Some(("106", "R-1")));
        assert_eq!(
            split_obligation_ref("106-obligations#V-1"),
            Some(("106-obligations", "V-1"))
        );
        for bad in ["R-1", "#R-1", "106#", "106#R#1", " #R-1"] {
            assert_eq!(split_obligation_ref(bad), None, "{bad}");
        }
    }
}
