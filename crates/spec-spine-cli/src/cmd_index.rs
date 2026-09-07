//! `spec-spine index`: write the per-spec/per-package index shards (spec 024)
//! under `<derived>/codebase-index/{by-spec,by-package}/`; `spec-spine index
//! check`: per-shard staleness; `spec-spine index render` / `index orphans`:
//! read-side projections of the committed shard set (spec 011; never recompute,
//! never check freshness); `spec-spine index coverage`: file-granular
//! ownership coverage of the tree against the committed shards (spec 032;
//! freshness-guarded like `couple`). The single monolithic `index.json` is no
//! longer emitted, so PRs touching different specs/packages write disjoint files.

use std::fs;
use std::path::Path;

use clap::Subcommand;
use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    DiagnosticCounts, Freshness, IndexCheckReport, check_index_freshness, check_slice_freshness,
    committed_counts, committed_diagnostics, coverage, index, index_dir, index_shard_files,
    load_committed_index, orphans, render_markdown, slices_path,
};
use spec_spine_types::{Config, CoverageReport, Error, Verdict, verdict::verb};

use crate::load_repo_config;
use crate::out;

#[derive(Subcommand)]
pub enum IndexAction {
    /// Check the committed index against current inputs (the staleness gate).
    Check {
        /// Gate one named [index.slices] slice instead of the shard set.
        #[arg(long, value_name = "NAME")]
        slice: Option<String>,
        /// Fail (exit 1) if the committed index records any unresolved unit
        /// (`W-001` / `W-002`). Opt-in: specs 025 and 044 exist to let a corpus
        /// that ratifies before it builds carry these while work is under way.
        #[arg(long)]
        fail_on_unresolved: bool,
        /// Emit the verdict as a JSON envelope on stdout (spec 037).
        #[arg(long)]
        json: bool,
    },
    /// Render the committed index as markdown (a projection; never recomputes).
    Render,
    /// List orphaned specs from the committed index.
    Orphans {
        #[arg(long)]
        json: bool,
    },
    /// List the diagnostics the committed index records (spec 050).
    ///
    /// A read verb beside `orphans` and `coverage`: it recomputes nothing and
    /// never refuses, so a consumer reaches a structured fact without parsing
    /// `index render`'s markdown. The refusal lives on `check`.
    Diagnostics {
        #[arg(long)]
        json: bool,
    },
    /// Report which source files no spec specifically claims (spec 032).
    Coverage {
        #[arg(long)]
        json: bool,
        /// Fail (exit 1) unless every source file has a specific owning spec.
        #[arg(long)]
        fail_on_untraced: bool,
    },
}

/// `index` (no action) writes the shard tree; `index check` verifies freshness.
pub fn run(repo: &Path, action: Option<&IndexAction>) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;

    match action {
        Some(IndexAction::Render) => {
            let idx = load_committed_index(&cfg, repo)?;
            out!("{}", render_markdown(&cfg, &idx));
            Ok(0)
        }
        Some(IndexAction::Orphans { json }) => {
            let idx = load_committed_index(&cfg, repo)?;
            let ids = orphans(&idx);
            if *json {
                let s =
                    serde_json::to_string_pretty(&ids).map_err(|e| Error::Schema(e.to_string()))?;
                outln!("{s}");
            } else {
                for id in ids {
                    outln!("{id}");
                }
            }
            Ok(0)
        }
        Some(IndexAction::Diagnostics { json }) => {
            let diags = committed_diagnostics(&cfg, repo)?;
            if *json {
                let s = serde_json::to_string_pretty(&diags)
                    .map_err(|e| Error::Schema(e.to_string()))?;
                outln!("{s}");
            } else {
                for d in &diags {
                    let at = d.path.as_deref().unwrap_or("-");
                    outln!("  {} [{}] [{}] {}", d.code, d.spec_id, at, d.message);
                }
            }
            Ok(0)
        }
        Some(IndexAction::Coverage {
            json,
            fail_on_untraced,
        }) => {
            // Freshness-guarded inside `coverage`: a stale index is `Error::Stale`
            // (exit 2), so the report never describes the wrong ledger.
            let report = coverage(&cfg, repo)?;
            if *json {
                let s = serde_json::to_string_pretty(&report)
                    .map_err(|e| Error::Schema(e.to_string()))?;
                outln!("{s}");
            } else {
                out!("{}", render_coverage(&report));
            }
            Ok(if *fail_on_untraced && !report.is_fully_claimed() {
                1
            } else {
                0
            })
        }
        Some(IndexAction::Check {
            slice,
            fail_on_unresolved,
            json,
        }) => {
            let (freshness, subject) = match slice {
                Some(name) => (
                    check_slice_freshness(&cfg, repo, name)?,
                    format!("slice '{name}'"),
                ),
                None => (check_index_freshness(&cfg, repo)?, "index".to_string()),
            };
            let counts = committed_counts(&cfg, repo)?;

            // Spec 050 3.3: staleness outranks unresolution. A stale index's
            // diagnostics describe a tree that no longer exists, so refusing
            // for them would name the wrong problem. The counts are still
            // reported either way; suppressing them would hide the number the
            // operator ran the command for.
            let code = if matches!(freshness, Freshness::Fresh) {
                if *fail_on_unresolved && counts.has_unresolved() {
                    1
                } else {
                    0
                }
            } else {
                2
            };

            if *json {
                // One shape, built in core (`IndexCheckReport`), so the facade
                // and this arm cannot drift; spec 037 pins them against each
                // other. `compile --check` keeps the bare freshness object:
                // index diagnostics are meaningless for the registry (3.1).
                let report =
                    serde_json::to_value(IndexCheckReport::new(&freshness, counts.clone()))
                        .map_err(|e| Error::Schema(e.to_string()))?;
                out::verdict(&Verdict::report(verb::INDEX_CHECK, code, report))?;
                return Ok(code);
            }

            match freshness {
                Freshness::Fresh => outln!("{subject} is fresh{}", counts_suffix(&counts)),
                Freshness::Stale { expected, actual } => {
                    eprintln!("{subject} is STALE (run `spec-spine index` to refresh)");
                    eprintln!("  expected: {expected}");
                    eprintln!("  actual:   {actual}");
                    if !counts.is_empty() {
                        eprintln!("  the stale ledger also records{}", counts_suffix(&counts));
                    }
                }
            }
            if code == 1 {
                eprintln!("{subject}: refusing on unresolved units (--fail-on-unresolved)");
            }
            Ok(code)
        }
        None => {
            let outcome = index(&cfg, repo)?;
            let dir = index_dir(&cfg, repo);
            fs::create_dir_all(&dir)
                .map_err(|e| Error::Io(format!("create {}: {e}", dir.display())))?;

            // Per-spec + per-package shards; `sync_dir` prunes a removed unit's
            // shard so the shard set always equals the current corpus.
            let (by_spec, by_package) = index_shard_files(&outcome.shards)?;
            shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec)?;
            shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package)?;
            write_slices(&cfg, repo, &outcome.index.build.slice_hashes)?;

            // Drop a pre-024 monolithic index.json on upgrade.
            let legacy = dir.join("index.json");
            if legacy.exists() {
                fs::remove_file(&legacy)
                    .map_err(|e| Error::Io(format!("remove {}: {e}", legacy.display())))?;
            }

            // Print both tiers. Spec 025 downgrades an unresolved unit on an
            // in-flight spec (or a `references` edge) to a counted `W-001` /
            // `W-002`; those land in the shard either way, but a warning the
            // operator never sees is a unit that quietly went unresolved.
            let idx = &outcome.index;
            for diag in idx
                .diagnostics
                .errors
                .iter()
                .chain(idx.diagnostics.warnings.iter())
            {
                let at = diag.path.as_deref().unwrap_or("-");
                eprintln!("  {} [{}] {}", diag.code, at, diag.message);
            }
            outln!(
                "indexed {} package(s), {} mapping(s) -> {} ({} error diagnostic(s), {} warning(s))",
                idx.packages.len(),
                idx.traceability.mappings.len(),
                dir.display(),
                idx.diagnostics.errors.len(),
                idx.diagnostics.warnings.len()
            );
            Ok(0)
        }
    }
}

/// The counts appended to `index check`'s verdict line, or `""` when the
/// committed index records nothing.
///
/// A clean corpus keeps printing the bare `index is fresh`, so a tree with no
/// diagnostics reads exactly as it did before spec 050 (3.1).
fn counts_suffix(counts: &DiagnosticCounts) -> String {
    if counts.is_empty() {
        return String::new();
    }
    format!(" ({})", counts_summary(counts))
}

/// `"1 warning(s), 0 error(s): 1 W-001"`. Bare, so a caller can place it in a
/// sentence as well as in parentheses.
fn counts_summary(counts: &DiagnosticCounts) -> String {
    let by_code = counts
        .by_code
        .iter()
        .map(|(code, n)| format!("{n} {code}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{} warning(s), {} error(s): {by_code}",
        counts.warnings, counts.errors
    )
}

/// The human form of the coverage report: one headline, one line per package,
/// then the two debt lists (omitted when empty).
fn render_coverage(report: &CoverageReport) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let total = report.source_files;
    if total == 0 {
        out.push_str("coverage: no source files under any discovered package\n");
        return out;
    }
    let pct = (report.claimed_files as f64) * 100.0 / (total as f64);
    let _ = writeln!(
        out,
        "coverage: {}/{total} source files specifically claimed ({pct:.1}%); {} floor-only, {} unclaimed",
        report.claimed_files,
        report.floor_only_files.len(),
        report.unclaimed_files.len()
    );
    for p in &report.packages {
        let path = if p.path.is_empty() {
            "."
        } else {
            p.path.as_str()
        };
        let floor = p
            .floor_spec
            .as_deref()
            .map(|s| format!("floor {s}"))
            .unwrap_or_else(|| "no floor".to_string());
        let _ = writeln!(
            out,
            "  {path} ({floor}): {}/{} claimed, {} floor-only, {} unclaimed",
            p.claimed_files, p.source_files, p.floor_only, p.unclaimed
        );
    }
    if !report.floor_only_files.is_empty() {
        out.push_str("\nfloor-only (owned only by a package floor; claim in a spec):\n");
        for f in &report.floor_only_files {
            let _ = writeln!(out, "  {f}");
        }
    }
    if !report.unclaimed_files.is_empty() {
        out.push_str("\nunclaimed (no owning spec):\n");
        for f in &report.unclaimed_files {
            let _ = writeln!(out, "  {f}");
        }
    }
    out
}

/// Write (or remove) the per-slice sidecar `slices.json` (spec 012/024). The
/// slices live in their own small file emitted only when `[index.slices]` is
/// configured, so a corpus with no slices commits no such file. Canonical
/// (`BTreeMap` ⇒ sorted keys, 2-space, trailing LF).
fn write_slices(
    cfg: &Config,
    repo: &Path,
    slice_hashes: &std::collections::BTreeMap<String, String>,
) -> Result<(), Error> {
    let path = slices_path(cfg, repo);
    if slice_hashes.is_empty() {
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| Error::Io(format!("remove {}: {e}", path.display())))?;
        }
        return Ok(());
    }
    let json = serde_json::to_string_pretty(slice_hashes)
        .map_err(|e| Error::Schema(e.to_string()))?
        + "\n";
    fs::write(&path, json).map_err(|e| Error::Io(format!("write {}: {e}", path.display())))?;
    Ok(())
}
