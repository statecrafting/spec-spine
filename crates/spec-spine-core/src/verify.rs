//! Declared acceptance (spec 049): read a spec's `## Verification` section and
//! report the commands it declares.
//!
//! Ported from `scripts/verify-spec.sh`, the 78-line runner three adopters
//! wrote independently and spec 048 vendored into `kit/`. The grammar is
//! preserved (spec 049 §3.2 is the table this module is measured against); what
//! changes is where it lives. A parse of authored markdown belongs to the
//! compiler, and constitution II says a consumer reads its typed answer rather
//! than re-deriving it with `awk`.
//!
//! **This module never runs anything.** It returns a [`VerifyPlan`] and the CLI
//! executes it, which is the seam spec 005 already draws for `git`: the library
//! stays a pure function of `(config, file contents)` and stays usable from a
//! binding that has no shell. Deciding to execute code is a decision this layer
//! declines to make for a caller (spec 049 §3.1).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use spec_spine_types::{Config, Error, SkippedBlocks, VerifyPlan};

/// The fence tag whose lines are commands. Every other `verify:*` tag is
/// counted and declined.
const CLI_TAG: &str = "verify:cli";

/// Read `<specs_dir>/<id>/spec.md` and return the commands it declares.
///
/// `id` accepts the short form (spec 016): `049` resolves to `049-slug` when
/// exactly one directory carries that ordinal. A `spec.md` that does not exist,
/// or a short id matching none or several, is [`Error::NotFound`], which maps
/// to exit 1. Spec 049 §3.3 is explicit that it must not be exit 2: in this
/// tool 2 means stale, and the ported script's use of it for a bad id would
/// have made `verify` the one verb where the code meant something else.
pub fn plan(cfg: &Config, repo_root: &Path, id: &str) -> Result<VerifyPlan, Error> {
    let specs_dir = repo_root.join(&cfg.layout.specs_dir);
    let spec_id = resolve_spec_id(&specs_dir, id)?;
    let spec_md = specs_dir.join(&spec_id).join("spec.md");
    let raw = fs::read_to_string(&spec_md)
        .map_err(|e| Error::Io(format!("read {}: {e}", spec_md.display())))?;
    Ok(plan_from_markdown(&spec_id, &raw))
}

/// The whole grammar, as a pure function of the spec's markdown.
///
/// Split from [`plan`] so spec 049 §3.2's table is testable as fixtures over
/// strings, with no directory to arrange. The script this replaces had no tests
/// in any of the four repositories carrying it.
pub fn plan_from_markdown(spec_id: &str, markdown: &str) -> VerifyPlan {
    let section = verification_section(markdown);
    let mut commands = Vec::new();
    let mut skipped: BTreeMap<String, usize> = BTreeMap::new();

    // Fence state: `Some(tag)` while inside a block opened with that tag. A
    // bare ``` closes whatever is open, matching the script's `/^```/` arm.
    let mut open: Option<String> = None;
    for line in section.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("```") {
            match open.take() {
                // A closing fence: `rest` is ignored, as the script ignores it.
                Some(_) => {}
                None => {
                    let tag = rest.trim().to_string();
                    if tag != CLI_TAG && !tag.is_empty() {
                        *skipped.entry(tag.clone()).or_insert(0) += 1;
                    }
                    open = Some(tag);
                }
            }
            continue;
        }
        if open.as_deref() != Some(CLI_TAG) {
            continue;
        }
        // Blank lines and comments are not commands. The script tests the
        // first non-space character, so an indented `#` is still a comment.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        commands.push(trimmed.to_string());
    }

    VerifyPlan {
        spec_id: spec_id.to_string(),
        commands,
        skipped: skipped
            .into_iter()
            .map(|(tag, count)| SkippedBlocks { tag, count })
            .collect(),
    }
}

/// The body of the `## Verification` section: from the heading to the next `##`
/// heading, exclusive of both.
///
/// A numbered heading (`## 5. Verification`) is the same section, which is what
/// this corpus actually writes. Returns an empty string when there is no such
/// heading, which [`plan_from_markdown`] then reports as no commands, since
/// spec 049 §3.2 makes "no section" and "a section with no commands" one
/// outcome.
fn verification_section(markdown: &str) -> String {
    let mut out = String::new();
    let mut on = false;
    for line in markdown.lines() {
        if is_verification_heading(line) {
            on = true;
            continue;
        }
        if on && line.starts_with("## ") {
            break;
        }
        if on {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// `## Verification` or `## <n>. Verification`, with optional trailing space.
fn is_verification_heading(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("## ") else {
        return false;
    };
    let rest = rest.trim();
    // Strip a leading `<digits>. ` if present, then require the bare word.
    let rest = match rest.find(". ") {
        Some(i) if !rest[..i].is_empty() && rest[..i].chars().all(|c| c.is_ascii_digit()) => {
            rest[i + 2..].trim()
        }
        _ => rest,
    };
    rest == "Verification"
}

/// Exact id, else the unique directory whose ordinal matches (spec 016).
fn resolve_spec_id(specs_dir: &Path, id: &str) -> Result<String, Error> {
    if specs_dir.join(id).join("spec.md").is_file() {
        return Ok(id.to_string());
    }
    let entries = fs::read_dir(specs_dir).map_err(|e| {
        Error::Io(format!(
            "cannot read specs dir {}: {e}",
            specs_dir.display()
        ))
    })?;
    let mut matches: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| Error::Io(e.to_string()))?;
        if !entry.path().join("spec.md").is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.split('-').next() == Some(id) {
            matches.push(name);
        }
    }
    matches.sort();
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(Error::NotFound(format!("no such spec: {id}"))),
        n => Err(Error::NotFound(format!(
            "ambiguous spec id {id}: {n} specs share that ordinal ({})",
            matches.join(", ")
        ))),
    }
}
