// Spec: specs/111-a-move-is-a-reviewed-mapping/spec.md
//! A move (spec 111 §3.1): the spec performing a relocation, split, merge or
//! removal of a path declares it here, as authored, reviewed data. The
//! declaration lives with the moving spec; there is no second, authored,
//! corpus-wide path map. Nothing here infers a move: no similarity, rename
//! detection or content comparison (§3.6).
//!
//! The authored shape and the compiled shape are the same type: a declaration
//! is carried into the registry record verbatim except for `answered_by`'s
//! spec half, which `compile` normalizes to its full id, the way it already
//! does for `depends_on` (spec 111 §3.2 by way of spec 015). The lookup
//! derived from every spec's declarations lives in `spec_spine_core::moves`.

use serde::{Deserialize, Serialize};

/// The four move kinds (spec 111 §3.1). There is no fifth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MoveKind {
    /// One path to one path.
    Relocated,
    /// One path to two or more.
    Split,
    /// Two or more paths to one.
    Merged,
    /// A path with no successor; `to` is `null`.
    Removed,
}

impl MoveKind {
    /// The frontmatter spelling, for messages.
    pub fn label(self) -> &'static str {
        match self {
            MoveKind::Relocated => "relocated",
            MoveKind::Split => "split",
            MoveKind::Merged => "merged",
            MoveKind::Removed => "removed",
        }
    }
}

/// One or more repo-relative paths on one side of a [`MoveDeclaration`]
/// (spec 111 §3.1): a single path (the common case) or a list, for `split`'s
/// `to` and `merged`'s `from`. Untagged: a bare YAML scalar parses as one
/// path, a sequence as several, matching the frontmatter's own shape rather
/// than forcing every `relocated` entry to write a one-element list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MovePaths {
    One(String),
    Many(Vec<String>),
}

impl MovePaths {
    /// Every path this side names, in authored order.
    pub fn paths(&self) -> Vec<&str> {
        match self {
            MovePaths::One(p) => vec![p.as_str()],
            MovePaths::Many(ps) => ps.iter().map(String::as_str).collect(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            MovePaths::One(_) => 1,
            MovePaths::Many(ps) => ps.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// One declared move (spec 111 §3.1): authored in the frontmatter of the spec
/// performing it.
///
/// `deny_unknown_fields`: a misspelt member is refused as malformed
/// frontmatter (`V-002`), the way [`crate::Obligation`] and [`crate::Impact`]
/// are.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoveDeclaration {
    /// The path(s) as they were. Need not exist at head.
    pub from: MovePaths,
    /// The path(s) as they are, `null` for `removed`, or the key omitted
    /// (`serde`'s ordinary `Option<T>` reading, both spellings equally
    /// `None`). Never `skip_serializing_if`: a `removed` entry keeps an
    /// explicit `to: null` in the *registry* record rather than the member
    /// silently vanishing, even when the author omitted the key.
    pub to: Option<MovePaths>,
    pub kind: MoveKind,
    /// Meaningful only under `removed` (§3.1); present under any other kind
    /// is refused (`V-040`). The spec this path's former responsibility now
    /// belongs to.
    ///
    /// Authored as `answered_by` (matching every other snake_case key in this
    /// grammar) and carried as `answeredBy` in the registry (the DTO's
    /// camelCase convention, spec 001 §3.2), the way
    /// [`crate::Conflict::settled_by`] is. `rename` sets the canonical/
    /// emitted spelling and `alias` accepts the authored one on read; this
    /// type is never re-serialized as YAML, so the two spellings never clash.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "answeredBy",
        alias = "answered_by"
    )]
    pub answered_by: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_scalar_and_a_sequence_both_parse() {
        let one: MovePaths = serde_yaml::from_str("\"a/b.rs\"").unwrap();
        assert_eq!(one, MovePaths::One("a/b.rs".to_string()));
        assert_eq!(one.paths(), vec!["a/b.rs"]);

        let many: MovePaths = serde_yaml::from_str("[\"a/b.rs\", \"a/c.rs\"]").unwrap();
        assert_eq!(
            many,
            MovePaths::Many(vec!["a/b.rs".to_string(), "a/c.rs".to_string()])
        );
        assert_eq!(many.paths(), vec!["a/b.rs", "a/c.rs"]);
    }

    #[test]
    fn a_removed_entry_round_trips_an_explicit_null() {
        let decl: MoveDeclaration = serde_yaml::from_str(
            "from: \"a/b.rs\"\nto: null\nkind: removed\nanswered_by: \"001\"\n",
        )
        .unwrap();
        assert_eq!(decl.to, None);
        let value = serde_json::to_value(&decl).unwrap();
        assert!(value.get("to").is_some());
        assert!(value["to"].is_null());
    }

    #[test]
    fn a_missing_to_key_parses_the_same_as_an_explicit_null() {
        let decl: MoveDeclaration =
            serde_yaml::from_str("from: \"a/b.rs\"\nkind: removed\n").unwrap();
        assert_eq!(decl.to, None);
    }

    #[test]
    fn an_unknown_member_is_refused() {
        let err = serde_yaml::from_str::<MoveDeclaration>(
            "from: \"a/b.rs\"\nto: \"a/c.rs\"\nkind: relocated\nreviewed_by: \"someone\"\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("reviewed_by"), "{err}");
    }
}
