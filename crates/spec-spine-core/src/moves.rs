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

/// One node's expansion, on the explicit work stack (spec 111 §3.4, D-11):
/// `node`'s declared targets, already resolved and sorted when the frame was
/// pushed, walked one at a time so the traversal never recurses.
struct Frame {
    node: String,
    targets: Vec<String>,
    next_idx: usize,
}

/// The iterative walk's state (D-11): an explicit stack replaces recursion
/// (unbounded native-stack depth on a long chain is a process abort, which
/// core's "panic-free on user input" invariant forbids), and `done` memoizes
/// every fully-expanded node so a DAG with reconverging branches (a `split`
/// followed by another `split`, and so on) expands each node exactly once
/// instead of once per root-to-leaf path.
///
/// `on_stack` (gray, in [`Self::on_stack`] order for cycle-chain
/// construction, mirrored in `on_stack_set` for lookup) and `done` (black)
/// are the classic DFS three-color coloring: white (absent from both) is
/// unvisited, gray is mid-expansion (an edge into a gray node is a back edge,
/// a cycle), black is fully resolved (an edge into a black node is safe to
/// reuse without re-expanding, and cannot itself close a cycle, because
/// every node reachable from it was already proven acyclic before it turned
/// black). This is traversal-order-independent: a graph reachable from
/// `path` has a cycle if and only if this single DFS finds a back edge,
/// regardless of which branch order `resolve_groups`' sorted targets walk
/// first, which is what lets a cycle reachable only through one branch of a
/// `split` (never the other) still be found.
struct Walker<'a> {
    index: &'a BTreeMap<String, Vec<StepGroup>>,
    hops: Vec<Hop>,
    terminals: BTreeSet<Terminal>,
    on_stack: Vec<String>,
    on_stack_set: BTreeSet<String>,
    done: BTreeSet<String>,
}

impl<'a> Walker<'a> {
    fn new(index: &'a BTreeMap<String, Vec<StepGroup>>) -> Self {
        Walker {
            index,
            hops: Vec::new(),
            terminals: BTreeSet::new(),
            on_stack: Vec::new(),
            on_stack_set: BTreeSet::new(),
            done: BTreeSet::new(),
        }
    }

    /// Expand `node` for the first time: resolve its declared group (or
    /// propagate [`Stop::Ambiguous`]), record its hop(s) - each distinct
    /// `(from, to, kind)` exactly once, since a node is only ever expanded
    /// once - and either close it immediately (`removed`, which has no
    /// further edges) or open it as a new stack frame. The caller guarantees
    /// `node` is neither `on_stack` nor `done`.
    fn expand(&mut self, node: &str, stack: &mut Vec<Frame>) -> Result<(), Stop> {
        // The caller only expands a path it already confirmed is in `index`
        // (the initial, checked query path, or a target [`Self::visit`]
        // confirmed), so this is present.
        let groups = self.index.get(node).expect("expanded only an indexed path");
        let resolved = resolve_groups(node, groups)?;

        if resolved.kind == MoveKind::Removed {
            self.hops.push(Hop {
                from: node.to_string(),
                to: None,
                kind: resolved.kind,
                declared_by: resolved.declared_by,
                answered_by: resolved.answered_by.clone(),
            });
            self.terminals.insert(Terminal {
                path: None,
                answered_by: resolved.answered_by,
            });
            self.done.insert(node.to_string());
            return Ok(());
        }

        let mut targets = resolved.targets;
        targets.sort();
        for to in &targets {
            self.hops.push(Hop {
                from: node.to_string(),
                to: Some(to.clone()),
                kind: resolved.kind,
                declared_by: resolved.declared_by.clone(),
                answered_by: None,
            });
        }
        self.on_stack.push(node.to_string());
        self.on_stack_set.insert(node.to_string());
        stack.push(Frame {
            node: node.to_string(),
            targets,
            next_idx: 0,
        });
        Ok(())
    }

    /// Step into `target` from the frame currently being walked: a back edge
    /// into a gray (`on_stack`) node is a cycle; an edge into a black
    /// (`done`) node, or a path nothing declares a further step from, needs
    /// no further work (its hops/terminals were already recorded, or it is
    /// an ordinary terminus); anything else is expanded for the first time.
    fn visit(&mut self, target: &str, stack: &mut Vec<Frame>) -> Result<(), Stop> {
        if self.on_stack_set.contains(target) {
            let pos = self
                .on_stack
                .iter()
                .position(|p| p == target)
                .expect("on_stack_set and on_stack agree");
            let mut chain: Vec<String> = self.on_stack[pos..].to_vec();
            chain.push(target.to_string());
            return Err(Stop::Cycle { chain });
        }
        if self.done.contains(target) || !self.index.contains_key(target) {
            if !self.index.contains_key(target) {
                self.terminals.insert(Terminal {
                    path: Some(target.to_string()),
                    answered_by: None,
                });
            }
            return Ok(());
        }
        self.expand(target, stack)
    }
}

/// Look up `path` against every declared move in `registry` (spec 111 §3.4).
/// Pure: consults nothing else, and consults no similarity, content or
/// timing signal (§3.6). Iterative (D-11): an explicit work stack, so
/// neither a long declared chain nor a deep reconverging `split` DAG can
/// overflow the native call stack or re-expand a shared node once per path
/// reaching it.
pub fn lookup(registry: &Registry, path: &str) -> MoveLookup {
    let index = build_step_index(registry);
    if !index.contains_key(path) {
        return MoveLookup::Unmapped {
            path: path.to_string(),
        };
    }

    let mut walker = Walker::new(&index);
    let mut stack: Vec<Frame> = Vec::new();
    let result = (|| -> Result<(), Stop> {
        walker.expand(path, &mut stack)?;
        while let Some(frame) = stack.last_mut() {
            if frame.next_idx >= frame.targets.len() {
                let node = frame.node.clone();
                stack.pop();
                walker.on_stack.pop();
                walker.on_stack_set.remove(&node);
                walker.done.insert(node);
                continue;
            }
            let target = frame.targets[frame.next_idx].clone();
            frame.next_idx += 1;
            // `frame` is not used again this iteration, so its mutable
            // borrow of `stack` ends here (NLL), freeing it for `visit`.
            walker.visit(&target, &mut stack)?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => {
            // Deterministic regardless of traversal order: sorted by
            // `(from, to)`, which is already unique per hop (§3.4's
            // dedupe-on-`(from, to, kind)`, guaranteed by construction since
            // `expand` runs at most once per node).
            let mut hops = walker.hops;
            hops.sort_by(|a, b| (&a.from, &a.to).cmp(&(&b.from, &b.to)));
            MoveLookup::Resolved {
                path: path.to_string(),
                hops,
                terminals: walker.terminals.into_iter().collect(),
            }
        }
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
