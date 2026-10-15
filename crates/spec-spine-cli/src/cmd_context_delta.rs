//! `spec-spine context delta` and `context apply` (spec 164): a context-packet
//! delta from caller-held predecessor pages to the page set spec 159 assembles
//! on one verified Git tree, and its exact application.

use std::path::{Path, PathBuf};

use spec_spine_core::{
    context_packet_apply, context_packet_delta, context_packet_delta_document,
    delta_target_incomplete, parse_delta,
};
use spec_spine_types::{
    CONTEXT_PACKET_DELTA_SCHEMA_VERSION, ContextDeltaRequest, ContextDeltaResult, Error,
    PacketRequest, verdict::Verdict, verdict::verb,
};

use crate::cmd_content::{bind_snapshot, confirm_unchanged};
use crate::cmd_context::own_build_digest;
use crate::out;

/// Run `context delta`: read the spec 159 request, the predecessor pages and
/// any manifests as exact bytes, bind the target snapshot, and write the
/// canonical delta or fallback.
pub fn run_delta(
    repo: &Path,
    request_path: &Path,
    predecessors: &[PathBuf],
    manifests: &[PathBuf],
    repository: &str,
    revision: Option<&str>,
) -> Result<u8, Error> {
    let text = read(request_path, "context-packet request")?;
    let request: PacketRequest = serde_json::from_str(&text)
        .map_err(|e| Error::Usage(format!("invalid context-packet request: {e}")))?;
    let delta_request = ContextDeltaRequest {
        request,
        predecessor_pages: predecessors
            .iter()
            .map(|p| read(p, "predecessor page"))
            .collect::<Result<_, _>>()?,
        manifests: manifests
            .iter()
            .map(|p| read(p, "documentation manifest"))
            .collect::<Result<_, _>>()?,
        consumer_schema_version: CONTEXT_PACKET_DELTA_SCHEMA_VERSION.to_string(),
    };

    let bound = bind_snapshot(repo, repository, revision)?;
    let build = own_build_digest();
    let result = context_packet_delta(
        &bound.config,
        bound.tree(),
        &delta_request,
        &bound.snapshot,
        build.as_deref(),
    )?;
    confirm_unchanged(repo, &bound.snapshot, revision)?;

    if let ContextDeltaResult::Delta(delta) = &result
        && delta_target_incomplete(delta)
    {
        // Spec 164 3.12: a delta whose target omits a required member is a
        // finding, carried whole as the envelope's report.
        let value = serde_json::to_value(&result).map_err(|e| Error::Internal(e.to_string()))?;
        out::verdict(
            &Verdict::report(verb::CONTEXT_DELTA, 1, value).with_summary(
                "context.delta: the target is incomplete; a required member is omitted",
            ),
        )?;
        return Ok(1);
    }
    out!("{}", context_packet_delta_document(&result)?);
    Ok(0)
}

/// Run `context apply`: reproduce every target page from the predecessor
/// pages and the delta, then write `page-NNNN.json` files into `out_dir`,
/// all or none. Every page is reproduced and checked before the first file is
/// written. Each written path is printed, one per line, or under `json` as
/// one canonical JSON array (D-20).
pub fn run_apply(
    predecessors: &[PathBuf],
    delta_path: &Path,
    out_dir: &Path,
    json: bool,
) -> Result<u8, Error> {
    let pages: Vec<String> = predecessors
        .iter()
        .map(|p| read(p, "predecessor page"))
        .collect::<Result<_, _>>()?;
    let delta = parse_delta(&read(delta_path, "context-packet delta")?)?;
    check_out_dir(out_dir)?;
    let reproduced = context_packet_apply(&pages, &delta)?;

    std::fs::create_dir_all(out_dir)
        .map_err(|e| Error::Io(format!("create {}: {e}", out_dir.display())))?;
    let mut written = Vec::with_capacity(reproduced.len());
    for (i, page) in reproduced.iter().enumerate() {
        let path = out_dir.join(format!("page-{:04}.json", i + 1));
        std::fs::write(&path, page)
            .map_err(|e| Error::Io(format!("write {}: {e}", path.display())))?;
        written.push(path.display().to_string());
    }
    if json {
        out!("{}", canonical_json(&written)?);
    } else {
        for path in &written {
            out!("{path}");
        }
    }
    Ok(0)
}

/// The output directory must be new or empty (spec 164 3.12): apply never
/// mixes its pages with files it did not write.
fn check_out_dir(dir: &Path) -> Result<(), Error> {
    match std::fs::read_dir(dir) {
        Ok(mut entries) => {
            if entries.next().is_some() {
                return Err(Error::Usage(format!(
                    "--out-dir {} is not empty",
                    dir.display()
                )));
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotADirectory => Err(Error::Usage(format!(
            "--out-dir {} is not a directory",
            dir.display()
        ))),
        Err(e) => Err(Error::Io(format!("read {}: {e}", dir.display()))),
    }
}

fn canonical_json(paths: &[String]) -> Result<String, Error> {
    let mut text =
        serde_json::to_string_pretty(paths).map_err(|e| Error::Internal(e.to_string()))?;
    text.push('\n');
    Ok(text)
}

fn read(path: &Path, what: &str) -> Result<String, Error> {
    std::fs::read_to_string(path)
        .map_err(|e| Error::Io(format!("read {what} {}: {e}", path.display())))
}
