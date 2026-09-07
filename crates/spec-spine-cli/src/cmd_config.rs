//! `spec-spine config show`: the effective configuration, as a governed read
//! (spec 054).
//!
//! `spec-spine.toml` is only half the configuration a coupling decision is made
//! from. The other half is `couple.rs::DEFAULT_BYPASS_PREFIXES`, thirteen
//! entries compiled into the binary and additive to the adopter's list, so the
//! list the gate matches against is a merge of a constant nobody outside the
//! process can see and a list the adopter wrote. Every table is
//! `#[serde(default)]` besides, so a file with three keys in it configures
//! thirty. Reading the file back gives you what you wrote, which is not what
//! the tool uses.
//!
//! This verb **reads**. It never writes, never creates a missing
//! `spec-spine.toml`, and never emits a "suggested" file: scaffolding is
//! `init`'s job and has been since spec 006.

use std::path::Path;

use clap::Subcommand;
use spec_spine_core::effective_bypass_prefixes;
use spec_spine_types::{EffectiveConfig, Error};

use crate::load_repo_config;
use crate::out;

/// Actions under `spec-spine config`.
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Print the effective configuration: every default resolved, and the
    /// bypass floor merged with the adopter's list and attributed.
    Show {
        /// Emit the configuration as a single JSON object on stdout.
        #[arg(long)]
        json: bool,
    },
}

pub fn run(repo: &Path, action: &ConfigAction) -> Result<u8, Error> {
    let ConfigAction::Show { json } = action;
    // The same load every other verb does, so what is reported is what they
    // would consume. No committed artifact is read, which is why this works on
    // a repository that has never compiled: the state an adopter is in when
    // they most need to see what their config resolved to.
    let cfg = load_repo_config(repo)?;
    let effective = EffectiveConfig::new(&cfg, effective_bypass_prefixes(&cfg));

    if *json {
        // Deliberately NOT the spec 037 verdict envelope. An envelope carries
        // `ok` and `exitCode`, and a verdict is what a gate returns. This verb
        // decides nothing and cannot fail a gate, so an envelope would put a
        // permanently-true `ok` on a verb with no notion of passing. It is a
        // query, and it takes `--json` the way `registry show` does: the
        // object itself.
        let s =
            serde_json::to_string_pretty(&effective).map_err(|e| Error::Schema(e.to_string()))?;
        out::line(format_args!("{s}"));
        return Ok(0);
    }

    out::line(format_args!(
        "config_version = \"{}\"",
        effective.config_version
    ));
    render(&effective);
    Ok(0)
}

/// The prose rendering. Carries no fact the JSON lacks (spec 054 §3.3): it is
/// the same object, laid out for a person.
fn render(e: &EffectiveConfig) {
    out::line(format_args!("\n[manifest]"));
    kv("metadata_namespace", &e.manifest.metadata_namespace);

    out::line(format_args!("\n[domains]"));
    list("allowed", &e.domains.allowed);
    out::line(format_args!("\n[kind]"));
    list("allowed", &e.kind.allowed);

    out::line(format_args!("\n[layout]"));
    kv("specs_dir", &e.layout.specs_dir);
    kv("derived_dir", &e.layout.derived_dir);
    kv("standards_dir", &e.layout.standards_dir);
    kv("schemas_dir", &e.layout.schemas_dir);
    kv("state_dir", &e.layout.state_dir);
    kv("cargo_workspace", &e.layout.cargo_workspace);
    list("npm_workspaces", &e.layout.npm_workspaces);
    list(
        "standalone_rust_workspaces",
        &e.layout.standalone_rust_workspaces,
    );
    list("standalone_npm_packages", &e.layout.standalone_npm_packages);

    out::line(format_args!("\n[index]"));
    list("extra_hashed_inputs", &e.index.extra_hashed_inputs);
    list("resolver_exclusions", &e.index.resolver_exclusions);

    out::line(format_args!("\n[branding]"));
    kv("compiler_id", &e.branding.compiler_id);
    kv("indexer_id", &e.branding.indexer_id);

    out::line(format_args!("\n[coupling]"));
    kv("waiver_keyword", &e.coupling.waiver_keyword);
    flag("require_ownership", e.coupling.require_ownership);
    flag(
        "auto_waive_dependency_only",
        e.coupling.auto_waive_dependency_only,
    );
    out::line(format_args!("  bypass_prefixes:"));
    // Padded so the attributions line up: an eye scanning for "which of these
    // did we ask for" is reading the right-hand column, not the left.
    let width = e
        .coupling
        .bypass_prefixes
        .iter()
        .map(|b| b.prefix.chars().count())
        .max()
        .unwrap_or(0);
    for b in &e.coupling.bypass_prefixes {
        let sources: Vec<&str> = b.sources.iter().map(|s| s.label()).collect();
        out::line(format_args!(
            "    {:<width$}  ({})",
            b.prefix,
            sources.join(", "),
            width = width
        ));
    }

    out::line(format_args!("\n[provenance.uri_schemes]"));
    for (k, v) in &e.provenance.uri_schemes {
        out::line(format_args!("  {k} = \"{v}\""));
    }

    out::line(format_args!("\n[frontmatter]"));
    list("extra_known_keys", &e.frontmatter.extra_known_keys);

    out::line(format_args!("\n[meta]"));
    match &e.meta.required_version {
        Some(v) => kv("required_version", v),
        None => out::line(format_args!("  required_version = (unpinned)")),
    }

    out::line(format_args!("\n[lint]"));
    flag(
        "require_ordinal_monotonic_depends_on",
        e.lint.require_ordinal_monotonic_depends_on,
    );
}

/// A string scalar, quoted, so an empty `state_dir` reads as `""` rather than
/// as a key with nothing after it.
fn kv(key: &str, value: &str) {
    out::line(format_args!("  {key} = \"{value}\""));
}

/// A boolean scalar, unquoted, as TOML writes it.
fn flag(key: &str, value: bool) {
    out::line(format_args!("  {key} = {value}"));
}

fn list(key: &str, values: &[String]) {
    if values.is_empty() {
        out::line(format_args!("  {key} = []"));
        return;
    }
    let quoted: Vec<String> = values.iter().map(|v| format!("\"{v}\"")).collect();
    out::line(format_args!("  {key} = [{}]", quoted.join(", ")));
}
