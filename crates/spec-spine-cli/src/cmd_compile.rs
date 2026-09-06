//! `spec-spine compile`: write the per-spec registry shards (deterministic;
//! spec 024) under `<derived_dir>/spec-registry/by-spec/`, plus the wall-clock
//! `build-meta.json` sidecar. The single monolithic `registry.json` is no
//! longer emitted, so two PRs that add or edit different specs write disjoint
//! files and never conflict on a global content-hash line.
//!
//! `--check` (spec 031) is the non-writing form: it compiles in memory and
//! compares against the committed shards, the registry counterpart of
//! `index check`.

use std::fs;
use std::path::Path;

use spec_spine_core::shard::{self, BY_SPEC_DIR};
use spec_spine_core::{
    CompileOutcome, Freshness, compare_committed_registry, registry_dir, registry_shard_files,
};
use spec_spine_types::{
    BUILD_META_SCHEMA_VERSION, BuildMeta, Error, Severity, Verdict, verdict::verb,
};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::load_repo_config;
use crate::out;

/// Returns the process exit code.
///
/// Writing form: `0` if validation passed, `1` if it failed. `--check` form:
/// `0` fresh, `1` validation failed, `2` stale (spec 031 §3.2). Validation
/// outranks staleness, because a corpus that does not validate cannot vouch
/// for its shards.
///
/// `json` (spec 037) replaces the stdout prose with one verdict envelope and
/// changes no exit code. It is rejected without `--check`, per spec 037 §4: the
/// writing form's purpose is to mutate `.derived`, and handing a driver a
/// machine-readable verdict from the command that just regenerated the shards
/// the next gate compares against invites exactly the mid-chain confusion the
/// non-writing `--check` exists to avoid.
pub fn run(repo: &Path, check: bool, json: bool) -> Result<u8, Error> {
    if json && !check {
        // Written here rather than raised for `main` to render: `json_verb`
        // deliberately maps only `compile --check`, so a caller that asked for
        // JSON still gets JSON and the envelope's verb is a local, documented
        // choice (`compile.check` is the only machine-readable form of this
        // command, and the message names the flag that reaches it).
        let err = Error::Config(
            "compile --json requires --check: the writing form has no machine-readable \
             verdict (spec 037 4); use `compile --check --json`"
                .to_string(),
        );
        crate::emit_error_envelope(verb::COMPILE_CHECK, &err);
        return Ok(err.exit_code());
    }
    let cfg = load_repo_config(repo)?;
    let outcome = spec_spine_core::compile(&cfg, repo)?;

    if check {
        if !outcome.validation_passed {
            // Detail stays on stderr in both modes so a CI log shows the
            // violations; under --json the *verdict* is the envelope `main`
            // renders from this error, whose kind is `validation` and whose
            // exit code is the 1 the prose form returns.
            report_validation_failure(&outcome);
            if json {
                return Err(Error::Validation(
                    outcome.registry.validation.violations.clone(),
                ));
            }
            return Ok(1);
        }
        let freshness = compare_committed_registry(&cfg, repo, &outcome.shards)?;
        if json {
            let code = if matches!(freshness, Freshness::Fresh) {
                0
            } else {
                2
            };
            out::verdict(&Verdict::report(
                verb::COMPILE_CHECK,
                code,
                freshness_report(&freshness),
            ))?;
            return Ok(code);
        }
        return match freshness {
            Freshness::Fresh => {
                outln!(
                    "spec-registry is fresh: {} shard(s) match the corpus",
                    outcome.registry.specs.len()
                );
                Ok(0)
            }
            // Stale detail goes to stderr so it surfaces in a CI log. `actual`
            // is already the count line plus one line per stale shard; the
            // paired `expected` ("N shard(s) matching the corpus") is
            // deliberately not printed, because the operator's next action does
            // not depend on it. It stays on the typed verdict for library and
            // JSON-facade consumers.
            Freshness::Stale { actual, .. } => {
                eprintln!("{actual}");
                eprintln!("spec-registry is STALE: run `spec-spine compile` and commit the result");
                Ok(2)
            }
        };
    }

    let out_dir = registry_dir(&cfg, repo);
    fs::create_dir_all(&out_dir)
        .map_err(|e| Error::Io(format!("create {}: {e}", out_dir.display())))?;

    // Per-spec shards. `sync_dir` prunes a removed spec's shard, so the shard set
    // always equals the current corpus.
    let shard_files = registry_shard_files(&outcome.shards)?;
    let by_spec = out_dir.join(BY_SPEC_DIR);
    shard::sync_dir(&by_spec, &shard_files)?;

    // Drop a pre-024 monolithic registry.json on upgrade (it is no longer the
    // committed form; the shard tree supersedes it).
    let legacy = out_dir.join("registry.json");
    if legacy.exists() {
        fs::remove_file(&legacy)
            .map_err(|e| Error::Io(format!("remove {}: {e}", legacy.display())))?;
    }

    // build-meta.json carries the wall clock; the CLI owns it. Excluded from
    // determinism/golden checks and from version control (see .gitignore).
    let meta = BuildMeta {
        schema_version: BUILD_META_SCHEMA_VERSION.to_string(),
        built_at: now_rfc3339(),
        compiler_id: cfg.branding.compiler_id.clone(),
        compiler_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let meta_json =
        serde_json::to_string_pretty(&meta).map_err(|e| Error::Schema(e.to_string()))? + "\n";
    let meta_path = out_dir.join("build-meta.json");
    fs::write(&meta_path, meta_json)
        .map_err(|e| Error::Io(format!("write {}: {e}", meta_path.display())))?;

    let warnings = outcome
        .registry
        .validation
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .count();

    if outcome.validation_passed {
        outln!(
            "compiled {} spec(s) -> {} ({} warning(s))",
            outcome.registry.specs.len(),
            by_spec.display(),
            warnings
        );
        Ok(0)
    } else {
        report_validation_failure(&outcome);
        Ok(1)
    }
}

/// The `{ fresh, expected?, actual? }` value, byte-for-byte the shape
/// `spec_spine_core::check_registry_freshness_json` (and its index twin
/// `check_freshness_json`) return, so a consumer handles one freshness type for
/// both committed trees.
///
/// Rebuilt here rather than shared with the facade because the facade recompiles
/// the corpus to answer, which is a second full compile pass on a gate that runs
/// in CI; the CLI already holds the typed verdict. `cli.rs` pins the two against
/// each other so the duplication cannot drift silently.
pub(crate) fn freshness_report(freshness: &Freshness) -> serde_json::Value {
    match freshness {
        Freshness::Fresh => serde_json::json!({ "fresh": true }),
        Freshness::Stale { expected, actual } => {
            serde_json::json!({ "fresh": false, "expected": expected, "actual": actual })
        }
    }
}

/// Print every error-tier violation, then the summary. Always stderr, so the
/// failures surface in a CI log. Shared by the writing and `--check` forms so
/// both fail with the same diagnostic.
fn report_validation_failure(outcome: &CompileOutcome) {
    let violations = &outcome.registry.validation.violations;
    for v in violations {
        if v.severity == Severity::Error {
            let at = v.path.as_deref().unwrap_or("-");
            eprintln!("  {} [{}] {}", v.code, at, v.message);
        }
    }
    let errors = violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .count();
    let warnings = violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .count();
    eprintln!(
        "validation FAILED: {errors} error(s), {warnings} warning(s) across {} spec(s)",
        outcome.registry.specs.len()
    );
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}
