//! # spec-spine-core
//!
//! The spec-spine engine. Phase 2 shipped **compile** + **query**; Phase 3 added
//! **index** (code-as-source view, staleness, authorities) and **lint**; Phase 4
//! adds **couple** (the PR-time drift gate) and **init** (the adopter scaffolder).
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
pub mod compile;
pub mod couple;
pub mod coverage;
pub mod dep_only;
pub mod diagnostics;
mod hash;
pub mod index;
pub mod kit_embedded;
pub mod lint;
pub mod manifest;
mod markdown;
pub mod pathutil;
pub mod query;
pub mod render;
pub mod scaffold;
pub mod sections;
pub mod shard;
pub mod symbols;
pub mod verify;

use serde::{Deserialize, Serialize};
use spec_spine_types::{Config, CorpusAttestation, Error, Status, load_config};

// Re-export the type substrate so callers depend on one crate.
pub use spec_spine_types as types;
pub use spec_spine_types::{
    CodebaseIndex, CoverageReport, Frontmatter, PackageCoverage, REGISTRY_SCHEMA_VERSION, Registry,
    SpecRecord, Unit, Violation,
};

pub use attest::{
    AttestOptions, AttestOutcome, SpecAttestOutcome, VerifyOutcome, attest, attest_spec,
    attestation_hash, spec_attestation_hash, verify_recompute, verify_spec_recompute,
};
pub use compile::{
    CompileOutcome, MAX_UNDECLARED_EXTRA_FRONTMATTER, RegistryShardSet, SpecCheckReport,
    check_registry_freshness, compare_committed_registry, compile, compile_spec,
    load_committed_registry, registry_dir, registry_shard_files,
};
pub use couple::{
    CoupleReport, DEFAULT_BYPASS_PREFIXES, DiffFile, DiffInput, Waiver, build_superseders, couple,
    couple_with, effective_bypass_prefixes, is_bypassed_path, owners_for_path, parse_waiver,
};
pub use coverage::{
    EmptyUniverse, Ownership, SOURCE_EXTS, classify, coverage, coverage_with, empty_universe,
    enumerate_source_files, in_coverage_universe,
};
pub use dep_only::{
    CARGO_DEPENDENCY_TABLES, DEPENDENCY_TABLES, FileContents, cargo_dependency_only_change,
    dependency_only_change, dependency_only_waiver, is_cargo_toml, is_dependency_manifest,
    is_package_json, is_workflow_yaml, workflow_dependency_only_change,
};
pub use diagnostics::{
    AttributedDiagnostic, CheckReport, DiagnosticCounts, IndexCheckReport, RegistryCheckReport,
    UNRESOLVED_CODES, UnwitnessedCounts, committed_counts, committed_diagnostics,
    count as count_diagnostics,
};
pub use index::{
    Freshness, IndexOutcome, IndexShardSet, OwnerKind, OwnerLink, OwnerReport, UnwitnessedClaim,
    authorities, check_index_freshness, check_slice_freshness, index, index_dir, index_shard_files,
    load_committed_index, owner, owner_with, slices_path, unwitnessed_claims, witnessed_paths,
};
pub use lint::{LintReport, lint};
pub use query::{
    BlockedSpec, Blocker, ListFilter, Plan, ReadySpec, RelationshipView, StatusReport,
    StatusReportNonzero, list, list_ids, load_index, load_registry, plan, relationships,
    shard_content_hash, show, status_report,
};
pub use render::{OrphanReport, orphans, partition_orphans, render_markdown};
pub use scaffold::{Scaffold, ScaffoldFile, scaffold_init, scaffold_init_with};
pub use verify::{plan as verify_plan, plan_from_markdown};

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

/// Run a read-only query described by `request_json`.
///
/// Request shape: `{ "registry": "<registry.json text>", "op": "list" |
/// "show" | "status-report" | "relationships", "id"?: string, "status"?: string,
/// "idsOnly"?: bool, "nonzeroOnly"?: bool }`. The projection fields (spec 010)
/// default to `false`, so pre-010 requests behave identically.
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
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "kebab-case")]
    enum Op {
        List,
        Show,
        StatusReport,
        Relationships,
        Plan,
    }

    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Parse(format!("invalid query request: {e}")))?;
    let registry = load_registry(request.registry.as_bytes())?;

    let json = match request.op {
        Op::List => {
            let filter = ListFilter {
                status: request.status,
            };
            if request.ids_only {
                to_json(&query::list_ids(&registry, &filter))?
            } else {
                to_json(&list(&registry, &filter))?
            }
        }
        Op::Show => {
            let id = request
                .id
                .ok_or_else(|| Error::NotFound("missing 'id' for show".into()))?;
            to_json(show(&registry, &id)?)?
        }
        Op::StatusReport => {
            let report = status_report(&registry);
            if request.nonzero_only {
                to_json(&report.nonzero_only())?
            } else {
                to_json(&report)?
            }
        }
        Op::Relationships => {
            let id = request
                .id
                .ok_or_else(|| Error::NotFound("missing 'id' for relationships".into()))?;
            to_json(&relationships(&registry, &id)?)?
        }
        // Spec 038. Emitted bare, like every other `registry` projection: the
        // spec 037 verdict envelope wraps the adjudicating verbs, and 037 4
        // keeps it off the read verbs so a shipped read surface is not broken
        // for symmetry alone.
        Op::Plan => to_json(&plan(&registry)?)?,
    };
    Ok(json)
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

/// Both freshness reads, composed (spec 075 §3.2): the registry's and the
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
/// spec 017/021 drift reached the default branch, as an apparent local edit
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
/// is the arrangement spec 037 pins for every other verdict verb.
pub fn check_report(config: &Config, repo_root: &std::path::Path) -> Result<CheckReport, Error> {
    // The typed path, not the facade: `compile` once, then compare. Calling
    // `check_registry_freshness` would compile a second time, and this verb
    // exists to make one question cost one ask.
    let outcome = compile(config, repo_root)?;
    let registry = if outcome.validation_passed {
        let freshness = compare_committed_registry(config, repo_root, &outcome.shards)?;
        match freshness {
            Freshness::Fresh => RegistryCheckReport {
                fresh: true,
                expected: None,
                actual: None,
                validation_passed: true,
            },
            Freshness::Stale { expected, actual } => RegistryCheckReport {
                fresh: false,
                expected: Some(expected),
                actual: Some(actual),
                validation_passed: true,
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
        }
    };

    let freshness = check_index_freshness(config, repo_root)?;
    let index = IndexCheckReport::with_unwitnessed(
        &freshness,
        diagnostics::committed_counts(config, repo_root)?,
        unwitnessed_counts(config, repo_root),
    );
    Ok(CheckReport { registry, index })
}

/// Check index freshness, returning `{ "fresh": bool, "expected"?, "actual"?,
/// "diagnostics": { "warnings", "errors", "byCode" } }`.
///
/// The `diagnostics` member (spec 050) counts what the **committed** shards
/// record, so it is answered from the same ledger the freshness verdict is
/// about. `check_registry_freshness_json` keeps the bare shape: index
/// diagnostics say nothing about the registry.
pub fn check_freshness_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let root = std::path::Path::new(repo_root);
    let freshness = check_index_freshness(&config, root)?;
    let counts = diagnostics::committed_counts(&config, root)?;
    // Spec 057 §3.3: the facade and the CLI emit one shape. `cli.rs` pins them
    // against each other, and it caught this: a payload member added on one
    // side only is exactly the drift that test exists to refuse.
    to_json(&IndexCheckReport::with_unwitnessed(
        &freshness,
        counts,
        unwitnessed_counts(&config, root),
    ))
}

/// The distinct-path tally spec 057 §3.3 reports, shared by the facade and the
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

/// Check registry-shard freshness (spec 031), returning the same
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
/// 032), returning the [`CoverageReport`] as JSON. Freshness-guarded like
/// `couple`: a stale committed index is [`Error::Stale`] (exit 2), never a
/// report over the wrong ledger.
pub fn coverage_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    to_json(&coverage(&config, std::path::Path::new(repo_root))?)
}

/// Read a spec's declared acceptance (spec 049), returning the [`VerifyPlan`]
/// as JSON: the `verify:cli` commands its `## Verification` section holds, in
/// document order, and the fence tags it declined.
///
/// The plan is all the engine produces. **Running the commands is the caller's
/// act**, never this library's: spec 049 §3.1 keeps process execution on the
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

/// Render the committed index as markdown (spec 011). `index_json` is the
/// `index.json` text; the returned string is the markdown projection,
/// JSON-encoded (a JSON string literal).
pub fn render_json(config_json: &str, index_json: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let index = load_index(index_json.as_bytes())?;
    to_json(&render::render_markdown(&config, &index))
}

/// List the committed index's orphaned specs as a JSON array of id strings
/// (spec 011). `index_json` is the `index.json` text.
pub fn orphans_json(index_json: &str) -> Result<String, Error> {
    let index = load_index(index_json.as_bytes())?;
    to_json(&render::orphans(&index))
}

/// Parse a `spec-spine.toml` and return the normalized [`Config`] as JSON.
pub fn load_config_json(toml_src: &str) -> Result<String, Error> {
    let config = load_config(toml_src)?;
    to_json(&config)
}

/// Run the coupling gate. `request_json` bundles config + repo_root + diff +
/// optional waiver:
/// `{ "config"?: Config, "repoRoot": string, "diff": DiffInput, "waiver"?: { "reason": string } }`.
/// Returns the [`CoupleReport`] as JSON (even when drift is present; the caller
/// inspects `violations` / `waiver`).
pub fn couple_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        #[serde(default)]
        config: Config,
        repo_root: String,
        diff: DiffInput,
        #[serde(default)]
        waiver: Option<Waiver>,
    }

    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Parse(format!("invalid couple request: {e}")))?;
    let report = couple(
        &request.config,
        std::path::Path::new(&request.repo_root),
        &request.diff,
        request.waiver.as_ref(),
    )?;
    to_json(&report)
}

/// Generate the adopter scaffold for `config_json` (`"{}"` ⇒ defaults), returning
/// the [`Scaffold`] (files-as-data) as JSON. The caller writes the files.
pub fn scaffold_init_json(config_json: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    to_json(&scaffold_init(&config)?)
}

/// Build a corpus attestation (spec 023). Returns
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

/// Build a per-spec attestation (spec 042). Returns
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

/// Verify a per-spec attestation by recompute (spec 042 3.5). Request:
/// `{ "config"?: Config, "repoRoot": string, "attestation": <SpecAttestation> }`.
/// Same outcome vocabulary as [`verify_attestation_json`].
pub fn verify_spec_attestation_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        #[serde(default)]
        config: Config,
        repo_root: String,
        attestation: spec_spine_types::SpecAttestation,
    }
    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Parse(format!("invalid verify-spec-attestation request: {e}")))?;
    let outcome = verify_spec_recompute(
        &request.config,
        std::path::Path::new(&request.repo_root),
        &request.attestation,
    )?;
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

/// Verify an attestation by recompute (spec 023 FR-004 `--recompute`). Request:
/// `{ "config"?: Config, "repoRoot": string, "attestation": <CorpusAttestation> }`.
/// Returns `{ "outcome": "match" }`, `{ "outcome": "versionMismatch", "expected",
/// "actual" }`, or `{ "outcome": "contentMismatch", "differences": [...] }`. This
/// mode needs no key and no signature: any third party can run it.
pub fn verify_attestation_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        #[serde(default)]
        config: Config,
        repo_root: String,
        attestation: CorpusAttestation,
    }
    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Parse(format!("invalid verify-attestation request: {e}")))?;
    let outcome = verify_recompute(
        &request.config,
        std::path::Path::new(&request.repo_root),
        &request.attestation,
    )?;
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

fn config_from_json(config_json: &str) -> Result<Config, Error> {
    serde_json::from_str(config_json)
        .map_err(|e| Error::Config(format!("invalid config JSON: {e}")))
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<String, Error> {
    serde_json::to_string(value).map_err(|e| Error::Schema(e.to_string()))
}
