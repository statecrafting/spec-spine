//! Declared acceptance (spec 043): read a spec's `## Verification` section and
//! report the commands it declares.
//!
//! Ported from `scripts/verify-spec.sh`, the 78-line runner three adopters
//! wrote independently and this repository vendored into `kit/`. Both copies
//! are gone: the kit's with the kit (spec 092), and this repository's on
//! 2026-09-21 once it had begun answering differently from the verb (spec 043
//! §3.9). They are in git history; nothing looks for them on disk. The grammar
//! is preserved (spec 043 §3.2 is the table this module is measured against);
//! what changed is where it lives. A parse of authored markdown belongs to the
//! compiler, and constitution II says a consumer reads its typed answer rather
//! than re-deriving it with `awk`.
//!
//! **This module never runs anything.** It returns a [`VerifyPlan`] and the CLI
//! executes it, which is the seam spec 005 already draws for `git`: the library
//! stays a pure function of `(config, file contents)` and stays usable from a
//! binding that has no shell. Deciding to execute code is a decision this layer
//! declines to make for a caller (spec 043 §3.1).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use spec_spine_types::{Config, Error, SkippedBlocks, VerifyPlan};

/// The fence tag whose lines are commands. Every other `verify:*` tag is
/// counted and declined.
const CLI_TAG: &str = "verify:cli";

/// Read `<specs_dir>/<id>/spec.md` and return the commands it declares.
///
/// `id` accepts the short form (spec 015): `049` resolves to `049-slug` when
/// exactly one directory carries that ordinal. A `spec.md` that does not exist,
/// or a short id matching none or several, is [`Error::NotFound`], which maps
/// to exit 1. Spec 043 §3.3 is explicit that it must not be exit 2: in this
/// tool 2 means stale, and the ported script's use of it for a bad id would
/// have made `verify` the one verb where the code meant something else.
pub fn plan(cfg: &Config, repo_root: &Path, id: &str) -> Result<VerifyPlan, Error> {
    let specs_dir = repo_root.join(&cfg.layout.specs_dir);
    // Spec 067 3.4: the one policy, over the ids this verb already reads.
    let spec_id = crate::spec_id::resolve_spec_id(id, crate::spec_id::spec_dir_ids(&specs_dir)?)?;

    // Spec 082 3.2: an amended acceptance is the one that runs. The block in
    // `<spec_id>/spec.md` is not read at all when another spec holds it, which
    // is the point: spec 037 forbids editing the amended file, so an acceptance
    // amendment that did not redirect the executor would change nothing about
    // what runs.
    let source = resolve_acceptance_source(&specs_dir, &spec_id)?;
    let read_from = source.as_deref().unwrap_or(&spec_id);
    let spec_md = specs_dir.join(read_from).join("spec.md");
    let raw = fs::read_to_string(&spec_md)
        .map_err(|e| Error::Io(format!("read {}: {e}", spec_md.display())))?;
    let mut plan = plan_from_markdown(&spec_id, &raw);
    plan.acceptance_from = source;
    Ok(plan)
}

/// The spec whose `## Verification` block answers for `spec_id`, when it is not
/// `spec_id` itself (spec 082 3.2).
///
/// Reads the corpus rather than the committed registry (D-6): `verify` must stay
/// runnable on a tree whose `.derived/` is stale or absent, since it is the verb
/// an operator reaches for while repairing one. The scan covers the same
/// directory `plan` already lists.
///
/// The chain is followed to its end, so a later amendment attaches to whichever
/// spec currently holds the acceptance. `compile` refuses a fork (`V-019`) and a
/// cycle (`V-020`); this function is defensive about a cycle anyway, because
/// `verify` can run against a corpus nobody has compiled.
fn resolve_acceptance_source(specs_dir: &Path, spec_id: &str) -> Result<Option<String>, Error> {
    let holders = acceptance_holders(specs_dir)?;
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut at = spec_id.to_string();
    while let Some(next) = holders.get(&at) {
        if !seen.insert(at.clone()) {
            // A cycle resolves to no block. Fall back to the spec's own rather
            // than looping; `compile` is where this is refused.
            return Ok(None);
        }
        at = next.clone();
    }
    Ok((at != spec_id).then_some(at))
}

/// `amended spec id -> the live spec that replaces its acceptance`.
///
/// A `superseded` or `retired` holder is skipped (spec 082 3.2, D-5): its
/// acceptance is no longer the corpus's, so the target keeps whatever held it
/// before. Ids are visited in sorted order, so a fork `compile` would refuse
/// resolves deterministically here rather than by directory-read order.
fn acceptance_holders(
    specs_dir: &Path,
) -> Result<std::collections::BTreeMap<String, String>, Error> {
    let mut out: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for id in crate::spec_id::spec_dir_ids(specs_dir)? {
        let path = specs_dir.join(&id).join("spec.md");
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(fm) = spec_spine_types::parse_frontmatter(&raw) else {
            continue;
        };
        if matches!(
            fm.status,
            spec_spine_types::Status::Superseded | spec_spine_types::Status::Retired
        ) {
            continue;
        }
        for target in &fm.amends_verification {
            out.entry(target.clone()).or_insert_with(|| id.clone());
        }
    }
    Ok(out)
}

/// The whole grammar, as a pure function of the spec's markdown.
///
/// Split from [`plan`] so spec 043 §3.2's table is testable as fixtures over
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
        acceptance_from: None,
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
/// spec 043 §3.2 makes "no section" and "a section with no commands" one
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

/// The markdown with its `## Verification` section removed: the exact
/// complement of what [`verification_section`] reads.
///
/// Spec 071 §3.3 classifies a change to a spec's body *outside* this section as
/// a `requirement` and a change to its plan as `verification`. Asking where the
/// section ends with a second copy of the heading grammar would let the two
/// classes disagree with the plan the moment either copy moved, so the question
/// is answered here, beside the one grammar.
pub fn without_verification_section(markdown: &str) -> String {
    let mut out = String::new();
    let mut on = false;
    let mut done = false;
    for line in markdown.lines() {
        if !done && is_verification_heading(line) {
            on = true;
            continue;
        }
        if on && line.starts_with("## ") {
            // `verification_section` stops reading here, so everything from
            // this heading on is outside it, a later `## Verification` included.
            on = false;
            done = true;
        }
        if !on {
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
