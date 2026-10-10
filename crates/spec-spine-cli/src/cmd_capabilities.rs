//! `spec-spine capabilities [--json]`: what this binary supports (spec 170
//! 3.5).
//!
//! The verb list is walked from the clap tree the binary parses its own
//! arguments with, so a verb that is wired is listed and a verb that is not
//! cannot be. What clap does not know is which schema a verb's `--json` output
//! carries; that is [`json_axes`], one table, and `tests/capabilities.rs`
//! fails naming any verb that takes `--json` and is missing from it.
//!
//! Reads no repository and writes nothing: `main` answers it before the
//! version pin is read, so it answers in any directory, including one whose
//! `required_version` this binary does not meet (170 3.6).
//!
//! Spec 162 rides the same answer rather than adding a second one (162 D-8):
//! the document carries the capability catalog under `catalog`,
//! `--operation <name>` shows one of its records, and `capabilities verify`
//! checks pinned operation digests against it (162 §3.11, §3.12).

use std::collections::BTreeMap;

use clap::Subcommand;
use spec_spine_types::{
    CAPABILITIES_SCHEMA_VERSION, CATALOG_SCHEMA_VERSION, CONFIG_VERSION, Capabilities,
    CapabilityVerifyRequest, DELTA_SCHEMA_VERSION, Error, Operation, READ_SCHEMA_VERSION,
    SchemaAxis, VERDICT_SCHEMA_VERSION, VerbCapability, Verdict, verdict::verb,
};

#[derive(Subcommand)]
pub enum CapabilitiesAction {
    /// Check pinned operation digests against this binary's catalog (spec
    /// 162). Reads no repository and writes nothing
    ///
    /// Exit 0 when every pin is `current`, 1 when any is `changed` or
    /// `missing`. The observed digest is printed for a human to copy; nothing
    /// writes a pin.
    Verify {
        /// `<operation>=sha256:<hex>`, the digest a consumer pinned. Repeat
        /// once per operation.
        #[arg(long, value_name = "NAME=DIGEST", required = true)]
        expect: Vec<String>,
        /// Emit the report as a verdict envelope on stdout (spec 034).
        #[arg(long)]
        json: bool,
    },
}

/// Route `capabilities`: the document, one record, or the pin check.
pub fn dispatch(
    cli: &clap::Command,
    json: bool,
    operation: Option<&str>,
    action: Option<&CapabilitiesAction>,
) -> Result<u8, Error> {
    match (action, operation) {
        (Some(CapabilitiesAction::Verify { expect, json }), _) => verify(expect, *json),
        (None, Some(name)) => show_operation(name, json),
        (None, None) => run(cli, json),
    }
}

/// `--operation <name>`: one catalog record (162 §3.12).
fn show_operation(name: &str, json: bool) -> Result<u8, Error> {
    let record = spec_spine_core::capability_operation(name)?;
    if json {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct OneOperation {
            schema_version: &'static str,
            operation: Operation,
        }
        out!(
            "{}",
            spec_spine_core::read_document(
                &OneOperation {
                    schema_version: CATALOG_SCHEMA_VERSION,
                    operation: record,
                },
                spec_spine_core::Versioning::Preexisting("schemaVersion")
            )?
        );
    } else {
        outln!(
            "{}  {}  {}",
            record.name,
            record.stability,
            effects_line(&record)
        );
    }
    Ok(0)
}

/// A compact effects string for the human projection.
fn effects_line(op: &Operation) -> String {
    let e = &op.effects;
    let mut parts = Vec::new();
    for (label, items) in [
        ("reads", &e.reads),
        ("writes", &e.writes),
        ("executes", &e.executes),
        ("env", &e.environment),
        ("authority", &e.authority),
    ] {
        if !items.is_empty() {
            parts.push(format!("{label}={}", items.join(",")));
        }
    }
    parts.push(format!("network={}", e.network));
    parts.join(" ")
}

/// Parse `--expect` arguments (162 §3.11): no `=`, an empty name or a name
/// given twice is usage (exit 3); the digest's form is checked by the core.
fn parse_expect(expect: &[String]) -> Result<CapabilityVerifyRequest, Error> {
    let mut pins = BTreeMap::new();
    for raw in expect {
        let Some((name, digest)) = raw.split_once('=') else {
            return Err(Error::Usage(format!(
                "`--expect {raw}`: expected `<operation>=sha256:<hex>`"
            )));
        };
        if name.is_empty() {
            return Err(Error::Usage(format!(
                "`--expect {raw}`: the operation name is empty"
            )));
        }
        if pins.insert(name.to_string(), digest.to_string()).is_some() {
            return Err(Error::Usage(format!(
                "`--expect`: `{name}` is pinned twice"
            )));
        }
    }
    Ok(CapabilityVerifyRequest { expect: pins })
}

/// `capabilities verify` (162 §3.11).
fn verify(expect: &[String], json: bool) -> Result<u8, Error> {
    let request = parse_expect(expect)?;
    let report = spec_spine_core::capability_verify(&request)?;
    let code = report.exit_code();
    if json {
        let value = serde_json::to_value(&report).map_err(|e| Error::Internal(e.to_string()))?;
        crate::out::verdict(&Verdict::report(verb::CAPABILITIES_VERIFY, code, value))?;
    } else {
        for r in &report.results {
            match (r.outcome.as_str(), &r.observed) {
                ("changed", Some(observed)) => {
                    outln!(
                        "{}: changed (pinned {}, observed {observed})",
                        r.name,
                        r.expected
                    )
                }
                (outcome, _) => outln!("{}: {outcome}", r.name),
            }
        }
    }
    Ok(code)
}

/// Flags every verb has that say nothing about the verb: clap's help, and the
/// global `--repo`.
const UNLISTED_FLAGS: &[&str] = &["help", "repo"];

fn run(cli: &clap::Command, json: bool) -> Result<u8, Error> {
    let doc = document(cli)?;
    if json {
        // The document names its own axis under `schemaVersion`, so the read
        // emitter sorts and lays it out without stamping the read axis over it.
        out!(
            "{}",
            spec_spine_core::read_document(
                &doc,
                spec_spine_core::Versioning::Preexisting("schemaVersion")
            )?
        );
    } else {
        outln!("spec-spine {}", doc.version);
        for verb in &doc.verbs {
            let axes: Vec<String> = verb
                .json
                .iter()
                .map(|a| format!("{} {}", a.axis, a.version))
                .collect();
            if axes.is_empty() {
                outln!("{}", verb.path);
            } else {
                outln!("{}  (--json: {})", verb.path, axes.join(", "));
            }
        }
    }
    Ok(0)
}

/// The document for the command tree `cli`.
pub fn document(cli: &clap::Command) -> Result<Capabilities, Error> {
    let mut verbs = Vec::new();
    for sub in cli.get_subcommands() {
        collect(sub, &mut Vec::new(), &mut verbs);
    }
    verbs.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Capabilities {
        schema_version: CAPABILITIES_SCHEMA_VERSION.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        verbs,
        catalog: spec_spine_core::capability_catalog()?,
    })
}

/// Add `cmd` (under `parents`) and its runnable descendants. A command is a
/// verb when it has no subcommands, or when it runs without one (`index`).
fn collect(cmd: &clap::Command, parents: &mut Vec<String>, out: &mut Vec<VerbCapability>) {
    if cmd.get_name() == "help" {
        return;
    }
    parents.push(cmd.get_name().to_string());
    let has_subcommands = cmd.get_subcommands().any(|s| s.get_name() != "help");
    if !has_subcommands || !cmd.is_subcommand_required_set() {
        let path = parents.join(" ");
        let mut flags: Vec<String> = cmd
            .get_arguments()
            .filter_map(|a| a.get_long())
            .filter(|l| !UNLISTED_FLAGS.contains(l))
            .map(str::to_string)
            .collect();
        flags.sort();
        flags.dedup();
        let json = if flags.iter().any(|f| f == "json") {
            json_axes(&path)
        } else {
            Vec::new()
        };
        out.push(VerbCapability { path, flags, json });
    }
    for sub in cmd.get_subcommands() {
        collect(sub, parents, out);
    }
    parents.pop();
}

/// The schema axes a verb's `--json` success output carries.
///
/// Three families: the verdict verbs answer with the verdict envelope (spec
/// 034), the reads with a bare read document (spec 074), and `config show`
/// with the configuration shape (spec 047). `delta`'s envelope carries its
/// own report axis inside it (spec 071). A verb that takes `--json` and is not
/// here gets an empty list, which `tests/capabilities.rs` refuses; a verb that
/// gains a second `--json` form adds its axis here in the same change.
pub fn json_axes(path: &str) -> Vec<SchemaAxis> {
    let verdict = || axis("verdict", VERDICT_SCHEMA_VERSION);
    let read = || axis("read", READ_SCHEMA_VERSION);
    match path {
        "check" | "compile" | "couple" | "attest" | "lint" | "index check" | "verify"
        | "verify-attestation" => vec![verdict()],
        "delta" => vec![verdict(), axis("delta", DELTA_SCHEMA_VERSION)],
        "content select"
        | "index coverage"
        | "index diagnostics"
        | "index orphans"
        | "index owner"
        | "interface verify"
        | "registry closure"
        | "registry impacts"
        | "registry list"
        | "registry moves"
        | "registry obligation"
        | "registry plan"
        | "registry relationships"
        | "registry show"
        | "registry status-report"
        | "scope compare"
        | "scope evaluate" => vec![read()],
        "config show" => vec![axis("config", CONFIG_VERSION)],
        "capabilities" => vec![axis("capabilities", CAPABILITIES_SCHEMA_VERSION)],
        "capabilities verify" => vec![verdict()],
        _ => Vec::new(),
    }
}

fn axis(name: &str, version: &str) -> SchemaAxis {
    SchemaAxis {
        axis: name.to_string(),
        version: version.to_string(),
    }
}
