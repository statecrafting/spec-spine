// Spec: specs/109-impact-and-conflict-are-declared/spec.md
//! The impact/conflict read (spec 109 §3.6): a declaration lives in the
//! declaring spec, so the target spec says nothing about it. This inverts the
//! declarations, from either side, over a loaded [`Registry`]. Pure: reads
//! nothing but its arguments, and answers no question a compiled corpus has
//! not already recorded.

use serde::Serialize;
use spec_spine_types::{ConflictResolution, Error, ImpactNature, Registry, split_obligation_ref};

use crate::spec_id::resolve_spec_id;

/// One inverted impact entry (spec 109 §3.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactEntry {
    /// The declaring spec's full id.
    pub declared_by: String,
    /// The target, as a full qualified reference.
    pub target: String,
    pub nature: ImpactNature,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub successor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub target_withdrawn: bool,
}

/// One inverted conflict entry (spec 109 §3.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictEntry {
    pub declared_by: String,
    pub target: String,
    pub reason: String,
    pub resolution: ConflictResolution,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled_by: Option<String>,
    pub target_withdrawn: bool,
}

/// The inverted answer (spec 109 §3.6): both arrays sorted by `(target,
/// declaredBy)`, so the same corpus always yields the same bytes. Neither
/// array carries a fact this filter cannot prove: an empty result under a
/// filter that itself resolved is a true "nothing is declared", not the
/// refusal `target`/`declaredBy` failing to resolve produces instead (§3.6).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactSet {
    pub impacts: Vec<ImpactEntry>,
    pub conflicts: Vec<ConflictEntry>,
}

/// What `--target` names, once resolved (spec 109 §3.6): a spec id (every
/// obligation it declares) or one qualified obligation.
enum TargetFilter {
    Spec(String),
    Obligation(String, String),
}

/// Resolve the `--target` argument against `registry`.
///
/// `None` when no filter was given. A reference carrying `#` that
/// [`split_obligation_ref`] rejects (an empty or blank half, or a second `#`)
/// is [`Error::Usage`] (exit 3), never resolved against a spec; a target spec
/// or target obligation that does not exist is [`Error::NotFound`] (exit 1).
fn resolve_target(
    registry: &Registry,
    target: Option<&str>,
) -> Result<Option<TargetFilter>, Error> {
    let Some(arg) = target else {
        return Ok(None);
    };
    if arg.contains('#') {
        let (spec_part, ob_id) = split_obligation_ref(arg).ok_or_else(|| {
            Error::Usage(format!(
                "'{arg}' is not a qualified obligation reference: a qualified form \
                 <spec-id>#<obligation-id> is required, and an unqualified id is never \
                 resolved against a spec"
            ))
        })?;
        let spec = crate::query::show(registry, spec_part)?;
        if !spec.obligations.iter().any(|o| o.id == ob_id) {
            return Err(Error::NotFound(format!(
                "spec '{}' declares no obligation '{ob_id}'",
                spec.id
            )));
        }
        Ok(Some(TargetFilter::Obligation(
            spec.id.clone(),
            ob_id.to_string(),
        )))
    } else {
        let id = resolve_spec_id(arg, registry.specs.iter().map(|s| &s.id))?;
        Ok(Some(TargetFilter::Spec(id)))
    }
}

/// Whether `reference` (a full or partially-resolved qualified obligation
/// reference) matches `filter`. A reference [`split_obligation_ref`] cannot
/// parse never matches a filter: a corpus that refused it at compile
/// (`V-025`) has no such entry, and a hand-built registry handed to
/// `query_json` gets a silent non-match rather than a panic.
fn matches_target(reference: &str, filter: &TargetFilter) -> bool {
    let Some((spec_part, ob_id)) = split_obligation_ref(reference) else {
        return false;
    };
    match filter {
        TargetFilter::Spec(id) => spec_part == id,
        TargetFilter::Obligation(id, target_ob) => spec_part == id && ob_id == target_ob,
    }
}

/// Whether the obligation `reference` names is withdrawn, per the target
/// spec's own record. `false` when the reference does not parse, or its spec
/// or obligation is not in `registry`: this read never refuses on account of
/// a dangling reference (§3.4's V-028 is what would have refused it at
/// compile), so an unresolved target is reported as not-withdrawn rather than
/// failing the whole answer.
fn target_withdrawn(registry: &Registry, reference: &str) -> bool {
    let Some((spec_part, ob_id)) = split_obligation_ref(reference) else {
        return false;
    };
    registry
        .specs
        .iter()
        .find(|s| s.id == spec_part)
        .and_then(|s| s.obligations.iter().find(|o| o.id == ob_id))
        .is_some_and(|o| o.withdrawn)
}

/// Answer `registry impacts` (spec 109 §3.6): every declared impact and
/// conflict, optionally filtered by `target` and `declared_by`, both
/// resolved against `registry` and composed by intersection.
pub fn impacts(
    registry: &Registry,
    target: Option<&str>,
    declared_by: Option<&str>,
) -> Result<ImpactSet, Error> {
    let declared_by_id = match declared_by {
        Some(arg) => Some(resolve_spec_id(arg, registry.specs.iter().map(|s| &s.id))?),
        None => None,
    };
    let target_filter = resolve_target(registry, target)?;

    let mut impacts: Vec<ImpactEntry> = Vec::new();
    let mut conflicts: Vec<ConflictEntry> = Vec::new();
    for spec in &registry.specs {
        if let Some(id) = &declared_by_id
            && &spec.id != id
        {
            continue;
        }
        for imp in &spec.impacts {
            if let Some(filter) = &target_filter
                && !matches_target(&imp.obligation, filter)
            {
                continue;
            }
            impacts.push(ImpactEntry {
                declared_by: spec.id.clone(),
                target: imp.obligation.clone(),
                nature: imp.nature,
                successor: imp.successor.clone(),
                note: imp.note.clone(),
                target_withdrawn: target_withdrawn(registry, &imp.obligation),
            });
        }
        for c in &spec.conflicts {
            if let Some(filter) = &target_filter
                && !matches_target(&c.obligation, filter)
            {
                continue;
            }
            conflicts.push(ConflictEntry {
                declared_by: spec.id.clone(),
                target: c.obligation.clone(),
                reason: c.reason.clone(),
                resolution: c.resolution,
                settled_by: c.settled_by.clone(),
                target_withdrawn: target_withdrawn(registry, &c.obligation),
            });
        }
    }
    impacts.sort_by(|a, b| (&a.target, &a.declared_by).cmp(&(&b.target, &b.declared_by)));
    conflicts.sort_by(|a, b| (&a.target, &a.declared_by).cmp(&(&b.target, &b.declared_by)));
    Ok(ImpactSet { impacts, conflicts })
}
