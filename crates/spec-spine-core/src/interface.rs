// Spec: specs/110-an-interface-reference-is-digest-pinned/spec.md
//! The interface-reference verifier (spec 110): recomputes a declared
//! cross-corpus citation's digest, and its pinned sections' digests, from a
//! corpus the caller supplies locally.
//!
//! [`verify_interface_references`] is pure: it reads nothing but its
//! arguments, over exported spec texts the caller already holds. [`load_export`]
//! is the one IO wrapper, reading a local export directory the caller asserts
//! is a named corpus's repository root. [`interface_verify`] composes the two
//! against the committed registry, refusing a stale ledger first, the way
//! [`crate::closure::closure`] does.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use spec_spine_types::{
    Config, Error, InterfaceReport, Outcome, ReferenceResult, Registry, SectionOutcome,
    SectionResult, Summary,
};

use crate::index::Freshness;
use crate::{compile, hash, spec_id};

/// One exported spec's text, as the caller supplies it (spec 110 §3.3, §3.4).
///
/// `path` is the repo-relative path in the **exporter's own** corpus (the
/// same string that corpus's own compile used as `spec_path`), because the
/// content-hash and section-digest constructions are keyed on it. A binding
/// that never touches a filesystem builds this directly and calls
/// [`verify_interface_references`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportedSpec {
    pub path: String,
    pub text: String,
}

/// Every corpus's exported specs, by corpus name then spec id.
pub type Exports = BTreeMap<String, BTreeMap<String, ExportedSpec>>;

/// Recompute every declared interface reference against `exports` (spec 110
/// §3.3). Pure: reads nothing but its arguments.
///
/// `only_spec`, when given, is resolved against `registry` (spec 067's one
/// policy); an argument that does not resolve is [`Error::NotFound`].
/// Otherwise every reference in the registry is checked. Results are sorted by
/// `(declaredBy, corpus, spec)`, so the same inputs give the same bytes.
pub fn verify_interface_references(
    registry: &Registry,
    exports: &Exports,
    only_spec: Option<&str>,
) -> Result<InterfaceReport, Error> {
    let declaring_id = match only_spec {
        Some(arg) => Some(spec_id::resolve_spec_id(
            arg,
            registry.specs.iter().map(|s| s.id.as_str()),
        )?),
        None => None,
    };

    let mut results: Vec<ReferenceResult> = Vec::new();
    let mut summary = Summary::default();

    for record in &registry.specs {
        if let Some(id) = &declaring_id
            && &record.id != id
        {
            continue;
        }
        for r in &record.interface_references {
            let result = verify_one(&record.id, r, exports);
            summary.record(result.outcome);
            results.push(result);
        }
    }

    results.sort_by(|a, b| {
        (&a.declared_by, &a.corpus, &a.spec).cmp(&(&b.declared_by, &b.corpus, &b.spec))
    });

    Ok(InterfaceReport {
        references: results,
        summary,
    })
}

/// One reference's recomputed outcome (spec 110 §3.3 table).
fn verify_one(
    declared_by: &str,
    r: &spec_spine_types::InterfaceReference,
    exports: &Exports,
) -> ReferenceResult {
    let base = |outcome: Outcome, observed_digest: Option<String>, sections: Vec<SectionResult>| {
        ReferenceResult {
            declared_by: declared_by.to_string(),
            corpus: r.corpus.clone(),
            spec: r.spec.clone(),
            digest: r.digest.clone(),
            observed_digest,
            outcome,
            sections,
        }
    };

    let Some(corpus_export) = exports.get(&r.corpus) else {
        // No export was supplied for the corpus at all (spec 110 §3.3):
        // nothing here can be recomputed, including per-section, because
        // there is no text to derive a section digest from.
        return base(Outcome::Unverified, None, Vec::new());
    };
    let Some(exported) = corpus_export.get(&r.spec) else {
        // An export was supplied for the corpus and the spec is not in it.
        return base(Outcome::Missing, None, Vec::new());
    };

    let observed_hex = hash::content_hash(vec![(exported.path.clone(), exported.text.clone())]);
    let observed_digest = format!("sha256:{observed_hex}");
    let content_current = observed_digest == r.digest;

    // The same construction the exporter's own compile used: the body after
    // the frontmatter fence (spec 077, spec 106 §3.5). A body that cannot be
    // split (the exported text carries no `---` frontmatter fence) recomputes
    // no section digests at all, so every pinned anchor reports `missing`
    // rather than the caller's malformed export aborting the whole answer.
    let body = spec_spine_types::split_frontmatter(&exported.text)
        .map(|(_, body)| body)
        .unwrap_or_default();
    let recomputed = compile::section_digests(&exported.path, &body);

    let mut sections = Vec::with_capacity(r.sections.len());
    // Starts false when nothing is pinned: a reference without sections is a
    // claim about the whole spec, so a moved content hash is `stale`, never
    // `sections-current` (§3.3).
    let mut sections_current = !r.sections.is_empty();
    for s in &r.sections {
        match recomputed.get(&s.anchor) {
            Some(hex) => {
                let observed = format!("sha256:{hex}");
                let outcome = if observed == s.digest {
                    SectionOutcome::Current
                } else {
                    sections_current = false;
                    SectionOutcome::Stale
                };
                sections.push(SectionResult {
                    anchor: s.anchor.clone(),
                    digest: s.digest.clone(),
                    observed_digest: Some(observed),
                    outcome,
                });
            }
            None => {
                sections_current = false;
                sections.push(SectionResult {
                    anchor: s.anchor.clone(),
                    digest: s.digest.clone(),
                    observed_digest: None,
                    outcome: SectionOutcome::Missing,
                });
            }
        }
    }

    let outcome = if content_current {
        Outcome::Current
    } else if sections_current {
        Outcome::SectionsCurrent
    } else {
        Outcome::Stale
    };

    base(outcome, Some(observed_digest), sections)
}

/// Read one corpus's export directory (spec 110 §3.3): the exporter's
/// `spec-spine.toml`, when present, only for its specs directory, and
/// `<specs-dir>/<id>/spec.md` for each of `spec_ids`. Reads nothing else,
/// follows no link out of `root`, and opens no network connection.
///
/// A spec id with no `spec.md` under `root` is not an error: its absence is
/// what makes the reference `missing` rather than `unverified`. An unreadable
/// `root`, an exporter config that does not parse, or a spec path that
/// escapes `root` (a symlink out) is [`Error::Io`] or [`Error::Config`]
/// (exit 3).
pub fn load_export(
    root: &Path,
    spec_ids: &[String],
) -> Result<BTreeMap<String, ExportedSpec>, Error> {
    let canonical_root = std::fs::canonicalize(root)
        .map_err(|e| Error::Io(format!("read export root {}: {e}", root.display())))?;

    let cfg = match std::fs::read_to_string(root.join("spec-spine.toml")) {
        Ok(src) => spec_spine_types::load_config(&src)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config::default(),
        Err(e) => {
            return Err(Error::Io(format!(
                "read {}/spec-spine.toml: {e}",
                root.display()
            )));
        }
    };
    let specs_dir = root.join(&cfg.layout.specs_dir);

    let mut out = BTreeMap::new();
    for id in spec_ids {
        let candidate = specs_dir.join(id).join("spec.md");
        if !candidate.exists() {
            continue;
        }
        let canonical_candidate = std::fs::canonicalize(&candidate)
            .map_err(|e| Error::Io(format!("read {}: {e}", candidate.display())))?;
        if !canonical_candidate.starts_with(&canonical_root) {
            return Err(Error::Io(format!(
                "export path for '{id}' escapes its root {}: {}",
                root.display(),
                candidate.display()
            )));
        }
        let text = std::fs::read_to_string(&candidate)
            .map_err(|e| Error::Io(format!("read {}: {e}", candidate.display())))?;
        let path = compile::rel_posix(root, &candidate);
        out.insert(id.clone(), ExportedSpec { path, text });
    }
    Ok(out)
}

/// Every corpus a reference names, mapped to the spec ids referenced under it
/// (across the whole registry): the set [`load_export`] needs to read, and
/// nothing more.
fn referenced_ids_by_corpus(registry: &Registry) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for record in &registry.specs {
        for r in &record.interface_references {
            out.entry(r.corpus.clone())
                .or_default()
                .insert(r.spec.clone());
        }
    }
    out
}

/// Verify against the committed ledger (spec 110 §3.3): a stale registry is
/// refused with [`Error::Stale`] (exit 2) **before** any export is read,
/// checking the committed references the way [`crate::closure::closure`]
/// checks the committed ledger, because verifying what `spec.md` currently
/// says while the committed shard says something else would verify a
/// statement nobody is currently making.
///
/// `exports_dirs` names a local directory per corpus the caller asserts is
/// that corpus's repository root; only the spec ids the committed registry
/// actually references under a named corpus are read from it.
pub fn interface_verify(
    cfg: &Config,
    repo_root: &Path,
    exports_dirs: &BTreeMap<String, PathBuf>,
    only_spec: Option<&str>,
) -> Result<InterfaceReport, Error> {
    if let Freshness::Stale { expected, actual } =
        compile::check_registry_freshness(cfg, repo_root)?
    {
        return Err(Error::Stale { expected, actual });
    }
    let registry = compile::load_committed_registry(cfg, repo_root)?;
    let referenced = referenced_ids_by_corpus(&registry);

    let mut exports: Exports = BTreeMap::new();
    for (corpus, dir) in exports_dirs {
        let ids: Vec<String> = referenced
            .get(corpus)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        exports.insert(corpus.clone(), load_export(dir, &ids)?);
    }

    verify_interface_references(&registry, &exports, only_spec)
}
