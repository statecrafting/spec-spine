//! Authority-snapshot DTOs (spec 070): the third attestation scope.
//!
//! An [`AuthoritySnapshot`] records, for one tree and one tool version, which
//! inputs were read and what they came to: the configuration, the corpus hashes
//! spec 021 already defines, the committed registry and index trees with a
//! digest each and whether each matches the recompute, the governance inputs by
//! path, the gate verdicts, the ownership counts, every spec's lifecycle with a
//! framed digest of its resolved territory, and the exclusions in force.
//!
//! Plain data. Every type refuses a member it does not know (spec 068 3.2), for
//! the reason `attest.rs` states: a verifier that drops an unknown member has
//! verified a smaller object than the one it was handed.
//!
//! Every digest this record introduces is `frame/1` (spec 070 §3.3), a binding
//! over **normalized text** for a UTF-8 file and exact bytes for any other. The
//! members that exist in other payloads (`inputsManifestHash`, `registryHash`,
//! `findingsHash`, `specAttestationHash`) keep their own constructions.

use serde::{Deserialize, Serialize};

use crate::attest::ToolStamp;

/// The `schemaVersion` of an [`AuthoritySnapshot`]: its own axis, starting at
/// `0.1.0`, independent of every other record line (spec 070 §3.5).
pub const SNAPSHOT_SCHEMA_VERSION: &str = "0.1.0";

/// The digest construction named in every snapshot (spec 070 §3.3).
pub const FRAME_DIGEST: &str = "frame/1";

/// The one reason a per-spec join hash may be absent (spec 070 §3.2.1). A closed
/// vocabulary of one: a second reason is a spec change, not a build's call.
pub const NON_UTF8_DIRECT_CLAIM: &str = "non-utf8-direct-claim";

/// An authority snapshot (spec 070 §3.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoritySnapshot {
    pub schema_version: String,
    pub tool: ToolStamp,
    /// Always [`FRAME_DIGEST`]: the construction every `hash` member uses.
    pub digest: String,
    pub config: SnapshotConfig,
    pub schemas: SnapshotSchemas,
    pub corpus: SnapshotCorpus,
    pub committed: SnapshotCommitted,
    pub governance_inputs: SnapshotGovernanceInputs,
    pub verdicts: SnapshotVerdicts,
    /// One entry per spec, in registry order.
    pub specs: Vec<SnapshotSpec>,
    pub exclusions: SnapshotExclusions,
}

/// `spec-spine.toml`, as read (spec 070 §3.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotConfig {
    /// False when there is no `spec-spine.toml` and the defaults applied.
    pub present: bool,
    /// `frame/1` over the one file; absent exactly when `present` is false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// The build's record-line constants, so a consumer knows which lines the rest
/// of the payload was computed under.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotSchemas {
    pub registry: String,
    pub index: String,
    pub corpus_attestation: String,
    pub spec_attestation: String,
}

/// The spec count and spec 021's two corpus hashes, under 023's construction,
/// so a snapshot joins an existing corpus attestation by value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotCorpus {
    pub specs: usize,
    pub inputs_manifest_hash: String,
    pub registry_hash: String,
}

/// The two committed shard trees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotCommitted {
    pub registry: CommittedTree,
    pub index: CommittedTree,
}

/// One committed shard tree, and whether it equals the recompute.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommittedTree {
    /// The committed files the freshness read compares.
    pub files: usize,
    /// `frame/1` over those files; absent when the tree is absent, rather than
    /// the digest of nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// True exactly when every committed shard file is byte-identical to what
    /// the recompute emits and the two sets match (spec 070 D-5).
    pub matches_recompute: bool,
}

/// The files that feed the index's global-inputs scalar, by path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotGovernanceInputs {
    /// Sorted repo-relative POSIX paths.
    pub paths: Vec<String>,
    /// `frame/1` over those files; absent when there are none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// The verdicts the gate would reach, as counts and flags.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotVerdicts {
    pub compile: SnapshotCompileVerdict,
    pub lint: crate::attest::LintVerdict,
    pub resolution: SnapshotResolutionVerdict,
    pub ownership: SnapshotOwnership,
    pub unwitnessed: SnapshotUnwitnessed,
}

/// Spec 050's pair: claimed paths no content hash covers, and how many of them a
/// `[lint] unwitnessed_allowed` pattern allows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotUnwitnessed {
    pub total: usize,
    pub allowed: usize,
}

/// Structural validation, with its counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotCompileVerdict {
    pub ok: bool,
    pub errors: usize,
    pub warnings: usize,
}

/// Resolution: false when any owning unit is unresolved or any blocking
/// resolver diagnostic exists; the two counts say which.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotResolutionVerdict {
    pub ok: bool,
    pub blocking: usize,
    pub unresolved: usize,
}

/// The `index coverage` report's four counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotOwnership {
    pub source_files: usize,
    pub claimed: usize,
    pub floor_only: usize,
    pub unclaimed: usize,
}

/// One spec's lifecycle and territory (spec 070 §3.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotSpec {
    pub id: String,
    pub status: String,
    /// Omitted only when the frontmatter omits it (spec 039 3.1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<String>,
    /// The content binding: `frame/1` over the spec's resolved owning units
    /// under §3.3.1's piece rule. Absent when they resolve to nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub territory_digest: Option<String>,
    /// Historical evidence only, never a content binding: the `attestationHash`
    /// `attest --spec <id>` would emit for the same tree and tool version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_attestation_hash: Option<String>,
    /// Why `spec_attestation_hash` is absent, when it is. Only
    /// [`NON_UTF8_DIRECT_CLAIM`] (§3.2.1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_attestation_unavailable: Option<String>,
}

/// What was deliberately not read or not held to account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotExclusions {
    pub resolver_exclusions: Vec<String>,
    /// The declared state root, or `null` when none is declared.
    pub state_dir: Option<String>,
    pub unwitnessed_allowed: Vec<String>,
    /// The effective bypass prefixes: the built-in floor plus the configured.
    pub bypass_prefixes: Vec<String>,
}
