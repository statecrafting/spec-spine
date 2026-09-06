//! The query capability (spec 002): typed, read-only access over a loaded
//! registry. Because `Registry` is defined in `spec-spine-types`, these are free
//! functions rather than inherent methods (the orphan rule), but the surface is
//! the same: list / show / status_report / relationships / plan, plus
//! `load_registry`.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use spec_spine_types::{
    CodebaseIndex, Error, INDEX_SCHEMA_VERSION, Implementation, REGISTRY_SCHEMA_VERSION, Registry,
    Severity, SpecRecord, Status, Violation, parse_semver,
};

/// Parse `registry.json` bytes into a typed [`Registry`], rejecting an unknown
/// MAJOR schema version (the versioning policy: a build understands its own
/// MAJOR line only).
pub fn load_registry(bytes: &[u8]) -> Result<Registry, Error> {
    let registry: Registry = serde_json::from_slice(bytes)
        .map_err(|e| Error::Parse(format!("invalid registry.json: {e}")))?;
    reject_unknown_major("registry", &registry.spec_version, REGISTRY_SCHEMA_VERSION)?;
    Ok(registry)
}

/// Parse `index.json` bytes into a typed [`CodebaseIndex`], rejecting an unknown
/// MAJOR schema version. The index-side overlay seam.
pub fn load_index(bytes: &[u8]) -> Result<CodebaseIndex, Error> {
    let index: CodebaseIndex = serde_json::from_slice(bytes)
        .map_err(|e| Error::Parse(format!("invalid index.json: {e}")))?;
    reject_unknown_major("index", &index.schema_version, INDEX_SCHEMA_VERSION)?;
    Ok(index)
}

fn reject_unknown_major(what: &str, found: &str, ours: &str) -> Result<(), Error> {
    let (want_major, ..) = parse_semver(ours).expect("our own version constant is semver");
    let (got_major, ..) = parse_semver(found)
        .ok_or_else(|| Error::Schema(format!("{what} schemaVersion '{found}' is not semver")))?;
    if got_major != want_major {
        return Err(Error::Schema(format!(
            "{what} schema MAJOR {got_major} is unsupported (this build understands {want_major}.x)"
        )));
    }
    Ok(())
}

/// Filter for [`list`]. Extend additively as needs grow.
#[derive(Debug, Default, Clone)]
pub struct ListFilter {
    pub status: Option<Status>,
}

/// Specs matching `filter`, in registry (id) order.
pub fn list<'a>(registry: &'a Registry, filter: &ListFilter) -> Vec<&'a SpecRecord> {
    registry
        .specs
        .iter()
        .filter(|s| filter.status.is_none_or(|st| s.status == st))
        .collect()
}

/// The `--ids-only` projection of [`list`] (spec 010 §3.1): the same filter and
/// order, reduced to bare spec ids.
pub fn list_ids<'a>(registry: &'a Registry, filter: &ListFilter) -> Vec<&'a str> {
    list(registry, filter)
        .iter()
        .map(|s| s.id.as_str())
        .collect()
}

/// One spec by id, or [`Error::NotFound`].
pub fn show<'a>(registry: &'a Registry, id: &str) -> Result<&'a SpecRecord, Error> {
    registry
        .specs
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| Error::NotFound(format!("spec '{id}'")))
}

/// Counts of specs by status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusReport {
    pub total: usize,
    pub draft: usize,
    pub approved: usize,
    pub superseded: usize,
    pub retired: usize,
}

/// The `--nonzero-only` projection of a [`StatusReport`] (spec 010 §3.2):
/// zero-count statuses are omitted from serialization; `total` always
/// serializes and still reflects the whole corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusReportNonzero {
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retired: Option<usize>,
}

impl StatusReport {
    /// Project to the `--nonzero-only` form.
    pub fn nonzero_only(&self) -> StatusReportNonzero {
        let keep = |n: usize| (n > 0).then_some(n);
        StatusReportNonzero {
            total: self.total,
            draft: keep(self.draft),
            approved: keep(self.approved),
            superseded: keep(self.superseded),
            retired: keep(self.retired),
        }
    }
}

/// Tally specs by status.
pub fn status_report(registry: &Registry) -> StatusReport {
    let mut r = StatusReport {
        total: registry.specs.len(),
        draft: 0,
        approved: 0,
        superseded: 0,
        retired: 0,
    };
    for spec in &registry.specs {
        match spec.status {
            Status::Draft => r.draft += 1,
            Status::Approved => r.approved += 1,
            Status::Superseded => r.superseded += 1,
            Status::Retired => r.retired += 1,
        }
    }
    r
}

/// The relationship neighborhood of a spec: its outgoing id-edges and the
/// incoming edges that target it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipView {
    pub id: String,
    // outgoing
    pub depends_on: Vec<String>,
    pub supersedes: Vec<String>,
    pub amends: Vec<String>,
    // incoming (computed by scanning the corpus)
    pub superseded_by: Vec<String>,
    pub amended_by: Vec<String>,
    pub depended_on_by: Vec<String>,
}

/// Build the relationship view for `id`, or [`Error::NotFound`].
pub fn relationships(registry: &Registry, id: &str) -> Result<RelationshipView, Error> {
    let spec = show(registry, id)?;
    let incoming = |pick: fn(&SpecRecord) -> &Vec<String>| -> Vec<String> {
        registry
            .specs
            .iter()
            .filter(|other| pick(other).iter().any(|t| t == id))
            .map(|other| other.id.clone())
            .collect()
    };
    // `supersedes` carries structured items (spec 019); the relationship view
    // is id-only, so project each item to its predecessor id.
    let superseded_by: Vec<String> = registry
        .specs
        .iter()
        .filter(|other| other.supersedes.iter().any(|x| x.spec() == id))
        .map(|other| other.id.clone())
        .collect();
    Ok(RelationshipView {
        id: spec.id.clone(),
        depends_on: spec.depends_on.clone(),
        supersedes: spec
            .supersedes
            .iter()
            .map(|x| x.spec().to_string())
            .collect(),
        amends: spec.amends.clone(),
        superseded_by,
        amended_by: incoming(|s| &s.amends),
        depended_on_by: incoming(|s| &s.depends_on),
    })
}

// ===== spec 038: `registry plan` =====

/// One unfinished dependency, with the state that makes it a blocker.
///
/// `state` is the blocker's `implementation` value, or the literal `unresolved`
/// for a `depends_on` target absent from the registry. It is not decoration: a
/// `deferred` blocker appears in neither output set while still blocking, so
/// without its state that entry would name a spec the reader can find nowhere
/// else in the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Blocker {
    pub id: String,
    pub state: String,
}

/// A spec that cannot be scheduled yet, and every reason why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedSpec {
    pub id: String,
    pub blocked_by: Vec<Blocker>,
}

/// The scheduling projection: what can be worked on now, and what cannot.
///
/// **Both sets are ordered, and both orderings are part of the contract.**
/// `ready` is topological over `depends_on` with ties by ascending id;
/// `blocked` is ascending id, and each entry's `blocked_by` follows that spec's
/// own authored `depends_on` order. Spec 038 3.2 requires the report to be a
/// pure function of the corpus rather than of a hash-map iteration order, and
/// that covers the whole document: a `--json` consumer diffing `blocked` across
/// runs is relying on a stated guarantee, not on the `BTreeMap` that happens to
/// produce it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub ready: Vec<String>,
    pub blocked: Vec<BlockedSpec>,
}

/// Partition the corpus into the ready set and the blocked set (spec 038).
///
/// **This answers what is *claimed*, never what is done.** `implementation` is
/// self-declared and no gate verifies that a spec marked `complete` has any
/// implemented code, so `plan` is the correct input to scheduling and the wrong
/// input to acceptance. A scheduler that reads a claim and an adjudicator that
/// verifies one must stay different mechanisms, or the claim becomes its own
/// proof.
///
/// A spec is **excluded** from both sets when the corpus has moved past it
/// (`status` `superseded` or `retired`) or when someone has already answered
/// "should this be scheduled" with no (`implementation` `complete`, `n-a` or
/// `deferred`). An absent `implementation` takes its answer from `status`
/// (spec 045): on a `draft` it reads as `pending`, because an unstated
/// intention is the same input to a scheduler as a stated intention to start;
/// on anything ratified it reads as settled, which is what the gate already
/// concludes about the same key (041 3.1, 044 3.1) and what keeps a bootstrap
/// spec that owns no code from being offered as ready forever.
///
/// Returns [`Error::Validation`] naming the path if `depends_on` contains a
/// cycle. Spec 033 refuses one at compile time, so a registry that exists is
/// acyclic; this guards a hand-edited shard or one written by another tool
/// version, and it terminates rather than looping or truncating.
///
/// That unreachability holds only while both walks cover the same specs.
/// `compile` passes every record to its detector with no `status` filter, and
/// this walks every entry in the registry, so the two agree today; nothing in
/// either function enforces the pairing, which is why
/// `plan_and_compile_agree_on_a_cycle_among_retired_specs` asserts it. If
/// `compile`'s check were ever scoped to active specs, `plan` would begin
/// refusing corpora `compile` accepts, and that test is what would catch it.
pub fn plan(registry: &Registry) -> Result<Plan, Error> {
    let by_id: BTreeMap<&str, &SpecRecord> =
        registry.specs.iter().map(|s| (s.id.as_str(), s)).collect();

    if let Some(cycle) = find_cycle(&by_id) {
        return Err(Error::Validation(vec![Violation {
            code: "V-014".to_string(),
            severity: Severity::Error,
            message: format!(
                "depends_on cycle refuses scheduling: {}",
                cycle.join(" -> ")
            ),
            path: None,
        }]));
    }

    let mut ready: Vec<&str> = Vec::new();
    let mut blocked: Vec<BlockedSpec> = Vec::new();

    // `by_id` is a BTreeMap, so this walks ids in ascending order and both
    // output vectors are built deterministically without a later sort.
    for (id, spec) in &by_id {
        if !schedulable(spec) {
            continue;
        }
        let blocked_by: Vec<Blocker> = spec
            .depends_on
            .iter()
            .filter_map(|dep| {
                blocker_state(&by_id, dep).map(|state| Blocker {
                    id: dep.clone(),
                    state,
                })
            })
            .collect();
        if blocked_by.is_empty() {
            ready.push(id);
        } else {
            blocked.push(BlockedSpec {
                id: (*id).to_string(),
                blocked_by,
            });
        }
    }

    Ok(Plan {
        ready: topological(&ready, &by_id),
        blocked,
    })
}

/// Whether this spec should be offered to a scheduler at all.
///
/// `deferred` sits beside `complete` here, and the two carry opposite
/// information about whether work remains, so the reason is worth stating.
/// `plan` does not ask "is this finished", it asks "should this be scheduled",
/// and to both the answer is no. `deferred` is the one value in the enum that is
/// a **decision** rather than a report: someone looked at the spec and took it
/// off the schedule. Offering it anyway would overrule that, and quietly
/// returning it to `ready` when a dependency landed would make deferral expire
/// on its own, which is not what deferring something means.
///
/// An absent key is not a fifth value with its own row. It defers to `status`
/// (spec 045): `draft` + absent is schedulable, exactly as `draft` + `pending`
/// is; anything else + absent is settled and is not offered. The alternative,
/// reading absence as `pending` regardless of `status`, made `plan` disagree
/// with `index` about the same spec and offered every `init`-scaffolded
/// bootstrap spec as the one ready item in an otherwise finished corpus.
fn schedulable(spec: &SpecRecord) -> bool {
    if matches!(spec.status, Status::Superseded | Status::Retired) {
        return false;
    }
    match spec.implementation {
        Some(Implementation::Complete | Implementation::Na | Implementation::Deferred) => false,
        Some(Implementation::Pending | Implementation::InProgress) => true,
        None => spec.status == Status::Draft,
    }
}

/// The state of `dep` if it blocks, or `None` if it is finished.
///
/// A dependency is finished only when its `implementation` is `complete` or
/// `n-a`, or when the key is absent on a spec that is not a `draft` (spec 045:
/// an absent key defers to `status`, and a ratified spec with nothing to say
/// about its implementation is settled, the same reading `index` gives it). A
/// target absent from the registry blocks as `unresolved`: a dangling
/// dependency is a corpus defect, and reading "the blocker does not exist" as
/// "the blocker is satisfied" would hand out work in an order the corpus does
/// not sanction.
fn blocker_state(by_id: &BTreeMap<&str, &SpecRecord>, dep: &str) -> Option<String> {
    let Some(spec) = by_id.get(dep) else {
        return Some("unresolved".to_string());
    };
    match spec.implementation {
        Some(Implementation::Complete | Implementation::Na) => None,
        Some(Implementation::InProgress) => Some("in-progress".to_string()),
        Some(Implementation::Deferred) => Some("deferred".to_string()),
        Some(Implementation::Pending) => Some("pending".to_string()),
        None if spec.status == Status::Draft => Some("pending".to_string()),
        None => None,
    }
}

/// The ready set in topological order over `depends_on`, ties by ascending id.
///
/// Kahn's algorithm with an id-ordered frontier, which is what makes the output
/// a pure function of the corpus rather than of a hash-map iteration order.
///
/// Under the current partition this is **equivalent to sorting by id**, and the
/// walk provides no ordering guarantee a `sort()` would not: the ready set
/// contains no edges among its own members, because a spec depending on an
/// unfinished spec is blocked rather than ready and a finished one is excluded,
/// so every member's restricted dependency set is empty and all of them enter
/// the frontier at once. `plan_ready_set_has_no_internal_edges` pins that.
///
/// It is kept rather than replaced by the sort because the degeneracy is a
/// property of `schedulable` and `blocker_state`, not of this function: a later
/// change to either could introduce an intra-set edge, and the walk is what
/// keeps that from silently emitting a dependent before its dependency.
fn topological(ready: &[&str], by_id: &BTreeMap<&str, &SpecRecord>) -> Vec<String> {
    let members: BTreeSet<&str> = ready.iter().copied().collect();
    let mut remaining: BTreeMap<&str, BTreeSet<&str>> = members
        .iter()
        .map(|id| {
            let deps = by_id[id]
                .depends_on
                .iter()
                .map(String::as_str)
                .filter(|d| members.contains(d))
                .collect();
            (*id, deps)
        })
        .collect();

    let mut out: Vec<String> = Vec::with_capacity(members.len());
    while !remaining.is_empty() {
        // Ascending id among everything currently unblocked: the tiebreak.
        let Some(next) = remaining
            .iter()
            .find(|(_, deps)| deps.is_empty())
            .map(|(id, _)| *id)
        else {
            // Unreachable: `plan` refused a cycle before calling this, and the
            // restriction to `members` cannot create one. Emitting the rest in
            // id order beats looping if the invariant is ever broken elsewhere.
            debug_assert!(false, "topological: no unblocked spec but work remains");
            out.extend(remaining.keys().map(|id| (*id).to_string()));
            break;
        };
        remaining.remove(next);
        for deps in remaining.values_mut() {
            deps.remove(next);
        }
        out.push(next.to_string());
    }
    out
}

/// The first `depends_on` cycle reachable in the graph, as the path that closes
/// it, or `None` when the graph is acyclic.
///
/// Colouring DFS over ids in ascending order, so the cycle reported for a given
/// registry is always the same one. Unvisited is absence from `colour` rather
/// than a third variant, so the map holds only what the walk has reached.
///
/// The walk carries its own stack rather than recursing. This runs only on
/// input `compile` refuses to emit, which is precisely the input that may be
/// arbitrarily malformed: a recursive walk over a hand-edited shard with a long
/// enough chain overflows the stack and aborts the process, which is a worse
/// outcome than the looping this guard exists to prevent, and one outside the
/// documented exit-code contract entirely.
fn find_cycle(by_id: &BTreeMap<&str, &SpecRecord>) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq)]
    enum Colour {
        Grey,
        Black,
    }
    // One frame per spec being explored: the spec, and how far through its
    // `depends_on` list the walk has got.
    struct Frame<'a> {
        id: &'a str,
        next_dep: usize,
    }

    let mut colour: BTreeMap<&str, Colour> = BTreeMap::new();

    for root in by_id.keys() {
        if colour.contains_key(root) {
            continue;
        }
        let mut stack: Vec<Frame<'_>> = vec![Frame {
            id: root,
            next_dep: 0,
        }];
        colour.insert(root, Colour::Grey);

        while let Some(frame) = stack.last_mut() {
            let deps = &by_id[frame.id].depends_on;
            let Some(dep) = deps.get(frame.next_dep) else {
                // Every dependency explored: this spec is off the current path.
                colour.insert(frame.id, Colour::Black);
                stack.pop();
                continue;
            };
            frame.next_dep += 1;

            // A dangling dependency is not a cycle; `blocker_state` reports it.
            let Some((dep_id, _)) = by_id.get_key_value(dep.as_str()) else {
                continue;
            };
            match colour.get(dep_id) {
                // Re-entering a spec still on the path closes a cycle.
                Some(Colour::Grey) => {
                    // Grey means "on the current path", so the frame is always
                    // found. Asserted rather than trusted: the fallback would
                    // otherwise report a path starting at the wrong frame, and
                    // any cycle test still passes under that misattribution
                    // because the whole stack contains the cycle's ids.
                    let from = stack.iter().position(|f| f.id == *dep_id);
                    debug_assert!(from.is_some(), "grey node absent from the walk stack");
                    let from = from.unwrap_or(0);
                    let mut cycle: Vec<String> =
                        stack[from..].iter().map(|f| f.id.to_string()).collect();
                    cycle.push((*dep_id).to_string());
                    return Some(cycle);
                }
                Some(Colour::Black) => {}
                None => {
                    colour.insert(dep_id, Colour::Grey);
                    stack.push(Frame {
                        id: dep_id,
                        next_dep: 0,
                    });
                }
            }
        }
    }
    None
}
