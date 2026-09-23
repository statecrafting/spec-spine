// Spec: specs/111-a-move-is-a-reviewed-mapping/spec.md
//! The move lookup (spec 111 §3.4): a path in, an answer derived from every
//! spec's declared moves in the committed registry, never a guess. Pure over
//! its argument, like [`crate::impact::impacts`]: it consults nothing but the
//! `Registry` it is handed, and it changes no verdict (§3.5) and infers
//! nothing (§3.6).
//!
//! Four outcomes, and no fifth: `unmapped` (no declaration names this path as
//! a `from`), `resolved` (every declared step was followed to a terminal),
//! `ambiguous` (two declarations disagree about the same `from`), and `cycle`
//! (following declared steps returns to a path already on the current
//! chain). A `split` entry's several branches come from one declaration and
//! are followed in parallel, never treated as ambiguous with each other
//! (§3.4).

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use spec_spine_types::{MoveKind, Registry};

/// One step the lookup followed (spec 111 §3.4): the declaring spec(s), the
/// declared kind, and where it led. `declared_by` is a list rather than a
/// single id so that two specs declaring the byte-identical step (D-6) can be
/// named on one hop instead of forcing an arbitrary pick between them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hop {
    pub from: String,
    /// Absent for a `removed` step, which has no `to` path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    pub kind: MoveKind,
    pub declared_by: Vec<String>,
    /// Present only on a `removed` hop that names one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answered_by: Option<String>,
}

/// One end of a resolved chain (spec 111 §3.4): either a path with no
/// declaration naming it as a `from` (an ordinary terminus), or a `removed`
/// step's end, which has no path, only (optionally) the spec that now
/// answers for it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Terminal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answered_by: Option<String>,
}

/// One of the disagreeing declarations the lookup found at an ambiguous
/// `from` (spec 111 §3.4): every candidate is reported, none is picked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AmbiguousCandidate {
    pub declared_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    pub kind: MoveKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answered_by: Option<String>,
}

/// The lookup's answer (spec 111 §3.4): exactly the four outcomes, tagged by
/// `outcome` so a consumer dispatches on one member rather than presence/
/// absence of others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum MoveLookup {
    /// No declaration in the corpus names `path` as a `from`.
    Unmapped { path: String },
    /// Every declared step from `path` was followed to a terminal.
    Resolved {
        path: String,
        hops: Vec<Hop>,
        terminals: Vec<Terminal>,
    },
    /// More than one declaration names the same `from` (here, `at`) with a
    /// different outcome. The lookup stops at the first such point it meets
    /// while following `path`'s chain; it never picks a candidate.
    Ambiguous {
        path: String,
        at: String,
        candidates: Vec<AmbiguousCandidate>,
    },
    /// Following declared steps from `path` returned to a path already on
    /// the chain. `chain` names the loop, starting and ending at the
    /// repeated path.
    Cycle { path: String, chain: Vec<String> },
}

/// One flattened declaration (spec 111 §3.4): the answer `registry moves`
/// gives with no path argument, "the derived path map". A `split` or
/// `merged` declaration expands to one entry per branch/source; a `relocated`
/// or `removed` declaration is already one entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveEntry {
    pub from: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    pub kind: MoveKind,
    pub declared_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answered_by: Option<String>,
}

/// One declaration's contribution to a single `from` path: every declaration
/// naming a given path as (one of) its `from`(s) contributes exactly one of
/// these to that path's entry in the step index, so a `split`'s several
/// branches - one declaration, several `to`s - are one group, and never
/// mistaken for disagreeing declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StepGroup {
    declared_by: String,
    kind: MoveKind,
    /// Empty for `removed`.
    targets: Vec<String>,
    /// Only meaningful for `removed`.
    answered_by: Option<String>,
}

/// [`StepGroup`]s at one `from` path, resolved past duplication (§3.4's
/// byte-identical-declaration case, D-6): the same declared outcome from more
/// than one spec collapses to one group naming every declaring spec.
struct ResolvedGroup {
    declared_by: Vec<String>,
    kind: MoveKind,
    targets: Vec<String>,
    answered_by: Option<String>,
}

/// Why the walk stopped short of every path resolving (spec 111 §3.4): a
/// `path` field is not carried here because both cases fold into
/// [`MoveLookup`] one level up, at the query path the caller asked about.
enum Stop {
    Ambiguous {
        at: String,
        candidates: Vec<AmbiguousCandidate>,
    },
    Cycle {
        chain: Vec<String>,
    },
}

/// Every declaration in `registry`, indexed by the `from` path(s) it names.
/// A `merged` declaration contributes one group per source path (all sharing
/// the one `to`); a `split` declaration contributes one group, carrying every
/// branch, under its one `from`.
fn build_step_index(registry: &Registry) -> BTreeMap<String, Vec<StepGroup>> {
    let mut index: BTreeMap<String, Vec<StepGroup>> = BTreeMap::new();
    for r in &registry.specs {
        for mv in &r.moves {
            let froms = mv.from.paths();
            match mv.kind {
                MoveKind::Relocated | MoveKind::Split => {
                    if let Some(from) = froms.first() {
                        let targets: Vec<String> = mv
                            .to
                            .as_ref()
                            .map(|t| t.paths().into_iter().map(str::to_string).collect())
                            .unwrap_or_default();
                        index
                            .entry((*from).to_string())
                            .or_default()
                            .push(StepGroup {
                                declared_by: r.id.clone(),
                                kind: mv.kind,
                                targets,
                                answered_by: None,
                            });
                    }
                }
                MoveKind::Merged => {
                    let to = mv
                        .to
                        .as_ref()
                        .and_then(|t| t.paths().into_iter().next())
                        .map(str::to_string);
                    for from in froms {
                        index.entry(from.to_string()).or_default().push(StepGroup {
                            declared_by: r.id.clone(),
                            kind: mv.kind,
                            targets: to.clone().into_iter().collect(),
                            answered_by: None,
                        });
                    }
                }
                MoveKind::Removed => {
                    if let Some(from) = froms.first() {
                        index
                            .entry((*from).to_string())
                            .or_default()
                            .push(StepGroup {
                                declared_by: r.id.clone(),
                                kind: mv.kind,
                                targets: Vec::new(),
                                answered_by: mv.answered_by.clone(),
                            });
                    }
                }
            }
        }
    }
    index
}

/// Resolve every [`StepGroup`] declared at `at` into one [`ResolvedGroup`],
/// or [`Stop::Ambiguous`] when they disagree (spec 111 §3.4). A single group
/// is never ambiguous. More than one group is ambiguous unless every group
/// is byte-identical (same kind, same target set, same `answered_by`), in
/// which case they name the same step and collapse to one, carrying every
/// declaring spec (D-6).
fn resolve_groups(at: &str, groups: &[StepGroup]) -> Result<ResolvedGroup, Stop> {
    let first = &groups[0];
    let same_targets = |a: &[String], b: &[String]| -> bool {
        let mut a: Vec<&String> = a.iter().collect();
        let mut b: Vec<&String> = b.iter().collect();
        a.sort();
        b.sort();
        a == b
    };
    let all_identical = groups.iter().all(|g| {
        g.kind == first.kind
            && same_targets(&g.targets, &first.targets)
            && g.answered_by == first.answered_by
    });
    if all_identical {
        let mut declared_by: Vec<String> = groups.iter().map(|g| g.declared_by.clone()).collect();
        declared_by.sort();
        declared_by.dedup();
        return Ok(ResolvedGroup {
            declared_by,
            kind: first.kind,
            targets: first.targets.clone(),
            answered_by: first.answered_by.clone(),
        });
    }

    let mut candidates: Vec<AmbiguousCandidate> = Vec::new();
    for g in groups {
        if g.kind == MoveKind::Removed {
            candidates.push(AmbiguousCandidate {
                declared_by: g.declared_by.clone(),
                to: None,
                kind: g.kind,
                answered_by: g.answered_by.clone(),
            });
        } else {
            for t in &g.targets {
                candidates.push(AmbiguousCandidate {
                    declared_by: g.declared_by.clone(),
                    to: Some(t.clone()),
                    kind: g.kind,
                    answered_by: None,
                });
            }
        }
    }
    candidates.sort_by(|a, b| (&a.declared_by, &a.to).cmp(&(&b.declared_by, &b.to)));
    Err(Stop::Ambiguous {
        at: at.to_string(),
        candidates,
    })
}

/// Follow declared steps from `current`, recording every [`Hop`] and
/// [`Terminal`] reached, until nothing further is declared, a cycle returns
/// to a path already on `ancestry`, or a step is ambiguous.
fn walk(
    current: &str,
    index: &BTreeMap<String, Vec<StepGroup>>,
    ancestry: &mut Vec<String>,
    hops: &mut Vec<Hop>,
    terminals: &mut BTreeSet<Terminal>,
) -> Result<(), Stop> {
    // The caller only recurses into a path it already confirmed is in
    // `index` (or is the initial, checked query path), so this is present.
    let groups = index
        .get(current)
        .expect("walked only into an indexed path");
    let resolved = resolve_groups(current, groups)?;

    if resolved.kind == MoveKind::Removed {
        hops.push(Hop {
            from: current.to_string(),
            to: None,
            kind: resolved.kind,
            declared_by: resolved.declared_by,
            answered_by: resolved.answered_by.clone(),
        });
        terminals.insert(Terminal {
            path: None,
            answered_by: resolved.answered_by,
        });
        return Ok(());
    }

    let mut targets = resolved.targets;
    targets.sort();
    for to in targets {
        hops.push(Hop {
            from: current.to_string(),
            to: Some(to.clone()),
            kind: resolved.kind,
            declared_by: resolved.declared_by.clone(),
            answered_by: None,
        });
        if let Some(pos) = ancestry.iter().position(|p| p == &to) {
            let mut chain: Vec<String> = ancestry[pos..].to_vec();
            chain.push(to);
            return Err(Stop::Cycle { chain });
        }
        if index.contains_key(&to) {
            ancestry.push(to.clone());
            walk(&to, index, ancestry, hops, terminals)?;
            ancestry.pop();
        } else {
            terminals.insert(Terminal {
                path: Some(to),
                answered_by: None,
            });
        }
    }
    Ok(())
}

/// Look up `path` against every declared move in `registry` (spec 111 §3.4).
/// Pure: consults nothing else, and consults no similarity, content or
/// timing signal (§3.6).
pub fn lookup(registry: &Registry, path: &str) -> MoveLookup {
    let index = build_step_index(registry);
    if !index.contains_key(path) {
        return MoveLookup::Unmapped {
            path: path.to_string(),
        };
    }
    let mut hops: Vec<Hop> = Vec::new();
    let mut terminals: BTreeSet<Terminal> = BTreeSet::new();
    let mut ancestry: Vec<String> = vec![path.to_string()];
    match walk(path, &index, &mut ancestry, &mut hops, &mut terminals) {
        Ok(()) => MoveLookup::Resolved {
            path: path.to_string(),
            hops,
            terminals: terminals.into_iter().collect(),
        },
        Err(Stop::Ambiguous { at, candidates }) => MoveLookup::Ambiguous {
            path: path.to_string(),
            at,
            candidates,
        },
        Err(Stop::Cycle { chain }) => MoveLookup::Cycle {
            path: path.to_string(),
            chain,
        },
    }
}

/// Every declared move in `registry`, flattened to one entry per branch/
/// source and sorted by `(from, to, declaredBy)` (spec 111 §3.4): the answer
/// `registry moves` gives with no path argument.
pub fn flattened_moves(registry: &Registry) -> Vec<MoveEntry> {
    let mut entries: Vec<MoveEntry> = Vec::new();
    for r in &registry.specs {
        for mv in &r.moves {
            let froms = mv.from.paths();
            match mv.kind {
                MoveKind::Relocated | MoveKind::Split => {
                    if let Some(from) = froms.first() {
                        let targets: Vec<&str> =
                            mv.to.as_ref().map(|t| t.paths()).unwrap_or_default();
                        for to in targets {
                            entries.push(MoveEntry {
                                from: (*from).to_string(),
                                to: Some(to.to_string()),
                                kind: mv.kind,
                                declared_by: r.id.clone(),
                                answered_by: None,
                            });
                        }
                    }
                }
                MoveKind::Merged => {
                    let to = mv.to.as_ref().and_then(|t| t.paths().into_iter().next());
                    for from in froms {
                        entries.push(MoveEntry {
                            from: from.to_string(),
                            to: to.map(str::to_string),
                            kind: mv.kind,
                            declared_by: r.id.clone(),
                            answered_by: None,
                        });
                    }
                }
                MoveKind::Removed => {
                    if let Some(from) = froms.first() {
                        entries.push(MoveEntry {
                            from: (*from).to_string(),
                            to: None,
                            kind: mv.kind,
                            declared_by: r.id.clone(),
                            answered_by: mv.answered_by.clone(),
                        });
                    }
                }
            }
        }
    }
    entries.sort_by(|a, b| (&a.from, &a.to, &a.declared_by).cmp(&(&b.from, &b.to, &b.declared_by)));
    entries
}
