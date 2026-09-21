//! The authority snapshot (spec 070): the third attestation scope.
//!
//! [`snapshot`] records, for one tree and one tool version, which inputs were
//! read and what they came to, in one payload a consumer can store beside a
//! revision: the configuration, spec 021's corpus hashes, the committed registry
//! and index trees with whether each equals the recompute, the governance inputs
//! by path, the gate verdicts as counts, every spec's lifecycle with a framed
//! digest of its territory, and the exclusions in force.
//!
//! Pure function of `(config, file contents)`, the committed shard files
//! included: no clock, no environment, no git, no key. Signing is the CLI's
//! post-pass, exactly as for the other two scopes.
//!
//! Every digest this record introduces is [`frame_digest`] (`frame/1`, §3.3).
//! The fold in `hash::content_hash` writes nothing between one piece's content
//! and the next piece's path, so two different trees can share one hash; the
//! framed construction length-prefixes both and tags each piece with its kind.
//! It binds **normalized text** for a UTF-8 file and exact bytes for any other,
//! which is a different contract from spec 068's exact-byte one, on purpose.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use spec_spine_types::{
    ATTESTATION_SCHEMA_VERSION, AuthoritySnapshot, CommittedTree, Error, FRAME_DIGEST,
    INDEX_SCHEMA_VERSION, NON_UTF8_DIRECT_CLAIM, REGISTRY_SCHEMA_VERSION, SNAPSHOT_SCHEMA_VERSION,
    SPEC_ATTESTATION_SCHEMA_VERSION, Severity, SnapshotCommitted, SnapshotCompileVerdict,
    SnapshotConfig, SnapshotCorpus, SnapshotExclusions, SnapshotGovernanceInputs,
    SnapshotOwnership, SnapshotResolutionVerdict, SnapshotSchemas, SnapshotSpec,
    SnapshotUnwitnessed, SnapshotVerdicts, ToolStamp,
};

use crate::attest::{SpecUnitsError, VerifyOutcome};
use crate::canonical_json;
use crate::hash;
use crate::index::TerritoryEntry;
use crate::pathutil::rel_posix;
use crate::shard;

/// The domain-separation prefix every `frame/1` digest starts with (§3.3).
const FRAME_PREFIX: &[u8] = b"spec-spine/frame/1";

/// Resolver diagnostics that block: the corpus attestation's own list, shared
/// rather than copied, so the snapshot's `resolution.blocking` and spec 021's
/// `couple.ok` can never count different codes.
use crate::attest::BLOCKING_RESOLVER_CODES;

/// What a piece is, which is part of what is hashed (§3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PieceKind {
    /// A UTF-8 file, normalized (BOM stripped, CRLF and CR folded to LF).
    Text,
    /// Any other file, as its exact bytes.
    Bytes,
    /// A symlink, as its target text; never followed.
    Link,
    /// An empty or wholly pruned directory; content always empty.
    EmptyDir,
}

impl PieceKind {
    fn byte(self) -> u8 {
        match self {
            PieceKind::Text => b't',
            PieceKind::Bytes => b'b',
            PieceKind::Link => b'l',
            PieceKind::EmptyDir => b'd',
        }
    }
}

/// A set of pieces keyed by repo-relative POSIX path.
///
/// A path appears at most once (§3.3.1), which is what `frame/1`'s injectivity
/// is a property of. The first piece inserted for a path is kept, so an
/// overlapping claim is not a doubled one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PieceSet {
    pieces: BTreeMap<String, (PieceKind, Vec<u8>)>,
}

impl PieceSet {
    /// An empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a piece unless its path is already present.
    pub fn insert(&mut self, path: String, kind: PieceKind, content: Vec<u8>) {
        self.pieces.entry(path).or_insert((kind, content));
    }

    /// Add a file's content as a `t` piece when it is UTF-8 (normalized first)
    /// and a `b` piece otherwise.
    pub fn insert_file_content(&mut self, path: String, bytes: Vec<u8>) {
        match String::from_utf8(bytes) {
            Ok(text) => self.insert(path, PieceKind::Text, hash::normalize(&text).into_bytes()),
            Err(e) => self.insert(path, PieceKind::Bytes, e.into_bytes()),
        }
    }

    pub fn len(&self) -> usize {
        self.pieces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    /// The `frame/1` digest, or `None` for an empty set: the digest of nothing
    /// is one constant shared by every empty set everywhere, which reads as
    /// evidence and distinguishes nothing (§3.2, §3.3.1).
    pub fn digest(&self) -> Option<String> {
        (!self.is_empty()).then(|| frame_digest(self))
    }
}

/// `frame/1` over a piece set (§3.3):
///
/// ```text
/// SHA-256( "spec-spine/frame/1" 0x00
///          for each piece, sorted by path in byte order:
///            kind (one byte), u64 BE len(path), path, u64 BE len(content), content )
/// ```
///
/// Byte order of the path strings is `BTreeMap<String, _>`'s order, which is
/// `str`'s `Ord`: lexicographic over UTF-8 bytes.
pub fn frame_digest(set: &PieceSet) -> String {
    let mut hasher = Sha256::new();
    hasher.update(FRAME_PREFIX);
    hasher.update([0u8]);
    for (path, (kind, content)) in &set.pieces {
        hasher.update([kind.byte()]);
        hasher.update((path.len() as u64).to_be_bytes());
        hasher.update(path.as_bytes());
        hasher.update((content.len() as u64).to_be_bytes());
        hasher.update(content);
    }
    hex(&hasher.finalize())
}

/// The result of a [`snapshot`] run, mirroring the other two scopes.
#[derive(Clone, Debug)]
pub struct SnapshotOutcome {
    pub snapshot: AuthoritySnapshot,
    /// Canonical JSON of [`SnapshotOutcome::snapshot`].
    pub json: String,
    /// SHA-256 over [`SnapshotOutcome::json`], emitted beside the payload.
    pub attestation_hash: String,
}

/// Build an [`AuthoritySnapshot`] over the tree under `repo_root` (spec 070).
///
/// A record, not a gate: failing verdicts and a stale committed ledger are
/// reported, never refused. An input that cannot be **read** is an error, as
/// everywhere in this tool (§3.2.1).
pub fn snapshot(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
) -> Result<SnapshotOutcome, Error> {
    let compiled = crate::compile::compile(cfg, repo_root)?;
    let indexed = crate::index::index(cfg, repo_root)?;
    let lint_report = crate::lint::lint(cfg, repo_root)?;

    let config = config_member(repo_root)?;
    let committed = SnapshotCommitted {
        registry: committed_registry(cfg, repo_root, &compiled)?,
        index: committed_index(cfg, repo_root, &indexed)?,
    };
    let governance_inputs = governance_inputs(cfg, repo_root)?;
    let verdicts = verdicts(cfg, repo_root, &compiled, &indexed, &lint_report)?;

    let mut specs = Vec::with_capacity(compiled.registry.specs.len());
    for record in &compiled.registry.specs {
        specs.push(spec_entry(
            cfg,
            repo_root,
            &compiled,
            &indexed,
            &lint_report,
            record,
        )?);
    }

    let snapshot = AuthoritySnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION.to_string(),
        tool: ToolStamp {
            name: cfg.branding.compiler_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        digest: FRAME_DIGEST.to_string(),
        config,
        schemas: SnapshotSchemas {
            registry: REGISTRY_SCHEMA_VERSION.to_string(),
            index: INDEX_SCHEMA_VERSION.to_string(),
            corpus_attestation: ATTESTATION_SCHEMA_VERSION.to_string(),
            spec_attestation: SPEC_ATTESTATION_SCHEMA_VERSION.to_string(),
        },
        corpus: SnapshotCorpus {
            specs: compiled.registry.specs.len(),
            inputs_manifest_hash: compiled.registry.build.content_hash.clone(),
            registry_hash: hex(&Sha256::digest(compiled.json.as_bytes())),
        },
        committed,
        governance_inputs,
        verdicts,
        specs,
        exclusions: exclusions(cfg),
    };
    let json = canonical_json::to_string(&snapshot)?;
    let attestation_hash = hex(&Sha256::digest(json.as_bytes()));
    Ok(SnapshotOutcome {
        snapshot,
        json,
        attestation_hash,
    })
}

/// Read a file's bytes for a piece, naming it on failure.
fn read_bytes(path: &Path, what: &str) -> Result<Vec<u8>, Error> {
    fs::read(path).map_err(|e| Error::Io(format!("read {} for {what}: {e}", path.display())))
}

/// `config`: `frame/1` over `spec-spine.toml`, or absent with `present: false`.
fn config_member(repo_root: &Path) -> Result<SnapshotConfig, Error> {
    let path = repo_root.join("spec-spine.toml");
    if !path.is_file() {
        return Ok(SnapshotConfig {
            present: false,
            hash: None,
        });
    }
    let mut set = PieceSet::new();
    set.insert_file_content(
        rel_posix(repo_root, &path),
        read_bytes(&path, "the snapshot's config")?,
    );
    Ok(SnapshotConfig {
        present: true,
        hash: set.digest(),
    })
}

/// Add every `.json` file of one committed shard directory to `set`.
fn add_shard_dir(repo_root: &Path, dir: &Path, set: &mut PieceSet) -> Result<(), Error> {
    for (name, bytes) in shard::read_shard_files(dir)? {
        set.insert_file_content(rel_posix(repo_root, &dir.join(name)), bytes);
    }
    Ok(())
}

/// `committed.registry`: the committed `by-spec/` shards, and whether they are
/// exactly what the corpus compiles to (spec 028 3.1's comparison).
fn committed_registry(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    compiled: &crate::compile::CompileOutcome,
) -> Result<CommittedTree, Error> {
    let by_spec = crate::compile::registry_dir(cfg, repo_root).join(shard::BY_SPEC_DIR);
    if !by_spec.is_dir() {
        return Ok(absent_tree());
    }
    let mut set = PieceSet::new();
    add_shard_dir(repo_root, &by_spec, &mut set)?;
    let matches = matches!(
        crate::compile::compare_committed_registry(cfg, repo_root, &compiled.shards)?,
        crate::index::Freshness::Fresh
    );
    Ok(CommittedTree {
        files: set.len(),
        hash: set.digest(),
        matches_recompute: matches,
    })
}

/// `committed.index`: both shard directories and the slices sidecar when it
/// exists, and whether the shard set is byte-identical to the recompute (spec
/// 069's comparison, without the blocking-diagnostic policy; D-5).
fn committed_index(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    indexed: &crate::index::IndexOutcome,
) -> Result<CommittedTree, Error> {
    let dir = crate::index::index_dir(cfg, repo_root);
    if !dir.is_dir() {
        return Ok(absent_tree());
    }
    let mut set = PieceSet::new();
    add_shard_dir(repo_root, &dir.join(shard::BY_SPEC_DIR), &mut set)?;
    add_shard_dir(repo_root, &dir.join(shard::BY_PACKAGE_DIR), &mut set)?;
    let slices = crate::index::slices_path(cfg, repo_root);
    if slices.is_file() {
        set.insert_file_content(
            rel_posix(repo_root, &slices),
            read_bytes(&slices, "the committed index")?,
        );
    }
    let matches = crate::index::committed_index_drift(cfg, repo_root, &indexed.shards)?.is_empty();
    Ok(CommittedTree {
        files: set.len(),
        hash: set.digest(),
        matches_recompute: matches,
    })
}

fn absent_tree() -> CommittedTree {
    CommittedTree {
        files: 0,
        hash: None,
        matches_recompute: false,
    }
}

/// `governanceInputs`: `spec-spine.toml` and every `[index] extra_hashed_inputs`
/// match outside the state root, as their own content rather than spec 060's
/// projection (D-4).
fn governance_inputs(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
) -> Result<SnapshotGovernanceInputs, Error> {
    let mut set = PieceSet::new();
    let config = repo_root.join("spec-spine.toml");
    if config.is_file() {
        set.insert_file_content(
            rel_posix(repo_root, &config),
            read_bytes(&config, "the snapshot's governance inputs")?,
        );
    }
    for pattern in &cfg.index.extra_hashed_inputs {
        for file in shard::glob_files(repo_root, pattern) {
            let rel = rel_posix(repo_root, &file);
            if cfg.layout.is_state_path(&rel) || set.pieces.contains_key(&rel) {
                continue;
            }
            let bytes = read_bytes(&file, "the snapshot's governance inputs")?;
            set.insert_file_content(rel, bytes);
        }
    }
    Ok(SnapshotGovernanceInputs {
        paths: set.pieces.keys().cloned().collect(),
        hash: set.digest(),
    })
}

/// `verdicts`: the gate's answers as flags and counts, computed over the
/// recompute so a stale committed tree still yields a record.
fn verdicts(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    compiled: &crate::compile::CompileOutcome,
    indexed: &crate::index::IndexOutcome,
    lint_report: &crate::lint::LintReport,
) -> Result<SnapshotVerdicts, Error> {
    let violations = &compiled.registry.validation.violations;
    let count = |severity: Severity| violations.iter().filter(|v| v.severity == severity).count();

    let lint_ok =
        lint_report.count(Severity::Error) == 0 && lint_report.count(Severity::Warning) == 0;
    let findings_hash = hex(&Sha256::digest(
        canonical_json::to_string(&lint_report.violations)?.as_bytes(),
    ));

    let blocking = indexed
        .index
        .diagnostics
        .errors
        .iter()
        .filter(|d| BLOCKING_RESOLVER_CODES.contains(&d.code.as_str()))
        .count();
    let unresolved = indexed
        .index
        .traceability
        .mappings
        .iter()
        .flat_map(|m| &m.resolved_units)
        .filter(|u| u.ownership && u.locations.is_empty())
        .count();

    let files = crate::coverage::enumerate_source_files(cfg, repo_root, &indexed.index);
    let coverage = crate::coverage::coverage_with(cfg, &indexed.index, &files);

    let claims = crate::index::unwitnessed_claims(cfg, repo_root, &indexed.index);
    let total: std::collections::BTreeSet<&str> = claims.iter().map(|c| c.path.as_str()).collect();
    let allowed: std::collections::BTreeSet<&str> = claims
        .iter()
        .filter(|c| c.allowed)
        .map(|c| c.path.as_str())
        .collect();

    Ok(SnapshotVerdicts {
        compile: SnapshotCompileVerdict {
            ok: compiled.validation_passed,
            errors: count(Severity::Error),
            warnings: count(Severity::Warning),
        },
        lint: spec_spine_types::LintVerdict {
            ok: lint_ok,
            findings_hash,
        },
        resolution: SnapshotResolutionVerdict {
            ok: blocking == 0 && unresolved == 0,
            blocking,
            unresolved,
        },
        ownership: SnapshotOwnership {
            source_files: coverage.source_files,
            claimed: coverage.claimed_files,
            floor_only: coverage.floor_only_files.len(),
            unclaimed: coverage.unclaimed_files.len(),
        },
        unwitnessed: SnapshotUnwitnessed {
            total: total.len(),
            allowed: allowed.len(),
        },
    })
}

/// One `specs` entry: lifecycle, the framed territory digest, and the join hash
/// or the stated reason it has no answer (§3.2, §3.2.1).
fn spec_entry(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    compiled: &crate::compile::CompileOutcome,
    indexed: &crate::index::IndexOutcome,
    lint_report: &crate::lint::LintReport,
    record: &spec_spine_types::SpecRecord,
) -> Result<SnapshotSpec, Error> {
    let territory_digest = territory_pieces(cfg, repo_root, indexed, &record.id)?.digest();

    let (spec_attestation_hash, spec_attestation_unavailable) =
        match crate::attest::spec_units(cfg, repo_root, &record.id, indexed) {
            Ok((units, all_resolved)) => {
                let source = crate::attest::spec_source_hash(repo_root, record)?;
                let outcome = crate::attest::build_spec_attestation(
                    cfg,
                    compiled,
                    record,
                    source,
                    units,
                    all_resolved,
                    lint_report,
                )?;
                (Some(outcome.attestation_hash), None)
            }
            Err(SpecUnitsError::NonUtf8DirectClaim(_)) => {
                (None, Some(NON_UTF8_DIRECT_CLAIM.to_string()))
            }
            Err(SpecUnitsError::Other(e)) => return Err(e),
        };

    Ok(SnapshotSpec {
        id: record.id.clone(),
        status: crate::attest::status_label(record.status).to_string(),
        implementation: record
            .implementation
            .map(crate::attest::implementation_label)
            .map(str::to_string),
        territory_digest,
        spec_attestation_hash,
        spec_attestation_unavailable,
    })
}

/// The pieces a spec's owning units resolve to, under §3.3.1's rule.
///
/// Whole files, never spans; one piece per path; a directory location is
/// expanded with `index::walk_territory` (spec 066's walk), and every empty or
/// wholly pruned directory it reaches is a `d` piece at its own path; a symlink
/// is an `l` piece and is never followed; a unit that resolves to nothing
/// contributes nothing.
pub fn territory_pieces(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    indexed: &crate::index::IndexOutcome,
    spec_id: &str,
) -> Result<PieceSet, Error> {
    let mut set = PieceSet::new();
    let Some(mapping) = indexed
        .index
        .traceability
        .mappings
        .iter()
        .find(|m| m.spec_id == spec_id)
    else {
        return Ok(set);
    };
    let what = format!("spec '{spec_id}' territory");
    for resolved in mapping.resolved_units.iter().filter(|u| u.ownership) {
        for loc in &resolved.locations {
            let path = repo_root.join(&loc.file);
            let rel = rel_posix(repo_root, &path);
            if set.pieces.contains_key(&rel) {
                continue;
            }
            let meta = fs::symlink_metadata(&path)
                .map_err(|e| Error::Io(format!("read {} for {what}: {e}", path.display())))?;
            if meta.file_type().is_symlink() {
                set.insert(rel, PieceKind::Link, link_target(&path));
            } else if meta.is_dir() {
                let entries = crate::index::walk_territory(
                    &path,
                    repo_root,
                    &cfg.index.resolver_exclusions,
                    &cfg.layout,
                );
                // Spec 070 §3.3.1, D-9: every empty or wholly pruned directory
                // the walk reaches is a `d` piece at its own path, the claimed
                // directory included, so an emptied directory and a truncated
                // file at one path never frame alike.
                for empty in crate::index::empty_territory_dirs(
                    &path,
                    repo_root,
                    &cfg.index.resolver_exclusions,
                    &cfg.layout,
                ) {
                    set.insert(
                        rel_posix(repo_root, &empty),
                        PieceKind::EmptyDir,
                        Vec::new(),
                    );
                }
                for entry in entries {
                    match entry {
                        TerritoryEntry::File(file) => {
                            let file_rel = rel_posix(repo_root, &file);
                            if !set.pieces.contains_key(&file_rel) {
                                set.insert_file_content(file_rel, read_bytes(&file, &what)?);
                            }
                        }
                        TerritoryEntry::Symlink { path, target } => {
                            set.insert(
                                rel_posix(repo_root, &path),
                                PieceKind::Link,
                                target.into_bytes(),
                            );
                        }
                    }
                }
            } else {
                set.insert_file_content(rel, read_bytes(&path, &what)?);
            }
        }
    }
    Ok(set)
}

/// A symlink's target text as stored, empty when it cannot be read (the rule
/// `walk_territory` applies).
fn link_target(path: &Path) -> Vec<u8> {
    fs::read_link(path)
        .map(|t| t.to_string_lossy().into_owned().into_bytes())
        .unwrap_or_default()
}

/// `exclusions`: what was deliberately not read or not held to account.
fn exclusions(cfg: &spec_spine_types::Config) -> SnapshotExclusions {
    let state_dir = cfg.layout.state_dir.trim();
    SnapshotExclusions {
        resolver_exclusions: cfg.index.resolver_exclusions.clone(),
        state_dir: (!state_dir.is_empty()).then(|| cfg.layout.state_dir.clone()),
        unwitnessed_allowed: cfg.lint.unwitnessed_allowed.clone(),
        bypass_prefixes: crate::couple::effective_bypass_prefixes(cfg)
            .into_iter()
            .map(|e| e.prefix)
            .collect(),
    }
}

/// The hash a seal signs and a consumer references, for a snapshot.
pub fn snapshot_hash(snapshot: &AuthoritySnapshot) -> Result<String, Error> {
    Ok(hex(&Sha256::digest(
        canonical_json::to_string(snapshot)?.as_bytes(),
    )))
}

/// Refuse a snapshot whose schema MAJOR this build does not understand (spec
/// 068 3.3).
pub fn check_snapshot_major(schema_version: &str) -> Result<(), Error> {
    shard::check_major("snapshot", schema_version, SNAPSHOT_SCHEMA_VERSION)
}

/// Verify a snapshot by recompute (§3.5), under spec 068's rules: a
/// `tool.version` mismatch is its own outcome, and a content mismatch names
/// every member that moved.
pub fn verify_snapshot_recompute(
    cfg: &spec_spine_types::Config,
    repo_root: &Path,
    attested: &AuthoritySnapshot,
) -> Result<VerifyOutcome, Error> {
    let actual_version = env!("CARGO_PKG_VERSION");
    if attested.tool.version != actual_version {
        return Ok(VerifyOutcome::VersionMismatch {
            expected: attested.tool.version.clone(),
            actual: actual_version.to_string(),
        });
    }
    let recomputed = snapshot(cfg, repo_root)?.snapshot;
    if recomputed == *attested {
        return Ok(VerifyOutcome::Match);
    }
    let (a, b) = (attested, &recomputed);
    let mut differences = Vec::new();
    let mut named = |name: &str, moved: bool| {
        if moved {
            differences.push(name.to_string());
        }
    };
    named("schemaVersion", a.schema_version != b.schema_version);
    named("tool.name", a.tool.name != b.tool.name);
    named("digest", a.digest != b.digest);
    named("config", a.config != b.config);
    named("schemas", a.schemas != b.schemas);
    named("corpus.specs", a.corpus.specs != b.corpus.specs);
    named(
        "corpus.inputsManifestHash",
        a.corpus.inputs_manifest_hash != b.corpus.inputs_manifest_hash,
    );
    named(
        "corpus.registryHash",
        a.corpus.registry_hash != b.corpus.registry_hash,
    );
    named(
        "committed.registry",
        a.committed.registry != b.committed.registry,
    );
    named("committed.index", a.committed.index != b.committed.index);
    named(
        "governanceInputs",
        a.governance_inputs != b.governance_inputs,
    );
    named("verdicts.compile", a.verdicts.compile != b.verdicts.compile);
    named("verdicts.lint", a.verdicts.lint != b.verdicts.lint);
    named(
        "verdicts.resolution",
        a.verdicts.resolution != b.verdicts.resolution,
    );
    named(
        "verdicts.ownership",
        a.verdicts.ownership != b.verdicts.ownership,
    );
    named(
        "verdicts.unwitnessed",
        a.verdicts.unwitnessed != b.verdicts.unwitnessed,
    );
    named("exclusions", a.exclusions != b.exclusions);
    // Matched by id, not by position, so one inserted spec does not report
    // every later entry as changed.
    for before in &a.specs {
        match b.specs.iter().find(|after| after.id == before.id) {
            Some(after) if after != before => {
                differences.push(format!("specs[{}]", before.id));
            }
            Some(_) => {}
            None => differences.push(format!("specs[{}] is gone", before.id)),
        }
    }
    for after in &b.specs {
        if !a.specs.iter().any(|before| before.id == after.id) {
            differences.push(format!("specs[{}] is new", after.id));
        }
    }
    if differences.is_empty() {
        // Every member above compared equal, so what moved is either the order
        // of `specs` or a member added to `AuthoritySnapshot` without a line
        // here. Say both rather than guess one.
        differences.push("specs order, or a member this build does not name".to_string());
    }
    Ok(VerifyOutcome::ContentMismatch { differences })
}

/// Fold spec 068 3.1's byte comparison into a snapshot recompute outcome.
pub fn with_stored_bytes_snapshot(
    outcome: VerifyOutcome,
    attested: &AuthoritySnapshot,
    stored: &[u8],
) -> Result<VerifyOutcome, Error> {
    match outcome {
        VerifyOutcome::Match => {
            if stored == canonical_json::to_string(attested)?.as_bytes() {
                Ok(VerifyOutcome::Match)
            } else {
                Ok(VerifyOutcome::ContentMismatch {
                    differences: vec![crate::attest::NON_CANONICAL_BYTES.to_string()],
                })
            }
        }
        other => Ok(other),
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}
