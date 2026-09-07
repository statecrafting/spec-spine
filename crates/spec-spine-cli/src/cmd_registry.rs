//! `spec-spine registry …`: typed, read-only queries over the compiled
//! registry. Assembles the registry from its committed per-spec shards via the
//! library (never ad-hoc parsing, per spec 000 §1; the shard tree replaces the
//! monolithic `registry.json` since spec 024).

use std::path::Path;

use clap::Subcommand;
use spec_spine_core::{
    ListFilter, Plan, list, list_ids, load_committed_registry, plan, relationships,
    shard_content_hash, show, status_report,
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
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Which specs can be worked on now, and what blocks the rest (spec 038).
    Plan {
        #[arg(long)]
        json: bool,
        /// Print only the single pick: the first ready spec (spec 060). An
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
                // Spec 010 §3.1: ids and nothing else; an empty corpus prints
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
            // Spec 055 §3.3: the hash the committed shard records, read and
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
                if let Some(h) = &content_hash {
                    // §3.4: say which hash this is in the same breath as
                    // reporting it. The registry's and the index's per-spec
                    // hashes are the same shape, and a consumer that confuses
                    // them gets a pin that fires on unrelated edits.
                    outln!("contentHash: {h}  (sha256 of this spec.md)");
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
                // Spec 060 §3.2: a projection of `plan`, never a second
                // selection. An empty ready set exits 0: "nothing to do" is a
                // true answer to "what should I work on", and a driven session
                // that treats it as an error stops for the wrong reason.
                match plan.next() {
                    Some(pick) if *json => print_json(pick)?,
                    // The object itself, not a one-element array: a consumer
                    // should not index into a list to reach the thing it asked
                    // for.
                    Some(pick) => outln!("{}  {}", pick.id, pick.title),
                    None if *json => print_json(&serde_json::Value::Null)?,
                    None => outln!("(nothing ready)"),
                }
            } else if *json {
                print_json(&plan)?;
            } else {
                print_plan(&plan);
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
    // Spec 060 §3.1: render what the structure already holds. Titles come from
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

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), Error> {
    let s = serde_json::to_string_pretty(value).map_err(|e| Error::Schema(e.to_string()))?;
    outln!("{s}");
    Ok(())
}
