//! A change classified under the base's rules (spec 071).
//!
//! [`delta`] is a pure function of `(base root, head root, changed paths,
//! commit ids)`: it reads two exported trees and never runs `git` or reads the
//! clock. The CLI resolves the merge base, lists the changed paths, exports both
//! trees and echoes the three commit ids in; this module only reads files.
//!
//! **The merge base's rules classify** (spec 071 §3.2, D-1). The configuration
//! is the base's, and so is the committed index, compared under spec 069 before
//! it is used. A candidate that edits `spec-spine.toml` gets that edit reported
//! as `policy`, and the rest of its diff classified exactly as it would have been
//! without it.
//!
//! **It decides nothing.** A path carries every class that applies; the report
//! names the classes a consumer must judge under the base's policy and stops
//! there. Conservative defaults (D-4): a frontmatter key this tool does not
//! model is a `requirement`, a parse failure is `unknown`, and neither is ever
//! reported as `implementation`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path};

use sha2::{Digest, Sha256};
use spec_spine_types::delta::CLASSIFIED_UNDER_BASE;
use spec_spine_types::{
    AuthorityDelta, ChangeKind, CodebaseIndex, Config, DELTA_SCHEMA_VERSION, DeltaChange,
    DeltaClass, DeltaCommits, DeltaReport, Error, Frontmatter, PriorPolicy, RelocationCheck,
    ToolStamp, ValueChange, VerificationDelta, load_config, parse_frontmatter_with,
    split_frontmatter,
};

use crate::canonical_json;
use crate::compile::compile;
use crate::couple::{
    build_superseders, is_bypassed_path, owners_for_path, spec_id_for_spec_md_path,
};
use crate::index::{Freshness, check_index_freshness, index, load_committed_index};
use crate::pathutil::rel_posix;
use crate::shard;
use crate::verify::{plan_from_markdown, without_verification_section};

/// The configuration file every tree declares its rules in.
const CONFIG_FILE: &str = "spec-spine.toml";

/// Frontmatter keys whose change is `authority`: the eight typed edges and
/// `depends_on` (spec 071 §3.3). `origin` is a bootstrap marker, not an edge.
const AUTHORITY_KEYS: &[&str] = &[
    "establishes",
    "extends",
    "refines",
    "supersedes",
    "amends",
    "co_authority",
    "constrains",
    "references",
    "depends_on",
];

/// Frontmatter keys whose change is `lifecycle`.
const LIFECYCLE_KEYS: &[&str] = &[
    "status",
    "implementation",
    "superseded_by",
    "retirement_rationale",
];

/// The frontmatter key whose presence makes a `spec.md` `constitutional`.
const UNAMENDABLE_KEY: &str = "unamendable";

/// The configuration a tree declares: `<root>/spec-spine.toml`, or the working
/// default when the file is absent. The facade uses it to read the base's rules
/// when its caller supplies none.
pub fn tree_config(root: &Path) -> Result<Config, Error> {
    let path = root.join(CONFIG_FILE);
    match fs::read_to_string(&path) {
        Ok(src) => load_config(&src),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(Error::Io(format!("read {}: {e}", path.display()))),
    }
}

/// One side of the change: its root, and the ownership it resolves to.
struct Side<'a> {
    root: &'a Path,
    index: &'a CodebaseIndex,
    superseders: &'a BTreeMap<String, BTreeSet<String>>,
}

/// Classify every changed path under the base's rules (spec 071 §3.2 to §3.6).
///
/// `cfg` is the **merge base's** configuration. `changed` is the path list of
/// `merge-base...head` with renames disabled, repo-relative and POSIX; it is
/// deduplicated and sorted here, and a path escaping the roots is refused.
///
/// The base's committed index is compared with its tree first (spec 069) and a
/// mismatch is [`Error::Stale`]: the rules this report is computed under would
/// otherwise be whatever the committed body says. The head's ownership is
/// resolved by indexing the head tree in memory under the same configuration,
/// never by reading the head's committed index, which the candidate wrote
/// (D-5).
pub fn delta(
    cfg: &Config,
    base_root: &Path,
    head_root: &Path,
    changed: &[String],
    commits: &DeltaCommits,
) -> Result<DeltaReport, Error> {
    let paths = checked_paths(changed)?;

    if let Freshness::Stale { expected, actual } = check_index_freshness(cfg, base_root)? {
        return Err(Error::Stale {
            expected: format!("{expected} at the merge base"),
            actual,
        });
    }
    let base_index = load_committed_index(cfg, base_root)?;
    let head_index = index(cfg, head_root)?.index;
    let base_superseders = build_superseders(&compile(cfg, base_root)?.registry);
    let head_registry = compile(cfg, head_root)?.registry;
    let head_superseders = build_superseders(&head_registry);
    let base = Side {
        root: base_root,
        index: &base_index,
        superseders: &base_superseders,
    };
    let head = Side {
        root: head_root,
        index: &head_index,
        superseders: &head_superseders,
    };

    let policy_inputs = hashed_input_paths(cfg, &[base_root, head_root]);

    // Spec 142 §3.3: every declared relocation this change touches, checked
    // against the merge base, and the anchors proven to have left each source.
    let relocations = check_relocations(cfg, base_root, head_root, &head_registry, &paths)?;
    let mut proven_from: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for r in relocations.iter().filter(|r| r.proven) {
        proven_from
            .entry(r.from_spec.clone())
            .or_default()
            .insert(r.from.clone());
    }

    let mut changes = Vec::with_capacity(paths.len());
    for path in &paths {
        changes.push(classify_path(
            cfg,
            &base,
            &head,
            &policy_inputs,
            &proven_from,
            path,
        )?);
    }

    let mut counts: BTreeMap<DeltaClass, usize> = DeltaClass::ALL.iter().map(|c| (*c, 0)).collect();
    let mut prior: BTreeSet<DeltaClass> = BTreeSet::new();
    for change in &changes {
        for class in &change.classes {
            *counts.entry(*class).or_insert(0) += 1;
            if class.requires_prior_policy() {
                prior.insert(*class);
            }
        }
    }

    Ok(DeltaReport {
        schema_version: DELTA_SCHEMA_VERSION.to_string(),
        tool: ToolStamp {
            name: cfg.branding.compiler_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        classified_under: CLASSIFIED_UNDER_BASE.to_string(),
        base: commits.base.clone(),
        merge_base: commits.merge_base.clone(),
        head: commits.head.clone(),
        changes,
        counts,
        prior_policy: PriorPolicy {
            required: !prior.is_empty(),
            classes: prior.into_iter().collect(),
        },
        relocations,
    })
}

/// Spec 142 §3.3: each head spec's declared relocations whose source or
/// receiving `spec.md` is among the changed paths, each checked by comparing the
/// source section at the merge base with the receiving section at head through
/// [`relocation_digest`](crate::relocation::relocation_digest). Sorted by
/// receiver, then source, then anchor.
fn check_relocations(
    cfg: &Config,
    base_root: &Path,
    head_root: &Path,
    head_registry: &spec_spine_types::Registry,
    changed: &BTreeSet<String>,
) -> Result<Vec<RelocationCheck>, Error> {
    let specs_dir = cfg.layout.specs_dir.as_str();
    let md = |id: &str| format!("{}/{id}/spec.md", specs_dir.trim_end_matches('/'));
    let body_at = |root: &Path, id: &str| -> Result<Option<String>, Error> {
        let Some(bytes) = read_side(root, &md(id))? else {
            return Ok(None);
        };
        let Ok(text) = String::from_utf8(bytes) else {
            return Ok(None);
        };
        Ok(Some(
            split_frontmatter(&text).map_or(text.clone(), |(_, body)| body.to_string()),
        ))
    };
    let mut out = Vec::new();
    for record in &head_registry.specs {
        for rel in &record.relocates {
            if !changed.contains(&md(&record.id)) && !changed.contains(&md(&rel.spec)) {
                continue;
            }
            let to = rel.to_anchor().to_string();
            let source = body_at(base_root, &rel.spec)?
                .and_then(|b| crate::relocation::section_relocation_digest(&b, &rel.from));
            let received = body_at(head_root, &record.id)?
                .and_then(|b| crate::relocation::section_relocation_digest(&b, &to));
            let reason = match (&source, &received) {
                (None, _) => Some(format!(
                    "the merge base has no section '{}' in {}",
                    rel.from, rel.spec
                )),
                (_, None) => Some(format!("{} has no section '{to}' at head", record.id)),
                (Some(a), Some(b)) if a != b => Some(
                    "the text differs: the receiving section is not the source section as it \
                     was at the merge base"
                        .to_string(),
                ),
                _ => None,
            };
            out.push(RelocationCheck {
                spec: record.id.clone(),
                from_spec: rel.spec.clone(),
                from: rel.from.clone(),
                to,
                proven: reason.is_none(),
                reason,
            });
        }
    }
    out.sort_by(|a, b| {
        (&a.spec, &a.from_spec, &a.from).cmp(&(&b.spec, &b.from_spec, &b.from))
    });
    Ok(out)
}

/// The changed paths, sorted and deduplicated, each a plain repo-relative path.
///
/// A facade caller supplies these as data, and each is joined onto a root, so
/// an absolute path or a `..` component would read outside the tree the report
/// claims to describe.
fn checked_paths(changed: &[String]) -> Result<BTreeSet<String>, Error> {
    let mut out = BTreeSet::new();
    for path in changed {
        let plain = !path.is_empty()
            && Path::new(path)
                .components()
                .all(|c| matches!(c, Component::Normal(_)));
        if !plain {
            return Err(Error::Usage(format!(
                "invalid changed path '{path}': expected a repo-relative path with no '.' or '..' component"
            )));
        }
        out.insert(path.clone());
    }
    Ok(out)
}

/// Every class one path carries (spec 071 §3.3), with the detail §3.4 attaches.
fn classify_path(
    cfg: &Config,
    base: &Side<'_>,
    head: &Side<'_>,
    policy_inputs: &BTreeSet<String>,
    proven_from: &BTreeMap<String, BTreeSet<String>>,
    path: &str,
) -> Result<DeltaChange, Error> {
    let base_bytes = read_side(base.root, path)?;
    let head_bytes = read_side(head.root, path)?;
    let change = match (&base_bytes, &head_bytes) {
        (None, Some(_)) => ChangeKind::Added,
        (Some(_), None) => ChangeKind::Deleted,
        _ => ChangeKind::Modified,
    };

    let specs_dir = cfg.layout.specs_dir.as_str();
    let mut classes: BTreeSet<DeltaClass> = BTreeSet::new();

    // Whole-file owner sets, from the base index and from the head tree
    // resolved under the base's rules.
    let base_owners = owners_for_path(specs_dir, path, &[], base.index, base.superseders);
    let head_owners = owners_for_path(specs_dir, path, &[], head.index, head.superseders);
    let mut authority = (base_owners != head_owners).then(|| AuthorityDelta {
        base_owners: base_owners.iter().cloned().collect(),
        head_owners: head_owners.iter().cloned().collect(),
        edges_added: BTreeMap::new(),
        edges_removed: BTreeMap::new(),
    });

    let spec_id = spec_id_for_spec_md_path(specs_dir, path);
    let standards = under_root(&cfg.layout.standards_dir, path);
    let policy = path == CONFIG_FILE || policy_inputs.contains(path);
    let derived = under_root(&cfg.layout.derived_dir, path);
    let bypass = is_bypassed_path(cfg, base.index, path);

    let mut verification = None;
    let mut lifecycle = None;
    if let Some(id) = spec_id {
        let empty = BTreeSet::new();
        let proven = proven_from.get(id).unwrap_or(&empty);
        let spec = classify_spec_md(cfg, id, proven, base_bytes.as_deref(), head_bytes.as_deref())?;
        classes.extend(&spec.classes);
        verification = spec.verification;
        if !spec.lifecycle.is_empty() {
            lifecycle = Some(spec.lifecycle);
        }
        if spec.classes.contains(&DeltaClass::Authority) {
            let detail = authority.get_or_insert_with(|| AuthorityDelta {
                base_owners: base_owners.iter().cloned().collect(),
                head_owners: head_owners.iter().cloned().collect(),
                edges_added: BTreeMap::new(),
                edges_removed: BTreeMap::new(),
            });
            detail.edges_added = spec.edges_added;
            detail.edges_removed = spec.edges_removed;
        }
    }
    if authority.is_some() {
        classes.insert(DeltaClass::Authority);
    }
    if standards {
        classes.insert(DeltaClass::Constitutional);
    }
    if policy {
        classes.insert(DeltaClass::Policy);
        if path == CONFIG_FILE
            && (config_fails_to_parse(base_bytes.as_deref())
                || config_fails_to_parse(head_bytes.as_deref()))
        {
            classes.insert(DeltaClass::Unknown);
        }
    }
    if derived {
        classes.insert(DeltaClass::Derived);
    }

    // `implementation` and `unowned` are the two answers for an ordinary file,
    // and neither applies to a path whose kind already placed it: a spec, a
    // standards document, a policy input, a derived artifact, or a bypassed
    // path (D-8).
    let special_kind = spec_id.is_some() || standards || policy || derived || bypass;
    let owned = !base_owners.is_empty() || !head_owners.is_empty();
    if owned && !special_kind {
        classes.insert(DeltaClass::Implementation);
    }
    if bypass && classes.is_empty() {
        classes.insert(DeltaClass::Bypassed);
    }
    if !owned && !special_kind {
        classes.insert(DeltaClass::Unowned);
    }
    // At least one class, and never a harmless one for a change nothing above
    // could place (§3.3's last row).
    if classes.is_empty() {
        classes.insert(DeltaClass::Unknown);
    }

    Ok(DeltaChange {
        path: path.to_string(),
        change,
        classes: classes.into_iter().collect(),
        verification,
        authority,
        lifecycle,
    })
}

/// The bytes a tree holds at `path`, or `None` when it holds no file there.
///
/// A symlink is read as its target, which is what git stores for one, rather
/// than followed: a link in a candidate's tree must not make the report read a
/// file outside that tree.
fn read_side(root: &Path, path: &str) -> Result<Option<Vec<u8>>, Error> {
    let full = root.join(path);
    let meta = match fs::symlink_metadata(&full) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::Io(format!("stat {}: {e}", full.display()))),
    };
    if meta.file_type().is_symlink() {
        let target = fs::read_link(&full)
            .map_err(|e| Error::Io(format!("read link {}: {e}", full.display())))?;
        return Ok(Some(target.to_string_lossy().into_owned().into_bytes()));
    }
    if !meta.is_file() {
        // A directory or gitlink: nothing a file-level class can read.
        return Ok(None);
    }
    fs::read(&full)
        .map(Some)
        .map_err(|e| Error::Io(format!("read {}: {e}", full.display())))
}

/// Whether `path` lies under a configured layout root, separator-aware.
fn under_root(root: &str, path: &str) -> bool {
    let root = root.trim_end_matches('/');
    let root = root.strip_prefix("./").unwrap_or(root);
    !root.is_empty() && path.strip_prefix(root).is_some_and(|r| r.starts_with('/'))
}

/// Every file the base's `[index] extra_hashed_inputs` folds into the shard
/// hashes, found in either tree: a path is policy exactly when editing it moves
/// the ledger (D-11).
///
/// Found with the hash's own matcher, [`shard::glob_files`], not a pattern test
/// over the path. The two disagree on a pattern ending in a bare `**`, which the
/// filesystem walk resolves to directories and therefore to no file (spec 058),
/// and a dead glob is dead everywhere (spec 065). Both trees are walked, so a
/// path deleted at head and one added at head are each found on the side where
/// it exists. A declared state root contributes to no hash (spec 036).
fn hashed_input_paths(cfg: &Config, roots: &[&Path]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for root in roots {
        for pattern in &cfg.index.extra_hashed_inputs {
            for file in shard::glob_files(root, pattern) {
                let rel = rel_posix(root, &file);
                if !cfg.layout.is_state_path(&rel) {
                    out.insert(rel);
                }
            }
        }
    }
    out
}

/// True when a configuration file is present on this side and does not load.
fn config_fails_to_parse(bytes: Option<&[u8]>) -> bool {
    match bytes {
        None => false,
        Some(bytes) => std::str::from_utf8(bytes).map_or(true, |src| load_config(src).is_err()),
    }
}

/// What a `spec.md` change is, before ownership is considered.
#[derive(Default)]
struct SpecDelta {
    classes: BTreeSet<DeltaClass>,
    verification: Option<VerificationDelta>,
    lifecycle: BTreeMap<String, ValueChange>,
    edges_added: BTreeMap<String, Vec<serde_json::Value>>,
    edges_removed: BTreeMap<String, Vec<serde_json::Value>>,
}

/// One side of a `spec.md`, read three ways: the typed grammar, the raw YAML
/// mapping, and the body.
struct SpecSide {
    typed: Option<Frontmatter>,
    raw: Option<serde_yaml::Mapping>,
    body: Option<String>,
}

impl SpecSide {
    fn read(cfg: &Config, text: &str) -> Self {
        let split = split_frontmatter(text).ok();
        let raw = split
            .as_ref()
            .and_then(|(yaml, _)| serde_yaml::from_str::<serde_yaml::Value>(yaml).ok())
            .and_then(|v| v.as_mapping().cloned());
        SpecSide {
            typed: parse_frontmatter_with(text, &cfg.frontmatter.extra_known_keys).ok(),
            raw,
            body: split.map(|(_, body)| body),
        }
    }

    fn declares_unamendable(&self) -> bool {
        self.raw
            .as_ref()
            .and_then(|m| m.get(UNAMENDABLE_KEY))
            .is_some_and(|v| match v {
                serde_yaml::Value::Null => false,
                serde_yaml::Value::Sequence(items) => !items.is_empty(),
                _ => true,
            })
    }
}

/// Classify a changed `spec.md` by what moved inside it (spec 071 §3.3, §3.4).
fn classify_spec_md(
    cfg: &Config,
    id: &str,
    proven_relocated: &BTreeSet<String>,
    base: Option<&[u8]>,
    head: Option<&[u8]>,
) -> Result<SpecDelta, Error> {
    let mut out = SpecDelta::default();

    let (Ok(base_text), Ok(head_text)) = (text_of(base), text_of(head)) else {
        // Not UTF-8 on a side where the file exists: nothing below can read it.
        out.classes.insert(DeltaClass::Unknown);
        return Ok(out);
    };

    // Verification: the plan, parsed by spec 043's grammar. An added or removed
    // spec differs by construction, since one side has no plan at all.
    let base_plan = base_text.map(|t| plan_from_markdown(id, t).commands);
    let head_plan = head_text.map(|t| plan_from_markdown(id, t).commands);
    if base_plan != head_plan {
        out.classes.insert(DeltaClass::Verification);
        let empty: Vec<String> = Vec::new();
        let b = base_plan.as_ref().unwrap_or(&empty);
        let h = head_plan.as_ref().unwrap_or(&empty);
        out.verification = Some(VerificationDelta {
            base_plan_hash: base_plan.as_deref().map(plan_hash).transpose()?,
            head_plan_hash: head_plan.as_deref().map(plan_hash).transpose()?,
            commands_only_in_base: count_only_in(b, h),
            commands_only_in_head: count_only_in(h, b),
        });
    }

    let base_side = base_text.map(|t| SpecSide::read(cfg, t));
    let head_side = head_text.map(|t| SpecSide::read(cfg, t));

    if base_side
        .iter()
        .chain(&head_side)
        .any(SpecSide::declares_unamendable)
    {
        out.classes.insert(DeltaClass::Constitutional);
    }

    // A side that exists and fails the grammar is `unknown`, and nothing is
    // guessed about which of its keys moved.
    if base_side
        .iter()
        .chain(&head_side)
        .any(|s| s.typed.is_none())
    {
        out.classes.insert(DeltaClass::Unknown);
        return Ok(out);
    }

    let body_outside = |side: &Option<SpecSide>| {
        side.as_ref()
            .map(|s| without_verification_section(s.body.as_deref().unwrap_or("")))
    };
    let (base_body, head_body) = (body_outside(&base_side), body_outside(&head_side));
    if base_body != head_body {
        // Spec 142 §3.3: a body that lost exactly the sections another spec
        // proved it received, and changed in no other way, is a relocation.
        let relocated = matches!((&base_body, &head_body), (Some(b), Some(h))
            if crate::relocation::relocation_only(b, h, proven_relocated));
        out.classes.insert(if relocated {
            DeltaClass::Relocation
        } else {
            DeltaClass::Requirement
        });
    }

    let null = serde_yaml::Value::Null;
    let base_raw = base_side.as_ref().and_then(|s| s.raw.as_ref());
    let head_raw = head_side.as_ref().and_then(|s| s.raw.as_ref());
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for mapping in base_raw.iter().chain(head_raw.iter()) {
        keys.extend(
            mapping
                .keys()
                .filter_map(|k| k.as_str().map(str::to_string)),
        );
    }
    let mut authority_moved = false;
    for key in &keys {
        let b = base_raw.and_then(|m| m.get(key.as_str())).unwrap_or(&null);
        let h = head_raw.and_then(|m| m.get(key.as_str())).unwrap_or(&null);
        if b == h {
            continue;
        }
        let key = key.as_str();
        if AUTHORITY_KEYS.contains(&key) {
            authority_moved = true;
        } else if LIFECYCLE_KEYS.contains(&key) {
            out.lifecycle.insert(
                key.to_string(),
                ValueChange {
                    base: yaml_value(b),
                    head: yaml_value(h),
                },
            );
        } else if key == UNAMENDABLE_KEY {
            out.classes.insert(DeltaClass::Constitutional);
        } else {
            // Unmodeled keys, declared `extra_known_keys` included, are a change
            // to what is required, never nothing (D-4).
            out.classes.insert(DeltaClass::Requirement);
        }
    }
    if !out.lifecycle.is_empty() {
        out.classes.insert(DeltaClass::Lifecycle);
    }

    if authority_moved {
        out.classes.insert(DeltaClass::Authority);
        let base_edges = base_side
            .as_ref()
            .and_then(|s| s.typed.as_ref())
            .map(edge_items)
            .transpose()?
            .unwrap_or_default();
        let head_edges = head_side
            .as_ref()
            .and_then(|s| s.typed.as_ref())
            .map(edge_items)
            .transpose()?
            .unwrap_or_default();
        for key in AUTHORITY_KEYS {
            let empty = Vec::new();
            let b = base_edges.get(*key).unwrap_or(&empty);
            let h = head_edges.get(*key).unwrap_or(&empty);
            let added = items_only_in(h, b);
            if !added.is_empty() {
                out.edges_added.insert((*key).to_string(), added);
            }
            let removed = items_only_in(b, h);
            if !removed.is_empty() {
                out.edges_removed.insert((*key).to_string(), removed);
            }
        }
    }

    Ok(out)
}

/// Decode a present side as UTF-8; an absent side is `Ok(None)`.
fn text_of(bytes: Option<&[u8]>) -> Result<Option<&str>, std::str::Utf8Error> {
    bytes.map(std::str::from_utf8).transpose()
}

/// SHA-256 (lowercase hex) of the canonical JSON of a plan's `commands` list.
fn plan_hash(commands: &[String]) -> Result<String, Error> {
    let canonical = canonical_json::to_string(&commands)?;
    let digest = Sha256::digest(canonical.as_bytes());
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(hex)
}

/// How many entries of `a` are not matched by an entry of `b`, with
/// multiplicity: a command declared twice and removed once still counts.
fn count_only_in(a: &[String], b: &[String]) -> usize {
    let mut remaining: BTreeMap<&str, usize> = BTreeMap::new();
    for item in b {
        *remaining.entry(item.as_str()).or_insert(0) += 1;
    }
    a.iter()
        .filter(|item| match remaining.get_mut(item.as_str()) {
            Some(n) if *n > 0 => {
                *n -= 1;
                false
            }
            _ => true,
        })
        .count()
}

/// The typed edge items of one side, keyed by frontmatter key, as JSON values.
///
/// Typed rather than raw so the `paths:` sugar (spec 013) and the full-scope
/// `supersedes` spelling (spec 018) compare as the edges they expand to.
fn edge_items(fm: &Frontmatter) -> Result<BTreeMap<&'static str, Vec<serde_json::Value>>, Error> {
    fn values<T: serde::Serialize>(items: &[T]) -> Result<Vec<serde_json::Value>, Error> {
        items
            .iter()
            .map(|i| serde_json::to_value(i).map_err(|e| Error::Internal(e.to_string())))
            .collect()
    }
    let mut out = BTreeMap::new();
    out.insert("establishes", values(&fm.establishes)?);
    out.insert("extends", values(&fm.extends)?);
    out.insert("refines", values(&fm.refines)?);
    out.insert("supersedes", values(&fm.supersedes)?);
    out.insert("amends", values(&fm.amends)?);
    out.insert("co_authority", values(&fm.co_authority)?);
    out.insert("constrains", values(&fm.constrains)?);
    out.insert("references", values(&fm.references)?);
    out.insert("depends_on", values(&fm.depends_on)?);
    Ok(out)
}

/// The items of `a` not matched in `b`, with multiplicity, sorted by their
/// canonical serialization so the report does not depend on authoring order.
fn items_only_in(a: &[serde_json::Value], b: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut remaining: BTreeMap<String, usize> = BTreeMap::new();
    for item in b {
        *remaining.entry(item.to_string()).or_insert(0) += 1;
    }
    let mut out: Vec<(String, serde_json::Value)> = Vec::new();
    for item in a {
        let key = item.to_string();
        match remaining.get_mut(&key) {
            Some(n) if *n > 0 => *n -= 1,
            _ => out.push((key, item.clone())),
        }
    }
    out.sort_by(|x, y| x.0.cmp(&y.0));
    out.into_iter().map(|(_, v)| v).collect()
}

/// A raw frontmatter value as JSON. `null` stands for absent.
fn yaml_value(v: &serde_yaml::Value) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or_else(|_| serde_json::Value::String(format!("{v:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_root_is_separator_aware() {
        assert!(under_root(
            "standards/spec",
            "standards/spec/constitution.md"
        ));
        assert!(under_root("./.derived/", ".derived/x.json"));
        assert!(!under_root("standards/spec", "standards/specs/x.md"));
        assert!(!under_root("", "anything"));
    }

    #[test]
    fn counts_respect_multiplicity() {
        let a = ["x".to_string(), "x".to_string(), "y".to_string()];
        let b = ["x".to_string()];
        assert_eq!(count_only_in(&a, &b), 2);
        assert_eq!(count_only_in(&b, &a), 0);
    }

    #[test]
    fn paths_escaping_the_root_are_refused() {
        for bad in ["", "/etc/passwd", "../x", "a/../b", "./a"] {
            assert!(checked_paths(&[bad.to_string()]).is_err(), "{bad}");
        }
        assert!(checked_paths(&["src/a.rs".to_string()]).is_ok());
    }
}
