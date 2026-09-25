//! `spec-spine attest`: emit a reproducible corpus attestation under
//! `<derived_dir>/attestation/`, and optionally seal it (spec 021).
//!
//! The attestation itself is pure (built in `spec-spine-core::attest`); this
//! command is the IO + clock shell: it writes the artifact and, under `--sign`,
//! the wall-clock-dated detached seal.

use std::path::{Path, PathBuf};

use spec_spine_core::{AttestOptions, attest, attest_spec, shard, snapshot};
use spec_spine_types::{Error, Verdict, verdict::verb};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::load_repo_config;
use crate::out;
use crate::seal;

/// Parsed `attest` arguments.
pub struct AttestArgs {
    /// Scope to one spec (spec 039); `None` attests the whole corpus (spec 021).
    pub spec: Option<String>,
    pub with_coupling: bool,
    /// Emit an authority snapshot (spec 070) instead of an attestation.
    pub snapshot: bool,
    pub sign: bool,
    pub key: Option<PathBuf>,
    pub key_id: Option<String>,
    /// Emit the verdict as a JSON envelope instead of prose (spec 034).
    pub json: bool,
}

/// Writes `attestation.json` (always) and `attestation.sig` (under `--sign`).
/// Exit `0` on success; a `--sign` with no `--key` is a visible config error
/// (FR-006: a mode that cannot run fails, never skip-as-pass).
pub fn run(repo: &Path, args: &AttestArgs) -> Result<u8, Error> {
    // Spec 070 §3.5: each flag names a different scope, so a snapshot combined
    // with either is refused rather than one silently winning.
    if args.snapshot {
        for (given, flag) in [
            (args.spec.is_some(), "--spec"),
            (args.with_coupling, "--with-coupling"),
        ] {
            if given {
                return Err(Error::Usage(format!(
                    "attest --snapshot cannot combine with {flag}: each names a different \
                     scope (spec 070 3.5); a snapshot already records every spec's territory \
                     and the resolution verdict"
                )));
            }
        }
    }
    // Spec 039 3.1: coupling is a property of a diff between two revisions, not
    // of a spec at one revision, so a per-spec attestation carries no couple
    // verdict and `attest_spec` takes no such option. Accepting the flag and
    // doing nothing would hand back an exit 0 and a payload missing the verdict
    // the caller asked for, with nothing said. A mode that cannot run fails
    // visibly (spec 021 FR-006).
    if args.spec.is_some() && args.with_coupling {
        return Err(Error::Usage(
            "attest --with-coupling is corpus-scoped and cannot combine with --spec: \
             coupling is a property of a diff between two revisions, not of a spec at one \
             (spec 039 3.1); run `spec-spine attest --with-coupling` for that verdict"
                .to_string(),
        ));
    }
    let cfg = load_repo_config(repo)?;

    // FR-006 (fail-closed, no side effects on a usage error): when signing,
    // resolve the key BEFORE building or writing the attestation, so a missing
    // or invalid key fails before any artifact lands on disk.
    let signer = if args.sign {
        let key_path = args.key.as_ref().ok_or_else(|| {
            Error::Usage(
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
    // Spec 067 3.3: the attestation file is named by the **resolved** id, which
    // the library hands back on the payload. Naming it after the argument would
    // write `by-spec/070.json` beside `by-spec/070-slug.json`: two files for one
    // spec, and `verify-attestation --spec <full id>` reads only one of them.
    let mut resolved_spec: Option<String> = None;
    let (json, attestation_hash, payload) = match &args.spec {
        None if args.snapshot => {
            let outcome = snapshot(&cfg, repo)?;
            let payload = serde_json::json!({
                "attestation": outcome.snapshot,
                "attestationHash": outcome.attestation_hash,
            });
            (outcome.json, outcome.attestation_hash, payload)
        }
        Some(id) => {
            let outcome = attest_spec(&cfg, repo, id)?;
            resolved_spec = Some(outcome.attestation.spec_id.clone());
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
    let (dir, name) = match &resolved_spec {
        // Spec 126 3.3: the resolved id is the corpus's choice, and a known id
        // is not a safe one, so its file name is checked (by the writer's
        // preflight, below) before the directory is created or the attestation
        // or its seal is written.
        Some(id) => (out_dir.join("by-spec"), format!("{id}.json")),
        None if args.snapshot => (out_dir.clone(), "snapshot.json".to_string()),
        None => (out_dir.clone(), "attestation.json".to_string()),
    };
    let attestation_path = dir.join(&name);

    // Spec 127 3.3: the seal is made before anything is written, so the
    // attestation and its seal are checked as one run, and a seal path that
    // refuses leaves the attestation unwritten too.
    let seal = match signer {
        Some((signing_key, key_id)) => {
            let ledger_seal = seal::sign(&attestation_hash, &signing_key, key_id, now_rfc3339())?;
            let seal_json = serde_json::to_string_pretty(&ledger_seal)
                .map_err(|e| Error::Internal(e.to_string()))?
                + "\n";
            let seal_path = attestation_path.with_extension("sig");
            Some((ledger_seal.key_id, seal_path, seal_json))
        }
        None => None,
    };
    let mut run = shard::DerivedWrites::new(repo).write(&dir, &name, json);
    if let Some((_, seal_path, seal_json)) = &seal {
        let seal_name = seal_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        run = run.write(&dir, seal_name, seal_json.clone());
    }
    run.apply()?;

    // Spec 067 3.3 again: the echoed scope is a value derived from the id, so
    // it is the resolved one. `attested 016 -> .../016-short.json` names two
    // different spellings of one spec on one line.
    let scope = match (&resolved_spec, args.with_coupling) {
        (Some(id), _) => id.as_str(),
        (None, _) if args.snapshot => "snapshot",
        (None, true) => "specs+code",
        (None, false) => "spec-corpus",
    };
    if !args.json {
        outln!("attested {scope} -> {}", attestation_path.display());
        outln!("  attestationHash: {attestation_hash}");
        if let Some((key_id, seal_path, _)) = &seal {
            outln!(
                "sealed -> {} (alg ed25519, keyId {key_id})",
                seal_path.display()
            );
        }
    }

    if args.json {
        // `{ attestation, attestationHash }`, the shape the matching facade
        // returns for whichever scope ran. The seal is deliberately absent:
        // signing is a CLI post-pass over the attestation hash, the facade does
        // not model it, and spec 034 3.1 requires one payload shape per verb
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
