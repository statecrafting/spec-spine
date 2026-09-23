//! `spec-spine registry …`: typed, read-only queries over the compiled
//! registry. Assembles the registry from its committed per-spec shards via the
//! library (never ad-hoc parsing, per spec 000 §1; the shard tree replaces the
//! monolithic `registry.json` since spec 022).

use std::path::Path;

use clap::Subcommand;
use spec_spine_core::{
    ListFilter, MoveLookup, Plan, Versioning, flattened_moves, list, list_ids,
    load_committed_registry, lookup_move, plan, read_document, relationships, shard_content_hash,
    show, status_report,
};
use spec_spine_types::{Error, Status};

use crate::load_repo_config;

#[derive(Subcommand)]
pub enum RegistryQuery {
    /// List specs (optionally filtered by status).
    List {
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,
        #[arg(long)]
        json: bool,
        /// Print bare spec ids, one per line (a JSON string array with --json).
        #[arg(long)]
        ids_only: bool,
    },
    /// Show one spec by id.
    Show {
        /// Spec id, full (`015-short-id-resolution`) or short (`016`).
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Counts of specs by status.
    StatusReport {
        #[arg(long)]
        json: bool,
        /// Omit statuses whose count is zero (the total still covers the corpus).
        #[arg(long)]
        nonzero_only: bool,
    },
    /// Show a spec's relationship neighborhood.
    Relationships {
        /// Spec id, full (`015-short-id-resolution`) or short (`016`).
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Resolve a qualified obligation reference, `<spec-id>#<obligation-id>`
    /// (spec 106). An unqualified id is refused, never resolved locally.
    Obligation {
        /// `<spec-id>#<obligation-id>`; the spec half may be short (`106`).
        reference: String,
        #[arg(long)]
        json: bool,
    },
    /// Resolve a context closure against the committed ledger (spec 107): every
    /// member's identity and one order-independent digest. Refuses a stale
    /// registry (exit 2).
    Closure {
        /// A closure request document, or `-` for stdin.
        #[arg(long, value_name = "FILE")]
        request: String,
        #[arg(long)]
        json: bool,
    },
    /// Every declared impact and conflict (spec 109), inverted so the target
    /// side can see what was declared about it. `--target` is a spec id
    /// (every obligation it declares) or a qualified `<spec-id>#<obligation-id>`
    /// reference; `--declared-by` is a spec id; both compose by intersection.
    Impacts {
        #[arg(long, value_name = "REF")]
        target: Option<String>,
        #[arg(long, value_name = "SPEC")]
        declared_by: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Look up a path against every declared move (spec 111 §3.4). Without a
    /// path, lists every declaration, flattened and sorted: the derived path
    /// map. Answers from the committed registry, like `show`; refuses
    /// nothing about the working tree. Exits 1 on `ambiguous` or `cycle`.
    Moves {
        /// The path to look up. Omit to list every declared move.
        path: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Which specs can be worked on now, and what blocks the rest (spec 035).
    Plan {
        #[arg(long)]
        json: bool,
        /// Print only the single pick: the first ready spec (spec 053). An
        /// empty ready set is `(nothing ready)` at exit 0, not a failure.
        #[arg(long)]
        next: bool,
    },
}

/// Returns `0` on success; `NotFound`/parse/schema errors propagate to the
/// caller's exit-code mapping.
pub fn run(repo: &Path, query: &RegistryQuery) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;
    let registry = load_committed_registry(&cfg, repo)?;

    match query {
        RegistryQuery::List {
            status,
            json,
            ids_only,
        } => {
            let filter = ListFilter {
                status: status.as_deref().map(parse_status).transpose()?,
            };
            if *ids_only {
                // Spec 009 §3.1: ids and nothing else; an empty corpus prints
                // nothing (no "(no specs)" placeholder) and still exits 0.
                let ids = list_ids(&registry, &filter);
                if *json {
                    print_json(&ids)?;
                } else {
                    for id in ids {
                        outln!("{id}");
                    }
                }
            } else {
                let specs = list(&registry, &filter);
                if *json {
                    print_json(&specs)?;
                } else if specs.is_empty() {
                    outln!("(no specs)");
                } else {
                    for s in specs {
                        outln!("{}  {:<11}  {}", s.id, status_label(s.status), s.title);
                    }
                }
            }
        }
        RegistryQuery::Show { id, json } => {
            let spec = show(&registry, id)?;
            // Spec 048 §3.3: the hash the committed shard records, read and
            // never recomputed. Added at the output boundary and nowhere else:
            // putting it on `SpecRecord` would write it into every shard, whose
            // schema is `additionalProperties: false`, for a value the shard
            // already carries one line above the record.
            let content_hash = shard_content_hash(&cfg, repo, &spec.id)?;
            if *json {
                let mut value =
                    serde_json::to_value(spec).map_err(|e| Error::Schema(e.to_string()))?;
                if let (Some(obj), Some(h)) = (value.as_object_mut(), content_hash.as_ref()) {
                    obj.insert("contentHash".to_string(), serde_json::json!(h));
                }
                print_json(&value)?;
            } else {
                outln!("id:      {}", spec.id);
                outln!("title:   {}", spec.title);
                outln!("status:  {}", status_label(spec.status));
                outln!("created: {}", spec.created);
                outln!("path:    {}", spec.spec_path);
                outln!("summary: {}", spec.summary.trim());
                // Spec 114 §3.6: the declared intent is printed beside the
                // summary, so a reader sees both statements of purpose
                // together. Neither is checked against the other; where they
                // disagree the prose governs and the intent is corrected.
                if let Some(intent) = &spec.intent {
                    outln!("intent:  {}", intent.goal.trim());
                    for ng in &intent.non_goals {
                        outln!("  not:   {}", ng.trim());
                    }
                }
                if let Some(h) = &content_hash {
                    // 048 §3.4: say which hash this is in the same breath as
                    // reporting it. The registry's and the index's per-spec
                    // hashes are the same shape, and a consumer that confuses
                    // them gets a pin that fires on unrelated edits. Spec 077:
                    // and name the construction, since it is framed by the path
                    // and so is not the digest of the file's bytes, which a
                    // consumer reproducing it would otherwise compute.
                    outln!(
                        "contentHash: {h}  (sha256 over path + NUL + normalized spec.md bytes; not the bare file digest)"
                    );
                }
            }
        }
        RegistryQuery::StatusReport { json, nonzero_only } => {
            let report = status_report(&registry);
            if *nonzero_only {
                let projected = report.nonzero_only();
                if *json {
                    print_json(&projected)?;
                } else {
                    outln!("total:      {}", projected.total);
                    print_count("draft:     ", projected.draft);
                    print_count("approved:  ", projected.approved);
                    print_count("superseded:", projected.superseded);
                    print_count("retired:   ", projected.retired);
                }
            } else if *json {
                print_json(&report)?;
            } else {
                outln!("total:      {}", report.total);
                outln!("draft:      {}", report.draft);
                outln!("approved:   {}", report.approved);
                outln!("superseded: {}", report.superseded);
                outln!("retired:    {}", report.retired);
            }
        }
        RegistryQuery::Plan { json, next } => {
            let plan = plan(&registry)?;
            if *next {
                // Spec 053 §3.2: a projection of `plan`, never a second
                // selection. An empty ready set exits 0: "nothing to do" is a
                // true answer to "what should I work on", and a driven session
                // that treats it as an error stops for the wrong reason.
                //
                // Spec 074 §3.3 (amending 060 §3.2): with `--json` the pick sits
                // under a named `next` member, built here for the populated
                // answer and the empty one alike, so both are one shape and
                // "nothing is ready" is a present `null` rather than a missing
                // key. Still the object itself, not a one-element array.
                match plan.next() {
                    pick if *json => {
                        print_json(&serde_json::json!({ "next": pick }))?;
                    }
                    Some(pick) => outln!("{}  {}", pick.id, pick.title),
                    None => outln!("(nothing ready)"),
                }
            } else if *json {
                print_json(&plan)?;
            } else {
                print_plan(&plan);
            }
        }
        RegistryQuery::Obligation { reference, json } => {
            let view = spec_spine_core::obligation(&registry, reference)?;
            // Spec 106 §3.6: the spec's full identity beside the section's,
            // read from the committed shard and never recomputed (048 §3.3).
            let content_hash = shard_content_hash(&cfg, repo, view.spec)?;
            if *json {
                let mut value =
                    serde_json::to_value(&view).map_err(|e| Error::Schema(e.to_string()))?;
                if let (Some(obj), Some(h)) = (value.as_object_mut(), content_hash.as_ref()) {
                    obj.insert("contentHash".to_string(), serde_json::json!(h));
                }
                print_json(&value)?;
            } else {
                let ob = view.obligation;
                let kind = serde_json::to_value(ob.kind)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_default();
                outln!(
                    "{}#{}{}",
                    view.spec,
                    ob.id,
                    if ob.withdrawn { "  (withdrawn)" } else { "" }
                );
                outln!("kind:    {kind}");
                outln!("text:    {}", ob.text);
                outln!("anchor:  {}#{}", view.spec_path, ob.anchor);
                if let Some(d) = view.section_digest {
                    outln!("sectionDigest: {d}");
                }
                if let Some(h) = &content_hash {
                    outln!("contentHash:   {h}");
                }
                for input in &ob.inputs {
                    outln!("input:   {input}");
                }
            }
        }
        RegistryQuery::Impacts {
            target,
            declared_by,
            json,
        } => {
            let set =
                spec_spine_core::impacts(&registry, target.as_deref(), declared_by.as_deref())?;
            if *json {
                print_json(&set)?;
            } else if set.impacts.is_empty() && set.conflicts.is_empty() {
                outln!("(no declarations)");
            } else {
                for imp in &set.impacts {
                    outln!(
                        "impact    {} -> {}  {}{}{}",
                        imp.declared_by,
                        imp.target,
                        label(imp.nature),
                        imp.successor
                            .as_deref()
                            .map(|s| format!("  successor={s}"))
                            .unwrap_or_default(),
                        if imp.target_withdrawn {
                            "  (target withdrawn)"
                        } else {
                            ""
                        }
                    );
                }
                for c in &set.conflicts {
                    outln!(
                        "conflict  {} -> {}  {}{}{}",
                        c.declared_by,
                        c.target,
                        label(c.resolution),
                        c.settled_by
                            .as_deref()
                            .map(|s| format!("  settled_by={s}"))
                            .unwrap_or_default(),
                        if c.target_withdrawn {
                            "  (target withdrawn)"
                        } else {
                            ""
                        }
                    );
                }
            }
        }
        RegistryQuery::Moves { path, json } => {
            return Ok(match path {
                Some(p) => {
                    let outcome = lookup_move(&registry, p);
                    // Spec 111 §3.4: `unmapped`/`resolved` exit 0; the lookup
                    // refusing to answer (`ambiguous`/`cycle`) exits 1, the
                    // way `couple` exits 1 on an open violation without that
                    // being an `Err` (spec 034): a full report is still
                    // printed, only the process's exit signals the refusal.
                    let code = match &outcome {
                        MoveLookup::Unmapped { .. } | MoveLookup::Resolved { .. } => 0,
                        MoveLookup::Ambiguous { .. } | MoveLookup::Cycle { .. } => 1,
                    };
                    if *json {
                        print_json(&outcome)?;
                    } else {
                        print_move_lookup(&outcome);
                    }
                    code
                }
                None => {
                    let entries = flattened_moves(&registry);
                    if *json {
                        print_json(&entries)?;
                    } else if entries.is_empty() {
                        outln!("(no declared moves)");
                    } else {
                        for e in &entries {
                            outln!(
                                "{}  {} -> {}  ({}){}",
                                e.declared_by,
                                e.from,
                                e.to.as_deref().unwrap_or("(removed)"),
                                e.kind.label(),
                                e.answered_by
                                    .as_deref()
                                    .map(|s| format!("  answered_by={s}"))
                                    .unwrap_or_default()
                            );
                        }
                    }
                    0
                }
            });
        }
        RegistryQuery::Closure { request, json } => {
            let text = if request == "-" {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
                    .map_err(|e| Error::Io(format!("read closure request from stdin: {e}")))?;
                buf
            } else {
                std::fs::read_to_string(request)
                    .map_err(|e| Error::Io(format!("read closure request {request}: {e}")))?
            };
            let req: spec_spine_core::ClosureRequest = serde_json::from_str(&text)
                .map_err(|e| Error::Parse(format!("invalid closure request: {e}")))?;
            let resolved = spec_spine_core::closure(&cfg, repo, &req)?;
            if *json {
                print_json(&resolved)?;
            } else {
                for m in &resolved.members {
                    match m {
                        spec_spine_core::ClosureMember::Spec { spec, content_hash } => {
                            outln!("spec        {spec}  {content_hash}")
                        }
                        spec_spine_core::ClosureMember::Section {
                            spec,
                            anchor,
                            digest,
                        } => {
                            outln!("section     {spec}#{anchor}  {digest}")
                        }
                        spec_spine_core::ClosureMember::Obligation {
                            spec,
                            id,
                            section_digest,
                            withdrawn,
                            ..
                        } => outln!(
                            "obligation  {spec}#{id}  {section_digest}{}",
                            if *withdrawn { "  (withdrawn)" } else { "" }
                        ),
                    }
                }
                outln!("digest: {}", resolved.digest);
            }
        }
        RegistryQuery::Relationships { id, json } => {
            let view = relationships(&registry, id)?;
            if *json {
                print_json(&view)?;
            } else {
                outln!("{}", view.id);
                print_ids("depends_on", &view.depends_on);
                print_ids("supersedes", &view.supersedes);
                print_ids("amends", &view.amends);
                print_ids("superseded_by (incoming)", &view.superseded_by);
                print_ids("amended_by (incoming)", &view.amended_by);
                print_ids("depended_on_by (incoming)", &view.depended_on_by);
            }
        }
    }
    Ok(0)
}

/// The human form: the ready set in order, then one line of what is blocked.
///
/// The prose form counts rather than enumerating the blocked set, because the
/// question a person asks at a terminal is "what can I do now"; `--json` carries
/// every blocker and its state for the consumer that asks "why not that one".
fn print_plan(plan: &Plan) {
    // Spec 053 §3.1: render what the structure already holds. Titles come from
    // the registry the plan was computed from, and each blocked spec's reasons
    // are printed rather than counted: `blocked_by` carries the state of every
    // blocker, and printing the count while discarding the states throws away
    // the part a reader needs.
    if plan.ready.is_empty() {
        // The line a finished corpus prints, unchanged: it already carries the
        // count it would otherwise repeat as "ready: 0".
        outln!("(nothing ready), blocked: {}", plan.blocked.len());
    } else {
        outln!("ready ({}):", plan.ready.len());
        let w = id_width(plan.ready.iter().map(|r| r.id.as_str()));
        for r in &plan.ready {
            outln!("  {:<w$}  {}", r.id, r.title, w = w);
        }
    }
    if !plan.blocked.is_empty() {
        outln!();
        outln!("blocked ({}):", plan.blocked.len());
        let w = id_width(plan.blocked.iter().map(|b| b.id.as_str()));
        for b in &plan.blocked {
            outln!("  {:<w$}  {}", b.id, b.title, w = w);
            let reasons: Vec<String> = b
                .blocked_by
                .iter()
                .map(|k| format!("{} ({})", k.id, k.state))
                .collect();
            outln!("       blocked by {}", reasons.join(", "));
        }
    }
    // Spec 063 §3.6: what the corpus has said it will own and has not written
    // yet. Reported below both sets rather than folded into either, because a
    // blocked spec's planned territory is exactly what a reader wants when
    // weighing what unblocking it would cost.
    if !plan.planned.is_empty() {
        outln!();
        let units: usize = plan.planned.iter().map(|p| p.units.len()).sum();
        outln!(
            "planned territory ({units} unit(s) across {} spec(s)):",
            plan.planned.len()
        );
        let w = id_width(plan.planned.iter().map(|p| p.id.as_str()));
        for entry in &plan.planned {
            outln!("  {:<w$}  {}", entry.id, entry.title, w = w);
            for unit in &entry.units {
                outln!("       {unit}");
            }
        }
    }
    // Spec 072 §3.3: printed after both sets and after planned territory,
    // because it is a fact *about* the ready set rather than a third set. The
    // caveat rides on the heading line and not in a footnote: a reader who
    // skims the count and stops must not come away with a clearance.
    if !plan.overlaps.is_empty() {
        outln!();
        outln!(
            "overlapping territory ({} pair(s), a lower bound on what the corpus declares; an absent pair is not a safety verdict):",
            plan.overlaps.len()
        );
        for o in &plan.overlaps {
            outln!("  {} + {}", o.specs[0], o.specs[1]);
            for unit in &o.units {
                outln!("       {unit}");
            }
        }
    }
    outln!();
    outln!(
        "{} specs: {} ready, {} blocked, {} not schedulable",
        plan.ready.len() + plan.blocked.len() + plan.not_schedulable,
        plan.ready.len(),
        plan.blocked.len(),
        plan.not_schedulable
    );
}

/// Column width for an id list, so titles line up. Per group, because the two
/// groups are read separately and a shared width would pad one of them for the
/// other's benefit.
fn id_width<'a>(ids: impl Iterator<Item = &'a str>) -> usize {
    ids.map(|id| id.chars().count()).max().unwrap_or(0)
}

fn parse_status(s: &str) -> Result<Status, Error> {
    match s {
        "draft" => Ok(Status::Draft),
        "approved" => Ok(Status::Approved),
        "superseded" => Ok(Status::Superseded),
        "retired" => Ok(Status::Retired),
        other => Err(Error::NotFound(format!(
            "unknown status '{other}' (expected draft|approved|superseded|retired)"
        ))),
    }
}

/// The lowercase wire spelling of a `lowercase`-serde enum (spec 109's
/// `ImpactNature` / `ConflictResolution`), for the text rendering: `{:?}`
/// would print `Refines`, and the wire form a reader of the frontmatter wrote
/// is `refines`.
fn label(v: impl serde::Serialize) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn status_label(s: Status) -> &'static str {
    match s {
        Status::Draft => "draft",
        Status::Approved => "approved",
        Status::Superseded => "superseded",
        Status::Retired => "retired",
    }
}

fn print_count(label: &str, count: Option<usize>) {
    if let Some(n) = count {
        outln!("{label} {n}");
    }
}

fn print_ids(label: &str, ids: &[String]) {
    if !ids.is_empty() {
        outln!("  {label}: {}", ids.join(", "));
    }
}

/// The prose form of a `registry moves <path>` lookup (spec 111 §3.4).
fn print_move_lookup(outcome: &MoveLookup) {
    match outcome {
        MoveLookup::Unmapped { path } => outln!("{path}  unmapped"),
        MoveLookup::Resolved {
            path,
            hops,
            terminals,
        } => {
            outln!("{path}  resolved");
            for h in hops {
                outln!(
                    "  {} -> {}  ({}, declared by {}){}",
                    h.from,
                    h.to.as_deref().unwrap_or("(removed)"),
                    h.kind.label(),
                    h.declared_by.join(", "),
                    h.answered_by
                        .as_deref()
                        .map(|s| format!("  answered_by={s}"))
                        .unwrap_or_default()
                );
            }
            for t in terminals {
                match (&t.path, &t.answered_by) {
                    (Some(p), _) => outln!("  terminal: {p}"),
                    (None, Some(by)) => outln!("  terminal: removed, answered_by={by}"),
                    (None, None) => outln!("  terminal: removed"),
                }
            }
        }
        MoveLookup::Ambiguous {
            path,
            at,
            candidates,
        } => {
            outln!("{path}  ambiguous at '{at}'");
            for c in candidates {
                outln!(
                    "  {} declares {}  ({}){}",
                    c.declared_by,
                    c.to.as_deref().unwrap_or("(removed)"),
                    c.kind.label(),
                    c.answered_by
                        .as_deref()
                        .map(|s| format!("  answered_by={s}"))
                        .unwrap_or_default()
                );
            }
        }
        MoveLookup::Cycle { path, chain } => {
            outln!("{path}  cycle: {}", chain.join(" -> "));
        }
    }
}

/// Emit a read document (spec 074): sorted keys, object form, `schemaVersion`.
/// Every `--json` arm of this verb comes through here, which is what makes a
/// projection flag a versioned document rather than a call site that forgot.
fn print_json<T: serde::Serialize>(value: &T) -> Result<(), Error> {
    out!("{}", read_document(value, Versioning::Stamp)?);
    Ok(())
}
