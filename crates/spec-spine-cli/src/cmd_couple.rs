//! `spec-spine couple`: the PR-time coupling gate (spec 005).
//!
//! This is the only place `git` runs: it invokes `git -c core.quotepath=false
//! diff --no-color -U0 --no-renames --end-of-options base...head`, parses the
//! unified diff into a typed [`DiffInput`] (new-side hunk spans), reads the PR
//! body for a waiver, and calls the pure `spec-spine-core` gate. Diff parsing is
//! ported from OAP `spec-code-coupling-check/src/main.rs` (`parse_unified_diff`,
//! `parse_hunk_header`). `--no-renames` makes a rename surface as a delete + add
//! (so a `git mv` of a governed file cannot slip past the gate),
//! `core.quotepath=false` keeps unicode paths matchable, and `--end-of-options`
//! stops a crafted ref from being parsed as a git flag. Git prints no `+++`
//! header for a mode-only or binary change, so membership is completed from
//! `git diff --name-status -z` over the same range (spec 073): the parser stays
//! the authority for spans, the name list for which paths changed.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use spec_spine_core::couple::spec_id_for_spec_md_path;
use spec_spine_core::{
    CoupleReport, DiffFile, DiffInput, FileContents, PriorOwnership, PriorSnapshots,
    couple_snapshots, dependency_only_waiver, is_bypassed_path, load_committed_index, parse_waiver,
    prior_ownership_from_root, tree_config,
};
use spec_spine_types::{Config, Error, LineSpan, Verdict, Violation, verdict::verb};

use crate::load_repo_config;
use crate::out;

/// Arguments for `spec-spine couple`.
pub struct CoupleArgs {
    pub base: String,
    pub head: String,
    pub pr_body: Option<PathBuf>,
    pub paths_from: Option<PathBuf>,
    /// Spec 081 §3.1: union the committed range with `git diff HEAD`, so a
    /// pre-commit run judges the change being committed rather than an empty
    /// set. Off by default: CI runs over a pushed range where the working tree
    /// is irrelevant and must stay so (§3.4).
    pub include_uncommitted: bool,
    /// Emit the verdict as a JSON envelope instead of prose (spec 034).
    pub json: bool,
}

pub fn run(repo: &Path, args: &CoupleArgs) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;

    let Segments {
        diff,
        worktree_deletions,
    } = build_diff_input(repo, args)?;
    let body = read_pr_body(args)?;
    let mut waiver = parse_waiver(&cfg, &body);
    let mut auto_waived = false;

    // Spec 005 §3.5: mechanical dependency-only auto-waiver. Opt-in,
    // git-diff mode only (`--paths-from` has no content to compare), and
    // never overrides an explicit PR-body waiver.
    if waiver.is_none() && cfg.coupling.auto_waive_dependency_only && args.paths_from.is_none() {
        waiver = try_dependency_only_waiver(repo, &cfg, args, &diff)?;
        auto_waived = waiver.is_some();
    }

    // Spec 100 §3.4: built only when a deletion needs one, and §3.5: a
    // required snapshot that cannot be obtained is a refusal, never a
    // head-only pass.
    let exports = PriorExports::build(repo, args, &diff, &worktree_deletions)?;
    let prior = PriorSnapshots {
        merge_base: exports.merge_base.as_ref(),
        head_commit: exports.head_commit.as_ref(),
        worktree_deletions,
    };

    let report = couple_snapshots(&cfg, repo, &diff, waiver.as_ref(), &prior)?;

    if args.json {
        // The `CoupleReport` verbatim, as `spec_spine_core::couple_json`
        // returns it: `violations`, `waiver` and `checkedPaths` are the reasons
        // a consumer needs, and the prose form's C-001/C-002 breakdown is a
        // rendering of the same `code` field rather than a second fact. The
        // auto-waiver distinction the prose draws is deliberately not in the
        // envelope: the report carries the waiver's reason, and how it was
        // obtained is a CLI concern, not part of the gate's verdict.
        let value = serde_json::to_value(&report).map_err(|e| Error::Schema(e.to_string()))?;
        let code = report.exit_code();
        out::verdict(&Verdict::report(verb::COUPLE, code, value))?;
        return Ok(code);
    }

    if report.has_blocking_drift() {
        let unclaimed = report
            .violations
            .iter()
            .filter(|v| v.code == "C-002")
            .count();
        let drift = report.violations.len() - unclaimed;
        if unclaimed == 0 {
            eprintln!(
                "spec-spine couple: {drift} drift violation(s): a changed path lacks an authoring edit to an owning spec.\n"
            );
        } else {
            eprintln!(
                "spec-spine couple: {} violation(s): {drift} drift (C-001), {unclaimed} unclaimed (C-002, require_ownership is on).\n",
                report.violations.len()
            );
        }
        for v in &report.violations {
            eprintln!("  {} {}", v.code, v.message);
        }
        eprint!("{}", resolution_footer(&cfg, &report, &diff));
    } else if let Some(reason) = &report.waiver {
        outln!(
            "spec-spine couple: {} violation(s) {}, reason: {reason}",
            report.violations.len(),
            if auto_waived { "auto-waived" } else { "waived" }
        );
        for v in &report.violations {
            outln!("  {} (waived)", v.message);
        }
    } else {
        outln!(
            "spec-spine couple: OK: {} path(s) checked, no drift.",
            report.checked_paths
        );
    }

    Ok(report.exit_code())
}

/// The resolution footer a blocking report ends with (spec 045 §3.2, §3.3).
///
/// A pure function of `(config, report, diff)`: no input is read and nothing is
/// written, so the same three arguments always produce the same bytes.
///
/// The gate has always named the specs that own a drifted path and never named
/// the mechanism for crossing into their territory. Of the two doors it used to
/// offer, both are shut for the case that produces most `C-001` refusals: an
/// author building their own spec who reached into a file another spec owns.
/// Editing the other spec is the illegitimate mid-build edit
/// `AGENTS.md` "Adversarial prompt refusal" forbids, and the waiver is a
/// human instrument an unattended session may not grant itself. The third door,
/// an `extends` edge in the author's own spec, is the corpus's actual answer,
/// and this is where the gate finally says so.
fn resolution_footer(cfg: &Config, report: &CoupleReport, diff: &DiffInput) -> String {
    let drift: Vec<&Violation> = report
        .violations
        .iter()
        .filter(|v| v.code == "C-001")
        .collect();
    let unclaimed = report.violations.len() - drift.len();
    let keyword = &cfg.coupling.waiver_keyword;

    // No `C-001`: the report is `C-002` only, and renders exactly the footer it
    // rendered before this spec (§3.2). Claiming a path is a different act from
    // crossing into somebody's territory, and the three doors do not apply.
    if drift.is_empty() {
        return format!(
            "\nResolve by editing an owning spec's spec.md (C-001), claiming the path in a spec's owning edge (C-002), or add a '{keyword}' line to the PR body.\n"
        );
    }

    let mut f = String::from("\nResolve, in the order an author should consider them");
    if unclaimed > 0 {
        f.push_str(" (1-3 answer C-001; 4 answers C-002)");
    }
    f.push_str(":\n\n");

    f.push_str(
        "  1. Edit the owning spec's spec.md. The right door when the spec that owns\n\
         \x20    the path is the one you are authoring.\n\n",
    );

    f.push_str(
        "  2. Declare an `extends` edge in your OWN spec, naming the owning spec\n\
         \x20    and the unit you touched. That makes your spec a legitimate owner of\n\
         \x20    the unit, so the gate clears on the next run. It amends nobody and\n\
         \x20    needs no waiver.\n\n",
    );
    f.push_str(&extends_guidance(cfg, &drift, diff));

    f.push_str(&format!(
        "  3. Add a '{keyword} <reason>' line to the PR body. A waiver is a\n\
         \x20    human instrument: it needs explicit human approval, and is not a\n\
         \x20    flag an unattended session sets for itself.\n"
    ));

    if unclaimed > 0 {
        f.push_str(
            "\n  4. For the unclaimed path(s) above, claim the path in a spec's owning\n\
             \x20    edge.\n",
        );
    }
    f
}

/// Door two's body: the concrete `extends` block when the diff makes it
/// concrete, else the shape to fill in (spec 045 §3.3).
///
/// The specific form triggers on **exactly one** edited `spec.md`, and needs no
/// further test: clearance is "any one owner's spec.md is in the diff", so a
/// single edited spec that owned a violating path would already have cleared it.
/// Every `C-001` reaching this footer is therefore a path that spec does not
/// own, which is exactly the crossing case. Zero or two-or-more edited specs
/// fall back to the generic form: an amendment pair is a legitimate shape and
/// the gate has no basis for guessing which of the two should declare the edge.
fn extends_guidance(cfg: &Config, drift: &[&Violation], diff: &DiffInput) -> String {
    let specs_dir = cfg.layout.specs_dir.as_str();
    let edited: Vec<&str> = diff
        .files
        .iter()
        .filter_map(|f| spec_id_for_spec_md_path(specs_dir, &f.path))
        .collect();

    let Some(id) = edited.first().filter(|_| edited.len() == 1) else {
        return concat!(
            "     In your spec's frontmatter:\n\n",
            "       extends:\n",
            "         - { spec: \"<owning-spec-id>\", unit: \"<path>\", nature: additive }\n\n",
        )
        .to_string();
    };

    let mut s = format!(
        "     spec {id} is the only spec.md edited in this diff, and owns none of\n\
         \x20    the paths above. Cross into the owning territory by declaring, in\n\
         \x20    {}:\n\n\
         \x20      extends:\n",
        spec_md_rel(specs_dir, id)
    );
    // One item per violating path, in the report's existing sorted-by-path
    // order. Three values are defaults rather than verdicts: the first owner in
    // sorted order (clearing needs only one), the file-shorthand unit, and
    // `nature: additive`. The note below says so, because a gate that presented
    // a guess as the answer would be worse than one that stayed silent.
    for v in drift {
        let (Some(path), Some(owner)) = (v.path.as_deref(), v.owners.first()) else {
            continue;
        };
        s.push_str(&format!(
            "         - {{ spec: \"{owner}\", unit: \"{path}\", nature: additive }}\n"
        ));
    }
    s.push_str(
        "\n     A suggestion, not an instruction, and not the only correct form:\n\
         \x20    where a path has several owners any one of them clears it, a narrower\n\
         \x20    section or symbol unit is available if you want the claim tighter,\n\
         \x20    and `nature` is a free-text hint (`superseding` describes a crossing\n\
         \x20    that replaces behavior rather than adding to it). If you did not mean\n\
         \x20    to touch the path, revert the touch and declare nothing.\n\n",
    );
    s
}

/// `<specs_dir>/<id>/spec.md`, matching the core gate's own `spec_md_rel`.
fn spec_md_rel(specs_dir: &str, id: &str) -> String {
    format!("{}/{id}/spec.md", specs_dir.trim_end_matches('/'))
}

/// Build the [`DiffInput`]: either from `--paths-from` (whole-file fallback, no
/// hunks) or from `git diff --no-color -U0 base...head`.
/// The diff the gate judges, plus which deletions the working-tree segment
/// recorded (spec 100 §3.1).
///
/// The segment matters because a deletion is judged at the snapshot preceding
/// the segment that recorded it: the merge base for the committed range, HEAD
/// for `git diff HEAD`. Carried here rather than on [`DiffFile`] so the diff's
/// serialized shape does not move.
pub(crate) struct Segments {
    pub(crate) diff: DiffInput,
    pub(crate) worktree_deletions: BTreeSet<String>,
}

fn build_diff_input(repo: &Path, args: &CoupleArgs) -> Result<Segments, Error> {
    if let Some(path) = &args.paths_from {
        // Spec 081 §3.3: `--paths-from` carries its own path list and no history
        // to union a working tree with, so the combination names no coherent
        // question. Refused (exit 3) rather than silently ignoring one of them.
        if args.include_uncommitted {
            return Err(Error::Config(
                "--include-uncommitted unions the working tree into a git diff, and \
                 --paths-from replaces that diff with a path list; pass one or the other"
                    .to_string(),
            ));
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| Error::Io(format!("read --paths-from {}: {e}", path.display())))?;
        let files = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|p| DiffFile {
                path: p.to_string(),
                hunks: Vec::new(),
                // `--paths-from` carries no history, so a listed path is never
                // known to be a deletion: the ratchet judges it as an edit.
                deleted: false,
            })
            .collect();
        // Spec 100 §3.9: a path list has no prior side, so no snapshot is
        // required and none is built.
        return Ok(Segments {
            diff: DiffInput { files },
            worktree_deletions: BTreeSet::new(),
        });
    }

    // Spec 100 §3.5: the three-dot range is itself a read of history, and it
    // is the read that fails first when the clone cannot reach a merge base
    // (a shallow fetch, or unrelated histories). Git's own message says what
    // broke; this adds the remedy and, more importantly, says plainly that
    // nothing was judged. A run that cannot read the range must never look
    // like a run that found nothing to judge.
    let raw =
        run_git_diff(repo, &[&format!("{}...{}", args.base, args.head)]).map_err(unobtainable)?;
    let mut diff = parse_unified_diff(&raw);
    let mut worktree_deletions: BTreeSet<String> = BTreeSet::new();
    // Spec 073 §3.1: the parser is the authority for spans, the name list for
    // membership. The range is the same three-dot `base...head` the text diff
    // read, so both answers describe one set of changes.
    let range = format!("{}...{}", args.base, args.head);
    let statuses = changed_path_statuses(repo, &[range.as_str()])?;
    union_name_statuses(&mut diff, statuses);

    // Spec 081 §3.1: the working tree, on request. `base...head` describes
    // history alone, so before the commit exists the gate reports
    // `0 path(s) checked, no drift` and exits 0, which reads as a pass.
    if args.include_uncommitted {
        // Spec 081 §3.3: the comparison is against HEAD, so a `--head` naming
        // anything else would union a working tree against an unrelated commit
        // and describe a state that never existed. Refused, not guessed.
        let head_oid = rev_parse(repo, "HEAD")?;
        if rev_parse(repo, &args.head)? != head_oid {
            return Err(Error::Config(format!(
                "--include-uncommitted compares the working tree with HEAD, so it cannot be \
                 combined with --head {}, which resolves to a different commit",
                args.head
            )));
        }
        // `git diff HEAD` covers staged and unstaged changes to tracked files:
        // exactly what a commit would record. A file never `git add`-ed is
        // absent from both, which is right, since a commit would not carry it
        // either (§3.2).
        let wt_raw = run_git_diff(repo, &["HEAD"])?;
        let wt = parse_unified_diff(&wt_raw);
        union_diff(&mut diff, wt);
        union_name_statuses(&mut diff, changed_path_statuses(repo, &["HEAD"])?);

        // Spec 100 §3.1: the working-tree segment's deletions, read back from
        // the unioned result rather than from either raw view. The union rule
        // (spec 081) is that the later view decides, so a path this segment
        // restored is not a deletion at all and must not be listed.
        let wt_paths: BTreeSet<String> = changed_path_statuses(repo, &["HEAD"])?
            .into_iter()
            .filter(|(status, _)| status == "D")
            .map(|(_, path)| path)
            .collect();
        worktree_deletions = diff
            .files
            .iter()
            .filter(|f| f.deleted && wt_paths.contains(&f.path))
            .map(|f| f.path.clone())
            .collect();
    }
    Ok(Segments {
        diff,
        worktree_deletions,
    })
}

/// `git rev-parse <rev>`, for the one comparison spec 081 §3.3 needs.
fn rev_parse(repo: &Path, rev: &str) -> Result<String, Error> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--verify", "--end-of-options"])
        .arg(rev)
        .output()
        .map_err(|e| Error::Io(format!("spawn git rev-parse: {e}")))?;
    if !out.status.success() {
        return Err(Error::Io(format!(
            "git rev-parse {rev} exited {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Fold a second parsed diff into the first (spec 081 §3.1).
///
/// A path only the later view knows enters whole. A path both views know keeps
/// the union of their hunks and takes the LATER view's deletion verdict: the
/// gate judges the state a commit would produce, so a file deleted in the range
/// and restored in the working tree is not a deletion, and the reverse is.
fn union_diff(diff: &mut DiffInput, later: DiffInput) {
    for f in later.files {
        match diff.files.iter_mut().find(|e| e.path == f.path) {
            Some(existing) => {
                existing.hunks.extend(f.hunks);
                existing.deleted = f.deleted;
            }
            None => diff.files.push(f),
        }
    }
    diff.files.sort_by(|a, b| a.path.cmp(&b.path));
}

/// Add every path git reports changed that the hunk parser did not register
/// (spec 073 §3.1, §3.2).
///
/// Git prints no `---`/`+++` header for a mode-only or a binary change, so
/// [`parse_unified_diff`] never sees those paths. Each enters as a whole-file
/// change (no hunks), deleted exactly when its status letter is `D`. A path the
/// parser already registered keeps its spans and its deletion verdict: the name
/// list contributes membership only. The result stays sorted by path, the order
/// the parser's map produces.
fn union_name_statuses(diff: &mut DiffInput, statuses: Vec<(String, String)>) {
    let mut known: std::collections::BTreeSet<String> =
        diff.files.iter().map(|f| f.path.clone()).collect();
    let mut added = false;
    for (status, path) in statuses {
        if !known.insert(path.clone()) {
            continue;
        }
        diff.files.push(DiffFile {
            deleted: status == "D",
            path,
            hunks: Vec::new(),
        });
        added = true;
    }
    if added {
        diff.files.sort_by(|a, b| a.path.cmp(&b.path));
    }
}

/// Attempt the spec 005 §3.5 mechanical auto-waiver (extended to cargo and
/// workflow manifests by spec 027): every non-bypassed changed path must be a
/// recognized dependency manifest (`package.json` / `Cargo.toml` /
/// `.github/workflows/*.yml`) whose base→head change is confined to dependency
/// version pins. Contents come from `git show` at the **merge base** (the diff
/// is three-dot, so the base side is `merge-base(base, head)`, not the base
/// branch tip) and at `head`. Any git failure refuses the auto-waiver
/// fail-closed rather than erroring the gate.
///
/// The bypass verdict is claim-aware (spec 008): it assembles the committed
/// index from its shard set so a claim-overridden floor path counts as a
/// candidate and refuses the waiver, matching exactly the path set the gate
/// evaluates. An unreadable index refuses fail-closed (the gate itself will
/// report the real error).
fn try_dependency_only_waiver(
    repo: &Path,
    cfg: &Config,
    args: &CoupleArgs,
    diff: &DiffInput,
) -> Result<Option<spec_spine_core::Waiver>, Error> {
    let Ok(index) = load_committed_index(cfg, repo) else {
        return Ok(None);
    };
    let candidates: Vec<&DiffFile> = diff
        .files
        .iter()
        .filter(|f| !is_bypassed_path(cfg, &index, &f.path))
        .collect();
    // Cheap pre-filter before any git spawn: the waiver can only ever apply
    // when every non-bypassed path is a recognized dependency manifest
    // (package.json / Cargo.toml / workflow YAML).
    if candidates.is_empty()
        || !candidates
            .iter()
            .all(|f| spec_spine_core::is_dependency_manifest(&f.path))
    {
        return Ok(None);
    }

    let Some(merge_base) = git_merge_base(repo, &args.base, &args.head) else {
        return Ok(None);
    };

    let mut files: Vec<FileContents> = Vec::with_capacity(candidates.len());
    for f in &candidates {
        files.push(FileContents {
            path: f.path.clone(),
            base: git_show(repo, &merge_base, &f.path),
            head: git_show(repo, &args.head, &f.path),
        });
    }
    Ok(dependency_only_waiver(&files))
}

fn git_merge_base(repo: &Path, base: &str, head: &str) -> Option<String> {
    merge_base(repo, base, head).ok()
}

/// `merge-base(base, head)`, the base side of a three-dot diff.
///
/// Shared with `delta` (spec 071 §3.1), which classifies under that commit's
/// rules and so needs the failure reason rather than the auto-waiver's
/// fail-closed `None`.
pub(crate) fn merge_base(repo: &Path, base: &str, head: &str) -> Result<String, Error> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        // `--end-of-options`: everything after is an operand, so a ref that looks
        // like a flag (`--foo`) can never be parsed as a git option.
        .args(["merge-base", "--end-of-options", base, head])
        .output()
        .map_err(|e| Error::Io(format!("spawn git merge-base: {e}")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(Error::Io(format!(
            "git merge-base {base} {head} exited {:?}: {}",
            out.status.code(),
            stderr.trim()
        )));
    }
    let rev = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if rev.is_empty() {
        return Err(Error::Io(format!(
            "git merge-base {base} {head} printed no commit"
        )));
    }
    Ok(rev)
}

/// Every path `from..to` changes, with renames disabled (spec 071 §3.1).
///
/// A name list rather than [`parse_unified_diff`]: that parser registers a path
/// from its `+++`/`---` headers, and git prints none for a binary file or a
/// mode-only change, so both would be absent from a report that claims to
/// classify every changed path. Expressed through [`changed_path_statuses`]
/// (spec 073 §3.3), so `couple` and `delta` cannot disagree about which paths
/// changed.
pub(crate) fn changed_path_names(repo: &Path, from: &str, to: &str) -> Result<Vec<String>, Error> {
    Ok(changed_path_statuses(repo, &[from, to])?
        .into_iter()
        .map(|(_, path)| path)
        .collect())
}

/// Every changed path with its status letter, `(status, path)`, in git's order
/// (spec 073 §3.3). `revs` is the revision operand list: `[from, to]` for
/// `delta`, or the single `base...head` operand `couple`'s text diff reads.
///
/// `-z` keeps a path containing a newline intact and unquoted,
/// `core.quotepath=false` keeps a non-ASCII path literal, `--no-renames` makes
/// a move a delete plus an add, and `--end-of-options` stops a ref that looks
/// like a flag from being parsed as one: the flags [`run_git_diff`] passes, for
/// the same reasons.
pub(crate) fn changed_path_statuses(
    repo: &Path,
    revs: &[&str],
) -> Result<Vec<(String, String)>, Error> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "core.quotepath=false"])
        .args([
            "diff",
            "--name-status",
            "-z",
            "--no-renames",
            "--end-of-options",
        ])
        .args(revs)
        .output()
        .map_err(|e| Error::Io(format!("spawn git diff: {e}")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(Error::Io(format!(
            "git diff --name-status exited {:?}: {}",
            out.status.code(),
            stderr.trim()
        )));
    }
    parse_name_status_z(&out.stdout)
}

/// Parse `git diff --name-status -z` output: a status field, then its path,
/// each NUL-terminated. A rename or copy status (`R<score>`, `C<score>`) carries
/// two paths; `--no-renames` means neither is expected, but both are consumed so
/// the fields cannot fall out of step, and the destination is the path reported.
fn parse_name_status_z(stdout: &[u8]) -> Result<Vec<(String, String)>, Error> {
    let mut fields = stdout
        .split(|b| *b == 0)
        .map(|f| String::from_utf8_lossy(f).into_owned());
    let mut entries = Vec::new();
    while let Some(status) = fields.next() {
        if status.is_empty() {
            continue;
        }
        let two_paths = status.starts_with('R') || status.starts_with('C');
        let mut path = fields.next();
        if two_paths {
            path = fields.next();
        }
        match path {
            Some(path) if !path.is_empty() => entries.push((status, path)),
            _ => {
                return Err(Error::Io(format!(
                    "git diff --name-status: status {status:?} with no path"
                )));
            }
        }
    }
    Ok(entries)
}

fn git_show(repo: &Path, rev: &str, path: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        // `--end-of-options`: the `rev:path` operand can never be parsed as a flag.
        .args(["show", "--end-of-options", &format!("{rev}:{path}")])
        .output()
        .ok()?;
    if !out.status.success() {
        return None; // absent at this rev (created/deleted); fail closed upstream
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn run_git_diff(repo: &Path, revs: &[&str]) -> Result<String, Error> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        // `core.quotepath=false`: keep non-ASCII paths literal instead of octal
        // C-quoting, so a governed file with a unicode/space name still matches
        // its owning unit rather than arriving as a quoted, unmatched string.
        .args(["-c", "core.quotepath=false"])
        // `--no-renames`: a pure rename emits `rename from/to` with no +++/---
        // headers and would be invisible to the parser, letting a `git mv` of a
        // governed file slip past the gate. Disabling rename detection surfaces
        // it as a delete of the old path + an add of the new path (both parsed as
        // whole-file changes), so the gate evaluates both. `--end-of-options`:
        // the `base...head` operand can never be parsed as a git flag.
        .args([
            "diff",
            "--no-color",
            "-U0",
            "--no-renames",
            "--end-of-options",
        ])
        .args(revs)
        .output()
        .map_err(|e| Error::Io(format!("spawn git diff: {e}")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(Error::Io(format!(
            "git diff exited {:?}: {stderr}",
            out.status.code()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Parse `git diff --no-color -U0` output into a [`DiffInput`]. New-side hunk
/// ranges become inclusive [`LineSpan`]s; a deleted file (`+++ /dev/null`) is
/// registered with no hunks (a whole-file change) and flagged `deleted` so the
/// spec 029 ownership ratchet can leave it alone.
fn parse_unified_diff(diff_text: &str) -> DiffInput {
    use std::collections::BTreeMap;
    /// Per-path accumulator: new-side hunks and whether the path was deleted.
    #[derive(Default)]
    struct Entry {
        hunks: Vec<LineSpan>,
        deleted: bool,
    }
    let mut files: BTreeMap<String, Entry> = BTreeMap::new();
    let mut current_path: Option<String> = None;
    let mut minus_path: Option<String> = None;

    for line in diff_text.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            minus_path = strip_diff_prefix(rest.trim());
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            let p = rest.trim();
            let deleted = p == "/dev/null";
            if deleted {
                // Deletion: the changed path is the old (minus) side, whole-file.
                current_path = minus_path.clone();
            } else {
                current_path = strip_diff_prefix(p);
            }
            if let Some(path) = &current_path {
                let entry = files.entry(path.clone()).or_default();
                entry.deleted = deleted;
            }
        } else if line.starts_with("@@") {
            if let Some(path) = &current_path {
                if let Some(span) = parse_hunk_header(line) {
                    files.entry(path.clone()).or_default().hunks.push(span);
                }
            }
        }
    }

    DiffInput {
        files: files
            .into_iter()
            .map(|(path, entry)| DiffFile {
                path,
                hunks: entry.hunks,
                deleted: entry.deleted,
            })
            .collect(),
    }
}

/// `a/<path>` / `b/<path>` → `<path>`; `/dev/null` → `None`.
fn strip_diff_prefix(p: &str) -> Option<String> {
    if p == "/dev/null" {
        return None;
    }
    Some(
        p.strip_prefix("a/")
            .or_else(|| p.strip_prefix("b/"))
            .unwrap_or(p)
            .to_string(),
    )
}

/// Parse `@@ -<old> +<new> @@` into the inclusive new-side span. A pure-deletion
/// hunk (`+start,0`) collapses to the single line at `start`.
fn parse_hunk_header(line: &str) -> Option<LineSpan> {
    let after_at = line.strip_prefix("@@")?.trim_start();
    let rest = after_at.strip_prefix('-')?;
    let plus_pos = rest.find('+')?;
    let new_part = rest[plus_pos + 1..].trim_start();
    let new_range = new_part.split_whitespace().next()?;
    let (start_s, count_s) = match new_range.split_once(',') {
        Some((a, b)) => (a, b),
        None => (new_range, "1"),
    };
    let start: usize = start_s.parse().ok()?;
    let count: usize = count_s.parse().ok()?;
    if start == 0 {
        // `+0,0`: a deletion with no new-side line. Whole-file fallback handles
        // the path; emit nothing for this hunk.
        return None;
    }
    let count = count.max(1);
    Some(LineSpan::new(start, start + count - 1))
}

fn read_pr_body(args: &CoupleArgs) -> Result<String, Error> {
    if let Some(path) = &args.pr_body {
        std::fs::read_to_string(path)
            .map_err(|e| Error::Io(format!("read --pr-body {}: {e}", path.display())))
    } else if let Ok(s) = std::env::var("SPEC_SPINE_PR_BODY") {
        Ok(s)
    } else {
        Ok(String::new())
    }
}

// ===== spec 100: the prior snapshots a deletion is judged against =====

/// The reconstructed prior snapshots for one run, with their exported trees.
///
/// The trees are removed when this is dropped, on the error path too. The
/// snapshots borrow nothing from them: `compile` and `index` return owned DTOs,
/// so the directories are needed only while they are being read.
pub(crate) struct PriorExports {
    pub(crate) merge_base: Option<PriorOwnership>,
    pub(crate) head_commit: Option<PriorOwnership>,
    _root: Option<TempRoot>,
}

impl PriorExports {
    /// Reconstruct only the snapshots this run's deletions actually need
    /// (spec 100 §3.4), refusing rather than falling back when a required one
    /// cannot be obtained (§3.5).
    fn build(
        repo: &Path,
        args: &CoupleArgs,
        diff: &DiffInput,
        worktree_deletions: &BTreeSet<String>,
    ) -> Result<Self, Error> {
        let deleted: Vec<&DiffFile> = diff.files.iter().filter(|f| f.deleted).collect();
        // §3.4: no deletion asks a question a prior snapshot could answer, so
        // nothing is resolved, nothing is exported, and a shallow clone keeps
        // working for the ordinary pull request.
        if deleted.is_empty() {
            return Ok(PriorExports {
                merge_base: None,
                head_commit: None,
                _root: None,
            });
        }
        let needs_merge_base = deleted
            .iter()
            .any(|f| !worktree_deletions.contains(&f.path));
        let needs_head_commit = deleted.iter().any(|f| worktree_deletions.contains(&f.path));

        let root = TempRoot::create()?;
        let mut exports = PriorExports {
            merge_base: None,
            head_commit: None,
            _root: None,
        };
        if needs_merge_base {
            let commit = merge_base(repo, &args.base, &args.head).map_err(unobtainable)?;
            exports.merge_base = Some(snapshot_at(repo, &commit, &root, "merge-base")?);
        }
        if needs_head_commit {
            exports.head_commit = Some(snapshot_at(repo, "HEAD", &root, "head-commit")?);
        }
        exports._root = Some(root);
        Ok(exports)
    }
}

/// Export `commit`'s tree and reconstruct ownership from its source bytes.
///
/// Spec 100 §3.2: compiled and indexed under the **exported tree's own**
/// `spec-spine.toml`, so a change cannot re-own a path it is deleting by
/// editing the configuration in the same commit. §3.5: every failure here is
/// an `Err` naming the cause and the remedy, never an empty snapshot and never
/// a substituted revision.
fn snapshot_at(
    repo: &Path,
    commit: &str,
    root: &TempRoot,
    label: &str,
) -> Result<PriorOwnership, Error> {
    let dest = root.0.join(label);
    std::fs::create_dir(&dest).map_err(|e| Error::Io(format!("create {}: {e}", dest.display())))?;
    crate::cmd_delta::export_tree(repo, commit, &root.0.join(format!("{label}.index")), &dest)
        .map_err(|e| {
            unobtainable(Error::Io(format!(
                "could not export the {label} tree ({commit}): {e}"
            )))
        })?;
    let cfg = tree_config(&dest)?;
    let paths = tracked_paths(repo, commit)?;
    let snapshot = prior_ownership_from_root(&cfg, &dest).map_err(|e| {
        Error::Parse(format!(
            "the {label} snapshot's corpus could not be compiled, so this change's \
             deletions cannot be judged: {e}. A snapshot that could not be built has not \
             answered; repair that commit's corpus and rebase rather than re-running."
        ))
    })?;
    Ok(snapshot.with_paths(paths))
}

/// `git ls-tree -r --name-only <commit>`: the snapshot's tracked inventory.
///
/// Used only to tell "the snapshot says nobody owned this path" from "the path
/// was not there at all" (spec 100 §3.5). Both give the same verdict; the
/// distinction is reported.
fn tracked_paths(repo: &Path, commit: &str) -> Result<BTreeSet<String>, Error> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "-c",
            "core.quotepath=false",
            "ls-tree",
            "-r",
            "--name-only",
            "--full-tree",
            "-z",
            "--end-of-options",
            commit,
        ])
        .output()
        .map_err(|e| Error::Io(format!("spawn git ls-tree: {e}")))?;
    if !out.status.success() {
        return Err(unobtainable(Error::Io(format!(
            "git ls-tree {commit} exited {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        ))));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

/// Wrap a git failure as the spec 100 §3.5 refusal, with the operator remedy.
///
/// Every state that reaches here has the same shape: the clone does not hold
/// the history the gate needs. The gate must not substitute another revision,
/// must not ignore the failure, and must not treat a missing history as an
/// empty diff, so it says what happened, that it judged nothing, and what to
/// do about it.
fn unobtainable(e: Error) -> Error {
    Error::Io(format!(
        "{e}\n\nThe gate reads history: the diff is a three-dot range, and a deleted path \
         is judged against the snapshot it lived in (spec 100). That history could not be \
         read, so the gate has not judged this change and will not report a pass it did \
         not compute.\n\
         If this is a shallow clone, fetch enough history to reach the merge base \
         (`git fetch --deepen=<n>`, or clone without `--depth`). If the histories are \
         unrelated, name a base that shares one."
    ))
}

/// A temporary directory removed when dropped.
struct TempRoot(PathBuf);

impl TempRoot {
    fn create() -> Result<Self, Error> {
        let parent = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        // `create_dir`, not `create_dir_all`: an existing directory is someone
        // else's, so a collision tries the next name rather than reusing it.
        for attempt in 0..16u32 {
            let root = parent.join(format!("spec-spine-couple-{pid}-{nanos}-{attempt}"));
            if std::fs::create_dir(&root).is_ok() {
                return Ok(TempRoot(root));
            }
        }
        Err(Error::Io(
            "could not create a temporary directory for the prior snapshot".to_string(),
        ))
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modification_hunks_to_inclusive_spans() {
        let diff = "diff --git a/Makefile b/Makefile\n\
                    --- a/Makefile\n\
                    +++ b/Makefile\n\
                    @@ -10,2 +10,3 @@ ctx\n\
                    @@ -50 +51,5 @@\n";
        let d = parse_unified_diff(diff);
        let f = d.files.iter().find(|f| f.path == "Makefile").unwrap();
        assert_eq!(f.hunks, vec![LineSpan::new(10, 12), LineSpan::new(51, 55)]);
    }

    #[test]
    fn deleted_file_is_whole_file_change() {
        let diff = "diff --git a/gone.rs b/gone.rs\n\
                    deleted file mode 100644\n\
                    --- a/gone.rs\n\
                    +++ /dev/null\n\
                    @@ -1,5 +0,0 @@\n";
        let d = parse_unified_diff(diff);
        let f = d.files.iter().find(|f| f.path == "gone.rs").unwrap();
        assert!(f.hunks.is_empty(), "deletion ⇒ whole-file (no hunks)");
        assert!(f.deleted, "deletion is flagged for the ownership ratchet");
    }

    #[test]
    fn modified_and_added_files_are_not_deleted() {
        let diff = "diff --git a/Makefile b/Makefile\n\
                    --- a/Makefile\n\
                    +++ b/Makefile\n\
                    @@ -10,2 +10,3 @@ ctx\n\
                    diff --git a/new.rs b/new.rs\n\
                    new file mode 100644\n\
                    --- /dev/null\n\
                    +++ b/new.rs\n\
                    @@ -0,0 +1,3 @@\n";
        let d = parse_unified_diff(diff);
        assert!(d.files.iter().all(|f| !f.deleted), "{:?}", d.files);
    }

    #[test]
    fn rename_under_no_renames_registers_both_paths() {
        // `--no-renames` turns a rename into a delete of the old path plus an add
        // of the new path, so BOTH surfaces reach the gate. Without it, a pure
        // rename emits only `rename from/to` lines (no +++/---) and would be
        // invisible: a `git mv` of a governed file could slip past coupling.
        let diff = "diff --git a/old/mod.rs b/old/mod.rs\n\
                    deleted file mode 100644\n\
                    --- a/old/mod.rs\n\
                    +++ /dev/null\n\
                    @@ -1,3 +0,0 @@\n\
                    diff --git a/new/mod.rs b/new/mod.rs\n\
                    new file mode 100644\n\
                    --- /dev/null\n\
                    +++ b/new/mod.rs\n\
                    @@ -0,0 +1,3 @@\n";
        let d = parse_unified_diff(diff);
        assert!(
            d.files.iter().any(|f| f.path == "old/mod.rs"),
            "vacated path must be seen"
        );
        assert!(
            d.files.iter().any(|f| f.path == "new/mod.rs"),
            "new path must be seen"
        );
    }

    #[test]
    fn name_status_z_pairs_each_status_with_its_path() {
        let out = b"M\0src/a.rs\0D\0gone.bin\0A\0path with\nnewline\0R100\0old.rs\0new.rs\0";
        let entries = parse_name_status_z(out).unwrap();
        assert_eq!(
            entries,
            vec![
                ("M".to_string(), "src/a.rs".to_string()),
                ("D".to_string(), "gone.bin".to_string()),
                ("A".to_string(), "path with\nnewline".to_string()),
                ("R100".to_string(), "new.rs".to_string()),
            ]
        );
        assert!(parse_name_status_z(b"").unwrap().is_empty());
        assert!(
            parse_name_status_z(b"M\0").is_err(),
            "a status with no path"
        );
    }

    #[test]
    fn union_adds_headerless_paths_and_keeps_parsed_spans() {
        // The parser saw only `lib.rs`; git also reports a mode flip and a binary
        // delete it printed no header for.
        let mut d = parse_unified_diff(
            "diff --git a/lib.rs b/lib.rs\n\
             --- a/lib.rs\n\
             +++ b/lib.rs\n\
             @@ -3 +3,2 @@\n",
        );
        union_name_statuses(
            &mut d,
            vec![
                ("M".into(), "run.sh".into()),
                ("M".into(), "lib.rs".into()),
                ("D".into(), "logo.bin".into()),
            ],
        );
        let paths: Vec<&str> = d.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, vec!["lib.rs", "logo.bin", "run.sh"]);
        let lib = &d.files[0];
        assert_eq!(lib.hunks, vec![LineSpan::new(3, 4)], "spans survive");
        assert!(!lib.deleted);
        assert!(d.files[1].deleted && d.files[1].hunks.is_empty());
        assert!(!d.files[2].deleted && d.files[2].hunks.is_empty());
    }

    /// Spec 073 §3.7 case 5, over a real repository: a text edit made together
    /// with a mode flip is reported by both sources, and the adapter keeps the
    /// hunk span the parser found rather than flattening it to whole-file. Here
    /// and not in `tests/couple.rs`, because no verdict the binary emits depends
    /// on spans for an index-resolved unit (073 D-3).
    #[test]
    fn real_git_text_change_keeps_spans_through_the_union() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["-c", "commit.gpgsign=false"])
                .args(args)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .output()
                .unwrap();
            assert!(out.status.success(), "git {args:?}: {out:?}");
        };
        git(&["init", "-q"]);
        std::fs::write(
            root.join("Makefile"),
            "top:\n\techo top\n\nbot:\n\techo bot\n",
        )
        .unwrap();
        std::fs::write(root.join("logo.png"), b"\x89PNG\0\x01").unwrap();
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", "base"]);
        std::fs::write(
            root.join("Makefile"),
            "top:\n\techo top\n\nbot:\n\techo bottom\n",
        )
        .unwrap();
        std::fs::write(root.join("logo.png"), b"\x89PNG\0\x02").unwrap();
        git(&["add", "-A"]);
        git(&["update-index", "--chmod=+x", "Makefile"]);
        git(&["commit", "-q", "-m", "head"]);

        let args = CoupleArgs {
            base: "HEAD~1".into(),
            head: "HEAD".into(),
            pr_body: None,
            paths_from: None,
            // Spec 081: this fixture asserts the committed range alone, which
            // is the default and what CI runs.
            include_uncommitted: false,
            json: false,
        };
        let d = build_diff_input(root, &args).unwrap().diff;
        let paths: Vec<&str> = d.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, vec!["Makefile", "logo.png"]);
        assert_eq!(d.files[0].hunks, vec![LineSpan::new(5, 5)], "span kept");
        assert!(d.files[1].hunks.is_empty(), "binary is whole-file");
        assert!(!d.files[0].deleted && !d.files[1].deleted);
    }
}
