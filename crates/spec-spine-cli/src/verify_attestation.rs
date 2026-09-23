//! `spec-spine verify-attestation`: two independent verification modes (spec 021
//! FR-004), either or both selectable in one invocation.
//!
//! `--recompute` re-reads the corpus and checks it reproduces the attestation:
//! no key, no signature, offline, runnable by any third party. It is the load
//! bearing property the run certificate structurally cannot have. `--signature`
//! checks the detached seal against a supplied public key. A mode that cannot
//! run fails visibly (FR-006); skip-as-pass is forbidden.
//!
//! Both modes decide on the **bytes of the file that was handed in** (spec 068
//! 3.1). The file is read once; the seal is checked over SHA-256 of those bytes
//! and `--recompute` compares them against the canonical serialization of what
//! they parsed to. For every file `attest` wrote the two are the same bytes, so
//! every existing seal verifies as before; for a file someone else produced they
//! are not, which is precisely the case a verifier exists to catch.

use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::{
    VerifyOutcome, check_attestation_major, check_snapshot_major, check_spec_attestation_major,
    payload_schema_version, stored_bytes_hash, verify_recompute, verify_snapshot_recompute,
    verify_spec_recompute, with_stored_bytes, with_stored_bytes_snapshot, with_stored_bytes_spec,
};
use spec_spine_types::{
    AuthoritySnapshot, Config, CorpusAttestation, Error, LedgerSeal, SpecAttestation, Verdict,
    verdict::verb,
};

use crate::load_repo_config;
use crate::out;
use crate::seal;

/// Parsed `verify-attestation` arguments.
pub struct VerifyArgs {
    /// Verify the per-spec attestation for this id (spec 039); `None` verifies
    /// the corpus-scoped one (spec 021).
    pub spec: Option<String>,
    /// Verify the authority snapshot (spec 070) rather than an attestation.
    pub snapshot: bool,
    pub recompute: bool,
    pub signature: bool,
    pub attestation: Option<PathBuf>,
    pub public_key: Option<PathBuf>,
    pub seal: Option<PathBuf>,
    /// Emit the verdict as a JSON envelope instead of prose (spec 034).
    pub json: bool,
}

/// Exit `0` only if every selected mode passes; `1` on any mismatch or version
/// mismatch (a named, non-pass outcome). A missing mode or missing key is a
/// visible config error (exit 3), never a silent pass.
pub fn run(repo: &Path, args: &VerifyArgs) -> Result<u8, Error> {
    if !args.recompute && !args.signature {
        return Err(Error::Config(
            "verify-attestation requires at least one mode: --recompute and/or --signature"
                .to_string(),
        ));
    }

    if args.snapshot && args.spec.is_some() {
        return Err(Error::Config(
            "verify-attestation --snapshot cannot combine with --spec: each names a different \
             record (spec 070 3.5)"
                .to_string(),
        ));
    }
    let cfg = load_repo_config(repo)?;
    if let Some(id) = &args.spec {
        validate_spec_id(id)?;
    }
    // Spec 067 3.2: the short form resolves here too, against the set this verb
    // already reads, which is the attestation files rather than the corpus. A
    // `--signature` check is legitimate on an attestation whose spec has since
    // left the corpus, and a corpus-wide set would refuse it (067 D-3).
    //
    // Resolution runs **before** any read, so an ambiguous argument is refused
    // without the file being opened, and it happens with `--attestation` too:
    // that form locates nothing by id, so the resolved value is unused there,
    // but refusing an ambiguous id consistently costs nothing and keeps the one
    // message of 3.1 true at all six arguments.
    let spec = match &args.spec {
        Some(id) => Some(resolve_attested_spec(repo, &cfg, id)?),
        None => None,
    };
    let attestation_path = args.attestation.clone().unwrap_or_else(|| {
        if args.snapshot {
            repo.join(&cfg.layout.derived_dir)
                .join("attestation")
                .join("snapshot.json")
        } else {
            default_attestation_path(repo, &cfg, spec.as_deref())
        }
    });

    // Read once, and hold the bytes: they are what both modes decide on (3.1).
    let hint = match &spec {
        Some(id) => format!("spec-spine attest --spec {id}"),
        None if args.snapshot => "spec-spine attest --snapshot".to_string(),
        None => "spec-spine attest".to_string(),
    };
    let bytes = read_artifact(&attestation_path, "attestation", &hint)?;

    // The MAJOR gate runs before the strict parse, so a payload from a schema
    // line this build does not understand is refused in the loaders' own words
    // rather than as whichever unknown member the parse happened to reach first
    // (3.3). The two scopes carry different payloads, so the loaded value and
    // both verification paths fork here and nowhere else.
    let version = payload_schema_version(&bytes, "attestation")?;
    let subject = match &spec {
        None if args.snapshot => {
            check_snapshot_major(&version)?;
            Subject::Snapshot(Box::new(parse_artifact(
                &bytes,
                &attestation_path,
                "snapshot",
            )?))
        }
        Some(_) => {
            check_spec_attestation_major(&version)?;
            Subject::Spec(parse_artifact(&bytes, &attestation_path, "attestation")?)
        }
        None => {
            check_attestation_major(&version)?;
            Subject::Corpus(parse_artifact(&bytes, &attestation_path, "attestation")?)
        }
    };

    let mut failed = false;
    // Under --json the two modes accumulate into one report object rather than
    // printing as they go. `outcome` is present exactly when --recompute ran and
    // is byte-for-byte what `spec_spine_core::verify_attestation_json` returns
    // for the same inputs (spec 034 3.1); `signature` is present exactly when
    // --signature ran, an additive member for the mode the facade does not model.
    // Both are needed because spec 034 3.2 requires the envelope to report the
    // same verdict the prose reports, and the prose reports both.
    let mut report = serde_json::Map::new();

    if args.recompute {
        let outcome = match &subject {
            Subject::Corpus(a) => with_stored_bytes(verify_recompute(&cfg, repo, a)?, a, &bytes)?,
            Subject::Spec(a) => {
                with_stored_bytes_spec(verify_spec_recompute(&cfg, repo, a)?, a, &bytes)?
            }
            Subject::Snapshot(a) => {
                with_stored_bytes_snapshot(verify_snapshot_recompute(&cfg, repo, a)?, a, &bytes)?
            }
        };
        match outcome {
            VerifyOutcome::Match => {
                if args.json {
                    report.insert("outcome".to_string(), serde_json::json!("match"));
                } else {
                    outln!("recompute: MATCH (the corpus reproduces this attestation)");
                }
            }
            VerifyOutcome::VersionMismatch { expected, actual } => {
                if args.json {
                    report.insert("outcome".to_string(), serde_json::json!("versionMismatch"));
                    report.insert("expected".to_string(), serde_json::json!(expected));
                    report.insert("actual".to_string(), serde_json::json!(actual));
                } else {
                    eprintln!(
                        "recompute: VERSION MISMATCH (attested under {expected}, this tool is {actual}); \
                         recompute under {expected} to verify"
                    );
                }
                failed = true;
            }
            VerifyOutcome::ContentMismatch { differences } => {
                if args.json {
                    report.insert("outcome".to_string(), serde_json::json!("contentMismatch"));
                    report.insert("differences".to_string(), serde_json::json!(differences));
                } else {
                    eprintln!(
                        "recompute: CONTENT MISMATCH ({} field(s) diverged):",
                        differences.len()
                    );
                    for d in &differences {
                        eprintln!("  - {d}");
                    }
                }
                failed = true;
            }
        }
    }

    if args.signature {
        let pk_path = args.public_key.as_ref().ok_or_else(|| {
            Error::Config(
                "verify-attestation --signature requires --public-key <path> (a 32-byte ed25519 public key)"
                    .to_string(),
            )
        })?;
        let verifying_key = seal::load_verifying_key(pk_path)?;
        let seal_path = args
            .seal
            .clone()
            .unwrap_or_else(|| attestation_path.with_extension("sig"));
        let seal_hint = match &spec {
            Some(id) => format!("spec-spine attest --spec {id} --sign"),
            None if args.snapshot => "spec-spine attest --snapshot --sign".to_string(),
            None => "spec-spine attest --sign".to_string(),
        };
        let ledger_seal: LedgerSeal = load_json(&seal_path, "seal", &seal_hint)?;
        // The hash is over the bytes that were read, never over a
        // re-serialization of their parse (3.1). Hashing the parse made the
        // seal a statement about whatever this build's DTOs happened to carry
        // across, so anything the parse dropped or reshaped was signed for
        // without ever being seen.
        let hash = stored_bytes_hash(&bytes);
        let valid = seal::verify(&hash, &ledger_seal, &verifying_key)?;
        if args.json {
            report.insert(
                "signature".to_string(),
                serde_json::json!({ "valid": valid, "keyId": ledger_seal.key_id }),
            );
        } else if valid {
            outln!("signature: VALID (sealed by keyId {})", ledger_seal.key_id);
        } else {
            eprintln!(
                "signature: INVALID (the seal does not verify against the supplied public key)"
            );
        }
        if !valid {
            failed = true;
        }
    }

    let code = if failed { 1 } else { 0 };
    if args.json {
        out::verdict(&Verdict::report(
            verb::VERIFY_ATTESTATION,
            code,
            serde_json::Value::Object(report),
        ))?;
    }
    Ok(code)
}

/// Which payload is under verification. The two scopes share every mode and
/// every exit code; only the deserialized type and the recompute call differ.
enum Subject {
    Corpus(CorpusAttestation),
    Spec(SpecAttestation),
    /// Boxed: a snapshot is much larger than either attestation.
    Snapshot(Box<AuthoritySnapshot>),
}

fn default_attestation_path(repo: &Path, cfg: &Config, spec: Option<&str>) -> PathBuf {
    let dir = repo.join(&cfg.layout.derived_dir).join("attestation");
    match spec {
        Some(id) => dir.join("by-spec").join(format!("{id}.json")),
        None => dir.join("attestation.json"),
    }
}

/// Resolve `--spec` against the per-spec attestation files (spec 067 3.2).
///
/// Step 4 of 067 3.1 does **not** refuse here. The argument falls through as
/// given and the read fails exactly as it does today: exit 3, with the hint to
/// run `attest --spec` first. A missing attestation file is I/O, which spec 039
/// 3.5 assigns to exit 3, and refusing at exit 1 to match the other five would
/// change that verb's code for a missing file (067 D-4). Only an argument that
/// resolves is newly accepted, and only an ambiguous one is newly refused.
///
/// A `by-spec/` that does not exist is an empty set, not an error: that is the
/// state before the first `attest --spec`, and it must reach the same exit 3.
fn resolve_attested_spec(repo: &Path, cfg: &Config, id: &str) -> Result<String, Error> {
    let dir = repo
        .join(&cfg.layout.derived_dir)
        .join("attestation")
        .join("by-spec");
    let mut stems: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(stem) = name.strip_suffix(".json") {
                stems.push(stem.to_string());
            }
        }
    }
    match spec_spine_core::match_spec_id(id, &stems) {
        spec_spine_core::SpecIdMatch::Resolved(r) => Ok(r),
        spec_spine_core::SpecIdMatch::Ambiguous(c) => {
            Err(spec_spine_core::spec_id::ambiguous(id, &c))
        }
        spec_spine_core::SpecIdMatch::NoMatch => Ok(id.to_string()),
    }
}

/// A spec id is one path segment, so interpolating it into a filename cannot
/// walk out of the attestation directory.
///
/// `attest --spec` is guarded where it writes (spec 126 3.3): its registry
/// lookup refuses an unknown id, but the corpus chooses which ids are known, so
/// the lookup alone never protected it. This side reads, and reads before any
/// lookup, so it is guarded here instead, with its own narrower rule. The
/// impact is confusion rather than exposure, since `--attestation` already lets
/// the caller name any path they can read, but a traversing id would fail with
/// a puzzling parse error on some unrelated file rather than saying what was
/// wrong.
fn validate_spec_id(id: &str) -> Result<(), Error> {
    let bad = id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id == "."
        || id == ".."
        || id.contains('\0');
    if bad {
        return Err(Error::Config(format!(
            "verify-attestation --spec '{id}' is not a spec id: an id is one path segment, \
             and pointing at another file is what --attestation is for"
        )));
    }
    Ok(())
}

/// Read and deserialize a JSON artifact, naming it in both failure messages.
///
/// `hint` is the command that would have produced the file. It is a parameter
/// rather than a constant because a missing seal and a missing attestation want
/// different advice: telling someone who forgot `--sign` to run `attest` sends
/// them to re-run the step that already succeeded.
fn load_json<T: serde::de::DeserializeOwned>(
    path: &Path,
    what: &str,
    hint: &str,
) -> Result<T, Error> {
    let bytes = read_artifact(path, what, hint)?;
    parse_artifact(&bytes, path, what)
}

/// Read an artifact's bytes, naming it and the command that would have produced
/// it. Separate from the parse because the attestation's bytes outlive the
/// parse: they are what both verification modes decide on (spec 068 3.1).
fn read_artifact(path: &Path, what: &str, hint: &str) -> Result<Vec<u8>, Error> {
    fs::read(path).map_err(|e| {
        Error::Io(format!(
            "read {what} {} (run `{hint}` first?): {e}",
            path.display()
        ))
    })
}

/// Deserialize bytes already read, naming the file in the failure. The DTOs
/// refuse unknown members (spec 068 3.2), so this is also where a payload
/// carrying a claim this build cannot evaluate is turned away.
fn parse_artifact<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    path: &Path,
    what: &str,
) -> Result<T, Error> {
    serde_json::from_slice(bytes)
        .map_err(|e| Error::Parse(format!("invalid {what} {}: {e}", path.display())))
}
