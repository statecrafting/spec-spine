//! `spec-spine documentation manifest validate|freshness` (spec 160): check a
//! documentation manifest against the snapshot it records, or evaluate its
//! freshness against the current one.
//!
//! The snapshot is bound exactly as `content select` and `context packet` bind
//! it: the clean working tree at `HEAD`, or one exported revision, refused when
//! dirty and refused again when it moved during the read. Packet pages,
//! interface exports and external resolutions are files the caller supplies.
//! Nothing here assembles a packet, runs a generator or an example, or opens a
//! network connection (spec 160 3.16).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use clap::Subcommand;
use spec_spine_core::{
    Exports, documentation_freshness, documentation_freshness_document,
    documentation_manifest_document, load_export, packet_snapshot, parse_documentation_manifest,
    read_documentation_outputs, validate_documentation_manifest,
};
use spec_spine_types::{
    ContextPacket, DocumentationManifest, Error, ExternalResolution, FreshnessStatus,
    MANIFEST_MAX_BYTES, verdict::Verdict, verdict::verb,
};

use crate::cmd_content::{BoundSnapshot, bind_snapshot, confirm_unchanged};
use crate::out;

#[derive(Subcommand)]
pub enum DocumentationAction {
    /// Documentation manifests (spec 160): declared inputs, generator
    /// identity, outputs and regions, bound to one context packet.
    Manifest {
        #[command(subcommand)]
        action: ManifestAction,
    },
}

#[derive(Subcommand)]
pub enum ManifestAction {
    /// Validate a documentation manifest against the snapshot it records: its
    /// packet pages, inputs, outputs and regions
    ///
    /// Prints the canonical manifest.
    Validate {
        /// The documentation manifest.
        #[arg(long, value_name = "FILE")]
        manifest: PathBuf,
        /// One context-packet page, repeated in page order.
        #[arg(long = "packet", value_name = "FILE", required = true)]
        packets: Vec<PathBuf>,
        /// Opaque repository identity, as the packet records it.
        #[arg(long)]
        repository: String,
        /// Commit-ish to export. Omit to bind the clean working tree at HEAD.
        #[arg(long)]
        revision: Option<String>,
        /// Repository-relative directory every output must lie beneath.
        #[arg(long, value_name = "DIR", default_value = ".")]
        output_root: String,
        /// Emit the canonical manifest document.
        #[arg(long, required = true)]
        json: bool,
    },
    /// Evaluate a documentation manifest's freshness against the current
    /// snapshot: exit 0 when fresh, 1 with the result when stale, incomplete or
    /// unverified
    ///
    /// The result is carried in the verdict envelope.
    Freshness {
        /// The documentation manifest.
        #[arg(long, value_name = "FILE")]
        manifest: PathBuf,
        /// One current context-packet page for the recorded request, repeated
        /// in page order.
        #[arg(long = "packet", value_name = "FILE", required = true)]
        packets: Vec<PathBuf>,
        /// Opaque repository identity, as the packet records it.
        #[arg(long)]
        repository: String,
        /// Commit-ish to export. Omit to bind the clean working tree at HEAD.
        #[arg(long)]
        revision: Option<String>,
        /// Repository-relative directory every output must lie beneath.
        #[arg(long, value_name = "DIR", default_value = ".")]
        output_root: String,
        /// `<corpus>=<dir>`: a local export of a corpus an interface input
        /// cites (spec 110). A corpus with none is `unverified`.
        #[arg(long = "export", value_name = "CORPUS=DIR")]
        exports: Vec<String>,
        /// JSON array of external resolutions. An external reference with
        /// none is `unverified`.
        #[arg(long, value_name = "FILE")]
        resolutions: Option<PathBuf>,
        /// Emit the freshness document.
        #[arg(long, required = true)]
        json: bool,
    },
}

pub fn run(repo: &Path, action: &DocumentationAction) -> Result<u8, Error> {
    let DocumentationAction::Manifest { action } = action;
    match action {
        ManifestAction::Validate {
            manifest,
            packets,
            repository,
            revision,
            output_root,
            json: _,
        } => validate(
            repo,
            manifest,
            packets,
            repository,
            revision.as_deref(),
            output_root,
        ),
        ManifestAction::Freshness {
            manifest,
            packets,
            repository,
            revision,
            output_root,
            exports,
            resolutions,
            json: _,
        } => freshness(
            repo,
            &Inputs {
                manifest,
                packets,
                repository,
                revision: revision.as_deref(),
                output_root,
            },
            exports,
            resolutions.as_deref(),
        ),
    }
}

struct Inputs<'a> {
    manifest: &'a Path,
    packets: &'a [PathBuf],
    repository: &'a str,
    revision: Option<&'a str>,
    output_root: &'a str,
}

fn validate(
    repo: &Path,
    manifest: &Path,
    packets: &[PathBuf],
    repository: &str,
    revision: Option<&str>,
    output_root: &str,
) -> Result<u8, Error> {
    let manifest = read_manifest(manifest)?;
    let pages = read_pages(packets)?;
    let bound = bind_snapshot(repo, repository, revision)?;
    let outputs = read_documentation_outputs(&bound.config, bound.tree(), output_root, &manifest)?;
    confirm_unchanged(repo, &bound.snapshot, revision)?;
    let canonical = validate_documentation_manifest(
        &manifest,
        &pages,
        &packet_snapshot(&bound.snapshot),
        &outputs,
    )?;
    out!("{}", documentation_manifest_document(&canonical)?);
    Ok(0)
}

fn freshness(
    repo: &Path,
    inputs: &Inputs<'_>,
    export_args: &[String],
    resolutions: Option<&Path>,
) -> Result<u8, Error> {
    let manifest = read_manifest(inputs.manifest)?;
    let pages = read_pages(inputs.packets)?;
    let resolutions: Vec<ExternalResolution> = match resolutions {
        None => Vec::new(),
        Some(path) => serde_json::from_str(&read_text(
            path,
            "external resolutions",
            MANIFEST_MAX_BYTES,
        )?)
        .map_err(|e| Error::Usage(format!("invalid external resolutions: {e}")))?,
    };
    let exports = load_exports(export_args, &manifest)?;
    let bound: BoundSnapshot = bind_snapshot(repo, inputs.repository, inputs.revision)?;
    let outputs =
        read_documentation_outputs(&bound.config, bound.tree(), inputs.output_root, &manifest)?;
    confirm_unchanged(repo, &bound.snapshot, inputs.revision)?;
    let result = documentation_freshness(
        &manifest,
        &packet_snapshot(&bound.snapshot),
        &pages,
        &exports,
        &resolutions,
        &outputs,
    )?;
    if result.status == FreshnessStatus::Fresh {
        out!("{}", documentation_freshness_document(&result)?);
        return Ok(0);
    }
    // Spec 160 3.15: a stale, incomplete or unverified result is a finding,
    // and the envelope carries the whole result.
    let status = match result.status {
        FreshnessStatus::Fresh => "fresh",
        FreshnessStatus::Unverified => "unverified",
        FreshnessStatus::Stale => "stale",
        FreshnessStatus::Incomplete => "incomplete",
    };
    let value = serde_json::to_value(&result).map_err(|e| Error::Internal(e.to_string()))?;
    out::verdict(
        &Verdict::report(verb::DOCUMENTATION_MANIFEST_FRESHNESS, 1, value).with_summary(format!(
            "documentation.manifest.freshness: {status}; {} finding(s)",
            result.findings.len()
        )),
    )?;
    Ok(1)
}

/// A context-packet page holds up to spec 159's 4 MiB of selected content
/// plus its JSON escaping and envelope, so pages get their own, wider
/// allocation bound than a manifest's 4 MiB canonical limit.
const PAGE_FILE_LIMIT: usize = 64 * 1024 * 1024;

/// Read a caller-named file of at most `limit` bytes: the size is checked
/// before the read allocates, and again on what was read, so a file that grew
/// in between is still refused.
fn read_text(path: &Path, what: &str, limit: usize) -> Result<String, Error> {
    let too_large = |len: u64| {
        Error::Usage(format!(
            "limit-exceeded: {what} {} is {len} bytes, above the {limit}-byte limit",
            path.display()
        ))
    };
    let len = std::fs::metadata(path)
        .map_err(|e| Error::Io(format!("read {what} {}: {e}", path.display())))?
        .len();
    if len > limit as u64 {
        return Err(too_large(len));
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| Error::Io(format!("read {what} {}: {e}", path.display())))?;
    if text.len() > limit {
        return Err(too_large(text.len() as u64));
    }
    Ok(text)
}

fn read_manifest(path: &Path) -> Result<DocumentationManifest, Error> {
    parse_documentation_manifest(&read_text(
        path,
        "documentation manifest",
        MANIFEST_MAX_BYTES,
    )?)
}

fn read_pages(paths: &[PathBuf]) -> Result<Vec<ContextPacket>, Error> {
    paths
        .iter()
        .map(|p| {
            let text = read_text(p, "context-packet page", PAGE_FILE_LIMIT)?;
            serde_json::from_str(&text).map_err(|e| {
                Error::Usage(format!("invalid context-packet page {}: {e}", p.display()))
            })
        })
        .collect()
}

/// Parse `--export <corpus>=<dir>` (the spec 110 form) and read, from each
/// directory, only the specs the manifest's interface inputs cite there.
fn load_exports(args: &[String], manifest: &DocumentationManifest) -> Result<Exports, Error> {
    let mut wanted: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for i in &manifest.interfaces {
        wanted.entry(&i.corpus).or_default().insert(i.spec.clone());
    }
    let mut exports = Exports::new();
    for arg in args {
        let Some((name, dir)) = arg
            .split_once('=')
            .filter(|(n, d)| !n.is_empty() && !d.is_empty())
        else {
            return Err(Error::Usage(format!(
                "--export '{arg}' is not <corpus>=<dir>"
            )));
        };
        if exports.contains_key(name) {
            return Err(Error::Usage(format!(
                "--export names corpus '{name}' twice: one directory per corpus"
            )));
        }
        let ids: Vec<String> = wanted
            .get(name)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        exports.insert(name.to_string(), load_export(Path::new(dir), &ids)?);
    }
    Ok(exports)
}
