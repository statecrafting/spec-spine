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
    Freshness, check_index_freshness, check_slice_freshness, coverage, index, index_dir,
    index_shard_files, load_committed_index, orphans, render_markdown, slices_path,
};
use spec_spine_types::{Config, CoverageReport, Error};

use crate::load_repo_config;

#[derive(Subcommand)]
pub enum IndexAction {
    /// Check the committed index against current inputs (the staleness gate).
    Check {
        /// Gate one named [index.slices] slice instead of the shard set.
        #[arg(long, value_name = "NAME")]
        slice: Option<String>,
    },
    /// Render the committed index as markdown (a projection; never recomputes).
    Render,
    /// List orphaned specs from the committed index.
    Orphans {
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
            print!("{}", render_markdown(&cfg, &idx));
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
                print!("{}", render_coverage(&report));
            }
            Ok(if *fail_on_untraced && !report.is_fully_claimed() {
                1
            } else {
                0
            })
        }
        Some(IndexAction::Check { slice }) => {
            let (freshness, subject) = match slice {
                Some(name) => (
                    check_slice_freshness(&cfg, repo, name)?,
                    format!("slice '{name}'"),
                ),
                None => (check_index_freshness(&cfg, repo)?, "index".to_string()),
            };
            match freshness {
                Freshness::Fresh => {
                    outln!("{subject} is fresh");
                    Ok(0)
                }
                Freshness::Stale { expected, actual } => {
                    eprintln!("{subject} is STALE (run `spec-spine index` to refresh)");
                    eprintln!("  expected: {expected}");
                    eprintln!("  actual:   {actual}");
                    Ok(2)
                }
            }
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

            let idx = &outcome.index;
            for diag in &idx.diagnostics.errors {
                let at = diag.path.as_deref().unwrap_or("-");
                eprintln!("  {} [{}] {}", diag.code, at, diag.message);
            }
            outln!(
                "indexed {} package(s), {} mapping(s) -> {} ({} error diagnostic(s))",
                idx.packages.len(),
                idx.traceability.mappings.len(),
                dir.display(),
                idx.diagnostics.errors.len()
            );
            Ok(0)
        }
    }
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
