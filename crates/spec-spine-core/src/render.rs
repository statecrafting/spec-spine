//! The render capability (spec 011): deterministic, human-shaped projections
//! of the **committed** `index.json`. Pure read-side: never recomputes the
//! index, never consults the working tree, never signals staleness
//! (recomputation is `index`, freshness is `index check`; three verbs, three
//! jobs). Output is a pure function of `(config, index)`: byte-identical
//! across platforms, LF line endings, trailing newline.

use std::fmt::Write as _;

use serde::Serialize;
use spec_spine_types::{
    CodebaseIndex, Config, Diagnostic, Implementation, PackageKind, SpecRecord, Status,
};

/// The id-sorted `traceability.orphanedSpecs` list (spec 011 §3.3).
pub fn orphans(index: &CodebaseIndex) -> Vec<&str> {
    let mut ids: Vec<&str> = index
        .traceability
        .orphaned_specs
        .iter()
        .map(String::as_str)
        .collect();
    ids.sort_unstable();
    ids
}

/// `orphans`, partitioned by whether the spec is in flight (spec 059 §3.1).
///
/// An orphan is a spec claiming nothing that resolves, and on a specify-first
/// corpus a spec whose code is not written yet claims nothing that resolves. So
/// the flat list is the corpus, and it says nothing: `index orphans` reported
/// sixty-four of hqgit's sixty-eight specs. Partitioning leaves four.
///
/// The first group is the finding: a spec that is not in flight, claiming
/// nothing that resolves, is one whose author said the work is done or does not
/// apply while the ledger says nothing of theirs exists. The second group is a
/// specify-first corpus's normal state and is reported rather than filtered,
/// because suppressing it would replace a useless answer with an incomplete one
/// and lose the verb that answers "what has no code yet".
///
/// The predicate is spec 044's, read from the **registry** rather than the
/// index: the index shard records `spec_status` but not `implementation`, and
/// adding it would move `INDEX_SCHEMA_VERSION` and restamp every shard for a
/// read verb's benefit (spec 059 §3.4 forbids that). A spec with no record is
/// treated as in flight, since a corpus that cannot say otherwise should not be
/// told its spec is abandoned. Taking the record slice rather than a `Registry`
/// is what lets the caller pass an empty one when no registry is committed: a
/// read verb that answered from the index alone must not start failing because
/// a different artifact is missing.
pub fn partition_orphans<'a>(index: &'a CodebaseIndex, records: &[SpecRecord]) -> OrphanReport<'a> {
    let mut orphaned: Vec<&'a str> = Vec::new();
    let mut in_flight: Vec<&'a str> = Vec::new();
    for id in orphans(index) {
        match records.iter().find(|s| s.id == id) {
            Some(rec) if !record_in_flight(rec) => orphaned.push(id),
            _ => in_flight.push(id),
        }
    }
    OrphanReport {
        orphaned,
        in_flight,
    }
}

/// Spec 044's in-flight predicate over a registry record: `complete` settles
/// it, else `draft` status or a `pending` / `in-progress` implementation.
///
/// Kept beside the partition rather than shared with `index.rs`, which asks the
/// same question of its own parsed frontmatter type. Both read the same two
/// fields under the same rule, and the acceptance pins them to the same answer.
fn record_in_flight(rec: &SpecRecord) -> bool {
    if matches!(rec.implementation, Some(Implementation::Complete)) {
        return false;
    }
    rec.status == Status::Draft
        || matches!(
            rec.implementation,
            Some(Implementation::Pending | Implementation::InProgress)
        )
}

/// The two groups `index orphans` reports (spec 059 §3.1). Both id-sorted, as
/// spec 011 §3.3 requires.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrphanReport<'a> {
    /// Claims nothing that resolves, and is not in flight: the finding.
    pub orphaned: Vec<&'a str>,
    /// Claims nothing that resolves *yet*: a specify-first corpus's normal
    /// state, reported rather than filtered.
    pub in_flight: Vec<&'a str>,
}

/// The markdown projection of the committed index (spec 011 §3.2).
///
/// Section inventory and order are the v1 contract: header, package
/// inventory, traceability (orphans / untraced flat lists omitted when
/// empty), diagnostics (omitted when empty). The prose between sections is
/// not contractual.
pub fn render_markdown(config: &Config, index: &CodebaseIndex) -> String {
    let mut out = String::new();

    // 1. Header, traceable to the exact artifact that produced it.
    let _ = writeln!(out, "# {} codebase index", config.branding.indexer_id);
    out.push('\n');
    let _ = writeln!(out, "- schemaVersion: {}", index.schema_version);
    let _ = writeln!(out, "- contentHash: {}", index.build.content_hash);

    // 2. Package inventory, sorted by name, ties by path.
    out.push_str("\n## Packages\n\n");
    let mut packages: Vec<_> = index.packages.iter().collect();
    packages.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.path.cmp(&b.path)));
    out.push_str("| name | path | kind | version | spec |\n");
    out.push_str("|---|---|---|---|---|\n");
    for p in packages {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            p.name,
            p.path,
            kind_label(p.kind),
            p.version.as_deref().unwrap_or("-"),
            p.spec_ref.as_deref().unwrap_or("-"),
        );
    }

    // 3. Traceability: per-spec summary, then the flat lists (omitted when
    //    empty).
    out.push_str("\n## Traceability\n\n");
    let mut mappings: Vec<_> = index.traceability.mappings.iter().collect();
    mappings.sort_by(|a, b| a.spec_id.cmp(&b.spec_id));
    out.push_str("| spec | status | paths | units |\n");
    out.push_str("|---|---|---|---|\n");
    for m in mappings {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            m.spec_id,
            m.spec_status.as_deref().unwrap_or("-"),
            m.implementing_paths.len(),
            m.resolved_units.len(),
        );
    }
    if !index.traceability.orphaned_specs.is_empty() {
        out.push_str("\n### Orphaned specs\n\n");
        for id in orphans(index) {
            let _ = writeln!(out, "- {id}");
        }
    }
    if !index.traceability.untraced_code.is_empty() {
        out.push_str("\n### Untraced code\n\n");
        let mut paths: Vec<&str> = index
            .traceability
            .untraced_code
            .iter()
            .map(String::as_str)
            .collect();
        paths.sort_unstable();
        for path in paths {
            let _ = writeln!(out, "- {path}");
        }
    }

    // 4. Diagnostics, sorted by (code, file); omitted when empty.
    let mut diagnostics: Vec<(&Diagnostic, &str)> = index
        .diagnostics
        .errors
        .iter()
        .map(|d| (d, "error"))
        .chain(index.diagnostics.warnings.iter().map(|d| (d, "warning")))
        .collect();
    if !diagnostics.is_empty() {
        diagnostics.sort_by_key(|(d, _)| (d.code.as_str(), d.path.as_deref().unwrap_or("")));
        out.push_str("\n## Diagnostics\n\n");
        for (d, severity) in diagnostics {
            match &d.path {
                Some(path) => {
                    let _ = writeln!(out, "- {} [{severity}] {} ({path})", d.code, d.message);
                }
                None => {
                    let _ = writeln!(out, "- {} [{severity}] {}", d.code, d.message);
                }
            }
        }
    }

    out
}

/// The kebab-case kind label, matching the serde wire form in `index.json`.
fn kind_label(kind: PackageKind) -> &'static str {
    match kind {
        PackageKind::RustLib => "rust-lib",
        PackageKind::RustBin => "rust-bin",
        PackageKind::RustLibBin => "rust-lib-bin",
        PackageKind::NpmPackage => "npm-package",
        PackageKind::NpmWorkspace => "npm-workspace",
    }
}
