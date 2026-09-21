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
}

impl Form {
    pub fn as_str(self) -> &'static str {
        match self {
            Form::FullId => "full-id",
            Form::Citation => "citation",
            Form::ShortIdArg => "short-id-arg",
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

    // The rewrite is a single simultaneous pass per file: a sequential
    // replace-per-key would apply one key's output to the next key's input,
    // which is how a renumber double-shifts (defect 3's shape, one level up).
    let keys = sorted_keys(&map);
    let mut files = Vec::new();
    let mut rewrites = Vec::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for form in [Form::FullId, Form::Citation, Form::ShortIdArg] {
        counts.insert(form.as_str().to_string(), 0);
    }

    let removed_ids: BTreeSet<&str> = map
        .iter()
        .filter(|(_, t)| t.removed)
        .map(|(k, _)| k.as_str())
        .collect();
    let mut removed_paths = Vec::new();

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
        let (new_contents, file_rewrites) = rewrite_file(&contents, &map, &keys, plan);
        let to = target_path(cfg, &rel, &map);
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

    let entries = map_entries(&map);
    let map_document = render_map(&entries);
    Ok(Compaction {
        files,
        removed_paths,
        map: entries,
        rewrites,
        counts,
        map_document,
    })
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
        let mut matched_any = false;
        loop {
            let Some(ord) = ordinal_at(src, pos) else {
                break;
            };
            if let Some(t) = by_ordinal.get(ord) {
                hits.push((pos, pos + 3, t.ordinal.clone(), Form::Citation));
            }
            matched_any = true;
            let next = pos + 3;
            let Some(sep_end) = citation_separator(src, next) else {
                break;
            };
            pos = sep_end;
        }
        let _ = matched_any;
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
