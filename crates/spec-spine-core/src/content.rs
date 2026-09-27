//! Bounded selected content from one caller-bound repository snapshot.

use std::fs;
use std::path::Path;

use spec_spine_types::{
    CoalescedSelector, Config, ContentCompleteness, ContentContinuation, ContentItem,
    ContentOmission, ContentOmissionReason, ContentProjection, ContentRequest, ContentResolution,
    ContentResponse, ContentSelector, ContentSnapshot, ContentSpan, Error, READ_SCHEMA_VERSION,
    RepoPath, ResolvedLocation, split_obligation_ref,
};

use crate::index::Freshness;
use crate::snapshot::{PieceKind, PieceSet};
use crate::{
    Versioning, canonical_json, check_registry_freshness, guard_committed_index, hash,
    load_committed_index, load_committed_registry, read_document, resolve_spec_id,
};

#[derive(Clone)]
struct Candidate {
    position: usize,
    identity: String,
    projection: ContentProjection,
    required: bool,
    selector: ContentSelector,
    resolved: Result<Selected, (ContentOmissionReason, String)>,
}

#[derive(Clone)]
struct Selected {
    path: RepoPath,
    span: ContentSpan,
    content: String,
}

type Selection = Result<Selected, (ContentOmissionReason, String)>;

/// Resolve a bounded content request without Git, network access, execution, or writes.
pub fn selected_content(
    cfg: &Config,
    repo_root: &Path,
    request: &ContentRequest,
    snapshot: &ContentSnapshot,
) -> Result<ContentResponse, Error> {
    validate(request, snapshot)?;
    crate::pathutil::refuse_links_leaving(cfg, repo_root)?;

    let needs_registry = request.selectors.iter().any(|s| {
        matches!(
            s,
            ContentSelector::Spec { .. }
                | ContentSelector::SpecSection { .. }
                | ContentSelector::Obligation { .. }
                | ContentSelector::OwnedUnit { .. }
        )
    });
    let needs_index = request.selectors.iter().any(|s| {
        matches!(
            s,
            ContentSelector::OwnedUnit { .. }
                | ContentSelector::SpecSection { .. }
                | ContentSelector::Symbol { .. }
                | ContentSelector::Module { .. }
                | ContentSelector::Test { .. }
        )
    });

    let registry = if needs_registry {
        match check_registry_freshness(cfg, repo_root)? {
            Freshness::Fresh => Some(load_committed_registry(cfg, repo_root)?),
            Freshness::Stale { expected, actual } => return Err(Error::Stale { expected, actual }),
        }
    } else {
        None
    };
    let index = if needs_index {
        guard_committed_index(cfg, repo_root)?;
        Some(load_committed_index(cfg, repo_root)?)
    } else {
        None
    };

    let mut candidates = Vec::new();
    for (position, selector) in request.selectors.iter().enumerate() {
        let projection = selector.projection(request.default_projection);
        let (identity, resolved) = resolve_selector(
            cfg,
            repo_root,
            selector,
            projection,
            registry.as_ref(),
            index.as_ref(),
            request.max_bytes,
        )?;
        candidates.push(Candidate {
            position,
            identity,
            projection,
            required: selector.required(),
            selector: selector.clone(),
            resolved,
        });
    }
    candidates.sort_by(|a, b| {
        (&a.identity, a.projection, a.position).cmp(&(&b.identity, b.projection, b.position))
    });

    let request_digest = request_digest(request)?;
    let snapshot_digest = snapshot_digest(snapshot)?;
    let mut unique: Vec<Candidate> = Vec::new();
    let mut coalesced = Vec::new();
    for candidate in candidates {
        if let Some(previous) = unique.last_mut()
            && previous.identity == candidate.identity
            && previous.projection == candidate.projection
        {
            previous.required |= candidate.required;
            let entry = coalesced.last_mut().filter(|e: &&mut CoalescedSelector| {
                e.identity == candidate.identity && e.requested_projection == candidate.projection
            });
            if let Some(entry) = entry {
                entry.positions.push(candidate.position);
            } else {
                coalesced.push(CoalescedSelector {
                    identity: candidate.identity.clone(),
                    requested_projection: candidate.projection,
                    positions: vec![previous.position, candidate.position],
                });
            }
            continue;
        }
        unique.push(candidate);
    }

    let resolved_order: Vec<&Candidate> = unique.iter().filter(|c| c.resolved.is_ok()).collect();
    let start = continuation_start(
        request.continuation.as_ref(),
        &request_digest,
        &snapshot_digest,
        &resolved_order,
    )?;
    let mut items = Vec::new();
    let mut omissions = Vec::new();
    let mut bytes = 0usize;
    let mut next = None;

    for candidate in unique.iter().filter(|c| c.resolved.is_err()) {
        let Err((reason, message)) = &candidate.resolved else {
            unreachable!()
        };
        omissions.push(ContentOmission {
            identity: candidate.identity.clone(),
            requested_projection: candidate.projection,
            required: candidate.required,
            reason: *reason,
            message: message.clone(),
        });
    }
    for (ordinal, candidate) in resolved_order.iter().enumerate().skip(start) {
        let selected = candidate
            .resolved
            .as_ref()
            .expect("filtered resolved candidate");
        let size = selected.content.len();
        if size > request.max_bytes {
            omissions.push(ContentOmission {
                identity: candidate.identity.clone(),
                requested_projection: candidate.projection,
                required: candidate.required,
                reason: ContentOmissionReason::ItemExceedsByteBudget,
                message: format!(
                    "selected item is {size} bytes, above maxBytes {}",
                    request.max_bytes
                ),
            });
            continue;
        }
        if items.len() == request.max_items || bytes + size > request.max_bytes {
            next = Some(ContentContinuation {
                request_digest: request_digest.clone(),
                snapshot_digest: snapshot_digest.clone(),
                next_identity: candidate.identity.clone(),
                next_ordinal: ordinal,
            });
            break;
        }
        bytes += size;
        items.push(make_item(candidate, selected, snapshot));
    }
    omissions.sort_by(|a, b| {
        (&a.identity, a.requested_projection).cmp(&(&b.identity, b.requested_projection))
    });
    let completeness = if !omissions.is_empty() {
        ContentCompleteness::Incomplete
    } else if next.is_some() {
        ContentCompleteness::Partial
    } else {
        ContentCompleteness::Complete
    };
    Ok(ContentResponse {
        repository: snapshot.repository.clone(),
        revision: snapshot.revision.clone(),
        tree: snapshot.tree.clone(),
        dirty_state: snapshot.dirty_state,
        completeness,
        items,
        omissions,
        coalesced_selectors: coalesced,
        continuation: next,
    })
}

/// JSON facade for [`selected_content`].
pub fn selected_content_json(
    config_json: &str,
    repo_root: &str,
    request_json: &str,
    snapshot_json: &str,
) -> Result<String, Error> {
    let cfg: Config = serde_json::from_str(config_json)
        .map_err(|e| Error::Usage(format!("invalid config JSON: {e}")))?;
    let request: ContentRequest = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid content request: {e}")))?;
    let snapshot: ContentSnapshot = serde_json::from_str(snapshot_json)
        .map_err(|e| Error::Usage(format!("invalid content snapshot: {e}")))?;
    let response = selected_content(&cfg, Path::new(repo_root), &request, &snapshot)?;
    read_document(&response, Versioning::Stamp)
}

fn validate(request: &ContentRequest, snapshot: &ContentSnapshot) -> Result<(), Error> {
    if request.selectors.is_empty() {
        return Err(Error::Usage("content selectors must not be empty".into()));
    }
    if !(1..=1_048_576).contains(&request.max_bytes) {
        return Err(Error::Usage(
            "maxBytes must be from 1 through 1048576".into(),
        ));
    }
    if !(1..=256).contains(&request.max_items) {
        return Err(Error::Usage("maxItems must be from 1 through 256".into()));
    }
    if snapshot.repository.trim().is_empty() {
        return Err(Error::Usage("snapshot repository must not be empty".into()));
    }
    for (name, id) in [("revision", &snapshot.revision), ("tree", &snapshot.tree)] {
        if !matches!(id.len(), 40 | 64)
            || !id
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(Error::Usage(format!(
                "snapshot {name} must be a full lowercase object identity"
            )));
        }
    }
    Ok(())
}

fn resolve_selector(
    cfg: &Config,
    root: &Path,
    selector: &ContentSelector,
    projection: ContentProjection,
    registry: Option<&spec_spine_types::Registry>,
    index: Option<&spec_spine_types::CodebaseIndex>,
    max_bytes: usize,
) -> Result<(String, Selection), Error> {
    let missing = |m: String| Err((ContentOmissionReason::MissingContent, m));
    let unsupported = |m: String| Err((ContentOmissionReason::UnsupportedProjection, m));
    match selector {
        ContentSelector::File { path, .. } => {
            let path = normalize_repo_path(path)?;
            let identity = format!("file:{}", path.as_str());
            if projection != ContentProjection::Full {
                return Ok((
                    identity,
                    unsupported("explicit files support only the full projection".into()),
                ));
            }
            Ok((identity, read_path(cfg, root, &path, None, Some(max_bytes))))
        }
        ContentSelector::DirectoryMember {
            directory, member, ..
        } => {
            let directory = normalize_repo_path(directory)?;
            let member = normalize_repo_path(member)?;
            let joined = [directory.as_str(), member.as_str()]
                .into_iter()
                .filter(|component| !component.is_empty())
                .collect::<Vec<_>>()
                .join("/");
            let path = RepoPath::parse(&joined).map_err(Error::Refused)?;
            let identity = format!(
                "directory-member:{}#{}",
                directory.as_str(),
                member.as_str()
            );
            if projection != ContentProjection::Full {
                return Ok((
                    identity,
                    unsupported("directory members support only the full projection".into()),
                ));
            }
            Ok((identity, read_path(cfg, root, &path, None, Some(max_bytes))))
        }
        ContentSelector::Spec { spec, .. } => {
            let record = resolve_record(spec, registry.expect("registry loaded"))?;
            let identity = format!("spec:{}", record.id);
            let path = RepoPath::parse(&record.spec_path).map_err(Error::Internal)?;
            let selected = match projection {
                ContentProjection::Full => read_path(cfg, root, &path, None, Some(max_bytes)),
                ContentProjection::Body => {
                    read_path(cfg, root, &path, None, None).and_then(spec_body)
                }
                _ => unsupported("a spec supports only full and body projections".into()),
            };
            Ok((identity, selected))
        }
        ContentSelector::SpecSection { spec, anchor, .. } => {
            let record = resolve_record(spec, registry.expect("registry loaded"))?;
            let identity = format!("section:{}#{anchor}", record.id);
            if !matches!(
                projection,
                ContentProjection::Full | ContentProjection::Body
            ) {
                return Ok((
                    identity,
                    unsupported("a spec section supports full and body projections".into()),
                ));
            }
            let path = RepoPath::parse(&record.spec_path).map_err(Error::Internal)?;
            let full = read_path(cfg, root, &path, None, None);
            let selected = full.and_then(|s| {
                let span = crate::sections::resolve_section(&s.content, path.as_str(), anchor)
                    .ok_or((
                        ContentOmissionReason::MissingContent,
                        format!("section '{anchor}' does not exist"),
                    ))?;
                Ok(select_lines(
                    s.path,
                    &s.content,
                    span.start_line,
                    span.end_line,
                ))
            });
            Ok((identity, selected))
        }
        ContentSelector::Obligation { obligation, .. } => {
            let Some((spec, id)) = split_obligation_ref(obligation) else {
                return Err(Error::Usage(format!(
                    "invalid qualified obligation '{obligation}'"
                )));
            };
            let record = resolve_record(spec, registry.expect("registry loaded"))?;
            let identity = format!("obligation:{}#{id}", record.id);
            let Some(item) = record
                .obligations
                .iter()
                .find(|o| o.id == id && !o.withdrawn)
            else {
                return Ok((
                    identity,
                    missing(format!("obligation '{id}' does not exist")),
                ));
            };
            if !matches!(
                projection,
                ContentProjection::Full | ContentProjection::Declaration
            ) {
                return Ok((
                    identity,
                    unsupported("an obligation supports full and declaration projections".into()),
                ));
            }
            let path = RepoPath::parse(&record.spec_path).map_err(Error::Internal)?;
            let selected =
                read_path(cfg, root, &path, None, None).and_then(|s| obligation_entry(s, &item.id));
            Ok((identity, selected))
        }
        ContentSelector::OwnedUnit { spec, unit, .. } => {
            let record = resolve_record(spec, registry.expect("registry loaded"))?;
            let unit_json = serde_json::to_string(&unit.subject())
                .map_err(|e| Error::Internal(e.to_string()))?;
            let identity = format!("owned-unit:{}#{unit_json}", record.id);
            if projection != ContentProjection::Full {
                return Ok((
                    identity,
                    unsupported("owned units support only the full projection".into()),
                ));
            }
            let mapping = index
                .expect("index loaded")
                .traceability
                .mappings
                .iter()
                .find(|m| m.spec_id == record.id);
            let locations: Vec<ResolvedLocation> = mapping
                .into_iter()
                .flat_map(|m| &m.resolved_units)
                .filter(|r| r.ownership && r.unit.subject() == unit.subject())
                .flat_map(|r| r.locations.clone())
                .collect();
            if locations.len() > 1 {
                return Err(Error::Refused(format!(
                    "owned unit selector '{identity}' is ambiguous"
                )));
            }
            let selected = locations.first().map_or_else(
                || missing("owned unit does not resolve".into()),
                |l| read_location(cfg, root, l, max_bytes),
            );
            Ok((identity, selected))
        }
        ContentSelector::Symbol { id, .. }
        | ContentSelector::Module { id, .. }
        | ContentSelector::Test { id, .. } => {
            let kind = match selector {
                ContentSelector::Symbol { .. } => "symbol",
                ContentSelector::Module { .. } => "module",
                _ => "test",
            };
            let identity = format!("{kind}:{id}");
            #[cfg(feature = "symbol-resolution")]
            let locations = {
                let idx = index.expect("index loaded");
                if kind == "module" {
                    crate::symbols::build_module_index(
                        root,
                        &idx.packages,
                        &cfg.index.resolver_exclusions,
                        &cfg.layout,
                    )
                    .resolve(id)
                } else {
                    crate::symbols::build_symbol_index(
                        root,
                        &idx.packages,
                        &cfg.index.resolver_exclusions,
                        &cfg.layout,
                    )
                    .resolve(id)
                }
            };
            #[cfg(not(feature = "symbol-resolution"))]
            let locations: Vec<ResolvedLocation> = Vec::new();
            if locations.len() > 1 {
                return Err(Error::Refused(format!(
                    "{kind} selector '{id}' is ambiguous"
                )));
            }
            let Some(location) = locations.first() else {
                return Ok((
                    identity,
                    Err((
                        if kind == "test" {
                            ContentOmissionReason::UnsupportedSelector
                        } else {
                            ContentOmissionReason::MissingContent
                        },
                        format!("{kind} '{id}' does not resolve"),
                    )),
                ));
            };
            let selected = read_structural(cfg, root, location, projection, kind == "test");
            Ok((identity, selected))
        }
    }
}

fn normalize_repo_path(path: &RepoPath) -> Result<RepoPath, Error> {
    let normalized = path
        .as_str()
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".")
        .collect::<Vec<_>>()
        .join("/");
    RepoPath::parse(&normalized).map_err(Error::Refused)
}

fn resolve_record<'a>(
    id: &str,
    registry: &'a spec_spine_types::Registry,
) -> Result<&'a spec_spine_types::SpecRecord, Error> {
    let full = resolve_spec_id(id, registry.specs.iter().map(|s| s.id.as_str()))?;
    registry
        .specs
        .iter()
        .find(|s| s.id == full)
        .ok_or(Error::NotFound(full))
}

fn read_location(
    cfg: &Config,
    root: &Path,
    location: &ResolvedLocation,
    max_bytes: usize,
) -> Result<Selected, (ContentOmissionReason, String)> {
    let path =
        RepoPath::parse(&location.file).map_err(|m| (ContentOmissionReason::MissingContent, m))?;
    read_path(
        cfg,
        root,
        &path,
        location.span.map(|s| (s.start_line, s.end_line)),
        location.span.is_none().then_some(max_bytes),
    )
}

fn read_path(
    cfg: &Config,
    root: &Path,
    path: &RepoPath,
    span: Option<(usize, usize)>,
    whole_item_budget: Option<usize>,
) -> Result<Selected, (ContentOmissionReason, String)> {
    let target = root.join(path.as_str());
    if path.as_str().is_empty()
        || path.as_str() == "."
        || crate::pathutil::is_excluded(root, &target, &cfg.index.resolver_exclusions)
        || path
            .as_str()
            .split('/')
            .any(|component| matches!(component, ".git" | ".statecraft"))
    {
        return Err((
            ContentOmissionReason::MissingContent,
            format!("path '{}' is excluded", path.as_str()),
        ));
    }
    let root_real = fs::canonicalize(root)
        .map_err(|e| (ContentOmissionReason::MissingContent, e.to_string()))?;
    let target_real = fs::canonicalize(&target).map_err(|_| {
        (
            ContentOmissionReason::MissingContent,
            format!("path '{}' does not exist", path.as_str()),
        )
    })?;
    if !target_real.starts_with(&root_real) {
        return Err((
            ContentOmissionReason::MissingContent,
            "path escapes the repository".into(),
        ));
    }
    if !target_real.is_file() {
        return Err((
            ContentOmissionReason::MissingContent,
            format!("path '{}' is not a file", path.as_str()),
        ));
    }
    if let Some(max_bytes) = whole_item_budget {
        let raw_len = fs::metadata(&target_real)
            .map_err(|e| (ContentOmissionReason::MissingContent, e.to_string()))?
            .len();
        let largest_raw_that_can_fit = max_bytes.saturating_mul(2).saturating_add(3) as u64;
        if raw_len > largest_raw_that_can_fit {
            return Err((
                ContentOmissionReason::ItemExceedsByteBudget,
                format!(
                    "path '{}' cannot fit within maxBytes {max_bytes}",
                    path.as_str()
                ),
            ));
        }
    }
    let bytes = fs::read(&target_real)
        .map_err(|e| (ContentOmissionReason::MissingContent, e.to_string()))?;
    let raw = String::from_utf8(bytes).map_err(|_| {
        (
            ContentOmissionReason::BinaryContent,
            format!("path '{}' is not UTF-8", path.as_str()),
        )
    })?;
    let content = hash::normalize(&raw);
    let line_count = content.lines().count().max(1);
    let (start, end) = span.unwrap_or((1, line_count));
    Ok(select_lines(path.clone(), &content, start, end))
}

fn select_lines(path: RepoPath, content: &str, start: usize, end: usize) -> Selected {
    let mut text = content
        .lines()
        .skip(start.saturating_sub(1))
        .take(end.saturating_sub(start) + 1)
        .collect::<Vec<_>>()
        .join("\n");
    if !text.is_empty() && content.ends_with('\n') && end == content.lines().count() {
        text.push('\n');
    }
    Selected {
        path,
        span: ContentSpan {
            start_line: start,
            end_line: end.max(start),
        },
        content: text,
    }
}

fn spec_body(selected: Selected) -> Result<Selected, (ContentOmissionReason, String)> {
    let lines: Vec<&str> = selected.content.lines().collect();
    let Some(close) = lines
        .iter()
        .skip(1)
        .position(|l| *l == "---")
        .map(|i| i + 1)
    else {
        return Err((
            ContentOmissionReason::MissingContent,
            "spec frontmatter is malformed".into(),
        ));
    };
    Ok(select_lines(
        selected.path,
        &selected.content,
        close + 2,
        lines.len().max(close + 2),
    ))
}

fn obligation_entry(
    selected: Selected,
    id: &str,
) -> Result<Selected, (ContentOmissionReason, String)> {
    let lines: Vec<&str> = selected.content.lines().collect();
    let needle = format!("- id: \"{id}\"");
    let alt = format!("- id: '{id}'");
    let Some(start0) = lines
        .iter()
        .position(|l| l.trim() == needle || l.trim() == alt || l.trim() == format!("- id: {id}"))
    else {
        return Err((
            ContentOmissionReason::MissingContent,
            "obligation declaration was not found".into(),
        ));
    };
    let mut end0 = start0;
    for (i, line) in lines.iter().enumerate().skip(start0 + 1) {
        if line.starts_with("  - id:") || *line == "---" {
            break;
        }
        end0 = i;
    }
    Ok(select_lines(
        selected.path,
        &selected.content,
        start0 + 1,
        end0 + 1,
    ))
}

fn structural_projection(
    selected: Selected,
    projection: ContentProjection,
    test: bool,
) -> Result<Selected, (ContentOmissionReason, String)> {
    if projection == ContentProjection::Full {
        return if test && !has_test_attribute(&selected) {
            Err((
                ContentOmissionReason::UnsupportedSelector,
                "test function lacks a supported test attribute".into(),
            ))
        } else {
            Ok(selected)
        };
    }
    if test && !has_test_attribute(&selected) {
        return Err((
            ContentOmissionReason::UnsupportedSelector,
            "test function lacks a supported test attribute".into(),
        ));
    }
    let lines: Vec<&str> = selected.content.lines().collect();
    match projection {
        ContentProjection::Declaration | ContentProjection::Signature => {
            let end = lines
                .iter()
                .position(|l| l.contains('{') || l.trim_end().ends_with(';'))
                .unwrap_or(0);
            Ok(relative_lines(&selected, 0, end))
        }
        ContentProjection::Documentation => {
            unreachable!("documentation projection is handled before structural projection")
        }
        ContentProjection::Body => {
            let Some(open) = lines.iter().position(|l| l.contains('{')) else {
                return Err((
                    ContentOmissionReason::UnsupportedProjection,
                    "item has no structural body".into(),
                ));
            };
            Ok(relative_lines(
                &selected,
                open,
                lines.len().saturating_sub(1),
            ))
        }
        ContentProjection::Full => unreachable!(),
    }
}

fn read_structural(
    cfg: &Config,
    root: &Path,
    location: &ResolvedLocation,
    projection: ContentProjection,
    test: bool,
) -> Result<Selected, (ContentOmissionReason, String)> {
    let path =
        RepoPath::parse(&location.file).map_err(|m| (ContentOmissionReason::MissingContent, m))?;
    let span = location.span.ok_or((
        ContentOmissionReason::UnsupportedProjection,
        "structural selector has no bounded span".into(),
    ))?;
    let whole = read_path(cfg, root, &path, None, None)?;
    let lines: Vec<&str> = whole.content.lines().collect();
    if projection == ContentProjection::Documentation {
        let Some((start, end)) = attached_documentation_span(&lines, span.start_line, test) else {
            return Err((
                ContentOmissionReason::MissingContent,
                "no attached documentation comments".into(),
            ));
        };
        return Ok(select_lines(path, &whole.content, start, end));
    }
    let mut start = span.start_line;
    if test {
        while start > 1
            && lines
                .get(start - 2)
                .is_some_and(|l| l.trim_start().starts_with("#["))
        {
            start -= 1;
        }
    }
    let selected = select_lines(path, &whole.content, start, span.end_line);
    structural_projection(selected, projection, test)
}

fn attached_documentation_span(
    lines: &[&str],
    item_start_line: usize,
    skip_attributes: bool,
) -> Option<(usize, usize)> {
    let mut cursor = item_start_line.saturating_sub(1);
    if skip_attributes {
        while cursor > 0 && lines[cursor - 1].trim_start().starts_with("#[") {
            cursor -= 1;
        }
    }
    let end = cursor;
    while cursor > 0 {
        let line = lines[cursor - 1].trim_start();
        if line.starts_with("///")
            || line.starts_with("//!")
            || line.starts_with("/**")
            || line.starts_with('*')
            || line.starts_with("*/")
        {
            cursor -= 1;
        } else {
            break;
        }
    }
    (cursor < end).then_some((cursor + 1, end))
}

fn relative_lines(selected: &Selected, start_offset: usize, end_offset: usize) -> Selected {
    let mut relative = select_lines(
        selected.path.clone(),
        &selected.content,
        start_offset + 1,
        end_offset + 1,
    );
    relative.span = ContentSpan {
        start_line: selected.span.start_line + start_offset,
        end_line: selected.span.start_line + end_offset,
    };
    relative
}

fn has_test_attribute(selected: &Selected) -> bool {
    selected.content.contains("#[test]") || selected.content.contains("#[tokio::test]")
}

fn make_item(
    candidate: &Candidate,
    selected: &Selected,
    snapshot: &ContentSnapshot,
) -> ContentItem {
    let mut set = PieceSet::new();
    set.insert(
        "identity".into(),
        PieceKind::Text,
        candidate.identity.as_bytes().to_vec(),
    );
    set.insert(
        "projection".into(),
        PieceKind::Text,
        serde_json::to_string(&candidate.projection)
            .unwrap_or_default()
            .into_bytes(),
    );
    set.insert(
        "path".into(),
        PieceKind::Text,
        selected.path.as_str().as_bytes().to_vec(),
    );
    set.insert(
        "span".into(),
        PieceKind::Text,
        serde_json::to_string(&selected.span)
            .unwrap_or_default()
            .into_bytes(),
    );
    set.insert(
        "content".into(),
        PieceKind::Text,
        selected.content.as_bytes().to_vec(),
    );
    ContentItem {
        identity: candidate.identity.clone(),
        repository: snapshot.repository.clone(),
        revision: snapshot.revision.clone(),
        tree: snapshot.tree.clone(),
        dirty_state: snapshot.dirty_state,
        selector: candidate.selector.clone(),
        requested_projection: candidate.projection,
        path: selected.path.clone(),
        span: selected.span,
        digest: format!("sha256:{}", set.digest().expect("nonempty framed item")),
        content: selected.content.clone(),
        resolution: ContentResolution::Resolved,
        completeness: ContentCompleteness::Complete,
        schema_version: READ_SCHEMA_VERSION.to_string(),
    }
}

fn request_digest(request: &ContentRequest) -> Result<String, Error> {
    let mut request = request.clone();
    request.continuation = None;
    digest_json("request", &request)
}
fn snapshot_digest(snapshot: &ContentSnapshot) -> Result<String, Error> {
    digest_json(
        "snapshot",
        &serde_json::json!({
            "repository": snapshot.repository,
            "revision": snapshot.revision,
            "tree": snapshot.tree,
            "dirtyState": snapshot.dirty_state,
        }),
    )
}
fn digest_json<T: serde::Serialize>(name: &str, value: &T) -> Result<String, Error> {
    let mut set = PieceSet::new();
    set.insert(
        name.into(),
        PieceKind::Text,
        canonical_json::to_string(value)?.into_bytes(),
    );
    Ok(format!(
        "sha256:{}",
        set.digest().expect("one-piece digest")
    ))
}

fn continuation_start(
    continuation: Option<&ContentContinuation>,
    request_digest: &str,
    snapshot_digest: &str,
    resolved: &[&Candidate],
) -> Result<usize, Error> {
    let Some(c) = continuation else {
        return Ok(0);
    };
    if c.request_digest != request_digest
        || c.snapshot_digest != snapshot_digest
        || resolved
            .get(c.next_ordinal)
            .is_none_or(|item| item.identity != c.next_identity)
    {
        return Err(Error::Refused(
            "stale-continuation: request, snapshot, or ordered content changed".into(),
        ));
    }
    Ok(c.next_ordinal)
}

#[cfg(test)]
mod tests {
    use super::attached_documentation_span;

    #[test]
    fn test_documentation_skips_attached_attributes() {
        let lines = [
            "/// Explains the test.",
            "#[cfg(unix)]",
            "#[test]",
            "fn works() {}",
        ];
        assert_eq!(attached_documentation_span(&lines, 4, true), Some((1, 1)));
        assert_eq!(attached_documentation_span(&lines, 4, false), None);
    }
}
