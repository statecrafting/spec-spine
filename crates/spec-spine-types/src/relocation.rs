// Spec: specs/142-a-relocation-is-proven/spec.md
//! A section relocation (spec 142 §3.2): a spec that takes over a section of
//! another spec's text declares where it came from, so `delta` can prove, against
//! the merge base, that the text arrived unchanged and classify the
//! predecessor's removal of it as `relocation` rather than `requirement`.
//!
//! This is text, not code: a code path's move is spec 111's `moves`. Nothing
//! here infers a relocation; a relocation not declared is not one, and a
//! declared relocation whose text differs is not proven.

use serde::{Deserialize, Serialize};

/// One declared relocation, authored in the frontmatter of the receiving spec.
///
/// `deny_unknown_fields`: a misspelt member is refused as malformed frontmatter
/// (`V-002`), as [`crate::MoveDeclaration`] is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relocation {
    /// The spec the text came from. A short id resolves like `depends_on`
    /// (spec 015); compile normalizes it to the full id.
    pub spec: String,
    /// The anchor of the section in `spec` as it was at the merge base.
    pub from: String,
    /// The anchor of the section in this spec that now holds the text. Omitted
    /// when it is the same as `from`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

impl Relocation {
    /// The receiving section's anchor: `to`, or `from` when `to` is absent.
    pub fn to_anchor(&self) -> &str {
        self.to.as_deref().unwrap_or(&self.from)
    }
}
