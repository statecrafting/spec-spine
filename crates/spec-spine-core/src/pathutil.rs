//! Shared path helpers for the indexer: repo-relative POSIX paths and
//! exclusion matching against `index.resolver_exclusions`.

use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_types::{Config, Error};

/// Repo-relative POSIX path (forward slashes) of `path` under `repo_root`.
pub fn rel_posix(repo_root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(repo_root).unwrap_or(path);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// True if any component of `path` (relative to `repo_root`) is an excluded
/// directory name (e.g. `target`, `node_modules`).
pub fn is_excluded(repo_root: &Path, path: &Path, exclusions: &[String]) -> bool {
    let rel = path.strip_prefix(repo_root).unwrap_or(path);
    rel.components().any(|c| {
        let seg = c.as_os_str().to_string_lossy();
        exclusions.iter().any(|ex| ex == seg.as_ref())
    })
}

/// Refuse a repository whose tree holds a link that resolves outside it (spec
/// 144 §3.4).
///
/// Every governed read (the corpus, the claimed files, the hashed inputs, the
/// manifests, the source walks) reaches a file by walking down from the root,
/// so a read can leave the repository only through a link inside it. This
/// walks the tree once and checks every link: one whose target, fully
/// resolved, is not below the root's own resolution refuses the run, exit 2,
/// naming it. A link to another place inside the repository, and a dangling
/// link (which reads nothing), are fine. Links are checked, never followed, so
/// a directory link cannot loop the walk.
///
/// Not walked: `.git`, the derived and state roots (the tool's own output and
/// state, whose links spec 127 already refuses on write), and the
/// `[index] resolver_exclusions` directory names (build output and installed
/// dependencies, which no governed read enters).
///
/// The root is resolved first, so a repository reached through a link (a
/// symlinked checkout, macOS's `/tmp` alias) is compared with where it really
/// is and still works.
pub fn refuse_links_leaving(cfg: &Config, repo_root: &Path) -> Result<(), Error> {
    let Ok(root) = fs::canonicalize(repo_root) else {
        return Ok(()); // a missing root is the caller's error to report
    };
    let skip_rel: Vec<String> = [&cfg.layout.derived_dir, &cfg.layout.state_dir]
        .iter()
        .map(|d| d.trim_start_matches("./").trim_end_matches('/').to_string())
        .filter(|d| !d.is_empty() && d != ".")
        .collect();
    let mut stack: Vec<PathBuf> = vec![repo_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                if let Ok(target) = fs::canonicalize(&path)
                    && !target.starts_with(&root)
                {
                    return Err(Error::Refused(format!(
                        "refused to read the repository: '{}' is a link to {}, outside it \
                         (spec 144). A governed read through it would judge content the \
                         repository does not hold; remove the link or point it inside the \
                         repository",
                        rel_posix(repo_root, &path),
                        target.display()
                    )));
                }
                continue;
            }
            if !meta.is_dir() {
                continue;
            }
            let rel = rel_posix(repo_root, &path);
            if name == ".git"
                || cfg.index.resolver_exclusions.iter().any(|ex| *ex == name)
                || skip_rel.iter().any(|s| *s == rel)
            {
                continue;
            }
            stack.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rel_posix_uses_forward_slashes() {
        let root = PathBuf::from("/repo");
        assert_eq!(rel_posix(&root, &root.join("a").join("b.rs")), "a/b.rs");
    }

    #[test]
    fn exclusion_matches_any_component() {
        let root = PathBuf::from("/repo");
        let ex = vec!["target".to_string(), "node_modules".to_string()];
        assert!(is_excluded(&root, &root.join("crates/x/target/debug"), &ex));
        assert!(is_excluded(&root, &root.join("web/node_modules/pkg"), &ex));
        assert!(!is_excluded(&root, &root.join("crates/x/src/lib.rs"), &ex));
    }
}
