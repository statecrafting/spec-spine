//! Documentation manifest and freshness DTOs (spec 160).
//!
//! A manifest records what one declared generator invocation says it read and
//! produced: one complete spec 159 context-packet page set, repository-local
//! spec 155 selected-content inputs, spec 110 interface pins, opaque external
//! references, the generator's identity, the outputs with their generated and
//! human-authored regions, and executable-example identities. A freshness
//! result names every declared identity that moved against one current
//! snapshot.
//!
//! Plain data. Every authored type refuses a member it does not know (spec 160
//! 3.2), and every collection is explicit, including an empty one. Invariants
//! the types cannot carry (nonempty `uses`, page lists and argument vectors,
//! 1-based ordered spans, id grammar, digest syntax, limits) are enforced by
//! `spec_spine_core`'s validation, which every consumer path goes through;
//! the embedded schema states the ones JSON Schema can express. Neither
//! document establishes narrative correctness, execution, evidence admission,
//! acceptance or authority (spec 160 3.16).

use serde::{Deserialize, Serialize};

use crate::{
    ContentProjection, PacketOmissionReason, PacketRequirement, PacketSnapshot, PinnedSection,
    RepoPath,
};

/// JSON Schema for the manifest and freshness documents (spec 160 3.13): one
/// file, one axis, [`crate::DOCUMENTATION_MANIFEST_SCHEMA_VERSION`]. Its two
/// `$defs`, `manifest` and `freshness`, are the two documents.
pub const DOCUMENTATION_MANIFEST_SCHEMA: &str =
    include_str!("../schemas/documentation-manifest.schema.json");

/// At most this many repository-local inputs (spec 160 3.13).
pub const MANIFEST_MAX_INPUTS: usize = 4096;
/// At most this many interface inputs.
pub const MANIFEST_MAX_INTERFACES: usize = 1024;
/// At most this many opaque external references.
pub const MANIFEST_MAX_EXTERNAL_REFERENCES: usize = 1024;
/// At most this many outputs.
pub const MANIFEST_MAX_OUTPUTS: usize = 256;
/// At most this many regions in total, summed across every output. The
/// schema can bound only each output's array; validation checks the total.
pub const MANIFEST_MAX_REGIONS: usize = 4096;
/// At most this many executable examples in total, summed across every
/// output. The schema can bound only each output's array; validation checks
/// the total.
pub const MANIFEST_MAX_EXAMPLES: usize = 2048;
/// At most this many bytes of canonical JSON, for a manifest and for a
/// freshness result alike.
pub const MANIFEST_MAX_BYTES: usize = 4 * 1024 * 1024;
/// At most this many UTF-8 bytes in an input's `label`.
pub const MANIFEST_LABEL_LIMIT: usize = 256;

/// One documentation manifest (spec 160 3.2), closed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocumentationManifest {
    pub schema_version: String,
    pub packet: ManifestPacket,
    pub inputs: Vec<ManifestInput>,
    pub interfaces: Vec<ManifestInterface>,
    pub external_references: Vec<ManifestExternalReference>,
    pub generator: ManifestGenerator,
    pub outputs: Vec<ManifestOutput>,
    pub omissions: Vec<ManifestOmission>,
    /// `sha256:` over the canonical manifest with this member absent.
    pub manifest_digest: String,
}

/// The one complete context-packet page set a manifest binds (spec 160 3.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestPacket {
    /// The pages' spec 159 `schemaVersion`.
    pub schema_version: String,
    pub snapshot: PacketSnapshot,
    pub request_digest: String,
    /// The pages' spec 107 closure digest: 64 lowercase hex, unprefixed, as
    /// the packet records it.
    pub closure_digest: String,
    /// Every page's `packetDigest`, in page order. Nonempty.
    pub page_digests: Vec<String>,
    /// Every page's `completeness`, in page order: `partial` for each page
    /// that carries a continuation and `complete` for the terminal page (spec
    /// 160 D-14).
    pub page_completeness: Vec<crate::ContentCompleteness>,
}

/// One repository-local input: a spec 155 selected-content identity the
/// named packet carries (spec 160 3.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestInput {
    pub input_id: String,
    pub identity: String,
    pub projection: ContentProjection,
    /// The selected-content item's `schemaVersion`.
    pub schema_version: String,
    /// The selected-content item's `digest`.
    pub digest: String,
    pub requirement: PacketRequirement,
    pub uses: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// One spec 110 interface pin used as an input (spec 160 3.5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestInterface {
    pub input_id: String,
    pub corpus: String,
    pub spec: String,
    pub digest: String,
    pub sections: Vec<PinnedSection>,
    pub requirement: PacketRequirement,
    pub uses: Vec<String>,
}

/// What an opaque external reference is for (spec 160 3.6). Closed in v1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalPurpose {
    Evidence,
    Policy,
    Toolchain,
}

/// One opaque external identity, compared only by digest (spec 160 3.6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestExternalReference {
    pub input_id: String,
    pub namespace: String,
    pub schema_version: String,
    pub identity: String,
    pub digest: String,
    pub requirement: PacketRequirement,
    pub uses: Vec<String>,
    pub purpose: ExternalPurpose,
}

/// The declared generator invocation (spec 160 3.7): an identity record, not
/// an execution request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestGenerator {
    pub name: String,
    pub version: String,
    pub build_digest: String,
    /// Literal arguments after redaction, in invocation order.
    pub arguments: Vec<String>,
    pub configuration_digest: String,
}

/// One produced text file (spec 160 3.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestOutput {
    pub output_id: String,
    pub path: RepoPath,
    pub media_type: String,
    /// `sha256:` over the normalized text (spec 160 D-15).
    pub digest: String,
    /// Bytes of the normalized text.
    pub byte_length: u64,
    pub regions: Vec<ManifestRegion>,
    pub examples: Vec<ManifestExample>,
}

/// Who wrote a region's bytes (spec 160 3.8). Closed in v1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RegionKind {
    Generated,
    HumanAuthored,
}

/// An inclusive, 1-based normalized-text line span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegionSpan {
    pub start_line: usize,
    pub end_line: usize,
}

/// One region of an output (spec 160 3.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestRegion {
    pub region_id: String,
    pub kind: RegionKind,
    pub span: RegionSpan,
    /// `sha256:` over the region's normalized bytes.
    pub digest: String,
    pub input_ids: Vec<String>,
}

/// The spec 132 outcome class an example expects (spec 160 3.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExampleExit {
    Ok,
    Finding,
    Refused,
    Usage,
    Failed,
}

/// One executable-example identity (spec 160 3.9). A declaration only: never
/// run, and never evidence that it passed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestExample {
    pub example_id: String,
    pub region_id: String,
    /// The argument vector, in order.
    pub arguments: Vec<String>,
    /// A normalized repository-relative directory, or `.` for the root.
    pub working_directory: String,
    pub input_ids: Vec<String>,
    pub expected_exit: ExampleExit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr_digest: Option<String>,
}

/// An optional repository-local input the packet omitted (spec 160 3.4,
/// 3.12), recorded instead of an input so no content is implied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestOmission {
    pub input_id: String,
    pub identity: String,
    pub projection: ContentProjection,
    pub uses: Vec<String>,
    /// The packet's exact omission reason.
    pub reason: PacketOmissionReason,
}

/// A caller-supplied current digest for one external reference (spec 160
/// 3.6). Matched by namespace, schema version and identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalResolution {
    pub namespace: String,
    pub schema_version: String,
    pub identity: String,
    pub digest: String,
}

/// The closed finding reasons (spec 160 3.10), in canonical order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FreshnessReason {
    Changed,
    Missing,
    Removed,
    Withdrawn,
    Unresolved,
    Ambiguous,
    Unsupported,
    Unverified,
    RegionShifted,
    OutputChanged,
}

/// The result status (spec 160 3.11), least severe first, so the derived
/// order's `max` is the status a set of findings decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FreshnessStatus {
    Fresh,
    Unverified,
    Stale,
    Incomplete,
}

/// One difference between the manifest and the current snapshot (spec 160
/// 3.10). Identities and digests only, never content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FreshnessFinding {
    pub kind: FreshnessReason,
    /// `input:<inputId>`, `packet`, `output:<outputId>` or
    /// `region:<outputId>#<regionId>` (spec 160 D-16).
    pub identity: String,
    pub requirement: PacketRequirement,
    /// Absent for an input the manifest recorded as omitted and for the
    /// packet finding, which has no aggregate digest (spec 160 D-16).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_digest: Option<String>,
    /// The output, region or example identities affected.
    pub affects: Vec<String>,
}

/// One freshness result (spec 160 3.11), closed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocumentationFreshness {
    pub schema_version: String,
    pub manifest_digest: String,
    pub recorded_snapshot: PacketSnapshot,
    pub current_snapshot: PacketSnapshot,
    pub repository_tree_changed: bool,
    pub status: FreshnessStatus,
    pub findings: Vec<FreshnessFinding>,
    /// `sha256:` over the canonical result with this member absent.
    pub freshness_digest: String,
}
