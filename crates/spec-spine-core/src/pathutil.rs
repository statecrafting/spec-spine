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
/// state), and the `[index] resolver_exclusions` directory names (build output
/// and installed dependencies, which no governed read enters).
///
/// A link AT a derived or state root, or at an ancestor of one, is checked
/// like any other and never entered (spec 147 §3.1): read-only verbs read the
/// committed derived tree through it, and spec 127's refusal runs only on
/// write. A link there that stays inside the repository is 127's to judge.
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
            let rel = rel_posix(repo_root, &path);
            // A root, or an ancestor of one. A link there is checked like any
            // other and never entered (spec 147 §3.1): every read-only verb
            // reads the committed derived tree through it, and spec 127's
            // refusal runs only on write. One resolving inside the repository
            // stays 127's to judge, on write. Checked before the name skips,
            // because the default derived root is itself a resolver exclusion.
            let at_or_above_root = skip_rel
                .iter()
                .any(|s| *s == rel || s.strip_prefix(&rel).is_some_and(|r| r.starts_with('/')));
            // A plain directory above a root whose NAME is a resolver exclusion
            // (a root configured under `build/`, say) is still skipped, as it
            // was before spec 147: walking it would walk the whole build tree.
            // A link at that name is checked all the same.
            if !(meta.file_type().is_symlink() && at_or_above_root)
                && (name == ".git" || cfg.index.resolver_exclusions.contains(&name))
            {
                continue;
            }
            if meta.file_type().is_symlink() {
                if let Ok(target) = fs::canonicalize(&path)
                    && !target.starts_with(&root)
                {
                    return Err(Error::Refused(if at_or_above_root {
                        format!(
                            "refused to read the repository: '{rel}' is a link to {}, outside \
                             it, and the derived tree or a governed file beside it is read \
                             through that link (spec 147). It is refused on read as well as \
                             on write, because a verdict read through it would judge content \
                             the repository does not hold; nothing was written. \
                             Remove the link or point it inside the repository (a link there \
                             that stays inside is spec 127's to judge, on write)",
                            target.display()
                        )
                    } else {
                        format!(
                            "refused to read the repository: '{rel}' is a link to {}, outside \
                             it (spec 144). A governed read through it would judge content the \
                             repository does not hold; remove the link or point it inside the \
                             repository",
                            target.display()
                        )
                    }));
                }
                continue;
            }
            // The roots themselves are the tool's own output and state: not
            // walked.
            if skip_rel.contains(&rel) {
                continue;
            }
            if meta.is_dir() {
                stack.push(path);
            }
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
