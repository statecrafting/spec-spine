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

use spec_spine_types::{
    CAPABILITIES_SCHEMA_VERSION, CONFIG_VERSION, Capabilities, DELTA_SCHEMA_VERSION, Error,
    READ_SCHEMA_VERSION, SchemaAxis, TRACEABILITY_SCHEMA_VERSION, VERDICT_SCHEMA_VERSION,
    VerbCapability,
};

/// Flags every verb has that say nothing about the verb: clap's help, and the
/// global `--repo`.
const UNLISTED_FLAGS: &[&str] = &["help", "repo"];

pub fn run(cli: &clap::Command, json: bool) -> Result<u8, Error> {
    let doc = document(cli);
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
pub fn document(cli: &clap::Command) -> Capabilities {
    let mut verbs = Vec::new();
    for sub in cli.get_subcommands() {
        collect(sub, &mut Vec::new(), &mut verbs);
    }
    verbs.sort_by(|a, b| a.path.cmp(&b.path));
    Capabilities {
        schema_version: CAPABILITIES_SCHEMA_VERSION.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        verbs,
    }
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
        // Spec 169 §3.9: a read document carrying its own axis beside the
        // read stamp.
        "registry traceability" => vec![read(), axis("traceability", TRACEABILITY_SCHEMA_VERSION)],
        "config show" => vec![axis("config", CONFIG_VERSION)],
        "capabilities" => vec![axis("capabilities", CAPABILITIES_SCHEMA_VERSION)],
        _ => Vec::new(),
    }
}

fn axis(name: &str, version: &str) -> SchemaAxis {
    SchemaAxis {
        axis: name.to_string(),
        version: version.to_string(),
    }
}
