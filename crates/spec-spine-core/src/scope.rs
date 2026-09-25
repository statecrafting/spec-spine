// Spec: specs/108-a-work-scope-is-declared/spec.md
//! WorkScope (spec 108): a consumer's declared partition of paths for one
//! piece of work, evaluated against the committed ownership index, and
//! compared against another declared partition for conflicting intentions.
//!
//! A scope lives in the consumer's record (spec 108 §3.1, spec 092), exactly
//! where spec 107 §3.1 put a context closure. This module only evaluates and
//! compares one: [`evaluate_scope`] and [`compare_scopes`] are pure over their
//! arguments, and [`evaluate`] is the IO wrapper that checks index freshness
//! and reads the committed index and registry.
//!
//! A scope is not a gate, a lock or a permission (§3.7): evaluating and
//! comparing write nothing, and no gate reads one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use spec_spine_types::{CodebaseIndex, Config, Error, Registry};

use crate::couple::claim_matches;
use crate::spec_id;

/// One path this work declares an intent for (spec 108 §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Mutable,
    Shared,
    ReadOnly,
}

impl Role {
    fn label(self) -> &'static str {
        match self {
            Role::Mutable => "mutable",
            Role::Shared => "shared",
            Role::ReadOnly => "readOnly",
        }
    }
}

/// A `shared` path and the specs it names as concurrent (spec 108 §3.1).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SharedPath {
    pub path: String,
    pub with: Vec<String>,
}

/// The document a consumer holds (spec 108 §3.1). Unknown members are
/// refused: a misspelt role would silently declare less than its author
/// named.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopeRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub own_spec: String,
    #[serde(default)]
    pub mutable: Vec<String>,
    #[serde(default)]
    pub shared: Vec<SharedPath>,
    #[serde(default)]
    pub read_only: Vec<String>,
}

/// One declared path, resolved to a role and (for `shared`) its declared
/// `with` set, after §3.1's structural validation. Unique by path: two roles
/// never share or overlap one (validated below).
#[derive(Debug, Clone)]
struct ParsedEntry {
    path: String,
    role: Role,
    with: BTreeSet<String>,
}

/// §3.1: at least one path in total, no absolute path or `..` segment, no
/// empty entry, a non-empty `with` for every `shared` entry, a path named
/// twice under one role collapsed to one, and no path (or subtree and a path
/// inside it) named under two roles. Refused as [`Error::Usage`] (exit 3).
fn validate(req: &ScopeRequest) -> Result<Vec<ParsedEntry>, Error> {
    if req.mutable.is_empty() && req.shared.is_empty() && req.read_only.is_empty() {
        return Err(Error::Usage(
            "a scope names at least one path across mutable, shared or readOnly".into(),
        ));
    }

    fn check_path(p: &str) -> Result<(), Error> {
        if p.is_empty() {
            return Err(Error::Usage("a scope path may not be empty".into()));
        }
        if p.starts_with('/') {
            return Err(Error::Usage(format!(
                "scope path '{p}' is absolute: paths are repo-relative"
            )));
        }
        if p.split('/').any(|seg| seg == "..") {
            return Err(Error::Usage(format!(
                "scope path '{p}' carries a '..' segment"
            )));
        }
        Ok(())
    }

    let mut mutable: BTreeSet<String> = BTreeSet::new();
    for p in &req.mutable {
        check_path(p)?;
        mutable.insert(p.clone());
    }
    let mut read_only: BTreeSet<String> = BTreeSet::new();
    for p in &req.read_only {
        check_path(p)?;
        read_only.insert(p.clone());
    }
    let mut shared: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for s in &req.shared {
        check_path(&s.path)?;
        if s.with.is_empty() {
            return Err(Error::Usage(format!(
                "shared path '{}' names an empty 'with': a shared path must name at least \
                 one other spec",
                s.path
            )));
        }
        shared
            .entry(s.path.clone())
            .or_default()
            .extend(s.with.iter().cloned());
    }

    let mut entries: Vec<ParsedEntry> = Vec::new();
    for p in &mutable {
        entries.push(ParsedEntry {
            path: p.clone(),
            role: Role::Mutable,
            with: BTreeSet::new(),
        });
    }
    for (p, with) in &shared {
        entries.push(ParsedEntry {
            path: p.clone(),
            role: Role::Shared,
            with: with.clone(),
        });
    }
    for p in &read_only {
        entries.push(ParsedEntry {
            path: p.clone(),
            role: Role::ReadOnly,
            with: BTreeSet::new(),
        });
    }

    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            let (a, b) = (&entries[i], &entries[j]);
            if a.role != b.role && paths_overlap(&a.path, &b.path) {
                return Err(Error::Usage(format!(
                    "'{}' ({}) and '{}' ({}) name one path under two roles",
                    a.path,
                    a.role.label(),
                    b.path,
                    b.role.label()
                )));
            }
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

/// Two paths overlap when they are equal, or one is a subtree containing the
/// other (spec 108 §3.5), which is [`claim_matches`] checked both ways.
fn paths_overlap(a: &str, b: &str) -> bool {
    claim_matches(a, b) || claim_matches(b, a)
}

/// One resolved path in a [`ScopeEvaluation`] (spec 108 §3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluatedEntry {
    pub path: String,
    pub role: Role,
    /// The specs the committed index says own this path, sorted. Empty means
    /// nothing owns it.
    pub owners: Vec<String>,
    /// Resolved `with` ids, only for a `shared` entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with: Option<Vec<String>>,
}

/// One disagreement between a declaration and the ownership index (spec 108
/// §3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub code: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub own_spec: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub owners: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub with: Vec<String>,
}

/// A scope resolved against the committed ownership index (spec 108 §3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeEvaluation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub own_spec: String,
    pub index_hash: String,
    pub entries: Vec<EvaluatedEntry>,
    pub findings: Vec<Finding>,
}

/// Every claim in `index` related to `entry` by containment either way: equal
/// to it, inside it, or a subtree containing it (spec 108 §3.2). For a file
/// entry this is exactly [`crate::couple::owners_for_path`]'s whole-file
/// reading; for a subtree entry it additionally picks up every claim inside
/// the subtree, which `owners_for_path`'s single-path containment test alone
/// does not reach.
fn resolve_entry_owners(
    cfg: &Config,
    index: &CodebaseIndex,
    superseders: &BTreeMap<String, BTreeSet<String>>,
    entry: &str,
) -> BTreeSet<String> {
    if !entry.ends_with('/') {
        let specs_dir = cfg.layout.specs_dir.as_str();
        return crate::couple::owners_for_path(specs_dir, entry, &[], index, superseders);
    }

    let mut owners: BTreeSet<String> = BTreeSet::new();
    for m in &index.traceability.mappings {
        for ru in m.resolved_units.iter().filter(|ru| ru.ownership) {
            for loc in &ru.locations {
                if paths_overlap(entry, &loc.file) {
                    owners.insert(m.spec_id.clone());
                }
            }
        }
        for ip in &m.implementing_paths {
            if paths_overlap(entry, &ip.path) {
                owners.insert(m.spec_id.clone());
            }
        }
    }

    // Supersedes transfer (spec 048): a successor inherits its predecessor's
    // claims, the whole-file reading `owners_for_path` also applies here.
    let mut transferred: BTreeSet<String> = BTreeSet::new();
    for owner in &owners {
        let mut stack = vec![owner.clone()];
        while let Some(cur) = stack.pop() {
            if let Some(succs) = superseders.get(&cur) {
                for s in succs {
                    if transferred.insert(s.clone()) {
                        stack.push(s.clone());
                    }
                }
            }
        }
    }
    owners.extend(transferred);
    owners
}

/// Resolve a scope request against a registry and the committed index (spec
/// 108 §3.2 - 3.3). Pure: reads nothing but its arguments.
///
/// Refusals, in order: a structurally malformed document is [`Error::Parse`]
/// (exit 3); `ownSpec` or any `with` entry that does not resolve to a spec in
/// `registry` is collected into one [`Error::NotFound`] (exit 1) naming each.
pub fn evaluate_scope(
    cfg: &Config,
    registry: &Registry,
    index: &CodebaseIndex,
    req: &ScopeRequest,
) -> Result<ScopeEvaluation, Error> {
    let parsed = validate(req)?;

    let ids = || registry.specs.iter().map(|s| s.id.as_str());
    let mut missing: Vec<String> = Vec::new();

    let own_spec = match spec_id::resolve_spec_id(&req.own_spec, ids()) {
        Ok(id) => Some(id),
        Err(e) => {
            missing.push(format!("ownSpec '{}': {e}", req.own_spec));
            None
        }
    };

    let mut with_resolved: BTreeMap<String, String> = BTreeMap::new();
    for entry in &parsed {
        for raw in &entry.with {
            if with_resolved.contains_key(raw) {
                continue;
            }
            match spec_id::resolve_spec_id(raw, ids()) {
                Ok(id) => {
                    with_resolved.insert(raw.clone(), id);
                }
                Err(e) => missing.push(format!("with '{raw}': {e}")),
            }
        }
    }

    if !missing.is_empty() {
        return Err(Error::NotFound(format!(
            "scope references that do not resolve: {}",
            missing.join("; ")
        )));
    }
    let own_spec = own_spec.expect("collected above when absent");

    let superseders = crate::couple::build_superseders(registry);
    let mut entries: Vec<EvaluatedEntry> = Vec::with_capacity(parsed.len());
    let mut findings: Vec<Finding> = Vec::new();

    for entry in &parsed {
        let owners = resolve_entry_owners(cfg, index, &superseders, &entry.path);
        let with_ids: BTreeSet<String> = entry
            .with
            .iter()
            .map(|raw| with_resolved[raw].clone())
            .collect();
        let others: BTreeSet<String> = owners.iter().filter(|o| **o != own_spec).cloned().collect();

        match entry.role {
            Role::Mutable => {
                if owners.is_empty() {
                    findings.push(Finding {
                        code: "S-001".into(),
                        path: entry.path.clone(),
                        own_spec: None,
                        owners: Vec::new(),
                        with: Vec::new(),
                    });
                } else if !others.is_empty() {
                    findings.push(Finding {
                        code: "S-002".into(),
                        path: entry.path.clone(),
                        own_spec: Some(own_spec.clone()),
                        owners: others.iter().cloned().collect(),
                        with: Vec::new(),
                    });
                }
            }
            Role::Shared => {
                if owners.is_empty() {
                    findings.push(Finding {
                        code: "S-001".into(),
                        path: entry.path.clone(),
                        own_spec: None,
                        owners: Vec::new(),
                        with: Vec::new(),
                    });
                } else if others != with_ids {
                    findings.push(Finding {
                        code: "S-003".into(),
                        path: entry.path.clone(),
                        own_spec: Some(own_spec.clone()),
                        owners: others.iter().cloned().collect(),
                        with: with_ids.iter().cloned().collect(),
                    });
                }
            }
            Role::ReadOnly => {}
        }

        entries.push(EvaluatedEntry {
            path: entry.path.clone(),
            role: entry.role,
            owners: owners.into_iter().collect(),
            with: if entry.role == Role::Shared {
                Some(with_ids.into_iter().collect())
            } else {
                None
            },
        });
    }

    // Findings are sorted by path, then code (§3.3); entries are already
    // sorted by path from `validate`, and paths are unique across roles.
    findings.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.code.cmp(&b.code)));

    Ok(ScopeEvaluation {
        id: req.id.clone(),
        own_spec,
        index_hash: index.build.content_hash.clone(),
        entries,
        findings,
    })
}

/// Evaluate a scope against the committed ledger (spec 108 §3.4). A stale
/// index is refused with [`Error::Stale`] (exit 1) before any path is
/// resolved.
pub fn evaluate(
    cfg: &Config,
    repo_root: &Path,
    req: &ScopeRequest,
) -> Result<ScopeEvaluation, Error> {
    crate::index::guard_committed_index(cfg, repo_root)?;
    let registry = crate::compile::load_committed_registry(cfg, repo_root)?;
    let index = crate::index::load_committed_index(cfg, repo_root)?;
    evaluate_scope(cfg, &registry, &index, req)
}

// ===== comparison (spec 108 §3.5) =====

/// One scope's identity as declared: `id` and `ownSpec` are carried through
/// unchanged, never resolved against a corpus (comparison reads no ledger).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub own_spec: String,
}

/// One overlapping declared entry, named by its path and role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeConflictEntry {
    pub path: String,
    pub role: Role,
}

/// One conflicting pair between two scopes (spec 108 §3.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub kind: String,
    pub a: ScopeConflictEntry,
    pub b: ScopeConflictEntry,
}

/// The answer to "do these two declared scopes conflict" (spec 108 §3.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeComparison {
    pub a: ScopeIdentity,
    pub b: ScopeIdentity,
    pub conflicts: Vec<Conflict>,
}

/// The conflict kind for one overlapping pair, or `None` when the roles do
/// not conflict (two `shared` entries, or two `readOnly` entries; spec 108
/// §3.5's table).
fn conflict_kind(a: Role, b: Role) -> Option<&'static str> {
    use Role::*;
    match (a, b) {
        (Mutable, Mutable) => Some("both-mutable"),
        (Mutable, Shared) | (Shared, Mutable) => Some("mutable-shared"),
        (Mutable, ReadOnly) | (ReadOnly, Mutable) => Some("changed-under-read"),
        (Shared, ReadOnly) | (ReadOnly, Shared) => Some("changed-under-read"),
        (Shared, Shared) => None,
        (ReadOnly, ReadOnly) => None,
    }
}

/// Compare two declared scopes for conflicting intentions (spec 108 §3.5).
/// Pure: a function of the two documents, which are validated as in §3.1 and
/// never resolved against a ledger.
pub fn compare_scopes(a: &ScopeRequest, b: &ScopeRequest) -> Result<ScopeComparison, Error> {
    let a_entries = validate(a)?;
    let b_entries = validate(b)?;

    let mut conflicts: Vec<Conflict> = Vec::new();
    for ea in &a_entries {
        for eb in &b_entries {
            if !paths_overlap(&ea.path, &eb.path) {
                continue;
            }
            let Some(kind) = conflict_kind(ea.role, eb.role) else {
                continue;
            };
            conflicts.push(Conflict {
                kind: kind.to_string(),
                a: ScopeConflictEntry {
                    path: ea.path.clone(),
                    role: ea.role,
                },
                b: ScopeConflictEntry {
                    path: eb.path.clone(),
                    role: eb.role,
                },
            });
        }
    }
    conflicts.sort_by(|x, y| {
        x.kind
            .cmp(&y.kind)
            .then_with(|| x.a.path.cmp(&y.a.path))
            .then_with(|| x.b.path.cmp(&y.b.path))
    });

    Ok(ScopeComparison {
        a: ScopeIdentity {
            id: a.id.clone(),
            own_spec: a.own_spec.clone(),
        },
        b: ScopeIdentity {
            id: b.id.clone(),
            own_spec: b.own_spec.clone(),
        },
        conflicts,
    })
}
