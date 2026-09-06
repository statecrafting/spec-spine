//! `spec-spine lint`: corpus conformance lint with tiered fail gating.

use std::path::Path;

use spec_spine_core::lint;
use spec_spine_types::{Error, Severity, Verdict, verdict::verb};

use crate::load_repo_config;
use crate::out;

/// Returns the exit code: `1` if any error-tier diagnostic (always), or any
/// warning/info under the matching `--fail-on-*` flag; otherwise `0`.
///
/// `json` (spec 037) replaces the stdout prose with one verdict envelope whose
/// `report` is the violation array `spec_spine_core::lint_json` returns. The
/// `--fail-on-*` flags still decide the code, so the same corpus can be `ok`
/// under one gating policy and not under another with an identical report.
pub fn run(repo: &Path, fail_on_warn: bool, fail_on_info: bool, json: bool) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;
    let report = lint(&cfg, repo)?;

    let errors = report.count(Severity::Error);
    let warnings = report.count(Severity::Warning);
    let infos = report.count(Severity::Info);
    let fail = errors > 0 || (fail_on_warn && warnings > 0) || (fail_on_info && infos > 0);
    let code = if fail { 1 } else { 0 };

    if json {
        let value =
            serde_json::to_value(&report.violations).map_err(|e| Error::Schema(e.to_string()))?;
        out::verdict(&Verdict::report(verb::LINT, code, value))?;
        return Ok(code);
    }

    for v in &report.violations {
        let tier = match v.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        let at = v.path.as_deref().unwrap_or("-");
        outln!("  {} [{tier}] [{}] {}", v.code, at, v.message);
    }

    outln!("lint: {errors} error(s), {warnings} warning(s), {infos} info");
    Ok(code)
}
