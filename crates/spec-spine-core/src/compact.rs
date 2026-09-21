// Spec: specs/096-compaction-is-a-verb-not-a-session/spec.md
//! `compact` (spec 096): corpus compaction as a deterministic function.
//!
//! Spec 095 removed 27 specs and renumbered 93, which meant rewriting about six
//! thousand references by hand. Five defects were introduced doing it and two
//! more were found afterwards, every one a rewrite rule that matched more or
//! less than its author meant, and every one silent: the corpus compiled, the
//! gate passed, and the citations pointed at the wrong documents.
//!
//! This module is the rules, written once. It reads the corpus and returns the
//! rewritten files **as data** (spec 096 §3.1, D-2): it performs no write, takes
//! no clock and shells out to nothing, so the engine's central invariant is
//! unchanged and `--plan` is the same call with nothing written.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use spec_spine_types::{Config, Error};

/// One spec leaving the corpus, with the spec that answers for it afterwards
/// (spec 096 §3.1). Authored, never inferred: which specs merge is judgement,
/// and §1.3 puts it outside this verb.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoveEntry {
    pub spec: String,
    pub answered_by: String,
}

/// Whether the survivors' ordinals are compacted (spec 096 §3.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Renumber {
    /// Survivors are renumbered in their existing order, from `000`.
    #[default]
    Contiguous,
    /// Removals only; every survivor keeps the ordinal it has.
    None,
}

/// The authored plan (spec 096 §3.1).
///
/// `Default` is load-bearing, as it is on `ScaffoldFile`: spec 097 added a
/// field and every struct literal that built one broke. Construct with
/// `..Default::default()` so the next field added here breaks nothing, and
/// `serde(default)` on each field does the same for a plan written before it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactPlan {
    #[serde(default)]
    pub remove: Vec<RemoveEntry>,
    #[serde(default)]
    pub renumber: Renumber,
    /// Names that precede `spec NNN` in a citation of a **different** project's
    /// corpus, whose ordinals are not this corpus's (spec 096 §3.3 form 2).
    ///
    /// Compared case-insensitively against the token before the keyword. The
    /// default carries the one this corpus actually contains; an unmapped
    /// ordinal is already left alone, so this list matters only where a foreign
    /// project's ordinal happens to collide with one of ours.
    #[serde(default = "default_foreign_projects")]
    pub foreign_projects: Vec<String>,
    /// Paths leaving the corpus (spec 097 §3.1). A spec id and a path are two
    /// spellings of the same retirement, so they share this plan, the refusal
    /// set, the idempotence requirement and the per-form report.
    #[serde(default)]
    pub retire: Vec<RetireEntry>,
}

fn default_foreign_projects() -> Vec<String> {
    vec!["OAP".to_string()]
}

/// Which rule produced a rewrite (spec 096 §3.6). Reported per rewrite and
/// counted per form, so a form that fired zero times in a corpus that obviously
/// contains it is visible: defect 2 was exactly that.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Form {
    /// `NNN-slug`, matched against the map's keys, longest key first.
    FullId,
    /// `spec NNN` / `specs NNN` in prose, including a citation broken across a
    /// line and one naming more than one ordinal.
    Citation,
    /// A bare short id as an argument to a command that takes a spec id.
    ShortIdArg,
    /// An occurrence of a path leaving the corpus (spec 097).
    RetiredPath,
}

impl Form {
    pub fn as_str(self) -> &'static str {
        match self {
            Form::FullId => "full-id",
            Form::Citation => "citation",
            Form::ShortIdArg => "short-id-arg",
            Form::RetiredPath => "retired-path",
        }
    }
}

/// One rewrite, as the report carries it (spec 096 §3.6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rewrite {
    /// 1-based line of the rewritten text in the file, before the rewrite.
    pub line: usize,
    pub old: String,
    pub new: String,
    pub form: Form,
}

/// Every rewrite in one file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRewrites {
    pub rel_path: String,
    pub rewrites: Vec<Rewrite>,
}

/// One file of the rewritten corpus, for the caller to write (spec 096 §3.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactFile {
    /// Where the file lands. Differs from `fromRelPath` when a spec directory
    /// is renumbered.
    pub rel_path: String,
    /// Where it came from, so the caller knows to remove the old path.
    pub from_rel_path: String,
    pub contents: String,
}

/// One entry of the map document (spec 096 §3.7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapEntry {
    pub from: String,
    /// The survivor's new id, or, for a removed spec, the id that answers for it.
    pub to: String,
    pub removed: bool,
}

/// What `compact` returns: the rewritten corpus, the map, and the review.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Compaction {
    pub files: Vec<CompactFile>,
    /// Directories that leave the tree, one per removed spec. The **directory**
    /// and not its `spec.md`: a spec is its directory, auxiliary files
    /// included, and reporting the document while deleting the folder would
    /// hide whatever else was in it from a consumer reading this list.
    pub removed_paths: Vec<String>,
    pub map: Vec<MapEntry>,
    pub rewrites: Vec<FileRewrites>,
    /// Per form, how many rewrites it produced.
    pub counts: BTreeMap<String, usize>,
    /// The map document's contents (spec 096 §3.7).
    pub map_document: String,
    /// Occurrences of a retired path deliberately left alone, with the clause
    /// that spared each (spec 097 §3.5). Reported, never silent.
    #[serde(default)]
    pub skipped: Vec<Skipped>,
    /// Occurrences of a retired path that survived the rewrite and were spared
    /// by no clause (spec 097 §3.7). The tool has found a spelling it has no
    /// rule for, and the consumer refuses on it: exit 1, with this list.
    #[serde(default)]
    pub leftover: Vec<Leftover>,
}

/// An occurrence §3.7 cannot account for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leftover {
    pub rel_path: String,
    pub line: usize,
    pub text: String,
    /// The retired path still present in `text`.
    pub path: String,
}

impl Compaction {
    /// Total rewrites across every file.
    pub fn rewrite_count(&self) -> usize {
        self.rewrites.iter().map(|f| f.rewrites.len()).sum()
    }
}

/// Where a mapped id resolves to.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Target {
    /// The full id to write instead.
    id: String,
    /// The ordinal to write instead, for a citation that carries no slug.
    ordinal: String,
    removed: bool,
}

/// Parse an authored plan (YAML).
pub fn parse_plan(src: &str) -> Result<CompactPlan, Error> {
    serde_yaml::from_str(src).map_err(|e| Error::Config(format!("compact plan: {e}")))
}

/// Compact `repo_root`'s corpus under `plan`, returning the rewritten files as
/// data. Reads; never writes (spec 096 §3.1).
pub fn compact(cfg: &Config, repo_root: &Path, plan: &CompactPlan) -> Result<Compaction, Error> {
    let corpus = read_corpus(cfg, repo_root)?;
    let map = build_map(&corpus, plan)?;
    // Spec 097 §3.1 and §3.7: every retirement is validated before anything is
    // rewritten, including the human acknowledgement an approved spec needs.
    if !plan.retire.is_empty() {
        let (corpus_ids, approved) = corpus_and_approved(cfg, repo_root)?;
        validate_retire(&plan.retire, repo_root, &corpus_ids, &approved)?;
    }

    // The rewrite is a single simultaneous pass per file: a sequential
    // replace-per-key would apply one key's output to the next key's input,
    // which is how a renumber double-shifts (defect 3's shape, one level up).
    // A renamed spec directory moves file by file, and only the files this
    // rewrite can carry. Anything else in it would be left behind at the old
    // path while the `spec.md` moved, splitting one spec across two
    // directories; the refusal names the file rather than letting that happen
    // quietly.
    refuse_uncarryable_in_renamed_dirs(cfg, repo_root, &map)?;

    let keys = sorted_keys(&map);
    let mut files = Vec::new();
    let mut rewrites = Vec::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for form in [
        Form::FullId,
        Form::Citation,
        Form::ShortIdArg,
        Form::RetiredPath,
    ] {
        counts.insert(form.as_str().to_string(), 0);
    }

    let removed_ids: BTreeSet<&str> = map
        .iter()
        .filter(|(_, t)| t.removed)
        .map(|(k, _)| k.as_str())
        .collect();
    let mut removed_paths = Vec::new();
    let mut skipped: Vec<Skipped> = Vec::new();
    let mut examined: Vec<(String, String)> = Vec::new();

    for rel in scannable_files(cfg, repo_root) {
        let Ok(contents) = std::fs::read_to_string(repo_root.join(&rel)) else {
            continue;
        };
        // A removed spec's own document leaves the tree rather than being
        // rewritten: it is not a citation of itself.
        if let Some(id) = spec_id_of_path(cfg, &rel)
            && removed_ids.contains(id.as_str())
        {
            let dir = format!("{}/{id}", cfg.layout.specs_dir.trim_end_matches('/'));
            if !removed_paths.contains(&dir) {
                removed_paths.push(dir);
            }
            continue;
        }
        let (new_contents, mut file_rewrites) = rewrite_file(&contents, &map, &keys, plan);
        let new_contents = if plan.retire.is_empty() {
            new_contents
        } else {
            // §3.3 FIRST: the frontmatter edit, then the form rewrite. In the
            // other order the `path` form rewrites `path: "rules/"` in place,
            // the unit matcher then finds nothing, and the action silently does
            // not fire: a withdrawal leaves the claim standing and a retarget
            // writes a path the `to` never named.
            let staged = match spec_id_of_path(cfg, &rel) {
                Some(id) => {
                    let (edited, unit_rewrites) =
                        apply_unit_actions(&id, &new_contents, &plan.retire);
                    file_rewrites.extend(unit_rewrites);
                    edited
                }
                None => new_contents,
            };
            let (after, retired, mut skips) = retire_file(&rel, &staged, &plan.retire);
            file_rewrites.extend(retired);
            skipped.append(&mut skips);
            after
        };
        let to = target_path(cfg, &rel, &map);
        // Every file the rewrite READ, with its final contents. §3.7's leftover
        // scan reads this rather than the emitted set: a file no rule touched is
        // precisely where an unaccounted occurrence hides, and it is not in the
        // emitted set at all.
        if !plan.retire.is_empty() {
            examined.push((rel.clone(), new_contents.clone()));
        }
        if file_rewrites.is_empty() && to == rel {
            continue;
        }
        for r in &file_rewrites {
            *counts.get_mut(r.form.as_str()).expect("form counted") += 1;
        }
        if !file_rewrites.is_empty() {
            rewrites.push(FileRewrites {
                rel_path: rel.clone(),
                rewrites: file_rewrites,
            });
        }
        files.push(CompactFile {
            rel_path: to,
            from_rel_path: rel,
            contents: new_contents,
        });
    }

    let leftover = unaccounted(&examined, &skipped, &plan.retire);
    let entries = map_entries(&map);
    let map_document = render_map(&entries);
    Ok(Compaction {
        files,
        removed_paths,
        map: entries,
        rewrites,
        counts,
        map_document,
        leftover,
        skipped,
    })
}

/// Every occurrence of a retired path still in the rewritten files that no skip
/// clause accounts for (spec 097 §3.7).
///
/// A form the rules do not cover leaves the path in place, and the first build
/// left it there in silence: the plan looked applied, the gate was green, and
/// the corpus still named a path that was gone. Read from the OUTPUT, so it
/// cannot be fooled by a rule that fired and then missed.
fn unaccounted(
    examined: &[(String, String)],
    skipped: &[Skipped],
    entries: &[RetireEntry],
) -> Vec<Leftover> {
    let mut out = Vec::new();
    for (rel, contents) in examined {
        for (n, line) in contents.split_inclusive('\n').enumerate() {
            for e in entries {
                if !line.contains(&e.path) {
                    continue;
                }
                let spared = skipped
                    .iter()
                    .any(|s| &s.rel_path == rel && s.line == n + 1);
                if spared {
                    continue;
                }
                out.push(Leftover {
                    rel_path: rel.clone(),
                    line: n + 1,
                    text: line.trim_end().to_string(),
                    path: e.path.clone(),
                });
            }
        }
    }
    out
}

/// Every spec id in the corpus, and the subset that is `approved`, from ONE
/// compile. Spec 097 §3.3 asks two questions of a named unit, "is the spec
/// there" and "does changing it need a human", and asking them separately cost
/// two compiles of the whole corpus for one validation.
///
/// `approved` is matched on the enum, never on a formatted string. The first
/// build compared `format!("{:?}", status)` to `"approved"`, and `Debug` is not
/// a stability contract: a rename or a variant with data would silently make
/// every approved spec editable without the acknowledgement, which is the one
/// thing this rule exists to demand.
fn corpus_and_approved(
    cfg: &Config,
    repo_root: &Path,
) -> Result<(BTreeSet<String>, BTreeSet<String>), Error> {
    let outcome = crate::compile::compile(cfg, repo_root)?;
    let mut all = BTreeSet::new();
    let mut approved = BTreeSet::new();
    for s in &outcome.registry.specs {
        all.insert(s.id.clone());
        if matches!(s.status, spec_spine_types::Status::Approved) {
            approved.insert(s.id.clone());
        }
    }
    Ok((all, approved))
}

// ── the corpus ───────────────────────────────────────────────────────────────

/// Every spec id in the corpus, in ordinal order, with its directory.
fn read_corpus(cfg: &Config, repo_root: &Path) -> Result<Vec<String>, Error> {
    // §3.2 before §3.8: an ordinal carried by two specs is refused by NAME
    // here. The compile below refuses the same corpus, but as a generic
    // validation failure, and defect 1 was precisely a reader who could not
    // see which two ids were in contention.
    refuse_ordinal_collision(&list_spec_dirs(cfg, repo_root))?;
    // Spec 096 §3.8: a rewrite of a corpus whose ids are not yet valid produces
    // ids that are differently invalid, so the compile has to pass first.
    let outcome = crate::compile::compile(cfg, repo_root)?;
    if !outcome.validation_passed {
        return Err(Error::Config(
            "compact: the corpus does not validate, so it cannot be rewritten (run `spec-spine compile` for the violations)".into(),
        ));
    }
    let mut ids: Vec<String> = outcome
        .registry
        .specs
        .iter()
        .map(|s| s.id.clone())
        .collect();
    ids.sort();
    Ok(ids)
}

/// The spec directory names under the corpus root, sorted. Read from the tree
/// rather than from a compile, because the collision check runs first.
fn list_spec_dirs(cfg: &Config, repo_root: &Path) -> Vec<String> {
    let dir = repo_root.join(cfg.layout.specs_dir.trim_end_matches('/'));
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = entries
        .flatten()
        // A directory with no `spec.md` is not a spec: an emptied directory
        // left behind by a previous apply must not read as a second carrier of
        // its ordinal.
        .filter(|e| e.path().join("spec.md").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.len() >= 4 && n.as_bytes()[..3].iter().all(u8::is_ascii_digit))
        .collect();
    ids.sort();
    ids
}

/// §3.2: two specs sharing an ordinal is an ambiguity in an identifier, and
/// last-write-wins is never the right answer to one.
fn refuse_ordinal_collision(ids: &[String]) -> Result<(), Error> {
    let mut by_ordinal: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for id in ids {
        by_ordinal.entry(&id[..3]).or_default().push(id);
    }
    if let Some((ord, ids)) = by_ordinal.iter().find(|(_, ids)| ids.len() > 1) {
        return Err(Error::Config(format!(
            "compact: ordinal {ord} is carried by more than one spec: {}",
            ids.join(", ")
        )));
    }
    Ok(())
}

/// The spec id a repo-relative path belongs to, if it is inside the corpus.
fn spec_id_of_path(cfg: &Config, rel: &str) -> Option<String> {
    let specs = format!("{}/", cfg.layout.specs_dir.trim_end_matches('/'));
    let rest = rel.strip_prefix(&specs)?;
    let id = rest.split('/').next()?;
    if id.len() >= 4 && id.as_bytes()[..3].iter().all(u8::is_ascii_digit) {
        Some(id.to_string())
    } else {
        None
    }
}

/// Where a file lands after the rewrite: a spec directory follows its id.
fn target_path(cfg: &Config, rel: &str, map: &BTreeMap<String, Target>) -> String {
    let Some(id) = spec_id_of_path(cfg, rel) else {
        return rel.to_string();
    };
    match map.get(&id) {
        Some(t) if !t.removed && t.id != id => {
            let specs = format!("{}/", cfg.layout.specs_dir.trim_end_matches('/'));
            let rest = rel[specs.len() + id.len()..].to_string();
            format!("{specs}{}{rest}", t.id)
        }
        _ => rel.to_string(),
    }
}

/// The files a rewrite may touch. The derived root, the state root, `.git` and
/// `resolver_exclusions` are already pruned by the walk; what is added here is
/// an extension filter, because a rewrite of a binary or a lockfile is never
/// what the plan meant.
fn scannable_files(cfg: &Config, repo_root: &Path) -> Vec<String> {
    const EXTS: &[&str] = &[
        "md", "rs", "toml", "yml", "yaml", "sh", "py", "js", "ts", "tsx", "json", "txt",
    ];
    const NAMES: &[&str] = &["Makefile", "AGENTS.md", "CLAUDE.md"];
    crate::coverage::walk_repository(cfg, repo_root)
        .into_iter()
        .filter(|rel| {
            if rel.ends_with("Cargo.lock") || rel.ends_with("package-lock.json") {
                return false;
            }
            let name = rel.rsplit('/').next().unwrap_or(rel);
            NAMES.contains(&name)
                || rel
                    .rsplit_once('.')
                    .is_some_and(|(_, ext)| EXTS.contains(&ext))
        })
        .collect()
}

// ── the map ──────────────────────────────────────────────────────────────────

/// Compute the map, keyed on the **full id** and never on the ordinal (spec 096
/// §3.2). Defect 1 was a last-write-wins on an ordinal collision.
fn build_map(corpus: &[String], plan: &CompactPlan) -> Result<BTreeMap<String, Target>, Error> {
    let present: BTreeSet<&str> = corpus.iter().map(String::as_str).collect();

    refuse_ordinal_collision(corpus)?;

    // §3.1: refuse a plan naming a spec the corpus does not have, before
    // anything is rewritten.
    let mut removed: BTreeMap<&str, &str> = BTreeMap::new();
    for entry in &plan.remove {
        if !present.contains(entry.spec.as_str()) {
            return Err(Error::Config(format!(
                "compact: the plan removes `{}`, which the corpus does not have",
                entry.spec
            )));
        }
        if !present.contains(entry.answered_by.as_str()) {
            return Err(Error::Config(format!(
                "compact: `{}` is answered by `{}`, which the corpus does not have",
                entry.spec, entry.answered_by
            )));
        }
        removed.insert(&entry.spec, &entry.answered_by);
    }
    for (spec, answered_by) in &removed {
        if removed.contains_key(answered_by) {
            return Err(Error::Config(format!(
                "compact: `{spec}` is answered by `{answered_by}`, which the same plan removes"
            )));
        }
    }

    // Survivors keep their order, so a spec filed earlier keeps a lower ordinal.
    let survivors: Vec<&String> = corpus
        .iter()
        .filter(|id| !removed.contains_key(id.as_str()))
        .collect();

    let mut new_id: BTreeMap<&str, String> = BTreeMap::new();
    for (i, id) in survivors.iter().enumerate() {
        let id = id.as_str();
        // `{:03}` pads to at LEAST three digits, so `i >= 1000` would emit a
        // four-digit ordinal. It cannot: `refuse_ordinal_collision` demands a
        // distinct three-digit ordinal per spec, there are exactly 1000 of
        // those, so a corpus reaching this line has at most 1000 specs and `i`
        // at most 999. The guard is that refusal, not a clamp here.
        let renamed = match plan.renumber {
            Renumber::None => id.to_string(),
            Renumber::Contiguous => format!("{:03}{}", i, &id[3..]),
        };
        new_id.insert(id, renamed);
    }

    let mut map = BTreeMap::new();
    for id in &survivors {
        let to = new_id[id.as_str()].clone();
        map.insert(
            (*id).clone(),
            Target {
                ordinal: to[..3].to_string(),
                id: to,
                removed: false,
            },
        );
    }
    for (spec, answered_by) in removed {
        let to = new_id
            .get(answered_by)
            .expect("an answering spec survives")
            .clone();
        map.insert(
            spec.to_string(),
            Target {
                ordinal: to[..3].to_string(),
                id: to,
                removed: true,
            },
        );
    }
    Ok(map)
}

/// A character that can be part of a path segment. `_` and `-` are in it and are
/// not alphanumeric, which is why they are named: a test that asked only
/// `is_ascii_alphanumeric` read `_rules/one.md` as a citation of `rules/one.md`.
fn is_path_char(b: u8) -> bool {
    matches!(b, b'/' | b'.' | b'_' | b'-') || b.is_ascii_alphanumeric()
}

/// Is the byte before a BARE prose occurrence one that disqualifies it?
///
/// A path character means this is a longer path. A quote or a backtick means
/// another form already owns the occurrence: the backticked citation and the
/// quoted `path` form are both replaced before this runs, so matching them
/// again would rewrite one occurrence twice.
fn preceded_by_path_char(bytes: &[u8], at: usize) -> bool {
    at > 0 && (is_path_char(bytes[at - 1]) || matches!(bytes[at - 1], b'`' | b'"' | b'\''))
}

/// Does `line` name `path` as itself, rather than as the head of a longer path?
///
/// Used for a frontmatter unit, where the value IS quoted (`path: "rules/"`), so
/// a quote is a delimiter rather than a disqualifier. `apply_unit_actions` used
/// a bare `contains` and would edit a unit claiming `kit/rules/sub/` when
/// `rules/` retires; this is that test, with the quote rule the other call site
/// needs deliberately left out.
fn names_path(line: &str, path: &str) -> bool {
    let bytes = line.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = line[from..].find(path) {
        let at = from + rel;
        if at == 0 || !is_path_char(bytes[at - 1]) {
            return true;
        }
        from = at + path.len();
    }
    false
}

/// Apply the plan's unit actions to one spec's frontmatter (spec 097 §3.3).
///
/// Line-scoped inside the frontmatter block and inside the named edge's list: a
/// unit is a list entry, and the entry is either dropped (`withdraw`) or has its
/// path rewritten (`retarget`). Where dropping the last entry would leave a key
/// with no list, the key goes too: `establishes:` followed by nothing is not the
/// same document minus a claim, it is a document that no longer parses the way
/// the corpus expects.
fn apply_unit_actions(spec_id: &str, src: &str, entries: &[RetireEntry]) -> (String, Vec<Rewrite>) {
    let actions: Vec<(&RetireEntry, &UnitAction)> = entries
        .iter()
        .flat_map(|e| e.units.iter().map(move |u| (e, u)))
        .filter(|(_, u)| u.spec == spec_id)
        .collect();
    if actions.is_empty() {
        return (src.to_string(), Vec::new());
    }

    let mut out: Vec<String> = Vec::new();
    let mut rewrites = Vec::new();
    let mut edge: Option<String> = None;
    let mut in_frontmatter = false;
    let mut delimiters = 0usize;

    for (n, raw) in src.split_inclusive('\n').enumerate() {
        let line = raw.trim_end_matches(['\n', '\r']);
        if line == "---" {
            delimiters += 1;
            in_frontmatter = delimiters == 1;
            out.push(raw.to_string());
            continue;
        }
        if !in_frontmatter {
            out.push(raw.to_string());
            continue;
        }
        // A key at column zero opens (or closes) an edge list.
        if !line.starts_with(' ') && !line.starts_with('-') && line.contains(':') {
            edge = Some(line.split(':').next().unwrap_or("").to_string());
            out.push(raw.to_string());
            continue;
        }
        let Some(current) = edge.as_deref() else {
            out.push(raw.to_string());
            continue;
        };
        let mut emitted = false;
        for (entry, action) in &actions {
            if action.edge != current || !names_path(line, &entry.path) {
                continue;
            }
            match action.action {
                UnitActionKind::Withdraw => {
                    rewrites.push(Rewrite {
                        line: n + 1,
                        old: line.to_string(),
                        new: String::new(),
                        form: Form::RetiredPath,
                    });
                }
                UnitActionKind::Retarget => {
                    let to = action.to.as_deref().unwrap_or_default();
                    let replaced = line.replace(&entry.path, to);
                    rewrites.push(Rewrite {
                        line: n + 1,
                        old: line.to_string(),
                        new: replaced.clone(),
                        form: Form::RetiredPath,
                    });
                    out.push(format!("{replaced}\n"));
                }
            }
            emitted = true;
            break;
        }
        if !emitted {
            out.push(raw.to_string());
        }
    }

    (drop_empty_edge_keys(&out.concat()), rewrites)
}

/// Remove an edge key whose list a withdrawal emptied.
fn drop_empty_edge_keys(src: &str) -> String {
    let lines: Vec<&str> = src.split_inclusive('\n').collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let mut delimiters = 0usize;
    while i < lines.len() {
        let line = lines[i].trim_end_matches(['\n', '\r']);
        if line == "---" {
            delimiters += 1;
        }
        let in_frontmatter = delimiters == 1 && line != "---";
        let is_key = in_frontmatter
            && !line.starts_with(' ')
            && !line.starts_with('-')
            && line.ends_with(':');
        if is_key {
            // Look past blank lines: a key separated from its first item by
            // one is still a key with items, and dropping it would delete a
            // live claim.
            let next = lines[i + 1..]
                .iter()
                .map(|l| l.trim_end_matches(['\n', '\r']))
                .find(|l| !l.trim().is_empty());
            // An INDENTED `-` is a list item. The closing `---` also starts
            // with one, and reading it as an item kept every emptied key.
            let has_items = next.is_some_and(|l| {
                l != "---" && l.starts_with(' ') && l.trim_start().starts_with('-')
            });
            if !has_items {
                i += 1;
                continue;
            }
        }
        out.push_str(lines[i]);
        i += 1;
    }
    out
}

/// Refuse a renamed spec directory holding a file the rewrite cannot carry.
///
/// `scannable_files` is an extension filter, so a binary or an image inside a
/// spec directory is invisible to the walk: the `spec.md` would move and it
/// would not. A removed spec is unaffected, because its whole directory goes.
fn refuse_uncarryable_in_renamed_dirs(
    cfg: &Config,
    repo_root: &Path,
    map: &BTreeMap<String, Target>,
) -> Result<(), Error> {
    let specs = cfg.layout.specs_dir.trim_end_matches('/');
    let carried: BTreeSet<String> = scannable_files(cfg, repo_root).into_iter().collect();
    for (id, target) in map {
        if target.removed || &target.id == id {
            continue;
        }
        let dir = repo_root.join(specs).join(id);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                continue;
            }
            let rel = format!("{specs}/{id}/{}", entry.file_name().to_string_lossy());
            if !carried.contains(&rel) {
                return Err(Error::Config(format!(
                    "compact: `{id}` is renumbered to `{}`, and `{rel}` is a file this rewrite \
                     cannot carry, so the spec would be split across two directories. Move or \
                     remove it first.",
                    target.id
                )));
            }
        }
    }
    Ok(())
}

/// Keys longest first, so no key shadows a longer one (spec 096 §3.3 form 1).
fn sorted_keys(map: &BTreeMap<String, Target>) -> Vec<String> {
    let mut keys: Vec<String> = map.keys().cloned().collect();
    keys.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    keys
}

fn map_entries(map: &BTreeMap<String, Target>) -> Vec<MapEntry> {
    map.iter()
        .map(|(from, t)| MapEntry {
            from: from.clone(),
            to: t.id.clone(),
            removed: t.removed,
        })
        .collect()
}

/// The map document spec 095 §3.4 establishes and spec 096 §3.7 emits.
fn render_map(entries: &[MapEntry]) -> String {
    let mut s = String::from(
        "# Corpus map\n\nGenerated by `spec-spine compact`. Every id this corpus \
         has carried, with the\nid that answers for it now.\n\n| Was | Is |\n|---|---|\n",
    );
    for e in entries {
        let note = if e.removed { " (removed)" } else { "" };
        s.push_str(&format!("| `{}` | `{}`{note} |\n", e.from, e.to));
    }
    s
}

// ── the rewrite ──────────────────────────────────────────────────────────────

/// Every match in one file, found against the ORIGINAL text and applied once.
fn rewrite_file(
    src: &str,
    map: &BTreeMap<String, Target>,
    keys: &[String],
    plan: &CompactPlan,
) -> (String, Vec<Rewrite>) {
    let mut hits: Vec<(usize, usize, String, Form)> = Vec::new();
    scan_full_ids(src, map, keys, &mut hits);
    scan_citations(src, map, plan, &mut hits);
    scan_short_id_args(src, map, &mut hits);

    // Earlier first; where two rules reach the same bytes, the longer match
    // wins, which is form 1 over form 2 inside a full id (defect 3).
    hits.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| (b.1 - b.0).cmp(&(a.1 - a.0))));
    let mut out = String::with_capacity(src.len());
    let mut rewrites = Vec::new();
    let mut cursor = 0usize;
    for (start, end, new, form) in hits {
        if start < cursor {
            continue; // overlapped by a longer, earlier match
        }
        let old = &src[start..end];
        if old == new {
            continue;
        }
        out.push_str(&src[cursor..start]);
        out.push_str(&new);
        rewrites.push(Rewrite {
            line: src[..start].matches('\n').count() + 1,
            old: old.to_string(),
            new,
            form,
        });
        cursor = end;
    }
    out.push_str(&src[cursor..]);
    (out, rewrites)
}

fn is_id_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

/// Form 1: a full id, longest key first.
fn scan_full_ids(
    src: &str,
    map: &BTreeMap<String, Target>,
    keys: &[String],
    hits: &mut Vec<(usize, usize, String, Form)>,
) {
    let b = src.as_bytes();
    let mut i = 0usize;
    'outer: while i < b.len() {
        if b[i].is_ascii_digit() && (i == 0 || !is_id_char(b[i - 1])) {
            for key in keys {
                let end = i + key.len();
                if end <= b.len() && &src[i..end] == key.as_str() {
                    let after_ok = end == b.len() || !is_id_char(b[end]);
                    if after_ok {
                        hits.push((i, end, map[key].id.clone(), Form::FullId));
                        i = end;
                        continue 'outer;
                    }
                }
            }
        }
        i += 1;
    }
}

/// The comment or quote prefix a wrapped line may carry before the ordinal.
const CONTINUATIONS: &[&str] = &["///", "//!", "//", "#", "*", ">"];

/// Skip inter-token space, including **one** line break with an optional
/// comment continuation prefix. Returns `None` when more than one line break is
/// crossed: two blank lines is a new paragraph, not a wrapped sentence.
///
/// Every byte this steps over is ASCII (space, tab, CR, LF) and the caller
/// enters on a character boundary, so the index returned is always one too: a
/// multi-byte character ends the walk at its LEADING byte, which is a boundary.
/// That is why the `src[i..]` below cannot split a character.
fn skip_gap(src: &str, mut i: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut newlines = 0;
    while i < b.len() {
        match b[i] {
            b' ' | b'\t' | b'\r' => i += 1,
            b'\n' => {
                newlines += 1;
                if newlines > 1 {
                    return None;
                }
                i += 1;
                while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
                    i += 1;
                }
                if let Some(p) = CONTINUATIONS
                    .iter()
                    .find(|p| src[i..].starts_with(**p))
                    .copied()
                {
                    i += p.len();
                }
            }
            _ => break,
        }
    }
    Some(i)
}

/// A three-digit ordinal at `i` that is not part of a longer token.
fn ordinal_at(src: &str, i: usize) -> Option<&str> {
    let b = src.as_bytes();
    if i + 3 > b.len() || !b[i..i + 3].iter().all(u8::is_ascii_digit) {
        return None;
    }
    if i + 3 < b.len() && is_id_char(b[i + 3]) {
        // A `-` or a word character means these digits open a full id, which
        // form 1 has already handled: matching them again is defect 3.
        return None;
    }
    if i > 0 && is_id_char(b[i - 1]) {
        return None;
    }
    Some(&src[i..i + 3])
}

/// Form 2: `spec NNN` / `specs NNN` in prose, the citation possibly broken
/// across a line and possibly naming more than one ordinal.
fn scan_citations(
    src: &str,
    map: &BTreeMap<String, Target>,
    plan: &CompactPlan,
    hits: &mut Vec<(usize, usize, String, Form)>,
) {
    let by_ordinal: BTreeMap<&str, &Target> = map.iter().map(|(k, t)| (&k[..3], t)).collect();
    let lower = src.to_ascii_lowercase();
    let b = src.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = lower[from..].find("spec") {
        let kw = from + rel;
        from = kw + 4;
        // Word-bounded, and `specs` is the only accepted plural.
        if kw > 0 && is_id_char(b[kw - 1]) {
            continue;
        }
        let mut after = kw + 4;
        if after < b.len() && (b[after] == b's' || b[after] == b'S') {
            after += 1;
        }
        if after < b.len() && is_id_char(b[after]) {
            continue;
        }
        // Not this corpus's ordinals when another project's name precedes.
        let before = src[..kw].trim_end();
        let prev = before
            .rsplit(|c: char| c.is_whitespace())
            .next()
            .unwrap_or("");
        let prev = prev.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        if plan
            .foreign_projects
            .iter()
            .any(|p| p.eq_ignore_ascii_case(prev))
        {
            continue;
        }
        // The first ordinal, then every further one joined by a separator.
        let Some(mut pos) = skip_gap(src, after) else {
            continue;
        };
        // Each ordinal in a list is decided on its own: an unmapped one is left
        // alone and the walk CONTINUES, so `spec 777/002` rewrites `002` and
        // leaves `777`. A list is a list of citations, not one citation whose
        // first member speaks for the rest.
        loop {
            let Some(ord) = ordinal_at(src, pos) else {
                break;
            };
            if let Some(t) = by_ordinal.get(ord) {
                hits.push((pos, pos + 3, t.ordinal.clone(), Form::Citation));
            }
            let next = pos + 3;
            let Some(sep_end) = citation_separator(src, next) else {
                break;
            };
            pos = sep_end;
        }
    }
}

/// A separator that continues a citation onto a further ordinal: `/`, `,`,
/// `and`, `or`, `&`, with surrounding space and at most one line break.
fn citation_separator(src: &str, i: usize) -> Option<usize> {
    let after_space = skip_gap(src, i)?;
    let rest = &src[after_space..];
    for sep in ["/", ",", "&", "and", "or"] {
        if rest.starts_with(sep) {
            let j = after_space + sep.len();
            // A word separator must itself be word-bounded.
            if sep.len() > 1 && j < src.len() && is_id_char(src.as_bytes()[j]) {
                continue;
            }
            return skip_gap(src, j);
        }
    }
    None
}

/// The commands that take a spec id positionally or after `--spec`.
const ID_COMMANDS: &[&str] = &[
    "registry show",
    "registry relationships",
    "index owner",
    "verify-attestation --spec",
    "attest --spec",
    "compile --spec",
    "verify",
    "delta",
];

/// Form 3: a bare short id as a command argument (defect 2). Line-scoped, and
/// never applied to a command carrying `--repo`, which addresses a fixture
/// corpus whose ids are its own.
fn scan_short_id_args(
    src: &str,
    map: &BTreeMap<String, Target>,
    hits: &mut Vec<(usize, usize, String, Form)>,
) {
    let by_ordinal: BTreeMap<&str, &Target> = map.iter().map(|(k, t)| (&k[..3], t)).collect();
    let mut offset = 0usize;
    for line in src.split_inclusive('\n') {
        let base = offset;
        offset += line.len();
        if line.contains("--repo") {
            continue;
        }
        for cmd in ID_COMMANDS {
            let mut from = 0usize;
            while let Some(rel) = line[from..].find(cmd) {
                let at = from + rel;
                from = at + cmd.len();
                // `verify` must not match inside `verify-attestation`.
                let after_cmd = at + cmd.len();
                if after_cmd < line.len() && is_id_char(line.as_bytes()[after_cmd]) {
                    continue;
                }
                let mut pos = after_cmd;
                // Flags between the verb and its argument.
                loop {
                    while pos < line.len() && (line.as_bytes()[pos] as char).is_whitespace() {
                        pos += 1;
                    }
                    if line[pos..].starts_with("--") {
                        while pos < line.len() && !(line.as_bytes()[pos] as char).is_whitespace() {
                            pos += 1;
                        }
                    } else {
                        break;
                    }
                }
                if let Some(ord) = ordinal_at(line, pos)
                    && let Some(t) = by_ordinal.get(ord)
                {
                    hits.push((
                        base + pos,
                        base + pos + 3,
                        t.ordinal.clone(),
                        Form::ShortIdArg,
                    ));
                }
            }
        }
    }
}

// ── spec 097: a path leaves the corpus the way a spec does ───────────────────

/// Whether a retired path names a file or a subtree (spec 097 §3.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetireKind {
    #[default]
    File,
    Directory,
}

/// What happens to a frontmatter unit naming a retired path (spec 097 §3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitActionKind {
    /// The unit leaves the spec's frontmatter.
    Withdraw,
    /// The unit points at `to` instead.
    Retarget,
}

/// A named change to one spec's frontmatter (spec 097 §3.3). Named per spec and
/// per edge, never inferred: there is no grammar in this corpus for withdrawing
/// a claim, and the only route is an edit to the owning spec's own frontmatter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitAction {
    pub spec: String,
    pub edge: String,
    pub action: UnitActionKind,
    #[serde(default)]
    pub to: Option<String>,
    /// A human's acknowledgement that this changes an `approved` spec. The flag
    /// is a human's to write, the way a `Spec-Drift-Waiver` is.
    #[serde(default)]
    pub acknowledge_approved: bool,
}

/// One path leaving the corpus (spec 097 §3.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetireEntry {
    pub path: String,
    #[serde(default)]
    pub kind: RetireKind,
    /// A replacement per FORM, not per path: in the retirement this was
    /// measured on, four files became one section each of another document,
    /// the directory became a phrase, and the glob became nothing at all.
    ///
    /// Keys are `citation`, `glob` and `path` (§3.2 and D-2). A `null` value is
    /// a deletion the plan states; an absent key is a form with no rule, which
    /// §3.1 refuses rather than silently skipping.
    #[serde(default)]
    pub forms: BTreeMap<String, Option<String>>,
    #[serde(default)]
    pub units: Vec<UnitAction>,
    /// Files whose occurrences are history rather than citations (§3.5).
    #[serde(default)]
    pub historical_files: Vec<String>,
    /// Headings under which an occurrence is history rather than a citation.
    #[serde(default)]
    pub historical_sections: Vec<String>,
}

/// Why an occurrence was left alone (spec 097 §3.5). Reported, never silent:
/// the eleven survivors of the retirement this was measured on were found by
/// reading a `grep`, and a tool that keeps them without saying so has only
/// moved the reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkipClause {
    /// The line negates the path: `! test -e`, `! grep`, `assert!(!`, `MUST NOT`.
    Negation,
    /// The occurrence sits under a heading the plan calls historical.
    HistoricalSection,
    /// The occurrence sits in a file the plan calls historical.
    HistoricalFile,
}

impl SkipClause {
    pub fn as_str(self) -> &'static str {
        match self {
            SkipClause::Negation => "negation",
            SkipClause::HistoricalSection => "historical-section",
            SkipClause::HistoricalFile => "historical-file",
        }
    }
}

/// One occurrence the rewrite deliberately did not touch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    pub rel_path: String,
    pub line: usize,
    pub text: String,
    pub clause: SkipClause,
}

/// The forms a path is spelled in (spec 097 §3.2).
const RETIRE_FORMS: &[&str] = &["citation", "glob", "path"];

/// Markers that turn an occurrence into a statement about an absence (§3.5).
const NEGATIONS: &[&str] = &[
    "! test ",
    "! grep",
    "!test ",
    "assert!(!",
    "MUST NOT",
    "! [ -",
];

/// Refuse a `retire` entry the rules cannot serve, before anything is rewritten
/// (spec 097 §3.1, §3.7).
fn validate_retire(
    entries: &[RetireEntry],
    repo_root: &Path,
    corpus: &BTreeSet<String>,
    approved: &BTreeSet<String>,
) -> Result<(), Error> {
    for e in entries {
        if !repo_root.join(&e.path).exists() {
            return Err(Error::Config(format!(
                "compact: the plan retires `{}`, which the tree does not have",
                e.path
            )));
        }
        for (form, replacement) in &e.forms {
            if !RETIRE_FORMS.contains(&form.as_str()) {
                return Err(Error::Config(format!(
                    "compact: `{}` declares the form `{form}`, which has no rule; the forms are {}",
                    e.path,
                    RETIRE_FORMS.join(", ")
                )));
            }
            if form == "glob"
                && let Some(text) = replacement
                && !text.ends_with('/')
            {
                return Err(Error::Config(format!(
                    "compact: `{}` replaces the glob form with `{text}`, which is not a directory \
                     prefix. A glob rule substitutes the path PREFIX and leaves the wildcard, so \
                     `rules/*.md` with `{text}` would read `{text}*.md`. End it with `/`, or use \
                     `~` to remove the pattern.",
                    e.path
                )));
            }
            if replacement.is_none() && form != "glob" {
                return Err(Error::Config(format!(
                    "compact: `{}` gives the form `{form}` no replacement; only `glob` may be removed outright",
                    e.path
                )));
            }
        }
        for u in &e.units {
            if u.action == UnitActionKind::Retarget && u.to.is_none() {
                return Err(Error::Config(format!(
                    "compact: `{}` retargets `{}`'s {} unit with no `to`",
                    e.path, u.spec, u.edge
                )));
            }
            if !corpus.contains(&u.spec) {
                return Err(Error::Config(format!(
                    "compact: `{}` names a unit on `{}`, which the corpus does not have",
                    e.path, u.spec
                )));
            }
            // A `to` on a withdrawal is discarded, and a plan whose author wrote
            // one meant something the tool will not do.
            if u.action == UnitActionKind::Withdraw && u.to.is_some() {
                return Err(Error::Config(format!(
                    "compact: `{}` withdraws `{}`'s {} unit and also names a `to`; a withdrawal \
                     has no target. Drop the `to`, or make it a retarget.",
                    e.path, u.spec, u.edge
                )));
            }
            if approved.contains(&u.spec) && !u.acknowledge_approved {
                return Err(Error::Config(format!(
                    "compact: `{}` changes the frontmatter of `{}`, which is approved. \
                     Withdrawing or retargeting a unit on an approved spec is a human's \
                     decision: add `acknowledge_approved: true` to that entry, or remove it.",
                    e.path, u.spec
                )));
            }
        }
    }
    Ok(())
}

/// Apply every retirement to one file's text, returning the rewrites and the
/// occurrences deliberately left alone.
fn retire_file(
    rel: &str,
    src: &str,
    entries: &[RetireEntry],
) -> (String, Vec<Rewrite>, Vec<Skipped>) {
    let mut out = String::with_capacity(src.len());
    let mut rewrites = Vec::new();
    let mut skipped = Vec::new();
    let mut heading = String::new();

    for (n, line) in src.split_inclusive('\n').enumerate() {
        if line.trim_start().starts_with('#') && line.contains(' ') {
            heading = line.trim().trim_start_matches('#').trim().to_string();
        }
        let mut current = line.to_string();
        // A line spared by one entry is spared, full stop. Advancing to the
        // next entry let it rewrite a line the report had already called left
        // alone, so the report described a file that was not the one emitted.
        let mut spared: Option<SkipClause> = None;
        for e in entries {
            if !current.contains(&e.path) {
                continue;
            }
            // The line is already spared by an earlier entry. This entry's
            // occurrence on it is spared too, and is RECORDED: "reported, never
            // silent" is a claim about occurrences, and a second path sharing a
            // line with a negation was previously left out of the report
            // entirely.
            if let Some(clause) = spared {
                skipped.push(Skipped {
                    rel_path: rel.to_string(),
                    line: n + 1,
                    text: current.trim_end().to_string(),
                    clause,
                });
                continue;
            }
            if e.historical_files.iter().any(|f| f == rel) {
                skipped.push(Skipped {
                    rel_path: rel.to_string(),
                    line: n + 1,
                    text: current.trim_end().to_string(),
                    clause: SkipClause::HistoricalFile,
                });
                spared = Some(SkipClause::HistoricalFile);
                continue;
            }
            if e.historical_sections
                .iter()
                .any(|h| heading.contains(h.as_str()))
            {
                skipped.push(Skipped {
                    rel_path: rel.to_string(),
                    line: n + 1,
                    text: current.trim_end().to_string(),
                    clause: SkipClause::HistoricalSection,
                });
                spared = Some(SkipClause::HistoricalSection);
                continue;
            }
            if NEGATIONS.iter().any(|m| current.contains(m)) {
                skipped.push(Skipped {
                    rel_path: rel.to_string(),
                    line: n + 1,
                    text: current.trim_end().to_string(),
                    clause: SkipClause::Negation,
                });
                spared = Some(SkipClause::Negation);
                continue;
            }
            let before = current.clone();
            current = apply_forms(&current, e);
            if current != before {
                rewrites.push(Rewrite {
                    line: n + 1,
                    old: before.trim_end().to_string(),
                    new: current.trim_end().to_string(),
                    form: Form::RetiredPath,
                });
            }
        }
        out.push_str(&current);
    }
    (out, rewrites, skipped)
}

/// The six spellings of §3.2, reduced to the three that take a replacement
/// (D-2): a glob line, a backticked or bare prose citation, and a path in a
/// YAML value, a shell word or a Rust string literal.
fn apply_forms(line: &str, e: &RetireEntry) -> String {
    let mut s = line.to_string();
    // A glob: the path followed by a wildcard segment. Removing it removes the
    // whole list entry, because a pattern matching nothing reads like a claim
    // being hashed and hashes nothing.
    if let Some(rule) = e.forms.get("glob") {
        let is_glob = s.contains(&format!("{}*", e.path))
            || s.contains(&format!("{}/*", e.path.trim_end_matches('/')));
        if is_glob {
            return match rule {
                None => String::new(),
                Some(text) => replace_glob(&s, &e.path, text),
            };
        }
    }
    if let Some(Some(text)) = e.forms.get("citation") {
        s = s.replace(&format!("`{}`", e.path), text);
        s = replace_bare(&s, &e.path, text);
    }
    if let Some(Some(text)) = e.forms.get("path") {
        s = s.replace(&format!("\"{}\"", e.path), &format!("\"{text}\""));
        s = s.replace(&format!("'{}'", e.path), &format!("'{text}'"));
    }
    s
}

fn replace_glob(line: &str, path: &str, text: &str) -> String {
    line.replace(path, text)
}

/// A bare occurrence, bounded by whitespace or by sentence punctuation, and
/// never a substring of a longer path (§3.2 form 2).
fn replace_bare(line: &str, path: &str, text: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find(path) {
        // A path character before the match means this is a DIFFERENT path:
        // `kit/rules/one.md` and `_rules/one.md` are not the retired one.
        let before_ok = !preceded_by_path_char(rest.as_bytes(), at);
        let after = at + path.len();
        // `.` is both sentence punctuation and an extension separator, and the
        // two need different answers: `rules/one.md.` ends a sentence while
        // `rules/one.md.bak` is a different file. A period terminates a citation
        // only when nothing but space follows it.
        let after_ok = match rest.as_bytes().get(after) {
            None => true,
            Some(b'.') => rest.as_bytes()[after + 1..]
                .first()
                .is_none_or(|n| n.is_ascii_whitespace()),
            // `\r` is in the set for a CRLF line: without it a path at the end
            // of one reads as unterminated and is left alone.
            Some(b) => matches!(b, b' ' | b',' | b';' | b':' | b')' | b'\n' | b'\r'),
        };
        out.push_str(&rest[..at]);
        if before_ok && after_ok {
            out.push_str(text);
        } else {
            out.push_str(path);
        }
        rest = &rest[after..];
    }
    out.push_str(rest);
    out
}

// ── fenced blocks (spec 096 §3.5) ────────────────────────────────────────────

/// The byte ranges of every ```verify:cli block body, located by scanning
/// lines: a block that greps for the literal opening fence closes a non-greedy
/// regex early and silently halves the rewrite, which is defect 4.
///
/// No rule in this module is block-scoped, so nothing here calls it: §3.3's
/// form 3 is keyed on the command that takes a spec id, which is a stronger
/// test than "inside a block" and needs no scoping. It is `pub` and asserted
/// because §3.5 requires the locator itself, for the rules that ARE scoped;
/// spec 097's shell-word form is the first of them.
pub fn verify_cli_blocks(src: &str) -> Vec<(usize, usize)> {
    const OPEN: &str = "```verify:cli";
    const CLOSE: &str = "```";
    let mut out = Vec::new();
    let mut offset = 0usize;
    let mut open_at: Option<usize> = None;
    for line in src.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim_end_matches(['\n', '\r']);
        match open_at {
            None => {
                if trimmed == OPEN {
                    open_at = Some(offset);
                }
            }
            Some(body) => {
                // Only a line that is EXACTLY the closing fence closes it.
                if trimmed == CLOSE {
                    out.push((body, start));
                    open_at = None;
                }
            }
        }
    }
    out
}
