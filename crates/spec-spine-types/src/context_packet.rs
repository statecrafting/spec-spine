//! Repository-scoped context packet request and response types (spec 159).
//!
//! A packet composes a spec 107 closure with spec 155 selected-content items
//! for one immutable repository snapshot. It reuses both identities and adds
//! only composition metadata: requirement, origin, omissions, warnings,
//! completeness, continuation and digests.

use serde::{Deserialize, Serialize};

use crate::{
    ContentCompleteness, ContentDirtyState, ContentItem, ContentProjection, ContentSelector,
};

/// Default and ceiling of [`PacketRequest::max_bytes`] (spec 159 3.2).
pub const PACKET_DEFAULT_MAX_BYTES: usize = 524_288;
/// Upper bound of [`PacketRequest::max_bytes`].
pub const PACKET_MAX_BYTES_LIMIT: usize = 4_194_304;
/// Default of [`PacketRequest::max_items`].
pub const PACKET_DEFAULT_MAX_ITEMS: usize = 96;
/// Upper bound of [`PacketRequest::max_items`].
pub const PACKET_MAX_ITEMS_LIMIT: usize = 512;
/// Upper bound, in bytes, of `rationale` and `workIdentity`.
pub const PACKET_OPAQUE_LIMIT: usize = 1024;

fn default_max_bytes() -> usize {
    PACKET_DEFAULT_MAX_BYTES
}

fn default_max_items() -> usize {
    PACKET_DEFAULT_MAX_ITEMS
}

/// One context-packet request (spec 159 3.2). Unknown members are refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PacketRequest {
    pub closure: PacketClosure,
    #[serde(default)]
    pub members: Vec<PacketMemberRequest>,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_max_items")]
    pub max_items: usize,
    /// The opaque continuation a previous page returned.
    #[serde(default)]
    pub continuation: Option<String>,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub work_identity: Option<String>,
    pub consumer_schema_version: String,
}

/// The closure a packet starts from: exactly one of `root` or `digest`
/// (spec 159 3.2). `digest` travels with the closure `document` it names,
/// which the producer re-resolves and compares; it never discovers a closure
/// by digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PacketClosure {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<serde_json::Value>,
}

/// One explicit member request: a spec 155 selector plus its requirement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PacketMemberRequest {
    pub selector: ContentSelector,
    pub requirement: PacketRequirement,
}

/// Whether a member's absence makes the packet incomplete.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PacketRequirement {
    Required,
    Optional,
}

/// Where a member came from. Declared in canonical order: a member with both
/// origins lists `closure` first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PacketOrigin {
    Closure,
    Additional,
}

/// The snapshot every member was read from (spec 159 3.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PacketSnapshot {
    pub repository: String,
    pub revision: String,
    pub tree: String,
    pub dirty_state: ContentDirtyState,
}

/// The build a packet was produced by (spec 159 3.5, D-5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PacketProducer {
    /// Always `spec-spine`.
    pub package: String,
    pub version: String,
    /// `sha256:<hex>` supplied by the binding, or `unverified`.
    pub build: String,
}

/// One returned member: the complete spec 155 item plus composition metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PacketMember {
    pub identity: String,
    pub projection: ContentProjection,
    pub requirement: PacketRequirement,
    pub origins: Vec<PacketOrigin>,
    pub item: ContentItem,
}

/// Why a member is absent (spec 159 3.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PacketOmissionReason {
    Missing,
    Removed,
    Withdrawn,
    Unresolved,
    Ambiguous,
    UnsupportedSelector,
    UnsupportedProjection,
    NonText,
    OversizedMember,
    ItemBudget,
    ByteBudget,
}

/// One absent member, by canonical key, with exactly one reason.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PacketOmission {
    pub identity: String,
    pub projection: ContentProjection,
    pub requirement: PacketRequirement,
    pub origins: Vec<PacketOrigin>,
    pub reason: PacketOmissionReason,
    /// Safe detail naming identities and reasons, never selected content.
    pub detail: String,
}

/// The closed warning vocabulary of schema 1.0 (spec 159 3.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PacketWarningCode {
    ClosureMemberUnsupported,
    OptionalMemberOmitted,
    UnverifiedProducerBuild,
}

/// One typed warning. Warnings never decide completeness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PacketWarning {
    pub code: PacketWarningCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection: Option<ContentProjection>,
}

/// One context-packet page (spec 159 3.5), under its own schema axis,
/// [`crate::CONTEXT_PACKET_SCHEMA_VERSION`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextPacket {
    pub schema_version: String,
    pub producer: PacketProducer,
    pub snapshot: PacketSnapshot,
    /// The normalized request: members canonical and deduplicated, and no
    /// continuation.
    pub request: PacketRequest,
    pub request_digest: String,
    /// The resolved spec 107 closure document.
    pub closure: serde_json::Value,
    pub closure_digest: String,
    pub members: Vec<PacketMember>,
    pub omissions: Vec<PacketOmission>,
    pub warnings: Vec<PacketWarning>,
    pub completeness: ContentCompleteness,
    pub continuation: Option<String>,
    pub packet_digest: String,
}
