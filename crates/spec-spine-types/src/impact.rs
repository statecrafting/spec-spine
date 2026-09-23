// Spec: specs/109-impact-and-conflict-are-declared/spec.md
//! Impact and conflict (spec 109): a spec's declared relation to another
//! spec's obligation (spec 106), and a spec's declared, unresolved
//! disagreement with one. Nothing here computes either; both are exactly what
//! an author wrote down (§3.5).
//!
//! One shared type per shape, reused unchanged between the authored
//! frontmatter and the compiled registry record (the way [`crate::Obligation`]
//! is): the compiler normalizes `obligation` to its full qualified form before
//! the record is built, in place, the way it already does for `depends_on`.

use serde::{Deserialize, Serialize};

/// The four impact natures (spec 109 §3.2). There is no fifth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImpactNature {
    /// The obligation still holds, more precisely.
    Refines,
    /// The obligation still holds, over more surface.
    Extends,
    /// The obligation is replaced by one this spec declares.
    Supersedes,
    /// The obligation is unaffected but a reader of it should know.
    Informs,
}

/// One declared impact (spec 109 §3.1, §3.2): this spec's relation to an
/// obligation another spec (or, under `supersedes`, this spec itself once
/// its successor lands) declares.
///
/// `deny_unknown_fields`: an unknown member is refused as malformed
/// frontmatter, as spec 106's obligation members are.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Impact {
    /// A qualified `<spec>#<obligation-id>` reference (spec 106 §3.6). As
    /// authored this may use a short spec id; the registry record carries the
    /// full qualified form (§3.7), when it resolves.
    pub obligation: String,
    pub nature: ImpactNature,
    /// Required, and meaningful, only when `nature` is `supersedes` (§3.2):
    /// the id of a non-withdrawn obligation THIS spec declares. Present under
    /// any other `nature` is refused (`V-026`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// The three conflict resolutions (spec 109 §3.3). There is no fourth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictResolution {
    /// The divergence is intended and `reason` says why.
    Deliberate,
    /// The contradiction is known and nobody has decided.
    Unresolved,
    /// A named spec, `settled_by`, is expected to settle it.
    Pending,
}

/// One declared conflict (spec 109 §3.1, §3.3): this spec knowingly diverges
/// from an obligation another spec declares, without resolving it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conflict {
    /// A qualified `<spec>#<obligation-id>` reference, as [`Impact::obligation`].
    pub obligation: String,
    pub reason: String,
    pub resolution: ConflictResolution,
    /// Present exactly when `resolution` is `pending` (§3.3), and a spec id.
    ///
    /// Authored as `settled_by` (the frontmatter spelling, matching every
    /// other snake_case key in this grammar) and carried as `settledBy` in the
    /// registry (the DTO's camelCase convention, spec 001 §3.2). `rename` sets
    /// the canonical/emitted spelling and `alias` accepts the authored one on
    /// read, the way [`crate::Implementation`]'s `n-a`/`n/a` pair does; this
    /// type is never re-serialized as YAML, so the two spellings never clash.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "settledBy",
        alias = "settled_by"
    )]
    pub settled_by: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_snake_case_and_registry_camel_case_both_parse() {
        let snake: Conflict = serde_yaml::from_str(
            "obligation: \"005#R-1\"\nreason: \"why\"\nresolution: pending\nsettled_by: \"006\"\n",
        )
        .unwrap();
        assert_eq!(snake.settled_by.as_deref(), Some("006"));

        let camel: Conflict = serde_json::from_value(serde_json::json!({
            "obligation": "005#R-1",
            "reason": "why",
            "resolution": "pending",
            "settledBy": "006",
        }))
        .unwrap();
        assert_eq!(camel, snake);

        let value = serde_json::to_value(&snake).unwrap();
        assert_eq!(value["settledBy"], "006");
        assert!(value.get("settled_by").is_none());
    }
}
