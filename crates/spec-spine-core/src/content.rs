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
    load_committed_index, load_committed_registry, read_document,
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
pub(crate) struct Selected {
    pub(crate) path: RepoPath,
    pub(crate) span: ContentSpan,
    pub(crate) content: String,
}

pub(crate) type Selection = Result<Selected, (ContentOmissionReason, String)>;

struct SelectorContext<'a> {
    cfg: &'a Config,
    root: &'a Path,
    root_real: &'a Path,
    registry: Option<&'a spec_spine_types::Registry>,
    index: Option<&'a spec_spine_types::CodebaseIndex>,
    max_bytes: usize,
    #[cfg(feature = "symbol-resolution")]
    resolver: Option<&'a crate::symbols::Resolver>,
}

/// Resolve a bounded content request without Git, network access, execution, or writes.
pub fn selected_content(
    cfg: &Config,
    repo_root: &Path,
    request: &ContentRequest,
    snapshot: &ContentSnapshot,
) -> Result<ContentResponse, Error> {
    validate(request, snapshot)?;
    let resolutions = resolve_selectors(
        cfg,
        repo_root,
        &request.selectors,
        request.default_projection,
        request.max_bytes,
    )?;
    let mut candidates: Vec<Candidate> = resolutions
        .into_iter()
        .enumerate()
        .map(|(position, r)| Candidate {
            position,
            required: r.selector.required(),
            identity: r.identity,
            projection: r.projection,
            selector: r.selector,
            resolved: r.outcome,
        })
        .collect();
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

/// Bind one selector structurally, through the resolver [`selected_content`]
/// uses, without a snapshot, a budget, or any content leaving this function
/// (spec 169 §3.5). `Ok(Ok(()))` binds; `Ok(Err(..))` carries the omission
/// reason a request would have reported. The caller supplies the committed
/// registry, and the committed index when the selector needs one.
pub(crate) fn bind_selector(
    cfg: &Config,
    repo_root: &Path,
    registry: &spec_spine_types::Registry,
    index: Option<&spec_spine_types::CodebaseIndex>,
    selector: &ContentSelector,
) -> Result<Result<(), (ContentOmissionReason, String)>, Error> {
    crate::pathutil::refuse_links_leaving(cfg, repo_root)?;
    let root_real = fs::canonicalize(repo_root).map_err(|error| Error::Io(error.to_string()))?;
    #[cfg(feature = "symbol-resolution")]
    let resolver = match (selector, index) {
        (
            ContentSelector::Symbol { .. }
            | ContentSelector::Module { .. }
            | ContentSelector::Test { .. },
            Some(idx),
        ) => Some(crate::symbols::build_resolver(
            repo_root,
            &idx.packages,
            &cfg.index.resolver_exclusions,
            &cfg.layout,
        )),
        _ => None,
    };
    let context = SelectorContext {
        cfg,
        root: repo_root,
        root_real: &root_real,
        registry: Some(registry),
        index,
        // The largest budget a request may name (spec 155), so a bind reads no
        // more than a request could.
        max_bytes: 1_048_576,
        #[cfg(feature = "symbol-resolution")]
        resolver: resolver.as_ref(),
    };
    let projection = selector.projection(ContentProjection::Full);
    let (_, selected) = resolve_selector(&context, selector, projection)?;
    Ok(selected.map(|_| ()))
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

/// One selector resolved against a repository export: the seam the context
/// packet (spec 159) shares with [`selected_content`], so a packet member and
/// a selected item can never name or read one selector differently.
pub(crate) struct Resolution {
    pub(crate) identity: String,
    pub(crate) projection: ContentProjection,
    pub(crate) selector: ContentSelector,
    pub(crate) outcome: Selection,
}

/// Resolve each selector, in the order given, against the export at
/// `repo_root`. Loads the committed registry and index only when a selector
/// needs them, refusing a stale ledger. `max_bytes` bounds every whole-file
/// read before it happens.
pub(crate) fn resolve_selectors(
    cfg: &Config,
    repo_root: &Path,
    selectors: &[ContentSelector],
    default_projection: ContentProjection,
    max_bytes: usize,
) -> Result<Vec<Resolution>, Error> {
    resolve_selectors_guarded(
        cfg,
        repo_root,
        selectors,
        default_projection,
        max_bytes,
        true,
    )
}

/// [`resolve_selectors`] for a caller that has already compared the committed
/// index with the corpus byte for byte (spec 166 3.9): the index is read
/// without [`guard_committed_index`], so a claim elsewhere that does not
/// resolve, which is not staleness, does not refuse this read. Each selector
/// still resolves, or is omitted, on its own.
pub(crate) fn resolve_selectors_after_freshness(
    cfg: &Config,
    repo_root: &Path,
    selectors: &[ContentSelector],
    default_projection: ContentProjection,
    max_bytes: usize,
) -> Result<Vec<Resolution>, Error> {
    resolve_selectors_guarded(
        cfg,
        repo_root,
        selectors,
        default_projection,
        max_bytes,
        false,
    )
}

fn resolve_selectors_guarded(
    cfg: &Config,
    repo_root: &Path,
    selectors: &[ContentSelector],
    default_projection: ContentProjection,
    max_bytes: usize,
    guard_index: bool,
) -> Result<Vec<Resolution>, Error> {
    crate::pathutil::refuse_links_leaving(cfg, repo_root)?;
    let root_real = fs::canonicalize(repo_root).map_err(|error| Error::Io(error.to_string()))?;

    let needs_registry = selectors.iter().any(|s| {
        matches!(
            s,
            ContentSelector::Spec { .. }
                | ContentSelector::SpecSection { .. }
                | ContentSelector::Obligation { .. }
                | ContentSelector::OwnedUnit { .. }
        )
    });
    let needs_index = selectors.iter().any(|s| {
        matches!(
            s,
            ContentSelector::OwnedUnit { .. }
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
        if guard_index {
            guard_committed_index(cfg, repo_root)?;
        }
        Some(load_committed_index(cfg, repo_root)?)
    } else {
        None
    };

    #[cfg(feature = "symbol-resolution")]
    let resolver = selectors
        .iter()
        .any(|s| {
            matches!(
                s,
                ContentSelector::Symbol { .. }
                    | ContentSelector::Module { .. }
                    | ContentSelector::Test { .. }
            )
        })
        .then(|| {
            let idx = index.as_ref().expect("index loaded");
            crate::symbols::build_resolver(
                repo_root,
                &idx.packages,
                &cfg.index.resolver_exclusions,
                &cfg.layout,
            )
        });
    let context = SelectorContext {
        cfg,
        root: repo_root,
        root_real: &root_real,
        registry: registry.as_ref(),
        index: index.as_ref(),
        max_bytes,
        #[cfg(feature = "symbol-resolution")]
        resolver: resolver.as_ref(),
    };
    let mut out = Vec::with_capacity(selectors.len());
    for selector in selectors {
        let projection = selector.projection(default_projection);
        let (identity, outcome) = resolve_selector(&context, selector, projection)?;
        out.push(Resolution {
            identity,
            projection,
            selector: selector.clone(),
            outcome,
        });
    }
    Ok(out)
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
    validate_snapshot(snapshot)
}

/// A snapshot names one repository and full lowercase object identities.
pub(crate) fn validate_snapshot(snapshot: &ContentSnapshot) -> Result<(), Error> {
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
    context: &SelectorContext<'_>,
    selector: &ContentSelector,
    projection: ContentProjection,
) -> Result<(String, Selection), Error> {
    let SelectorContext {
        cfg,
        root,
        root_real,
        registry,
        index,
        max_bytes,
        ..
    } = context;
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
            Ok((
                identity,
                read_path(cfg, root, root_real, &path, None, Some(*max_bytes)),
            ))
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
            require_directory_member(root, &directory, &path)?;
            Ok((
                identity,
                read_path(cfg, root, root_real, &path, None, Some(*max_bytes)),
            ))
        }
        ContentSelector::Spec { spec, .. } => {
            let Some(record) = resolve_record(spec, registry.expect("registry loaded"))? else {
                return Ok((
                    format!("spec:{spec}"),
                    missing(format!("spec '{spec}' does not exist")),
                ));
            };
            let identity = format!("spec:{}", record.id);
            let path = RepoPath::parse(&record.spec_path).map_err(Error::Internal)?;
            let selected = match projection {
                ContentProjection::Full => {
                    read_path(cfg, root, root_real, &path, None, Some(*max_bytes))
                }
                ContentProjection::Body => {
                    read_path(cfg, root, root_real, &path, None, None).and_then(spec_body)
                }
                _ => unsupported("a spec supports only full and body projections".into()),
            };
            Ok((identity, selected))
        }
        ContentSelector::SpecSection { spec, anchor, .. } => {
            let Some(record) = resolve_record(spec, registry.expect("registry loaded"))? else {
                return Ok((
                    format!("section:{spec}#{anchor}"),
                    missing(format!("spec '{spec}' does not exist")),
                ));
            };
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
            let full = read_path(cfg, root, root_real, &path, None, None);
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
            let Some(record) = resolve_record(spec, registry.expect("registry loaded"))? else {
                return Ok((
                    format!("obligation:{spec}#{id}"),
                    missing(format!("spec '{spec}' does not exist")),
                ));
            };
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
            let selected = read_path(cfg, root, root_real, &path, None, None)
                .and_then(|s| obligation_entry(s, &item.id));
            Ok((identity, selected))
        }
        ContentSelector::OwnedUnit { spec, unit, .. } => {
            let Some(record) = resolve_record(spec, registry.expect("registry loaded"))? else {
                let unit_json = canonical_json::to_string(&unit.subject())?;
                return Ok((
                    format!("owned-unit:{spec}#{unit_json}"),
                    missing(format!("spec '{spec}' does not exist")),
                ));
            };
            let unit_json = canonical_json::to_string(&unit.subject())?;
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
                return Ok((
                    identity,
                    Err((
                        ContentOmissionReason::UnsupportedSelector,
                        "owned unit resolves to more than one textual location".into(),
                    )),
                ));
            }
            let selected = locations.first().map_or_else(
                || missing("owned unit does not resolve".into()),
                |l| read_location(cfg, root, root_real, l, *max_bytes),
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
            let (lookup, is_symbol) = {
                use crate::symbols::Namespace;
                let ns = match kind {
                    "symbol" => Namespace::Symbol,
                    "module" => Namespace::Module,
                    _ => Namespace::Test,
                };
                let resolver = context.resolver.expect("resolver built");
                let is_symbol = ns == Namespace::Test
                    && matches!(
                        resolver.lookup(Namespace::Symbol, id),
                        crate::symbols::Lookup::Resolved(_)
                    );
                (resolver.lookup(ns, id), is_symbol)
            };
            #[cfg(not(feature = "symbol-resolution"))]
            let (lookup, is_symbol) = (
                crate::symbols::Lookup::Unknown(crate::symbols::reason::RESOLVER_DISABLED),
                false,
            );
            // Each outcome maps to exactly one omission (spec 163 §3.5), and
            // its reason token rides in the message.
            let locations = match lookup {
                crate::symbols::Lookup::Resolved(locations) => locations,
                crate::symbols::Lookup::Unsupported(reason) => {
                    return Ok((
                        identity,
                        Err((
                            ContentOmissionReason::UnsupportedSelector,
                            format!("{kind} '{id}' is unsupported: {reason}"),
                        )),
                    ));
                }
                crate::symbols::Lookup::Unknown(reason) => {
                    return Ok((
                        identity,
                        Err((
                            ContentOmissionReason::IndeterminateSelector,
                            format!("{kind} '{id}' is unknown: {reason}"),
                        )),
                    ));
                }
                crate::symbols::Lookup::Unresolved => {
                    let message = if is_symbol {
                        format!("{kind} '{id}' is not a test function")
                    } else {
                        format!("{kind} '{id}' does not resolve")
                    };
                    return Ok((
                        identity,
                        Err((ContentOmissionReason::MissingContent, message)),
                    ));
                }
            };
            if locations.len() > 1 {
                return Err(Error::Refused(format!(
                    "{kind} selector '{id}' is ambiguous"
                )));
            }
            let Some(location) = locations.first() else {
                return Ok((
                    identity,
                    Err((
                        ContentOmissionReason::MissingContent,
                        format!("{kind} '{id}' does not resolve"),
                    )),
                ));
            };
            let selected =
                read_structural(cfg, root, root_real, location, projection, kind == "test");
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
) -> Result<Option<&'a spec_spine_types::SpecRecord>, Error> {
    match crate::spec_id::match_spec_id(id, registry.specs.iter().map(|s| s.id.as_str())) {
        crate::spec_id::SpecIdMatch::Resolved(full) => {
            Ok(registry.specs.iter().find(|s| s.id == full))
        }
        crate::spec_id::SpecIdMatch::Ambiguous(candidates) => Err(Error::Refused(format!(
            "spec '{id}' is ambiguous: {} specs share that ordinal ({})",
            candidates.len(),
            candidates.join(", ")
        ))),
        crate::spec_id::SpecIdMatch::NoMatch => Ok(None),
    }
}

fn require_directory_member(
    root: &Path,
    directory: &RepoPath,
    path: &RepoPath,
) -> Result<(), Error> {
    let Ok(directory) = fs::canonicalize(root.join(directory.as_str())) else {
        return Ok(());
    };
    let Ok(target) = fs::canonicalize(root.join(path.as_str())) else {
        return Ok(());
    };
    if !target.starts_with(directory) {
        return Err(Error::Refused(format!(
            "path-escape: directory member '{}' leaves its named directory",
            path.as_str()
        )));
    }
    Ok(())
}

fn read_location(
    cfg: &Config,
    root: &Path,
    root_real: &Path,
    location: &ResolvedLocation,
    max_bytes: usize,
) -> Result<Selected, (ContentOmissionReason, String)> {
    let path =
        RepoPath::parse(&location.file).map_err(|m| (ContentOmissionReason::MissingContent, m))?;
    if root.join(path.as_str()).is_dir() {
        return Err((
            ContentOmissionReason::UnsupportedSelector,
            "directory owned units require a directory-member selector".into(),
        ));
    }
    read_path(
        cfg,
        root,
        root_real,
        &path,
        location.span.map(|s| (s.start_line, s.end_line)),
        location.span.is_none().then_some(max_bytes),
    )
}

fn read_path(
    cfg: &Config,
    root: &Path,
    root_real: &Path,
    path: &RepoPath,
    span: Option<(usize, usize)>,
    whole_item_budget: Option<usize>,
) -> Result<Selected, (ContentOmissionReason, String)> {
    let target = root.join(path.as_str());
    if path.as_str().is_empty()
        || path.as_str() == "."
        || crate::pathutil::is_excluded(root, &target, &cfg.index.resolver_exclusions)
        || path.as_str().split('/').any(|component| {
            component.eq_ignore_ascii_case(".git")
                || (!cfg.layout.state_dir.is_empty()
                    && component.eq_ignore_ascii_case(cfg.layout.state_dir.trim_matches('/')))
        })
    {
        return Err((
            ContentOmissionReason::MissingContent,
            format!("path '{}' is excluded", path.as_str()),
        ));
    }
    let target_real = fs::canonicalize(&target).map_err(|_| {
        (
            ContentOmissionReason::MissingContent,
            format!("path '{}' does not exist", path.as_str()),
        )
    })?;
    if !target_real.starts_with(root_real) {
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
    if span.is_some() {
        let raw_len = fs::metadata(&target_real)
            .map_err(|e| (ContentOmissionReason::MissingContent, e.to_string()))?
            .len();
        if raw_len > 8 * 1024 * 1024 {
            return Err((
                ContentOmissionReason::ItemExceedsByteBudget,
                format!(
                    "path '{}' exceeds the structural read ceiling",
                    path.as_str()
                ),
            ));
        }
    } else if let Some(max_bytes) = whole_item_budget {
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
    let Some(frontmatter_end) = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(i, line)| (*line == "---").then_some(i))
    else {
        return Err((
            ContentOmissionReason::MissingContent,
            "spec frontmatter is malformed".into(),
        ));
    };
    let Some(obligations_start) = lines[..frontmatter_end]
        .iter()
        .position(|line| line.trim() == "obligations:")
    else {
        return Err((
            ContentOmissionReason::MissingContent,
            "spec has no obligations".into(),
        ));
    };
    let needle = format!("- id: \"{id}\"");
    let alt = format!("- id: '{id}'");
    let Some(start0) = lines
        .iter()
        .enumerate()
        .take(frontmatter_end)
        .skip(obligations_start + 1)
        .find_map(|(i, l)| {
            (l.trim() == needle || l.trim() == alt || l.trim() == format!("- id: {id}"))
                .then_some(i)
        })
    else {
        return Err((
            ContentOmissionReason::MissingContent,
            "obligation declaration was not found".into(),
        ));
    };
    let mut end0 = start0;
    let indent = lines[start0].len() - lines[start0].trim_start().len();
    for (i, line) in lines
        .iter()
        .enumerate()
        .take(frontmatter_end)
        .skip(start0 + 1)
    {
        if line.trim().is_empty() {
            // A blank line belongs to the entry only when a deeper line follows
            // it, so a blank separating two entries never ends the selection.
            continue;
        }
        let line_indent = line.len() - line.trim_start().len();
        if line_indent <= indent {
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
    structural_spans: Option<(
        Option<spec_spine_types::LineSpan>,
        Option<spec_spine_types::LineSpan>,
    )>,
) -> Result<Selected, (ContentOmissionReason, String)> {
    if projection == ContentProjection::Full {
        return Ok(selected);
    }
    match projection {
        ContentProjection::Declaration => Err((
            ContentOmissionReason::UnsupportedProjection,
            "structural declarations are not separately defined".into(),
        )),
        ContentProjection::Signature => {
            let Some((Some(span), _)) = structural_spans else {
                return Err((
                    ContentOmissionReason::UnsupportedProjection,
                    "the grammar does not expose a line-bounded signature".into(),
                ));
            };
            let start = span.start_line - selected.span.start_line + 1;
            let end = span.end_line - selected.span.start_line + 1;
            let mut signature = select_lines(selected.path, &selected.content, start, end);
            signature.span = ContentSpan {
                start_line: span.start_line,
                end_line: span.end_line,
            };
            Ok(signature)
        }
        ContentProjection::Documentation => {
            unreachable!("documentation projection is handled before structural projection")
        }
        ContentProjection::Body => {
            let Some((_, Some(span))) = structural_spans else {
                return Err((
                    ContentOmissionReason::UnsupportedProjection,
                    "the grammar does not expose a line-bounded body".into(),
                ));
            };
            let start = span.start_line - selected.span.start_line + 1;
            let end = span.end_line - selected.span.start_line + 1;
            let mut body = select_lines(selected.path, &selected.content, start, end);
            body.span = ContentSpan {
                start_line: span.start_line,
                end_line: span.end_line,
            };
            Ok(body)
        }
        ContentProjection::Full => unreachable!(),
    }
}

fn read_structural(
    cfg: &Config,
    root: &Path,
    root_real: &Path,
    location: &ResolvedLocation,
    projection: ContentProjection,
    test: bool,
) -> Result<Selected, (ContentOmissionReason, String)> {
    let path =
        RepoPath::parse(&location.file).map_err(|m| (ContentOmissionReason::MissingContent, m))?;
    let Some(span) = location.span else {
        return if projection == ContentProjection::Full {
            read_path(cfg, root, root_real, &path, None, Some(1_048_576))
        } else {
            Err((
                ContentOmissionReason::UnsupportedProjection,
                "a file module supports only the full projection".into(),
            ))
        };
    };
    let whole = read_path(cfg, root, root_real, &path, None, Some(8 * 1024 * 1024))?;
    let lines: Vec<&str> = whole.content.lines().collect();
    if projection == ContentProjection::Documentation {
        return match documentation(&lines, span.start_line) {
            Documentation::Span(start, end) => Ok(select_lines(path, &whole.content, start, end)),
            Documentation::Attribute => Err((
                ContentOmissionReason::UnsupportedProjection,
                "attribute documentation".into(),
            )),
            Documentation::None => Err((
                ContentOmissionReason::MissingContent,
                "no attached documentation comments".into(),
            )),
        };
    }
    let selected = select_lines(path.clone(), &whole.content, span.start_line, span.end_line);
    #[cfg(feature = "symbol-resolution")]
    let spans = crate::symbols::structural_spans(
        &whole.content,
        path.as_str()
            .rsplit_once('.')
            .map(|(_, ext)| ext)
            .unwrap_or(""),
        // A test's span opens on its attribute run, so its function is the
        // item ending on the span's last line (spec 163 D-15).
        if test {
            crate::symbols::SpanAnchor::EndsOn(span.end_line)
        } else {
            crate::symbols::SpanAnchor::StartsOn(span.start_line)
        },
    );
    #[cfg(not(feature = "symbol-resolution"))]
    let spans = {
        let _ = test;
        None
    };
    structural_projection(selected, projection, spans)
}

/// What sits above an item as its documentation (spec 163 §3.7).
#[derive(Debug, PartialEq, Eq)]
enum Documentation {
    /// A comment run, as 1-based inclusive lines.
    Span(usize, usize),
    /// A contiguous `#[doc = …]` attribute, which is not a comment span.
    Attribute,
    None,
}

/// The 0.28.0 scan first, and its span unchanged when it finds one; only when
/// it finds nothing do the one-line `/** … */` block, a multi-line block whose
/// closing line carries text, and the `#[doc = …]` report apply (spec 163
/// D-13). A blank line still ends the run (D-6).
fn documentation(lines: &[&str], item_start_line: usize) -> Documentation {
    if let Some((start, end)) = attached_documentation_span(lines, item_start_line) {
        return Documentation::Span(start, end);
    }
    let end = skip_attributes(lines, item_start_line);
    if end > 0 {
        let closing = lines[end - 1].trim();
        // One line: `/** text */`.
        if closing.starts_with("/**") && closing.ends_with("*/") && closing.len() > 4 {
            return Documentation::Span(end, end);
        }
        // Several lines, the closing one carrying text before its `*/`.
        if closing.ends_with("*/") && !closing.starts_with("*/") && !closing.starts_with("/*") {
            let mut cursor = end - 1;
            while cursor > 0 {
                let line = lines[cursor - 1].trim();
                if line.is_empty() || line.contains("*/") {
                    break;
                }
                if line.starts_with("/**") {
                    return Documentation::Span(cursor, end);
                }
                if line.starts_with("/*") {
                    break;
                }
                cursor -= 1;
            }
        }
    }
    let attributes = &lines[end..item_start_line.saturating_sub(1)];
    if attributes
        .iter()
        .any(|line| line.trim_start().starts_with("#[doc"))
    {
        return Documentation::Attribute;
    }
    Documentation::None
}

/// The 1-based line just above the attribute run that precedes an item (0 when
/// the run reaches the top of the file).
fn skip_attributes(lines: &[&str], item_start_line: usize) -> usize {
    let mut cursor = item_start_line.saturating_sub(1);
    let mut attribute_depth = 0isize;
    while cursor > 0 {
        let line = lines[cursor - 1].trim();
        attribute_depth += line.matches(']').count() as isize;
        attribute_depth -= line.matches('[').count() as isize;
        if attribute_depth > 0 || line.starts_with("#[") {
            cursor -= 1;
        } else {
            break;
        }
    }
    cursor
}

fn attached_documentation_span(lines: &[&str], item_start_line: usize) -> Option<(usize, usize)> {
    let mut cursor = skip_attributes(lines, item_start_line);
    let end = cursor;
    let mut in_doc_block = false;
    while cursor > 0 {
        let line = lines[cursor - 1].trim_start();
        if line.starts_with("///") {
            cursor -= 1;
        } else if line.starts_with("*/") || (in_doc_block && line.starts_with('*')) {
            in_doc_block = true;
            cursor -= 1;
        } else if in_doc_block && line.starts_with("/**") {
            cursor -= 1;
            in_doc_block = false;
        } else {
            break;
        }
    }
    (cursor < end).then_some((cursor + 1, end))
}

fn make_item(
    candidate: &Candidate,
    selected: &Selected,
    snapshot: &ContentSnapshot,
) -> ContentItem {
    content_item(
        &candidate.identity,
        candidate.projection,
        &candidate.selector,
        selected,
        snapshot,
    )
}

/// The item one resolved selector yields, with its framed digest (spec 155).
pub(crate) fn content_item(
    identity: &str,
    projection: ContentProjection,
    selector: &ContentSelector,
    selected: &Selected,
    snapshot: &ContentSnapshot,
) -> ContentItem {
    ContentItem {
        identity: identity.to_string(),
        repository: snapshot.repository.clone(),
        revision: snapshot.revision.clone(),
        tree: snapshot.tree.clone(),
        dirty_state: snapshot.dirty_state,
        selector: selector.clone(),
        requested_projection: projection,
        path: selected.path.clone(),
        span: selected.span,
        digest: item_digest(identity, projection, selected),
        content: selected.content.clone(),
        resolution: ContentResolution::Resolved,
        completeness: ContentCompleteness::Complete,
        schema_version: READ_SCHEMA_VERSION.to_string(),
    }
}

/// A content item's digest: its identity, projection, path, span and content,
/// framed. The snapshot is not an input, so a reader that has no snapshot
/// (a reference page, spec 166) gets the digest [`content_item`] would give.
pub(crate) fn item_digest(
    identity: &str,
    projection: ContentProjection,
    selected: &Selected,
) -> String {
    let mut set = PieceSet::new();
    set.insert(
        "identity".into(),
        PieceKind::Text,
        identity.as_bytes().to_vec(),
    );
    set.insert(
        "projection".into(),
        PieceKind::Text,
        serde_json::to_string(&projection)
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
    format!("sha256:{}", set.digest().expect("nonempty framed item"))
}

fn request_digest(request: &ContentRequest) -> Result<String, Error> {
    let mut request = request.clone();
    request.continuation = None;
    digest_json("request", &request)
}
pub(crate) fn snapshot_digest(snapshot: &ContentSnapshot) -> Result<String, Error> {
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
    use super::*;

    #[test]
    fn an_item_digest_does_not_depend_on_the_snapshot() {
        let selected = Selected {
            path: RepoPath::parse("src/lib.rs").unwrap(),
            span: ContentSpan {
                start_line: 1,
                end_line: 1,
            },
            content: "pub fn f() {}".into(),
        };
        let selector = ContentSelector::Symbol {
            id: "crate::f".into(),
            projection: Some(ContentProjection::Signature),
            required: true,
        };
        let snapshot = |revision: &str, dirty_state| ContentSnapshot {
            repository: format!("repo-{revision}"),
            revision: revision.repeat(40),
            tree: revision.repeat(40),
            dirty_state,
            binding: spec_spine_types::ContentSnapshotBinding::CallerSupplied,
        };
        let digest = item_digest("crate::f", ContentProjection::Signature, &selected);
        for s in [
            snapshot("0", spec_spine_types::ContentDirtyState::CleanWorkingTree),
            snapshot("a", spec_spine_types::ContentDirtyState::CleanExport),
        ] {
            let item = content_item(
                "crate::f",
                ContentProjection::Signature,
                &selector,
                &selected,
                &s,
            );
            assert_eq!(item.digest, digest);
        }
    }

    #[test]
    fn last_obligation_stops_before_the_next_frontmatter_key() {
        let source = "---\nobligations:\n  - id: R-1\n    kind: requirement\n    text: first\n  - id: V-1\n    kind: verification\n    text: last\nintent:\n  goal: no bleed\n---\n# Body\n";
        let selected = Selected {
            path: RepoPath::parse("specs/001/spec.md").unwrap(),
            span: ContentSpan {
                start_line: 1,
                end_line: 13,
            },
            content: source.into(),
        };
        let entry = obligation_entry(selected, "V-1").unwrap();
        assert_eq!(
            entry.content,
            "  - id: V-1\n    kind: verification\n    text: last"
        );
        assert_eq!(
            entry.span,
            ContentSpan {
                start_line: 6,
                end_line: 8
            }
        );
    }

    #[test]
    fn a_blank_separator_line_is_not_part_of_the_obligation() {
        let source = "---\nobligations:\n  - id: R-1\n    kind: requirement\n\n    text: first\n\n  - id: V-1\n    text: last\n\nintent:\n  goal: x\n---\n";
        let selected = Selected {
            path: RepoPath::parse("specs/001/spec.md").unwrap(),
            span: ContentSpan {
                start_line: 1,
                end_line: 13,
            },
            content: source.into(),
        };
        let first = obligation_entry(selected.clone(), "R-1").unwrap();
        assert_eq!(
            first.content,
            "  - id: R-1\n    kind: requirement\n\n    text: first"
        );
        assert_eq!(
            first.span,
            ContentSpan {
                start_line: 3,
                end_line: 6
            }
        );
        let last = obligation_entry(selected, "V-1").unwrap();
        assert_eq!(last.content, "  - id: V-1\n    text: last");
    }

    #[test]
    fn documentation_skips_attributes_and_rejects_non_doc_comments() {
        let lines = [
            "/// docs",
            "#[derive(Clone)]",
            "pub struct A;",
            "",
            "/* license */",
            "pub struct B;",
        ];
        assert_eq!(attached_documentation_span(&lines, 3), Some((1, 1)));
        assert_eq!(attached_documentation_span(&lines, 6), None);
    }

    /// Spec 163 §3.7 and D-13: the forms 0.28.0 missed, its own spans kept.
    #[test]
    fn documentation_recognizes_every_outer_doc_comment_form() {
        let lines = [
            "/** single */",           // 1
            "pub fn a() {}",           // 2
            "/**",                     // 3
            " * several",              // 4
            " */",                     // 5
            "pub fn b() {}",           // 6
            "/** opens",               // 7
            "    closes with text */", // 8
            "#[inline]",               // 9
            "pub fn c() {}",           // 10
            "#[doc = \"attr\"]",       // 11
            "pub fn d() {}",           // 12
            "/// separated",           // 13
            "",                        // 14
            "pub fn e() {}",           // 15
            "/* not a doc */",         // 16
            "pub fn f() {}",           // 17
        ];
        assert_eq!(documentation(&lines, 2), Documentation::Span(1, 1));
        // A 0.28.0 span is returned unchanged.
        assert_eq!(attached_documentation_span(&lines, 6), Some((3, 5)));
        assert_eq!(documentation(&lines, 6), Documentation::Span(3, 5));
        assert_eq!(documentation(&lines, 10), Documentation::Span(7, 8));
        assert_eq!(documentation(&lines, 12), Documentation::Attribute);
        // A blank line still ends the run (D-6).
        assert_eq!(documentation(&lines, 15), Documentation::None);
        assert_eq!(documentation(&lines, 17), Documentation::None);
    }
}
