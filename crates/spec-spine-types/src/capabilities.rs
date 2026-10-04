//! What a binary supports (spec 170 3.5): the `capabilities --json` document.
//!
//! A consumer asks this, not a verb's `--help` or its exit status, whether a
//! verb exists and which schema its `--json` output carries. The document is
//! a fact about the binary alone: it reads no repository and writes nothing,
//! so it answers the same in every directory.

use serde::{Deserialize, Serialize};

/// The document. Versioned on its own axis,
/// [`crate::CAPABILITIES_SCHEMA_VERSION`], under `schemaVersion`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub schema_version: String,
    /// The binary's package version, as `--version` prints it after the name.
    pub version: String,
    /// Every verb the binary wires, sorted by path.
    pub verbs: Vec<VerbCapability>,
}

/// One runnable verb.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerbCapability {
    /// The command path after the binary name, space separated
    /// (`registry closure`).
    pub path: String,
    /// Its long flags without the leading `--`, sorted. `help` and the global
    /// `repo` are not listed.
    pub flags: Vec<String>,
    /// The schema axes its `--json` success output carries, empty when the
    /// verb takes no `--json`. A failure under `--json` is always the verdict
    /// envelope (spec 152), so it is not repeated here.
    pub json: Vec<SchemaAxis>,
}

/// One schema axis and the version this binary emits on it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaAxis {
    /// `verdict`, `read`, `delta` or `config`.
    pub axis: String,
    pub version: String,
}
