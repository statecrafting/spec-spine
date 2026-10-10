//! The capability catalog (spec 162): every operation this binary performs,
//! described once.
//!
//! One record per CLI invocation form and per facade entry point: what it is
//! called, what it reads, writes and runs, which exit codes it answers with,
//! which schema its answer follows, whether it is bounded, how stable it is,
//! and how to run it. The vocabularies are closed (162 §3.4, §3.5), and the
//! embedded schema [`crate::CAPABILITY_CATALOG_SCHEMA`] enforces them.
//!
//! These are plain data. The one definition lives in
//! `spec-spine-core::capability`; the catalog describes, it authorizes nothing
//! (162 §3.14).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The catalog document. Versioned on its own axis,
/// [`crate::CATALOG_SCHEMA_VERSION`], under `schemaVersion`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityCatalog {
    pub schema_version: String,
    /// Always `spec-spine`.
    pub tool: String,
    /// The core crate's compile-time package version.
    pub tool_version: String,
    /// Every schema axis the types crate declares, by name, at its current
    /// version (162 §3.1).
    pub compatibility: BTreeMap<String, String>,
    /// Where the digest-pinned interface machinery is found (162 §3.11).
    pub discovery: CatalogDiscovery,
    /// Sorted by `name`.
    pub operations: Vec<Operation>,
    /// `sha256:<hex>` over the canonical catalog with this member omitted.
    pub catalog_digest: String,
}

/// 162 §3.11: the operations that declare and verify pins.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogDiscovery {
    pub interface_references: InterfaceDiscovery,
    pub capabilities: CapabilitiesDiscovery,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InterfaceDiscovery {
    pub declared_by: String,
    pub member: String,
    pub pin_sources: Vec<String>,
    pub verified_by: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilitiesDiscovery {
    pub verified_by: String,
}

/// One operation record (162 §3.3). Every member is present; `null` or empty
/// where nothing applies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Operation {
    pub name: String,
    /// One sentence; equal to the clap `about` of the command it binds.
    pub summary: String,
    /// Sorted spec ids.
    pub governed_by: Vec<String>,
    pub cli: Option<CliBinding>,
    /// Sorted by `function`, then `selector`.
    pub facade: Vec<FacadeBinding>,
    pub request: RequestRef,
    /// Sorted, the `when: null` entry first.
    pub response: Vec<ResponseRef>,
    pub effects: Effects,
    /// Sorted by `flag`.
    pub effects_when: Vec<ConditionalEffects>,
    /// Sorted by `name`.
    pub preconditions: Vec<Precondition>,
    /// Sorted by `exitCode`.
    pub outcomes: Vec<ExitOutcome>,
    pub budget: Option<Budget>,
    /// `none` or `continuation`.
    pub pagination: String,
    /// `stable`, `experimental` or `deprecated`.
    pub stability: String,
    /// The release that first shipped it; `null` before the catalog (D-4).
    pub since: Option<String>,
    pub deprecation: Option<Deprecation>,
    /// Sorted by `id`.
    pub examples: Vec<Example>,
    /// `sha256:<hex>` over the canonical record with this member omitted.
    pub operation_digest: String,
}

/// How the operation is invoked from the command line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CliBinding {
    /// The fixed token path after the binary name.
    pub argv: Vec<String>,
    /// Every clap argument of the command, sorted by `name`.
    pub flags: Vec<Flag>,
    /// `verdict-envelope`, `read-document`, `text` or `files`.
    pub output: String,
}

/// One clap argument.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Flag {
    /// `--long` for a flag, the bare argument id for a positional.
    pub name: String,
    pub positional: bool,
    /// `null` for a switch that takes no value.
    pub value_name: Option<String>,
    pub required: bool,
    pub repeatable: bool,
    pub conflicts_with: Vec<String>,
}

/// One facade entry point serving the operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FacadeBinding {
    pub function: String,
    /// The discriminator a multi-operation function takes (`query_json`'s
    /// `op`), or `null`.
    pub selector: Option<String>,
    /// Whether the function takes `repo_root` and so reads under it (162 §3.4).
    pub reads_repository: bool,
}

/// 162 §3.6.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestRef {
    /// `argv` or `document`.
    pub kind: String,
    pub schema: Option<String>,
}

/// 162 §3.6.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResponseRef {
    /// `null` for the default answer, or the flag that selects this one.
    pub when: Option<String>,
    pub axis: String,
    pub version: String,
    pub document: String,
    pub schema: Option<String>,
}

/// 162 §3.4. Every list is sorted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Effects {
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub executes: Vec<String>,
    /// `none` or `delegated`.
    pub network: String,
    pub environment: Vec<String>,
    pub authority: Vec<String>,
}

/// No effect at all: every list empty and `network: none`, the one value of
/// the closed vocabulary that claims nothing. Written out rather than derived,
/// because a derived default would leave `network` empty, which the schema
/// refuses.
impl Default for Effects {
    fn default() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
            executes: Vec::new(),
            network: "none".to_string(),
            environment: Vec::new(),
            authority: Vec::new(),
        }
    }
}

/// Effects a flag adds to the base (162 §3.4). `network` is omitted: no flag
/// changes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConditionalEffects {
    pub flag: String,
    pub effects: AddedEffects,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddedEffects {
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub executes: Vec<String>,
    pub environment: Vec<String>,
    pub authority: Vec<String>,
}

/// 162 §3.5.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Precondition {
    pub name: String,
    /// The exit codes its failure produces, sorted.
    pub exit_codes: Vec<u8>,
}

/// One exit code an operation answers with (162 §3.5, spec 132's pairs).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExitOutcome {
    pub exit_code: u8,
    pub outcome: String,
    /// A subset of `verdict::ERROR_KINDS` that maps to `exit_code`, sorted.
    pub kinds: Vec<String>,
}

/// 162 §3.7.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Budget {
    pub max_bytes: Bound,
    pub max_items: Bound,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bound {
    pub default: u64,
    pub min: u64,
    pub max: u64,
}

/// 162 §3.8.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Deprecation {
    pub since: String,
    pub replacement: Option<String>,
    pub note: String,
}

/// One example the CLI suite runs against the built binary (162 §3.9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Example {
    pub id: String,
    /// A directory under the CLI crate's
    /// `tests/fixtures/capability-catalog/`, or `null` for none.
    pub fixture: Option<String>,
    /// Arguments after the binary name; empty for a facade-only example.
    pub argv: Vec<String>,
    /// Bytes fed to stdin, or the request of a facade-only example.
    pub stdin: Option<String>,
    pub exit_code: u8,
    /// Literal substrings stdout must contain, sorted.
    pub stdout_includes: Vec<String>,
}

/// `capability_verify_json`'s request (162 §3.11): operation name to the
/// digest the caller pinned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityVerifyRequest {
    pub expect: BTreeMap<String, String>,
}

/// One pin's outcome: `current`, `changed` or `missing`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PinResult {
    pub name: String,
    pub outcome: String,
    pub expected: String,
    /// The running binary's digest; `null` when `missing`.
    pub observed: Option<String>,
}

/// `capability_verify_json`'s answer, sorted by `name`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityVerifyReport {
    pub catalog_digest: String,
    pub results: Vec<PinResult>,
}

impl CapabilityVerifyReport {
    /// 0 when every pin is `current`, else 1 (162 §3.11).
    pub fn exit_code(&self) -> u8 {
        if self.results.iter().all(|r| r.outcome == "current") {
            0
        } else {
            1
        }
    }
}
