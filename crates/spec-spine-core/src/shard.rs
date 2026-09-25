//! Shared shard-storage primitives (spec 022).
//!
//! The two committed artifacts (the spec registry and the codebase index) are
//! stored as one file per authority unit (`by-spec/<id>.json`,
//! `by-package/<slug>.json`) instead of one monolithic file behind a global
//! content-hash line. Two PRs that touch different units then write disjoint
//! files and never conflict textually, so GitHub's server-side merge and the
//! merge queue's speculative build form clean stacks (the spec 094 merge driver
//! is needed only for the rare same-shard conflict).
//!
//! This module holds the storage mechanics common to both artifacts:
//! directory synchronization (write current shards, prune removed ones), the
//! global-inputs hash folded into every shard, the aggregate content hash
//! recomputed from shard hashes on read, and the filesystem-safe package slug.
//! The per-artifact projection (record/mapping shapes, per-shard hash inputs)
//! lives with each producer: [`crate::compile`] and [`crate::index`].

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use spec_spine_types::{Config, Error, parse_semver};

use crate::hash;
use crate::pathutil::rel_posix;

/// Reject a shard whose schema MAJOR differs from this build's (the versioning
/// policy: a build understands its own MAJOR line only). Mirrors
/// `query::reject_unknown_major`, applied per shard at the read boundary so a
/// stale-major shard fails with a clean [`Error::Schema`] (exit 4) rather than a
/// silent misread.
pub fn check_major(what: &str, found: &str, ours: &str) -> Result<(), Error> {
    let (want_major, ..) = parse_semver(ours).expect("our own version constant is semver");
    let (got_major, ..) = parse_semver(found)
        .ok_or_else(|| Error::Schema(format!("{what} schemaVersion '{found}' is not semver")))?;
    if got_major != want_major {
        return Err(Error::Schema(format!(
            "{what} schema MAJOR {got_major} is unsupported (this build understands {want_major}.x)"
        )));
    }
    Ok(())
}

/// A set of shard files to write into one directory: `(filename, content)`,
/// each content already canonical JSON.
pub type ShardFiles = Vec<(String, String)>;

/// The shard subdirectory holding per-spec shards under an artifact's dir.
pub const BY_SPEC_DIR: &str = "by-spec";
/// The shard subdirectory holding per-package shards under an artifact's dir.
pub const BY_PACKAGE_DIR: &str = "by-package";

/// The global inputs as hash pieces, `(repo-relative POSIX path, piece)`, sorted
/// and deduplicated by path: `spec-spine.toml` and every
/// `index.extra_hashed_inputs` match outside a declared state root. A workflow
/// contributes its governance projection rather than its raw bytes (spec 060
/// 3.1). Pure function of `(config, file contents)`.
///
/// Spec 141 moved these out of every shard's `shardHash`: they are recorded one
/// entry per file in the `codebase-index/inputs.json` sidecar
/// ([`input_digests`]) and folded into the aggregate content hash, so a
/// governance edit rewrites one committed file rather than every shard.
pub fn global_input_pieces(cfg: &Config, repo_root: &Path) -> Vec<(String, String)> {
    let mut pieces: Vec<(String, String)> = Vec::new();
    let cfg_path = repo_root.join("spec-spine.toml");
    if let Ok(content) = fs::read_to_string(&cfg_path) {
        pieces.push((rel_posix(repo_root, &cfg_path), content));
    }
    for pattern in &cfg.index.extra_hashed_inputs {
        for file in glob_files(repo_root, pattern) {
            let rel = rel_posix(repo_root, &file);
            // Spec 036 3.2: nothing under a declared state root contributes to
            // any content hash, so a tool writing its own state can never make
            // the committed ledger stale. Filtered here rather than left to the
            // adopter's glob, because a pattern wide enough to reach in (`**`,
            // or a shared parent directory) is the ordinary case, and the point
            // of the key is that the adopter states the root once.
            if cfg.layout.is_state_path(&rel) {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&file) {
                // Spec 060 3.1: a workflow folds as its governance projection,
                // not as raw bytes, so a one-character action-ref bump leaves
                // the ledger fresh and the bot that made it is not walled.
                // Unparseable falls back to raw bytes, as the npm and cargo
                // projections do.
                let piece = if crate::dep_only::is_workflow_yaml(&rel) {
                    crate::manifest::workflow_hash_projection(&content).unwrap_or(content)
                } else {
                    content
                };
                pieces.push((rel, piece));
            }
        }
    }
    pieces.sort_by(|a, b| a.0.cmp(&b.0));
    pieces.dedup_by(|a, b| a.0 == b.0);
    pieces
}

/// One content hash over every global input (spec 022's scalar). Since spec 141
/// no shard folds it; it remains the single-value answer to "did any governance
/// input change", with the same construction as before.
pub fn global_inputs_hash(cfg: &Config, repo_root: &Path) -> String {
    hash::content_hash(global_input_pieces(cfg, repo_root))
}

/// Each global input's own digest, keyed by path (spec 141 3.2): the
/// `contentHash` of the one-piece set `(path, piece)`, the same construction a
/// shard hash uses, so an input is hashed the same way wherever it is hashed.
pub fn input_digests(cfg: &Config, repo_root: &Path) -> BTreeMap<String, String> {
    global_input_pieces(cfg, repo_root)
        .into_iter()
        .map(|(path, piece)| {
            let digest = hash::content_hash(vec![(path.clone(), piece)]);
            (path, digest)
        })
        .collect()
}

/// The value the aggregate index content hash folds for the inputs sidecar
/// (spec 141 3.4): a content hash over the sorted `(path, contentHash)` pairs.
/// Computable from the sidecar alone, so the aggregate assembled from the
/// committed tree equals the one computed at emit.
pub fn inputs_fold<'a>(digests: impl IntoIterator<Item = (&'a String, &'a String)>) -> String {
    hash::content_hash(
        digests
            .into_iter()
            .map(|(path, digest)| (path.clone(), digest.clone()))
            .collect(),
    )
}

/// The aggregate content hash recomputed from the shard set on read: SHA-256
/// over the sorted `(shardKey, shardHash)` pairs. It is a pure function of the
/// shard hashes (each of which is a pure function of that shard's inputs), so it
/// is identical whether computed at emit or assembled on read, and it is never
/// committed to a shared file. `keyed` entries use a stable, collision-free key
/// such as `"spec:<id>"` / `"package:<slug>"`.
pub fn aggregate_content_hash(keyed: &[(String, String)]) -> String {
    hash::content_hash(keyed.to_vec())
}

/// A filesystem-safe slug for a package-shard filename, derived from the package
/// name: any character outside `[A-Za-z0-9._-]` becomes `_` (so a scoped npm
/// name like `@scope/pkg` yields `_scope_pkg`). Leading dots are escaped so a
/// shard is never a hidden file, and so is a Windows device-name stem, so a
/// package named `aux` is never the device (spec 127 3.6).
pub fn package_slug(name: &str) -> String {
    let mut slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    // Spec 127 3.6: a device-name stem (`aux`, `con.x`) is escaped the same
    // way, so a slug stays a plain file name on every platform.
    if slug.starts_with('.') || is_device_name(&slug) {
        slug.insert(0, '_');
    }
    if slug.is_empty() {
        slug.push('_');
    }
    slug
}

/// Refuse a file name that is not one plain file name before it is joined to
/// `dir` (spec 126 3.1, widened by spec 127 3.5). A per-spec shard or
/// attestation is named after the spec's frontmatter `id`, which the corpus
/// chooses, so an id such as `../../../package` or an absolute path would
/// otherwise write wherever it points. Plain means: not empty, no leading `.`,
/// no trailing `.` or space, no `/`, `\`, `:` or NUL, exactly one ordinary path
/// component, and not a reserved Windows device name ([`is_device_name`]). The
/// separators, the colon and the device names are refused on every platform,
/// so a corpus gets the same verdict on every release triple.
///
/// This is a check on the name, not on the id grammar: `V-012` is unchanged,
/// and an id that fails it but is a plain name (`001-Foo`) passes here. It is
/// lexical; the links a path passes through are [`DerivedWrites`]' check.
pub fn check_file_name(name: &str, dir: &Path) -> Result<(), Error> {
    let mut components = Path::new(name).components();
    let one_component = matches!(components.next(), Some(Component::Normal(c)) if c == name)
        && components.next().is_none();
    let plain = !name.is_empty()
        && !name.starts_with('.')
        && !name.ends_with(['.', ' '])
        && !name.contains(['/', '\\', ':', '\0'])
        && one_component
        && !is_device_name(name);
    if plain {
        return Ok(());
    }
    Err(Error::Refused(format!(
        "refused to write {name:?} into {}: a file name derived from a spec id must be one \
         plain file name (not empty, no leading `.`, no trailing `.` or space, no `/`, `\\`, \
         `:` or NUL, and not a reserved Windows device name such as `CON` or `NUL.txt`; \
         specs 126 and 127); nothing was written. Run `spec-spine compile --check` to see \
         the spec's id violation without writing",
        dir.display()
    )))
}

/// Whether `name` is a reserved Windows device name (spec 127 3.5): its stem,
/// the text before the first `.` with trailing spaces removed, is `CON`, `PRN`,
/// `AUX`, `NUL`, `COM1`..`COM9`, `LPT1`..`LPT9` or a superscript `COM¹²³` /
/// `LPT¹²³`, ignoring ASCII case. Windows treats `NUL.tar.gz` as `NUL`, hence
/// the first dot, and strips trailing spaces, hence the trim.
pub fn is_device_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).trim_end_matches(' ');
    if ["CON", "PRN", "AUX", "NUL"]
        .iter()
        .any(|d| stem.eq_ignore_ascii_case(d))
    {
        return true;
    }
    let mut chars = stem.chars();
    let prefix: String = chars.by_ref().take(3).collect();
    let (digit, rest) = (chars.next(), chars.next());
    (prefix.eq_ignore_ascii_case("COM") || prefix.eq_ignore_ascii_case("LPT"))
        && rest.is_none()
        && digit.is_some_and(|d| matches!(d, '1'..='9' | '\u{b9}' | '\u{b2}' | '\u{b3}'))
}

/// Write `files` (`(filename, content)`) into `dir`, creating it, and prune any
/// `*.json` already there whose name is not in `files`. This is what makes a
/// removed spec/package delete its shard: emit is a directory *sync*, not a
/// blind write, so the shard set always equals the current authority set.
///
/// Every name is checked with [`check_file_name`] before anything happens, so
/// one unsafe name refuses the whole batch with `dir` neither created, pruned
/// nor written (spec 126 3.2). It is [`DerivedWrites`] rooted at `dir`'s parent
/// (spec 127 D-3): `dir` and every entry it touches are checked for links, and
/// nothing above `dir` is. A writer into a repository uses [`DerivedWrites`]
/// with the repository root, which is what the CLI does.
pub fn sync_dir(dir: &Path, files: &[(String, String)]) -> Result<(), Error> {
    let root = dir.parent().unwrap_or(dir);
    DerivedWrites::new(root)
        .sync_dir(dir, files.to_vec())
        .apply()
}

/// One output of a [`DerivedWrites`] run.
enum Op {
    /// Write `files` into `dir` and prune every other `*.json` there.
    Sync { dir: PathBuf, files: ShardFiles },
    /// Write one file.
    Write {
        dir: PathBuf,
        name: String,
        content: String,
    },
    /// Remove one file, if it is there.
    Remove { dir: PathBuf, name: String },
}

/// What a checked path component is expected to be.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dir,
    File,
}

impl Kind {
    fn noun(self) -> &'static str {
        match self {
            Kind::Dir => "a directory",
            Kind::File => "a file",
        }
    }
}

/// Every output one verb writes into the derived tree, checked as a whole
/// before any of it happens (spec 127).
///
/// [`apply`](Self::apply) first checks every name against
/// [`check_file_name`], that every path is wholly below `root` (spec 128 3.3),
/// and every component of every path below `root` that
/// the run writes, creates, removes or prunes: a symbolic link, or an existing
/// component of the wrong kind, refuses the whole run with exit 2 and nothing
/// written (3.1 to 3.4). Only then does it create directories, one level at a
/// time and re-checked as it goes, prune, write and remove, in the order the
/// outputs were added. `root` itself and its ancestors are never checked, so a
/// repository reached through a link still works (3.1). Another process
/// changing the tree while this runs is outside the claim (4).
pub struct DerivedWrites {
    root: PathBuf,
    ops: Vec<Op>,
}

impl DerivedWrites {
    /// A run writing below `root`, the repository root.
    pub fn new(root: &Path) -> Self {
        DerivedWrites {
            root: root.to_path_buf(),
            ops: Vec::new(),
        }
    }

    /// Synchronize `dir` to exactly `files`, as [`sync_dir`] does.
    #[must_use]
    pub fn sync_dir(mut self, dir: &Path, files: ShardFiles) -> Self {
        self.ops.push(Op::Sync {
            dir: dir.to_path_buf(),
            files,
        });
        self
    }

    /// Write `content` to `dir/name`, creating `dir`.
    #[must_use]
    pub fn write(mut self, dir: &Path, name: &str, content: String) -> Self {
        self.ops.push(Op::Write {
            dir: dir.to_path_buf(),
            name: name.to_string(),
            content,
        });
        self
    }

    /// Remove `dir/name` if it exists (a legacy or no-longer-configured file).
    #[must_use]
    pub fn remove(mut self, dir: &Path, name: &str) -> Self {
        self.ops.push(Op::Remove {
            dir: dir.to_path_buf(),
            name: name.to_string(),
        });
        self
    }

    /// Check the whole run, then perform it.
    pub fn apply(self) -> Result<(), Error> {
        self.preflight()?;
        for op in &self.ops {
            match op {
                Op::Sync { dir, files } => {
                    self.create_dirs(dir)?;
                    for path in prune_entries(dir, files) {
                        fs::remove_file(&path)
                            .map_err(|e| Error::Io(format!("prune {}: {e}", path.display())))?;
                    }
                    for (name, content) in files {
                        let path = dir.join(name);
                        fs::write(&path, content)
                            .map_err(|e| Error::Io(format!("write {}: {e}", path.display())))?;
                    }
                }
                Op::Write { dir, name, content } => {
                    self.create_dirs(dir)?;
                    let path = dir.join(name);
                    fs::write(&path, content)
                        .map_err(|e| Error::Io(format!("write {}: {e}", path.display())))?;
                }
                Op::Remove { dir, name } => {
                    let path = dir.join(name);
                    if fs::symlink_metadata(&path).is_ok() {
                        fs::remove_file(&path)
                            .map_err(|e| Error::Io(format!("remove {}: {e}", path.display())))?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Every check, before any mutation. Names first, so 126's refusal keeps
    /// its message whatever else is wrong with the tree.
    fn preflight(&self) -> Result<(), Error> {
        for op in &self.ops {
            match op {
                Op::Sync { dir, files } => {
                    for (name, _) in files {
                        check_file_name(name, dir)?;
                    }
                }
                Op::Write { dir, name, .. } | Op::Remove { dir, name } => {
                    check_file_name(name, dir)?;
                }
            }
        }
        for op in &self.ops {
            match op {
                Op::Sync { dir, .. } => self.check_below_root(dir)?,
                Op::Write { dir, name, .. } | Op::Remove { dir, name } => {
                    self.check_below_root(&dir.join(name))?;
                }
            }
        }
        for op in &self.ops {
            match op {
                Op::Sync { dir, files } => {
                    self.check_path(dir, Kind::Dir)?;
                    for (name, _) in files {
                        self.check_path(&dir.join(name), Kind::File)?;
                    }
                    for path in prune_entries(dir, files) {
                        self.check_path(&path, Kind::File)?;
                    }
                }
                Op::Write { dir, name, .. } | Op::Remove { dir, name } => {
                    self.check_path(&dir.join(name), Kind::File)?;
                }
            }
        }
        Ok(())
    }

    /// Refuse a path that is not wholly below the root: one that does not
    /// start with it, or whose remainder has a `..`, a root or a prefix
    /// component (spec 128 3.3, withdrawing 127 D-4). Lexical, so the root is
    /// never resolved and a root reached through a link still works. A
    /// synced directory's prune entries are read from inside it, so checking
    /// the directory covers them.
    fn check_below_root(&self, path: &Path) -> Result<(), Error> {
        let below = path.strip_prefix(&self.root).is_ok_and(|rel| {
            rel.components()
                .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
        });
        if below {
            return Ok(());
        }
        Err(Error::Refused(format!(
            "refused to write {}: it is not below {}, the directory this run writes into \
             (spec 128); nothing was written. A derived path must stay inside the \
             repository: check `[layout] derived_dir`",
            path.display(),
            self.root.display()
        )))
    }

    /// The ordinary components of `path` below the root. The preflight has
    /// already refused a path that is not wholly below it (spec 128 3.3), so
    /// the walk stopping early is a guard, not a case.
    fn below_root<'p>(&self, path: &'p Path) -> Vec<&'p std::ffi::OsStr> {
        let Ok(rel) = path.strip_prefix(&self.root) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for c in rel.components() {
            match c {
                Component::CurDir => {}
                Component::Normal(n) => out.push(n),
                _ => break,
            }
        }
        out
    }

    /// Refuse a link, or an existing component of the wrong kind, anywhere
    /// from the root down to `path` (3.1, 3.2, 3.4). A missing or unreadable
    /// component ends the walk (D-5).
    fn check_path(&self, path: &Path, last: Kind) -> Result<(), Error> {
        let parts = self.below_root(path);
        let mut cur = self.root.clone();
        for (i, part) in parts.iter().enumerate() {
            cur.push(part);
            let want = if i + 1 == parts.len() {
                last
            } else {
                Kind::Dir
            };
            let Ok(md) = fs::symlink_metadata(&cur) else {
                return Ok(());
            };
            let found = md.file_type();
            if found.is_symlink() {
                return Err(self.refusal(&cur, want, "is a symbolic link"));
            }
            if (want == Kind::Dir) != found.is_dir() {
                return Err(self.refusal(&cur, want, "exists and is not one"));
            }
        }
        Ok(())
    }

    fn refusal(&self, at: &Path, want: Kind, what: &str) -> Error {
        let rel = rel_posix(&self.root, at);
        Error::Refused(format!(
            "refused to write under {}: `{rel}` {what}, where the derived tree expects {} \
             (spec 127); nothing was written. The derived tree must contain no symbolic links \
             and no component of the wrong kind: remove `{rel}` and run again",
            self.root.display(),
            want.noun()
        ))
    }

    /// Create `dir` below the root one level at a time, re-checking each level
    /// as it is reached so none is created through a link (3.4). The re-check
    /// repeats the preflight on purpose: it is the only check between the
    /// preflight and a `create_dir`, so do not fold it away. The root is
    /// created with its ancestors if missing, which only [`sync_dir`]'s root can
    /// be. The final `create_dir_all` finds every level already made: a path
    /// not wholly below the root never reaches here (spec 128 3.3).
    fn create_dirs(&self, dir: &Path) -> Result<(), Error> {
        let io = |p: &Path, e: std::io::Error| Error::Io(format!("create {}: {e}", p.display()));
        fs::create_dir_all(&self.root).map_err(|e| io(&self.root, e))?;
        let mut cur = self.root.clone();
        for part in self.below_root(dir) {
            cur.push(part);
            match fs::symlink_metadata(&cur) {
                Ok(md) if md.is_dir() => continue,
                Ok(_) => return Err(changed(&cur)),
                Err(_) => match fs::create_dir(&cur) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                        if !fs::symlink_metadata(&cur).is_ok_and(|md| md.is_dir()) {
                            return Err(changed(&cur));
                        }
                    }
                    Err(e) => return Err(io(&cur, e)),
                },
            }
        }
        fs::create_dir_all(dir).map_err(|e| io(dir, e))
    }
}

/// A directory the preflight passed is no longer one: another process changed
/// the tree during the run, which is outside spec 127's claim (4). Unlike a
/// preflight refusal, earlier outputs of this run may already be written.
fn changed(at: &Path) -> Error {
    Error::Io(format!(
        "stopped at {}: it was a directory when the run was checked and is not one now; \
         the tree changed while this ran (spec 127 4), and earlier outputs of this run may \
         already be written",
        at.display()
    ))
}

/// The `*.json` entries in `dir` a sync to `files` would prune, sorted. A
/// missing or unreadable directory has none.
fn prune_entries(dir: &Path, files: &[(String, String)]) -> Vec<PathBuf> {
    let keep: BTreeSet<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            let is_json = path.extension().and_then(|e| e.to_str()) == Some("json");
            let name = path.file_name().and_then(|n| n.to_str());
            is_json && name.is_some_and(|n| !keep.contains(n))
        })
        .collect();
    out.sort();
    out
}

/// Read every `*.json` file in `dir`, sorted by filename, as raw bytes. A
/// missing directory yields an empty list (an artifact with no shards of that
/// kind, e.g. a corpus with no packages, or a not-yet-built artifact).
///
/// A directory that exists and cannot be read is `Error::Io` (spec 132 §3.2):
/// reading it as empty reported every committed shard as missing, so a read
/// the tool could not perform arrived as a stale ledger and a prescription to
/// regenerate it.
pub fn read_shard_files(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, Error> {
    let mut out: Vec<(String, Vec<u8>)> = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(Error::Io(format!("read {}: {e}", dir.display()))),
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    paths.sort();
    for path in paths {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let bytes =
            fs::read(&path).map_err(|e| Error::Io(format!("read {}: {e}", path.display())))?;
        out.push((name, bytes));
    }
    Ok(out)
}

/// Glob `pattern` under `repo_root`, returning matched files, sorted.
///
/// Crate-visible since spec 071: `delta` classifies a path as policy exactly
/// when [`global_inputs_hash`] folds it, and asks with this matcher rather than
/// a pattern test over the path, which would disagree on a pattern ending in a
/// bare `**` (it walks to directories, so it folds no file; spec 058).
pub(crate) fn glob_files(repo_root: &Path, pattern: &str) -> Vec<PathBuf> {
    let joined = repo_root.join(pattern);
    let mut out: Vec<PathBuf> = match glob::glob(&joined.to_string_lossy()) {
        Ok(paths) => paths
            .filter_map(std::result::Result::ok)
            .filter(|p| p.is_file())
            .collect(),
        Err(_) => Vec::new(),
    };
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_slug_is_filesystem_safe() {
        assert_eq!(package_slug("spec-spine-cli"), "spec-spine-cli");
        assert_eq!(package_slug("@scope/pkg"), "_scope_pkg");
        assert_eq!(package_slug(".hidden"), "_.hidden");
    }

    /// Spec 126 3.2: a package shard's name is a slug, so it cannot reach the
    /// refusal. `index` still checks both batches before writing either (D-5);
    /// this is the assertion that the by-package half of that check is
    /// unreachable today, and fails if the slug ever stops being plain.
    #[test]
    fn a_package_slug_is_always_a_plain_file_name() {
        let dir = Path::new("by-package");
        for name in [
            "",
            ".",
            "..",
            "../../x",
            "/abs",
            "a\\b",
            "C:evil",
            "a:b",
            "a\0b",
            ".hidden",
            "@scope/pkg",
            "é",
            // Spec 127 3.6: device names, with extensions, in any case.
            "aux",
            "CON",
            "nul.tar",
            "Com1",
            "lpt9.x",
            "prn.",
            "con .x",
        ] {
            let file = format!("{}.json", package_slug(name));
            assert!(check_file_name(&file, dir).is_ok(), "{name:?} -> {file:?}");
        }
    }

    #[test]
    fn sync_dir_prunes_removed_shards() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("by-spec");
        sync_dir(
            &dir,
            &[("a.json".into(), "1".into()), ("b.json".into(), "2".into())],
        )
        .unwrap();
        sync_dir(&dir, &[("a.json".into(), "1".into())]).unwrap();
        assert!(dir.join("a.json").is_file());
        assert!(!dir.join("b.json").is_file(), "removed shard is pruned");
    }

    #[test]
    fn check_file_name_accepts_exactly_one_plain_name() {
        let dir = Path::new("by-spec");
        // Spec 126 3.1: `V-012` is not this rule, so an id failing the grammar
        // but plain as a name (`001-Foo`) is accepted.
        for ok in ["001-a.json", "001-Foo.json", "a.b.json", "_scope_pkg.json"] {
            assert!(check_file_name(ok, dir).is_ok(), "{ok:?} is plain");
        }
        for bad in [
            "",
            ".",
            "..",
            ".json",
            ".hidden.json",
            "...json",
            "a/b.json",
            "../x.json",
            "../../../package.json",
            "/abs.json",
            "a\\b.json",
            "..\\x.json",
            "C:x.json",
            "a:b.json",
            "a\0b.json",
        ] {
            let err = check_file_name(bad, dir).expect_err(bad);
            assert_eq!(err.exit_code(), 2, "{bad:?}");
            let msg = err.to_string();
            assert!(msg.contains(&format!("{bad:?}")), "names the name: {msg}");
            assert!(msg.contains("by-spec"), "names the directory: {msg}");
            assert!(msg.contains("nothing was written"), "{msg}");
            assert!(msg.contains("spec-spine compile --check"), "{msg}");
        }
    }

    /// Spec 126 3.2: safe names ahead of an unsafe one are not written, the
    /// shard the batch would have pruned survives, and nothing lands outside.
    #[test]
    fn sync_dir_refuses_the_whole_batch_before_touching_anything() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("a").join("b").join("by-spec");
        sync_dir(
            &dir,
            &[
                ("old.json".into(), "old".into()),
                ("a.json".into(), "1".into()),
            ],
        )
        .unwrap();
        fs::write(tmp.path().join("victim.json"), "victim").unwrap();
        let batch: ShardFiles = vec![
            ("a.json".into(), "changed".into()),
            ("new.json".into(), "new".into()),
            ("../../../victim.json".into(), "pwned".into()),
        ];
        let err = sync_dir(&dir, &batch).expect_err("an unsafe name refuses");
        assert_eq!(err.exit_code(), 2);
        assert_eq!(fs::read_to_string(dir.join("old.json")).unwrap(), "old");
        assert_eq!(fs::read_to_string(dir.join("a.json")).unwrap(), "1");
        assert!(!dir.join("new.json").exists());
        assert_eq!(
            fs::read_to_string(tmp.path().join("victim.json")).unwrap(),
            "victim"
        );
        let mut names: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        names.sort();
        assert_eq!(names, ["a.json", "old.json"]);

        // A directory that does not exist yet is not created.
        let fresh = tmp.path().join("fresh").join("by-spec");
        sync_dir(
            &fresh,
            &[("a.json".into(), "1".into()), (String::new(), "x".into())],
        )
        .expect_err("an empty name refuses");
        assert!(!tmp.path().join("fresh").exists());
    }

    /// Spec 127 3.5, platform-independent: every reserved device stem refuses
    /// in three cases, bare, with one extension and with two, and with the
    /// trailing spaces and dots Windows strips; the near misses stay plain.
    #[test]
    fn a_device_name_is_not_a_plain_file_name() {
        let dir = Path::new("by-spec");
        let mut stems: Vec<String> = ["CON", "PRN", "AUX", "NUL"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        for port in ["COM", "LPT"] {
            for d in ['1', '2', '3', '4', '5', '6', '7', '8', '9', '¹', '²', '³'] {
                stems.push(format!("{port}{d}"));
            }
        }
        assert_eq!(stems.len(), 4 + 2 * 12);
        for stem in &stems {
            let cases = [stem.clone(), stem.to_ascii_lowercase(), {
                // Mixed case: every other letter lowered.
                stem.chars()
                    .enumerate()
                    .map(|(i, c)| {
                        if i % 2 == 1 {
                            c.to_ascii_lowercase()
                        } else {
                            c
                        }
                    })
                    .collect()
            }];
            for form in cases {
                for name in [
                    format!("{form}.json"),
                    format!("{form}.sig"),
                    format!("{form}.tar.json"),
                    format!("{form} .json"),
                    format!("{form}  .json"),
                    format!("{form}..json"),
                    format!("{form}. .json"),
                ] {
                    assert!(is_device_name(&name), "{name:?} is a device");
                    let err = check_file_name(&name, dir).expect_err(&name);
                    assert_eq!(err.exit_code(), 2, "{name:?}");
                    let msg = err.to_string();
                    assert!(msg.contains(&format!("{name:?}")), "{msg}");
                    assert!(msg.contains("nothing was written"), "{msg}");
                    assert!(msg.contains("reserved Windows device name"), "{msg}");
                }
            }
        }
        for plain in [
            "CONSOLE.json",
            "COM0.json",
            "COM10.json",
            "LPT0.json",
            "NULL.json",
            "LPT.json",
            "COM.json",
            "xCON.json",
            "CON-1.json",
            "CON_.json",
            "001-con.json",
            "001-a.json",
            "_aux.json",
            "COM⁴.json",
            "a.con.json",
        ] {
            assert!(!is_device_name(plain), "{plain:?} is not a device");
            assert!(check_file_name(plain, dir).is_ok(), "{plain:?} is plain");
        }
    }

    /// Spec 127 3.5 (D-6): a whole name ending in a dot or a space is not
    /// plain, because Windows strips them and the name would be another one.
    #[test]
    fn a_trailing_dot_or_space_is_not_a_plain_file_name() {
        let dir = Path::new("by-spec");
        for bad in [
            "a.json.",
            "a.json ",
            "a.",
            "a ",
            "a. ",
            "a .",
            "001-a.json..",
        ] {
            let err = check_file_name(bad, dir).expect_err(bad);
            assert_eq!(err.exit_code(), 2, "{bad:?}");
        }
        for ok in ["a.json", "a b.json", "a .b.json"] {
            assert!(check_file_name(ok, dir).is_ok(), "{ok:?}");
        }
    }

    /// Spec 127 3.6: the escape only fires on a device stem, so every other
    /// slug is byte-for-byte what it was.
    #[test]
    fn the_slug_escape_touches_only_device_names() {
        assert_eq!(package_slug("aux"), "_aux");
        assert_eq!(package_slug("CON"), "_CON");
        assert_eq!(package_slug("nul.tar"), "_nul.tar");
        assert_eq!(package_slug("console"), "console");
        assert_eq!(package_slug("my-aux"), "my-aux");
        assert_eq!(package_slug("com10"), "com10");
    }

    /// Spec 127 3.2 to 3.4 at the library seam, on Unix where a test can make
    /// a link: a linked directory, a linked output file, a linked prune entry
    /// and a wrong-kind component each refuse the whole run with nothing
    /// changed, and the root itself may be reached through a link.
    #[cfg(unix)]
    #[test]
    fn derived_writes_refuse_a_link_below_the_root_and_change_nothing() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("real");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(real.join("d/by-spec")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(real.join("d/by-spec/old.json"), "old").unwrap();
        fs::write(outside.join("o.json"), "o").unwrap();
        // The root, reached through a link, is not checked.
        let root = tmp.path().join("root-link");
        symlink(&real, &root).unwrap();

        let run = |root: &Path| {
            DerivedWrites::new(root)
                .sync_dir(&root.join("d/by-spec"), vec![("a.json".into(), "1".into())])
                .write(&root.join("d"), "meta.json", "m".into())
                .remove(&root.join("d"), "legacy.json")
        };
        let listing = |p: &Path| {
            let mut v: Vec<_> = fs::read_dir(p)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            v.sort();
            v
        };

        for (at, target, is_dir) in [
            ("d", outside.clone(), true),
            ("d/meta.json", outside.join("o.json"), false),
            ("d/legacy.json", outside.join("o.json"), false),
            ("d/by-spec/a.json", outside.join("o.json"), false),
            ("d/by-spec/stray.json", outside.join("o.json"), false),
        ] {
            let path = real.join(at);
            let moved = tmp.path().join("moved");
            if is_dir {
                fs::rename(&path, &moved).unwrap();
            }
            symlink(&target, &path).unwrap();
            let err = run(&root).apply().expect_err(at);
            assert_eq!(err.exit_code(), 2, "{at}");
            let msg = err.to_string();
            assert!(msg.contains(&format!("`{at}`")), "{at}: {msg}");
            assert!(msg.contains("symbolic link"), "{at}: {msg}");
            assert!(msg.contains("nothing was written"), "{at}: {msg}");
            assert_eq!(listing(&outside), ["o.json"], "{at}: outside untouched");
            assert_eq!(fs::read_to_string(outside.join("o.json")).unwrap(), "o");
            fs::remove_file(&path).unwrap();
            if is_dir {
                fs::rename(&moved, &path).unwrap();
            }
            assert_eq!(
                listing(&real.join("d")),
                ["by-spec"],
                "{at}: nothing written"
            );
            assert_eq!(listing(&real.join("d/by-spec")), ["old.json"], "{at}");
        }

        // A wrong-kind component refuses in the same preflight.
        fs::create_dir(real.join("d/meta.json")).unwrap();
        let err = run(&root).apply().expect_err("a directory at a file");
        assert!(err.to_string().contains("exists and is not one"), "{err}");
        assert_eq!(listing(&real.join("d/by-spec")), ["old.json"]);
        fs::remove_dir(real.join("d/meta.json")).unwrap();

        // And the clean run, through the linked root, syncs, writes, prunes.
        fs::write(real.join("d/legacy.json"), "l").unwrap();
        run(&root).apply().unwrap();
        assert_eq!(listing(&real.join("d/by-spec")), ["a.json"]);
        assert_eq!(listing(&real.join("d")), ["by-spec", "meta.json"]);
    }

    /// Spec 128 3.3: a path that is not wholly below the root refuses the
    /// whole run in the preflight, wherever it sits in the run, with nothing
    /// written, created, removed or pruned inside the root or out of it.
    #[test]
    fn derived_writes_refuse_a_path_not_below_the_root_and_change_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("d")).unwrap();
        fs::create_dir_all(outside.join("by-spec")).unwrap();
        fs::write(outside.join("by-spec/o.json"), "o").unwrap();
        fs::write(outside.join("o.json"), "o").unwrap();
        let listing = |p: &Path| {
            let mut v: Vec<_> = fs::read_dir(p)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            v.sort();
            v
        };
        let shard = || vec![("a.json".to_string(), "1".to_string())];

        type Escape = Box<dyn Fn(DerivedWrites) -> DerivedWrites>;
        let escapes: Vec<(&str, Escape)> = vec![
            ("sync through ..", {
                let dir = root.join("../outside/by-spec");
                Box::new(move |w| w.sync_dir(&dir, shard()))
            }),
            ("sync to a path elsewhere", {
                let dir = outside.join("by-spec");
                Box::new(move |w| w.sync_dir(&dir, shard()))
            }),
            ("write through a mid-path ..", {
                let dir = root.join("d/../../outside");
                Box::new(move |w| w.write(&dir, "m.json", "m".into()))
            }),
            ("remove through ..", {
                let dir = root.join("..").join("outside");
                Box::new(move |w| w.remove(&dir, "o.json"))
            }),
        ];
        for (what, escape) in &escapes {
            // The escaping output comes after a legitimate one, so a run that
            // wrote before it checked would leave `d/meta.json` behind.
            let run =
                escape(DerivedWrites::new(&root).write(&root.join("d"), "meta.json", "m".into()));
            let err = run.apply().expect_err(what);
            assert_eq!(err.exit_code(), 2, "{what}");
            let msg = err.to_string();
            assert!(msg.contains("not below"), "{what}: {msg}");
            assert!(msg.contains("nothing was written"), "{what}: {msg}");
            assert_eq!(listing(&outside), ["by-spec", "o.json"], "{what}");
            assert_eq!(listing(&outside.join("by-spec")), ["o.json"], "{what}");
            assert_eq!(fs::read_to_string(outside.join("o.json")).unwrap(), "o");
            assert!(
                listing(&root.join("d")).is_empty(),
                "{what}: nothing written"
            );
        }

        // A `.` component is ignored, and the clean run still writes.
        DerivedWrites::new(&root)
            .write(&root.join("./d"), "meta.json", "m".into())
            .apply()
            .unwrap();
        assert_eq!(listing(&root.join("d")), ["meta.json"]);
    }

    /// Spec 127 D-3: `sync_dir` keeps its signature and checks `dir` and every
    /// entry it touches, rooted at `dir`'s parent.
    #[cfg(unix)]
    #[test]
    fn sync_dir_refuses_a_linked_directory_or_entry() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::tempdir().unwrap();
        let outside = tmp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("o.json"), "o").unwrap();
        let dir = tmp.path().join("by-spec");
        symlink(&outside, &dir).unwrap();
        let err = sync_dir(&dir, &[("a.json".into(), "1".into())]).expect_err("linked dir");
        assert_eq!(err.exit_code(), 2);
        fs::remove_file(&dir).unwrap();
        fs::create_dir(&dir).unwrap();
        symlink(outside.join("o.json"), dir.join("a.json")).unwrap();
        sync_dir(&dir, &[("a.json".into(), "1".into())]).expect_err("linked entry");
        let mut names: Vec<_> = fs::read_dir(&outside)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        names.sort();
        assert_eq!(names, ["o.json"]);
        assert_eq!(fs::read_to_string(outside.join("o.json")).unwrap(), "o");
    }
}
