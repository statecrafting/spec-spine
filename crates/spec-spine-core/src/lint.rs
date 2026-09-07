//! The conformance lint (spec 003): corpus convention checks (`L-` codes),
//! disjoint from compile's structural `V-` codes. Severity gating
//! (error always / warning under `--fail-on-warn` / info under `--fail-on-info`)
//! is applied by the CLI; this layer just produces the diagnostics.

use std::collections::BTreeSet;
use std::path::Path;

use spec_spine_types::{Config, Error, Severity, SpecRecord, Unit, Violation};

use crate::compile::compile;

/// The result of a lint run.
pub struct LintReport {
    pub violations: Vec<Violation>,
}

impl LintReport {
    /// Count of violations at a given severity.
    pub fn count(&self, severity: Severity) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .count()
    }
}

/// Lint the corpus under `repo_root`: compile it, then run conformance checks.
pub fn lint(cfg: &Config, repo_root: &Path) -> Result<LintReport, Error> {
    let registry = compile(cfg, repo_root)?.registry;
    let ids: BTreeSet<&str> = registry.specs.iter().map(|s| s.id.as_str()).collect();
    let domains_enabled = !cfg.domains.allowed.is_empty();
    let kind_enabled = !cfg.kind.allowed.is_empty();

    let mut violations = Vec::new();
    for spec in &registry.specs {
        let at = || Some(spec.spec_path.clone());

        // L-001: ordinary spec claims no territory.
        let retroactive = spec.origin.as_ref().is_some_and(|o| o.retroactive);
        if !retroactive && !has_ownership_edge(spec) {
            violations.push(warn(
                "L-001",
                format!(
                    "spec '{}' declares no ownership edge (claims no territory)",
                    spec.id
                ),
                at(),
            ));
        }
        // L-002 / L-003: unclassified under an enabled taxonomy.
        if domains_enabled && spec.domain.is_none() {
            violations.push(warn(
                "L-002",
                format!("spec '{}' has no domain", spec.id),
                at(),
            ));
        }
        if kind_enabled && spec.kind.is_none() {
            violations.push(warn(
                "L-003",
                format!("spec '{}' has no kind", spec.id),
                at(),
            ));
        }
        // L-004: dangling edge target.
        for target in edge_targets(spec) {
            if !ids.contains(target.as_str()) {
                violations.push(warn(
                    "L-004",
                    format!("spec '{}' references unknown spec '{target}'", spec.id),
                    at(),
                ));
            }
        }
        // L-006: a unit claimed inside the declared, ungoverned state root
        // (spec 039 3.4). Error tier: neither the claim nor the bypass wins,
        // because letting the claim win would reintroduce spec 009's override
        // into a directory whose whole purpose is to be ungoverned, and letting
        // the bypass win would silently discard a unit an author wrote
        // deliberately. Both are wrong, so the corpus is told instead.
        for unit_path in claimed_paths(spec) {
            if cfg.layout.is_state_path(&unit_path) {
                violations.push(error(
                    "L-006",
                    format!(
                        "spec '{}' claims '{unit_path}', which is inside the ungoverned \
                         layout.state_dir '{}': move the file out of the state root, or \
                         stop claiming it",
                        spec.id, cfg.layout.state_dir
                    ),
                    at(),
                ));
            }
        }

        // L-007: a `depends_on` entry that does not point backward in filing
        // order (spec 053 §3.2). Opt-in: when the knob is off nothing is
        // emitted at all, not emitted-and-filtered, so a corpus that has not
        // opted in sees byte-identical output before and after spec 053.
        //
        // Error tier, matching `L-006`: the knob alone decides whether the
        // corpus is held to this, and an adopter who turned it on turned it on
        // to be refused. A warning would have meant two knobs (this one and
        // `--fail-on-warn`) for one decision.
        if cfg.lint.require_ordinal_monotonic_depends_on {
            for target in &spec.depends_on {
                if let Some((mine, theirs)) = ordinal_pair(&spec.id, target) {
                    if theirs >= mine {
                        violations.push(error(
                            "L-007",
                            format!(
                                "spec '{}' depends_on '{target}', which is not a lower \
                                 ordinal ({theirs:03} >= {mine:03}): a dependency points \
                                 backward in filing order",
                                spec.id
                            ),
                            at(),
                        ));
                    }
                }
            }
        }

        // L-005: stub (no body sections).
        if spec.section_headings.is_empty() {
            violations.push(info(
                "L-005",
                format!("spec '{}' has no body sections", spec.id),
                at(),
            ));
        }
    }

    // L-008 (spec 057): a claimed path that exists and that no content hash
    // covers. Its contents can be rewritten end to end with `index check` and
    // `compile --check` both reporting fresh, which is the sentence this
    // diagnostic qualifies.
    //
    // Read from the **committed** index rather than recomputed: `lint` is a
    // read verb and indexing here would make it write-shaped and slow. A corpus
    // with no committed index is silent, which is right: there is no ledger yet
    // for a claim to be invisible to.
    if let Ok(index) = crate::index::load_committed_index(cfg, repo_root) {
        for claim in crate::index::unwitnessed_claims(cfg, repo_root, &index)
            .into_iter()
            .filter(|c| !c.allowed)
        {
            let spec_path = registry
                .specs
                .iter()
                .find(|s| s.id == claim.spec_id)
                .map(|s| s.spec_path.clone());
            violations.push(warn(
                "L-008",
                format!(
                    "spec '{}' claims '{}', which is in no content hash: its contents can \
                     change without staling any shard. Add a covering glob to [index] \
                     extra_hashed_inputs, or claim a section or symbol unit, whose span \
                     is hashed",
                    claim.spec_id, claim.path
                ),
                spec_path,
            ));
        }
    }

    Ok(LintReport { violations })
}

/// Both ids' ordinals, or `None` when either lacks one (spec 053 §3.3).
///
/// Silence is the honest answer when the order is undefined. The corpus does
/// not require numeric ids: `V-001` requires only that the directory equal the
/// id, so `auth-login` is well-formed, and telling such a corpus that its ids
/// have no ordinal would be telling it that it does not hold a convention it
/// never claimed.
fn ordinal_pair(declaring: &str, target: &str) -> Option<(u64, u64)> {
    Some((ordinal(declaring)?, ordinal(target)?))
}

/// The leading decimal digit run of an id, as an integer.
///
/// Numeric rather than lexical, so a corpus that outgrows three digits and
/// files `1001-foo` orders above `999-bar` instead of below it. Iterating
/// `char`s rather than slicing bytes keeps this safe on a non-ASCII id, which
/// is the defect `detect_duplicates` carries and which spec 053 §4 declines to
/// fix from inside a lint change.
fn ordinal(id: &str) -> Option<u64> {
    let digits: String = id.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn has_ownership_edge(spec: &SpecRecord) -> bool {
    !spec.establishes.is_empty()
        || !spec.extends.is_empty()
        || !spec.refines.is_empty()
        || !spec.supersedes.is_empty()
        || !spec.amends.is_empty()
        || !spec.co_authority.is_empty()
        || !spec.constrains.is_empty()
}

/// Every spec id this spec names across its relationship edges.
fn edge_targets(spec: &SpecRecord) -> Vec<String> {
    let mut targets = Vec::new();
    targets.extend(spec.supersedes.iter().map(|s| s.spec().to_string()));
    targets.extend(spec.amends.iter().cloned());
    targets.extend(spec.extends.iter().map(|e| e.spec.clone()));
    targets.extend(spec.refines.iter().flat_map(|r| r.refines_specs.clone()));
    targets.extend(spec.co_authority.iter().flat_map(|c| c.with_specs.clone()));
    targets.extend(spec.constrains.iter().flat_map(|c| c.target_specs.clone()));
    targets
}

/// Every repo-relative path a spec claims through an ownership-bearing edge.
///
/// All six of them: `establishes`, `extends`, `refines`, `supersedes`,
/// `co_authority` and `constrains`. A partial `supersedes` item carries the unit
/// whose authority transfers (spec 019), so it claims a path exactly as the
/// others do; omitting it would let a superseding spec hold a claim inside the
/// state root that no diagnostic ever named, which is the contradiction `L-006`
/// exists to surface.
///
/// `references` is excluded: spec 034 settled that a cited file is not a claimed
/// one, so citing something inside the state root is not that contradiction.
/// `amends` is excluded too, because its subject is the amended spec's `spec.md`
/// rather than an arbitrary unit, and a `spec.md` lives under `specs_dir`, which
/// `state_dir` may not overlap.
///
/// Section and file units carry a path; symbol, crate and module units are
/// resolved by id and have none to test here.
fn claimed_paths(spec: &SpecRecord) -> Vec<String> {
    let mut paths = Vec::new();
    let mut push = |unit: &Unit| {
        if let Some(p) = unit_path(unit) {
            paths.push(p);
        }
    };
    for unit in &spec.establishes {
        push(unit);
    }
    for item in &spec.extends {
        if let Some(u) = &item.unit {
            push(u);
        }
    }
    for item in &spec.refines {
        if let Some(u) = &item.unit {
            push(u);
        }
    }
    for item in &spec.supersedes {
        if let spec_spine_types::SupersedeItem::Scoped(scoped) = item {
            if let Some(u) = &scoped.unit {
                push(u);
            }
        }
    }
    for item in &spec.co_authority {
        push(&item.unit);
    }
    for item in &spec.constrains {
        if let Some(u) = &item.unit {
            push(u);
        }
    }
    paths
}

/// The repo-relative path a unit names, for the unit kinds that carry one.
fn unit_path(unit: &Unit) -> Option<String> {
    match unit {
        Unit::File { path } | Unit::Directory { path } => Some(path.clone()),
        Unit::Section { file, .. } => Some(file.clone()),
        Unit::Symbol { .. } | Unit::Crate { .. } | Unit::Module { .. } => None,
    }
}

fn error(code: &str, message: String, path: Option<String>) -> Violation {
    Violation::new(code, Severity::Error, message).at_opt(path)
}

fn warn(code: &str, message: String, path: Option<String>) -> Violation {
    Violation::new(code, Severity::Warning, message).at_opt(path)
}

fn info(code: &str, message: String, path: Option<String>) -> Violation {
    Violation::new(code, Severity::Info, message).at_opt(path)
}
