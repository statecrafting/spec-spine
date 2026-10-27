//! Settling a stopped merge's derived-tree conflicts (spec 206).
//!
//! A conflict in the committed derived tree carries no information: the answer
//! is whatever the compiler and the indexer produce over the merged inputs.
//! This module is the planning half of `spec-spine resolve-derived`. It spawns
//! nothing and reads no git state (spec 206 3.1): the CLI reads the unmerged
//! entries, the working-tree state and the operation in progress from git and
//! hands them in as text, and this module classifies them, decides the
//! refusals, builds the one regeneration run and compares what a merge did to
//! the carried change. The CLI carries the plan out.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Serialize;
use spec_spine_types::{Config, Error};

use crate::compile::{CompileOutcome, registry_dir, registry_shard_files};
use crate::index::{IndexOutcome, index_dir, index_inputs_file, index_shard_files, slices_path};
use crate::pathutil::rel_posix;
use crate::shard::{BY_PACKAGE_DIR, BY_SPEC_DIR, DerivedWrites};

/// Which side of the derived root an unmerged path lies on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    /// Under the configured `[layout] derived_dir`.
    Derived,
    /// Anywhere else.
    Authored,
}

/// One unmerged path and its side (3.7 `unmerged`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UnmergedPath {
    pub path: String,
    pub side: Side,
}

/// Why the verb refuses (3.4), one per item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RefusalReason {
    /// 3.4 item 1: an authored path is unmerged.
    AuthoredUnmerged,
    /// 3.4 item 2: the compile would read what the commit will not hold.
    Unstaged,
    /// 3.4 item 3: a derived entry regeneration would not settle.
    UnsettledDerived,
    /// 3.4 item 4: no operation is recorded.
    NoOperation,
}

/// One refusal and every path it names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Refusal {
    pub reason: RefusalReason,
    pub paths: Vec<String>,
    pub message: String,
}

impl Refusal {
    /// A refusal with 3.4's standard message for `reason`.
    pub fn new(reason: RefusalReason, paths: Vec<String>) -> Self {
        let message = match reason {
            RefusalReason::AuthoredUnmerged => {
                "an authored path is unmerged; resolving it is the author's job, and this verb \
                 takes no side on authored content"
            }
            RefusalReason::Unstaged => {
                "the working tree holds paths the commit will not: untracked and not ignored, or \
                 differing from the index. The regenerated tree would describe them; stage, \
                 ignore or remove each one first"
            }
            RefusalReason::UnsettledDerived => {
                "an unmerged path under the derived root is neither written nor pruned by the \
                 compiler and the indexer, so regeneration would not settle it"
            }
            RefusalReason::NoOperation => {
                "no merge, rebase or cherry-pick is in progress (none of MERGE_HEAD, REBASE_HEAD \
                 or CHERRY_PICK_HEAD), so there is no carried change to compare; a conflicted \
                 `git stash pop` is such a stop"
            }
        };
        Refusal {
            reason,
            paths,
            message: message.to_string(),
        }
    }
}

/// What the state read before any write decides (3.3, and 3.4 items 1, 2, 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// Every unmerged path, sorted, each with its side.
    pub unmerged: Vec<UnmergedPath>,
    /// The paths outside the derived root that 3.4 item 2 names.
    pub unstaged: Vec<String>,
    /// True when no unmerged entry is under the derived root (3.3).
    pub nothing_to_resolve: bool,
    /// The refusals decided before regeneration, in 3.4's order.
    pub refusals: Vec<Refusal>,
}

impl Plan {
    /// The unmerged paths under the derived root.
    pub fn derived_unmerged(&self) -> Vec<String> {
        self.unmerged
            .iter()
            .filter(|u| u.side == Side::Derived)
            .map(|u| u.path.clone())
            .collect()
    }
}

/// The unmerged paths in `git ls-files --unmerged -z` output, sorted and
/// deduplicated (one path carries up to three stage entries).
pub fn parse_unmerged(listing: &str) -> Vec<String> {
    let paths: BTreeSet<String> = listing
        .split('\0')
        .filter_map(|entry| entry.split_once('\t').map(|(_, path)| path.to_string()))
        .filter(|path| !path.is_empty())
        .collect();
    paths.into_iter().collect()
}

/// The paths `git status --porcelain=v1 -z --untracked-files=all` reports as
/// untracked, or as differing between the working tree and the index, sorted.
///
/// Unmerged entries (`DD`, `AU`, `UD`, `UA`, `DU`, `AA`, `UU`) are left out:
/// they are the unmerged set, read separately. A rename or copy carries its
/// source as the next NUL-separated field, which is skipped.
pub fn parse_status(listing: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    let mut fields = listing.split('\0');
    while let Some(entry) = fields.next() {
        if entry.len() < 4 {
            continue;
        }
        let bytes = entry.as_bytes();
        let (x, y) = (bytes[0] as char, bytes[1] as char);
        let path = &entry[3..];
        if matches!(x, 'R' | 'C') {
            fields.next();
        }
        let unmerged = x == 'U' || y == 'U' || (x == 'A' && y == 'A') || (x == 'D' && y == 'D');
        if unmerged {
            continue;
        }
        if (x == '?' && y == '?') || y != ' ' {
            out.insert(path.to_string());
        }
    }
    out.into_iter().collect()
}

/// Classify the state read before any write and decide 3.3 and 3.4 items 1,
/// 2 and 4.
///
/// `unmerged` and `dirty` are repository-relative POSIX paths; `operation`
/// says whether one of the three heads of 3.2 item 3 is recorded.
pub fn plan(cfg: &Config, unmerged: &[String], dirty: &[String], operation: bool) -> Plan {
    let mut classified: Vec<UnmergedPath> = unmerged
        .iter()
        .map(|path| UnmergedPath {
            path: path.clone(),
            side: if cfg.layout.is_derived_path(path) {
                Side::Derived
            } else {
                Side::Authored
            },
        })
        .collect();
    classified.sort();
    classified.dedup();
    let mut unstaged: Vec<String> = dirty
        .iter()
        .filter(|path| !cfg.layout.is_derived_path(path))
        .cloned()
        .collect();
    unstaged.sort();
    unstaged.dedup();

    let nothing_to_resolve = !classified.iter().any(|u| u.side == Side::Derived);
    let mut refusals = Vec::new();
    if !nothing_to_resolve {
        let authored: Vec<String> = classified
            .iter()
            .filter(|u| u.side == Side::Authored)
            .map(|u| u.path.clone())
            .collect();
        if !authored.is_empty() {
            refusals.push(Refusal::new(RefusalReason::AuthoredUnmerged, authored));
        }
        if !unstaged.is_empty() {
            refusals.push(Refusal::new(RefusalReason::Unstaged, unstaged.clone()));
        }
        if !operation {
            refusals.push(Refusal::new(RefusalReason::NoOperation, Vec::new()));
        }
    }
    Plan {
        unmerged: classified,
        unstaged,
        nothing_to_resolve,
        refusals,
    }
}

/// The one regeneration run (3.5 step 1): exactly the shard outputs `compile`
/// and `index` write, with the same pruning, from one in-memory compile and
/// one in-memory index. `build-meta.json` is not among them: it carries the
/// wall clock and the derived root's own ignore rules exclude it (206 D-6).
pub fn regeneration(
    cfg: &Config,
    repo: &Path,
    compiled: &CompileOutcome,
    indexed: &IndexOutcome,
) -> Result<DerivedWrites, Error> {
    let registry = registry_dir(cfg, repo);
    let index = index_dir(cfg, repo);
    let (by_spec, by_package) = index_shard_files(&indexed.shards)?;
    let (inputs_name, inputs) = index_inputs_file(&indexed.shards)?;
    let run = DerivedWrites::new(repo)
        .sync_dir(
            &registry.join(BY_SPEC_DIR),
            registry_shard_files(&compiled.shards)?,
        )
        .remove(&registry, "registry.json")
        .sync_dir(&index.join(BY_SPEC_DIR), by_spec)
        .sync_dir(&index.join(BY_PACKAGE_DIR), by_package)
        .write(&index, &inputs_name, inputs);
    let slices = slices_path(cfg, repo);
    let slices_dir = slices.parent().unwrap_or(repo).to_path_buf();
    let slice_hashes = &indexed.index.build.slice_hashes;
    let run = if slice_hashes.is_empty() {
        run.remove(&slices_dir, "slices.json")
    } else {
        let json = serde_json::to_string_pretty(slice_hashes)
            .map_err(|e| Error::Internal(e.to_string()))?
            + "\n";
        run.write(&slices_dir, "slices.json", json)
    };
    Ok(run.remove(&index, "index.json"))
}

/// 3.4 item 3: the unmerged derived paths `run` neither writes nor prunes,
/// sorted. Decided from the run's plan, before anything is written.
pub fn unsettled(repo: &Path, derived_unmerged: &[String], run: &DerivedWrites) -> Vec<String> {
    let (written, removed) = run.planned();
    let touched: BTreeSet<String> = written
        .iter()
        .chain(removed.iter())
        .map(|p| rel_posix(repo, p))
        .collect();
    let mut out: Vec<String> = derived_unmerged
        .iter()
        .filter(|p| !touched.contains(*p))
        .cloned()
        .collect();
    out.sort();
    out
}

/// One staged conflict marker (3.6 item 1).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Marker {
    pub path: String,
    pub line: u64,
    pub text: String,
}

/// The hits in `git grep --cached -n --null` output: `path\0line\0text`.
pub fn parse_markers(listing: &str) -> Vec<Marker> {
    let mut out: Vec<Marker> = listing
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\0');
            let path = parts.next()?;
            let number = parts.next()?.parse().ok()?;
            let text = parts.next().unwrap_or_default();
            Some(Marker {
                path: path.to_string(),
                line: number,
                text: text.to_string(),
            })
        })
        .collect();
    out.sort();
    out
}

/// A change reduced to the lines it adds and removes (3.6 item 2): one entry
/// per line, `(file, "+text")` or `(file, "-text")`, positions and context
/// dropped, sorted.
///
/// Reads a unified diff of `--unified=0`, `--no-renames`, `a/` and `b/`
/// prefixes. Hunk lengths are tracked, so a removed line whose own text begins
/// `-- ` is never read as a file header. Binary changes, mode changes and
/// `\ No newline` notes carry no line and are not listed.
pub fn change_lines(patch: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut old_path = String::new();
    let mut file = String::new();
    let (mut old_left, mut new_left) = (0u64, 0u64);
    for line in patch.lines() {
        if old_left > 0 || new_left > 0 {
            if let Some(text) = line.strip_prefix('-') {
                old_left = old_left.saturating_sub(1);
                out.push((file.clone(), format!("-{text}")));
                continue;
            }
            if let Some(text) = line.strip_prefix('+') {
                new_left = new_left.saturating_sub(1);
                out.push((file.clone(), format!("+{text}")));
                continue;
            }
            if line.starts_with(' ') {
                old_left = old_left.saturating_sub(1);
                new_left = new_left.saturating_sub(1);
                continue;
            }
            if line.starts_with('\\') {
                continue;
            }
            // Anything else ends the hunk early; fall through to the headers.
            old_left = 0;
            new_left = 0;
        }
        if line.starts_with("diff --git ") {
            old_path.clear();
            file.clear();
        } else if let Some(path) = line.strip_prefix("--- ") {
            old_path = path.strip_prefix("a/").unwrap_or(path).to_string();
            file = old_path.clone();
        } else if let Some(path) = line.strip_prefix("+++ ") {
            file = if path == "/dev/null" {
                old_path.clone()
            } else {
                path.strip_prefix("b/").unwrap_or(path).to_string()
            };
        } else if line.starts_with("@@ ") {
            let (o, n) = hunk_lengths(line);
            old_left = o;
            new_left = n;
        }
    }
    out.sort();
    out
}

/// `@@ -a[,b] +c[,d] @@` to `(b, d)`; an omitted length is 1.
fn hunk_lengths(header: &str) -> (u64, u64) {
    let mut words = header.split_whitespace().skip(1);
    let length = |word: Option<&str>| -> u64 {
        match word.and_then(|w| w.get(1..)) {
            Some(range) => match range.split_once(',') {
                Some((_, n)) => n.parse().unwrap_or(0),
                None => 1,
            },
            None => 0,
        }
    };
    let old = length(words.next());
    let new = length(words.next());
    (old, new)
}

/// One file whose added and removed lines differ between the carried change
/// and what the merge staged (3.6 item 2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDifference {
    pub path: String,
    /// Lines the carried change adds or removes that the merge's result does
    /// not: carried work the merge lost.
    pub only_in_carried: Vec<String>,
    /// Lines the merge's result adds or removes that the carried change does
    /// not.
    pub only_in_merged: Vec<String>,
}

/// 3.6 item 2's verdict.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoredChange {
    pub preserved: bool,
    pub differences: Vec<FileDifference>,
}

/// Compare the carried change with the index's difference from the receiving
/// side, both as [`change_lines`] lists: equal lists are a preserved change.
/// Under a difference, each file names the lines present in one list only,
/// counted as multisets so a line repeated is compared by its count.
pub fn compare_changes(
    carried: &[(String, String)],
    merged: &[(String, String)],
) -> AuthoredChange {
    let count = |lines: &[(String, String)]| {
        let mut m: BTreeMap<(String, String), i64> = BTreeMap::new();
        for l in lines {
            *m.entry(l.clone()).or_default() += 1;
        }
        m
    };
    let c = count(carried);
    let m = count(merged);
    let keys: BTreeSet<&(String, String)> = c.keys().chain(m.keys()).collect();
    let mut by_file: BTreeMap<String, FileDifference> = BTreeMap::new();
    for key in keys {
        let delta = c.get(key).copied().unwrap_or(0) - m.get(key).copied().unwrap_or(0);
        if delta == 0 {
            continue;
        }
        let entry = by_file
            .entry(key.0.clone())
            .or_insert_with(|| FileDifference {
                path: key.0.clone(),
                only_in_carried: Vec::new(),
                only_in_merged: Vec::new(),
            });
        let target = if delta > 0 {
            &mut entry.only_in_carried
        } else {
            &mut entry.only_in_merged
        };
        for _ in 0..delta.unsigned_abs() {
            target.push(key.1.clone());
        }
    }
    let differences: Vec<FileDifference> = by_file.into_values().collect();
    AuthoredChange {
        preserved: differences.is_empty(),
        differences,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config::default()
    }

    #[test]
    fn status_names_untracked_and_unstaged_and_skips_unmerged_and_staged() {
        let listing = "?? new.txt\0 M edited.rs\0M  staged.rs\0UU .derived/x.json\0\
                       R  to.rs\0from.rs\0AA both.rs\0 D gone.rs\0";
        assert_eq!(
            parse_status(listing),
            vec!["edited.rs", "gone.rs", "new.txt"]
        );
    }

    #[test]
    fn unmerged_paths_are_deduplicated_across_stages() {
        let listing = "100644 aaa 1\ta b.json\x00100644 bbb 2\ta b.json\x00100644 ccc 3\tz.rs\x00";
        assert_eq!(parse_unmerged(listing), vec!["a b.json", "z.rs"]);
    }

    #[test]
    fn plan_refuses_each_set_only_when_a_derived_path_is_unmerged() {
        let derived = format!("{}/spec-registry/by-spec/x.json", cfg().layout.derived_dir);
        let p = plan(&cfg(), &["specs/x.md".into()], &["dirty".into()], false);
        assert!(p.nothing_to_resolve);
        assert!(p.refusals.is_empty());
        assert_eq!(p.unstaged, vec!["dirty"]);

        let p = plan(
            &cfg(),
            &[derived.clone(), "specs/x.md".into()],
            &["dirty".into()],
            false,
        );
        assert!(!p.nothing_to_resolve);
        let reasons: Vec<_> = p.refusals.iter().map(|r| r.reason).collect();
        assert_eq!(
            reasons,
            vec![
                RefusalReason::AuthoredUnmerged,
                RefusalReason::Unstaged,
                RefusalReason::NoOperation
            ]
        );
        assert_eq!(p.derived_unmerged(), vec![derived]);
    }

    #[test]
    fn change_lines_drop_context_and_read_dashes_inside_a_hunk_as_content() {
        let patch = "diff --git a/f.md b/f.md\n--- a/f.md\n+++ b/f.md\n@@ -3 +3,2 @@\n\
                     --- a rule\n+kept\n+added\ndiff --git a/g b/g\n--- a/g\n+++ /dev/null\n\
                     @@ -1 +0,0 @@\n-only\n";
        assert_eq!(
            change_lines(patch),
            vec![
                ("f.md".to_string(), "+added".to_string()),
                ("f.md".to_string(), "+kept".to_string()),
                ("f.md".to_string(), "--- a rule".to_string()),
                ("g".to_string(), "-only".to_string()),
            ]
        );
    }

    #[test]
    fn compare_names_lost_and_unexpected_lines_per_file() {
        let carried = vec![("f".to_string(), "+b".to_string())];
        let merged: Vec<(String, String)> = Vec::new();
        let c = compare_changes(&carried, &merged);
        assert!(!c.preserved);
        assert_eq!(c.differences[0].only_in_carried, vec!["+b"]);
        assert!(compare_changes(&carried, &carried).preserved);
    }

    #[test]
    fn markers_parse_path_line_and_text() {
        let m = parse_markers("a.md\x0012\x00<<<<<<< HEAD\n");
        assert_eq!(
            m,
            vec![Marker {
                path: "a.md".into(),
                line: 12,
                text: "<<<<<<< HEAD".into()
            }]
        );
    }
}
