//! `spec-spine attest`: emit a reproducible corpus attestation under
//! `<derived_dir>/attestation/`, and optionally seal it (spec 023).
//!
//! The attestation itself is pure (built in `spec-spine-core::attest`); this
//! command is the IO + clock shell: it writes the artifact and, under `--sign`,
//! the wall-clock-dated detached seal.

use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::{AttestOptions, attest, attest_spec};
use spec_spine_types::{Error, Verdict, verdict::verb};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::load_repo_config;
use crate::out;
use crate::seal;

/// Parsed `attest` arguments.
pub struct AttestArgs {
    /// Scope to one spec (spec 042); `None` attests the whole corpus (spec 023).
    pub spec: Option<String>,
    pub with_coupling: bool,
    pub sign: bool,
    pub key: Option<PathBuf>,
    pub key_id: Option<String>,
    /// Emit the verdict as a JSON envelope instead of prose (spec 037).
    pub json: bool,
}

/// Writes `attestation.json` (always) and `attestation.sig` (under `--sign`).
/// Exit `0` on success; a `--sign` with no `--key` is a visible config error
/// (FR-006: a mode that cannot run fails, never skip-as-pass).
pub fn run(repo: &Path, args: &AttestArgs) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;

    // FR-006 (fail-closed, no side effects on a usage error): when signing,
    // resolve the key BEFORE building or writing the attestation, so a missing
    // or invalid key fails before any artifact lands on disk.
    let signer = if args.sign {
        let key_path = args.key.as_ref().ok_or_else(|| {
            Error::Config(
                "attest --sign requires --key <path> (a 32-byte ed25519 signing key, raw or hex)"
                    .to_string(),
            )
        })?;
        let signing_key = seal::load_signing_key(key_path)?;
        let key_id = args
            .key_id
            .clone()
            .unwrap_or_else(|| seal::default_key_id(&signing_key));
        Some((signing_key, key_id))
    } else {
        None
    };

    // One payload, one hash, whichever scope: the two verbs differ only in what
    // they cover, so the seal, the write and the reporting below are shared.
    let (json, attestation_hash, payload) = match &args.spec {
        Some(id) => {
            let outcome = attest_spec(&cfg, repo, id)?;
            let payload = serde_json::json!({
                "attestation": outcome.attestation,
                "attestationHash": outcome.attestation_hash,
            });
            (outcome.json, outcome.attestation_hash, payload)
        }
        None => {
            let outcome = attest(
                &cfg,
                repo,
                AttestOptions {
                    with_coupling: args.with_coupling,
                },
            )?;
            let payload = serde_json::json!({
                "attestation": outcome.attestation,
                "attestationHash": outcome.attestation_hash,
            });
            (outcome.json, outcome.attestation_hash, payload)
        }
    };

    let out_dir = repo.join(&cfg.layout.derived_dir).join("attestation");
    let attestation_path = match &args.spec {
        Some(id) => out_dir.join("by-spec").join(format!("{id}.json")),
        None => out_dir.join("attestation.json"),
    };
    let parent = attestation_path.parent().unwrap_or(&out_dir);
    fs::create_dir_all(parent)
        .map_err(|e| Error::Io(format!("create {}: {e}", parent.display())))?;
    fs::write(&attestation_path, &json)
        .map_err(|e| Error::Io(format!("write {}: {e}", attestation_path.display())))?;

    let scope = match (&args.spec, args.with_coupling) {
        (Some(id), _) => id.as_str(),
        (None, true) => "specs+code",
        (None, false) => "spec-corpus",
    };
    if !args.json {
        outln!("attested {scope} -> {}", attestation_path.display());
        outln!("  attestationHash: {attestation_hash}");
    }

    if let Some((signing_key, key_id)) = signer {
        let ledger_seal = seal::sign(&attestation_hash, &signing_key, key_id, now_rfc3339())?;
        let seal_json = serde_json::to_string_pretty(&ledger_seal)
            .map_err(|e| Error::Schema(e.to_string()))?
            + "\n";
        let seal_path = attestation_path.with_extension("sig");
        fs::write(&seal_path, seal_json)
            .map_err(|e| Error::Io(format!("write {}: {e}", seal_path.display())))?;
        if !args.json {
            outln!(
                "sealed -> {} (alg ed25519, keyId {})",
                seal_path.display(),
                ledger_seal.key_id
            );
        }
    }

    if args.json {
        // `{ attestation, attestationHash }`, the shape the matching facade
        // returns for whichever scope ran. The seal is deliberately absent:
        // signing is a CLI post-pass over the attestation hash, the facade does
        // not model it, and spec 037 3.1 requires one payload shape per verb
        // rather than a CLI spelling that diverges from the library's. A
        // consumer that needs the seal reads the sibling `.sig`, whose path is
        // a function of the attestation's.
        out::verdict(&Verdict::report(verb::ATTEST, 0, payload))?;
    }

    Ok(0)
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}
