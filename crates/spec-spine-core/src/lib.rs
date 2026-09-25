//! # spec-spine-core
//!
//! The spec-spine engine. Phase 2 shipped **compile** + **query**; Phase 3 added
//! **index** (code-as-source view, staleness, authorities) and **lint**; Phase 4
//! adds **couple** (the PR-time drift gate) and the governance **scaffold**
//! (starter corpus content as data; spec 092 §3.2).
//!
//! Every artifact-producing function is a pure function of `(config, file
//! contents)`: no ambient clock or environment reads, and **no git** (the CLI
//! parses the diff and passes a typed [`DiffInput`] in). The public API returns
//! owned, `serde`-serializable DTOs (from [`spec_spine_types`]); the
//! JSON-in/JSON-out facade ([`compile_json`], [`query_json`], [`index_json`],
//! [`lint_json`], [`couple_json`], [`coverage_json`], [`scaffold_init_json`], …) is the seam future
//! FFI bindings wrap.

pub mod attest;
mod canonical_json;
pub mod closure;
pub mod compact;
pub mod compile;
pub mod couple;
pub mod coverage;
pub mod delta;
pub mod dep_only;
pub mod diagnostics;
mod hash;
pub mod impact;
pub mod index;
pub mod interface;
pub mod lint;
pub mod manifest;
mod markdown;
pub mod moves;
pub mod pathutil;
pub mod query;
pub mod read;
pub mod render;
pub mod scaffold;
pub mod scope;
pub mod sections;
pub mod shard;
pub mod snapshot;
pub mod spec_id;
pub mod symbols;
pub mod verify;
pub mod waiver;

use serde::{Deserialize, Serialize};
use spec_spine_types::{Config, CorpusAttestation, Error, Status, load_config, validate_config};

// Re-export the type substrate so callers depend on one crate.
pub use spec_spine_types as types;
pub use spec_spine_types::{
    CodebaseIndex, CoverageReport, Frontmatter, PackageCoverage, REGISTRY_SCHEMA_VERSION, Registry,
    SpecRecord, Unit, Violation,
};

pub use attest::{
    AttestOptions, AttestOutcome, NON_CANONICAL_BYTES, SpecAttestOutcome, VerifyOutcome, attest,
    attest_spec, attestation_hash, check_attestation_major, check_spec_attestation_major,
    payload_schema_version, spec_attestation_hash, stored_bytes_hash, verify_recompute,
    verify_spec_recompute, with_stored_bytes, with_stored_bytes_spec,
};
pub use closure::{
    ClosureMember, ClosureRequest, ResolvedClosure, SectionRef, closure, committed_content_hashes,
    resolve_closure,
};
pub use compact::{
    CompactPlan, Compaction, Leftover, RetireEntry, RetireKind, SkipClause, Skipped, UnitAction,
    UnitActionKind, compact, parse_plan,
};
pub use compile::{
    CompileOutcome, MAX_UNDECLARED_EXTRA_FRONTMATTER, RegistryShardSet, SpecCheckReport,
    check_registry_freshness, compare_committed_registry, compile, compile_spec,
    load_committed_registry, registry_dir, registry_shard_files,
};
pub use couple::{
    CoupleReport, DEFAULT_BYPASS_PREFIXES, DeletionProvenance, DiffFile, DiffInput, PriorOwnership,
    PriorSnapshots, SNAPSHOT_HEAD_COMMIT, SNAPSHOT_HEAD_TREE, SNAPSHOT_MERGE_BASE, Waiver,
    build_superseders, couple, couple_snapshots, couple_snapshots_waived, couple_with,
    couple_with_prior, couple_with_prior_waived, couple_with_scope, effective_bypass_prefixes,
    is_bypassed_path, owners_for_path, parse_waiver, prior_ownership_from_root,
};
pub use coverage::{
    EmptyUniverse, GovernedScope, Ownership, SOURCE_EXTS, classify, coverage, coverage_with,
    coverage_with_inventory, coverage_with_scope, empty_universe, enumerate_source_files,
    in_coverage_universe, in_coverage_universe_with, walk_repository,
};
pub use delta::{delta, tree_config};
pub use dep_only::{
    CARGO_DEPENDENCY_TABLES, DEPENDENCY_TABLES, FileContents, cargo_dependency_only_change,
    dependency_only_change, dependency_only_waiver, is_cargo_toml, is_dependency_manifest,
    is_package_json, is_workflow_yaml, workflow_dependency_only_change,
};
pub use diagnostics::{
    AttributedDiagnostic, CheckReport, DiagnosticCounts, IndexCheckReport, RegistryCheckReport,
    UNRESOLVED_CODES, UnwitnessedCounts, annotate_unreadable, committed_counts,
    committed_diagnostics, count as count_diagnostics,
};
pub use impact::{ConflictEntry, ImpactEntry, ImpactSet, impacts};
pub use index::{
    BlockingClaim, Freshness, INPUTS_FILE, IndexFreshnessReport, IndexOutcome, IndexShardSet,
    OwnerKind, OwnerLink, OwnerReport, UnwitnessedClaim, authorities, check_index_freshness,
    check_slice_freshness, index, index_dir, index_freshness_report, index_inputs_file,
    index_shard_files, load_committed_index, owner, owner_with, slices_path, unwitnessed_claims,
    witnessed_paths,
};
pub use interface::{
    ExportedSpec, Exports, interface_verify, load_export, verify_interface_references,
};
pub use lint::{LintReport, lint};
pub use moves::{
    AmbiguousCandidate, Hop, MoveEntry, MoveLookup, Terminal as MoveTerminal, flattened_moves,
    lookup as lookup_move,
};
pub use query::{
    BlockedSpec, Blocker, ListFilter, ObligationView, Plan, ReadySpec, RelationshipView,
    StatusReport, StatusReportNonzero, list, list_ids, load_index, load_registry, obligation, plan,
    relationships, shard_content_hash, show, status_report,
};
pub use read::{Versioning, read_document};
pub use render::{OrphanReport, orphans, partition_orphans, render_markdown};
pub use scaffold::{Scaffold, ScaffoldFile, ScaffoldOptions, scaffold_init, scaffold_init_opts};
pub use scope::{
    Conflict, EvaluatedEntry, Finding, Role, ScopeComparison, ScopeConflictEntry, ScopeEvaluation,
    ScopeIdentity, ScopeRequest, SharedPath, compare_scopes, evaluate, evaluate_scope,
};
pub use snapshot::{
    SnapshotOutcome, check_snapshot_major, snapshot, snapshot_hash, verify_snapshot_recompute,
    with_stored_bytes_snapshot,
};
pub use waiver::{
    CheckOutcome, ClearedViolation, WaiverCheck, WaiverDeclaration, WaiverInputs, WaiverOutcome,
    WaiverSet, parse_waivers,
};
// Spec 067 3.4: the one spec-id policy, public because the CLI's two
// non-library arguments (the attestation file name, and the attestation
// directory `verify-attestation` resolves against) call it directly.
pub use spec_id::{SpecIdMatch, match_spec_id, resolve_spec_id, resolve_spec_ref, spec_dir_ids};
pub use verify::{plan as verify_plan, plan_from_markdown, without_verification_section};

// ===== JSON-in / JSON-out facade (the FFI seam) =====

/// Compile the corpus under `repo_root`, returning the registry as JSON.
///
/// `config_json` is a JSON object matching [`Config`] (`"{}"` ⇒ defaults). The
/// returned string is the canonical `registry.json`; the caller inspects its
/// embedded `validation.passed`.
pub fn compile_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let outcome = compile(&config, std::path::Path::new(repo_root))?;
    Ok(outcome.json)
}

/// Resolve a context closure against the committed ledger (spec 107).
///
/// `request_json` is a [`ClosureRequest`]: `{ "specs"?: [id], "sections"?:
/// [{ "spec", "anchor" }], "obligations"?: ["<spec-id>#<obligation-id>"],
/// "rationale"?: string }`. The answer is a read document (spec 074) with
/// `members`, `digest` and `rationale`. A stale registry is refused (exit 1)
/// before anything is digested.
pub fn closure_json(
    config_json: &str,
    repo_root: &str,
    request_json: &str,
) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let request: ClosureRequest = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid closure request: {e}")))?;
    let resolved = closure(&config, std::path::Path::new(repo_root), &request)?;
    read_document(&resolved, Versioning::Stamp)
}

/// Evaluate a work scope against the committed ownership index (spec 108).
///
/// `scope_json` is a [`ScopeRequest`]: `{ "id"?: string, "ownSpec": id,
/// "mutable"?: [path], "shared"?: [{ "path", "with": [id] }], "readOnly"?:
/// [path] }`. The answer is a read document (spec 074) naming each path's
/// resolved owners and every `S-00x` finding where the declaration and the
/// index disagree. A stale index is refused (exit 1) before any path is
/// resolved. It is a report, not a gate: exit 0 whether or not it found
/// anything.
pub fn scope_json(config_json: &str, repo_root: &str, scope_json: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let request: ScopeRequest = serde_json::from_str(scope_json)
        .map_err(|e| Error::Usage(format!("invalid scope request: {e}")))?;
    let evaluated = evaluate(&config, std::path::Path::new(repo_root), &request)?;
    read_document(&evaluated, Versioning::Stamp)
}

/// Compare two declared work scopes for conflicting intentions (spec 108
/// §3.5). Pure: reads no ledger, and validates each document as
/// [`scope_json`] does.
pub fn scope_compare_json(a_json: &str, b_json: &str) -> Result<String, Error> {
    let a: ScopeRequest = serde_json::from_str(a_json)
        .map_err(|e| Error::Usage(format!("invalid scope request 'a': {e}")))?;
    let b: ScopeRequest = serde_json::from_str(b_json)
        .map_err(|e| Error::Usage(format!("invalid scope request 'b': {e}")))?;
    let comparison = compare_scopes(&a, &b)?;
    read_document(&comparison, Versioning::Stamp)
}

/// Run a read-only query described by `request_json`.
///
/// Request shape: `{ "registry": "<registry.json text>", "op": "list" |
/// "show" | "status-report" | "relationships" | "plan" | "obligation", "id"?:
/// string (for `obligation`, a qualified `<spec-id>#<obligation-id>`, spec
/// 106), "status"?: string,
/// "idsOnly"?: bool, "nonzeroOnly"?: bool }`. The projection fields (spec 009)
/// default to `false`, so pre-010 requests behave identically.
///
/// Every answer is a read document (spec 074): sorted keys, `schemaVersion`,
/// and `list`'s array under `items`.
pub fn query_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        registry: String,
        op: Op,
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        status: Option<Status>,
        #[serde(default)]
        ids_only: bool,
        #[serde(default)]
        nonzero_only: bool,
        /// Spec 109 §3.6: `impacts`' filters. `target` is a spec id or a
        /// qualified obligation reference; `declaredBy` is a spec id.
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        declared_by: Option<String>,
        /// Spec 111 §3.4: `moves`' optional lookup path. Absent, the answer
        /// is every declaration, flattened and sorted.
        #[serde(default)]
        path: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "kebab-case")]
    enum Op {
        List,
        Show,
        StatusReport,
        Relationships,
        Plan,
        /// Spec 106 §3.6: `id` is a qualified `<spec-id>#<obligation-id>`.
        Obligation,
        /// Spec 109 §3.6: every declared impact and conflict, filtered by
        /// `target` and `declaredBy`.
        Impacts,
        /// Spec 111 §3.4: the move lookup, or (with no `path`) every
        /// declaration flattened and sorted.
        Moves,
    }

    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid query request: {e}")))?;
    let registry = load_registry(request.registry.as_bytes())?;

    // Spec 074 §3.3: every answer here is a read document, so it goes through
    // the one emitter the CLI's read verbs use. The CLI and the facade cannot
    // then emit different shapes: `list` wraps under `items`, and every
    // document carries `schemaVersion` with its keys sorted.
    let json = match request.op {
        Op::List => {
            let filter = ListFilter {
                status: request.status,
            };
            if request.ids_only {
                read_document(&query::list_ids(&registry, &filter), Versioning::Stamp)?
            } else {
                read_document(&list(&registry, &filter), Versioning::Stamp)?
            }
        }
        Op::Show => {
            let id = request
                .id
                .ok_or_else(|| Error::NotFound("missing 'id' for show".into()))?;
            read_document(show(&registry, &id)?, Versioning::Stamp)?
        }
        Op::StatusReport => {
            let report = status_report(&registry);
            if request.nonzero_only {
                read_document(&report.nonzero_only(), Versioning::Stamp)?
            } else {
                read_document(&report, Versioning::Stamp)?
            }
        }
        Op::Relationships => {
            let id = request
                .id
                .ok_or_else(|| Error::NotFound("missing 'id' for relationships".into()))?;
            read_document(&relationships(&registry, &id)?, Versioning::Stamp)?
        }
        Op::Obligation => {
            let reference = request
                .id
                .ok_or_else(|| Error::NotFound("missing 'id' for obligation".into()))?;
            read_document(
                &query::obligation(&registry, &reference)?,
                Versioning::Stamp,
            )?
        }
        // Spec 035. Not the spec 034 verdict envelope, which wraps the
        // adjudicating verbs; a read document instead (spec 074).
        Op::Plan => read_document(&plan(&registry)?, Versioning::Stamp)?,
        Op::Impacts => read_document(
            &impacts(
                &registry,
                request.target.as_deref(),
                request.declared_by.as_deref(),
            )?,
            Versioning::Stamp,
        )?,
        Op::Moves => match request.path.as_deref() {
            Some(p) => read_document(&lookup_move(&registry, p), Versioning::Stamp)?,
            None => read_document(&flattened_moves(&registry), Versioning::Stamp)?,
        },
    };
    Ok(json)
}

/// Recompute every declared cross-corpus interface reference against
/// caller-supplied export text (spec 110 §3.3, §3.4). No filesystem: a
/// binding builds `exports` directly and gets the same answer `interface
/// verify` gives from a local checkout.
///
/// `request_json`: `{ "registry": "<registry.json text>", "exports": {
/// "<corpus>": { "<spec-id>": { "path", "text" } } }, "spec"?: "<id>" }`,
/// unknown members refused. The answer is a read document (spec 074, read
/// schema `0.6.0`), the same shape `interface verify --json` prints; an
/// unresolved `spec` is [`Error::NotFound`] (exit 1).
pub fn interface_verify_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        registry: String,
        #[serde(default)]
        exports: interface::Exports,
        #[serde(default)]
        spec: Option<String>,
    }

    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid interface verify request: {e}")))?;
    let registry = load_registry(request.registry.as_bytes())?;
    let report = verify_interface_references(&registry, &request.exports, request.spec.as_deref())?;
    read_document(&report, Versioning::Stamp)
}

/// Index the corpus under `repo_root`, returning `index.json`.
pub fn index_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    Ok(index(&config, std::path::Path::new(repo_root))?.json)
}

/// Lint the corpus, returning the `L-` violations as a JSON array.
pub fn lint_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let report = lint(&config, std::path::Path::new(repo_root))?;
    to_json(&report.violations)
}

/// Both freshness reads, composed (spec 062 §3.2): the registry's and the
/// index's, each keeping the shape its own primitive emits.
///
/// The session protocol asks one question, "is the committed state current",
/// and before this verb it had to know two spellings to ask it: `compile
/// --check` is a flag and `index check` is a subcommand. This is the verb above
/// them. It is **additive**; both primitives keep their flags, their output
/// contracts and their tests, and a caller that regenerated only one tree can
/// still ask about that tree alone.
///
/// It **never writes**. A verb the protocol calls to read the committed state
/// cannot repair that state as a side effect of reading it: that is how the
/// spec 016/021 drift reached the default branch, as an apparent local edit
/// rather than a defect already on the branch.
///
/// The exit code the CLI folds from this is not decided here; the report
/// carries the facts and [`spec_spine_cli`] composes them. A binding wanting
/// the fold gets it from the verdict envelope's `exitCode`.
pub fn check_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let root = std::path::Path::new(repo_root);
    to_json(&check_report(&config, root)?)
}

/// Both halves of the composed freshness read, as data.
///
/// Shared by [`check_json`] and the CLI so the two payloads cannot drift, which
/// is the arrangement spec 034 pins for every other verdict verb.
pub fn check_report(config: &Config, repo_root: &std::path::Path) -> Result<CheckReport, Error> {
    Ok(check_report_full(config, repo_root)?.0)
}

/// [`check_report`], plus the index half's two refusals kept apart (spec 079
/// §3.2).
///
/// One index run answers both: the partition is a by-product of the read the
/// report is already built from, so a reporting layer that needs to tell an
/// unresolved claim from a stale shard does not index a second time and does
/// not parse the text of the first answer.
///
/// [`CheckReport`] itself is deliberately not widened. It is the `--json`
/// payload verbatim, spec 079 FR-009 freezes that envelope at `schemaVersion`
/// `0.4.0` with its current members, and a JSON consumer can already separate
/// the two refusals structurally through `report.index.diagnostics.byCode`. The
/// surface that could not tell them apart was the rendered text, so that is the
/// surface this returns the extra data for.
pub fn check_report_full(
    config: &Config,
    repo_root: &std::path::Path,
) -> Result<(CheckReport, IndexFreshnessReport), Error> {
    // The typed path, not the facade: `compile` once, then compare. Calling
    // `check_registry_freshness` would compile a second time, and this verb
    // exists to make one question cost one ask.
    let outcome = compile(config, repo_root)?;
    // Spec 064 §3.4: the registry half carries its own warning tally, so the
    // composed verb can refuse under `--fail-on-warn` and name the tree that
    // refused without a second compile.
    let warnings = outcome.warning_count();
    let registry = if outcome.validation_passed {
        let freshness = compare_committed_registry(config, repo_root, &outcome.shards)?;
        match freshness {
            Freshness::Fresh => RegistryCheckReport {
                fresh: true,
                expected: None,
                actual: None,
                validation_passed: true,
                warnings,
            },
            Freshness::Stale { expected, actual } => RegistryCheckReport {
                fresh: false,
                expected: Some(expected),
                actual: Some(actual),
                validation_passed: true,
                warnings,
            },
        }
    } else {
        // Staleness is not meaningful against a corpus that does not validate,
        // so it is not computed and `fresh` is reported false rather than
        // guessed. This is the same conclusion the exit-code fold draws when it
        // lets `1` outrank `2`.
        RegistryCheckReport {
            fresh: false,
            expected: None,
            actual: None,
            validation_passed: false,
            warnings,
        }
    };

    let freshness_report = index_freshness_report(config, repo_root)?;
    let index = IndexCheckReport::with_unwitnessed(
        &freshness_report.freshness(),
        verdict_tally(config, repo_root),
        unwitnessed_counts(config, repo_root),
    );
    Ok((CheckReport { registry, index }, freshness_report))
}

/// Check index freshness, returning `{ "fresh": bool, "expected"?, "actual"?,
/// "diagnostics": { "warnings", "errors", "byCode" } }`.
///
/// The `diagnostics` member (spec 044) counts what the **committed** shards
/// record, so it is answered from the same ledger the freshness verdict is
/// about. `check_registry_freshness_json` keeps the bare shape: index
/// diagnostics say nothing about the registry.
pub fn check_freshness_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let root = std::path::Path::new(repo_root);
    let freshness = check_index_freshness(&config, root)?;
    let counts = verdict_tally(&config, root);
    // Spec 050 §3.3: the facade and the CLI emit one shape. `cli.rs` pins them
    // against each other, and it caught this: a payload member added on one
    // side only is exactly the drift that test exists to refuse.
    to_json(&IndexCheckReport::with_unwitnessed(
        &freshness,
        counts,
        unwitnessed_counts(&config, root),
    ))
}

/// The diagnostics tally that accompanies an index freshness verdict (spec 076
/// §3.1, §3.4).
///
/// The one path `check_report`, `check_freshness_json`, `index check` and
/// `check` all take. Call it only **after** `check_index_freshness` has
/// answered: the verdict outranks the tally, so nothing here may replace it.
/// `committed_counts` skips a shard it cannot parse and reports the skip, and
/// its one error (an index never built) is the state the freshness check has
/// already refused, so the default below is unreachable in practice and is
/// never a verdict.
pub fn verdict_tally(config: &Config, repo_root: &std::path::Path) -> DiagnosticCounts {
    diagnostics::committed_counts(config, repo_root).unwrap_or_default()
}

/// The distinct-path tally spec 050 §3.3 reports, shared by the facade and the
/// CLI so the two payloads cannot diverge.
///
/// Distinct **paths**, not `(spec, path)` claims: several specs claiming one
/// unhashed file is one hole in the ledger, and counting it per claimant would
/// report the corpus's edge density rather than its gap. An unreadable index is
/// an empty tally rather than an error, because the freshness verdict this
/// accompanies has already been reached.
pub fn unwitnessed_counts(config: &Config, repo_root: &std::path::Path) -> UnwitnessedCounts {
    let Ok(index) = load_committed_index(config, repo_root) else {
        return UnwitnessedCounts::default();
    };
    let claims = unwitnessed_claims(config, repo_root, &index);
    let mut all: std::collections::BTreeSet<&str> = Default::default();
    let mut allowed: std::collections::BTreeSet<&str> = Default::default();
    for c in &claims {
        all.insert(c.path.as_str());
        if c.allowed {
            allowed.insert(c.path.as_str());
        }
    }
    UnwitnessedCounts {
        total: all.len(),
        allowed: allowed.len(),
    }
}

/// Check registry-shard freshness (spec 028), returning the same
/// `{ "fresh": bool, "expected"?, "actual"? }` shape as
/// [`check_freshness_json`] so a binding handles one verdict type for both
/// committed trees. Staleness only: the validation verdict rides on
/// [`compile_json`].
///
/// Each call compiles the corpus, so a consumer that wants *both* verdicts
/// pays two compile passes across this and [`compile_json`]. That is the cost
/// of keeping the facade one-verdict-per-call; a caller that minds it should
/// use the typed API ([`compile`] once, then [`compare_committed_registry`]),
/// which is what the CLI does.
pub fn check_registry_freshness_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let value = freshness_to_json(check_registry_freshness(
        &config,
        std::path::Path::new(repo_root),
    )?);
    Ok(value.to_string())
}

fn freshness_to_json(freshness: Freshness) -> serde_json::Value {
    match freshness {
        Freshness::Fresh => serde_json::json!({ "fresh": true }),
        Freshness::Stale { expected, actual } => {
            serde_json::json!({ "fresh": false, "expected": expected, "actual": actual })
        }
    }
}

/// Report file-granular ownership coverage against the committed index (spec
/// 029), returning the [`CoverageReport`] as JSON. Freshness-guarded like
/// `couple`: a stale committed index is [`Error::Stale`] (exit 1), never a
/// report over the wrong ledger.
pub fn coverage_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    read_document(
        &coverage(&config, std::path::Path::new(repo_root))?,
        Versioning::Stamp,
    )
}

/// [`coverage_json`] with the inventory a declared governed scope is matched
/// against (spec 078 §3.6), the facade half of `index coverage --paths-from`.
/// Request: `{ "config"?: Config, "repoRoot": string, "inventory"?:
/// { "provenance": "tracked" | "supplied", "paths": [string] } }`. An absent
/// `inventory` walks the repository root; a present one with no paths matches
/// nothing. Returns the same read document the CLI emits.
pub fn coverage_inventory_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        #[serde(default)]
        config: Config,
        repo_root: String,
        #[serde(default)]
        inventory: Option<spec_spine_types::Inventory>,
    }
    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid coverage request: {e}")))?;
    validate_config(&request.config)?;
    read_document(
        &coverage_with_inventory(
            &request.config,
            std::path::Path::new(&request.repo_root),
            request.inventory.as_ref(),
        )?,
        Versioning::Stamp,
    )
}

/// Read a spec's declared acceptance (spec 043), returning the [`VerifyPlan`]
/// as JSON: the `verify:cli` commands its `## Verification` section holds, in
/// document order, and the fence tags it declined.
///
/// The plan is all the engine produces. **Running the commands is the caller's
/// act**, never this library's: spec 043 §3.1 keeps process execution on the
/// CLI side of the same seam that keeps `git` there, so the engine stays a pure
/// function of `(config, file contents)` and stays callable from a binding with
/// no shell. A caller that wants them run decides that for itself.
pub fn verify_plan_json(
    config_json: &str,
    repo_root: &str,
    spec_id: &str,
) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    to_json(&verify::plan(
        &config,
        std::path::Path::new(repo_root),
        spec_id,
    )?)
}

/// Render the committed index as markdown (spec 010). `index_json` is the
/// `index.json` text; the returned string is the markdown projection,
/// JSON-encoded (a JSON string literal).
pub fn render_json(config_json: &str, index_json: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let index = load_index(index_json.as_bytes())?;
    to_json(&render::render_markdown(&config, &index))
}

/// List the committed index's orphaned specs (spec 010), as a read document
/// carrying the id strings under `items` (spec 074). `index_json` is the
/// `index.json` text.
pub fn orphans_json(index_json: &str) -> Result<String, Error> {
    let index = load_index(index_json.as_bytes())?;
    // Spec 074 §3.3: a read document, so the id array is wrapped under `items`.
    read_document(&render::orphans(&index), Versioning::Stamp)
}

/// Parse a `spec-spine.toml` and return the normalized [`Config`] as JSON.
pub fn load_config_json(toml_src: &str) -> Result<String, Error> {
    let config = load_config(toml_src)?;
    to_json(&config)
}

/// Run the coupling gate. `request_json` bundles config + repo_root + diff +
/// optional waiver:
/// `{ "config"?: Config, "repoRoot": string, "diff": DiffInput, "waiver"?: { "reason": string },
///    "waivers"?: [WaiverDeclaration], "prBody"?: string,
///    "waiverInputs"?: { "asOf"?: string, "ancestry"?: { commit: bool },
///                       "uses"?: { waiverId: number } },
///    "priorRoots"?: { "mergeBase"?: string, "headCommit"?: string,
///                     "worktreeDeletions"?: [string] } }`.
/// Returns the [`CoupleReport`] as JSON (even when drift is present; the caller
/// inspects `violations` / `waiver`, and `waivers` for what each cleared).
///
/// At most one of `waiver`, `waivers` and `prBody` may be given (spec 113).
/// `waiverInputs` is the only place a lifecycle check gets its input: the
/// library reads no clock and runs no git, so an input left out leaves its
/// check `not-evaluated` (spec 113 §3.4).
///
/// `priorRoots` names exported trees in exactly the sense [`delta_json`] takes
/// `baseRoot` and `headRoot` (spec 100 §3.8). Each root is compiled and indexed
/// under **its own** `spec-spine.toml`, and a deleted path is judged at the
/// snapshot preceding the segment that recorded it: `headCommit` for a path
/// listed in `worktreeDeletions`, `mergeBase` otherwise.
///
/// Absent, the facade behaves exactly as before: deletions resolve at head,
/// which is the compatibility path and carries none of spec 100's guarantee.
/// The member is additive, so no schema MAJOR moves.
pub fn couple_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct PriorRoots {
        #[serde(default)]
        merge_base: Option<String>,
        #[serde(default)]
        head_commit: Option<String>,
        #[serde(default)]
        worktree_deletions: Vec<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        #[serde(default)]
        config: Config,
        repo_root: String,
        diff: DiffInput,
        #[serde(default)]
        waiver: Option<Waiver>,
        /// Spec 113: every declared waiver, as data.
        #[serde(default)]
        waivers: Option<Vec<WaiverDeclaration>>,
        /// Spec 113: a pull request body, parsed with the configured keyword
        /// exactly as the CLI parses `--pr-body`.
        #[serde(default)]
        pr_body: Option<String>,
        /// Spec 113 §3.3: the caller's lifecycle inputs.
        #[serde(default)]
        waiver_inputs: WaiverInputs,
        #[serde(default)]
        prior_roots: Option<PriorRoots>,
    }

    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid couple request: {e}")))?;
    validate_config(&request.config)?;

    let roots = request.prior_roots.unwrap_or_default();
    let load = |root: &Option<String>| -> Result<Option<PriorOwnership>, Error> {
        match root {
            None => Ok(None),
            Some(r) => {
                let path = std::path::Path::new(r);
                let cfg = tree_config(path)?;
                Ok(Some(couple::prior_ownership_from_root(&cfg, path)?))
            }
        }
    };
    let merge_base = load(&roots.merge_base)?;
    let head_commit = load(&roots.head_commit)?;
    let prior = PriorSnapshots {
        merge_base: merge_base.as_ref(),
        head_commit: head_commit.as_ref(),
        worktree_deletions: roots.worktree_deletions.into_iter().collect(),
    };

    // Spec 113: one source of waivers. Two would leave the precedence to a
    // rule the caller cannot see.
    let sources = [
        request.waiver.is_some(),
        request.waivers.is_some(),
        request.pr_body.is_some(),
    ];
    if sources.iter().filter(|s| **s).count() > 1 {
        return Err(Error::Usage(
            "invalid couple request: give at most one of `waiver`, `waivers` and `prBody`".into(),
        ));
    }
    let waivers = match (request.waiver, request.waivers, request.pr_body) {
        (Some(w), _, _) => WaiverSet {
            declarations: vec![WaiverDeclaration::unscoped(w.reason)],
            unattached: Vec::new(),
        },
        (_, Some(declarations), _) => WaiverSet {
            declarations,
            unattached: Vec::new(),
        },
        (_, _, Some(body)) => parse_waivers(&request.config, &body),
        _ => WaiverSet::default(),
    };

    let report = couple_snapshots_waived(
        &request.config,
        std::path::Path::new(&request.repo_root),
        &request.diff,
        &waivers,
        &request.waiver_inputs,
        &prior,
    )?;
    to_json(&report)
}

/// Classify a change under the base's rules (spec 071). `request_json`:
/// `{ "config"?: Config, "baseRoot": string, "headRoot": string, "changed":
/// [string], "commits": { "base", "mergeBase", "head" } }`.
///
/// `config` is the **merge base's** configuration. Absent, it is read from
/// `<baseRoot>/spec-spine.toml` (the working default when that file is absent),
/// never defaulted silently: a caller that omitted it and got the default rules
/// would get a report classified under rules neither tree declares. Returns the
/// [`DeltaReport`](spec_spine_types::DeltaReport) as JSON; the report records
/// and never refuses, so a caller reads `priorPolicy` rather than an error.
pub fn delta_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        #[serde(default)]
        config: Option<Config>,
        base_root: String,
        head_root: String,
        changed: Vec<String>,
        commits: spec_spine_types::DeltaCommits,
    }

    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid delta request: {e}")))?;
    let base_root = std::path::Path::new(&request.base_root);
    let config = match request.config {
        Some(config) => {
            validate_config(&config)?;
            config
        }
        None => tree_config(base_root)?,
    };
    to_json(&delta(
        &config,
        base_root,
        std::path::Path::new(&request.head_root),
        &request.changed,
        &request.commits,
    )?)
}

/// Generate the governance scaffold for `config_json` (`"{}"` ⇒ defaults),
/// returning the [`Scaffold`] (files-as-data) as JSON. The caller writes the
/// files.
///
/// **This is the producer boundary Statecraft consumes** (spec 092 §3.2), and
/// it is pure: it writes nothing, reads no environment variable, discovers no
/// installation, launches no process, opens no connection, reads no clock,
/// registers nothing and activates nothing. It does not require the Statecraft
/// CLI to be installed and behaves identically with an empty home directory.
///
/// It produces governance starter content only. §3.3 of that spec is the file
/// list; everything an agent harness needs is deliberately not in it.
pub fn scaffold_init_json(config_json: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    to_json(&scaffold_init(&config)?)
}

/// [`scaffold_init_json`] with an options document (spec 131), the
/// [`ScaffoldOptions`] as JSON: `{"pinExactVersion": true}` emits an active
/// `[meta]` table pinning `required_version = "=<this producer's version>"`.
/// `"{}"` is exactly [`scaffold_init_json`]. An unknown option key, or options
/// that are not JSON, is `Error::Config`: a consumer asking for a pin must not
/// silently receive an unpinned file. Equally pure.
pub fn scaffold_init_opts_json(config_json: &str, options_json: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let options: ScaffoldOptions = serde_json::from_str(options_json)
        .map_err(|e| Error::Config(format!("invalid scaffold options JSON: {e}")))?;
    to_json(&scaffold_init_opts(&config, &options)?)
}

/// Compact the corpus under an authored plan (spec 096). `plan_yaml` is the
/// plan document; the result is the `Compaction`, files included, for the
/// caller to write. Pure with respect to the tree: reads, never writes.
pub fn compact_json(config_json: &str, repo_root: &str, plan_yaml: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let plan = parse_plan(plan_yaml)?;
    to_json(&compact(&config, std::path::Path::new(repo_root), &plan)?)
}

/// Build a corpus attestation (spec 021). Returns
/// `{ "attestation": <CorpusAttestation>, "attestationHash": "<hex>" }`. Pure:
/// no key (signing is a CLI post-pass), no clock. `with_coupling` records the
/// in-sync coupling verdict as well (FR-002).
pub fn attest_json(
    config_json: &str,
    repo_root: &str,
    with_coupling: bool,
) -> Result<String, Error> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        attestation: CorpusAttestation,
        attestation_hash: String,
    }
    let config = config_from_json(config_json)?;
    let outcome = attest(
        &config,
        std::path::Path::new(repo_root),
        AttestOptions { with_coupling },
    )?;
    to_json(&Response {
        attestation: outcome.attestation,
        attestation_hash: outcome.attestation_hash,
    })
}

/// Build a per-spec attestation (spec 039). Returns
/// `{ "attestation": <SpecAttestation>, "attestationHash": "<hex>" }`, the same
/// envelope shape [`attest_json`] uses for the corpus scope. Pure: no key, no
/// clock. A failing verdict still yields a payload; it is a record, not a gate.
pub fn attest_spec_json(
    config_json: &str,
    repo_root: &str,
    spec_id: &str,
) -> Result<String, Error> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        attestation: spec_spine_types::SpecAttestation,
        attestation_hash: String,
    }
    let config = config_from_json(config_json)?;
    let outcome = attest_spec(&config, std::path::Path::new(repo_root), spec_id)?;
    to_json(&Response {
        attestation: outcome.attestation,
        attestation_hash: outcome.attestation_hash,
    })
}

/// Build an authority snapshot (spec 070 §3.5), returning
/// `{ "attestation": <AuthoritySnapshot>, "attestationHash": "<hex>" }`, the
/// shape the other two scopes' facades return. Pure: no key, no clock; signing
/// is a CLI post-pass.
pub fn attest_snapshot_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        attestation: spec_spine_types::AuthoritySnapshot,
        attestation_hash: String,
    }
    let config = config_from_json(config_json)?;
    let outcome = snapshot(&config, std::path::Path::new(repo_root))?;
    to_json(&Response {
        attestation: outcome.snapshot,
        attestation_hash: outcome.attestation_hash,
    })
}

/// Verify an authority snapshot by recompute (spec 070 §3.5), under spec 068's
/// rules. Request: `{ "config"?: Config, "repoRoot": string, "attestation":
/// <AuthoritySnapshot> }`, or `"attestationText": string` in place of
/// `attestation` (spec 068 3.4). Same outcome vocabulary as
/// [`verify_attestation_json`].
pub fn verify_snapshot_attestation_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        #[serde(default)]
        config: Config,
        repo_root: String,
        #[serde(default)]
        attestation: Option<spec_spine_types::AuthoritySnapshot>,
        #[serde(default)]
        attestation_text: Option<String>,
    }
    const VERB: &str = "verify-snapshot-attestation";
    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid {VERB} request: {e}")))?;
    validate_config(&request.config)?;
    let (attested, stored) = match (request.attestation, request.attestation_text) {
        (Some(_), Some(_)) => return Err(both_supplied(VERB)),
        (None, None) => return Err(neither_supplied(VERB)),
        (Some(a), None) => {
            check_snapshot_major(&a.schema_version)?;
            (a, None)
        }
        (None, Some(text)) => {
            check_snapshot_major(&attest::payload_schema_version(
                text.as_bytes(),
                "snapshot",
            )?)?;
            let a = serde_json::from_str(&text)
                .map_err(|e| Error::Schema(format!("invalid {VERB} attestationText: {e}")))?;
            (a, Some(text))
        }
    };
    let outcome = verify_snapshot_recompute(
        &request.config,
        std::path::Path::new(&request.repo_root),
        &attested,
    )?;
    let outcome = match &stored {
        Some(text) => with_stored_bytes_snapshot(outcome, &attested, text.as_bytes())?,
        None => outcome,
    };
    let value = match outcome {
        VerifyOutcome::Match => serde_json::json!({ "outcome": "match" }),
        VerifyOutcome::VersionMismatch { expected, actual } => {
            serde_json::json!({ "outcome": "versionMismatch", "expected": expected, "actual": actual })
        }
        VerifyOutcome::ContentMismatch { differences } => {
            serde_json::json!({ "outcome": "contentMismatch", "differences": differences })
        }
    };
    Ok(value.to_string())
}

/// Verify a per-spec attestation by recompute (spec 039 3.5). Request:
/// `{ "config"?: Config, "repoRoot": string, "attestation": <SpecAttestation> }`,
/// or `"attestationText": string` in place of `attestation` (spec 068 3.4).
/// Same outcome vocabulary as [`verify_attestation_json`].
pub fn verify_spec_attestation_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        #[serde(default)]
        config: Config,
        repo_root: String,
        #[serde(default)]
        attestation: Option<spec_spine_types::SpecAttestation>,
        #[serde(default)]
        attestation_text: Option<String>,
    }
    const VERB: &str = "verify-spec-attestation";
    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid {VERB} request: {e}")))?;
    validate_config(&request.config)?;
    let (attestation, stored) = match (request.attestation, request.attestation_text) {
        (Some(_), Some(_)) => return Err(both_supplied(VERB)),
        (None, None) => return Err(neither_supplied(VERB)),
        (Some(a), None) => {
            attest::check_spec_attestation_major(&a.schema_version)?;
            (a, None)
        }
        (None, Some(text)) => {
            attest::check_spec_attestation_major(&attest::payload_schema_version(
                text.as_bytes(),
                "attestation",
            )?)?;
            let a = serde_json::from_str(&text)
                .map_err(|e| Error::Schema(format!("invalid {VERB} attestationText: {e}")))?;
            (a, Some(text))
        }
    };
    let outcome = verify_spec_recompute(
        &request.config,
        std::path::Path::new(&request.repo_root),
        &attestation,
    )?;
    let outcome = match &stored {
        Some(text) => attest::with_stored_bytes_spec(outcome, &attestation, text.as_bytes())?,
        None => outcome,
    };
    let value = match outcome {
        VerifyOutcome::Match => serde_json::json!({ "outcome": "match" }),
        VerifyOutcome::VersionMismatch { expected, actual } => {
            serde_json::json!({ "outcome": "versionMismatch", "expected": expected, "actual": actual })
        }
        VerifyOutcome::ContentMismatch { differences } => {
            serde_json::json!({ "outcome": "contentMismatch", "differences": differences })
        }
    };
    Ok(value.to_string())
}

/// Verify an attestation by recompute (spec 021 FR-004 `--recompute`). Request:
/// `{ "config"?: Config, "repoRoot": string, "attestation": <CorpusAttestation> }`.
/// Returns `{ "outcome": "match" }`, `{ "outcome": "versionMismatch", "expected",
/// "actual" }`, or `{ "outcome": "contentMismatch", "differences": [...] }`. This
/// mode needs no key and no signature: any third party can run it.
///
/// `"attestationText": string`, the attestation's exact bytes, may be sent in
/// place of `attestation` (spec 068 3.4). A facade receives a value inside a
/// larger request, so the stored bytes never reach it and 3.1's rule that a
/// verifier decides on the bytes it was given would have no subject here
/// otherwise. Sent that way, a payload whose values recompute but whose bytes
/// are not the canonical serialization is a `contentMismatch` naming exactly
/// that. Either form applies 3.2 and 3.3 to what it receives.
pub fn verify_attestation_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        #[serde(default)]
        config: Config,
        repo_root: String,
        #[serde(default)]
        attestation: Option<CorpusAttestation>,
        #[serde(default)]
        attestation_text: Option<String>,
    }
    const VERB: &str = "verify-attestation";
    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid {VERB} request: {e}")))?;
    validate_config(&request.config)?;
    let (attestation, stored) = match (request.attestation, request.attestation_text) {
        (Some(_), Some(_)) => return Err(both_supplied(VERB)),
        (None, None) => return Err(neither_supplied(VERB)),
        (Some(a), None) => {
            attest::check_attestation_major(&a.schema_version)?;
            (a, None)
        }
        (None, Some(text)) => {
            attest::check_attestation_major(&attest::payload_schema_version(
                text.as_bytes(),
                "attestation",
            )?)?;
            let a = serde_json::from_str(&text)
                .map_err(|e| Error::Schema(format!("invalid {VERB} attestationText: {e}")))?;
            (a, Some(text))
        }
    };
    let outcome = verify_recompute(
        &request.config,
        std::path::Path::new(&request.repo_root),
        &attestation,
    )?;
    let outcome = match &stored {
        Some(text) => attest::with_stored_bytes(outcome, &attestation, text.as_bytes())?,
        None => outcome,
    };
    let value = match outcome {
        VerifyOutcome::Match => serde_json::json!({ "outcome": "match" }),
        VerifyOutcome::VersionMismatch { expected, actual } => {
            serde_json::json!({ "outcome": "versionMismatch", "expected": expected, "actual": actual })
        }
        VerifyOutcome::ContentMismatch { differences } => {
            serde_json::json!({ "outcome": "contentMismatch", "differences": differences })
        }
    };
    Ok(value.to_string())
}

// --- facade helpers ---

/// Spec 068 3.4: `attestation` and `attestationText` are alternatives, and a
/// request carrying both is refused rather than silently resolved.
///
/// Picking one would make the answer depend on which the facade happened to
/// prefer, and the two can disagree: `attestationText` is the only form the byte
/// rule of 3.1 can be applied to, so a caller that sent both and got the value
/// form checked would be told its bytes were verified when they were not.
fn both_supplied(verb: &str) -> Error {
    Error::Usage(format!(
        "invalid {verb} request: `attestation` and `attestationText` are alternatives; \
         supply exactly one (`attestationText` carries the stored bytes, so only it can be \
         checked against them)"
    ))
}

fn neither_supplied(verb: &str) -> Error {
    Error::Usage(format!(
        "invalid {verb} request: one of `attestation` (the parsed value) or `attestationText` \
         (its exact bytes) is required"
    ))
}

/// Deserialize a facade configuration and hold it to the loader's rules
/// (spec 129 3.1): a configuration arriving as JSON is refused exactly where,
/// and with exactly the message, `load_config` would refuse it as TOML.
fn config_from_json(config_json: &str) -> Result<Config, Error> {
    let config = serde_json::from_str(config_json)
        .map_err(|e| Error::Config(format!("invalid config JSON: {e}")))?;
    validate_config(&config)?;
    Ok(config)
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<String, Error> {
    serde_json::to_string(value).map_err(|e| Error::Internal(e.to_string()))
}
