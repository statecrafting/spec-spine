//! `spec-spine reference generate | check | model` (spec 166): one
//! deterministic reference page per exported spec. Generation writes only
//! under the request's `outputDir`; check and model write nothing, and no
//! verb runs an example.

use std::path::Path;

use clap::Subcommand;
use spec_spine_core::{
    reference_check, reference_check_document, reference_generate, reference_model,
    reference_model_document,
};
use spec_spine_types::{Error, ReferenceRequest, verdict::Verdict, verdict::verb};

use crate::load_repo_config;

#[derive(Subcommand)]
pub enum ReferenceAction {
    /// Write one reference page per exported spec under the request's outputDir.
    Generate {
        /// JSON export request document.
        #[arg(long, value_name = "FILE")]
        request: std::path::PathBuf,
        /// List the written, unchanged and removed paths as JSON; a refusal is an envelope.
        #[arg(long)]
        json: bool,
    },
    /// Regenerate in memory and compare with the committed pages; never writes.
    Check {
        /// JSON export request document.
        #[arg(long, value_name = "FILE")]
        request: std::path::PathBuf,
        /// Emit the check document; findings are the envelope's report.
        #[arg(long)]
        json: bool,
    },
    /// Emit every page model the export would project, as one document.
    Model {
        /// JSON export request document.
        #[arg(long, value_name = "FILE")]
        request: std::path::PathBuf,
        /// Emit the canonical model document.
        #[arg(long, required = true)]
        json: bool,
    },
}

pub fn run(repo: &Path, action: &ReferenceAction) -> Result<u8, Error> {
    match action {
        ReferenceAction::Generate { request, json } => generate(repo, request, *json),
        ReferenceAction::Check { request, json } => check(repo, request, *json),
        ReferenceAction::Model { request, json: _ } => {
            let (cfg, request) = load(repo, request)?;
            out!(
                "{}",
                reference_model_document(&reference_model(&cfg, repo, &request)?)?
            );
            Ok(0)
        }
    }
}

fn load(repo: &Path, path: &Path) -> Result<(spec_spine_types::Config, ReferenceRequest), Error> {
    let cfg = load_repo_config(repo)?;
    let text = std::fs::read_to_string(path)
        .map_err(|e| Error::Io(format!("read reference request {}: {e}", path.display())))?;
    let request: ReferenceRequest = serde_json::from_str(&text)
        .map_err(|e| Error::Usage(format!("invalid reference request: {e}")))?;
    Ok((cfg, request))
}

/// Build every page before writing any (spec 166 3.9), then write the pages
/// and remove the empty orphans.
fn generate(repo: &Path, request_path: &Path, json: bool) -> Result<u8, Error> {
    let (cfg, request) = load(repo, request_path)?;
    let export = reference_generate(&cfg, repo, &request)?;
    let dir = repo.join(&request.output_dir);
    let mut written = Vec::new();
    let mut unchanged = Vec::new();
    for (path, text) in &export.pages {
        let target = repo.join(path);
        let same = std::fs::read(&target).is_ok_and(|b| b == text.as_bytes());
        if same {
            unchanged.push(path.clone());
        } else {
            // Only a page to write creates the directory: an empty selection
            // leaves the tree as it was.
            std::fs::create_dir_all(&dir)
                .map_err(|e| Error::Io(format!("create {}: {e}", dir.display())))?;
            std::fs::write(&target, text).map_err(|e| Error::Io(format!("write {path}: {e}")))?;
            written.push(path.clone());
        }
    }
    for path in &export.remove {
        std::fs::remove_file(repo.join(path))
            .map_err(|e| Error::Io(format!("remove {path}: {e}")))?;
    }
    if json {
        let doc = serde_json::json!({
            "written": written,
            "unchanged": unchanged,
            "removed": export.remove,
        });
        let mut text =
            serde_json::to_string_pretty(&doc).map_err(|e| Error::Internal(e.to_string()))?;
        text.push('\n');
        out!("{text}");
    } else {
        // A page already current is left untouched (D-20), and said so.
        for (path, _) in &export.pages {
            if unchanged.contains(path) {
                outln!("unchanged {path}");
            } else {
                outln!("wrote {path}");
            }
        }
        for path in &export.remove {
            outln!("removed {path}");
        }
    }
    Ok(0)
}

/// Any finding exits 1 (spec 166 3.10).
fn check(repo: &Path, request_path: &Path, json: bool) -> Result<u8, Error> {
    let (cfg, request) = load(repo, request_path)?;
    let result = reference_check(&cfg, repo, &request)?;
    if result.findings.is_empty() {
        if json {
            out!("{}", reference_check_document(&result)?);
        } else {
            outln!("reference: {} pages are current", request.output_dir);
        }
        return Ok(0);
    }
    if json {
        let value = serde_json::to_value(&result).map_err(|e| Error::Internal(e.to_string()))?;
        crate::out::verdict(
            &Verdict::report(verb::REFERENCE_CHECK, 1, value).with_summary(format!(
                "reference.check: {} finding(s)",
                result.findings.len()
            )),
        )?;
    } else {
        for f in &result.findings {
            let kind = serde_json::to_value(f.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .ok_or_else(|| Error::Internal(format!("unnamed finding kind {:?}", f.kind)))?;
            let regions = if f.regions.is_empty() {
                String::new()
            } else {
                format!(" [{}]", f.regions.join(", "))
            };
            outln!(
                "{kind}: {}{regions}: {}",
                f.path.as_deref().unwrap_or("-"),
                f.detail
            );
        }
    }
    Ok(1)
}
