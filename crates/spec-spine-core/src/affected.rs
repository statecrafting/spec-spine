//! The affected-acceptance selection (spec 158): which specs' declared
//! acceptance a change can break.
//!
//! Spec 150 chose the rules and embedded them in a script only this repository
//! carried. Spec 158 moves them here, so every governed repository asks one
//! governed read, and the merge-queue job shards the plan it returns.
//!
//! **Pure.** [`select_affected`] is a function of the configuration, the corpus
//! read as text at the head and the changed paths. It opens no file, runs no
//! `git`, reads no clock and no environment (spec 158 §3.2): the CLI resolves
//! the range, lists the changed paths and reads the corpus at the head, then
//! hands them in. **It runs nothing**: it names plans, the caller executes them.
//!
//! The rules, first match winning, in this order:
//!
//! 1. `select-all`: a changed path matches `[acceptance] select_all_on`
//!    (spec 158 §3.3; spec 150 called it `engine-source` and built it in);
//! 2. `changed-spec`: the spec's own `spec.md` changed;
//! 3. `names-spec`: its effective plan names a changed spec, by full id or by
//!    its three-digit ordinal as a token (150 D-3);
//! 4. `names-path`: its plan contains a changed path as written from the root;
//! 5. `names-test`: its plan runs `cargo test` over a changed test file's crate
//!    and target (150 D-3).
//!
//! A plan that cannot be read is a spec the selector cannot rule out, so the
//! read refuses and names it rather than leave it out (150 D-4).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use spec_spine_types::{Config, Error};

use crate::read::{Versioning, read_document};
use crate::verify::plans_in_corpus;

/// One spec of the corpus as read at the head.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AffectedSpec {
    /// The spec's directory name, `NNN-slug`.
    pub id: String,
    /// The text of its `spec.md`; `None` when the caller could not read it,
    /// which makes the selection refuse and name this spec.
    pub markdown: Option<String>,
}

/// The rule that selected a spec: the first that matched, in declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AffectedRule {
    /// A changed path matched `[acceptance] select_all_on`.
    SelectAll,
    /// The spec's own `spec.md` changed.
    ChangedSpec,
    /// Its plan names a changed spec.
    NamesSpec,
    /// Its plan names a changed path.
    NamesPath,
    /// Its plan runs a changed test file.
    NamesTest,
}

impl AffectedRule {
    /// The token the document and the text form print.
    pub fn as_str(self) -> &'static str {
        match self {
            AffectedRule::SelectAll => "select-all",
            AffectedRule::ChangedSpec => "changed-spec",
            AffectedRule::NamesSpec => "names-spec",
            AffectedRule::NamesPath => "names-path",
            AffectedRule::NamesTest => "names-test",
        }
    }
}

/// One selected spec.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffectedEntry {
    pub id: String,
    pub rule: AffectedRule,
    /// The command lines `verify <id> --plan` prints for this spec at the head.
    pub plan: Vec<String>,
}

/// The answer of [`select_affected`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Affected {
    /// Repository-relative POSIX paths the change touches, sorted, unique.
    pub changed_paths: Vec<String>,
    pub corpus_size: usize,
    /// True when `[acceptance] select_all_on` fired.
    pub select_all: bool,
    /// The changed paths that fired it, sorted; empty when it did not.
    pub select_all_paths: Vec<String>,
    /// One entry per selected spec, in corpus (id) order.
    pub selected: Vec<AffectedEntry>,
}

/// The three commits the CLI resolved, as full SHAs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AffectedCommits {
    pub base: String,
    pub merge_base: String,
    pub head: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AffectedDocument<'a> {
    base: &'a str,
    head: &'a str,
    merge_base: &'a str,
    #[serde(flatten)]
    affected: &'a Affected,
}

/// Select the specs whose acceptance the change can break (spec 158 §3.2).
///
/// `corpus` is every spec at the head; `changed` the paths the change touches,
/// deletions and both sides of a rename included, in any order.
///
/// # Errors
///
/// [`Error::NotFound`] (exit 1) naming the first spec whose `spec.md` the
/// caller could not read, and [`Error::Config`] for a `select_all_on` entry
/// that is not a glob.
pub fn select_affected(
    cfg: &Config,
    corpus: &[AffectedSpec],
    changed: &[String],
) -> Result<Affected, Error> {
    let mut sources: BTreeMap<String, String> = BTreeMap::new();
    for spec in corpus {
        match &spec.markdown {
            Some(md) => {
                sources.insert(spec.id.clone(), md.clone());
            }
            None => {
                return Err(Error::NotFound(format!(
                    "cannot read the plan of {}: its spec.md was not readable at the head, \
                     so the selector cannot rule it out (spec 158 §3.2)",
                    spec.id
                )));
            }
        }
    }
    let plans = plans_in_corpus(&sources);

    let mut paths: Vec<String> = changed.iter().filter(|p| !p.is_empty()).cloned().collect();
    paths.sort();
    paths.dedup();

    let select_all_paths = select_all_paths(cfg, &paths)?;
    let select_all = !select_all_paths.is_empty();

    let specs_prefix = format!("{}/", cfg.layout.specs_dir.trim_matches('/'));
    let changed_specs: Vec<&String> = sources
        .keys()
        .filter(|id| paths.contains(&format!("{specs_prefix}{id}/spec.md")))
        .collect();
    let named_specs: Vec<(&str, &str)> = changed_specs
        .iter()
        .map(|id| (id.as_str(), id.split('-').next().unwrap_or(id.as_str())))
        .collect();
    let tests = changed_tests(&paths);

    let mut selected = Vec::new();
    for (id, plan) in &plans {
        let text: String = plan.commands.iter().map(|c| format!("{c}\n")).collect();
        let rule = if select_all {
            Some(AffectedRule::SelectAll)
        } else if changed_specs.contains(&id) {
            Some(AffectedRule::ChangedSpec)
        } else if named_specs
            .iter()
            .any(|(full, ordinal)| text.contains(full) || has_token(&text, ordinal))
        {
            Some(AffectedRule::NamesSpec)
        } else if paths.iter().any(|p| text.contains(p.as_str())) {
            Some(AffectedRule::NamesPath)
        } else if plan.commands.iter().any(|line| {
            tests
                .iter()
                .any(|(krate, target)| runs_test(line, krate, target.as_deref()))
        }) {
            Some(AffectedRule::NamesTest)
        } else {
            None
        };
        if let Some(rule) = rule {
            selected.push(AffectedEntry {
                id: id.clone(),
                rule,
                plan: plan.commands.clone(),
            });
        }
    }

    Ok(Affected {
        changed_paths: paths,
        corpus_size: sources.len(),
        select_all,
        select_all_paths,
        selected,
    })
}

/// The read document (spec 158 §3.1): the commits, the change and the
/// selection, on the read axis. Deterministic: the same inputs give the same
/// bytes.
pub fn affected_document(affected: &Affected, commits: &AffectedCommits) -> Result<String, Error> {
    read_document(
        &AffectedDocument {
            base: &commits.base,
            head: &commits.head,
            merge_base: &commits.merge_base,
            affected,
        },
        Versioning::Stamp,
    )
}

/// The changed paths a `select_all_on` glob matches. `*` and `?` stop at `/`;
/// `**` spans directories.
fn select_all_paths(cfg: &Config, paths: &[String]) -> Result<Vec<String>, Error> {
    let options = glob::MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };
    let mut patterns = Vec::new();
    for raw in &cfg.acceptance.select_all_on {
        patterns.push(glob::Pattern::new(raw).map_err(|e| {
            Error::Config(format!(
                "[acceptance] select_all_on entry '{raw}' is not a glob: {e}"
            ))
        })?);
    }
    Ok(paths
        .iter()
        .filter(|p| patterns.iter().any(|pat| pat.matches_with(p, options)))
        .cloned()
        .collect())
}

/// Changed files under `crates/<crate>/tests/`, as `(crate, target)`.
/// `tests/<name>.rs` is the integration test target `<name>`; anything else
/// there (a fixture, a shared module, a directory target) may be read by any
/// target of the crate, so its target is `None` (150 D-3).
fn changed_tests(paths: &[String]) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    for p in paths {
        let q: Vec<&str> = p.split('/').collect();
        if q.len() >= 4 && q[0] == "crates" && q[2] == "tests" {
            let target = if q.len() == 4 {
                q[3].strip_suffix(".rs").map(str::to_string)
            } else {
                None
            };
            out.push((q[1].to_string(), target));
        }
    }
    out
}

fn is_alnum(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

/// `needle` as a token: not joined to an ASCII letter or digit on either side.
fn has_token(text: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    text.match_indices(needle).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + needle.len()..].chars().next();
        !before.is_some_and(is_alnum) && !after.is_some_and(is_alnum)
    })
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// A line that runs `cargo [+toolchain] test`, where `cargo` is not the tail of
/// a longer word.
fn runs_cargo_test(line: &str) -> bool {
    line.match_indices("cargo").any(|(i, _)| {
        let before = line[..i].chars().next_back();
        if before.is_some_and(|c| is_alnum(c) || c == '_' || c == '-') {
            return false;
        }
        let rest = &line[i + "cargo".len()..];
        let trimmed = rest.trim_start();
        if trimmed.len() == rest.len() {
            return false;
        }
        let rest = match trimmed.strip_prefix('+') {
            Some(after) => {
                let tool_end = after.find(char::is_whitespace).unwrap_or(after.len());
                let tail = after[tool_end..].trim_start();
                if tail.len() == after[tool_end..].len() {
                    return false;
                }
                tail
            }
            None => trimmed,
        };
        rest.strip_prefix("test")
            .is_some_and(|after| !after.chars().next().is_some_and(is_name_char))
    })
}

/// Every value given to a flag: `--flag value` or `--flag=value`; with
/// `short`, also `-x value` / `-x=value` after whitespace.
fn flag_values<'a>(line: &'a str, long: &str, short: Option<&str>) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < line.len() {
        if !line.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let rest = &line[i..];
        let after_flag = if let Some(r) = rest.strip_prefix(long) {
            Some(r)
        } else {
            match short {
                Some(s) if rest.starts_with(char::is_whitespace) => {
                    rest[rest.chars().next().map_or(0, char::len_utf8)..].strip_prefix(s)
                }
                _ => None,
            }
        };
        let value = after_flag.and_then(|r| {
            let r = if let Some(eq) = r.strip_prefix('=') {
                eq
            } else {
                let t = r.trim_start();
                if t.len() == r.len() {
                    return None;
                }
                t
            };
            let end = r.find(|c: char| !is_name_char(c)).unwrap_or(r.len());
            (end > 0).then(|| (&r[..end], r.len() - end))
        });
        match value {
            Some((v, remaining)) => {
                out.push(v);
                i = line.len() - remaining;
            }
            None => i += 1,
        }
    }
    out
}

/// Whether `line` runs `cargo test` over `krate` and, when the file is a named
/// target, that target (150 D-3).
fn runs_test(line: &str, krate: &str, target: Option<&str>) -> bool {
    if !runs_cargo_test(line) {
        return false;
    }
    let packages = flag_values(line, "--package", Some("-p"));
    if !packages.is_empty() && !packages.contains(&krate) {
        return false;
    }
    let named = flag_values(line, "--test", None);
    match target {
        Some(t) => named.is_empty() || named.contains(&t),
        None => true,
    }
}
