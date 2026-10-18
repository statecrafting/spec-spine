//! Digest-guarded edit plans (spec 165).
//!
//! A plan answers, before anything is written, what a governed edit would be:
//! whether each target is still the bytes its caller read, what authority the
//! edit touches, why it is refused, and, when nothing is, the exact
//! post-image bytes of every file it changes. Five operations, closed. The
//! planner reads one repository export and the committed ledger; it never
//! writes, formats, executes or applies (3.16). There is no apply function.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};
use spec_spine_types::{
    AuthorityEffect, ClaimEdge, CodebaseIndex, Config, ConflictKind, ContentProjection,
    ContentSelector, ContentSnapshot, ContextPacket, DerivedTree, DocumentationManifest,
    EDIT_PLAN_MAX_BYTES, EDIT_PLAN_MAX_OPERATIONS, EDIT_PLAN_SCHEMA_VERSION, EXPECTED_ABSENT,
    EditOperation, EditPlan, EditPlanRequest, Error, FilePreview, FindingSeverity, Implementation,
    PacketSnapshot, PlanConflict, PlanRefusal, PlanStatus, PlannedOperation, PredictedFinding,
    RefusalReason, RegionKind, Registry, Severity, SpecRecord, Status, TraceDeclaration,
    TraceTarget, TraceabilityRequest, Unit, parse_semver, split_obligation_ref,
};

use crate::content::{content_item, resolve_selectors, validate_snapshot};
use crate::scope::ScopeRequest;
use crate::{Versioning, canonical_json, hash, read_document};

/// The heading forms a decisions section takes in this corpus (3.8).
const DECISION_HEADINGS: [&str; 3] = ["resolved decisions", "design decisions", "decisions"];

/// One operation's target, resolved: its identity, the file it lives in, and
/// the pre-image span it touches.
#[derive(Clone, Debug)]
struct Located {
    identity: String,
    path: String,
    span: Span,
}

/// What an operation's pre-image span is (3.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Span {
    /// The frontmatter block, from the opening fence through the closing one.
    Frontmatter(usize, usize),
    /// A resolved unit's lines, 1-based inclusive.
    Lines(usize, usize),
    /// The whole file.
    Whole,
}

impl Span {
    fn range(self) -> Option<(usize, usize)> {
        match self {
            Span::Frontmatter(a, b) | Span::Lines(a, b) => Some((a, b)),
            Span::Whole => None,
        }
    }
}

/// The edit an accepted operation makes to its file.
#[derive(Clone, Debug)]
enum Edit {
    /// One flow line appended under a frontmatter key.
    Frontmatter { key: &'static str, line: String },
    /// Lines `start..=end` replaced by `content`.
    Replace {
        start: usize,
        end: usize,
        content: String,
    },
    /// The whole file replaced (or created).
    Whole { content: String },
}

/// One operation as checked.
struct Checked {
    planned: PlannedOperation,
    located: Option<Located>,
    edit: Option<Edit>,
    refusals: Vec<(RefusalReason, String)>,
    conflicts: Vec<(ConflictKind, Vec<String>)>,
    /// For `propose-amendment`: the amended specs' files.
    amended_files: Vec<String>,
}

/// The planner's read-only view of the snapshot.
struct View<'a> {
    cfg: &'a Config,
    root: &'a Path,
    snapshot: &'a ContentSnapshot,
    registry: &'a Registry,
    index: &'a CodebaseIndex,
    subject: &'a SpecRecord,
    scope: &'a ScopeRequest,
    manifests: &'a [DocumentationManifest],
}

/// Plan the request's operations against the export at `repo_root` (spec
/// 165). Returns a `ready` plan with exact previews, or a `refused` plan
/// listing every refusal of every operation and no preview; a refused plan is
/// a finding, not an error. A malformed request is [`Error::Usage`]; a scope
/// whose subject does not resolve is [`Error::NotFound`].
pub fn edit_plan(
    cfg: &Config,
    repo_root: &Path,
    request: &EditPlanRequest,
    snapshot: &ContentSnapshot,
) -> Result<EditPlan, Error> {
    validate_snapshot(snapshot)?;
    check_consumer_schema(&request.consumer_schema_version)?;
    if request.operations.is_empty() {
        return Err(Error::Usage(
            "an edit plan names at least one operation".into(),
        ));
    }
    let scope: ScopeRequest = serde_json::from_value(request.scope.clone())
        .map_err(|e| Error::Usage(format!("invalid work scope: {e}")))?;
    let manifests = request
        .manifests
        .iter()
        .enumerate()
        .map(|(i, m)| {
            serde_json::from_value::<DocumentationManifest>(m.clone())
                .map_err(|e| Error::Usage(format!("manifest {}: {e}", i + 1)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let request_digest = digest_of(request)?;
    let packet_snapshot = PacketSnapshot {
        repository: snapshot.repository.clone(),
        revision: snapshot.revision.clone(),
        tree: snapshot.tree.clone(),
        dirty_state: snapshot.dirty_state,
    };
    let refused_whole = |subject: String, reason, detail: String| {
        finish(EditPlan {
            schema_version: EDIT_PLAN_SCHEMA_VERSION.to_string(),
            status: PlanStatus::Refused,
            snapshot: packet_snapshot.clone(),
            subject,
            request_digest: request_digest.clone(),
            authority_effect: AuthorityEffect::None,
            operations: Vec::new(),
            refusals: vec![PlanRefusal {
                index: None,
                target: None,
                reason,
                detail,
            }],
            conflicts: Vec::new(),
            previews: Vec::new(),
            predicted_findings: Vec::new(),
            stales: Vec::new(),
            plan_digest: String::new(),
        })
    };

    // 3.1: ownership read from a stale ledger is no answer.
    let stale = match crate::compile::check_registry_freshness(cfg, repo_root)? {
        crate::index::Freshness::Stale { .. } => Some("the committed registry is stale"),
        crate::index::Freshness::Fresh => None,
    }
    .or_else(|| {
        crate::index::guard_committed_index(cfg, repo_root)
            .err()
            .map(|_| "the committed index is stale")
    });
    if let Some(why) = stale {
        return refused_whole(
            scope.own_spec.clone(),
            RefusalReason::StaleLedger,
            format!("stale-ledger: {why} for this snapshot"),
        );
    }
    if request.operations.len() > EDIT_PLAN_MAX_OPERATIONS {
        return refused_whole(
            scope.own_spec.clone(),
            RefusalReason::PlanTooLarge,
            format!(
                "plan-too-large: {} operations, above the {EDIT_PLAN_MAX_OPERATIONS}-operation limit",
                request.operations.len()
            ),
        );
    }

    let registry = crate::compile::load_committed_registry(cfg, repo_root)?;
    let index = crate::index::load_committed_index(cfg, repo_root)?;
    let evaluated = crate::scope::evaluate_scope(cfg, &registry, &index, &scope)?;
    let subject_id = evaluated.own_spec.clone();
    let subject = registry
        .specs
        .iter()
        .find(|r| r.id == subject_id)
        .ok_or_else(|| Error::NotFound(format!("ownSpec '{subject_id}' is not in the registry")))?;
    let view = View {
        cfg,
        root: repo_root,
        snapshot,
        registry: &registry,
        index: &index,
        subject,
        scope: &scope,
        manifests: &manifests,
    };

    // Each operation on its own, in canonical order (3.11).
    let mut order: Vec<usize> = (0..request.operations.len()).collect();
    let keys: Vec<(String, String, String)> = request
        .operations
        .iter()
        .map(|op| {
            Ok((
                canonical_target_path(op, &view),
                op.kind().to_string(),
                canonical_json::to_string(op)?,
            ))
        })
        .collect::<Result<_, Error>>()?;
    order.sort_by(|a, b| {
        let (ka, kb) = (&keys[*a], &keys[*b]);
        ka.0.as_bytes()
            .cmp(kb.0.as_bytes())
            .then_with(|| ka.1.cmp(&kb.1))
            .then_with(|| ka.2.as_bytes().cmp(kb.2.as_bytes()))
            .then_with(|| a.cmp(b))
    });
    let mut checked: Vec<Checked> = Vec::with_capacity(order.len());
    for &i in &order {
        checked.push(check_operation(&view, i, &request.operations[i])?);
    }
    overlaps(&mut checked);
    repeated_declarations(&mut checked);

    // 3.12: when every check passes, compose and compile the post-images.
    let mut previews = Vec::new();
    let mut findings = Vec::new();
    let mut stales = BTreeSet::new();
    if checked.iter().all(|c| c.refusals.is_empty()) {
        let images = compose(&view, &checked)?;
        for (path, image) in &images {
            if is_spec_file(cfg, path) {
                let dirname = path
                    .trim_start_matches(&format!("{}/", cfg.layout.specs_dir.trim_end_matches('/')))
                    .trim_end_matches("/spec.md")
                    .to_string();
                let committed: Vec<SpecRecord> = registry
                    .specs
                    .iter()
                    .filter(|r| &r.spec_path != path)
                    .cloned()
                    .collect();
                let (report, _) = crate::compile::check_spec_source(
                    cfg,
                    committed,
                    dirname,
                    path.clone(),
                    &image.after,
                );
                for v in &report.violations {
                    if v.severity == Severity::Error {
                        for c in checked
                            .iter_mut()
                            .filter(|c| c.located.as_ref().is_some_and(|l| &l.path == path))
                        {
                            c.refusals.push((
                                RefusalReason::PostImageInvalid,
                                format!("post-image-invalid: {path}: {} {}", v.code, v.message),
                            ));
                        }
                    } else {
                        findings.push(PredictedFinding {
                            path: path.clone(),
                            code: v.code.clone(),
                            severity: if v.severity == Severity::Warning {
                                FindingSeverity::Warning
                            } else {
                                FindingSeverity::Info
                            },
                            message: v.message.clone(),
                        });
                    }
                }
                stales.insert(DerivedTree::SpecRegistry);
            }
            stales.insert(DerivedTree::CodebaseIndex);
        }
        if checked.iter().all(|c| c.refusals.is_empty()) {
            predict_trace_states(&view, &mut checked, &images)?;
            for (path, image) in images {
                previews.push(FilePreview {
                    before_raw_digest: image
                        .before_raw
                        .as_deref()
                        .map(sha256_bytes)
                        .unwrap_or_else(|| EXPECTED_ABSENT.to_string()),
                    after_digest: sha256_bytes(image.after.as_bytes()),
                    diff: unified_diff(&path, image.before.as_deref().unwrap_or(""), &image.after),
                    after_content: image.after,
                    path,
                });
            }
        }
    }

    let refused = checked.iter().any(|c| !c.refusals.is_empty());
    let mut operations = Vec::new();
    let mut refusals = Vec::new();
    let mut conflicts = Vec::new();
    for c in &checked {
        for (reason, detail) in &c.refusals {
            refusals.push(PlanRefusal {
                index: Some(c.planned.index),
                target: Some(c.planned.target.clone()),
                reason: *reason,
                detail: detail.clone(),
            });
        }
        for (kind, specs) in &c.conflicts {
            conflicts.push(PlanConflict {
                kind: *kind,
                index: c.planned.index,
                target: c.planned.target.clone(),
                specs: specs.clone(),
            });
        }
        operations.push(c.planned.clone());
    }
    refusals.sort_by(|a, b| (a.index, a.reason, &a.detail).cmp(&(b.index, b.reason, &b.detail)));
    refusals.dedup();
    conflicts.sort_by(|a, b| (a.index, a.kind, &a.specs).cmp(&(b.index, b.kind, &b.specs)));
    findings.sort_by(|a, b| (&a.path, &a.code, &a.message).cmp(&(&b.path, &b.code, &b.message)));
    let authority_effect = operations
        .iter()
        .map(|o| o.authority_effect)
        .max()
        .unwrap_or(AuthorityEffect::None);
    let plan = EditPlan {
        schema_version: EDIT_PLAN_SCHEMA_VERSION.to_string(),
        status: if refused {
            PlanStatus::Refused
        } else {
            PlanStatus::Ready
        },
        snapshot: packet_snapshot.clone(),
        subject: subject_id.clone(),
        request_digest: request_digest.clone(),
        authority_effect,
        operations,
        refusals,
        conflicts,
        previews: if refused { Vec::new() } else { previews },
        predicted_findings: if refused { Vec::new() } else { findings },
        stales: if refused {
            Vec::new()
        } else {
            stales.into_iter().collect()
        },
        plan_digest: String::new(),
    };
    let plan = finish(plan)?;
    if edit_plan_document(&plan)?.len() > EDIT_PLAN_MAX_BYTES {
        return refused_whole(
            subject_id,
            RefusalReason::PlanTooLarge,
            format!("plan-too-large: the plan exceeds {EDIT_PLAN_MAX_BYTES} bytes"),
        );
    }
    Ok(plan)
}

/// Write a plan as its canonical document: sorted keys, two-space indent, LF
/// and one trailing newline.
pub fn edit_plan_document(plan: &EditPlan) -> Result<String, Error> {
    read_document(plan, Versioning::Preexisting("schemaVersion"))
}

fn finish(mut plan: EditPlan) -> Result<EditPlan, Error> {
    let mut value = serde_json::to_value(&plan).map_err(|e| Error::Internal(e.to_string()))?;
    if let Some(map) = value.as_object_mut() {
        map.remove("planDigest");
    }
    plan.plan_digest = digest_of(&value)?;
    Ok(plan)
}

fn check_consumer_schema(version: &str) -> Result<(), Error> {
    let v = parse_semver(version).ok_or_else(|| {
        Error::Usage(format!(
            "consumerSchemaVersion '{version}' is not MAJOR.MINOR.PATCH"
        ))
    })?;
    let ours = parse_semver(EDIT_PLAN_SCHEMA_VERSION).expect("own version parses");
    if v.0 != ours.0 || v.1 > ours.1 {
        return Err(Error::Usage(format!(
            "unsupported-schema: consumer schema {version} is not served by edit-plan schema \
             {EDIT_PLAN_SCHEMA_VERSION}"
        )));
    }
    Ok(())
}

/// The path an operation's target lives at, for canonical ordering only.
fn canonical_target_path(op: &EditOperation, view: &View<'_>) -> String {
    match op {
        EditOperation::ProposeAmendment { target, .. }
        | EditOperation::UpdateManifest { target, .. } => target.as_str().to_string(),
        EditOperation::AddClaim { target, .. }
        | EditOperation::AddTraceRelation { target, .. }
        | EditOperation::ReplaceUnit { target, .. } => match target {
            ContentSelector::Spec { spec, .. } | ContentSelector::SpecSection { spec, .. } => {
                record_of(view.registry, spec)
                    .map(|r| r.spec_path.clone())
                    .unwrap_or_else(|| spec.clone())
            }
            ContentSelector::File { path, .. } => path.as_str().to_string(),
            other => canonical_json::to_string(other).unwrap_or_default(),
        },
    }
}

fn record_of<'a>(registry: &'a Registry, spec: &str) -> Option<&'a SpecRecord> {
    let ids = registry.specs.iter().map(|s| s.id.as_str());
    let id = crate::spec_id::resolve_spec_id(spec, ids).ok()?;
    registry.specs.iter().find(|r| r.id == id)
}

/// Check one operation: its target and digest guard, its kind's rules, and
/// authority over every file it changes (3.4 to 3.10, 3.13).
fn check_operation(view: &View<'_>, index: usize, op: &EditOperation) -> Result<Checked, Error> {
    let mut c = Checked {
        planned: PlannedOperation {
            index,
            op: op.kind().to_string(),
            target: String::new(),
            expected: op.expected().to_string(),
            authority_effect: AuthorityEffect::None,
            predicted_state: None,
        },
        located: None,
        edit: None,
        refusals: Vec::new(),
        conflicts: Vec::new(),
        amended_files: Vec::new(),
    };
    match op {
        EditOperation::AddClaim {
            target,
            expected,
            edge,
            unit,
            spec,
            nature,
        } => {
            let Some(file) = subject_target(view, target, expected, &mut c)? else {
                return Ok(c);
            };
            c.planned.authority_effect = if view.subject.status == Status::Approved
                && view.subject.implementation == Some(Implementation::Complete)
            {
                AuthorityEffect::RequiresHumanRatification
            } else {
                AuthorityEffect::ClaimsTerritory
            };
            check_claim(
                view,
                &file,
                *edge,
                unit,
                spec.as_deref(),
                nature.as_deref(),
                &mut c,
            )?;
        }
        EditOperation::AddTraceRelation {
            target,
            expected,
            entry,
        } => {
            let Some(file) = subject_target(view, target, expected, &mut c)? else {
                return Ok(c);
            };
            c.planned.authority_effect = if view.subject.status == Status::Approved {
                AuthorityEffect::RequiresHumanRatification
            } else {
                AuthorityEffect::None
            };
            check_trace(view, &file, entry, &mut c)?;
        }
        EditOperation::ProposeAmendment {
            target,
            expected,
            spec,
            obligations,
            content,
        } => {
            c.planned.authority_effect = AuthorityEffect::RequiresHumanRatification;
            check_amendment(
                view,
                target.as_str(),
                expected,
                spec,
                obligations,
                content,
                &mut c,
            )?;
        }
        EditOperation::UpdateManifest {
            target,
            expected,
            manifest,
            pages,
        } => check_manifest(view, target.as_str(), expected, manifest, pages, &mut c)?,
        EditOperation::ReplaceUnit {
            target,
            expected,
            content,
        } => check_replace(view, target, expected, content, &mut c)?,
    }
    if let Some(located) = c.located.clone() {
        let created = matches!(op, EditOperation::ProposeAmendment { .. });
        if let Some((reason, detail)) = authority(view, &located, created) {
            c.refusals.push((reason, detail));
        }
    }
    Ok(c)
}

/// The metadata of a path in the export, or `None` when it does not exist.
/// Every component is checked, not only the last: a linked parent directory
/// would otherwise carry the read outside the root, so a link anywhere on the
/// path is refused.
fn contained_metadata(
    view: &View<'_>,
    path: &str,
) -> Result<Option<(std::path::PathBuf, fs::Metadata)>, Error> {
    let rel = spec_spine_types::RepoPath::parse(path).map_err(Error::Usage)?;
    let mut full = view.root.to_path_buf();
    let mut parts = rel.as_str().split('/').peekable();
    while let Some(part) = parts.next() {
        full.push(part);
        let last = parts.peek().is_none();
        match fs::symlink_metadata(&full) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(Error::Io(format!("read {path}: {e}"))),
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(Error::Refused(if last {
                    format!("containment: '{path}' is a symbolic link")
                } else {
                    format!("containment: '{path}' is reached through a symbolic link")
                }));
            }
            Ok(meta) if last => return Ok(Some((full, meta))),
            Ok(_) => {}
        }
    }
    Ok(None)
}

/// The raw bytes of a file in the export, or `None` when it does not exist.
fn read_raw(view: &View<'_>, path: &str) -> Result<Option<Vec<u8>>, Error> {
    match contained_metadata(view, path)? {
        Some((full, meta)) if meta.is_file() => fs::read(&full)
            .map(Some)
            .map_err(|e| Error::Io(format!("read {path}: {e}"))),
        _ => Ok(None),
    }
}

/// 3.9: a target whose raw bytes differ from their normalization cannot have
/// exact post-image bytes that are also faithful.
fn encoding_ok(raw: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(raw).ok()?;
    (hash::normalize(text) == text).then(|| text.to_string())
}

/// One selector resolved at its `full` projection.
struct Resolved {
    identity: String,
    path: String,
    span: (usize, usize),
    digest: String,
}

/// The spec 155 item digest of `selector` at its `full` projection, and the
/// resolved path and span; `Err` carries the omission reason and detail.
fn item_of(
    view: &View<'_>,
    selector: &ContentSelector,
) -> Result<Result<Resolved, (RefusalReason, String)>, Error> {
    let full = with_projection(selector, ContentProjection::Full);
    let mut resolved = resolve_selectors(
        view.cfg,
        view.root,
        std::slice::from_ref(&full),
        ContentProjection::Full,
        usize::MAX / 4,
    )?;
    let r = resolved.pop().expect("one resolution per selector");
    Ok(match r.outcome {
        Ok(selected) => {
            let item = content_item(
                &r.identity,
                r.projection,
                &r.selector,
                &selected,
                view.snapshot,
            );
            Ok(Resolved {
                identity: r.identity,
                path: selected.path.as_str().to_string(),
                span: (selected.span.start_line, selected.span.end_line),
                digest: item.digest,
            })
        }
        Err((reason, detail)) => {
            let refusal = match reason {
                spec_spine_types::ContentOmissionReason::MissingContent => {
                    RefusalReason::UnresolvedTarget
                }
                spec_spine_types::ContentOmissionReason::UnsupportedSelector
                    if detail.contains("more than one") || detail.contains("ambiguous") =>
                {
                    RefusalReason::AmbiguousTarget
                }
                _ => RefusalReason::UnsupportedTargetKind,
            };
            Err((refusal, format!("{}: {detail}", r.identity)))
        }
    })
}

fn with_projection(selector: &ContentSelector, p: ContentProjection) -> ContentSelector {
    let mut s = selector.clone();
    match &mut s {
        ContentSelector::Spec { projection, .. }
        | ContentSelector::SpecSection { projection, .. }
        | ContentSelector::Obligation { projection, .. }
        | ContentSelector::OwnedUnit { projection, .. }
        | ContentSelector::File { projection, .. }
        | ContentSelector::DirectoryMember { projection, .. }
        | ContentSelector::Symbol { projection, .. }
        | ContentSelector::Module { projection, .. }
        | ContentSelector::Test { projection, .. } => *projection = Some(p),
    }
    s
}

fn stale(c: &mut Checked, expected: &str, current: &str) {
    c.refusals.push((
        RefusalReason::StaleTarget,
        format!("stale-target: planned against {expected}, current {current}"),
    ));
}

/// The subject spec a frontmatter addition targets: resolved, digest
/// guarded, encoded faithfully, and located at its frontmatter block.
fn subject_target(
    view: &View<'_>,
    target: &ContentSelector,
    expected: &str,
    c: &mut Checked,
) -> Result<Option<String>, Error> {
    let ContentSelector::Spec { spec, .. } = target else {
        c.planned.target = canonical_json::to_string(target)?;
        c.refusals.push((
            RefusalReason::UnsupportedTargetKind,
            "unsupported-target-kind: a frontmatter addition targets a `spec` selector".into(),
        ));
        return Ok(None);
    };
    let Some(record) = record_of(view.registry, spec) else {
        c.planned.target = format!("spec:{spec}");
        c.refusals.push((
            RefusalReason::UnresolvedTarget,
            format!("unresolved-target: no spec '{spec}'"),
        ));
        return Ok(None);
    };
    c.planned.target = record.spec_path.clone();
    if record.id != view.subject.id {
        c.refusals.push((
            RefusalReason::UnauthorizedTarget,
            format!(
                "unauthorized-target: a frontmatter addition edits the subject {}, not {}",
                view.subject.id, record.id
            ),
        ));
        return Ok(None);
    }
    let digest = match item_of(view, target)? {
        Ok(found) => found.digest,
        Err((reason, detail)) => {
            c.refusals.push((reason, detail));
            return Ok(None);
        }
    };
    if digest != expected {
        stale(c, expected, &digest);
        return Ok(None);
    }
    let raw = read_raw(view, &record.spec_path)?.unwrap_or_default();
    let Some(text) = encoding_ok(&raw) else {
        c.refusals.push((
            RefusalReason::UnsupportedEncoding,
            format!(
                "unsupported-encoding: {} is not LF UTF-8 without a BOM",
                record.spec_path
            ),
        ));
        return Ok(None);
    };
    let Some((open, close)) = frontmatter_block(&text) else {
        c.refusals.push((
            RefusalReason::UnsupportedLayout,
            format!(
                "unsupported-layout: {} has no frontmatter block",
                record.spec_path
            ),
        ));
        return Ok(None);
    };
    c.located = Some(Located {
        identity: record.spec_path.clone(),
        path: record.spec_path.clone(),
        span: Span::Frontmatter(open, close),
    });
    Ok(Some(text))
}

/// The frontmatter block's fence lines, 1-based.
fn frontmatter_block(text: &str) -> Option<(usize, usize)> {
    let mut lines = text.lines();
    if lines.next()? != "---" {
        return None;
    }
    lines.position(|l| l == "---").map(|i| (1, i + 2))
}

/// 3.4: one unit on `establishes`, or one `extends` edge.
fn check_claim(
    view: &View<'_>,
    file: &str,
    edge: ClaimEdge,
    unit: &Unit,
    spec: Option<&str>,
    nature: Option<&str>,
    c: &mut Checked,
) -> Result<(), Error> {
    let subject = view.subject;
    let path = unit_path(unit);
    match edge {
        ClaimEdge::Establishes => {
            if spec.is_some() || nature.is_some() {
                return Err(Error::Usage(
                    "an establishes claim names no spec and no nature".into(),
                ));
            }
            if subject
                .establishes
                .iter()
                .any(|u| u.subject() == unit.subject())
            {
                c.refusals.push((
                    RefusalReason::DuplicateDeclaration,
                    "duplicate-declaration: the subject already establishes this unit".into(),
                ));
            }
            let others: Vec<String> = view
                .registry
                .specs
                .iter()
                .filter(|r| r.id != subject.id)
                .filter(|r| r.establishes.iter().any(|u| u.subject() == unit.subject()))
                .map(|r| r.id.clone())
                .collect();
            if !others.is_empty() {
                c.refusals.push((
                    RefusalReason::UnauthorizedTarget,
                    format!(
                        "unauthorized-target: {} already establishes this unit; claim it on an \
                         extends edge",
                        others.join(", ")
                    ),
                ));
            }
        }
        ClaimEdge::Extends => {
            let Some(named) = spec else {
                return Err(Error::Usage(
                    "an extends claim names the spec it extends".into(),
                ));
            };
            match record_of(view.registry, named) {
                None => c.refusals.push((
                    RefusalReason::UnresolvedTarget,
                    format!("unresolved-target: no spec '{named}' to extend"),
                )),
                Some(r) => {
                    // An extends-carried unit is a first-class claim: the
                    // named spec need not claim it itself (3.4).
                    if subject.extends.iter().any(|e| {
                        e.spec == r.id && e.unit.as_ref().map(Unit::subject) == Some(unit.subject())
                    }) {
                        c.refusals.push((
                            RefusalReason::DuplicateDeclaration,
                            format!(
                                "duplicate-declaration: the subject already extends {} with this \
                                 unit",
                                r.id
                            ),
                        ));
                    }
                }
            }
        }
    }
    if let Some(p) = &path
        && !unit_planned(unit)
        && read_raw_exists(view, p)?.is_none()
    {
        c.refusals.push((
            RefusalReason::UnresolvedTarget,
            format!("unresolved-target: '{p}' does not exist and the unit is not planned"),
        ));
    }
    if let Some(p) = &path {
        let owners: Vec<String> = crate::index::owner_with(view.cfg, view.registry, view.index, p)
            .owners
            .into_iter()
            .map(|o| o.spec_id)
            .filter(|id| id != &subject.id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if !owners.is_empty() {
            c.conflicts.push((ConflictKind::SharedOwnership, owners));
        }
    }
    let key = match edge {
        ClaimEdge::Establishes => "establishes",
        ClaimEdge::Extends => "extends",
    };
    let unit_flow = flow(&serde_json::to_value(unit).map_err(|e| Error::Internal(e.to_string()))?);
    let line = match edge {
        ClaimEdge::Establishes => unit_flow,
        ClaimEdge::Extends => {
            let named = spec
                .and_then(|s| record_of(view.registry, s))
                .map(|r| r.id.clone())
                .unwrap_or_default();
            let mut entry = format!("{{ spec: {}, unit: {unit_flow}", quote(&named));
            if let Some(n) = nature {
                entry.push_str(&format!(", nature: {}", scalar(n)));
            }
            entry.push_str(" }");
            entry
        }
    };
    frontmatter_edit(file, key, line, c);
    Ok(())
}

/// Whether a path exists in the export, as a file or a directory.
fn read_raw_exists(view: &View<'_>, path: &str) -> Result<Option<()>, Error> {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return Ok(Some(()));
    }
    Ok(contained_metadata(view, trimmed)?.map(|_| ()))
}

/// The repository path a unit names, when it names one.
fn unit_path(unit: &Unit) -> Option<String> {
    let v = serde_json::to_value(unit).ok()?;
    v.get("path")
        .or_else(|| v.get("file"))
        .and_then(|p| p.as_str())
        .map(str::to_string)
}

fn unit_planned(unit: &Unit) -> bool {
    serde_json::to_value(unit)
        .ok()
        .and_then(|v| v.get("planned").and_then(|p| p.as_bool()))
        .unwrap_or(false)
}

/// Plan one flow line under `key`, or refuse the layout (3.4).
fn frontmatter_edit(file: &str, key: &'static str, line: String, c: &mut Checked) {
    match frontmatter_slot(file, key) {
        Ok(_) => c.edit = Some(Edit::Frontmatter { key, line }),
        Err(why) => c.refusals.push((
            RefusalReason::UnsupportedLayout,
            format!("unsupported-layout: {why}"),
        )),
    }
}

/// Where a new entry under `key` goes: `Ok(Some((after_line, indent)))` to
/// append after an existing block sequence, `Ok(None)` to add the key before
/// the closing fence. Anything but a plain block sequence refuses.
fn frontmatter_slot(file: &str, key: &str) -> Result<Option<(usize, String)>, String> {
    let (_, close) = frontmatter_block(file).ok_or("no frontmatter block")?;
    let lines: Vec<&str> = file.lines().collect();
    let head = format!("{key}:");
    let Some(at) =
        (1..close - 1).find(|&i| lines[i] == head || lines[i].starts_with(&format!("{head} ")))
    else {
        return Ok(None);
    };
    let rest = lines[at][head.len()..].trim();
    if !rest.is_empty() {
        return Err(format!("`{key}` is not a block sequence"));
    }
    let mut last = at;
    let mut indent: Option<String> = None;
    for (i, line) in lines.iter().enumerate().take(close - 1).skip(at + 1) {
        if !line.starts_with(' ') && !line.is_empty() {
            break;
        }
        let t = line.trim_start();
        if t.starts_with('#') || has_anchor_or_alias(t) {
            return Err(format!("`{key}` carries a comment, anchor or alias"));
        }
        if t.is_empty() {
            continue;
        }
        if indent.is_none() {
            if !t.starts_with("- ") {
                return Err(format!("`{key}` is not a block sequence"));
            }
            indent = Some(line[..line.len() - t.len()].to_string());
        }
        last = i;
    }
    Ok(Some((last, indent.unwrap_or_else(|| "  ".into()))))
}

fn has_anchor_or_alias(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.iter().enumerate().any(|(i, b)| {
        (*b == b'&' || *b == b'*')
            && (i == 0 || matches!(bytes[i - 1], b' ' | b'[' | b'{' | b',' | b':'))
            // YAML's ns-anchor-char: any non-blank character but a flow
            // indicator, so `&_root` and `*-x` are names too.
            && bytes
                .get(i + 1)
                .is_some_and(|n| !n.is_ascii_whitespace() && !b",[]{}".contains(n))
            && !in_quotes(text, i)
    })
}

/// Whether byte `at` sits inside a quoted YAML scalar, double-quoted (with
/// backslash escapes) or single-quoted (where `''` is a literal quote and
/// toggles out and back in). A single quote opens a scalar only where one can
/// start; an apostrophe inside a plain word does not, so it cannot hide a
/// real anchor from the check.
fn in_quotes(text: &str, at: usize) -> bool {
    let bytes = text.as_bytes();
    let mut double = false;
    let mut single = false;
    let mut escaped = false;
    for (i, ch) in text.char_indices() {
        if i >= at {
            break;
        }
        match ch {
            '\\' if double => escaped = !escaped,
            '"' if !single && !escaped => double = !double,
            '\'' if single => single = false,
            '\'' if !double
                && (i == 0 || matches!(bytes[i - 1], b' ' | b'[' | b'{' | b',' | b':' | b'\'')) =>
            {
                single = true;
            }
            _ => escaped = false,
        }
    }
    double || single
}

/// A JSON value as one YAML flow node: strings double-quoted, the closed
/// vocabulary values bare.
fn flow(value: &serde_json::Value) -> String {
    flow_keyed(None, value)
}

fn flow_keyed(key: Option<&str>, value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let parts: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{k}: {}", flow_keyed(Some(k), v)))
                .collect();
            format!("{{ {} }}", parts.join(", "))
        }
        serde_json::Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|v| flow_keyed(key, v))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        serde_json::Value::String(s)
            if matches!(key, Some("kind" | "nature" | "relation" | "role")) =>
        {
            scalar(s)
        }
        serde_json::Value::String(s) => quote(s),
        other => other.to_string(),
    }
}

fn quote(s: &str) -> String {
    serde_json::Value::String(s.to_string()).to_string()
}

/// A bare scalar when it cannot be read as anything but a string.
fn scalar(s: &str) -> String {
    let bare = !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && s.as_bytes()[0].is_ascii_lowercase()
        && !matches!(
            s,
            "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "y" | "n"
        );
    if bare { s.to_string() } else { quote(s) }
}

/// 3.5: one spec 169 entry on the subject.
fn check_trace(
    view: &View<'_>,
    file: &str,
    entry: &TraceDeclaration,
    c: &mut Checked,
) -> Result<(), Error> {
    let subject = view.subject;
    if subject.traceability.iter().any(|d| d.id == entry.id) {
        c.refusals.push((
            RefusalReason::DuplicateDeclaration,
            format!(
                "duplicate-declaration: the subject already declares trace '{}'",
                entry.id
            ),
        ));
    }
    let tuple = |d: &TraceDeclaration| {
        (
            d.obligation.clone(),
            d.relation,
            crate::traceability::target_identity(&d.target, &subject.id).unwrap_or_default(),
        )
    };
    if subject
        .traceability
        .iter()
        .any(|d| !d.withdrawn && tuple(d) == tuple(entry))
    {
        c.refusals.push((
            RefusalReason::DuplicateDeclaration,
            "duplicate-declaration: the subject already declares this relation".into(),
        ));
    }
    if let TraceTarget::Interface {
        spec,
        corpus: Some(corpus),
        unit: None,
        ..
    } = &entry.target
    {
        let n = subject
            .interface_references
            .iter()
            .filter(|r| &r.corpus == corpus && &r.spec == spec)
            .count();
        if n > 1 {
            c.refusals.push((
                RefusalReason::AmbiguousTarget,
                format!(
                    "ambiguous-target: '{corpus}' / '{spec}' matches {n} of the subject's \
                     interface_references"
                ),
            ));
        } else if n == 0 {
            c.refusals.push((
                RefusalReason::UnresolvedInterface,
                format!(
                    "unresolved-interface: '{corpus}' / '{spec}' matches {n} of the subject's \
                     interface_references"
                ),
            ));
        }
    }
    let line = flow(&serde_json::to_value(entry).map_err(|e| Error::Internal(e.to_string()))?);
    frontmatter_edit(file, "traceability", line, c);
    Ok(())
}

/// 3.6: a new draft spec that amends approved obligations, never an edit of
/// the amended spec.
#[allow(clippy::too_many_arguments)]
fn check_amendment(
    view: &View<'_>,
    target: &str,
    expected: &str,
    spec: &str,
    obligations: &[String],
    content: &str,
    c: &mut Checked,
) -> Result<(), Error> {
    c.planned.target = target.to_string();
    c.located = Some(Located {
        identity: target.to_string(),
        path: target.to_string(),
        span: Span::Whole,
    });
    let specs_dir = view.cfg.layout.specs_dir.trim_end_matches('/');
    if target != format!("{specs_dir}/{spec}/spec.md") {
        c.refusals.push((
            RefusalReason::UnauthorizedTarget,
            format!("unauthorized-target: a new spec '{spec}' lives at {specs_dir}/{spec}/spec.md"),
        ));
    }
    let present = read_raw(view, target)?.is_some();
    if expected != EXPECTED_ABSENT || present {
        let current = if present { "present" } else { EXPECTED_ABSENT };
        stale(c, expected, current);
    }
    // The ordinal must be unused and next free (contiguity).
    let ordinal = |id: &str| -> Option<u32> { crate::spec_id::leading_segment(id).parse().ok() };
    let max = view
        .registry
        .specs
        .iter()
        .filter_map(|r| ordinal(&r.id))
        .max()
        .unwrap_or(0);
    match ordinal(spec) {
        Some(n) if n == max + 1 => {}
        _ => c.refusals.push((
            RefusalReason::UnauthorizedTarget,
            format!(
                "unauthorized-target: '{spec}' is not the next free ordinal ({:03})",
                max + 1
            ),
        )),
    }
    if encoding_ok(content.as_bytes()).is_none() {
        c.refusals.push((
            RefusalReason::UnsupportedEncoding,
            "unsupported-encoding: the new file is not LF UTF-8 without a BOM".into(),
        ));
        return Ok(());
    }

    let committed: Vec<SpecRecord> = view.registry.specs.clone();
    let (report, record) = crate::compile::check_spec_source(
        view.cfg,
        committed,
        spec.to_string(),
        target.to_string(),
        content,
    );
    let invalid = |rule: String, c: &mut Checked| {
        c.refusals.push((
            RefusalReason::PostImageInvalid,
            format!("post-image-invalid: {rule}"),
        ))
    };
    for v in report
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
    {
        invalid(format!("{} {}", v.code, v.message), c);
    }
    let Some(record) = record else {
        return Ok(());
    };
    if record.status != Status::Draft || record.implementation != Some(Implementation::Pending) {
        invalid(
            "the new spec is status: draft, implementation: pending".into(),
            c,
        );
    }
    let mut amended: BTreeSet<String> = BTreeSet::new();
    for reference in obligations {
        let Some((spec_part, ob_id)) = split_obligation_ref(reference) else {
            return Err(Error::Usage(format!(
                "obligation '{reference}' is not a qualified <spec>#<id> reference"
            )));
        };
        let Some(target_spec) = record_of(view.registry, spec_part) else {
            c.refusals.push((
                RefusalReason::UnresolvedTarget,
                format!("unresolved-target: no spec '{spec_part}'"),
            ));
            continue;
        };
        let Some(ob) = target_spec.obligations.iter().find(|o| o.id == ob_id) else {
            c.refusals.push((
                RefusalReason::UnresolvedTarget,
                format!(
                    "unresolved-target: '{}' declares no '{ob_id}'",
                    target_spec.id
                ),
            ));
            continue;
        };
        let qualified = format!("{}#{ob_id}", target_spec.id);
        if target_spec.status != Status::Approved {
            c.refusals.push((
                RefusalReason::AmendmentTargetNotApproved,
                format!(
                    "amendment-target-not-approved: {} is not approved; edit a draft directly",
                    target_spec.id
                ),
            ));
        }
        if target_spec.unamendable.iter().any(|a| a == &ob.anchor) {
            c.refusals.push((
                RefusalReason::UnamendableAnchor,
                format!(
                    "unamendable-anchor: {}#{} is unamendable",
                    target_spec.id, ob.anchor
                ),
            ));
        }
        if !record.amends.iter().any(|a| a == &target_spec.id) {
            invalid(format!("`amends` does not name {}", target_spec.id), c);
        }
        if !record.amends_sections.iter().any(|a| a == &ob.anchor) {
            invalid(format!("`amends_sections` does not name {}", ob.anchor), c);
        }
        match record.impacts.iter().find(|i| i.obligation == qualified) {
            None => invalid(format!("no `impacts` entry for {qualified}"), c),
            Some(impact)
                if impact.nature == spec_spine_types::ImpactNature::Supersedes
                    && !impact.successor.as_ref().is_some_and(|s| {
                        record
                            .obligations
                            .iter()
                            .any(|o| &o.id == s && !o.withdrawn)
                    }) =>
            {
                invalid(
                    format!("the `supersedes` impact on {qualified} names no declared successor"),
                    c,
                )
            }
            Some(_) => {}
        }
        let declared: Vec<String> = view
            .registry
            .specs
            .iter()
            .filter(|r| r.conflicts.iter().any(|x| x.obligation == qualified))
            .map(|r| r.id.clone())
            .collect();
        if !declared.is_empty() {
            c.conflicts.push((ConflictKind::DeclaredConflict, declared));
        }
        amended.insert(target_spec.spec_path.clone());
    }
    c.amended_files = amended.into_iter().collect();
    c.edit = Some(Edit::Whole {
        content: content.to_string(),
    });
    Ok(())
}

/// 3.7: one manifest file, replaced whole by a manifest that validates under
/// spec 160 against this snapshot and its own pages.
fn check_manifest(
    view: &View<'_>,
    target: &str,
    expected: &str,
    manifest: &serde_json::Value,
    pages: &[String],
    c: &mut Checked,
) -> Result<(), Error> {
    c.planned.target = target.to_string();
    let Some(raw) = read_raw(view, target)? else {
        c.refusals.push((
            RefusalReason::UnresolvedTarget,
            format!("unresolved-target: '{target}' does not exist"),
        ));
        return Ok(());
    };
    c.located = Some(Located {
        identity: target.to_string(),
        path: target.to_string(),
        span: Span::Whole,
    });
    let current = std::str::from_utf8(&raw)
        .ok()
        .and_then(|t| crate::documentation_manifest::parse_documentation_manifest(t).ok());
    let Some(current) = current else {
        c.refusals.push((
            RefusalReason::UnsupportedTargetKind,
            format!("unsupported-target-kind: '{target}' is not a documentation manifest"),
        ));
        return Ok(());
    };
    if current.manifest_digest != expected {
        stale(c, expected, &current.manifest_digest);
        return Ok(());
    }
    let replacement: Result<DocumentationManifest, String> =
        serde_json::from_value(manifest.clone()).map_err(|e| e.to_string());
    let parsed_pages: Result<Vec<ContextPacket>, String> = pages
        .iter()
        .enumerate()
        .map(|(i, p)| serde_json::from_str(p).map_err(|e| format!("page {}: {e}", i + 1)))
        .collect();
    let snapshot = PacketSnapshot {
        repository: view.snapshot.repository.clone(),
        revision: view.snapshot.revision.clone(),
        tree: view.snapshot.tree.clone(),
        dirty_state: view.snapshot.dirty_state,
    };
    let validated = match (replacement, parsed_pages) {
        (Ok(m), Ok(p)) => {
            crate::documentation_manifest::validate_manifest_inputs(&m, &p, &snapshot)
                .map_err(|e| e.to_string())
        }
        (Err(e), _) | (_, Err(e)) => Err(e),
    };
    match validated {
        Ok(m) => {
            c.edit = Some(Edit::Whole {
                content: crate::documentation_manifest::documentation_manifest_document(&m)?,
            });
        }
        Err(e) => c.refusals.push((
            RefusalReason::ManifestInvalid,
            format!("manifest-invalid: {e}"),
        )),
    }
    Ok(())
}

/// 3.8: one uniquely resolved spec or documentation unit's bytes.
fn check_replace(
    view: &View<'_>,
    target: &ContentSelector,
    expected: &str,
    content: &str,
    c: &mut Checked,
) -> Result<(), Error> {
    c.planned.target = canonical_json::to_string(target)?;
    let supported = match target {
        ContentSelector::SpecSection { .. } => true,
        ContentSelector::File { path, .. } => path.as_str().ends_with(".md"),
        ContentSelector::OwnedUnit { unit, .. } => {
            unit_path(unit).is_some_and(|p| p.ends_with(".md"))
        }
        _ => false,
    };
    if !supported {
        c.refusals.push((
            RefusalReason::UnsupportedTargetKind,
            "unsupported-target-kind: replace-unit edits a spec section or a Markdown \
             documentation file or section"
                .into(),
        ));
        return Ok(());
    }
    let Resolved {
        identity,
        path,
        span: (start, end),
        digest,
    } = match item_of(view, target)? {
        Ok(found) => found,
        Err((reason, detail)) => {
            c.refusals.push((reason, detail));
            return Ok(());
        }
    };
    c.planned.target = identity.clone();
    if digest != expected {
        stale(c, expected, &digest);
        return Ok(());
    }
    let raw = read_raw(view, &path)?.unwrap_or_default();
    let Some(text) = encoding_ok(&raw) else {
        c.refusals.push((
            RefusalReason::UnsupportedEncoding,
            format!("unsupported-encoding: {path} is not LF UTF-8 without a BOM"),
        ));
        return Ok(());
    };
    if !content.is_empty() && !content.ends_with('\n') {
        c.refusals.push((
            RefusalReason::PostImageInvalid,
            "post-image-invalid: the replacement does not end at a line boundary".into(),
        ));
    }
    let span = if matches!(target, ContentSelector::File { .. }) {
        Span::Whole
    } else {
        Span::Lines(start, end)
    };
    c.located = Some(Located {
        identity,
        path: path.clone(),
        span,
    });
    let old = slice_lines(&text, span);

    // The spec this span lives in, if any, decides the effect and whether an
    // approved text may change at all.
    let spec_record = view.registry.specs.iter().find(|r| r.spec_path == path);
    match spec_record {
        Some(r) if r.status != Status::Draft => {
            let decision_anchor = decisions_anchor(&text);
            let is_append = matches!(target, ContentSelector::SpecSection { anchor, .. }
                if decision_anchor.as_deref() == Some(anchor.as_str()))
                && content.starts_with(&old)
                && r.id == view.subject.id;
            if is_append {
                c.planned.authority_effect = AuthorityEffect::RequiresHumanRatification;
            } else {
                c.refusals.push((
                    RefusalReason::ApprovedSpecText,
                    format!(
                        "approved-spec-text: {} is {}; only an append to its decisions section \
                         is admissible, and propose-amendment is the governed route",
                        r.id,
                        status_name(r.status)
                    ),
                ));
            }
        }
        Some(r) => {
            let anchor = match target {
                ContentSelector::SpecSection { anchor, .. } => Some(anchor.as_str()),
                _ => None,
            };
            c.planned.authority_effect = if anchor
                .is_some_and(|a| a == "verification" || r.obligations.iter().any(|o| o.anchor == a))
            {
                AuthorityEffect::ChangesObligation
            } else {
                AuthorityEffect::None
            };
        }
        None => c.planned.authority_effect = AuthorityEffect::None,
    }
    c.edit = Some(match span {
        Span::Whole => Edit::Whole {
            content: content.to_string(),
        },
        _ => Edit::Replace {
            start,
            end,
            content: content.to_string(),
        },
    });
    Ok(())
}

/// The anchor of the one level-2 decisions heading (3.8), or `None` when
/// there is not exactly one.
fn decisions_anchor(text: &str) -> Option<String> {
    let body = spec_spine_types::split_frontmatter(text)
        .map(|(_, b)| b)
        .unwrap_or_default();
    let found: Vec<String> = body
        .lines()
        .filter_map(|l| l.strip_prefix("## "))
        .filter(|h| {
            let h = h.trim();
            let h = h
                .split_once(". ")
                .filter(|(n, _)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
                .map_or(h, |(_, rest)| rest)
                .to_ascii_lowercase();
            DECISION_HEADINGS.iter().any(|d| h.starts_with(d))
        })
        .map(crate::sections::anchor_of)
        .collect();
    (found.len() == 1).then(|| found[0].clone())
}

fn slice_lines(text: &str, span: Span) -> String {
    match span.range() {
        None => text.to_string(),
        Some((a, b)) => {
            let mut out: String = text
                .split_inclusive('\n')
                .skip(a.saturating_sub(1))
                .take((b + 1).saturating_sub(a))
                .collect();
            if !out.ends_with('\n') && !out.is_empty() {
                out.push('\n');
            }
            out
        }
    }
}

/// 3.10: authority over one changed or created file, checks in order.
fn authority(view: &View<'_>, located: &Located, created: bool) -> Option<(RefusalReason, String)> {
    let path = located.path.as_str();
    let under = |dir: &str| {
        let dir = dir.trim_end_matches('/');
        !dir.is_empty() && (path == dir || path.starts_with(&format!("{dir}/")))
    };
    if under(&view.cfg.layout.derived_dir) || under(&view.cfg.layout.state_dir) {
        return Some((
            RefusalReason::GeneratedTarget,
            format!("generated-target: '{path}' is a derived or state path"),
        ));
    }
    for m in view.manifests {
        for output in m.outputs.iter().filter(|o| o.path.as_str() == path) {
            for r in output
                .regions
                .iter()
                .filter(|r| r.kind == RegionKind::Generated)
            {
                let hit = match located.span.range() {
                    None => true,
                    Some((a, b)) => a <= r.span.end_line && r.span.start_line <= b,
                };
                if hit {
                    return Some((
                        RefusalReason::GeneratedTarget,
                        format!(
                            "generated-target: '{path}' lines {}-{} are generated region '{}'",
                            r.span.start_line, r.span.end_line, r.region_id
                        ),
                    ));
                }
            }
        }
    }
    let in_scope = view
        .scope
        .mutable
        .iter()
        .chain(view.scope.shared.iter().map(|s| &s.path))
        .any(|entry| crate::couple::claim_matches(entry, path));
    if !in_scope {
        return Some((
            RefusalReason::OutOfScope,
            format!("out-of-scope: '{path}' is not under the scope's mutable or shared paths"),
        ));
    }
    if is_spec_file(view.cfg, path) {
        if !created && path != view.subject.spec_path {
            return Some((
                RefusalReason::UnauthorizedTarget,
                format!("unauthorized-target: '{path}' is not the subject's spec"),
            ));
        }
        return None;
    }
    let owners: BTreeSet<String> =
        crate::index::owner_with(view.cfg, view.registry, view.index, path)
            .owners
            .into_iter()
            .filter(|o| o.kind != crate::index::OwnerKind::Floor)
            .map(|o| o.spec_id)
            .collect();
    if owners.is_empty() {
        return Some((
            RefusalReason::UnownedTarget,
            format!("unowned-target: no spec owns '{path}'"),
        ));
    }
    if !owners.contains(&view.subject.id) {
        return Some((
            RefusalReason::UnauthorizedTarget,
            format!(
                "unauthorized-target: '{path}' is owned by {}, not the subject",
                owners.into_iter().collect::<Vec<_>>().join(", ")
            ),
        ));
    }
    None
}

fn is_spec_file(cfg: &Config, path: &str) -> bool {
    let dir = cfg.layout.specs_dir.trim_end_matches('/');
    path.strip_prefix(&format!("{dir}/"))
        .and_then(|rest| rest.strip_suffix("/spec.md"))
        .is_some_and(|id| !id.is_empty() && !id.contains('/'))
}

/// 3.11 and 3.6: two operations overlap when they name one target identity
/// and either is not a frontmatter addition, when a `replace-unit` span meets
/// another's, when one replaces a file another changes, or when an operation
/// touches a file a `propose-amendment` amends.
fn overlaps(checked: &mut [Checked]) {
    let mut found: Vec<(usize, usize)> = Vec::new();
    for a in 0..checked.len() {
        for b in a + 1..checked.len() {
            let (Some(la), Some(lb)) = (&checked[a].located, &checked[b].located) else {
                continue;
            };
            let fa = matches!(la.span, Span::Frontmatter(..));
            let fb = matches!(lb.span, Span::Frontmatter(..));
            let same_identity = la.identity == lb.identity && !(fa && fb);
            let same_file = la.path == lb.path;
            let replace = |c: &Checked| c.planned.op == "replace-unit";
            let spans_meet = same_file
                && (replace(&checked[a]) || replace(&checked[b]))
                && match (la.span.range(), lb.span.range()) {
                    (Some((a1, a2)), Some((b1, b2))) => a1 <= b2 && b1 <= a2,
                    _ => true,
                };
            let whole = same_file && (la.span == Span::Whole || lb.span == Span::Whole);
            let amended = checked[a].amended_files.contains(&lb.path)
                || checked[b].amended_files.contains(&la.path);
            if same_identity || spans_meet || whole || amended {
                found.push((a, b));
            }
        }
    }
    for (a, b) in found {
        let (ia, ib) = (checked[a].planned.index, checked[b].planned.index);
        let detail = format!(
            "overlapping-operations: operations {} and {} overlap",
            ia.min(ib),
            ia.max(ib)
        );
        for x in [a, b] {
            checked[x]
                .refusals
                .push((RefusalReason::OverlappingOperations, detail.clone()));
        }
    }
}

/// 3.4, 3.5: two frontmatter additions in one plan that declare the same
/// entry (one flow line under one key) are duplicates of each other, which
/// the single-spec compile does not see across a plan.
fn repeated_declarations(checked: &mut [Checked]) {
    let mut seen: BTreeMap<(&'static str, String), usize> = BTreeMap::new();
    let mut repeats: Vec<(usize, usize)> = Vec::new();
    for (pos, c) in checked.iter().enumerate() {
        let Some(Edit::Frontmatter { key, line }) = &c.edit else {
            continue;
        };
        let identity = trace_id(line).unwrap_or_else(|| line.clone());
        match seen.get(&(*key, identity.clone())) {
            Some(&first) => repeats.push((first, pos)),
            None => {
                seen.insert((*key, identity), pos);
            }
        }
    }
    for (first, pos) in repeats {
        let (a, b) = (checked[first].planned.index, checked[pos].planned.index);
        for x in [first, pos] {
            checked[x].refusals.push((
                RefusalReason::DuplicateDeclaration,
                format!(
                    "duplicate-declaration: operations {} and {} declare the same entry",
                    a.min(b),
                    a.max(b)
                ),
            ));
        }
    }
}

/// The `id` of a rendered trace entry, which alone identifies it. `flow`
/// renders it as a JSON string, so it is read as one: a quoted id holding
/// `, ` is not cut short at the first separator.
fn trace_id(line: &str) -> Option<String> {
    let rest = line.strip_prefix("{ id: ")?;
    let mut values = serde_json::Deserializer::from_str(rest).into_iter::<String>();
    values.next()?.ok()
}

/// One file's pre- and post-image.
struct Image {
    before: Option<String>,
    before_raw: Option<Vec<u8>>,
    after: String,
}

/// Apply every accepted operation in memory, per file: body replacements
/// from the bottom up, then frontmatter additions in canonical order.
fn compose(view: &View<'_>, checked: &[Checked]) -> Result<BTreeMap<String, Image>, Error> {
    let mut by_file: BTreeMap<String, Vec<&Edit>> = BTreeMap::new();
    for c in checked {
        if let (Some(l), Some(e)) = (&c.located, &c.edit) {
            by_file.entry(l.path.clone()).or_default().push(e);
        }
    }
    let mut images = BTreeMap::new();
    for (path, edits) in by_file {
        let raw = read_raw(view, &path)?;
        let before = raw
            .as_deref()
            .map(|b| String::from_utf8_lossy(b).into_owned());
        let mut after = before.clone().unwrap_or_default();
        let mut replaces: Vec<(usize, usize, &str)> = Vec::new();
        let mut additions: Vec<(&str, &str)> = Vec::new();
        for e in edits {
            match e {
                Edit::Whole { content } => after = content.clone(),
                Edit::Replace {
                    start,
                    end,
                    content,
                } => replaces.push((*start, *end, content)),
                Edit::Frontmatter { key, line } => additions.push((key, line)),
            }
        }
        replaces.sort_by(|a, b| b.0.cmp(&a.0));
        for (start, end, content) in replaces {
            let lines: Vec<&str> = after.split_inclusive('\n').collect();
            let mut next: String = lines[..start.saturating_sub(1).min(lines.len())].concat();
            next.push_str(content);
            next.push_str(&lines[end.min(lines.len())..].concat());
            after = next;
        }
        for (key, line) in additions {
            after = insert_frontmatter(&after, key, line)
                .map_err(|why| Error::Internal(format!("frontmatter insert into {path}: {why}")))?;
        }
        images.insert(
            path,
            Image {
                before,
                before_raw: raw,
                after,
            },
        );
    }
    Ok(images)
}

fn insert_frontmatter(text: &str, key: &str, entry: &str) -> Result<String, String> {
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    match frontmatter_slot(text, key)? {
        Some((after, indent)) => lines.insert(after + 1, format!("{indent}- {entry}\n")),
        None => {
            let (_, close) = frontmatter_block(text).ok_or("no frontmatter block")?;
            lines.insert(close - 1, format!("{key}:\n  - {entry}\n"));
        }
    }
    Ok(lines.concat())
}

/// For each accepted `add-trace-relation`: the spec 169 state the entry
/// would have once the subject's post-image is compiled (3.5). An
/// `ambiguous` target refuses (D-7).
fn predict_trace_states(
    view: &View<'_>,
    checked: &mut [Checked],
    images: &BTreeMap<String, Image>,
) -> Result<(), Error> {
    let Some(image) = images.get(&view.subject.spec_path) else {
        return Ok(());
    };
    if !checked.iter().any(|c| c.planned.op == "add-trace-relation") {
        return Ok(());
    }
    let committed: Vec<SpecRecord> = view
        .registry
        .specs
        .iter()
        .filter(|r| r.id != view.subject.id)
        .cloned()
        .collect();
    let (_, record) = crate::compile::check_spec_source(
        view.cfg,
        committed,
        view.subject.id.clone(),
        view.subject.spec_path.clone(),
        &image.after,
    );
    let Some(record) = record else {
        return Ok(());
    };
    let mut registry = view.registry.clone();
    for r in registry.specs.iter_mut().filter(|r| r.id == record.id) {
        *r = record.clone();
    }
    let report = crate::traceability::traceability(
        view.cfg,
        view.root,
        &registry,
        Some(view.index),
        &TraceabilityRequest {
            declared_by: Some(record.id.clone()),
            ..TraceabilityRequest::default()
        },
    )?;
    for c in checked
        .iter_mut()
        .filter(|c| c.planned.op == "add-trace-relation")
    {
        let Some(Edit::Frontmatter { line, .. }) = &c.edit else {
            continue;
        };
        let added: Vec<&spec_spine_types::TraceRelation> = report
            .relations
            .iter()
            .filter(|r| line.contains(&format!("id: {}", quote(&r.id))))
            .collect();
        if let Some(rel) = added.first() {
            c.planned.predicted_state = Some(rel.state);
            if rel.state == spec_spine_types::TraceState::Ambiguous {
                c.refusals.push((
                    RefusalReason::AmbiguousTarget,
                    format!(
                        "ambiguous-target: trace '{}' binds more than one target",
                        rel.id
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// A unified diff with three lines of context, derived from the two texts.
fn unified_diff(path: &str, before: &str, after: &str) -> String {
    let a: Vec<&str> = before.split_inclusive('\n').collect();
    let b: Vec<&str> = after.split_inclusive('\n').collect();
    let ops = line_ops(&a, &b);
    if ops.iter().all(|o| matches!(o, Op::Same(..))) {
        return String::new();
    }
    let from = if before.is_empty() {
        "/dev/null".to_string()
    } else {
        format!("a/{path}")
    };
    let mut out = format!("--- {from}\n+++ b/{path}\n");
    const CONTEXT: usize = 3;
    let changed: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, o)| !matches!(o, Op::Same(..)))
        .map(|(i, _)| i)
        .collect();
    let mut i = 0;
    while i < changed.len() {
        let start = changed[i].saturating_sub(CONTEXT);
        let mut end = changed[i];
        while i + 1 < changed.len() && changed[i + 1] <= end + 2 * CONTEXT + 1 {
            i += 1;
            end = changed[i];
        }
        let end = (end + CONTEXT + 1).min(ops.len());
        let (mut a_start, mut b_start) = (1, 1);
        for o in &ops[..start] {
            match o {
                Op::Same(..) => {
                    a_start += 1;
                    b_start += 1;
                }
                Op::Del(_) => a_start += 1,
                Op::Add(_) => b_start += 1,
            }
        }
        let a_len = ops[start..end]
            .iter()
            .filter(|o| !matches!(o, Op::Add(_)))
            .count();
        let b_len = ops[start..end]
            .iter()
            .filter(|o| !matches!(o, Op::Del(_)))
            .count();
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            if a_len == 0 { a_start - 1 } else { a_start },
            a_len,
            if b_len == 0 { b_start - 1 } else { b_start },
            b_len
        ));
        for o in &ops[start..end] {
            let (mark, line) = match o {
                Op::Same(l) => (' ', *l),
                Op::Del(l) => ('-', *l),
                Op::Add(l) => ('+', *l),
            };
            out.push(mark);
            out.push_str(line);
            if !line.ends_with('\n') {
                out.push_str("\n\\ No newline at end of file\n");
            }
        }
        i += 1;
    }
    out
}

enum Op<'a> {
    Same(&'a str),
    Del(&'a str),
    Add(&'a str),
}

/// A longest-common-subsequence line diff; past a size ceiling, one hunk
/// replacing the differing middle.
fn line_ops<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<Op<'a>> {
    let prefix = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (am, bm) = (&a[prefix..a.len() - suffix], &b[prefix..b.len() - suffix]);
    let mut ops: Vec<Op<'a>> = a[..prefix]
        .iter()
        .zip(b)
        .map(|(x, _)| Op::Same(x))
        .collect();
    if am.len().saturating_mul(bm.len()) > 4_000_000 {
        ops.extend(am.iter().map(|l| Op::Del(l)));
        ops.extend(bm.iter().map(|l| Op::Add(l)));
    } else {
        let (n, m) = (am.len(), bm.len());
        let mut table = vec![0u32; (n + 1) * (m + 1)];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                table[i * (m + 1) + j] = if am[i] == bm[j] {
                    table[(i + 1) * (m + 1) + j + 1] + 1
                } else {
                    table[(i + 1) * (m + 1) + j].max(table[i * (m + 1) + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < n || j < m {
            if i < n && j < m && am[i] == bm[j] {
                ops.push(Op::Same(am[i]));
                i += 1;
                j += 1;
            } else if i < n
                && (j == m || table[(i + 1) * (m + 1) + j] >= table[i * (m + 1) + j + 1])
            {
                ops.push(Op::Del(am[i]));
                i += 1;
            } else {
                ops.push(Op::Add(bm[j]));
                j += 1;
            }
        }
    }
    ops.extend(
        a[a.len() - suffix..]
            .iter()
            .zip(&b[b.len() - suffix..])
            .map(|(x, _)| Op::Same(x)),
    );
    ops
}

fn status_name(s: Status) -> &'static str {
    match s {
        Status::Draft => "draft",
        Status::Approved => "approved",
        Status::Superseded => "superseded",
        Status::Retired => "retired",
    }
}

fn digest_of<T: Serialize>(value: &T) -> Result<String, Error> {
    Ok(sha256_bytes(canonical_json::to_string(value)?.as_bytes()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!(
        "sha256:{}",
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trace_id_is_read_as_the_quoted_string_flow_renders() {
        let line = flow(&serde_json::json!({ "id": "a, b", "relation": "implements" }));
        assert_eq!(trace_id(&line).as_deref(), Some("a, b"));
        let line = flow(&serde_json::json!({ "id": "T-1", "relation": "implements" }));
        assert_eq!(trace_id(&line).as_deref(), Some("T-1"));
        assert_eq!(trace_id("{ kind: file }"), None);
    }

    #[test]
    fn a_quoted_ampersand_is_text_and_a_bare_one_is_an_anchor() {
        assert!(!has_anchor_or_alias("- 'a &b *c'"));
        assert!(!has_anchor_or_alias("- 'it''s &b'"));
        assert!(!has_anchor_or_alias("- \"a &b\""));
        assert!(has_anchor_or_alias("- &x \"a\""));
        assert!(has_anchor_or_alias("- &_root \"a\""));
        assert!(has_anchor_or_alias("- { path: 'a', ref: *-x }"));
        assert!(!has_anchor_or_alias("- { note: a & b }"));
        assert!(has_anchor_or_alias("- { path: 'a', ref: *x }"));
        // An apostrophe inside a plain word opens no scalar to hide behind.
        assert!(has_anchor_or_alias("- { note: it's, ref: *x }"));
    }

    #[test]
    fn a_frontmatter_entry_appends_to_its_block_or_adds_the_key() {
        let text = "---\nid: \"001-a\"\nestablishes:\n  - { kind: file, path: \"a\" }\nsummary: \"s\"\n---\n# A\n";
        let out = insert_frontmatter(text, "establishes", "{ kind: file, path: \"b\" }").unwrap();
        assert!(
            out.contains(
                "  - { kind: file, path: \"a\" }\n  - { kind: file, path: \"b\" }\nsummary"
            )
        );
        let out = insert_frontmatter(text, "extends", "{ spec: \"x\" }").unwrap();
        assert!(out.contains("summary: \"s\"\nextends:\n  - { spec: \"x\" }\n---\n"));
        let flow_seq = "---\nid: \"a\"\nestablishes: [\"a\"]\n---\n";
        assert!(frontmatter_slot(flow_seq, "establishes").is_err());
        let commented = "---\nid: \"a\"\nestablishes:\n  # note\n  - \"a\"\n---\n";
        assert!(frontmatter_slot(commented, "establishes").is_err());
        let anchored = "---\nid: \"a\"\nestablishes:\n  - &x \"a\"\n---\n";
        assert!(frontmatter_slot(anchored, "establishes").is_err());
        let quoted = "---\nid: \"a\"\nestablishes:\n  - \"a &b\"\n---\n";
        assert!(frontmatter_slot(quoted, "establishes").is_ok());
    }

    #[test]
    fn flow_rendering_quotes_strings_and_keeps_vocabulary_bare() {
        let v = serde_json::json!({ "kind": "file", "path": "src/a.rs", "planned": true });
        assert_eq!(
            flow(&v),
            "{ kind: file, path: \"src/a.rs\", planned: true }"
        );
        assert_eq!(scalar("true"), "\"true\"");
        assert_eq!(scalar("additive"), "additive");
    }

    #[test]
    fn a_unified_diff_names_its_hunks() {
        let d = unified_diff("f.md", "a\nb\nc\n", "a\nB\nc\n");
        assert_eq!(
            d,
            "--- a/f.md\n+++ b/f.md\n@@ -1,3 +1,3 @@\n a\n-b\n+B\n c\n"
        );
        let created = unified_diff("n.md", "", "x\n");
        assert_eq!(created, "--- /dev/null\n+++ b/n.md\n@@ -0,0 +1,1 @@\n+x\n");
        assert_eq!(unified_diff("f", "same\n", "same\n"), "");
    }
}
