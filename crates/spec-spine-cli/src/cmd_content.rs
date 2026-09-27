//! `spec-spine content select`: bind selected content to a verified Git tree.

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Subcommand;
use spec_spine_core::{Versioning, read_document, selected_content};
use spec_spine_types::{
    ContentDirtyState, ContentRequest, ContentSnapshot, ContentSnapshotBinding, Error, RepoPath,
};

use crate::{cmd_delta::export_tree, load_repo_config};

#[derive(Subcommand)]
pub enum ContentAction {
    /// Select bounded content from HEAD or one exact exported revision.
    Select {
        /// JSON content request document.
        #[arg(long, value_name = "FILE")]
        request: PathBuf,
        /// Opaque repository identity carried into the response.
        #[arg(long)]
        repository: String,
        /// Commit-ish to export. Omit to bind the clean working tree at HEAD.
        #[arg(long)]
        revision: Option<String>,
        /// Emit the canonical read document.
        #[arg(long, required = true)]
        json: bool,
    },
}

pub fn run(repo: &Path, action: &ContentAction) -> Result<u8, Error> {
    match action {
        ContentAction::Select {
            request,
            repository,
            revision,
            json: _,
        } => run_select(repo, request, repository, revision.as_deref()),
    }
}

fn run_select(
    repo: &Path,
    request_path: &Path,
    repository: &str,
    revision: Option<&str>,
) -> Result<u8, Error> {
    let text = std::fs::read_to_string(request_path).map_err(|e| {
        Error::Io(format!(
            "read content request {}: {e}",
            request_path.display()
        ))
    })?;
    let request: ContentRequest = serde_json::from_str(&text)
        .map_err(|e| Error::Usage(format!("invalid content request: {e}")))?;

    let (commitish, dirty_state) = if let Some(rev) = revision {
        (rev, ContentDirtyState::CleanExport)
    } else {
        require_clean(repo)?;
        ("HEAD", ContentDirtyState::CleanWorkingTree)
    };
    let commit = resolve_object(repo, commitish, "commit")?;
    let tree = resolve_object(repo, &commit, "tree")?;
    let export = TempExport::create()?;
    let index = export.root.join("index");
    export_tree(repo, &commit, &index, &export.checkout)?;
    materialize_tree(repo, &tree, &export.tree)?;
    let cfg = load_repo_config(&export.tree)?;
    let snapshot = ContentSnapshot {
        repository: repository.to_string(),
        revision: commit,
        tree,
        dirty_state,
        binding: ContentSnapshotBinding::CallerSupplied,
    };
    let response = selected_content(&cfg, &export.tree, &request, &snapshot)?;
    let _guard = export;
    let document = read_document(&response, Versioning::Stamp)?;
    out!("{document}");
    Ok(0)
}

/// Materialize the exact blob bytes named by `tree` without checkout filters.
///
/// `checkout-index` is not suitable here: attributes such as `eol=crlf` and
/// configured smudge filters can change or execute while materializing. Every
/// regular file and symlink is instead represented by its Git blob bytes.
fn materialize_tree(repo: &Path, tree: &str, export: &Path) -> Result<(), Error> {
    let listed = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-tree", "-r", "-z", "--full-tree", tree])
        .output()
        .map_err(|e| Error::Io(format!("spawn git ls-tree: {e}")))?;
    if !listed.status.success() {
        return Err(Error::Io("could not enumerate exported revision".into()));
    }
    for entry in listed.stdout.split(|b| *b == 0).filter(|e| !e.is_empty()) {
        let Some(tab) = entry.iter().position(|b| *b == b'\t') else {
            return Err(Error::Io("invalid git ls-tree output".into()));
        };
        let header = String::from_utf8_lossy(&entry[..tab]);
        let mut fields = header.split_whitespace();
        let _mode = fields.next().unwrap_or_default();
        let kind = fields.next().unwrap_or_default();
        let object = fields.next().unwrap_or_default();
        if kind == "commit" {
            continue;
        }
        if kind != "blob" {
            return Err(Error::Io(format!("unsupported tree entry type '{kind}'")));
        }
        let path = std::str::from_utf8(&entry[tab + 1..])
            .map_err(|_| Error::Io("non-UTF-8 path in exported revision".into()))?;
        let path = RepoPath::parse(path)
            .map_err(|why| Error::Io(format!("unsafe path in exported revision: {why}")))?;
        let blob = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["cat-file", "blob", object])
            .output()
            .map_err(|e| Error::Io(format!("spawn git cat-file: {e}")))?;
        if !blob.status.success() {
            return Err(Error::Io(format!(
                "could not read exported blob for '{}'",
                path.as_str()
            )));
        }
        let materialized = export.join(path.as_str());
        let parent = materialized
            .parent()
            .ok_or_else(|| Error::Io("exported path has no parent".into()))?;
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::Io(format!("create {}: {e}", parent.display())))?;
        std::fs::write(&materialized, &blob.stdout).map_err(|e| {
            Error::Io(format!(
                "write exported path {}: {e}",
                materialized.display()
            ))
        })?;
    }
    Ok(())
}

fn resolve_object(repo: &Path, rev: &str, kind: &str) -> Result<String, Error> {
    let suffix = if kind == "commit" { "commit" } else { "tree" };
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--verify", "--quiet", "--end-of-options"])
        .arg(format!("{rev}^{{{suffix}}}"))
        .output()
        .map_err(|e| Error::Io(format!("spawn git rev-parse: {e}")))?;
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !out.status.success() || id.is_empty() {
        return Err(Error::Io(format!("git: '{rev}' does not name a {kind}")));
    }
    Ok(id)
}

fn require_clean(repo: &Path) -> Result<(), Error> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["status", "--porcelain=v1", "--untracked-files=normal"])
        .output()
        .map_err(|e| Error::Io(format!("spawn git status: {e}")))?;
    if !out.status.success() {
        return Err(Error::Io(format!(
            "git status exited {:?}",
            out.status.code()
        )));
    }
    if !out.stdout.is_empty() {
        return Err(Error::Refused(
            "dirty-tree: working tree or index is not clean".into(),
        ));
    }
    Ok(())
}

struct TempExport {
    root: PathBuf,
    checkout: PathBuf,
    tree: PathBuf,
}

impl TempExport {
    fn create() -> Result<Self, Error> {
        let parent = std::env::temp_dir();
        let seed = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        for attempt in 0..16 {
            let root = parent.join(format!("spec-spine-content-{seed}-{attempt}"));
            match std::fs::create_dir(&root) {
                Ok(()) => {
                    let checkout = root.join("checkout");
                    let tree = root.join("tree");
                    for dir in [&checkout, &tree] {
                        std::fs::create_dir(dir)
                            .map_err(|e| Error::Io(format!("create {}: {e}", dir.display())))?;
                    }
                    return Ok(Self {
                        root,
                        checkout,
                        tree,
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(Error::Io(format!("create {}: {e}", root.display()))),
            }
        }
        Err(Error::Io(
            "could not create content export directory".into(),
        ))
    }
}

impl Drop for TempExport {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
