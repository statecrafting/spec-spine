//! `spec-spine content select`: bind selected content to a verified Git tree.

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Subcommand;
use spec_spine_core::{Versioning, read_document, selected_content};
use spec_spine_types::{
    ContentDirtyState, ContentRequest, ContentSelector, ContentSnapshot, ContentSnapshotBinding,
    Error,
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
        #[arg(long)]
        json: bool,
    },
}

pub fn run(repo: &Path, action: &ContentAction) -> Result<u8, Error> {
    match action {
        ContentAction::Select {
            request,
            repository,
            revision,
            json,
        } => run_select(repo, request, repository, revision.as_deref(), *json),
    }
}

fn run_select(
    repo: &Path,
    request_path: &Path,
    repository: &str,
    revision: Option<&str>,
    _json: bool,
) -> Result<u8, Error> {
    let text = std::fs::read_to_string(request_path).map_err(|e| {
        Error::Io(format!(
            "read content request {}: {e}",
            request_path.display()
        ))
    })?;
    let request: ContentRequest = serde_json::from_str(&text)
        .map_err(|e| Error::Usage(format!("invalid content request: {e}")))?;

    let (response, _guard) = if let Some(rev) = revision {
        let commit = resolve_object(repo, rev, "commit")?;
        let tree = resolve_object(repo, &commit, "tree")?;
        let export = TempExport::create()?;
        export_tree(repo, &commit, &export.root.join("index"), &export.tree)?;
        let exported_tree = resolve_object(repo, &commit, "tree")?;
        if tree != exported_tree {
            return Err(Error::Io("exported revision tree identity changed".into()));
        }
        let cfg = load_repo_config(&export.tree)?;
        let snapshot = ContentSnapshot {
            repository: repository.to_string(),
            revision: commit,
            tree,
            dirty_state: ContentDirtyState::CleanExport,
            binding: ContentSnapshotBinding::CallerSupplied,
        };
        let response = selected_content(&cfg, &export.tree, &request, &snapshot)?;
        (response, Some(export))
    } else {
        require_clean(repo)?;
        refuse_ignored_explicit_paths(repo, &request)?;
        let commit = resolve_object(repo, "HEAD", "commit")?;
        let tree = resolve_object(repo, "HEAD", "tree")?;
        let cfg = load_repo_config(repo)?;
        let snapshot = ContentSnapshot {
            repository: repository.to_string(),
            revision: commit.clone(),
            tree: tree.clone(),
            dirty_state: ContentDirtyState::CleanWorkingTree,
            binding: ContentSnapshotBinding::CallerSupplied,
        };
        let response = selected_content(&cfg, repo, &request, &snapshot)?;
        require_clean(repo)?;
        if resolve_object(repo, "HEAD", "commit")? != commit
            || resolve_object(repo, "HEAD", "tree")? != tree
        {
            return Err(Error::Refused(
                "snapshot-changed: HEAD or its tree changed during selection".into(),
            ));
        }
        (response, None)
    };
    let document = read_document(&response, Versioning::Stamp)?;
    out!("{document}");
    Ok(0)
}

fn refuse_ignored_explicit_paths(repo: &Path, request: &ContentRequest) -> Result<(), Error> {
    for selector in &request.selectors {
        let path = match selector {
            ContentSelector::File { path, .. } => Some(path.as_str().to_string()),
            ContentSelector::DirectoryMember {
                directory, member, ..
            } => Some(format!(
                "{}/{}",
                directory.as_str().trim_end_matches('/'),
                member.as_str().trim_start_matches('/')
            )),
            _ => None,
        };
        let Some(path) = path else { continue };
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["check-ignore", "--quiet", "--"])
            .arg(&path)
            .status()
            .map_err(|e| Error::Io(format!("spawn git check-ignore: {e}")))?;
        match status.code() {
            Some(0) => {
                return Err(Error::Refused(format!(
                    "ignored-content: selector names ignored path '{path}'"
                )));
            }
            Some(1) => {}
            code => {
                return Err(Error::Io(format!(
                    "git check-ignore exited {code:?} for '{path}'"
                )));
            }
        }
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
                    let tree = root.join("tree");
                    std::fs::create_dir(&tree)
                        .map_err(|e| Error::Io(format!("create {}: {e}", tree.display())))?;
                    return Ok(Self { root, tree });
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
