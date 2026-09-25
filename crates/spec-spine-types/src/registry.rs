//! Registry DTOs: the spec-as-source view, emitted by `compile` as
//! `registry.json`. Field names serialize to `camelCase` (the JSON contract),
//! distinct from the `snake_case` authored [`crate::Frontmatter`] grammar.
//!
//! The compiler (Phase 2) populates these from parsed frontmatter plus computed
//! fields (`spec_path`, `section_headings`, the content hash). Shapes are ported
//! from OAP `registry.schema.json` (`featureRecord`, `build`, `violation`),
//! pruned to the generic v1 surface; overlay fields (compliance, factory,
//! capability/registry/profile) are intentionally absent (see §10.4).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::edges::{
    CoAuthorityItem, ConstrainItem, ExtendItem, Origin, ReferenceItem, RefineItem, SupersedeItem,
};
use crate::frontmatter::{Implementation, Risk, Status};
use crate::impact::{Conflict, Impact};
use crate::intent::Intent;
use crate::interface::InterfaceReference;
use crate::moves::MoveDeclaration;
use crate::obligation::Obligation;
use crate::unit::Unit;

/// The compiled registry: `registry.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registry {
    /// `MAJOR.MINOR.PATCH`; see [`crate::version::REGISTRY_SCHEMA_VERSION`].
    pub spec_version: String,
    pub build: Build,
    pub specs: Vec<SpecRecord>,
    pub validation: ValidationReport,
}

/// Deterministic build metadata embedded in `registry.json` (no timestamps:
/// the wall clock lives in the separate, non-deterministic `build-meta.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Build {
    pub compiler_id: String,
    pub compiler_version: String,
    /// The input root the registry was compiled from, repo-relative (e.g. `.`).
    pub input_root: String,
    /// SHA-256 over the normalized, path-sorted spec inputs (64 lowercase hex).
    pub content_hash: String,
}

/// One spec's entry in the registry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecRecord {
    // --- required ---
    pub id: String,
    pub title: String,
    pub status: Status,
    pub created: String,
    pub summary: String,
    /// Repo-relative path: `specs/NNN-slug/spec.md`.
    pub spec_path: String,

    // --- optional descriptive ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<Risk>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<Implementation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code_aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_branch: Option<String>,
    /// Markdown headings discovered in the spec body (anchors for sections).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub section_headings: Vec<String>,

    // --- typed edges (8) ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub establishes: Vec<Unit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extends: Vec<ExtendItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refines: Vec<RefineItem>,
    /// Full supersession serializes as a bare predecessor id; a partial item
    /// serializes as an object (spec 018).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supersedes: Vec<SupersedeItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amends: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub co_authority: Vec<CoAuthorityItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constrains: Vec<ConstrainItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<ReferenceItem>,

    // --- lifecycle / amendment ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retirement_rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amends_sections: Vec<String>,
    /// Spec 082 3.1: amended specs whose `## Verification` block this spec's
    /// block replaces. Additive, so a MINOR of `REGISTRY_SCHEMA_VERSION`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amends_verification: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unamendable: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment_record: Option<String>,

    // --- bootstrap marker ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,

    // --- declared constraints and section identity (spec 106) ---
    /// The spec's obligations, verbatim as declared (spec 106 §3.1).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligations: Vec<Obligation>,
    /// Every body section's anchor mapped to its digest (spec 106 §3.5):
    /// SHA-256 over `<specPath>#<anchor>`, NUL, and the section's normalized
    /// lines. Beside the spec's full content hash, never instead of it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub section_digests: BTreeMap<String, String>,

    // --- declared impact and conflict (spec 109) ---
    /// This spec's declared relations to other specs' obligations, `obligation`
    /// normalized to its full qualified form (spec 109 §3.7).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impacts: Vec<Impact>,
    /// This spec's declared, unresolved disagreements with other specs'
    /// obligations, `obligation` normalized the same way.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<Conflict>,

    // --- cross-corpus interface references (spec 110) ---
    /// Carried verbatim from frontmatter, omitted when empty (spec 110 §3.2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interface_references: Vec<InterfaceReference>,

    // --- declared intent (spec 114) ---
    /// The spec's standing goal and non-goals, as authored (spec 114 §3.3).
    /// Omitted when the spec declares none. Read by no gate (§3.5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<Intent>,

    // --- declared moves (spec 111) ---
    /// Paths this spec declares it has relocated, split, merged or removed
    /// (spec 111 §3.1), carried verbatim from frontmatter except
    /// `answered_by`'s spec half, normalized to its full id (spec 111 §3.2 by
    /// way of spec 015). Omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub moves: Vec<MoveDeclaration>,
    /// Declared section relocations (spec 142 §3.2), `spec` normalized to its
    /// full id. Omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relocates: Vec<crate::Relocation>,

    // --- overflow ---
    /// Declared keys carry any JSON value (spec 012); undeclared keys are
    /// scalars or string arrays.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra_frontmatter: BTreeMap<String, serde_json::Value>,
}

// ===== sharded committed form (spec 022) =====
//
// The committed registry is stored as one file per spec so two PRs that add or
// edit different specs write disjoint files and never conflict textually on a
// shared content-hash line. The aggregate [`Registry`] above stays the universal
// in-memory currency: the compiler projects it to shards, and a reader assembles
// it back from the shard set. The aggregate `validation` and `build.contentHash`
// are recomputed on read (cross-spec checks like duplicate-id / dangling edges
// are pure functions of the assembled record set), never committed.

/// One spec's registry shard: `<derived>/spec-registry/by-spec/<id>.json`.
/// A PR that adds or edits spec X rewrites only X's shard (spec 022 FR-002).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySpecShard {
    /// `specVersion`; see [`crate::version::REGISTRY_SCHEMA_VERSION`].
    pub spec_version: String,
    /// SHA-256 over this spec's `spec.md` (the registry's only hashed input,
    /// matching the pre-shard `build.contentHash` input set). Self-describing
    /// per-shard staleness.
    pub shard_hash: String,
    /// This spec's compiled record.
    pub record: SpecRecord,
    /// Validation findings that are a pure function of THIS spec (V-001/002/
    /// 005/006/007/011/012/013). Cross-spec findings (duplicate id/prefix,
    /// dangling edges) are recomputed on read from the assembled record set, so
    /// they are never stored here (storing them would make a sibling spec's PR
    /// stale this shard). Omitted when empty: a clean spec carries none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_violations: Vec<Violation>,
}

/// Non-deterministic build metadata sidecar (`build-meta.json`). The wall-clock
/// `built_at` lives here, never in `registry.json`, and is excluded from every
/// determinism/golden check. The CLI populates `built_at`; the library never
/// reads the clock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildMeta {
    pub schema_version: String,
    pub built_at: String,
    pub compiler_id: String,
    pub compiler_version: String,
}

/// Severity tier of a diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A single validation/lint/coupling diagnostic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    /// A stable code such as `V-001`, `L-003`, `I-004`.
    pub code: String,
    pub severity: Severity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The spec ids that own `path`, sorted, for the codes where "owner" is a
    /// fact the producer computed (spec 045: `C-001` alone). Empty everywhere
    /// else, and omitted from serialization when empty, so every other
    /// producer's JSON is byte-identical to what it emitted before this field
    /// existed. It carries as data the owner set `message` renders into
    /// English, so a consumer of the spec 034 `--json` envelope reads the
    /// owners instead of regexing a sentence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owners: Vec<String>,
}

impl Violation {
    /// A violation carrying no owner set: every code except `C-001`.
    ///
    /// The owner-less shape is the overwhelming majority, so it gets the
    /// constructor and `C-001` names `owners` explicitly at its one site.
    pub fn new(code: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        Violation {
            code: code.into(),
            severity,
            message: message.into(),
            path: None,
            owners: Vec::new(),
        }
    }

    /// Attach the path this violation is about.
    pub fn at(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Attach the path this violation is about, when there may not be one.
    pub fn at_opt(mut self, path: Option<String>) -> Self {
        self.path = path;
        self
    }
}

/// The registry's validation summary. `passed` is false iff any `error`-tier
/// violation is present.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub passed: bool,
    #[serde(default)]
    pub violations: Vec<Violation>,
}

impl ValidationReport {
    /// Build a report from violations, setting `passed` per the error-tier rule.
    pub fn from_violations(violations: Vec<Violation>) -> Self {
        let passed = !violations.iter().any(|v| v.severity == Severity::Error);
        ValidationReport { passed, violations }
    }
}
