//! `spec-spine context packet`: one repository context packet, bound to a
//! verified Git tree (spec 159); `context delta` and `context apply` (spec
//! 164) are dispatched from here to `cmd_context_delta`.

use std::path::{Path, PathBuf};

use clap::Subcommand;
use spec_spine_core::{build_digest, context_packet, context_packet_document};
use spec_spine_types::{
    ContentCompleteness, Error, PacketRequest, verdict::Verdict, verdict::verb,
};

use crate::cmd_content::{bind_snapshot, confirm_unchanged};
use crate::out;

#[derive(Subcommand)]
pub enum ContextAction {
    /// Assemble one bounded context packet from HEAD or one exported revision.
    Packet {
        /// JSON context-packet request document.
        #[arg(long, value_name = "FILE")]
        request: PathBuf,
        /// Opaque repository identity carried into the packet.
        #[arg(long)]
        repository: String,
        /// Commit-ish to export. Omit to bind the clean working tree at HEAD.
        #[arg(long)]
        revision: Option<String>,
        /// Emit the canonical packet document.
        #[arg(long, required = true)]
        json: bool,
    },
    /// Compute the delta from caller-held predecessor pages to the packet
    /// this snapshot yields for the same request (spec 164).
    Delta {
        /// JSON context-packet request document (no continuation).
        #[arg(long, value_name = "FILE")]
        request: PathBuf,
        /// One predecessor page, in page order; repeat for every page.
        #[arg(long, value_name = "FILE", required = true)]
        predecessor: Vec<PathBuf>,
        /// A spec 160 documentation manifest to name invalidation candidates
        /// for; repeatable.
        #[arg(long, value_name = "FILE")]
        manifest: Vec<PathBuf>,
        /// Opaque repository identity carried into the target pages.
        #[arg(long)]
        repository: String,
        /// Commit-ish to export. Omit to bind the clean working tree at HEAD.
        #[arg(long)]
        revision: Option<String>,
        /// Emit the canonical delta or fallback document.
        #[arg(long, required = true)]
        json: bool,
    },
    /// Apply a delta to the predecessor pages it names and write every
    /// reproduced page, or refuse and write nothing (spec 164).
    Apply {
        /// One predecessor page, in page order; repeat for every page.
        #[arg(long, value_name = "FILE", required = true)]
        predecessor: Vec<PathBuf>,
        /// The delta document `context delta` wrote.
        #[arg(long, value_name = "FILE")]
        delta: PathBuf,
        /// A new or empty directory to write `page-NNNN.json` files into.
        #[arg(long, value_name = "DIR")]
        out_dir: PathBuf,
        /// List the written paths as a JSON array; a refusal is an envelope.
        #[arg(long)]
        json: bool,
    },
}

pub fn run(repo: &Path, action: &ContextAction) -> Result<u8, Error> {
    match action {
        ContextAction::Packet {
            request,
            repository,
            revision,
            json: _,
        } => run_packet(repo, request, repository, revision.as_deref()),
        ContextAction::Delta {
            request,
            predecessor,
            manifest,
            repository,
            revision,
            json: _,
        } => crate::cmd_context_delta::run_delta(
            repo,
            request,
            predecessor,
            manifest,
            repository,
            revision.as_deref(),
        ),
        ContextAction::Apply {
            predecessor,
            delta,
            out_dir,
            json,
        } => crate::cmd_context_delta::run_apply(predecessor, delta, out_dir, *json),
    }
}

fn run_packet(
    repo: &Path,
    request_path: &Path,
    repository: &str,
    revision: Option<&str>,
) -> Result<u8, Error> {
    let text = std::fs::read_to_string(request_path).map_err(|e| {
        Error::Io(format!(
            "read context-packet request {}: {e}",
            request_path.display()
        ))
    })?;
    let request: PacketRequest = serde_json::from_str(&text)
        .map_err(|e| Error::Usage(format!("invalid context-packet request: {e}")))?;

    let bound = bind_snapshot(repo, repository, revision)?;
    let build = own_build_digest();
    let packet = context_packet(
        &bound.config,
        bound.tree(),
        &request,
        &bound.snapshot,
        build.as_deref(),
    )?;
    confirm_unchanged(repo, &bound.snapshot, revision)?;

    if packet.completeness == ContentCompleteness::Incomplete {
        // Spec 159 3.8, 3.12: a required omission is a finding. The envelope
        // carries the whole deterministic packet, so the caller can see which
        // member is missing and why. The envelope writer sorts and lays it
        // out, so the packet is serialized once, here.
        let value = serde_json::to_value(&packet).map_err(|e| Error::Internal(e.to_string()))?;
        out::verdict(
            &Verdict::report(verb::CONTEXT_PACKET, 1, value)
                .with_summary("context.packet: incomplete; a required member is omitted"),
        )?;
        return Ok(1);
    }
    out!("{}", context_packet_document(&packet)?);
    Ok(0)
}

/// The running executable's SHA-256 (spec 159 D-5): the binding supplies the
/// build identity, and a build it cannot read is recorded as unverified.
pub(crate) fn own_build_digest() -> Option<String> {
    let path = std::env::current_exe().ok()?;
    let bytes = std::fs::read(path).ok()?;
    Some(build_digest(&bytes))
}
