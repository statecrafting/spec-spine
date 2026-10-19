//! Reference-page export request, page model, region and check-finding
//! types (spec 166).
//!
//! One page per exported spec, projected from a typed model whose every field
//! comes from a named ledger source. The model is what `reference model`
//! emits; the Markdown page carries no fact the model lacks. Its own schema
//! axis is [`crate::REFERENCE_PAGE_SCHEMA_VERSION`].

use serde::{Deserialize, Serialize};

use crate::{Example, Obligation};

/// Default and ceiling of [`ReferenceLimits::max_pages`] (spec 166 3.2).
pub const REFERENCE_MAX_PAGES: usize = 1024;
/// Default and ceiling of [`ReferenceLimits::max_page_bytes`].
pub const REFERENCE_MAX_PAGE_BYTES: usize = 524_288;
/// Default and ceiling of [`ReferenceLimits::max_signatures`].
pub const REFERENCE_MAX_SIGNATURES: usize = 256;
/// The most bytes an authored `notes` region may hold (spec 166 3.6).
pub const REFERENCE_MAX_NOTES_BYTES: usize = 65_536;

/// The fixed product limitation statement every page ends `limitations`
/// with (spec 166 3.14).
pub const REFERENCE_LIMITATION: &str = "This page restates declared and structurally resolved records. It does not state that code is correct, that a test ran or passed, or that anything was accepted, released, adopted, or observed.";

/// The twelve regions of a page, in page order (spec 166 3.4). `notes` is the
/// one authored region and is always last (D-4).
pub const REFERENCE_REGIONS: [&str; 12] = [
    "header",
    "lifecycle",
    "requirements",
    "ownership",
    "signatures",
    "relations",
    "operations",
    "evidence",
    "limitations",
    "changes",
    "inputs",
    "notes",
];

fn default_max_pages() -> usize {
    REFERENCE_MAX_PAGES
}
fn default_max_page_bytes() -> usize {
    REFERENCE_MAX_PAGE_BYTES
}
fn default_max_signatures() -> usize {
    REFERENCE_MAX_SIGNATURES
}

/// One export request (spec 166 3.2). Closed: unknown members are usage
/// errors.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceRequest {
    pub schema_version: String,
    /// A normalized repository-relative directory, outside the derived and
    /// state directories.
    pub output_dir: String,
    pub specs: ReferenceSpecs,
    /// Whether the spec 169 `relations` field is supplied.
    #[serde(default)]
    pub traceability: bool,
    /// The repository-relative path of a committed spec 162 catalog
    /// document, or null (D-11).
    #[serde(default)]
    pub catalog: Option<String>,
    #[serde(default)]
    pub limits: ReferenceLimits,
}

/// Which specs an export covers: every registry spec, or an explicit list of
/// full ids.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReferenceSpecs {
    Select(SelectAll),
    Ids(SpecIds),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectAll {
    /// Always `all`.
    pub select: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecIds {
    pub ids: Vec<String>,
}

/// Export limits; each may only lower its default (spec 166 3.2, 3.12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceLimits {
    #[serde(default = "default_max_pages")]
    pub max_pages: usize,
    #[serde(default = "default_max_page_bytes")]
    pub max_page_bytes: usize,
    #[serde(default = "default_max_signatures")]
    pub max_signatures: usize,
}

impl Default for ReferenceLimits {
    fn default() -> Self {
        ReferenceLimits {
            max_pages: REFERENCE_MAX_PAGES,
            max_page_bytes: REFERENCE_MAX_PAGE_BYTES,
            max_signatures: REFERENCE_MAX_SIGNATURES,
        }
    }
}

/// The `header` region.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceHeader {
    pub id: String,
    pub title: String,
    pub spec_path: String,
    pub summary: String,
}

/// The `lifecycle` region.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceLifecycle {
    pub status: String,
    pub implementation: Option<String>,
    pub superseded_by: Option<String>,
    pub retirement_rationale: Option<String>,
    pub created: String,
}

/// One owning unit and how the committed index resolved it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceUnit {
    /// `establishes`, `extends`, `refines`, `co_authority` or `constrains`.
    pub edge: String,
    /// The spec an `extends` edge names.
    pub target_spec: Option<String>,
    /// The unit's canonical JSON.
    pub unit: String,
    pub planned: bool,
    /// `resolved`, `unresolved`, or `unknown` when the index records no
    /// resolution for it.
    pub resolution: String,
}

/// The `ownership` region: owning units, then non-owning references.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceOwnership {
    pub units: Vec<ReferenceUnit>,
    /// Canonical JSON of each `references` unit.
    pub references: Vec<String>,
}

/// One owned symbol unit's signature, or why it has none.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSignature {
    /// The symbol id.
    pub symbol: String,
    pub path: Option<String>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub digest: Option<String>,
    pub text: Option<String>,
    /// `rust` or `typescript`, from the source kind.
    pub language: Option<String>,
    /// The spec 155 omission reason when no signature was projected.
    pub omission: Option<String>,
}

/// One declared spec 169 relation whose source obligation is this spec's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceRelation {
    pub id: String,
    pub obligation: String,
    pub relation: String,
    pub target_identity: String,
    pub state: String,
}

/// One catalog operation whose `governedBy` names this spec.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceOperation {
    pub name: String,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub executes: Vec<String>,
    pub network: String,
    pub examples: Vec<Example>,
}

/// One verification obligation's declared inputs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceEvidenceInputs {
    pub obligation: String,
    pub inputs: Vec<String>,
}

/// The `evidence` region.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceEvidence {
    pub inputs: Vec<ReferenceEvidenceInputs>,
    /// The `verify:cli` commands, declared and not executed; empty when the
    /// spec declares no block.
    pub commands: Vec<String>,
}

/// The `changes` region: outgoing typed edges, then incoming ones computed
/// from other records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceChanges {
    pub amends: Vec<String>,
    pub amends_sections: Vec<String>,
    pub amends_verification: Vec<String>,
    pub supersedes: Vec<String>,
    pub amendment_record: Option<String>,
    pub amended_by: Vec<String>,
    pub superseded_by: Vec<String>,
    pub extended_by: Vec<String>,
}

/// One input identity and digest a page was generated from (spec 166 3.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceInput {
    pub identity: String,
    pub digest: String,
}

/// One reference page model (spec 166 3.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferencePage {
    pub schema_version: String,
    pub spec_id: String,
    /// `<outputDir>/<spec-id>.md`.
    pub path: String,
    pub header: ReferenceHeader,
    pub lifecycle: ReferenceLifecycle,
    pub requirements: Vec<Obligation>,
    pub ownership: ReferenceOwnership,
    pub signatures: Vec<ReferenceSignature>,
    /// Null when the request does not supply traceability.
    pub relations: Option<Vec<ReferenceRelation>>,
    /// Null when the request names no catalog.
    pub operations: Option<Vec<ReferenceOperation>>,
    pub evidence: ReferenceEvidence,
    pub non_goals: Vec<String>,
    pub changes: ReferenceChanges,
    pub inputs: Vec<ReferenceInput>,
    /// The authored `notes` bytes, LF-normalized.
    pub notes: String,
}

/// `reference model --json`: every page model, in spec order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceModel {
    pub schema_version: String,
    pub pages: Vec<ReferencePage>,
}

/// The closed check findings (spec 166 3.10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceFindingKind {
    PageMissing,
    PageChanged,
    PageOrphaned,
    PageMalformed,
    LedgerStale,
}

/// One finding: paths, region ids and digests, never page content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceFinding {
    pub kind: ReferenceFindingKind,
    /// The page path, or null for `ledger-stale`.
    pub path: Option<String>,
    /// For `page-changed`: the differing region ids, in page order.
    pub regions: Vec<String>,
    pub detail: String,
}

/// `reference check --json`: the findings, ordered by page path then kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceCheck {
    pub schema_version: String,
    pub output_dir: String,
    pub findings: Vec<ReferenceFinding>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_defaults_its_limits_and_refuses_unknown_members() {
        let r: ReferenceRequest = serde_json::from_str(
            r#"{"schemaVersion":"1.0.0","outputDir":"docs/reference","specs":{"select":"all"}}"#,
        )
        .unwrap();
        assert_eq!(r.limits, ReferenceLimits::default());
        assert!(!r.traceability);
        let ids: ReferenceRequest = serde_json::from_str(
            r#"{"schemaVersion":"1.0.0","outputDir":"d","specs":{"ids":["001-a"]},"catalog":null}"#,
        )
        .unwrap();
        assert!(matches!(ids.specs, ReferenceSpecs::Ids(_)));
        assert!(
            serde_json::from_str::<ReferenceRequest>(
                r#"{"schemaVersion":"1.0.0","outputDir":"d","specs":{"select":"all"},"extra":1}"#
            )
            .is_err()
        );
    }
}
