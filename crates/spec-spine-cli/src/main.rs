//! `spec-spine`: the multi-call CLI. A thin wrapper over `spec-spine-core`:
//! it parses args, loads config, calls the engine, prints results, and maps the
//! typed `Error` to a stable process exit code. All `process::exit`, stdout, and
//! `git`/clock side effects live here, never in the library.

/// `println!` for stdout that does not panic when the reader goes away.
///
/// Defined before the `mod` items below so every submodule sees it (textual
/// macro scoping). See `out.rs` for why this exists (spec 035).
macro_rules! outln {
    () => { $crate::out::line(format_args!("")) };
    ($($arg:tt)*) => { $crate::out::line(format_args!($($arg)*)) };
}

/// `print!` for stdout that does not panic when the reader goes away. For
/// pre-formatted blocks that already carry their own trailing newline.
macro_rules! out {
    ($($arg:tt)*) => { $crate::out::block(format_args!($($arg)*)) };
}

mod cmd_attest;
mod cmd_compile;
mod cmd_config;
mod cmd_couple;
mod cmd_index;
mod cmd_init;
mod cmd_lint;
mod cmd_registry;
mod cmd_verify;
mod out;
mod seal;
mod verify_attestation;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use spec_spine_types::{Config, Error, Verdict, verdict::verb};

#[derive(Parser)]
#[command(
    name = "spec-spine",
    version,
    about = "A typed, hash-verifiable authority ledger over a markdown spec corpus."
)]
struct Cli {
    /// Repository root (defaults to the current directory).
    #[arg(long, global = true, value_name = "DIR")]
    repo: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile specs/*/spec.md into a deterministic registry.
    Compile {
        /// Verify the committed shards match the corpus without writing
        /// anything (exit 2 if stale). The registry counterpart of
        /// `index check`.
        #[arg(long)]
        check: bool,
        /// Emit the verdict as a JSON envelope on stdout (spec 037). Requires
        /// `--check` or `--spec`: the writing form mutates `.derived`, and its
        /// verdict is deliberately not machine-readable (spec 037 4).
        #[arg(long)]
        json: bool,
        /// Validate exactly one spec and write nothing (spec 056). Accepts the
        /// short id (`056`). Incompatible with `--check`.
        #[arg(long, value_name = "ID")]
        spec: Option<String>,
    },
    /// Read the effective configuration: every default resolved, and the
    /// built-in bypass floor merged with the adopter's list and attributed.
    Config {
        #[command(subcommand)]
        action: cmd_config::ConfigAction,
    },
    /// Read-only queries over the compiled registry.
    Registry {
        #[command(subcommand)]
        query: cmd_registry::RegistryQuery,
    },
    /// Build the codebase index, or check it for staleness.
    Index {
        #[command(subcommand)]
        action: Option<cmd_index::IndexAction>,
    },
    /// Run the corpus conformance lint.
    Lint {
        /// Fail (exit 1) if any warning-tier diagnostic is present.
        #[arg(long)]
        fail_on_warn: bool,
        /// Fail (exit 1) if any info-tier diagnostic is present.
        #[arg(long)]
        fail_on_info: bool,
        /// Emit the verdict as a JSON envelope on stdout (spec 037).
        #[arg(long)]
        json: bool,
    },
    /// Run a spec's declared acceptance: the `verify:cli` commands under its
    /// `## Verification` heading, in order, stopping at the first failure.
    ///
    /// Runs code the corpus declares (spec 049), so it is deliberately not part
    /// of the gate chain. `<id>` accepts the short form (`049`).
    Verify {
        /// Spec id, full (`049-slug`) or short (`049`).
        id: String,
        /// Print the commands that would run, one per line, and run none of
        /// them. Reading the plan before executing it is the safety affordance
        /// for the one verb that runs what the corpus declares.
        #[arg(long)]
        plan: bool,
        /// Emit the verdict as a JSON envelope on stdout (spec 037).
        #[arg(long)]
        json: bool,
    },
    /// The PR-time coupling gate: refuse code that drifts from its owning spec.
    Couple {
        /// Base ref for the diff (merge-base of `base...head`).
        #[arg(long, default_value = "origin/main")]
        base: String,
        /// Head ref for the diff.
        #[arg(long, default_value = "HEAD")]
        head: String,
        /// PR body (waiver source); a file path. Falls back to $SPEC_SPINE_PR_BODY.
        #[arg(long)]
        pr_body: Option<PathBuf>,
        /// Override the diff: read newline-delimited changed paths from this file
        /// (whole-file authority; no hunk data).
        #[arg(long)]
        paths_from: Option<PathBuf>,
        /// Emit the verdict as a JSON envelope on stdout (spec 037).
        #[arg(long)]
        json: bool,
    },
    /// Scaffold a new adopter: config, standards, a bootstrap spec, agent rules.
    Init {
        /// Overwrite existing files instead of skipping them.
        #[arg(long)]
        force: bool,
    },
    /// Emit a reproducible corpus attestation; optionally seal it (spec 023).
    Attest {
        /// Scope the attestation to one spec (spec 042), writing
        /// `<derived>/attestation/by-spec/<id>.json`. Records the verdicts; it
        /// is not a gate, and exit 0 means only that an attestation was written.
        #[arg(long, value_name = "ID")]
        spec: Option<String>,
        /// Also record the coupling (specs-and-code-in-sync) verdict.
        #[arg(long)]
        with_coupling: bool,
        /// Produce a detached Ed25519 seal over the attestation hash.
        #[arg(long)]
        sign: bool,
        /// The ed25519 signing key (32-byte seed; raw or hex). Required with --sign.
        #[arg(long, value_name = "PATH")]
        key: Option<PathBuf>,
        /// Override the seal's key id (defaults to the hex public key).
        #[arg(long, value_name = "ID")]
        key_id: Option<String>,
        /// Emit the verdict as a JSON envelope on stdout (spec 037).
        #[arg(long)]
        json: bool,
    },
    /// Verify a corpus attestation by recompute and/or detached signature.
    VerifyAttestation {
        /// Verify the per-spec attestation for this id (spec 042).
        #[arg(long, value_name = "ID")]
        spec: Option<String>,
        /// Re-read the corpus and check it reproduces the attestation (no key).
        #[arg(long)]
        recompute: bool,
        /// Check the detached seal against a supplied public key.
        #[arg(long)]
        signature: bool,
        /// The attestation file (defaults to <derived>/attestation/attestation.json).
        #[arg(long, value_name = "PATH")]
        attestation: Option<PathBuf>,
        /// The ed25519 public key (32 bytes; raw or hex). Required with --signature.
        #[arg(long, value_name = "PATH")]
        public_key: Option<PathBuf>,
        /// The detached seal file (defaults to the attestation's sibling .sig).
        #[arg(long, value_name = "PATH")]
        seal: Option<PathBuf>,
        /// Emit the verdict as a JSON envelope on stdout (spec 037).
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    // Spec 063 §3.1: a command line clap cannot parse is a usage error, and a
    // usage error is exit 3. Clap's own default is 2, which this tool spends on
    // staleness, so an unknown flag was indistinguishable from a stale ledger
    // except by matching clap's English on stderr. After this, exit 2 from any
    // verb means staleness and nothing else.
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => return exit_for_clap_error(e),
    };
    let repo = match cli.repo {
        Some(p) => p,
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let json_verb = cli.command.json_verb();
    // Spec 062 §3.2: the version pin is checked before any work. A read from a
    // mismatched binary is the quiet failure this exists to prevent: `registry
    // plan` from an old binary answers a question about a corpus it may
    // misunderstand, and answers it confidently.
    if let Err(e) = check_version_pin(&repo, &cli.command) {
        eprintln!("spec-spine: {e}");
        return ExitCode::from(e.exit_code());
    }

    let result = match &cli.command {
        Command::Compile { check, json, spec } => {
            cmd_compile::run(&repo, *check, *json, spec.as_deref())
        }
        Command::Config { action } => cmd_config::run(&repo, action),
        Command::Registry { query } => cmd_registry::run(&repo, query),
        Command::Index { action } => cmd_index::run(&repo, action.as_ref()),
        Command::Lint {
            fail_on_warn,
            fail_on_info,
            json,
        } => cmd_lint::run(&repo, *fail_on_warn, *fail_on_info, *json),
        Command::Verify { id, plan, json } => cmd_verify::run(&repo, id, *json, *plan),
        Command::Couple {
            base,
            head,
            pr_body,
            paths_from,
            json,
        } => cmd_couple::run(
            &repo,
            &cmd_couple::CoupleArgs {
                base: base.clone(),
                head: head.clone(),
                pr_body: pr_body.clone(),
                paths_from: paths_from.clone(),
                json: *json,
            },
        ),
        Command::Init { force } => cmd_init::run(&repo, *force),
        Command::Attest {
            spec,
            with_coupling,
            sign,
            key,
            key_id,
            json,
        } => cmd_attest::run(
            &repo,
            &cmd_attest::AttestArgs {
                spec: spec.clone(),
                with_coupling: *with_coupling,
                sign: *sign,
                key: key.clone(),
                key_id: key_id.clone(),
                json: *json,
            },
        ),
        Command::VerifyAttestation {
            spec,
            recompute,
            signature,
            attestation,
            public_key,
            seal,
            json,
        } => verify_attestation::run(
            &repo,
            &verify_attestation::VerifyArgs {
                spec: spec.clone(),
                recompute: *recompute,
                signature: *signature,
                attestation: attestation.clone(),
                public_key: public_key.clone(),
                seal: seal.clone(),
                json: *json,
            },
        ),
    };

    match result {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            // Spec 037 3.3: under `--json` a failure is an envelope on stdout,
            // not bare prose on stderr, so a consumer's happy path and error
            // path have the same shape. Handled once here rather than in six
            // commands: every `run` returning `Err` lands in this arm.
            let code = e.exit_code();
            match json_verb {
                Some(v) => emit_error_envelope(v, &e),
                None => eprintln!("spec-spine: {e}"),
            }
            ExitCode::from(code)
        }
    }
}

/// Render an error envelope, falling back to prose if the envelope itself
/// cannot be serialized.
///
/// The fallback is unreachable in practice (the envelope is two strings and two
/// scalars), but silently emitting nothing on stdout would leave a consumer
/// waiting on a document that never comes, which is worse than a prose line on
/// a stream it was not reading.
pub(crate) fn emit_error_envelope(verb: &str, error: &Error) {
    let verdict = Verdict::failure(verb, error);
    if let Err(e) = out::verdict(&verdict) {
        eprintln!("spec-spine: {error}");
        eprintln!("spec-spine: (could not render the JSON verdict: {e})");
    }
}

impl Command {
    /// The verdict envelope's `verb` when this invocation passed `--json`, and
    /// `None` otherwise. Drives only the failure path in `main`; the success
    /// path is each command's own, because only it holds the report.
    fn json_verb(&self) -> Option<&'static str> {
        match self {
            // `check: true` as well as `json: true`: `compile --json` without
            // `--check` is refused inside `cmd_compile`, which writes its own
            // envelope, so no envelope from this generic path ever carries a
            // verb the invocation did not qualify for.
            Command::Compile {
                json: true,
                check: true,
                ..
            } => Some(verb::COMPILE_CHECK),
            // Spec 056: `--spec` is its own verb. A consumer that branched on
            // `compile.check` must not silently receive a single-spec verdict.
            Command::Compile {
                json: true,
                spec: Some(_),
                ..
            } => Some(verb::COMPILE_SPEC),
            Command::Lint { json: true, .. } => Some(verb::LINT),
            Command::Couple { json: true, .. } => Some(verb::COUPLE),
            Command::Verify { json: true, .. } => Some(verb::VERIFY),
            Command::Attest { json: true, .. } => Some(verb::ATTEST),
            Command::VerifyAttestation { json: true, .. } => Some(verb::VERIFY_ATTESTATION),
            Command::Index {
                action: Some(cmd_index::IndexAction::Check { json: true, .. }),
            } => Some(verb::INDEX_CHECK),
            _ => None,
        }
    }
}

/// Load `<repo>/spec-spine.toml`, or the working default if it is absent.
pub(crate) fn load_repo_config(repo: &Path) -> Result<Config, Error> {
    let path = repo.join("spec-spine.toml");
    match std::fs::read_to_string(&path) {
        Ok(src) => spec_spine_types::load_config(&src),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(Error::Io(format!("read {}: {e}", path.display()))),
    }
}

/// Enforce `[meta] required_version` (spec 062 §3.2), except for the verbs that
/// must stay available when the pin is unsatisfiable.
///
/// `--version` and `--help` are clap's, and never reach here. `init` is exempt
/// because it scaffolds a repository that has no configuration yet, and in one
/// that does it is the verb an operator reaches for when things are wrong.
///
/// A configuration that cannot be read at all is left to the verb: this returns
/// `Ok` rather than pre-empting the real error with a worse one.
fn check_version_pin(repo: &Path, command: &Command) -> Result<(), Error> {
    if matches!(command, Command::Init { .. }) {
        return Ok(());
    }
    match load_repo_config(repo) {
        Ok(cfg) => cfg.check_required_version(env!("CARGO_PKG_VERSION")),
        Err(_) => Ok(()),
    }
}

/// Render a clap error and map it to this tool's exit-code contract
/// (spec 063 §3.1).
///
/// Help and version are successful requests for information: stdout, exit 0.
/// Everything else is the invocation failing to parse, which belongs in the
/// same cell as I/O, parse, schema and config failures. `3` rather than a new
/// code, because the contract is documented by four repositories and two
/// package shims, and a fourth code would extend it to distinguish a case none
/// of them needs distinguished.
fn exit_for_clap_error(e: clap::Error) -> ExitCode {
    use clap::error::ErrorKind;
    match e.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            // Asked for explicitly, and clap writes them to stdout itself.
            let _ = e.print();
            ExitCode::from(0)
        }
        // `spec-spine` with no subcommand prints help, but nobody asked for
        // help: the invocation was incomplete. It exits 3 with its siblings
        // rather than 0, so a script that dropped the verb still fails. See
        // spec 063 §3.1's decision entry.
        _ => {
            // Clap's message, unchanged: it names the offending argument better
            // than a paraphrase would.
            let _ = e.print();
            ExitCode::from(3)
        }
    }
}
