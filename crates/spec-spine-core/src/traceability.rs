// Spec: specs/169-declared-obligation-traceability/spec.md
//! Declared obligation traceability (spec 169): the compile-time checks on an
//! authored relation, its canonical identities, and the read that resolves
//! each declared target structurally.
//!
//! Nothing here infers a relation. Ownership, names, paths, imports and prose
//! never create one (§3.5, §3.10); a relation exists only because an author
//! wrote it, and a target resolves only through the closed checks below.
//! `resolved` never means executed, passed or accepted (§3.8).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use spec_spine_types::{
    CodebaseIndex, Config, ContentOmissionReason, ContentSelector, Error, Frontmatter,
    InterfaceRole, ObligationKind, Registry, Severity, SpecRecord, TRACEABILITY_SCHEMA_VERSION,
    TraceDeclaration, TraceRelation, TraceRelationKind, TraceState, TraceTarget,
    TraceabilityReport, TraceabilityRequest, Unit, Violation, parse_semver, split_obligation_ref,
    valid_obligation_id,
};

use crate::{Versioning, read_document};

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

/// The inputs a resolution may consult. `index` is `None` when no fresh
/// committed index is available; every target that needs one is then
/// `unknown`, never guessed.
struct Inputs<'a> {
    cfg: &'a Config,
    repo_root: &'a Path,
    registry: &'a Registry,
    index: Option<&'a CodebaseIndex>,
}

/// Resolve every declared relation in `registry` (§3.5, §3.6), filtered by
/// `request` (§3.9). Reads the working tree only through spec 155's
/// selector resolver; executes nothing.
pub fn traceability(
    cfg: &Config,
    repo_root: &Path,
    registry: &Registry,
    index: Option<&CodebaseIndex>,
    request: &TraceabilityRequest,
) -> Result<TraceabilityReport, Error> {
    let ids = registry.specs.iter().map(|s| s.id.as_str());
    let declared_by = match &request.declared_by {
        Some(raw) => Some(resolve_filter_spec(raw, ids.clone())?),
        None => None,
    };
    let obligation = match &request.obligation {
        Some(raw) => {
            let Some((spec, ob)) = split_obligation_ref(raw) else {
                return Err(Error::Usage(format!(
                    "--obligation '{raw}' is not a qualified <spec-id>#<obligation-id> reference"
                )));
            };
            Some(format!("{}#{ob}", resolve_filter_spec(spec, ids)?))
        }
        None => None,
    };

    let inputs = Inputs {
        cfg,
        repo_root,
        registry,
        index,
    };
    let mut relations = Vec::new();
    for record in &registry.specs {
        if declared_by.as_deref().is_some_and(|d| d != record.id) {
            continue;
        }
        for decl in &record.traceability {
            if obligation.as_deref().is_some_and(|o| o != decl.obligation) {
                continue;
            }
            let (state, detail) = resolve(&inputs, record, decl)?;
            if request.state.is_some_and(|s| s != state) {
                continue;
            }
            relations.push(TraceRelation {
                identity: relation_identity(&record.id, &decl.id),
                declared_by: record.id.clone(),
                id: decl.id.clone(),
                obligation: decl.obligation.clone(),
                relation: decl.relation,
                target: decl.target.clone(),
                target_identity: target_identity(&decl.target, &record.id)?,
                withdrawn: decl.withdrawn,
                state,
                detail,
            });
        }
    }
    relations.sort_by(|a, b| {
        (&a.declared_by, &a.id, a.state, &a.target_identity).cmp(&(
            &b.declared_by,
            &b.id,
            b.state,
            &b.target_identity,
        ))
    });
    let mut summary: BTreeMap<TraceState, usize> =
        TraceState::ALL.into_iter().map(|s| (s, 0)).collect();
    for r in &relations {
        *summary.entry(r.state).or_default() += 1;
    }
    Ok(TraceabilityReport {
        traceability_version: TRACEABILITY_SCHEMA_VERSION.to_string(),
        relations,
        summary,
    })
}

/// A filter's spec id, short or full; an unknown one is `NotFound`, an
/// ambiguous one refused, as every other registry read answers.
fn resolve_filter_spec<'a>(
    raw: &str,
    ids: impl IntoIterator<Item = &'a str>,
) -> Result<String, Error> {
    match crate::spec_id::match_spec_id(raw, ids) {
        crate::spec_id::SpecIdMatch::Resolved(full) => Ok(full.to_string()),
        crate::spec_id::SpecIdMatch::Ambiguous(candidates) => Err(Error::Refused(format!(
            "spec '{raw}' is ambiguous: {} specs share that ordinal ({})",
            candidates.len(),
            candidates.join(", ")
        ))),
        crate::spec_id::SpecIdMatch::NoMatch => {
            Err(Error::NotFound(format!("no spec '{raw}' in the registry")))
        }
    }
}

type Resolution = (TraceState, Option<String>);

/// One resolved location: a file and its optional inclusive line span.
type Located = (String, Option<(usize, usize)>);

fn resolve(
    inputs: &Inputs<'_>,
    record: &SpecRecord,
    decl: &TraceDeclaration,
) -> Result<Resolution, Error> {
    // §3.7: a withdrawn relation is never bound.
    if decl.withdrawn {
        return Ok((
            TraceState::Withdrawn,
            Some("the relation is withdrawn".into()),
        ));
    }
    // §3.5's first check: the source resolves and is not withdrawn. Compile
    // refuses both; a registry this read did not compile is still answered.
    match lookup_obligation(inputs.registry, &decl.obligation) {
        None => {
            return Ok((
                TraceState::Unresolved,
                Some(format!("source '{}' does not resolve", decl.obligation)),
            ));
        }
        Some(o) if o.withdrawn => {
            return Ok((
                TraceState::Withdrawn,
                Some(format!("source '{}' is withdrawn", decl.obligation)),
            ));
        }
        Some(_) => {}
    }
    Ok(match &decl.target {
        TraceTarget::Unit { spec, unit } => resolve_unit(inputs, spec, unit),
        TraceTarget::Interface {
            spec,
            unit: Some(unit),
            corpus: None,
            ..
        } => resolve_unit(inputs, spec, unit),
        TraceTarget::Interface {
            spec,
            unit: None,
            corpus: Some(corpus),
            ..
        } => {
            let n = record
                .interface_references
                .iter()
                .filter(|r| r.corpus == *corpus && r.spec == *spec)
                .count();
            match n {
                1 => (TraceState::Resolved, None),
                0 => (
                    TraceState::Unresolved,
                    Some(format!(
                        "'{}' declares no interface reference to '{corpus}' / '{spec}'",
                        record.id
                    )),
                ),
                n => (
                    TraceState::Ambiguous,
                    Some(format!(
                        "'{}' declares {n} interface references to '{corpus}' / '{spec}'",
                        record.id
                    )),
                ),
            }
        }
        TraceTarget::Interface { .. } => (
            TraceState::Unknown,
            Some("the interface names neither or both of its forms".into()),
        ),
        TraceTarget::Invariant { obligation } => {
            match lookup_obligation(inputs.registry, obligation) {
                None => (
                    TraceState::Unresolved,
                    Some(format!("invariant '{obligation}' does not resolve")),
                ),
                Some(o) if o.withdrawn => (
                    TraceState::Withdrawn,
                    Some(format!("invariant '{obligation}' is withdrawn")),
                ),
                Some(o) if o.kind != ObligationKind::Invariant => (
                    TraceState::Unresolved,
                    Some(format!(
                        "'{obligation}' resolves to a {} obligation, not an invariant",
                        kind_label(o.kind)
                    )),
                ),
                Some(_) => (TraceState::Resolved, None),
            }
        }
        // §3.3: a test target reuses spec 155's `test` selector unchanged.
        TraceTarget::Test { selector } => resolve_selector(
            inputs,
            &ContentSelector::Test {
                id: selector.clone(),
                projection: None,
                required: true,
            },
        )?,
        TraceTarget::Documentation { selector } => resolve_selector(inputs, selector)?,
    })
}

fn kind_label(kind: ObligationKind) -> &'static str {
    match kind {
        ObligationKind::Requirement => "requirement",
        ObligationKind::Invariant => "invariant",
        ObligationKind::Verification => "verification",
    }
}

fn lookup_obligation<'a>(
    registry: &'a Registry,
    reference: &str,
) -> Option<&'a spec_spine_types::Obligation> {
    let (spec, id) = split_obligation_ref(reference)?;
    registry
        .specs
        .iter()
        .find(|r| r.id == spec)?
        .obligations
        .iter()
        .find(|o| o.id == id)
}

/// A unit target binds when the named spec claims exactly that unit and the
/// index resolves it (§3.5). The claim is checked, and is never proof of
/// behavior: ownership answers only which spec, not what the unit does.
fn resolve_unit(inputs: &Inputs<'_>, spec: &str, unit: &Unit) -> Resolution {
    let Some(record) = inputs.registry.specs.iter().find(|r| r.id == spec) else {
        return (
            TraceState::Unresolved,
            Some(format!("no spec '{spec}' in the registry")),
        );
    };
    let Some(index) = inputs.index else {
        return (
            TraceState::Unknown,
            Some("no fresh committed codebase index is available".into()),
        );
    };
    let subject = unit.subject();
    let entries: Vec<&spec_spine_types::ResolvedUnit> = index
        .traceability
        .mappings
        .iter()
        .filter(|m| m.spec_id == record.id)
        .flat_map(|m| &m.resolved_units)
        .filter(|r| r.ownership && r.unit.subject() == subject)
        .collect();
    if entries.is_empty() {
        return (
            TraceState::Unresolved,
            Some(format!("'{}' does not claim this unit", record.id)),
        );
    }
    let bound: BTreeSet<Vec<Located>> = entries
        .iter()
        .filter(|r| !r.locations.is_empty())
        .map(|r| {
            r.locations
                .iter()
                .map(|l| (l.file.clone(), l.span.map(|s| (s.start_line, s.end_line))))
                .collect()
        })
        .collect();
    match bound.len() {
        0 => (
            TraceState::Unresolved,
            Some(if unit_is_planned(record, &subject) {
                format!(
                    "'{}' claims this unit as planned; it is not present",
                    record.id
                )
            } else {
                "the claimed unit does not resolve in the index".into()
            }),
        ),
        1 => (TraceState::Resolved, None),
        n => (
            TraceState::Ambiguous,
            Some(format!(
                "the claimed unit binds {n} distinct indexed targets"
            )),
        ),
    }
}

fn unit_is_planned(record: &SpecRecord, subject: &Unit) -> bool {
    record
        .establishes
        .iter()
        .chain(record.extends.iter().filter_map(|e| e.unit.as_ref()))
        .any(|u| u.is_planned() && u.subject() == *subject)
}

/// A selector target binds through spec 155's resolver (§3.5). Its omission
/// reasons map onto the closed states: missing is `unresolved`, a form or
/// projection outside the matrix (including every test form today, §3.3) is
/// `unsupported`, and a bound item larger than any request budget is still
/// bound.
fn resolve_selector(inputs: &Inputs<'_>, selector: &ContentSelector) -> Result<Resolution, Error> {
    let needs_index = matches!(
        selector,
        ContentSelector::OwnedUnit { .. }
            | ContentSelector::Symbol { .. }
            | ContentSelector::Module { .. }
    );
    // A build without the `symbol-resolution` feature (spec 025) resolves no
    // symbol or module: the form is outside this build's matrix, which is
    // `unsupported`, not a target found missing.
    #[cfg(not(feature = "symbol-resolution"))]
    if matches!(
        selector,
        ContentSelector::Symbol { .. } | ContentSelector::Module { .. }
    ) {
        return Ok((
            TraceState::Unsupported,
            Some("this build resolves no symbols or modules (symbol-resolution is off)".into()),
        ));
    }
    if needs_index && inputs.index.is_none() {
        return Ok((
            TraceState::Unknown,
            Some("no fresh committed codebase index is available".into()),
        ));
    }
    let bound = match crate::content::bind_selector(
        inputs.cfg,
        inputs.repo_root,
        inputs.registry,
        inputs.index,
        selector,
    ) {
        Ok(bound) => bound,
        // Spec 155 refuses an ambiguous structural selector rather than
        // choosing; here that is a state, not a refusal of the whole read.
        Err(Error::Refused(m))
            if matches!(
                selector,
                ContentSelector::Symbol { .. } | ContentSelector::Module { .. }
            ) || m.contains("is ambiguous") =>
        {
            return Ok((TraceState::Ambiguous, Some(m)));
        }
        Err(Error::Usage(m)) => return Ok((TraceState::Unresolved, Some(m))),
        Err(e) => return Err(e),
    };
    Ok(match bound {
        Ok(()) => (TraceState::Resolved, None),
        Err((ContentOmissionReason::MissingContent, m)) => (TraceState::Unresolved, Some(m)),
        Err((ContentOmissionReason::ItemExceedsByteBudget, _)) => (TraceState::Resolved, None),
        Err((
            ContentOmissionReason::UnsupportedSelector
            | ContentOmissionReason::UnsupportedProjection
            | ContentOmissionReason::BinaryContent,
            m,
        )) => (TraceState::Unsupported, Some(m)),
    })
}

/// JSON facade for [`traceability`]: loads the committed registry, and the
/// committed index when it is fresh, from `repo_root`. `request_json` is a
/// [`TraceabilityRequest`]; unknown members are refused.
pub fn traceability_json(
    config_json: &str,
    repo_root: &str,
    request_json: &str,
) -> Result<String, Error> {
    let cfg: Config = serde_json::from_str(config_json)
        .map_err(|e| Error::Usage(format!("invalid config JSON: {e}")))?;
    let request: TraceabilityRequest = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid traceability request: {e}")))?;
    let root = Path::new(repo_root);
    let registry = crate::load_committed_registry(&cfg, root)?;
    let index = committed_index_if_fresh(&cfg, root)?;
    let report = traceability(&cfg, root, &registry, index.as_ref(), &request)?;
    read_document(&report, Versioning::Stamp)
}

/// The committed index, or `None` when it is stale or absent, so a unit
/// target reads `unknown` rather than binding against a stale tree. Any other
/// failure keeps its exit contract.
pub fn committed_index_if_fresh(
    cfg: &Config,
    repo_root: &Path,
) -> Result<Option<CodebaseIndex>, Error> {
    if !crate::index::index_dir(cfg, repo_root).exists() {
        return Ok(None);
    }
    match crate::guard_committed_index(cfg, repo_root) {
        Ok(()) => Ok(Some(crate::load_committed_index(cfg, repo_root)?)),
        Err(Error::Stale { .. } | Error::NotFound(_) | Error::Validation(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Read a traceability document (§3.9): accept any MINOR of the supported
/// MAJOR, and refuse another MAJOR. An unknown relation, target kind or
/// state is a parse failure, never downcast.
pub fn parse_traceability_report(json: &str) -> Result<TraceabilityReport, Error> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| Error::Parse(format!("traceability: {e}")))?;
    let version = value
        .get("traceabilityVersion")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Parse("traceability: no traceabilityVersion".into()))?;
    let (major, _, _) = parse_semver(version)
        .ok_or_else(|| Error::Parse(format!("traceability: version '{version}'")))?;
    let (supported, _, _) = parse_semver(TRACEABILITY_SCHEMA_VERSION).expect("valid const");
    if major != supported {
        return Err(Error::Refused(format!(
            "traceability document is version {version}; this reader supports {supported}.x"
        )));
    }
    serde_json::from_value(value).map_err(|e| Error::Parse(format!("traceability: {e}")))
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
