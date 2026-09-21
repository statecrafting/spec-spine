// Spec: specs/096-compaction-is-a-verb-not-a-session/spec.md
//! `spec-spine compact`: apply an authored compaction plan to the corpus.
//!
//! The library returns the rewritten corpus as data and this is the consumer
//! that writes it (spec 096 §3.1, D-2), which is the shape the governance
//! scaffold already has. `--plan` is therefore free: the same call, nothing
//! written.

use std::path::{Path, PathBuf};
use std::process::Command;

use spec_spine_core::compact::{Compaction, compact, parse_plan};
use spec_spine_types::Error;

use crate::load_repo_config;
use crate::out;

/// Where the map document lands unless the caller says otherwise (spec 096
/// §3.7, over the path spec 095 §3.4 establishes).
pub const DEFAULT_MAP_PATH: &str = "docs/corpus-map.md";

pub struct CompactArgs {
    pub plan_file: PathBuf,
    pub plan: bool,
    pub force: bool,
    pub map_out: PathBuf,
}

pub fn run(repo: &Path, args: &CompactArgs) -> Result<u8, Error> {
    let src = std::fs::read_to_string(&args.plan_file).map_err(|e| {
        Error::Io(format!(
            "compact: cannot read the plan at {}: {e}",
            args.plan_file.display()
        ))
    })?;
    let plan = parse_plan(&src)?;

    // Spec 096 §3.8: the output is a rewrite of the whole tree, and an
    // uncommitted edit underneath it is unreviewable. Refused before the
    // corpus is read, so a dirty tree costs nothing.
    if !args.plan && !args.force {
        refuse_dirty_tree(repo)?;
    }

    let cfg = load_repo_config(repo)?;
    let outcome = compact(&cfg, repo, &plan)?;

    // Spec 097 §3.7: an occurrence the rules could not account for means the
    // tool has met a spelling it has no rule for. Reported and refused, whether
    // or not anything was written: a rewrite that silently left the path in
    // place is the failure this verb exists to prevent.
    // The refusal comes FIRST, and the summary does not print at all. Printing
    // the rewrite summary and then "nothing was written" gave a reader two
    // contradictory accounts of the same run and made them decide which one was
    // true.
    if !outcome.leftover.is_empty() {
        out::line(format_args!(
            "\ncompact: REFUSED, {} occurrence(s) of a retired path survived the rewrite and no \
             clause spared them:",
            outcome.leftover.len()
        ));
        for l in &outcome.leftover {
            out::line(format_args!(
                "  {}:{} [{}] {}",
                l.rel_path, l.line, l.path, l.text
            ));
        }
        out::line(format_args!(
            "compact: declare a form rule that covers them, name the file or section historical, \
             or fix the occurrence. Nothing was written."
        ));
        return Ok(1);
    }

    report(&outcome, args.plan);

    if args.plan {
        out::line(format_args!(
            "\ncompact: --plan, so nothing was written. {} rewrite(s) in {} file(s) would be applied.",
            outcome.rewrite_count(),
            outcome.rewrites.len()
        ));
        return Ok(0);
    }

    apply(repo, &outcome, &args.map_out)?;
    out::line(format_args!(
        "\ncompact: applied {} rewrite(s) across {} file(s); {} spec(s) removed. \
         Regenerate the derived trees (`spec-spine compile` and `index`) and commit them with this change.",
        outcome.rewrite_count(),
        outcome.rewrites.len(),
        outcome.removed_paths.len()
    ));
    Ok(0)
}

/// The report IS the review (spec 096 §3.6): a six-thousand-line diff is not
/// reviewable, and the question a reviewer has is not "what changed" but "was
/// each change the right rule".
fn report(outcome: &Compaction, verbose: bool) {
    out::line(format_args!("map ({} id(s)):", outcome.map.len()));
    for e in &outcome.map {
        let note = if e.removed { "  (removed)" } else { "" };
        if e.from != e.to || e.removed {
            out::line(format_args!("  {} -> {}{note}", e.from, e.to));
        }
    }
    out::line(format_args!("\nrewrites by form:"));
    for (form, n) in &outcome.counts {
        out::line(format_args!("  {form:<14} {n}"));
    }
    // Spec 097 §3.5: an occurrence left alone is reported with the clause that
    // spared it. Printed on every run, not only the verbose one: the exclusions
    // are where a retirement goes quietly wrong, and a reader who has to ask
    // for them is the reader who will not.
    if !outcome.skipped.is_empty() {
        out::line(format_args!(
            "\nleft alone ({}), with the clause that spared each:",
            outcome.skipped.len()
        ));
        for s in &outcome.skipped {
            // The path is printed, not only the clause: two retired paths can
            // be spared on one line, and two records differing only in a field
            // the report omits are two lines a reader cannot tell apart.
            out::line(format_args!(
                "  {}:{} [{} {}] {}",
                s.rel_path,
                s.line,
                s.path,
                s.clause.as_str(),
                s.text
            ));
        }
    }
    if verbose {
        out::line(format_args!("\nrewrites by file:"));
        for file in &outcome.rewrites {
            out::line(format_args!("  {}", file.rel_path));
            for r in &file.rewrites {
                out::line(format_args!(
                    "    {}:{} [{}] {} -> {}",
                    file.rel_path,
                    r.line,
                    r.form.as_str(),
                    r.old,
                    r.new
                ));
            }
        }
    }
}

/// Write the rewritten corpus.
///
/// **Not atomic**, and deliberately not: the three phases (remove, write and
/// rename, emit the map) run in order with no rollback, so a failure part way
/// through leaves a partly rewritten tree. `refuse_dirty_tree` is what makes
/// that recoverable rather than safe: the tree was clean when this started, so
/// `git checkout .` restores it exactly. A transactional writer would need a
/// staging copy of the whole repository to buy a property `git` already has.
fn apply(repo: &Path, outcome: &Compaction, map_out: &Path) -> Result<(), Error> {
    let io = |e: std::io::Error, what: &str| Error::Io(format!("compact: {what}: {e}"));
    // Each entry is a spec DIRECTORY, which is what the report lists and what
    // leaves the tree.
    for rel in &outcome.removed_paths {
        std::fs::remove_dir_all(repo.join(rel)).map_err(|e| io(e, &format!("removing {rel}")))?;
    }
    for f in &outcome.files {
        let to = repo.join(&f.rel_path);
        if let Some(dir) = to.parent() {
            std::fs::create_dir_all(dir).map_err(|e| io(e, &format!("creating {}", f.rel_path)))?;
        }
        std::fs::write(&to, &f.contents).map_err(|e| io(e, &format!("writing {}", f.rel_path)))?;
        if f.rel_path != f.from_rel_path {
            let from = repo.join(&f.from_rel_path);
            std::fs::remove_file(&from)
                .map_err(|e| io(e, &format!("removing {}", f.from_rel_path)))?;
            if let Some(dir) = from.parent() {
                let _ = std::fs::remove_dir(dir);
            }
        }
    }
    let map_path = repo.join(map_out);
    if let Some(dir) = map_path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| io(e, "creating the map directory"))?;
    }
    std::fs::write(&map_path, &outcome.map_document)
        .map_err(|e| io(e, &format!("writing {}", map_out.display())))?;
    Ok(())
}

/// A git failure refuses rather than falling back: a rewrite of this size onto
/// a tree whose state could not be read is exactly what `--force` is for, and
/// it should be the caller saying so.
fn refuse_dirty_tree(repo: &Path) -> Result<(), Error> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| {
            Error::Io(format!(
                "compact: the working tree's state could not be read ({e}); \
                 run inside a git repository, or pass --force"
            ))
        })?;
    if !out.status.success() {
        return Err(Error::Io(format!(
            "compact: `git status` exited {:?}: {}; pass --force to rewrite anyway",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    let dirty = String::from_utf8_lossy(&out.stdout);
    if !dirty.trim().is_empty() {
        return Err(Error::Config(format!(
            "compact: the working tree is dirty, and a rewrite of the whole corpus \
             underneath an uncommitted edit is unreviewable. Commit or stash first, \
             or pass --force. If a previous run failed part way through, \
             `git checkout .` restores the tree it started from.\n{}",
            dirty.trim()
        )));
    }
    Ok(())
}
