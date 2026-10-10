// Spec: specs/169-declared-obligation-traceability/spec.md
//! Declared obligation traceability (spec 169): the compile-time checks on an
//! authored relation and its canonical identities.
//!
//! Nothing here infers a relation. Ownership, names, paths, imports and prose
//! never create one (§3.5, §3.10); a relation exists only because an author
//! wrote it.

use std::collections::{BTreeMap, BTreeSet};

use spec_spine_types::{
    ContentSelector, Error, Frontmatter, InterfaceRole, Severity, SpecRecord, TraceDeclaration,
    TraceRelationKind, TraceTarget, Unit, Violation, split_obligation_ref, valid_obligation_id,
};

/// The local shape rules of §3.1, §3.2 and §4 that need no other spec.
pub const LOCAL_CODE: &str = "V-044";
/// A source obligation that is unqualified, dangling, ambiguous or withdrawn
/// (§3.1, §4). Corpus-wide.
pub const SOURCE_CODE: &str = "V-045";
/// A duplicate normalized `(obligation, relation, target)` tuple (§3.4).
/// Corpus-wide, since normalization resolves short ids against the corpus.
pub const DUPLICATE_CODE: &str = "V-046";

fn violation(code: &str, message: String, path: &str) -> Violation {
    Violation::new(code, Severity::Error, message).at_opt(Some(path.to_string()))
}

/// The qualified identity of one relation (§3.1).
pub fn relation_identity(declaring_spec: &str, id: &str) -> String {
    format!("{declaring_spec}#trace:{id}")
}

/// The canonical identity of one target (§3.4). `declaring_spec` names the
/// spec that declares the relation; only a cross-corpus interface uses it.
pub fn target_identity(target: &TraceTarget, declaring_spec: &str) -> Result<String, Error> {
    Ok(match target {
        TraceTarget::Unit { spec, unit } => {
            format!("unit:{spec}:{}", unit_json(unit)?)
        }
        TraceTarget::Test { selector } => format!(
            "test:{}",
            compact(&serde_json::json!({ "id": selector, "kind": "test" }))?
        ),
        TraceTarget::Invariant { obligation } => format!("invariant:{obligation}"),
        TraceTarget::Documentation { selector } => {
            format!("documentation:{}", selector_json(selector)?)
        }
        TraceTarget::Interface {
            role,
            spec,
            unit,
            corpus,
        } => match (unit, corpus) {
            (Some(unit), None) => {
                format!(
                    "interface:{}:local:{spec}:{}",
                    role.as_str(),
                    unit_json(unit)?
                )
            }
            (None, Some(corpus)) => format!(
                "interface:{}:external:{declaring_spec}:{corpus}:{spec}",
                role.as_str()
            ),
            // Refused at compile (§4); still given one stable spelling, so a
            // reader never panics on a registry it did not compile.
            _ => format!("interface:{}:invalid:{spec}", role.as_str()),
        },
    })
}

/// A unit's canonical JSON, with `planned` cleared: the flag annotates a
/// claim's state, never which territory it names (spec 063 §3.4).
fn unit_json(unit: &Unit) -> Result<String, Error> {
    compact(&unit.subject())
}

/// The canonical encoding §3.4 names: sorted object keys (serde_json's
/// default map is ordered, as [`crate::canonical_json`] relies on) and no
/// insignificant whitespace, so an identity is one line.
fn compact<T: serde::Serialize>(value: &T) -> Result<String, Error> {
    let value = serde_json::to_value(value).map_err(|e| Error::Internal(e.to_string()))?;
    serde_json::to_string(&value).map_err(|e| Error::Internal(e.to_string()))
}

/// A selector's canonical JSON without `required`, which is a request
/// concern of spec 155 and never part of a target's identity.
fn selector_json(selector: &ContentSelector) -> Result<String, Error> {
    let mut value = serde_json::to_value(selector).map_err(|e| Error::Internal(e.to_string()))?;
    if let Some(map) = value.as_object_mut() {
        map.remove("required");
    }
    compact(&value)
}

/// Rewrite every spec reference in one declaration to its full id (spec 015).
/// A dangling or ambiguous reference is left unchanged, so the check that
/// names it still fires.
pub(crate) fn normalize(decl: &mut TraceDeclaration, all_ids: &BTreeSet<String>) {
    let spec = |s: &str| crate::spec_id::resolve_spec_ref(s, all_ids);
    let obligation = |r: &str| match split_obligation_ref(r) {
        Some((s, id)) => format!("{}#{id}", spec(s)),
        None => r.to_string(),
    };
    decl.obligation = obligation(&decl.obligation);
    match &mut decl.target {
        TraceTarget::Unit { spec: s, .. } => *s = spec(s),
        TraceTarget::Test { .. } => {}
        TraceTarget::Invariant { obligation: o } => *o = obligation(o),
        TraceTarget::Documentation { selector } => match selector {
            ContentSelector::Spec { spec: s, .. }
            | ContentSelector::SpecSection { spec: s, .. }
            | ContentSelector::OwnedUnit { spec: s, .. } => *s = spec(s),
            ContentSelector::Obligation { obligation: o, .. } => *o = obligation(o),
            _ => {}
        },
        // A cross-corpus `spec` names another corpus's spec by its full id
        // (spec 110 §3.1), so only the local form normalizes.
        TraceTarget::Interface {
            spec: s,
            unit: Some(_),
            ..
        } => *s = spec(s),
        TraceTarget::Interface { .. } => {}
    }
}

/// §3.1, §3.2 and §4's single-spec rules (`V-044`). Malformed members, an
/// unknown relation or target kind, and a missing identity member were
/// already refused at parse (`V-002`).
pub(crate) fn validate_local(spec_path: &str, fm: &Frontmatter, out: &mut Vec<Violation>) {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for decl in &fm.traceability {
        let at = |m: String| violation(LOCAL_CODE, m, spec_path);
        if !valid_obligation_id(&decl.id) {
            out.push(at(format!(
                "traceability id '{}' is not a valid relation id \
                 (^[A-Za-z][A-Za-z0-9]*(-[A-Za-z0-9]+)*$)",
                decl.id
            )));
        } else if !seen.insert(decl.id.as_str()) {
            out.push(at(format!(
                "traceability id '{}' is declared twice: an id names one relation, and a \
                 withdrawn id is never reused",
                decl.id
            )));
        }
        if split_obligation_ref(&decl.obligation).is_none() {
            out.push(at(format!(
                "traceability '{}' names source '{}', which is not a qualified obligation \
                 reference (<spec-id>#<obligation-id>)",
                decl.id, decl.obligation
            )));
        }
        if !decl.target.allows(decl.relation) {
            out.push(at(format!(
                "traceability '{}' pairs relation '{}' with a '{}' target, which §3.2 does \
                 not allow",
                decl.id,
                decl.relation.as_str(),
                decl.target.kind_str()
            )));
        }
        match &decl.target {
            TraceTarget::Invariant { obligation } if split_obligation_ref(obligation).is_none() => {
                out.push(at(format!(
                    "traceability '{}' names invariant '{obligation}', which is not a \
                     qualified obligation reference",
                    decl.id
                )));
            }
            TraceTarget::Documentation {
                selector: ContentSelector::Test { .. },
            } => out.push(at(format!(
                "traceability '{}' names a test selector as documentation; a test is a \
                 'test' target",
                decl.id
            ))),
            TraceTarget::Test { selector } if selector.trim().is_empty() => out.push(at(format!(
                "traceability '{}' names an empty test selector",
                decl.id
            ))),
            TraceTarget::Interface {
                role,
                spec,
                unit,
                corpus,
            } => {
                let expected = match decl.relation {
                    TraceRelationKind::ProducedBy => Some(InterfaceRole::Producer),
                    TraceRelationKind::ConsumedBy => Some(InterfaceRole::Consumer),
                    _ => None,
                };
                if expected.is_some_and(|e| e != *role) {
                    out.push(at(format!(
                        "traceability '{}' is '{}' with interface role '{}': a producer is \
                         'produced-by' and a consumer is 'consumed-by'",
                        decl.id,
                        decl.relation.as_str(),
                        role.as_str()
                    )));
                }
                match (unit, corpus) {
                    (Some(_), Some(_)) | (None, None) => out.push(at(format!(
                        "traceability '{}' names an interface with {} of its local (`unit`) \
                         and cross-corpus (`corpus`) forms; exactly one is required",
                        decl.id,
                        if unit.is_some() { "both" } else { "neither" }
                    ))),
                    (None, Some(corpus)) => {
                        let n = fm
                            .interface_references
                            .iter()
                            .filter(|r| r.corpus == *corpus && r.spec == *spec)
                            .count();
                        if n != 1 {
                            out.push(at(format!(
                                "traceability '{}' names interface '{corpus}' / '{spec}', which \
                                 matches {n} of this spec's interface_references; exactly one \
                                 is required",
                                decl.id
                            )));
                        }
                    }
                    (Some(_), None) => {}
                }
            }
            _ => {}
        }
    }
}

/// The corpus-wide rules: a source obligation that does not resolve or is
/// withdrawn (`V-045`), and a duplicate normalized tuple (`V-046`). Over the
/// records, so the short ids are already resolved.
pub(crate) fn detect_cross_spec(records: &[SpecRecord], out: &mut Vec<Violation>) {
    let by_id: BTreeMap<&str, &SpecRecord> = records.iter().map(|r| (r.id.as_str(), r)).collect();
    for r in records {
        let mut tuples: BTreeSet<(String, TraceRelationKind, String)> = BTreeSet::new();
        for decl in &r.traceability {
            if let Some((spec, ob)) = split_obligation_ref(&decl.obligation) {
                let found = by_id
                    .get(spec)
                    .and_then(|t| t.obligations.iter().find(|o| o.id == ob));
                match found {
                    None => out.push(violation(
                        SOURCE_CODE,
                        format!(
                            "traceability '{}' source '{}' does not resolve to one obligation \
                             in this corpus",
                            decl.id, decl.obligation
                        ),
                        &r.spec_path,
                    )),
                    // A withdrawn relation keeps its identity even once its
                    // source is retired (§3.7); a live one may not cite it.
                    Some(o) if o.withdrawn && !decl.withdrawn => out.push(violation(
                        SOURCE_CODE,
                        format!(
                            "traceability '{}' source '{}' is withdrawn; withdraw the relation \
                             too",
                            decl.id, decl.obligation
                        ),
                        &r.spec_path,
                    )),
                    Some(_) => {}
                }
            }
            let Ok(target) = target_identity(&decl.target, &r.id) else {
                continue;
            };
            if !tuples.insert((decl.obligation.clone(), decl.relation, target.clone())) {
                out.push(violation(
                    DUPLICATE_CODE,
                    format!(
                        "traceability '{}' repeats ('{}', '{}', '{target}'), which spec '{}' \
                         already declares under another id",
                        decl.id,
                        decl.obligation,
                        decl.relation.as_str(),
                        r.id
                    ),
                    &r.spec_path,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §3.4 and D-4: identities sort object keys whatever the field order of
    /// the type that produced them. This holds because `preserve_order` is
    /// not enabled for this crate's `serde_json` (tree-sitter enables it only
    /// for its own build script, which resolver 2 keeps separate), and this
    /// test fails if a dependency change ever unifies it in.
    #[test]
    fn compact_sorts_keys_regardless_of_field_order() {
        #[derive(serde::Serialize)]
        struct Unsorted {
            zebra: u8,
            apple: u8,
            mango: u8,
        }
        assert_eq!(
            compact(&Unsorted {
                zebra: 1,
                apple: 2,
                mango: 3
            })
            .unwrap(),
            r#"{"apple":2,"mango":3,"zebra":1}"#
        );
        let selector = ContentSelector::DirectoryMember {
            directory: spec_spine_types::RepoPath::parse("docs").unwrap(),
            member: spec_spine_types::RepoPath::parse("docs/a.md").unwrap(),
            projection: None,
            required: true,
        };
        assert_eq!(
            selector_json(&selector).unwrap(),
            r#"{"directory":"docs","kind":"directory-member","member":"docs/a.md"}"#
        );
    }
}
