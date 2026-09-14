//! The one spec-id resolution policy (spec 084 §3.4).
//!
//! Spec 016 §3.1 defines what a short id is: an exact id first, otherwise the
//! one id whose **whole leading dash-segment** equals the reference. `070`
//! resolves `070-a-slug`; `70` resolves nothing; `070-typo` resolves nothing
//! rather than snapping to a neighbour.
//!
//! Before 084 that rule existed in four private copies (084 §1.1) which
//! disagreed about their refusal messages, and the two strict ones tested the
//! exact case by joining the argument onto a path, so `verify ../specs/<id>`
//! resolved through the filesystem and `compile --spec ./<id>` reported a false
//! `V-001` on a valid spec (084 §1.2). Everything here is **string comparison
//! over a set of ids**: an argument is never joined onto a path before it has
//! resolved, which is what closes that (084 §3.2, D-6).
//!
//! The module belongs to neither 001 nor 004. Spec 016 §2 used a mirror rather
//! than a shared call to keep the compile gate from taking a code dependency on
//! the indexer's file; that reason survives here, because both now depend on a
//! third file instead of on each other (084 D-1).
//!
//! The **set** the policy runs over is per verb, and is whatever that verb
//! reads anyway (084 §3.2, D-3). This module supplies only the one every
//! filesystem caller needs, [`spec_dir_ids`]; a verb reading the committed
//! registry, an in-memory compile or a directory of attestations passes its own.

use std::path::Path;

use spec_spine_types::Error;

/// What an argument matched in a set of spec ids (spec 084 §3.1, steps 1-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecIdMatch {
    /// Steps 1 and 2: an exact id, or the one id whose leading dash-segment
    /// equals the argument.
    Resolved(String),
    /// Step 3: several ids share that segment. The candidates, sorted. Never
    /// guessed, at any verb.
    Ambiguous(Vec<String>),
    /// Step 4: nothing matched.
    NoMatch,
}

/// Apply spec 016 §3.1 to any set of ids.
///
/// An exact match wins over a segment match regardless of iteration order, so
/// a corpus holding both `070` and `070-slug` resolves `070` to itself rather
/// than calling it ambiguous. Candidates are sorted and deduplicated, so
/// [`SpecIdMatch::Ambiguous`] carries the same list whatever order the caller's
/// set iterates in: the refusal is part of the contract (084 §3.1) and cannot
/// depend on a `read_dir` order.
pub fn match_spec_id<I, S>(arg: &str, ids: I) -> SpecIdMatch
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut exact = false;
    let mut candidates: Vec<String> = Vec::new();
    for id in ids {
        let id = id.as_ref();
        if id == arg {
            exact = true;
        } else if id.split('-').next() == Some(arg) {
            candidates.push(id.to_string());
        }
    }
    if exact {
        return SpecIdMatch::Resolved(arg.to_string());
    }
    candidates.sort();
    candidates.dedup();
    match candidates.len() {
        0 => SpecIdMatch::NoMatch,
        1 => SpecIdMatch::Resolved(candidates.remove(0)),
        _ => SpecIdMatch::Ambiguous(candidates),
    }
}

/// The strict form: ambiguous and no match become the one [`Error::NotFound`]
/// of spec 084 §3.1, which is exit 1.
///
/// Both messages are fixed here and nowhere else. Spec 084 §3.1 requires that
/// for the same argument and the same candidates a reader cannot tell from the
/// refusal which verb produced it, and a message assembled at the call site is
/// exactly how the four copies drifted apart.
pub fn resolve_spec_id<I, S>(arg: &str, ids: I) -> Result<String, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    match match_spec_id(arg, ids) {
        SpecIdMatch::Resolved(id) => Ok(id),
        SpecIdMatch::Ambiguous(candidates) => Err(ambiguous(arg, &candidates)),
        SpecIdMatch::NoMatch => Err(no_match(arg)),
    }
}

/// The lenient form: a reference that resolves to nothing, or to several, keeps
/// the raw string.
///
/// This is what `compile.rs::resolve_spec_ref` and `index.rs::resolve_id` are,
/// and their contract is unchanged: `V-008` and `V-010` still see the raw
/// string and still name a dangling reference. Frontmatter is validated, not
/// refused at an argument boundary.
pub fn resolve_spec_ref<I, S>(arg: &str, ids: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    match match_spec_id(arg, ids) {
        SpecIdMatch::Resolved(id) => id,
        SpecIdMatch::Ambiguous(_) | SpecIdMatch::NoMatch => arg.to_string(),
    }
}

/// The refusal for step 3, shared by every argument that resolves.
pub fn ambiguous(arg: &str, candidates: &[String]) -> Error {
    Error::NotFound(format!(
        "spec '{arg}' is ambiguous: {} specs share that ordinal ({})",
        candidates.len(),
        candidates.join(", ")
    ))
}

/// The refusal for step 4, shared by the five arguments that refuse it.
///
/// `verify-attestation` is the exception and does not call this: its argument
/// falls through unresolved so a missing attestation file stays the exit 3 that
/// spec 042 §3.5 assigns to I/O (084 §3.2, D-4).
pub fn no_match(arg: &str) -> Error {
    Error::NotFound(format!("spec '{arg}'"))
}

/// The ids of the spec directories under `specs_dir`: every entry holding a
/// `spec.md`, sorted.
///
/// The set for `compile --spec` and `verify`, which resolve against the
/// filesystem because a draft that has never compiled has no shard and spec 056
/// §3.1 exists for exactly that draft. It lives here so the two callers stop
/// carrying a `read_dir` loop each (084 §3.4).
pub fn spec_dir_ids(specs_dir: &Path) -> Result<Vec<String>, Error> {
    let entries = std::fs::read_dir(specs_dir).map_err(|e| {
        Error::Io(format!(
            "cannot read specs dir {}: {e}",
            specs_dir.display()
        ))
    })?;
    let mut ids: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| Error::Io(e.to_string()))?;
        if !entry.path().join("spec.md").is_file() {
            continue;
        }
        ids.push(entry.file_name().to_string_lossy().into_owned());
    }
    ids.sort();
    Ok(ids)
}
