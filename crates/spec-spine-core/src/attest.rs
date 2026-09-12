//! The attest capability (spec 023): a pure, reproducible attestation over the
//! spec corpus state.
//!
//! `attest` freezes a verdict spec-spine already computes (`compile`, `lint`,
//! and optionally `couple`) into a [`CorpusAttestation`]: a pure function of
//! `(config, file contents)` with no clock, no env, and **no key**. Re-running
//! it on an unchanged corpus at the same tool version yields a byte-identical
//! payload, which is exactly what makes `verify_recompute` runnable by any third
//! party with no key and no trust in the signer. Signing is a separate, key-only
//! post-pass that lives in the CLI (`seal.rs`); this module never touches a key.

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use spec_spine_types::{
    ATTESTATION_SCHEMA_VERSION, AttestedLifecycle, AttestedUnit, CompileVerdict, CorpusAttestation,
    CoupleVerdict, Error, LintVerdict, ResolutionVerdict, SPEC_ATTESTATION_SCHEMA_VERSION,
    Severity, SpecAttestation, SpecVerdicts, ToolStamp, Verdicts,
};

use crate::canonical_json;
use crate::compile::compile;
use crate::hash;
use crate::index::index;
use crate::lint::lint;
use crate::pathutil;

/// Resolver diagnostics that mark code as out of sync with its claiming spec.
/// A **local mirror** of `index.rs::BLOCKING_CODES` (kept equal by value and
/// this comment, not by linkage, so the attest capability does not take a code
/// dependency on the indexer's private constant): the same set that
/// `check_index_freshness` treats as a hard failure.
const BLOCKING_RESOLVER_CODES: &[&str] = &[
    "I-003", "I-004", "I-005", "I-006", "I-007", "I-008", "I-009",
];

/// Inputs to a [`attest`] run beyond `(config, file contents)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct AttestOptions {
    /// Record the coupling (specs-and-code-in-sync) verdict as well (FR-002).
    /// Without it the attestation covers spec-corpus state only.
    pub with_coupling: bool,
}

/// The result of an [`attest`] run: the typed attestation, its canonical JSON,
/// and the hash a consumer references and a seal signs.
pub struct AttestOutcome {
    pub attestation: CorpusAttestation,
    /// Canonical `CorpusAttestation` JSON (sorted keys, 2-space, trailing LF).
    pub json: String,
    /// SHA-256 (lowercase hex) over [`AttestOutcome::json`]. Emitted alongside,
    /// never inside, the attestation: it is the chain handle a consumer
    /// references and the message a [`crate::types::LedgerSeal`] signs.
    pub attestation_hash: String,
}

/// The outcome of a `--recompute` verification (FR-004/FR-005).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// The recomputed attestation is byte-identical to the supplied one.
    Match,
    /// The attestation was produced by a different tool version; recompute is
    /// not meaningful. A distinct, named outcome: never a false content
    /// mismatch, never a skip-as-pass (FR-005).
    VersionMismatch { expected: String, actual: String },
    /// The corpus recomputes to a different attestation. Each entry names a
    /// field that diverged (FR-004).
    ContentMismatch { differences: Vec<String> },
}

/// The difference reported when a payload's values are exactly what the corpus
/// recomputes and its stored bytes are not the ones that were attested (spec
/// 085 3.1).
pub const NON_CANONICAL_BYTES: &str = "bytes are not the canonical serialization";

/// Build a [`CorpusAttestation`] over the corpus under `repo_root`.
///
/// Pure function of `(config, file contents)`: it runs `compile` and `lint`
/// (and `index` under `--with-coupling`), all themselves pure, and hashes their
/// canonical outputs. No clock, no env, no key.
pub fn attest(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    opts: AttestOptions,
) -> Result<AttestOutcome, Error> {
    // compile: the inputs-manifest and registry hashes plus the compile verdict.
    let compiled = compile(cfg, repo_root)?;
    let inputs_manifest_hash = compiled.registry.build.content_hash.clone();
    let registry_hash = sha256_hex(compiled.json.as_bytes());
    let compile_ok = compiled.validation_passed;

    // lint: the verdict and a hash over the canonical findings. `ok` mirrors the
    // repo's own `lint --fail-on-warn` gate (no error and no warning; info-tier
    // is advisory), which is the meaningful "corpus is consistent" claim for an
    // audit attestation. `findings_hash` captures every finding (including info),
    // so any change in the findings set is detectable on recompute.
    let lint_report = lint(cfg, repo_root)?;
    let lint_ok =
        lint_report.count(Severity::Error) == 0 && lint_report.count(Severity::Warning) == 0;
    let findings_hash = sha256_hex(canonical_json::to_string(&lint_report.violations)?.as_bytes());

    // couple (optional, FR-002): specs and code are in sync iff the index built
    // with no blocking resolver diagnostic (every claimed unit resolves). Pure:
    // no git diff is needed, so the core stays git-free.
    let couple = if opts.with_coupling {
        let outcome = index(cfg, repo_root)?;
        let index_hash = outcome.index.build.content_hash.clone();
        let blocking = outcome
            .index
            .diagnostics
            .errors
            .iter()
            .any(|d| BLOCKING_RESOLVER_CODES.contains(&d.code.as_str()));
        let join_hash = sha256_hex(format!("{registry_hash}:{index_hash}").as_bytes());
        Some(CoupleVerdict {
            ok: !blocking,
            index_hash,
            join_hash,
        })
    } else {
        None
    };

    let attestation = CorpusAttestation {
        schema_version: ATTESTATION_SCHEMA_VERSION.to_string(),
        tool: ToolStamp {
            name: cfg.branding.compiler_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        inputs_manifest_hash,
        registry_hash,
        verdicts: Verdicts {
            compile: CompileVerdict { ok: compile_ok },
            lint: LintVerdict {
                ok: lint_ok,
                findings_hash,
            },
            couple,
        },
    };

    let json = canonical_json::to_string(&attestation)?;
    let hash = sha256_hex(json.as_bytes());
    Ok(AttestOutcome {
        attestation,
        json,
        attestation_hash: hash,
    })
}

/// Build a [`SpecAttestation`] over one spec's territory (spec 042).
///
/// Pure function of `(config, file contents)`, exactly as [`attest`]: it runs
/// `compile`, `lint` and `index`, all themselves pure, and reduces their output
/// to this spec. No clock, no environment, no git.
///
/// **A failing verdict never suppresses the payload.** This returns a complete,
/// hashable, signable attestation whether the verdicts are true or false, and
/// the CLI exits `0` for having produced one. It is a record, not a gate: an
/// attestation that refused to exist when the news was bad would be worth
/// nothing as evidence, and a caller wanting a gate calls the verb that refuses.
pub fn attest_spec(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    spec_id: &str,
) -> Result<SpecAttestOutcome, Error> {
    let compiled = compile(cfg, repo_root)?;
    let record = compiled
        .registry
        .specs
        .iter()
        .find(|s| s.id == spec_id)
        .ok_or_else(|| Error::NotFound(format!("spec '{spec_id}'")))?;

    let spec_source_hash = {
        let path = repo_root.join(&record.spec_path);
        let text = fs::read_to_string(&path)
            .map_err(|e| Error::Io(format!("read {}: {e}", path.display())))?;
        // The project's standing normalization (BOM stripped, CRLF/CR to LF),
        // so the payload is platform-independent like every other hash here.
        sha256_hex(hash::normalize(&text).as_bytes())
    };

    // The resolved territory: owning units only, in the registry's canonical
    // order, each with the content hash of what it resolved to.
    let indexed = index(cfg, repo_root)?;
    let mapping = indexed
        .index
        .traceability
        .mappings
        .iter()
        .find(|m| m.spec_id == spec_id);
    let mut units: Vec<AttestedUnit> = Vec::new();
    let mut all_resolved = true;
    for resolved in mapping.into_iter().flat_map(|m| &m.resolved_units) {
        if !resolved.ownership {
            continue;
        }
        if resolved.locations.is_empty() {
            all_resolved = false;
            units.push(AttestedUnit {
                unit: resolved.unit.clone(),
                content_hash: None,
            });
            continue;
        }
        // A unit may resolve to several locations (a subtree, a module split
        // across files). One hash over the sorted set, which is exactly what
        // `hash::content_hash` is: path-sorted, normalized, NUL-separated.
        // A read failure propagates rather than being skipped. Dropping it
        // would hash only the locations that could be read, or nothing at all
        // if none could, and still emit `Some(hash)` with `resolution.ok` left
        // true: an attestation reporting a resolved unit whose files are gone,
        // which is the one thing 3.1 says this payload exists to make
        // impossible. Same treatment `specSourceHash` gives an unreadable
        // `spec.md` above.
        let mut pieces: Vec<(String, String)> = Vec::with_capacity(resolved.locations.len());
        for loc in &resolved.locations {
            let path = repo_root.join(&loc.file);
            // Spec 083 3.1: the branch is on what the location IS, never on the
            // unit's declared kind. A `directory` unit and a trailing-slash
            // `file` unit both resolve to a directory (spec 000 4.2 calls them
            // one concept), and a `crate` unit resolves to a package root, so a
            // match on `Unit::Directory` would leave most of the corpus opening
            // a directory as a file and failing at exit 3.
            if path.is_dir() {
                let entries = crate::index::walk_territory(
                    &path,
                    repo_root,
                    &cfg.index.resolver_exclusions,
                    &cfg.layout,
                );
                if entries.is_empty() {
                    // Spec 083 3.3: an empty or fully pruned directory hashes
                    // its own repo-relative POSIX path. Hashing nothing would
                    // emit SHA-256 of the empty input, one constant shared by
                    // every empty claim in every repository, which reads as
                    // evidence and distinguishes nothing.
                    pieces.push((pathutil::rel_posix(repo_root, &path), String::new()));
                    continue;
                }
                for entry in entries {
                    match entry {
                        crate::index::TerritoryEntry::File(file) => {
                            let bytes = fs::read(&file).map_err(|e| {
                                Error::Io(format!(
                                    "read {} for spec '{spec_id}' unit {:?}: {e}",
                                    file.display(),
                                    resolved.unit
                                ))
                            })?;
                            // Spec 083 3.2 + D-5: a claimed subtree contains
                            // whatever it contains, so the walk meets bytes that
                            // are not UTF-8 (an image, a `.DS_Store`). Text
                            // keeps the standing normalization; anything else
                            // contributes a digest of its exact bytes, which is
                            // deterministic, never lossy, and cannot be dropped.
                            // Refusing here would put 3.1's guarantee back out
                            // of reach for twelve of the fourteen specs.
                            let content = match String::from_utf8(bytes) {
                                Ok(text) => text,
                                Err(e) => {
                                    format!("sha256:{}", sha256_hex(e.as_bytes()))
                                }
                            };
                            pieces.push((pathutil::rel_posix(repo_root, &file), content));
                        }
                        // Spec 083 3.2: the link's target text, never its
                        // target's content. That records a retarget (3.4) while
                        // keeping content outside the repository out of the
                        // payload, and makes a cycle unreachable.
                        crate::index::TerritoryEntry::Symlink { path, target } => {
                            pieces.push((pathutil::rel_posix(repo_root, &path), target));
                        }
                    }
                }
                continue;
            }
            let content = fs::read_to_string(&path).map_err(|e| {
                Error::Io(format!(
                    "read {} for spec '{spec_id}' unit {:?}: {e}",
                    path.display(),
                    resolved.unit
                ))
            })?;
            pieces.push((loc.file.clone(), content));
        }
        units.push(AttestedUnit {
            unit: resolved.unit.clone(),
            content_hash: Some(hash::content_hash(pieces)),
        });
    }

    // Verdicts restricted to this spec. `lint.ok` and `findingsHash` cover the
    // findings attributed to it, using 023's findings-hash construction so a
    // changed finding set is detectable even when `ok` is unchanged.
    //
    // Attribution is by `path`, so a violation carrying none would belong to no
    // spec and appear in no per-spec attestation. Every current `L-` rule sets
    // the offending spec's path, so nothing is dropped today; a future
    // cross-cutting rule that did not would need a scoping answer here rather
    // than inheriting silence. The corpus-scoped attestation (spec 023) hashes
    // the whole findings set and is unaffected either way.
    let lint_report = lint(cfg, repo_root)?;
    let mine: Vec<_> = lint_report
        .violations
        .iter()
        .filter(|v| v.path.as_deref() == Some(record.spec_path.as_str()))
        .cloned()
        .collect();
    let lint_ok = !mine
        .iter()
        .any(|v| matches!(v.severity, Severity::Error | Severity::Warning));
    let findings_hash = sha256_hex(canonical_json::to_string(&mine)?.as_bytes());

    // Error-tier only, which is what `validation.passed` means and therefore
    // what spec 023's corpus-scoped `compile.ok` reports. `lint_ok` above uses
    // the wider error-or-warning floor for the same reason: it mirrors the
    // `--fail-on-warn` gate 023 chose, a lint report having no single flag to
    // read. Both floors are copied exactly so the two scopes stay comparable
    // (spec 042 D-4).
    let compile_ok = !compiled
        .registry
        .validation
        .violations
        .iter()
        .any(|v| v.severity == Severity::Error && v.path.as_deref() == Some(&record.spec_path));

    let attestation = SpecAttestation {
        schema_version: SPEC_ATTESTATION_SCHEMA_VERSION.to_string(),
        tool: ToolStamp {
            name: cfg.branding.compiler_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        spec_id: spec_id.to_string(),
        spec_source_hash,
        lifecycle: AttestedLifecycle {
            status: status_label(record.status).to_string(),
            // Emitted in its canonical kebab-case spelling: spec 015 accepts
            // `n/a` on the way in and normalizes it, so two specs written in
            // different dialects attest identically.
            implementation: record
                .implementation
                .map(implementation_label)
                .map(str::to_string),
        },
        units,
        verdicts: SpecVerdicts {
            compile: CompileVerdict { ok: compile_ok },
            resolution: ResolutionVerdict { ok: all_resolved },
            lint: LintVerdict {
                ok: lint_ok,
                findings_hash,
            },
        },
    };

    let json = canonical_json::to_string(&attestation)?;
    let hash = sha256_hex(json.as_bytes());
    Ok(SpecAttestOutcome {
        attestation,
        json,
        attestation_hash: hash,
    })
}

/// The result of an [`attest_spec`] run, mirroring [`AttestOutcome`].
#[derive(Clone, Debug)]
pub struct SpecAttestOutcome {
    pub attestation: SpecAttestation,
    /// Canonical `SpecAttestation` JSON (sorted keys, 2-space, trailing LF).
    pub json: String,
    /// SHA-256 (lowercase hex) over [`SpecAttestOutcome::json`].
    pub attestation_hash: String,
}

/// Verify a [`SpecAttestation`] by recompute (spec 042 3.5), mirroring
/// [`verify_recompute`]: a `tool.version` mismatch stays a distinct, named
/// outcome rather than a false content mismatch.
pub fn verify_spec_recompute(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    attestation: &SpecAttestation,
) -> Result<VerifyOutcome, Error> {
    let actual_version = env!("CARGO_PKG_VERSION");
    if attestation.tool.version != actual_version {
        return Ok(VerifyOutcome::VersionMismatch {
            expected: attestation.tool.version.clone(),
            actual: actual_version.to_string(),
        });
    }
    let recomputed = attest_spec(cfg, repo_root, &attestation.spec_id)?.attestation;
    if recomputed == *attestation {
        return Ok(VerifyOutcome::Match);
    }

    // Field-level differences, so the report names what moved rather than
    // saying only that something did.
    let mut differences = Vec::new();
    let (a, b) = (attestation, &recomputed);
    // Named rather than folded into the catch-all below (spec 085 3.3): a
    // report reading "tool.name or schemaVersion" tells a consumer which two
    // fields to go and diff by hand, which is the work it was meant to save.
    if a.schema_version != b.schema_version {
        differences.push(format!(
            "schemaVersion ({} -> {})",
            a.schema_version, b.schema_version
        ));
    }
    if a.tool.name != b.tool.name {
        differences.push(format!("tool.name ({} -> {})", a.tool.name, b.tool.name));
    }
    // The recompute is keyed on the attestation's own `spec_id`, so this
    // differs only when that id resolved to a spec recorded under another
    // spelling of it, which is exactly the case worth naming.
    if a.spec_id != b.spec_id {
        differences.push(format!("specId ({} -> {})", a.spec_id, b.spec_id));
    }
    if a.spec_source_hash != b.spec_source_hash {
        differences.push("specSourceHash (the spec's own text changed)".to_string());
    }
    if a.lifecycle != b.lifecycle {
        differences.push(format!(
            "lifecycle ({:?} -> {:?})",
            a.lifecycle, b.lifecycle
        ));
    }
    // Matched by unit, not by position. Zipping would compare unrelated units
    // after an insertion and report every one of them as a changed hash, so the
    // count line would arrive buried in noise it caused.
    for before in &a.units {
        match b.units.iter().find(|after| after.unit == before.unit) {
            Some(after) if after.content_hash != before.content_hash => {
                differences.push(format!("units[{:?}].contentHash", before.unit));
            }
            Some(_) => {}
            None => differences.push(format!("units[{:?}] is gone", before.unit)),
        }
    }
    for after in &b.units {
        if !a.units.iter().any(|before| before.unit == after.unit) {
            differences.push(format!("units[{:?}] is new", after.unit));
        }
    }
    if a.verdicts != b.verdicts {
        differences.push(format!("verdicts ({:?} -> {:?})", a.verdicts, b.verdicts));
    }
    // Unreachable: `tool.version` was gated above and every other member is
    // compared here. Kept so a member added later without a comparison reports
    // something rather than an empty difference list.
    if differences.is_empty() {
        differences.push("an unnamed member".to_string());
    }
    Ok(VerifyOutcome::ContentMismatch { differences })
}

/// The hash a seal signs and a consumer references, for a per-spec attestation.
pub fn spec_attestation_hash(attestation: &SpecAttestation) -> Result<String, Error> {
    Ok(sha256_hex(
        canonical_json::to_string(attestation)?.as_bytes(),
    ))
}

/// The canonical lowercase label for a status.
fn status_label(status: spec_spine_types::Status) -> &'static str {
    use spec_spine_types::Status;
    match status {
        Status::Draft => "draft",
        Status::Approved => "approved",
        Status::Superseded => "superseded",
        Status::Retired => "retired",
    }
}

/// The canonical kebab-case label for an implementation value.
fn implementation_label(implementation: spec_spine_types::Implementation) -> &'static str {
    use spec_spine_types::Implementation;
    match implementation {
        Implementation::Pending => "pending",
        Implementation::InProgress => "in-progress",
        Implementation::Complete => "complete",
        Implementation::Na => "n-a",
        Implementation::Deferred => "deferred",
    }
}

/// The hash a seal signs and a consumer references: SHA-256 (lowercase hex) over
/// the canonical JSON of `attestation`. Recomputing it from a *loaded* payload
/// is what keeps a signature check tamper-evident: a single changed byte changes
/// the canonical bytes and therefore the hash, so the seal no longer verifies.
pub fn attestation_hash(attestation: &CorpusAttestation) -> Result<String, Error> {
    Ok(sha256_hex(
        canonical_json::to_string(attestation)?.as_bytes(),
    ))
}

/// SHA-256 (lowercase hex) over the stored bytes of an attestation file: the
/// digest a seal is checked against (spec 085 3.1).
///
/// For every file `attest` has written the file *is* the canonical JSON, so this
/// equals [`attestation_hash`] and every seal ever produced verifies exactly as
/// before. For any other file the two differ, and that difference is the point:
/// a verifier that approves a record approves the bytes it was handed, not a
/// re-serialization of its own parse of them.
pub fn stored_bytes_hash(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

/// Read `schemaVersion` out of a JSON payload before it is strictly parsed
/// (spec 085 3.3).
///
/// Reading it loosely first is what lets a payload from a MAJOR line this build
/// does not understand be refused as unreadable rather than as an unknown
/// member: a later producer's additive field would otherwise be the first thing
/// the strict parse trips on, and "unknown field `obligations`" is a poor way of
/// saying "this file was written by a later spec-spine".
pub fn payload_schema_version(bytes: &[u8], what: &str) -> Result<String, Error> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| Error::Parse(format!("invalid {what}: {e}")))?;
    match value.get("schemaVersion").and_then(|v| v.as_str()) {
        Some(version) => Ok(version.to_string()),
        None => Err(Error::Schema(format!(
            "{what} has no string schemaVersion, so its schema line cannot be checked"
        ))),
    }
}

/// Refuse a corpus attestation whose schema MAJOR this build does not
/// understand (spec 085 3.3), in the registry and index loaders' own words.
pub fn check_attestation_major(schema_version: &str) -> Result<(), Error> {
    crate::shard::check_major("attestation", schema_version, ATTESTATION_SCHEMA_VERSION)
}

/// Refuse a per-spec attestation whose schema MAJOR this build does not
/// understand (spec 085 3.3).
pub fn check_spec_attestation_major(schema_version: &str) -> Result<(), Error> {
    crate::shard::check_major(
        "spec attestation",
        schema_version,
        SPEC_ATTESTATION_SCHEMA_VERSION,
    )
}

/// Fold spec 085 3.1's byte comparison into a corpus recompute outcome.
///
/// A [`VerifyOutcome::Match`] says the parsed values are what the corpus
/// recomputes. It becomes a content mismatch when the stored bytes are not that
/// value's canonical serialization, because the digest a record is referenced by
/// is over the stored bytes: approving a re-serialization approves an object
/// nobody stored. Any other outcome passes through unchanged, since an
/// unreadable or a differing payload already carries the more specific answer.
pub fn with_stored_bytes(
    outcome: VerifyOutcome,
    attestation: &CorpusAttestation,
    stored: &[u8],
) -> Result<VerifyOutcome, Error> {
    match outcome {
        VerifyOutcome::Match => Ok(byte_outcome(
            &canonical_json::to_string(attestation)?,
            stored,
        )),
        other => Ok(other),
    }
}

/// [`with_stored_bytes`] for a per-spec attestation (spec 085 3.1).
pub fn with_stored_bytes_spec(
    outcome: VerifyOutcome,
    attestation: &SpecAttestation,
    stored: &[u8],
) -> Result<VerifyOutcome, Error> {
    match outcome {
        VerifyOutcome::Match => Ok(byte_outcome(
            &canonical_json::to_string(attestation)?,
            stored,
        )),
        other => Ok(other),
    }
}

fn byte_outcome(canonical: &str, stored: &[u8]) -> VerifyOutcome {
    if stored == canonical.as_bytes() {
        VerifyOutcome::Match
    } else {
        VerifyOutcome::ContentMismatch {
            differences: vec![NON_CANONICAL_BYTES.to_string()],
        }
    }
}

/// Re-read the corpus and verify it still recomputes to `attestation`
/// (FR-004 `--recompute`). Version-aware (FR-005): if the attestation's
/// `tool.version` differs from this build's, the result is
/// [`VerifyOutcome::VersionMismatch`], not a content mismatch. The recompute
/// scope mirrors the attestation (coupling is recomputed iff the attestation
/// carries a coupling block).
pub fn verify_recompute(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    attestation: &CorpusAttestation,
) -> Result<VerifyOutcome, Error> {
    let actual_version = env!("CARGO_PKG_VERSION");
    if attestation.tool.version != actual_version {
        return Ok(VerifyOutcome::VersionMismatch {
            expected: attestation.tool.version.clone(),
            actual: actual_version.to_string(),
        });
    }

    let recomputed = attest(
        cfg,
        repo_root,
        AttestOptions {
            with_coupling: attestation.verdicts.couple.is_some(),
        },
    )?
    .attestation;

    let mut differences = Vec::new();
    let a = attestation;
    let b = &recomputed;
    // Compared like every other member (spec 085 3.3). Skipping it let an
    // attestation claiming a schema this build never emitted recompute as a
    // match: the values agreed, and the one field saying what shape they were
    // written in went unread.
    if a.schema_version != b.schema_version {
        differences.push(format!(
            "schemaVersion ({} -> {})",
            a.schema_version, b.schema_version
        ));
    }
    if a.tool.name != b.tool.name {
        differences.push(format!("tool.name ({} -> {})", a.tool.name, b.tool.name));
    }
    if a.inputs_manifest_hash != b.inputs_manifest_hash {
        differences.push("inputsManifestHash (corpus inputs changed)".to_string());
    }
    if a.registry_hash != b.registry_hash {
        differences.push("registryHash (compiled registry changed)".to_string());
    }
    if a.verdicts.compile.ok != b.verdicts.compile.ok {
        differences.push(format!(
            "verdicts.compile.ok ({} -> {})",
            a.verdicts.compile.ok, b.verdicts.compile.ok
        ));
    }
    if a.verdicts.lint.ok != b.verdicts.lint.ok {
        differences.push(format!(
            "verdicts.lint.ok ({} -> {})",
            a.verdicts.lint.ok, b.verdicts.lint.ok
        ));
    }
    if a.verdicts.lint.findings_hash != b.verdicts.lint.findings_hash {
        differences.push("verdicts.lint.findingsHash (lint findings changed)".to_string());
    }
    match (&a.verdicts.couple, &b.verdicts.couple) {
        (Some(av), Some(bv)) => {
            if av.ok != bv.ok {
                differences.push(format!("verdicts.couple.ok ({} -> {})", av.ok, bv.ok));
            }
            if av.index_hash != bv.index_hash {
                differences.push("verdicts.couple.indexHash (code index changed)".to_string());
            }
            if av.join_hash != bv.join_hash {
                differences.push("verdicts.couple.joinHash".to_string());
            }
        }
        (None, None) => {}
        // Scope is recomputed from the attestation, so this is unreachable in
        // practice; recorded as a difference rather than panicking.
        _ => differences.push("verdicts.couple (scope presence changed)".to_string()),
    }

    if differences.is_empty() {
        Ok(VerifyOutcome::Match)
    } else {
        Ok(VerifyOutcome::ContentMismatch { differences })
    }
}

/// SHA-256 (lowercase hex) over raw bytes. (Distinct from
/// `hash::content_hash`, which hashes path-keyed, normalized input pieces; here
/// the inputs are already-canonical bytes.)
fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_is_lowercase_64_hex() {
        let h = sha256_hex(b"abc");
        assert_eq!(
            h,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(h.len(), 64);
    }
}
