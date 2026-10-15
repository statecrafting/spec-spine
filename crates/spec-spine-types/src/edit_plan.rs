//! Edit-plan request, operation, preview, conflict, refusal and plan types
//! (spec 165).
//!
//! A plan is a proposal: a closed list of governed operations, each guarded
//! by the digest of the target it was planned against, checked against
//! ownership, a work scope and the amendment rules, previewed as exact
//! post-image bytes, and classified by authority effect. Nothing in
//! spec-spine applies one. Its own schema axis is
//! [`crate::EDIT_PLAN_SCHEMA_VERSION`].

use serde::{Deserialize, Serialize};

use crate::{ContentSelector, PacketSnapshot, RepoPath, TraceDeclaration, TraceState, Unit};

/// Most operations one plan may carry; more refuses `plan-too-large` (spec 165
/// 3.15).
pub const EDIT_PLAN_MAX_OPERATIONS: usize = 64;
/// Most bytes a canonical plan may hold; more refuses `plan-too-large`.
pub const EDIT_PLAN_MAX_BYTES: usize = 4_194_304;
/// The `expected` literal for a file that must not exist (spec 165 3.9).
pub const EXPECTED_ABSENT: &str = "absent";

/// One edit-plan request (spec 165 3.2). Unknown members are refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditPlanRequest {
    /// A spec 108 work-scope document; its `ownSpec` is the plan's subject.
    pub scope: serde_json::Value,
    /// The operations, 1 through 64 (an empty list is a usage error).
    pub operations: Vec<EditOperation>,
    /// Spec 160 manifests the caller asserts describe outputs in this
    /// snapshot, used only to identify generated regions (3.10).
    #[serde(default)]
    pub manifests: Vec<serde_json::Value>,
    pub consumer_schema_version: String,
}

/// The edge an `add-claim` declares its unit on (spec 165 3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaimEdge {
    Establishes,
    Extends,
}

/// The closed operation vocabulary (spec 165 3.3). There is no sixth kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum EditOperation {
    /// One unit appended to the subject's `establishes`, or one `extends`
    /// edge.
    AddClaim {
        /// The subject spec, as a spec 155 `spec` selector.
        target: ContentSelector,
        expected: String,
        edge: ClaimEdge,
        unit: Unit,
        /// For `extends`: the spec the edge names.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spec: Option<String>,
        /// For `extends`: the edge's `nature`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nature: Option<String>,
    },
    /// One spec 169 `traceability` entry on the subject.
    AddTraceRelation {
        /// The subject spec, as a spec 155 `spec` selector.
        target: ContentSelector,
        expected: String,
        entry: TraceDeclaration,
    },
    /// A new draft spec that amends an approved one.
    ProposeAmendment {
        /// The new file, `specs/<id>/spec.md`.
        target: RepoPath,
        /// Always `absent`.
        expected: String,
        /// The new spec's full id.
        spec: String,
        /// The qualified obligations the new spec would change.
        obligations: Vec<String>,
        /// The new file's complete bytes, authored by the caller.
        content: String,
    },
    /// One documentation-manifest file, replaced whole.
    UpdateManifest {
        target: RepoPath,
        /// The file's recorded `manifestDigest`.
        expected: String,
        /// The complete replacement manifest.
        manifest: serde_json::Value,
        /// The replacement's complete context-packet page set, as exact
        /// strings in page order: what its packet and input digests are
        /// checked against (D-17).
        pages: Vec<String>,
    },
    /// One uniquely resolved spec or documentation unit's bytes.
    ReplaceUnit {
        /// A spec 155 `spec-section`, `owned-unit` or `file` selector.
        target: ContentSelector,
        expected: String,
        /// The span's complete new bytes, ending at a line boundary.
        content: String,
    },
}

impl EditOperation {
    /// The `op` wire name.
    pub fn kind(&self) -> &'static str {
        match self {
            EditOperation::AddClaim { .. } => "add-claim",
            EditOperation::AddTraceRelation { .. } => "add-trace-relation",
            EditOperation::ProposeAmendment { .. } => "propose-amendment",
            EditOperation::UpdateManifest { .. } => "update-manifest",
            EditOperation::ReplaceUnit { .. } => "replace-unit",
        }
    }

    /// The digest the caller read the target at.
    pub fn expected(&self) -> &str {
        match self {
            EditOperation::AddClaim { expected, .. }
            | EditOperation::AddTraceRelation { expected, .. }
            | EditOperation::ProposeAmendment { expected, .. }
            | EditOperation::UpdateManifest { expected, .. }
            | EditOperation::ReplaceUnit { expected, .. } => expected,
        }
    }
}

/// The closed, ordered authority effects (spec 165 3.13).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorityEffect {
    None,
    ClaimsTerritory,
    ChangesObligation,
    RequiresHumanRatification,
}

/// Whether a plan can be handed to an applier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanStatus {
    Ready,
    Refused,
}

/// The closed refusal reasons (spec 165 3.14). There is no other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RefusalReason {
    StaleTarget,
    StaleLedger,
    AmbiguousTarget,
    UnresolvedTarget,
    UnresolvedInterface,
    UnsupportedTargetKind,
    UnsupportedLayout,
    UnsupportedEncoding,
    OverlappingOperations,
    DuplicateDeclaration,
    GeneratedTarget,
    OutOfScope,
    UnownedTarget,
    UnauthorizedTarget,
    ApprovedSpecText,
    AmendmentTargetNotApproved,
    UnamendableAnchor,
    ManifestInvalid,
    PostImageInvalid,
    PlanTooLarge,
}

/// One refusal: the operation, its target, the reason and safe detail that
/// names identities and digests, never target content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanRefusal {
    /// The operation's position in the request; null for a plan-wide refusal.
    pub index: Option<usize>,
    pub target: Option<String>,
    pub reason: RefusalReason,
    pub detail: String,
}

/// The closed non-refusing conflict records (spec 165 3.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConflictKind {
    /// A claimed path has other owners.
    SharedOwnership,
    /// A changed obligation carries a spec 109 `conflicts` entry.
    DeclaredConflict,
}

/// One conflict record. It informs routing and never changes the effect.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanConflict {
    pub kind: ConflictKind,
    pub index: usize,
    pub target: String,
    /// The other owners, or the specs declaring the conflict.
    pub specs: Vec<String>,
}

/// One accepted or refused operation, in canonical order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlannedOperation {
    pub index: usize,
    pub op: String,
    /// The target identity: a spec 155 identity or a repository path.
    pub target: String,
    pub expected: String,
    pub authority_effect: AuthorityEffect,
    /// For `add-trace-relation`: the spec 169 state the entry would have.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicted_state: Option<TraceState>,
}

/// One changed or created file, as exact bytes (spec 165 3.12).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilePreview {
    pub path: String,
    /// `sha256:` over the file's raw bytes, or `absent` for a new file: the
    /// guard an applier checks before writing.
    pub before_raw_digest: String,
    /// `sha256:` over `afterContent`'s bytes.
    pub after_digest: String,
    /// The authoritative post-image, LF.
    pub after_content: String,
    /// A derived three-line-context unified diff.
    pub diff: String,
}

/// How severe a predicted finding is. An error is never predicted: it
/// refuses the plan as `post-image-invalid`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingSeverity {
    Warning,
    Info,
}

/// One compile finding predicted on a post-image.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PredictedFinding {
    pub path: String,
    pub code: String,
    pub severity: FindingSeverity,
    pub message: String,
}

/// A committed derived tree a change would stale (spec 165 3.12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DerivedTree {
    SpecRegistry,
    CodebaseIndex,
}

/// An edit plan (spec 165 3.15). A refused plan lists every refusal and
/// carries no preview.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditPlan {
    pub schema_version: String,
    pub status: PlanStatus,
    pub snapshot: PacketSnapshot,
    /// The subject: the scope's `ownSpec`, resolved to its full id.
    pub subject: String,
    pub request_digest: String,
    /// The greatest of the operations' effects.
    pub authority_effect: AuthorityEffect,
    pub operations: Vec<PlannedOperation>,
    pub refusals: Vec<PlanRefusal>,
    pub conflicts: Vec<PlanConflict>,
    pub previews: Vec<FilePreview>,
    pub predicted_findings: Vec<PredictedFinding>,
    /// The derived trees the change would stale, in canonical order; none is
    /// regenerated.
    pub stales: Vec<DerivedTree>,
    pub plan_digest: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sixth_operation_kind_is_not_an_operation() {
        let ok = r#"{"op":"update-manifest","target":"docs/m.json","expected":"sha256:x","manifest":{},"pages":[]}"#;
        assert!(serde_json::from_str::<EditOperation>(ok).is_ok());
        let sixth = r#"{"op":"move-file","target":"a","expected":"b"}"#;
        assert!(serde_json::from_str::<EditOperation>(sixth).is_err());
        let extra = ok.replacen('{', r#"{"apply":true,"#, 1);
        assert!(serde_json::from_str::<EditOperation>(&extra).is_err());
    }

    #[test]
    fn severities_and_derived_trees_are_closed() {
        let finding = |severity: &str| {
            serde_json::from_value::<PredictedFinding>(serde_json::json!({
                "path": "p", "code": "L-001", "severity": severity, "message": "m",
            }))
        };
        assert!(finding("warning").is_ok());
        assert!(
            finding("error").is_err(),
            "an error refuses; it is never predicted"
        );
        assert!(serde_json::from_str::<DerivedTree>(r#""spec-registry""#).is_ok());
        assert!(serde_json::from_str::<DerivedTree>(r#""attestation""#).is_err());
    }

    #[test]
    fn effects_are_ordered() {
        assert!(AuthorityEffect::None < AuthorityEffect::ClaimsTerritory);
        assert!(AuthorityEffect::ClaimsTerritory < AuthorityEffect::ChangesObligation);
        assert!(AuthorityEffect::ChangesObligation < AuthorityEffect::RequiresHumanRatification);
    }
}
