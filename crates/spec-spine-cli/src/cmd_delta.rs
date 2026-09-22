//! `spec-spine delta`: a change classified under the base's rules (spec 071).
//!
//! The git half of the verb, and nothing else. It resolves `merge-base(base,
//! head)` and both refs to commits, lists the changed paths of
//! `merge-base...head` with renames disabled, exports the merge-base tree and
//! the head tree (the committed `.derived/` included) into two temporary
//! directories, and hands the roots, the paths and the three commit ids to the
//! pure core. The directories are removed afterwards, on the error path too.
//!
//! It is a record, not a gate: exit 0 whenever a report was produced, whatever
//! it says (spec 071 §3.1, following spec 039 3.1 for `attest`).

use std::path::{Path, PathBuf};
use std::process::Command;

use spec_spine_core::delta;
use spec_spine_types::{DeltaCommits, DeltaReport, Error, verdict::Verdict, verdict::verb};

use crate::cmd_couple::{changed_path_names, merge_base};
use crate::load_repo_config;
use crate::out;

/// Arguments for `spec-spine delta`.
pub struct DeltaArgs {
    pub base: String,
    pub head: String,
    /// Emit the report as a JSON envelope instead of prose (spec 034).
    pub json: bool,
}

pub fn run(repo: &Path, args: &DeltaArgs) -> Result<u8, Error> {
    let commits = DeltaCommits {
        base: resolve_commit(repo, &args.base)?,
        merge_base: merge_base(repo, &args.base, &args.head)?,
        head: resolve_commit(repo, &args.head)?,
    };
    let changed = changed_path_names(repo, &commits.merge_base, &commits.head)?;

    let trees = TempTrees::create()?;
    export_tree(
        repo,
        &commits.merge_base,
        &trees.root.join("base.index"),
        &trees.base,
    )?;
    export_tree(
        repo,
        &commits.head,
        &trees.root.join("head.index"),
        &trees.head,
    )?;
    // The merge base's rules (spec 071 §3.2, D-1): the configuration is read
    // from the exported base tree, never from the working tree or the head.
    let cfg = load_repo_config(&trees.base)?;
    let report = delta(&cfg, &trees.base, &trees.head, &changed, &commits)?;
    drop(trees);

    if args.json {
        let value = serde_json::to_value(&report).map_err(|e| Error::Schema(e.to_string()))?;
        out::verdict(&Verdict::report(verb::DELTA, 0, value))?;
    } else {
        render(&report);
    }
    Ok(0)
}

/// The prose rendering: one line per path, then the prior-policy answer.
fn render(report: &DeltaReport) {
    outln!(
        "spec-spine delta: {} path(s) changed, classified under the base's rules (merge base {})",
        report.changes.len(),
        short(&report.merge_base)
    );
    let width = report
        .changes
        .iter()
        .map(|c| c.path.chars().count())
        .max()
        .unwrap_or(0);
    for change in &report.changes {
        let classes: Vec<String> = change.classes.iter().map(|c| class_token(*c)).collect();
        outln!(
            "  {:<width$}  {}",
            change.path,
            classes.join(", "),
            width = width
        );
    }
    if report.prior_policy.required {
        let classes: Vec<String> = report
            .prior_policy
            .classes
            .iter()
            .map(|c| class_token(*c))
            .collect();
        outln!(
            "prior policy: REQUIRED for {} (judge these under the base revision's policy)",
            classes.join(", ")
        );
    } else {
        outln!(
            "prior policy: not required. This means only that no structural class changed; \
             it does not mean the change is safe, correct or approved."
        );
    }
}

/// A class's report token, read from its own serialization so the prose and the
/// JSON cannot spell one class two ways.
fn class_token(class: spec_spine_types::DeltaClass) -> String {
    serde_json::to_value(class)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn short(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}

/// `<rev>^{commit}`, as a full commit id.
fn resolve_commit(repo: &Path, rev: &str) -> Result<String, Error> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        // `--end-of-options`: a ref that looks like a flag stays an operand.
        .args(["rev-parse", "--verify", "--quiet", "--end-of-options"])
        .arg(format!("{rev}^{{commit}}"))
        .output()
        .map_err(|e| Error::Io(format!("spawn git rev-parse: {e}")))?;
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !out.status.success() || id.is_empty() {
        return Err(Error::Io(format!("git: '{rev}' does not name a commit")));
    }
    Ok(id)
}

/// Write every file of `commit`'s tree under `dest`.
///
/// Through a private index file (`read-tree`, then `checkout-index`), so the
/// repository's own index, working tree and worktree list are never touched.
/// `git archive` is not used: it honors `export-ignore`, and a candidate could
/// then hide a path from the tree its change is classified against.
///
/// `pub(crate)` since spec 100: the coupling gate reconstructs a deleted path's
/// prior ownership from the same kind of exported tree, and a second exporter
/// that agreed today and drifted next quarter would be worse than sharing one.
pub(crate) fn export_tree(
    repo: &Path,
    commit: &str,
    index_file: &Path,
    dest: &Path,
) -> Result<(), Error> {
    let git = |args: &[&str]| -> Result<(), Error> {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .env("GIT_INDEX_FILE", index_file)
            .output()
            .map_err(|e| Error::Io(format!("spawn git {}: {e}", args[0])))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(Error::Io(format!(
                "git {} exited {:?}: {}",
                args[0],
                out.status.code(),
                String::from_utf8_lossy(&out.stderr).trim()
            )))
        }
    };
    git(&["read-tree", "--end-of-options", commit])?;
    let prefix = format!("--prefix={}/", dest.display());
    git(&["checkout-index", "--all", "--force", &prefix])
}

/// The two export directories, removed when this is dropped.
struct TempTrees {
    root: PathBuf,
    base: PathBuf,
    head: PathBuf,
}

impl TempTrees {
    fn create() -> Result<Self, Error> {
        let parent = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        // `create_dir`, not `create_dir_all`: an existing directory is someone
        // else's, so a name collision tries the next name rather than reusing it.
        for attempt in 0..16u32 {
            let root = parent.join(format!("spec-spine-delta-{pid}-{nanos}-{attempt}"));
            match std::fs::create_dir(&root) {
                Ok(()) => {
                    let trees = TempTrees {
                        base: root.join("base"),
                        head: root.join("head"),
                        root,
                    };
                    for dir in [&trees.base, &trees.head] {
                        std::fs::create_dir(dir)
                            .map_err(|e| Error::Io(format!("create {}: {e}", dir.display())))?;
                    }
                    return Ok(trees);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => {
                    return Err(Error::Io(format!("create {}: {e}", root.display())));
                }
            }
        }
        Err(Error::Io(format!(
            "could not create a temporary directory under {}",
            parent.display()
        )))
    }
}

impl Drop for TempTrees {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
