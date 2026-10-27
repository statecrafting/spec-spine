//! `spec-spine resolve-derived`: settle a stopped merge's derived-tree
//! conflicts by regenerating the tree from the complete merged inputs, or
//! refuse and say why (spec 206).
//!
//! Run after a merge, rebase or cherry-pick has stopped. The planning is the
//! core's (`spec_spine_core::resolve_derived`, which spawns nothing); this
//! module reads the state from git, writes the one regeneration run, stages
//! exactly the derived root, and asks git what the merge did to authored text.
//! It never continues, commits or aborts the operation (3.5).

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde::Serialize;
use spec_spine_core::resolve_derived::{
    self as plan, AuthoredChange, Marker, Refusal, RefusalReason, UnmergedPath,
};
use spec_spine_types::{Error, Severity, Verdict, Violation};

use crate::load_repo_config;
use crate::out;

/// The verdict envelope's `verb` (3.7).
pub const VERB: &str = "resolve-derived";

/// The operation in progress (3.2 item 3) and the two sides 3.6 compares.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Operation {
    /// `merge`, `rebase` or `cherry-pick`.
    kind: &'static str,
    /// The carried change: the diff from `carriedFrom` to `carriedTo`.
    carried_from: String,
    carried_to: String,
    /// The receiving side's commit.
    receiving: String,
}

/// The report the envelope carries (3.7).
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    /// `nothing-to-resolve`, `refused`, `invalid` or `resolved`.
    result: &'static str,
    unmerged: Vec<UnmergedPath>,
    operation: Option<Operation>,
    unstaged: Vec<String>,
    refusals: Vec<Refusal>,
    /// The merged corpus's error-tier violations, under `invalid`.
    violations: Vec<Violation>,
    regenerated: Vec<String>,
    removed: Vec<String>,
    markers: Vec<Marker>,
    authored_change: Option<AuthoredChange>,
    freshness: Option<serde_json::Value>,
    /// Set when the staged tree is not fresh after regeneration: an engine
    /// defect, not the caller's (3.7).
    engine_defect: Option<String>,
}

pub fn run(repo: &Path, json: bool) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;
    let git = Git { repo };
    git.require_top_level()?;
    let derived = trim_root(&cfg.layout.derived_dir);

    // 3.2: everything is read before anything is written.
    let unmerged = plan::parse_unmerged(&git.read(&["ls-files", "--unmerged", "-z"])?);
    let dirty = plan::parse_status(&git.read(&[
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
    ])?);
    let operation = git.operation()?;
    let decided = plan::plan(&cfg, &unmerged, &dirty, operation.is_some());

    let mut report = Report {
        unmerged: decided.unmerged.clone(),
        unstaged: decided.unstaged.clone(),
        ..Report::default()
    };

    // 3.3: nothing under the derived root is conflicted.
    if decided.nothing_to_resolve {
        report.result = "nothing-to-resolve";
        return emit(&report, 0, json, derived);
    }
    report.operation = operation;

    // 3.4 items 1, 2 and 4, decided before step 1.
    if !decided.refusals.is_empty() {
        report.result = "refused";
        report.refusals = decided.refusals;
        return emit(&report, 2, json, derived);
    }

    // 3.5 step 1: one compile and one index, in memory.
    let compiled = spec_spine_core::compile(&cfg, repo)?;
    if !compiled.validation_passed {
        report.result = "invalid";
        report.violations = compiled
            .registry
            .validation
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .cloned()
            .collect();
        return emit(&report, 1, json, derived);
    }
    let indexed = spec_spine_core::index(&cfg, repo)?;
    let run = plan::regeneration(&cfg, repo, &compiled, &indexed)?;

    // 3.4 item 3, decided from the in-memory result before anything is written.
    let unsettled = plan::unsettled(repo, &decided.derived_unmerged(), &run);
    if !unsettled.is_empty() {
        report.result = "refused";
        report.refusals = vec![Refusal::new(RefusalReason::UnsettledDerived, unsettled)];
        return emit(&report, 2, json, derived);
    }

    // 3.5 step 2: write the result and stage exactly the derived root.
    let before = git.stage_entries(derived)?;
    run.apply()?;
    git.read(&["add", "-A", "--", &literal(derived)])?;
    let after = git.stage_entries(derived)?;
    for (path, entries) in &after {
        if before.get(path) != Some(entries) {
            report.regenerated.push(path.clone());
        }
    }
    for path in before.keys() {
        if !after.contains_key(path) {
            report.removed.push(path.clone());
        }
    }
    report.result = "resolved";

    // 3.6: what the merge did to authored text.
    report.markers = git.markers(derived)?;
    let carried = report
        .operation
        .as_ref()
        .map(|op| git.change_lines(&[op.carried_from.as_str(), op.carried_to.as_str()], derived))
        .transpose()?
        .unwrap_or_default();
    let merged = report
        .operation
        .as_ref()
        .map(|op| git.change_lines(&["--cached", op.receiving.as_str()], derived))
        .transpose()?
        .unwrap_or_default();
    let change = plan::compare_changes(&carried, &merged);

    // 3.5 step 4: the reads `check` performs.
    let (check, freshness) = spec_spine_core::check_report_full(&cfg, repo)?;
    let fresh = check.registry.validation_passed
        && check.registry.fresh
        && check.index.fresh
        && freshness.blocking.is_empty();
    if !fresh {
        let mut named: Vec<String> = Vec::new();
        if let Some(actual) = &check.registry.actual {
            named.push(actual.clone());
        }
        named.extend(freshness.stale.iter().cloned());
        if !freshness.blocking.is_empty() {
            named.push(format!(
                "unresolved claim: {}",
                freshness.unresolved_claim_summary()
            ));
        }
        report.engine_defect = Some(format!(
            "the staged derived tree is not fresh after regeneration; this is a defect in the \
             engine, not the caller's: {}",
            named.join("; ")
        ));
    }
    report.freshness =
        Some(serde_json::to_value(&check).map_err(|e| Error::Internal(e.to_string()))?);

    let code = if !report.markers.is_empty() || !change.preserved || !fresh {
        1
    } else {
        0
    };
    report.authored_change = Some(change);
    emit(&report, code, json, derived)
}

/// Write the verdict: the envelope under `--json`, prose otherwise.
fn emit(report: &Report, code: u8, json: bool, derived: &str) -> Result<u8, Error> {
    if json {
        let value = serde_json::to_value(report).map_err(|e| Error::Internal(e.to_string()))?;
        out::verdict(&Verdict::report(VERB, code, value))?;
        return Ok(code);
    }
    match report.result {
        "nothing-to-resolve" => {
            outln!("resolve-derived: nothing under {derived} is conflicted; nothing written");
            for u in &report.unmerged {
                outln!("  unmerged (authored): {}", u.path);
            }
            for path in &report.unstaged {
                outln!("  not staged: {path}");
            }
        }
        "refused" => {
            for r in &report.refusals {
                eprintln!("resolve-derived: REFUSED: {}", r.message);
                for path in &r.paths {
                    eprintln!("  {path}");
                }
            }
            eprintln!("resolve-derived: nothing written; the index is unchanged");
        }
        "invalid" => {
            eprintln!(
                "resolve-derived: the merged corpus does not validate; nothing written, the \
                 conflicts under {derived} are left in place"
            );
            for v in &report.violations {
                match &v.path {
                    Some(path) => eprintln!("  {} [{path}] {}", v.code, v.message),
                    None => eprintln!("  {} {}", v.code, v.message),
                }
            }
        }
        _ => {
            for path in &report.regenerated {
                outln!("regenerated {path}");
            }
            for path in &report.removed {
                outln!("removed {path}");
            }
            if report.markers.is_empty() {
                outln!("conflict markers: none staged");
            } else {
                eprintln!("conflict markers: {} staged", report.markers.len());
                for m in &report.markers {
                    eprintln!("  {}:{}: {}", m.path, m.line, m.text);
                }
            }
            match &report.authored_change {
                Some(c) if c.preserved => outln!("carried change: preserved"),
                Some(c) => {
                    eprintln!("carried change: NOT preserved by the merge");
                    for d in &c.differences {
                        for line in &d.only_in_carried {
                            eprintln!("  {}: carried, not in the result: {line}", d.path);
                        }
                        for line in &d.only_in_merged {
                            eprintln!("  {}: in the result, not carried: {line}", d.path);
                        }
                    }
                }
                None => {}
            }
            match &report.engine_defect {
                None => {
                    outln!("spec-registry: fresh");
                    outln!("codebase-index: fresh");
                }
                Some(defect) => eprintln!("resolve-derived: {defect}"),
            }
        }
    }
    Ok(code)
}

/// `.statecraft/derived/` and `./.statecraft/derived` name one root.
fn trim_root(root: &str) -> &str {
    let root = root.strip_prefix("./").unwrap_or(root);
    root.trim_end_matches('/')
}

/// The derived root as a literal, top-anchored pathspec.
fn literal(root: &str) -> String {
    format!(":(top,literal){root}")
}

/// Everything outside the derived root.
fn outside(root: &str) -> String {
    format!(":(top,exclude,literal){root}")
}

/// The git reads and writes, all from the repository root.
struct Git<'a> {
    repo: &'a Path,
}

impl Git<'_> {
    /// Run git; exit 0 or one of `ok` is success, anything else a failure
    /// (exit 4, 3.2).
    fn run(&self, args: &[&str], ok: &[i32]) -> Result<(i32, String), Error> {
        let out = Command::new("git")
            .arg("-C")
            .arg(self.repo)
            .args(["-c", "core.quotePath=false"])
            .args(args)
            .output()
            .map_err(|e| Error::Io(format!("spawn git {}: {e}", args[0])))?;
        let code = out.status.code().unwrap_or(-1);
        if code == 0 || ok.contains(&code) {
            return Ok((code, String::from_utf8_lossy(&out.stdout).into_owned()));
        }
        Err(Error::Io(format!(
            "git {} exited {code}: {}",
            args[0],
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }

    fn read(&self, args: &[&str]) -> Result<String, Error> {
        Ok(self.run(args, &[])?.1)
    }

    /// The paths git reports are relative to the working tree's top, and the
    /// derived root is relative to `--repo`: the two must be one directory
    /// (206 D-7).
    fn require_top_level(&self) -> Result<(), Error> {
        let prefix = self.read(&["rev-parse", "--show-prefix"])?;
        if prefix.trim().is_empty() {
            return Ok(());
        }
        Err(Error::Refused(format!(
            "resolve-derived runs at the top of the working tree; {} is the subdirectory {}",
            self.repo.display(),
            prefix.trim()
        )))
    }

    /// A commit id for `rev`, or `None` when git has no such ref.
    fn commit(&self, rev: &str) -> Result<Option<String>, Error> {
        let (code, out) = self.run(
            &["rev-parse", "-q", "--verify", "--end-of-options", rev],
            &[1],
        )?;
        let id = out.trim();
        Ok((code == 0 && !id.is_empty()).then(|| id.to_string()))
    }

    /// The empty tree, for a carried change with no parent. `output()` gives
    /// git a null stdin, so `--stdin` hashes empty input on every platform,
    /// with no `/dev/null` path.
    fn empty_tree(&self) -> Result<String, Error> {
        let (_, out) = self.run(&["hash-object", "-t", "tree", "--stdin"], &[])?;
        Ok(out.trim().to_string())
    }

    /// 3.2 item 3: the operation in progress and its two sides.
    fn operation(&self) -> Result<Option<Operation>, Error> {
        if let Some(theirs) = self.commit("MERGE_HEAD^{commit}")? {
            let ours = self
                .commit("HEAD^{commit}")?
                .ok_or_else(|| Error::Io("git: a merge is in progress with no HEAD".into()))?;
            let (code, out) = self.run(&["merge-base", &ours, &theirs], &[1])?;
            let base = match out.trim() {
                id if code == 0 && !id.is_empty() => id.to_string(),
                _ => self.empty_tree()?,
            };
            return Ok(Some(Operation {
                kind: "merge",
                carried_from: base,
                carried_to: ours,
                receiving: theirs,
            }));
        }
        for (head, kind) in [
            ("REBASE_HEAD", "rebase"),
            ("CHERRY_PICK_HEAD", "cherry-pick"),
        ] {
            if let Some(applied) = self.commit(&format!("{head}^{{commit}}"))? {
                let parent = match self.commit(&format!("{applied}^1"))? {
                    Some(p) => p,
                    None => self.empty_tree()?,
                };
                let receiving = self.commit("HEAD^{commit}")?.ok_or_else(|| {
                    Error::Io(format!("git: a {kind} is in progress with no HEAD"))
                })?;
                return Ok(Some(Operation {
                    kind,
                    carried_from: parent,
                    carried_to: applied,
                    receiving,
                }));
            }
        }
        Ok(None)
    }

    /// Every index entry under the derived root: path to its sorted
    /// `mode oid stage` entries.
    fn stage_entries(&self, derived: &str) -> Result<BTreeMap<String, Vec<String>>, Error> {
        let listing = self.read(&["ls-files", "-s", "-z", "--", &literal(derived)])?;
        let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for entry in listing.split('\0') {
            if let Some((meta, path)) = entry.split_once('\t') {
                out.entry(path.to_string())
                    .or_default()
                    .push(meta.to_string());
            }
        }
        for entries in out.values_mut() {
            entries.sort();
        }
        Ok(out)
    }

    /// 3.6 item 1: staged lines outside the derived root that begin a
    /// conflict marker.
    fn markers(&self, derived: &str) -> Result<Vec<Marker>, Error> {
        let (_, listing) = self.run(
            &[
                "grep",
                "--cached",
                "--no-color",
                "-n",
                "--null",
                "-I",
                "-E",
                "-e",
                "^(<<<<<<<|>>>>>>>) ",
                "--",
                ":(top)",
                &outside(derived),
            ],
            &[1],
        )?;
        Ok(plan::parse_markers(&listing))
    }

    /// 3.6 item 2: a diff outside the derived root, reduced to its added and
    /// removed lines.
    fn change_lines(&self, sides: &[&str], derived: &str) -> Result<Vec<(String, String)>, Error> {
        let mut args = vec![
            "diff",
            "--no-color",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--no-relative",
            "--unified=0",
            "--src-prefix=a/",
            "--dst-prefix=b/",
        ];
        args.extend_from_slice(sides);
        let excluded = outside(derived);
        args.extend_from_slice(&["--", ":(top)", &excluded]);
        Ok(plan::change_lines(&self.read(&args)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_empty_tree_needs_no_dev_null() {
        let dir = tempfile::tempdir().unwrap();
        let init = Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(["init", "-q"])
            .status()
            .unwrap();
        assert!(init.success());
        let git = Git { repo: dir.path() };
        assert_eq!(
            git.empty_tree().unwrap(),
            "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
        );
    }
}
