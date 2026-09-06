//! `spec-spine verify-attestation`: two independent verification modes (spec 023
//! FR-004), either or both selectable in one invocation.
//!
//! `--recompute` re-reads the corpus and checks it reproduces the attestation:
//! no key, no signature, offline, runnable by any third party. It is the load
//! bearing property the run certificate structurally cannot have. `--signature`
//! checks the detached seal against a supplied public key. A mode that cannot
//! run fails visibly (FR-006); skip-as-pass is forbidden.

use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::{
    VerifyOutcome, attestation_hash, spec_attestation_hash, verify_recompute, verify_spec_recompute,
};
use spec_spine_types::{
    Config, CorpusAttestation, Error, LedgerSeal, SpecAttestation, Verdict, verdict::verb,
};

use crate::load_repo_config;
use crate::out;
use crate::seal;

/// Parsed `verify-attestation` arguments.
pub struct VerifyArgs {
    /// Verify the per-spec attestation for this id (spec 042); `None` verifies
    /// the corpus-scoped one (spec 023).
    pub spec: Option<String>,
    pub recompute: bool,
    pub signature: bool,
    pub attestation: Option<PathBuf>,
    pub public_key: Option<PathBuf>,
    pub seal: Option<PathBuf>,
    /// Emit the verdict as a JSON envelope instead of prose (spec 037).
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

    let cfg = load_repo_config(repo)?;
    if let Some(id) = &args.spec {
        validate_spec_id(id)?;
    }
    let attestation_path = args
        .attestation
        .clone()
        .unwrap_or_else(|| default_attestation_path(repo, &cfg, args.spec.as_deref()));

    // The two scopes carry different payloads, so the loaded value and both
    // verification paths fork here and nowhere else.
    let subject = match &args.spec {
        Some(id) => Subject::Spec(load_json(
            &attestation_path,
            "attestation",
            &format!("spec-spine attest --spec {id}"),
        )?),
        None => Subject::Corpus(load_json(
            &attestation_path,
            "attestation",
            "spec-spine attest",
        )?),
    };

    let mut failed = false;
    // Under --json the two modes accumulate into one report object rather than
    // printing as they go. `outcome` is present exactly when --recompute ran and
    // is byte-for-byte what `spec_spine_core::verify_attestation_json` returns
    // for the same inputs (spec 037 3.1); `signature` is present exactly when
    // --signature ran, an additive member for the mode the facade does not model.
    // Both are needed because spec 037 3.2 requires the envelope to report the
    // same verdict the prose reports, and the prose reports both.
    let mut report = serde_json::Map::new();

    if args.recompute {
        let outcome = match &subject {
            Subject::Corpus(a) => verify_recompute(&cfg, repo, a)?,
            Subject::Spec(a) => verify_spec_recompute(&cfg, repo, a)?,
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
        let seal_hint = match &args.spec {
            Some(id) => format!("spec-spine attest --spec {id} --sign"),
            None => "spec-spine attest --sign".to_string(),
        };
        let ledger_seal: LedgerSeal = load_json(&seal_path, "seal", &seal_hint)?;
        // Recompute the hash from the loaded payload: a tampered byte changes it
        // and the seal stops verifying.
        let hash = match &subject {
            Subject::Corpus(a) => attestation_hash(a)?,
            Subject::Spec(a) => spec_attestation_hash(a)?,
        };
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
}

fn default_attestation_path(repo: &Path, cfg: &Config, spec: Option<&str>) -> PathBuf {
    let dir = repo.join(&cfg.layout.derived_dir).join("attestation");
    match spec {
        Some(id) => dir.join("by-spec").join(format!("{id}.json")),
        None => dir.join("attestation.json"),
    }
}

/// A spec id is one path segment, so interpolating it into a filename cannot
/// walk out of the attestation directory.
///
/// `attest --spec` is already protected by its registry lookup, which refuses an
/// unknown id before anything is written. This side reads, and reads before any
/// lookup, so it is guarded here instead. The impact is confusion rather than
/// exposure, since `--attestation` already lets the caller name any path they
/// can read, but a traversing id would fail with a puzzling parse error on some
/// unrelated file rather than saying what was wrong.
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
    let bytes = fs::read(path).map_err(|e| {
        Error::Io(format!(
            "read {what} {} (run `{hint}` first?): {e}",
            path.display()
        ))
    })?;
    serde_json::from_slice(&bytes)
        .map_err(|e| Error::Parse(format!("invalid {what} {}: {e}", path.display())))
}
