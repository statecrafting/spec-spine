//! The change-classification report (spec 071).
//!
//! `couple` answers one question about a change: did owned code move without an
//! owning `spec.md` in the same diff. Spec 071 §1 measures two changes that pass
//! it and should never be approved on the candidate's own say-so: a candidate
//! deleting a command from the `## Verification` block that judges it, and a
//! candidate claiming another spec's unit with a new `extends` edge. Both are
//! also the sanctioned shapes of legitimate work, which is why note 02 G7 refuses
//! to turn either into a refusal before a design exists.
//!
//! What this module carries instead is a record: every changed path, the
//! structural classes it falls into **under the merge base's rules**, and the
//! classes a consumer must judge under the base's policy. It decides nothing.
//!
//! These are read-side report shapes, not committed artifacts. The report is
//! versioned on its own axis by [`DELTA_SCHEMA_VERSION`], and the `--json`
//! envelope around it by `VERDICT_SCHEMA_VERSION`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::attest::ToolStamp;

pub use crate::version::DELTA_SCHEMA_VERSION;

/// The value of [`DeltaReport::classified_under`]. The only side the candidate
/// did not write (spec 071 D-1).
pub const CLASSIFIED_UNDER_BASE: &str = "base";

/// One structural class a changed path can carry (spec 071 §3.3).
///
/// Declared in alphabetical order so the derived `Ord` sorts by token: every
/// list of classes in a report is emitted in that order, which is what makes two
/// reports of the same change byte-identical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeltaClass {
    /// A `spec.md` whose typed edges or `depends_on` changed, or any path whose
    /// owner set differs between base and head.
    Authority,
    /// Covered by the base's bypass floor and configured prefixes, and by no
    /// other class.
    Bypassed,
    /// A path under the base's `standards_dir`, or a `spec.md` declaring
    /// `unamendable` anchors at base or head.
    Constitutional,
    /// A path under the base's `derived_dir`.
    Derived,
    /// A path with an owner at base or head that is none of the special kinds.
    Implementation,
    /// `status`, `implementation`, `superseded_by` or `retirement_rationale`
    /// changed.
    Lifecycle,
    /// `spec-spine.toml`, or a path matched by the base's
    /// `[index] extra_hashed_inputs`.
    Policy,
    /// A `spec.md` whose body outside `## Verification` changed only by losing
    /// sections another spec proved it received unchanged (spec 142 §3.3).
    /// Carried instead of `requirement` for that body change, never with it.
    Relocation,
    /// A `spec.md` whose body outside `## Verification` changed, or whose
    /// frontmatter changed in a key no other class names.
    Requirement,
    /// A `spec.md` or configuration that fails to parse on either side, or a
    /// change the rules cannot place.
    Unknown,
    /// No owner at base or head, not bypassed, and none of the special kinds.
    Unowned,
    /// A `spec.md` whose `verify:cli` plan differs between base and head,
    /// including a spec added or removed.
    Verification,
}

impl DeltaClass {
    /// Every class, in token order.
    pub const ALL: [DeltaClass; 12] = [
        DeltaClass::Authority,
        DeltaClass::Bypassed,
        DeltaClass::Constitutional,
        DeltaClass::Derived,
        DeltaClass::Implementation,
        DeltaClass::Lifecycle,
        DeltaClass::Policy,
        DeltaClass::Relocation,
        DeltaClass::Requirement,
        DeltaClass::Unknown,
        DeltaClass::Unowned,
        DeltaClass::Verification,
    ];

    /// Whether a consumer must judge this class under the base's policy
    /// (spec 071 §3.5).
    ///
    /// `implementation`, `derived`, `bypassed` and `unowned` are not listed.
    /// That is a statement about structure only: a path carrying none of the
    /// listed classes changed nothing the corpus models as a rule, which is not
    /// a statement that the change is safe, correct or approved.
    pub fn requires_prior_policy(self) -> bool {
        matches!(
            self,
            DeltaClass::Requirement
                | DeltaClass::Verification
                | DeltaClass::Authority
                | DeltaClass::Lifecycle
                | DeltaClass::Constitutional
                | DeltaClass::Policy
                | DeltaClass::Unknown
        )
    }
}

/// Whether a path exists on each side of the change.
///
/// Reported, not inferred from: with renames disabled a move is a `deleted`
/// entry and an `added` one, and the report never says the two are one file
/// (spec 071 §3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeKind {
    Added,
    Deleted,
    Modified,
}

/// The three commit ids a report is about. Echoed, never computed: the core
/// reads trees, and the caller resolved these from git.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaCommits {
    /// The base ref as given, resolved to a commit.
    pub base: String,
    /// `merge-base(base, head)`: the tree whose rules classify.
    pub merge_base: String,
    /// The head ref, resolved to a commit.
    pub head: String,
}

/// The classification of one change (spec 071 §3.6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaReport {
    /// [`DELTA_SCHEMA_VERSION`] at the time of emission.
    pub schema_version: String,
    pub tool: ToolStamp,
    /// Always [`CLASSIFIED_UNDER_BASE`].
    pub classified_under: String,
    pub base: String,
    pub merge_base: String,
    pub head: String,
    /// One entry per changed path, sorted by path.
    pub changes: Vec<DeltaChange>,
    /// How many paths carry each class. Every class is present, zero included,
    /// so a consumer reads a count rather than inferring one from absence.
    pub counts: BTreeMap<DeltaClass, usize>,
    pub prior_policy: PriorPolicy,
    /// Every declared section relocation whose source or receiving `spec.md`
    /// this change touches, and whether it was proven (spec 142 §3.3). Omitted
    /// when there are none, so a report about a change with no relocation is
    /// byte-identical to one before 0.2.0 except for `schemaVersion`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relocations: Vec<RelocationCheck>,
}

/// One declared relocation, checked against the merge base (spec 142 §3.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelocationCheck {
    /// The spec declaring the relocation (the receiver).
    pub spec: String,
    /// The source spec and its section anchor, as at the merge base.
    pub from_spec: String,
    pub from: String,
    /// The receiving section's anchor at head.
    pub to: String,
    /// True when the source section at the merge base and the receiving
    /// section at head have the same relocation digest.
    pub proven: bool,
    /// Why it was not proven; absent when it was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// One changed path and every class that applies to it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaChange {
    pub path: String,
    pub change: ChangeKind,
    /// At least one class, in token order.
    pub classes: Vec<DeltaClass>,
    /// Present exactly when `classes` holds `verification`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationDelta>,
    /// Present exactly when `classes` holds `authority`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<AuthorityDelta>,
    /// Present exactly when `classes` holds `lifecycle`: each changed key with
    /// its base and head value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<BTreeMap<String, ValueChange>>,
}

/// How a spec's declared acceptance moved (spec 071 §3.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationDelta {
    /// SHA-256 of the canonical JSON of the base plan's `commands` list.
    /// Absent when the spec does not exist at base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_plan_hash: Option<String>,
    /// The same digest at head. Absent when the spec does not exist at head.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_plan_hash: Option<String>,
    /// Commands present at base and not at head, counted with multiplicity.
    pub commands_only_in_base: usize,
    /// Commands present at head and not at base, counted with multiplicity.
    pub commands_only_in_head: usize,
}

/// How authority over a path moved (spec 071 §3.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityDelta {
    /// The owners the base index gives this path, whole-file, sorted.
    pub base_owners: Vec<String>,
    /// The owners the head tree resolves to under the base's rules, sorted.
    pub head_owners: Vec<String>,
    /// For a `spec.md`: the edge items only at head, keyed by frontmatter key
    /// (the eight typed edges, and `depends_on`). Omitted when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub edges_added: BTreeMap<String, Vec<serde_json::Value>>,
    /// For a `spec.md`: the edge items only at base. Omitted when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub edges_removed: BTreeMap<String, Vec<serde_json::Value>>,
}

/// One frontmatter key's value on each side. `null` means absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueChange {
    pub base: serde_json::Value,
    pub head: serde_json::Value,
}

/// The classes a consumer must judge under the base revision's policy.
///
/// `required: false` means only that no class [`DeltaClass::requires_prior_policy`]
/// names is present. It does not mean the change is safe, correct or approved,
/// and spec-spine never evaluates the approval (spec 071 §3.5).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriorPolicy {
    pub required: bool,
    /// The listed classes present in this change, in token order.
    pub classes: Vec<DeltaClass>,
}
