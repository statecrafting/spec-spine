//! Deterministic reference documentation (spec 166).
//!
//! One Markdown page per exported spec, projected from a typed page model
//! whose every field comes from a named ledger source: the registry record,
//! the committed index, spec 155 signatures, a spec 169 traceability read and
//! a committed spec 162 catalog. Pages carry fixed generated regions and one
//! authored `notes` region kept byte for byte; the check regenerates in
//! memory and compares, never writing.
//!
//! [`reference_export`] and [`reference_check_with`] are pure over a
//! [`ReferenceInputs`]; [`gather_reference_inputs`] is the IO wrapper that
//! reads only the paths 3.8 names. Files are returned as data: nothing here
//! writes, executes an example, or reads Git.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};
use spec_spine_types::{
    CapabilityCatalog, CodebaseIndex, Config, ContentProjection, ContentSelector, Error,
    OwningEdge, REFERENCE_LIMITATION, REFERENCE_MAX_NOTES_BYTES, REFERENCE_MAX_PAGE_BYTES,
    REFERENCE_MAX_PAGES, REFERENCE_MAX_SIGNATURES, REFERENCE_PAGE_SCHEMA_VERSION,
    REFERENCE_REGIONS, ReferenceChanges, ReferenceCheck, ReferenceEvidence,
    ReferenceEvidenceInputs, ReferenceFinding, ReferenceFindingKind, ReferenceHeader,
    ReferenceInput, ReferenceLifecycle, ReferenceModel, ReferenceOperation, ReferenceOwnership,
    ReferencePage, ReferenceRelation, ReferenceRequest, ReferenceSignature, ReferenceSpecs,
    ReferenceUnit, Registry, SelectAll, SelectKeyword, SignatureLanguage, SpecRecord,
    TraceabilityReport, TraceabilityRequest, Unit, UnitResolution, parse_semver,
};

use crate::content::{Resolution, item_digest, resolve_selectors_after_freshness};
use crate::{Versioning, canonical_json, hash, read_document};

/// Everything one export reads, already read (spec 166 3.8).
#[derive(Clone, Debug)]
pub struct ReferenceInputs {
    pub registry: Registry,
    pub index: CodebaseIndex,
    /// Each spec's registry `shardHash` (its full `spec.md` content hash).
    pub registry_shard_hashes: BTreeMap<String, String>,
    /// Each spec's committed index shard `shardHash`.
    pub index_shard_hashes: BTreeMap<String, String>,
    /// Each exported spec's `spec.md` text, for its `verify:cli` block.
    pub spec_sources: BTreeMap<String, String>,
    /// Each owned symbol's signature row, by symbol id.
    pub signatures: BTreeMap<String, ReferenceSignature>,
    pub traceability: Option<TraceabilityReport>,
    /// The committed catalog and its document digest.
    pub catalog: Option<(CapabilityCatalog, String)>,
    /// The catalog's request path, as its input identity.
    pub catalog_path: Option<String>,
    /// Every page-pattern file directly under `outputDir`, by file name.
    pub existing_pages: BTreeMap<String, Vec<u8>>,
    /// Whether the committed registry and index are fresh.
    pub ledger_fresh: bool,
}

/// The files one generation writes and removes, as data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceExport {
    pub models: Vec<ReferencePage>,
    /// `(path, bytes)` for every page, in spec order.
    pub pages: Vec<(String, String)>,
    /// Orphaned pages with an empty `notes` region, to remove.
    pub remove: Vec<String>,
}

/// A parsed region: its kind, id and the lines strictly between its
/// delimiters.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Region {
    authored: bool,
    id: String,
    body: Vec<String>,
}

// ===== request =====

/// Validate a request on its own terms (spec 166 3.2): schema major, a
/// normalized repository-relative `outputDir` outside the derived and state
/// directories, full ids only, and limits that only lower their defaults.
pub fn validate_reference_request(cfg: &Config, request: &ReferenceRequest) -> Result<(), Error> {
    let ours = parse_semver(REFERENCE_PAGE_SCHEMA_VERSION).expect("own version parses");
    match parse_semver(&request.schema_version) {
        Some(v) if v.0 == ours.0 && v.1 <= ours.1 => {}
        _ => {
            return Err(Error::Usage(format!(
                "unsupported-schema: reference request schema '{}' is not served by {}",
                request.schema_version, REFERENCE_PAGE_SCHEMA_VERSION
            )));
        }
    }
    let dir = request.output_dir.as_str();
    // The repository path contract first (spec 144), then the stricter
    // normalized form this request needs on top of it.
    spec_spine_types::RepoPath::parse(dir)
        .map_err(|why| Error::Usage(format!("outputDir {why}")))?;
    let normalized = !dir.is_empty()
        && !dir.starts_with('/')
        && !dir.ends_with('/')
        && !dir.contains('\\')
        && dir
            .split('/')
            .all(|c| !c.is_empty() && c != "." && c != "..");
    if !normalized {
        return Err(Error::Usage(format!(
            "outputDir '{dir}' is not a normalized repository-relative path"
        )));
    }
    for (name, reserved) in [
        ("derived_dir", &cfg.layout.derived_dir),
        ("state_dir", &cfg.layout.state_dir),
    ] {
        let reserved = reserved.trim_end_matches('/');
        if !reserved.is_empty()
            && (dir == reserved
                || dir.starts_with(&format!("{reserved}/"))
                || reserved.starts_with(&format!("{dir}/")))
        {
            return Err(Error::Refused(format!(
                "containment: outputDir '{dir}' overlaps the configured {name} '{reserved}'"
            )));
        }
    }
    if let ReferenceSpecs::Ids(ids) = &request.specs {
        for id in &ids.ids {
            if !full_id(id) {
                return Err(Error::Usage(format!(
                    "spec '{id}' is not a full id; short ids are refused"
                )));
            }
        }
    }
    let l = request.limits;
    if !(1..=REFERENCE_MAX_PAGES).contains(&l.max_pages)
        || !(1..=REFERENCE_MAX_PAGE_BYTES).contains(&l.max_page_bytes)
        || l.max_signatures > REFERENCE_MAX_SIGNATURES
    {
        return Err(Error::Usage(
            "limits may only lower their defaults (maxPages 1024, maxPageBytes 524288, \
             maxSignatures 256)"
                .into(),
        ));
    }
    Ok(())
}

fn full_id(id: &str) -> bool {
    let mut parts = id.splitn(2, '-');
    let ordinal = parts.next().unwrap_or("");
    let slug = parts.next().unwrap_or("");
    ordinal.len() == 3
        && ordinal.bytes().all(|b| b.is_ascii_digit())
        && !slug.is_empty()
        && slug.split('-').all(|w| {
            !w.is_empty()
                && w.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

/// A file name in the page pattern: `<full-spec-id>.md`.
fn page_spec_id(file_name: &str) -> Option<&str> {
    file_name.strip_suffix(".md").filter(|id| full_id(id))
}

/// The specs an export covers, in registry order. An unknown id is
/// [`Error::NotFound`].
fn exported<'a>(
    registry: &'a Registry,
    specs: &ReferenceSpecs,
) -> Result<Vec<&'a SpecRecord>, Error> {
    match specs {
        // `all` is the one selector; naming it means a new keyword fails to
        // compile here rather than exporting everything.
        ReferenceSpecs::Select(SelectAll {
            select: SelectKeyword::All,
        }) => Ok(registry.specs.iter().collect()),
        ReferenceSpecs::Ids(ids) => {
            let wanted: BTreeSet<&str> = ids.ids.iter().map(String::as_str).collect();
            let missing: Vec<&str> = wanted
                .iter()
                .filter(|id| !registry.specs.iter().any(|r| r.id == **id))
                .copied()
                .collect();
            if !missing.is_empty() {
                return Err(Error::NotFound(format!(
                    "no spec {} in the registry",
                    missing.join(", ")
                )));
            }
            Ok(registry
                .specs
                .iter()
                .filter(|r| wanted.contains(r.id.as_str()))
                .collect())
        }
    }
}

/// `maxPages` (3.12), checked as soon as the exported set is known.
fn check_page_count(request: &ReferenceRequest, pages: usize) -> Result<(), Error> {
    if pages > request.limits.max_pages {
        return Err(Error::Refused(format!(
            "limit-exceeded: {pages} pages, above maxPages {}",
            request.limits.max_pages
        )));
    }
    Ok(())
}

// ===== IO =====

/// Read every input an export needs, and nothing else (spec 166 3.8): the
/// committed registry and index, each exported `spec.md`, the source of each
/// owned symbol, the named catalog, and page-pattern files directly under
/// `outputDir`. Reads no environment, no Git object, and no ignored file.
pub fn gather_reference_inputs(
    cfg: &Config,
    repo_root: &Path,
    request: &ReferenceRequest,
) -> Result<ReferenceInputs, Error> {
    gather_if_fresh(cfg, repo_root, request)?.ok_or_else(|| Error::Stale {
        expected: "a fresh committed registry and index".into(),
        actual: "a stale ledger".into(),
    })
}

/// [`gather_reference_inputs`], with a stale ledger as `None` rather than an
/// error. The request is validated and the ledger's freshness read exactly
/// once, so a caller that reports staleness as a finding cannot see a
/// different answer from a second read.
fn gather_if_fresh(
    cfg: &Config,
    repo_root: &Path,
    request: &ReferenceRequest,
) -> Result<Option<ReferenceInputs>, Error> {
    validate_reference_request(cfg, request)?;
    crate::pathutil::refuse_links_leaving(cfg, repo_root)?;
    if !ledger_is_fresh(cfg, repo_root)? {
        return Ok(None);
    }
    let registry = crate::compile::load_committed_registry(cfg, repo_root)?;
    let index = crate::index::load_committed_index(cfg, repo_root)?;
    let specs = exported(&registry, &request.specs)?;
    // Before any spec source, signature or page is read (3.12).
    check_page_count(request, specs.len())?;

    let registry_shard_hashes = crate::closure::committed_content_hashes(cfg, repo_root)?;
    let mut index_shard_hashes = BTreeMap::new();
    let by_spec = crate::index::index_dir(cfg, repo_root).join(crate::shard::BY_SPEC_DIR);
    let mut spec_sources = BTreeMap::new();
    for r in &specs {
        let shard = by_spec.join(format!("{}.json", r.id));
        if let Ok(bytes) = fs::read(&shard) {
            let v: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| Error::Schema(format!("index shard {}: {e}", r.id)))?;
            if let Some(h) = v.get("shardHash").and_then(|h| h.as_str()) {
                index_shard_hashes.insert(r.id.clone(), h.to_string());
            }
        }
        let text = fs::read_to_string(repo_root.join(&r.spec_path))
            .map_err(|e| Error::Io(format!("read {}: {e}", r.spec_path)))?;
        spec_sources.insert(r.id.clone(), hash::normalize(&text));
    }

    let symbols: BTreeSet<String> = specs
        .iter()
        .flat_map(|r| owned_units(r))
        .filter_map(|(_, _, u)| match u {
            Unit::Symbol { id, .. } => Some(id),
            _ => None,
        })
        .collect();
    let signatures = resolve_signatures(cfg, repo_root, &symbols)?;

    let traceability = if request.traceability {
        Some(crate::traceability::traceability(
            cfg,
            repo_root,
            &registry,
            Some(&index),
            &TraceabilityRequest::default(),
        )?)
    } else {
        None
    };
    let catalog = match &request.catalog {
        None => None,
        Some(path) => {
            let rel = spec_spine_types::RepoPath::parse(path).map_err(Error::Usage)?;
            let text = fs::read_to_string(repo_root.join(rel.as_str()))
                .map_err(|e| Error::Io(format!("read catalog {path}: {e}")))?;
            let catalog: CapabilityCatalog = serde_json::from_str(&text).map_err(|e| {
                Error::Usage(format!("catalog {path} is not a spec 162 catalog: {e}"))
            })?;
            Some((catalog, sha256(hash::normalize(&text).as_bytes())))
        }
    };

    let out = repo_root.join(&request.output_dir);
    reject_symlinked_output(repo_root, &request.output_dir)?;
    let mut existing_pages = BTreeMap::new();
    if out.is_dir() {
        for entry in fs::read_dir(&out).map_err(|e| Error::Io(e.to_string()))? {
            let entry = entry.map_err(|e| Error::Io(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let kind = entry.file_type().map_err(|e| Error::Io(e.to_string()))?;
            if page_spec_id(&name).is_some() && kind.is_symlink() {
                // `generate` would write through it, onto whatever it names.
                return Err(Error::Refused(format!(
                    "containment: page '{}/{name}' is a symbolic link",
                    request.output_dir
                )));
            }
            if page_spec_id(&name).is_some() && kind.is_file() {
                let bytes = read_committed_page(
                    &entry.path(),
                    &format!("{}/{name}", request.output_dir),
                    request.limits.max_page_bytes,
                )?;
                existing_pages.insert(name, bytes);
            }
        }
    }

    Ok(Some(ReferenceInputs {
        registry,
        index,
        registry_shard_hashes,
        index_shard_hashes,
        spec_sources,
        signatures,
        traceability,
        catalog,
        catalog_path: request.catalog.clone(),
        existing_pages,
        ledger_fresh: true,
    }))
}

/// Whether the committed registry and index match the corpus, as `check`
/// compares them (spec 166 3.9). An unresolved claim is reported by the
/// page's ownership rows, not as a stale ledger.
pub fn ledger_is_fresh(cfg: &Config, repo_root: &Path) -> Result<bool, Error> {
    let registry_fresh = matches!(
        crate::compile::check_registry_freshness(cfg, repo_root)?,
        crate::index::Freshness::Fresh
    );
    // A claim that does not resolve is not staleness (spec 079): only a
    // committed shard whose bytes moved is.
    let index = crate::index::index_freshness_report(cfg, repo_root)?;
    Ok(registry_fresh && index.stale.is_empty() && index.blocked_drift.is_empty())
}

/// 3.2: `outputDir` must not cross a symbolic link.
fn reject_symlinked_output(repo_root: &Path, dir: &str) -> Result<(), Error> {
    let mut at = repo_root.to_path_buf();
    for c in dir.split('/') {
        at.push(c);
        if fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(Error::Refused(format!(
                "containment: outputDir '{dir}' crosses a symbolic link"
            )));
        }
    }
    Ok(())
}

/// Each owned symbol's spec 155 `signature` projection, or its omission
/// reason. Never falls back to another projection (3.4).
fn resolve_signatures(
    cfg: &Config,
    repo_root: &Path,
    symbols: &BTreeSet<String>,
) -> Result<BTreeMap<String, ReferenceSignature>, Error> {
    if symbols.is_empty() {
        return Ok(BTreeMap::new());
    }
    let selectors: Vec<ContentSelector> = symbols
        .iter()
        .map(|id| ContentSelector::Symbol {
            id: id.clone(),
            projection: Some(ContentProjection::Signature),
            required: true,
        })
        .collect();
    let resolutions = resolve_selectors_after_freshness(
        cfg,
        repo_root,
        &selectors,
        ContentProjection::Signature,
        REFERENCE_MAX_PAGE_BYTES,
    )?;
    let mut out = BTreeMap::new();
    for (id, r) in pair_resolutions(symbols, resolutions)? {
        let row = match r.outcome {
            Ok(selected) => {
                let digest = item_digest(&r.identity, r.projection, &selected);
                let path = selected.path.as_str().to_string();
                let language = if path.ends_with(".rs") {
                    Some(SignatureLanguage::Rust)
                } else if path.ends_with(".ts") || path.ends_with(".tsx") {
                    Some(SignatureLanguage::Typescript)
                } else {
                    None
                };
                ReferenceSignature {
                    symbol: id.clone(),
                    path: Some(path),
                    start_line: Some(selected.span.start_line),
                    end_line: Some(selected.span.end_line),
                    digest: Some(digest),
                    text: Some(selected.content),
                    language,
                    omission: None,
                }
            }
            Err((reason, _)) => ReferenceSignature {
                symbol: id.clone(),
                path: None,
                start_line: None,
                end_line: None,
                digest: None,
                text: None,
                language: None,
                omission: Some(
                    serde_json::to_value(reason)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_string))
                        .unwrap_or_default(),
                ),
            },
        };
        out.insert(id.clone(), row);
    }
    Ok(out)
}

// ===== the page model =====

/// Every owning unit of `r`: `(edge, target spec, unit)`.
fn owned_units(r: &SpecRecord) -> Vec<(OwningEdge, Option<String>, Unit)> {
    let mut out = Vec::new();
    for u in &r.establishes {
        out.push((OwningEdge::Establishes, None, u.clone()));
    }
    for e in &r.extends {
        if let Some(u) = &e.unit {
            out.push((OwningEdge::Extends, Some(e.spec.clone()), u.clone()));
        }
    }
    for e in &r.refines {
        if let Some(u) = &e.unit {
            out.push((OwningEdge::Refines, None, u.clone()));
        }
    }
    for e in &r.co_authority {
        out.push((OwningEdge::CoAuthority, None, e.unit.clone()));
    }
    for e in &r.constrains {
        if let Some(u) = &e.unit {
            out.push((OwningEdge::Constrains, None, u.clone()));
        }
    }
    out
}

/// A unit's canonical JSON on one line: sorted keys, no whitespace.
fn unit_json(u: &Unit) -> String {
    canonical_json::to_string(&u.subject())
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .map(|v| v.to_string())
        .unwrap_or_default()
}

fn planned(u: &Unit) -> bool {
    serde_json::to_value(u)
        .ok()
        .and_then(|v| v.get("planned").and_then(|p| p.as_bool()))
        .unwrap_or(false)
}

fn wire<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// Build one page model (spec 166 3.3, 3.4).
fn page_model(
    request: &ReferenceRequest,
    inputs: &ReferenceInputs,
    r: &SpecRecord,
    notes: String,
) -> ReferencePage {
    let mapping = inputs
        .index
        .traceability
        .mappings
        .iter()
        .find(|m| m.spec_id == r.id);
    let mut units: Vec<ReferenceUnit> = owned_units(r)
        .into_iter()
        .map(|(edge, target, u)| {
            let resolved = mapping.and_then(|m| {
                m.resolved_units
                    .iter()
                    .find(|ru| ru.ownership && ru.unit.subject() == u.subject())
            });
            ReferenceUnit {
                edge,
                target_spec: target,
                unit: unit_json(&u),
                planned: planned(&u),
                resolution: match resolved {
                    Some(ru) if !ru.locations.is_empty() => UnitResolution::Resolved,
                    Some(_) => UnitResolution::Unresolved,
                    None => UnitResolution::Unknown,
                },
            }
        })
        .collect();
    units.sort_by(|a, b| {
        a.edge
            .cmp(&b.edge)
            .then_with(|| a.unit.as_bytes().cmp(b.unit.as_bytes()))
            .then_with(|| a.target_spec.cmp(&b.target_spec))
    });
    units.dedup();
    let mut references: Vec<String> = r
        .references
        .iter()
        .filter_map(|x| x.unit.as_ref())
        .map(unit_json)
        .collect();
    references.sort();
    references.dedup();

    let mut requirements = r.obligations.clone();
    requirements.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));

    let symbols: BTreeSet<String> = owned_units(r)
        .into_iter()
        .filter_map(|(_, _, u)| match u {
            Unit::Symbol { id, .. } => Some(id),
            _ => None,
        })
        .collect();
    let signatures: Vec<ReferenceSignature> = symbols
        .iter()
        .filter_map(|s| inputs.signatures.get(s).cloned())
        .collect();

    let relations = inputs.traceability.as_ref().map(|t| {
        let prefix = format!("{}#", r.id);
        let mut rows: Vec<ReferenceRelation> = t
            .relations
            .iter()
            .filter(|rel| rel.obligation.starts_with(&prefix))
            .map(|rel| ReferenceRelation {
                id: rel.identity.clone(),
                obligation: rel.obligation.clone(),
                relation: rel.relation.as_str().to_string(),
                target_identity: rel.target_identity.clone(),
                state: rel.state,
            })
            .collect();
        rows.sort_by(|a, b| {
            (a.relation != "tested-by", &a.relation, &a.id).cmp(&(
                b.relation != "tested-by",
                &b.relation,
                &b.id,
            ))
        });
        rows
    });

    let operations = inputs.catalog.as_ref().map(|(c, _)| {
        let mut rows: Vec<ReferenceOperation> = c
            .operations
            .iter()
            .filter(|o| o.governed_by.iter().any(|g| g == &r.id))
            .map(|o| ReferenceOperation {
                name: o.name.clone(),
                reads: o.effects.reads.clone(),
                writes: o.effects.writes.clone(),
                executes: o.effects.executes.clone(),
                network: o.effects.network.clone(),
                examples: o.examples.clone(),
            })
            .collect();
        rows.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
        rows
    });

    let mut evidence_inputs: Vec<ReferenceEvidenceInputs> = r
        .obligations
        .iter()
        .filter(|o| o.kind == spec_spine_types::ObligationKind::Verification)
        .map(|o| ReferenceEvidenceInputs {
            obligation: format!("{}#{}", r.id, o.id),
            inputs: o.inputs.clone(),
        })
        .collect();
    evidence_inputs.sort_by(|a, b| a.obligation.cmp(&b.obligation));
    let commands = inputs
        .spec_sources
        .get(&r.id)
        .map(|src| crate::verify::plan_from_markdown(&r.id, src).commands)
        .unwrap_or_default();

    let sorted = |v: &[String]| {
        let mut v = v.to_vec();
        v.sort();
        v.dedup();
        v
    };
    let incoming = |pred: &dyn Fn(&SpecRecord) -> bool| {
        let mut v: Vec<String> = inputs
            .registry
            .specs
            .iter()
            .filter(|o| o.id != r.id && pred(o))
            .map(|o| o.id.clone())
            .collect();
        v.sort();
        v
    };
    let changes = ReferenceChanges {
        amends: sorted(&r.amends),
        amends_sections: sorted(&r.amends_sections),
        amends_verification: sorted(&r.amends_verification),
        supersedes: sorted(
            &r.supersedes
                .iter()
                .map(|s| s.spec().to_string())
                .collect::<Vec<_>>(),
        ),
        amendment_record: r.amendment_record.clone(),
        amended_by: incoming(&|o| o.amends.iter().any(|a| a == &r.id)),
        superseded_by: incoming(&|o| o.supersedes.iter().any(|s| s.spec() == r.id)),
        extended_by: incoming(&|o| o.extends.iter().any(|e| e.spec == r.id)),
    };

    let mut page_inputs = Vec::new();
    if let Some(h) = inputs.registry_shard_hashes.get(&r.id) {
        page_inputs.push(ReferenceInput {
            identity: format!("registry:{}", r.id),
            digest: format!("sha256:{h}"),
        });
    }
    if let Some(h) = inputs.index_shard_hashes.get(&r.id) {
        page_inputs.push(ReferenceInput {
            identity: format!("index:{}", r.id),
            digest: format!("sha256:{h}"),
        });
    }
    for s in &signatures {
        if let Some(d) = &s.digest {
            page_inputs.push(ReferenceInput {
                identity: format!("signature:{}", s.symbol),
                digest: d.clone(),
            });
        }
    }
    if let Some(rows) = &relations {
        page_inputs.push(ReferenceInput {
            identity: format!("traceability:{}", r.id),
            digest: sha256(
                canonical_json::to_string(rows)
                    .unwrap_or_default()
                    .as_bytes(),
            ),
        });
    }
    if let (Some((_, digest)), Some(path)) = (&inputs.catalog, &inputs.catalog_path) {
        page_inputs.push(ReferenceInput {
            identity: format!("catalog:{path}"),
            digest: digest.clone(),
        });
    }
    page_inputs.sort_by(|a, b| a.identity.as_bytes().cmp(b.identity.as_bytes()));

    ReferencePage {
        schema_version: REFERENCE_PAGE_SCHEMA_VERSION.to_string(),
        spec_id: r.id.clone(),
        path: format!("{}/{}.md", request.output_dir, r.id),
        header: ReferenceHeader {
            id: r.id.clone(),
            title: r.title.clone(),
            spec_path: r.spec_path.clone(),
            summary: r.summary.clone(),
        },
        lifecycle: ReferenceLifecycle {
            status: r.status,
            implementation: r.implementation,
            superseded_by: r.superseded_by.clone(),
            retirement_rationale: r.retirement_rationale.clone(),
            created: r.created.clone(),
        },
        requirements,
        ownership: ReferenceOwnership { units, references },
        signatures,
        relations,
        operations,
        evidence: ReferenceEvidence {
            inputs: evidence_inputs,
            commands,
        },
        non_goals: r
            .intent
            .as_ref()
            .map(|i| i.non_goals.clone())
            .unwrap_or_default(),
        changes,
        inputs: page_inputs,
        notes,
    }
}

// ===== Markdown projection (3.5, 3.6) =====

/// One free-text value: line breaks folded to spaces, trimmed, and every
/// character that could break a table cell, open an HTML comment or form a
/// delimiter escaped.
fn esc(text: &str) -> String {
    let text = hash::normalize(text).replace('\n', " ");
    let mut out = String::with_capacity(text.len());
    for ch in text.trim().chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '|' => out.push_str("\\|"),
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

fn longest_backtick_run(text: &str) -> usize {
    let mut best = 0;
    let mut run = 0;
    for ch in text.chars() {
        if ch == '`' {
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
    }
    best
}

/// An identifier as a code span one backtick longer than its longest run.
fn code(text: &str) -> String {
    let text = hash::normalize(text).replace('\n', " ");
    let ticks = "`".repeat(longest_backtick_run(&text) + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    // A pipe would still end a table cell inside a code span.
    let text = text.replace('|', "\\|");
    format!("{ticks}{pad}{text}{pad}{ticks}")
}

/// A fenced block at least three backticks and one longer than any run in
/// its body.
fn fence(body: &str, info: &str) -> String {
    let body = hash::normalize(body);
    let ticks = "`".repeat((longest_backtick_run(&body) + 1).max(3));
    let mut out = format!("{ticks}{info}\n{body}");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&ticks);
    out.push('\n');
    out
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "None declared.".to_string()
    } else {
        items.iter().map(|i| code(i)).collect::<Vec<_>>().join(", ")
    }
}

fn opt_or_none(v: &Option<String>) -> String {
    v.as_deref().map_or("None declared.".to_string(), code)
}

fn console_line(argv: &[String]) -> String {
    let mut words = vec!["spec-spine".to_string()];
    for a in argv {
        let plain = !a.is_empty()
            && a.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_./=:@#,+".contains(&b));
        words.push(if plain {
            a.clone()
        } else {
            format!("'{}'", a.replace('\'', "'\\''"))
        });
    }
    words.join(" ")
}

/// The body of each generated region, in page order (3.4, 3.5).
fn region_bodies(p: &ReferencePage) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();

    out.push((
        "header",
        format!(
            "# {} {}\n\n- Path: {}\n- Summary: {}\n- Reference page schema: {}\n",
            code(&p.header.id),
            esc(&p.header.title),
            code(&p.header.spec_path),
            esc(&p.header.summary),
            code(&p.schema_version)
        ),
    ));

    let l = &p.lifecycle;
    out.push((
        "lifecycle",
        format!(
            "## Lifecycle\n\n| Field | Value |\n| --- | --- |\n| status | {} |\n| implementation | {} |\n| superseded_by | {} |\n| retirement_rationale | {} |\n| created | {} |\n",
            code(&wire(&l.status)),
            opt_or_none(&l.implementation.as_ref().map(wire)),
            opt_or_none(&l.superseded_by),
            l.retirement_rationale
                .as_deref()
                .map_or("None declared.".to_string(), esc),
            code(&l.created)
        ),
    ));

    let mut req = String::from("## Requirements\n\n");
    if p.requirements.is_empty() {
        req.push_str("None declared.\n");
    } else {
        req.push_str(
            "| Obligation | Kind | Text | Anchor | Withdrawn |\n| --- | --- | --- | --- | --- |\n",
        );
        for o in &p.requirements {
            req.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                code(&format!("{}#{}", p.spec_id, o.id)),
                code(&wire(&o.kind)),
                esc(&o.text),
                code(&o.anchor),
                if o.withdrawn { "yes" } else { "no" }
            ));
        }
    }
    out.push(("requirements", req));

    let mut own = String::from("## Ownership\n\n");
    if p.ownership.units.is_empty() {
        own.push_str("None declared.\n");
    } else {
        own.push_str("| Edge | Target spec | Unit | Planned | Resolution |\n| --- | --- | --- | --- | --- |\n");
        for u in &p.ownership.units {
            own.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                code(u.edge.as_str()),
                u.target_spec
                    .as_deref()
                    .map_or("None declared.".to_string(), code),
                code(&u.unit),
                if u.planned { "yes" } else { "no" },
                code(u.resolution.as_str())
            ));
        }
    }
    own.push_str("\n### References (non-owning)\n\n");
    if p.ownership.references.is_empty() {
        own.push_str("None declared.\n");
    } else {
        for r in &p.ownership.references {
            own.push_str(&format!("- {}\n", code(r)));
        }
    }
    out.push(("ownership", own));

    let mut sig = String::from("## Signatures\n");
    if p.signatures.is_empty() {
        sig.push_str("\nNone declared.\n");
    } else {
        for s in &p.signatures {
            sig.push_str(&format!("\n### {}\n\n", code(&s.symbol)));
            match (&s.text, &s.omission) {
                (Some(text), _) => {
                    sig.push_str(&format!(
                        "- Path: {}, lines {}-{}\n- Digest: {}\n\n",
                        code(s.path.as_deref().unwrap_or("")),
                        s.start_line.unwrap_or(0),
                        s.end_line.unwrap_or(0),
                        code(s.digest.as_deref().unwrap_or(""))
                    ));
                    sig.push_str(&fence(
                        text,
                        s.language.map_or("", SignatureLanguage::as_str),
                    ));
                }
                (None, omission) => sig.push_str(&format!(
                    "- Omitted: {}\n",
                    code(omission.as_deref().unwrap_or("unresolved"))
                )),
            }
        }
    }
    out.push(("signatures", sig));

    let mut rel = String::from("## Relations\n\n");
    match &p.relations {
        None => rel.push_str("Not supplied.\n"),
        Some(rows) if rows.is_empty() => rel.push_str("None declared.\n"),
        Some(rows) => {
            rel.push_str("| Relation | Identity | Obligation | Target | State |\n| --- | --- | --- | --- | --- |\n");
            for r in rows {
                rel.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    code(&r.relation),
                    code(&r.id),
                    code(&r.obligation),
                    code(&r.target_identity),
                    code(&wire(&r.state))
                ));
            }
        }
    }
    out.push(("relations", rel));

    let mut ops = String::from("## Operations\n");
    match &p.operations {
        None => ops.push_str("\nNot supplied.\n"),
        Some(rows) if rows.is_empty() => ops.push_str("\nNone declared.\n"),
        Some(rows) => {
            for o in rows {
                ops.push_str(&format!(
                    "\n### {}\n\n- Reads: {}\n- Writes: {}\n- Executes: {}\n- Network: {}\n",
                    code(&o.name),
                    list_or_none(&o.reads),
                    list_or_none(&o.writes),
                    list_or_none(&o.executes),
                    code(&o.network)
                ));
                for e in &o.examples {
                    ops.push_str(&format!(
                        "\n#### Example {}\n\n- Fixture: {}\n- Exit code: {}\n- Stdout includes: {}\n\n",
                        code(&e.id),
                        e.fixture.as_deref().map_or("None declared.".to_string(), code),
                        code(&e.exit_code.to_string()),
                        list_or_none(&e.stdout_includes)
                    ));
                    ops.push_str(&fence(&console_line(&e.argv), "console"));
                    if let Some(stdin) = &e.stdin {
                        ops.push_str("\nStdin:\n\n");
                        ops.push_str(&fence(stdin, ""));
                    }
                }
            }
        }
    }
    out.push(("operations", ops));

    let mut ev = String::from("## Evidence\n\n### Declared inputs\n\n");
    if p.evidence.inputs.is_empty() {
        ev.push_str("None declared.\n");
    } else {
        ev.push_str("| Obligation | Inputs |\n| --- | --- |\n");
        for i in &p.evidence.inputs {
            ev.push_str(&format!(
                "| {} | {} |\n",
                code(&i.obligation),
                list_or_none(&i.inputs)
            ));
        }
    }
    ev.push_str("\n### Acceptance commands (declared, not executed)\n\n");
    if p.evidence.commands.is_empty() {
        ev.push_str("No declared acceptance block.\n");
    } else {
        ev.push_str(&fence(&p.evidence.commands.join("\n"), "console"));
    }
    out.push(("evidence", ev));

    let mut lim = String::from("## Limitations\n\n");
    if p.non_goals.is_empty() {
        lim.push_str("None declared.\n");
    } else {
        for g in &p.non_goals {
            lim.push_str(&format!("- {}\n", esc(g)));
        }
    }
    lim.push_str(&format!("\n{REFERENCE_LIMITATION}\n"));
    out.push(("limitations", lim));

    let c = &p.changes;
    out.push((
        "changes",
        format!(
            "## Changes\n\n| Edge | Specs |\n| --- | --- |\n| amends | {} |\n| amends_sections | {} |\n| amends_verification | {} |\n| supersedes | {} |\n| amendment_record | {} |\n| amended by | {} |\n| superseded by | {} |\n| extended by | {} |\n",
            list_or_none(&c.amends),
            list_or_none(&c.amends_sections),
            list_or_none(&c.amends_verification),
            list_or_none(&c.supersedes),
            c.amendment_record.as_deref().map_or("None declared.".to_string(), esc),
            list_or_none(&c.amended_by),
            list_or_none(&c.superseded_by),
            list_or_none(&c.extended_by)
        ),
    ));

    let mut inp = String::from("## Inputs\n\n");
    if p.inputs.is_empty() {
        inp.push_str("None declared.\n");
    } else {
        inp.push_str("| Identity | Digest |\n| --- | --- |\n");
        for i in &p.inputs {
            inp.push_str(&format!(
                "| {} | {} |\n",
                code(&i.identity),
                code(&i.digest)
            ));
        }
    }
    out.push(("inputs", inp));
    out
}

/// No generated line ends in whitespace (3.5): an empty field or a source
/// line carried into a fence would otherwise keep its trailing blanks.
fn strip_trailing_whitespace(body: &str) -> String {
    body.split_inclusive('\n')
        .map(|l| match l.strip_suffix('\n') {
            Some(t) => format!("{}\n", t.trim_end_matches([' ', '\t'])),
            None => l.trim_end_matches([' ', '\t']).to_string(),
        })
        .collect()
}

fn delimiter(begin: bool, authored: bool, id: &str) -> String {
    format!(
        "<!-- spec-spine:{} {} id=\"{id}\" -->",
        if begin { "begin" } else { "end" },
        if authored { "authored" } else { "generated" }
    )
}

/// The canonical Markdown bytes of one page (3.5, 3.6).
pub fn render_reference_page(p: &ReferencePage) -> String {
    let mut out = String::new();
    for (id, body) in region_bodies(p) {
        out.push_str(&delimiter(true, false, id));
        out.push('\n');
        out.push_str(&strip_trailing_whitespace(&body));
        out.push_str(&delimiter(false, false, id));
        out.push_str("\n\n");
    }
    out.push_str(&delimiter(true, true, "notes"));
    out.push('\n');
    out.push_str(&p.notes);
    out.push_str(&delimiter(false, true, "notes"));
    out.push('\n');
    out
}

/// Parse a page into its regions (3.6): delimiters only as whole lines
/// outside backtick fences, every byte in exactly one region, and exactly
/// one authored `notes` region, last. `Err` names why it is malformed.
fn parse_page(bytes: &[u8]) -> Result<Vec<Region>, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "the page is not valid UTF-8".to_string())?;
    let text = hash::normalize(text);
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut regions: Vec<Region> = Vec::new();
    let mut open: Option<Region> = None;
    let mut fence: Option<usize> = None;
    let begin = |l: &str| -> Option<(bool, String)> {
        let rest = l
            .strip_prefix("<!-- spec-spine:begin ")?
            .strip_suffix("\" -->")?;
        let (kind, id) = rest.split_once(" id=\"")?;
        let authored = match kind {
            "generated" => false,
            "authored" => true,
            _ => return None,
        };
        Some((authored, id.to_string()))
    };
    for (n, raw) in lines.iter().enumerate() {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        match &mut open {
            None => {
                if line.is_empty() {
                    continue;
                }
                match begin(line) {
                    Some((authored, id)) => {
                        open = Some(Region {
                            authored,
                            id,
                            body: Vec::new(),
                        });
                        fence = None;
                    }
                    None => return Err(format!("line {} lies outside every region", n + 1)),
                }
            }
            Some(region) => {
                let ticks = line.trim_start_matches(' ');
                let run = ticks.bytes().take_while(|b| *b == b'`').count();
                let indent = line.len() - ticks.len();
                match fence {
                    Some(len) => {
                        if indent <= 3 && run >= len && ticks[run..].trim().is_empty() {
                            fence = None;
                        }
                        region.body.push(line.to_string());
                        continue;
                    }
                    None if indent <= 3 && run >= 3 && !region.authored => {
                        fence = Some(run);
                        region.body.push(line.to_string());
                        continue;
                    }
                    None => {}
                }
                if line == delimiter(false, region.authored, &region.id) {
                    regions.push(open.take().expect("open region"));
                } else if line.starts_with("<!-- spec-spine:") {
                    return Err(format!("line {} is a stray delimiter", n + 1));
                } else {
                    region.body.push(line.to_string());
                }
            }
        }
    }
    if open.is_some() {
        return Err("a region is not closed".into());
    }
    let notes: Vec<&Region> = regions.iter().filter(|r| r.authored).collect();
    if notes.len() != 1 || notes[0].id != "notes" {
        return Err("the page has no single authored notes region".into());
    }
    if !regions.last().is_some_and(|r| r.authored) {
        return Err("the authored notes region is not last".into());
    }
    let body = notes_text(notes[0]);
    if body.len() > REFERENCE_MAX_NOTES_BYTES {
        return Err(format!(
            "the notes region exceeds {REFERENCE_MAX_NOTES_BYTES} bytes"
        ));
    }
    if notes[0].body.iter().any(|l| {
        l.trim_start_matches([' ', '\t'])
            .starts_with("<!-- spec-spine:")
    }) {
        return Err("the notes region holds a line that reads as a delimiter".into());
    }
    Ok(regions)
}

fn notes_text(region: &Region) -> String {
    // Each body line ended in a newline on the page, so a region holding one
    // blank line round-trips as "\n", not as empty notes.
    region.body.iter().map(|l| format!("{l}\n")).collect()
}

// ===== export and check =====

/// Build every model and page, check every limit, and decide the orphans,
/// before anything is written (3.6, 3.9, 3.12). A malformed committed page
/// or an orphan with notes refuses; a stale ledger is [`Error::Stale`].
pub fn reference_export(
    request: &ReferenceRequest,
    inputs: &ReferenceInputs,
) -> Result<ReferenceExport, Error> {
    if !inputs.ledger_fresh {
        return Err(Error::Stale {
            expected: "a fresh committed registry and index".into(),
            actual: "a stale ledger".into(),
        });
    }
    let built = build(request, inputs, true)?;
    let mut remove = Vec::new();
    for (name, bytes) in &inputs.existing_pages {
        let id = page_spec_id(name).expect("pattern-matched");
        if built.iter().any(|(m, _)| m.spec_id == id) {
            continue;
        }
        let regions = parse_page(bytes).map_err(|why| {
            Error::Refused(format!(
                "page-malformed: {}/{name}: {why}",
                request.output_dir
            ))
        })?;
        let notes = regions
            .iter()
            .find(|r| r.authored)
            .map(notes_text)
            .unwrap_or_default();
        if !notes.is_empty() {
            return Err(Error::Refused(format!(
                "orphaned-notes: {}/{name} is no longer exported and its notes region is not \
                 empty; move the text or delete the file",
                request.output_dir
            )));
        }
        remove.push(format!("{}/{name}", request.output_dir));
    }
    Ok(ReferenceExport {
        pages: built
            .iter()
            .map(|(m, text)| (m.path.clone(), text.clone()))
            .collect(),
        models: built.into_iter().map(|(m, _)| m).collect(),
        remove,
    })
}

/// Every exported spec's model and page, carrying committed notes forward.
/// With `strict`, a malformed committed page refuses; without it, its notes
/// are taken as empty (the check reports it instead).
fn build(
    request: &ReferenceRequest,
    inputs: &ReferenceInputs,
    strict: bool,
) -> Result<Vec<(ReferencePage, String)>, Error> {
    let specs = exported(&inputs.registry, &request.specs)?;
    check_page_count(request, specs.len())?;
    let mut out = Vec::with_capacity(specs.len());
    for r in specs {
        let name = format!("{}.md", r.id);
        let notes = match inputs.existing_pages.get(&name) {
            None => String::new(),
            Some(bytes) => match parse_page(bytes) {
                Ok(regions) => regions
                    .iter()
                    .find(|x| x.authored)
                    .map(notes_text)
                    .unwrap_or_default(),
                Err(why) if strict => {
                    return Err(Error::Refused(format!(
                        "page-malformed: {}/{name}: {why}",
                        request.output_dir
                    )));
                }
                Err(_) => String::new(),
            },
        };
        let model = page_model(request, inputs, r, notes);
        if model.signatures.len() > request.limits.max_signatures {
            return Err(Error::Refused(format!(
                "limit-exceeded: spec {} has {} signatures, above maxSignatures {}",
                r.id,
                model.signatures.len(),
                request.limits.max_signatures
            )));
        }
        let text = render_reference_page(&model);
        if text.len() > request.limits.max_page_bytes {
            return Err(Error::Refused(format!(
                "limit-exceeded: the page for {} is {} bytes, above maxPageBytes {}",
                r.id,
                text.len(),
                request.limits.max_page_bytes
            )));
        }
        out.push((model, text));
    }
    Ok(out)
}

/// The regeneration check (3.10), pure: generate in memory and compare with
/// the committed pages. Never writes and never refuses on page content.
pub fn reference_check_with(
    request: &ReferenceRequest,
    inputs: &ReferenceInputs,
) -> Result<ReferenceCheck, Error> {
    let mut findings = Vec::new();
    if !inputs.ledger_fresh {
        findings.push(ReferenceFinding {
            kind: ReferenceFindingKind::LedgerStale,
            path: None,
            regions: Vec::new(),
            detail: "the committed registry or index is not fresh".into(),
        });
    } else {
        let built = build(request, inputs, false)?;
        let exported: BTreeSet<String> = built
            .iter()
            .map(|(m, _)| format!("{}.md", m.spec_id))
            .collect();
        for (model, text) in &built {
            let name = format!("{}.md", model.spec_id);
            match inputs.existing_pages.get(&name) {
                None => findings.push(finding(
                    ReferenceFindingKind::PageMissing,
                    &model.path,
                    Vec::new(),
                    "no committed page",
                )),
                Some(bytes) => match parse_page(bytes) {
                    Err(why) => findings.push(finding(
                        ReferenceFindingKind::PageMalformed,
                        &model.path,
                        Vec::new(),
                        &why,
                    )),
                    Ok(committed) => {
                        if bytes != text.as_bytes() {
                            let fresh = parse_page(text.as_bytes()).expect("generated pages parse");
                            let regions = differing_regions(&fresh, &committed);
                            findings.push(finding(
                                ReferenceFindingKind::PageChanged,
                                &model.path,
                                regions,
                                &format!(
                                    "committed {}, generated {}",
                                    sha256(bytes),
                                    sha256(text.as_bytes())
                                ),
                            ));
                        }
                    }
                },
            }
        }
        for name in inputs.existing_pages.keys() {
            if !exported.contains(name) {
                let path = format!("{}/{name}", request.output_dir);
                findings.push(finding(
                    ReferenceFindingKind::PageOrphaned,
                    &path,
                    Vec::new(),
                    "no exported spec has this page",
                ));
            }
        }
    }
    findings.sort_by(|a, b| (&a.path, a.kind).cmp(&(&b.path, b.kind)));
    Ok(ReferenceCheck {
        schema_version: REFERENCE_PAGE_SCHEMA_VERSION.to_string(),
        output_dir: request.output_dir.clone(),
        findings,
    })
}

fn finding(
    kind: ReferenceFindingKind,
    path: &str,
    regions: Vec<String>,
    detail: &str,
) -> ReferenceFinding {
    ReferenceFinding {
        kind,
        path: Some(path.to_string()),
        regions,
        detail: detail.to_string(),
    }
}

/// The region ids whose bytes differ, in page order, including any region
/// present on one side only. A page whose only difference is outside the
/// regions' bodies names no region.
fn differing_regions(fresh: &[Region], committed: &[Region]) -> Vec<String> {
    REFERENCE_REGIONS
        .iter()
        .filter(|id| {
            let a = fresh.iter().find(|r| r.id == **id).map(|r| &r.body);
            let b = committed.iter().find(|r| r.id == **id).map(|r| &r.body);
            a != b
        })
        .map(|s| s.to_string())
        .collect()
}

/// Read a committed page, never more than `max_page_bytes` plus one byte
/// (spec 166 3.12: limits before unbounded allocation). A larger page is
/// `limit-exceeded`, the refusal a generated page of that size gets.
fn read_committed_page(
    path: &Path,
    display: &str,
    max_page_bytes: usize,
) -> Result<Vec<u8>, Error> {
    use std::io::Read;
    let file = fs::File::open(path).map_err(|e| Error::Io(format!("read {display}: {e}")))?;
    let mut bytes = Vec::new();
    file.take(max_page_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| Error::Io(format!("read {display}: {e}")))?;
    if bytes.len() > max_page_bytes {
        return Err(Error::Refused(format!(
            "limit-exceeded: the committed page {display} is above maxPageBytes {max_page_bytes}"
        )));
    }
    Ok(bytes)
}

/// Pair each symbol with its resolution, one to one and in order, checking
/// each resolution's own selector names that symbol. A resolver that returned
/// a different count or order is [`Error::Internal`], never a signature filed
/// under the wrong symbol.
fn pair_resolutions(
    symbols: &BTreeSet<String>,
    resolutions: Vec<Resolution>,
) -> Result<Vec<(&String, Resolution)>, Error> {
    if resolutions.len() != symbols.len() {
        return Err(Error::Internal(format!(
            "{} signature selector(s) resolved to {} result(s)",
            symbols.len(),
            resolutions.len()
        )));
    }
    symbols
        .iter()
        .zip(resolutions)
        .map(|(id, r)| match &r.selector {
            ContentSelector::Symbol { id: resolved, .. } if resolved == id => Ok((id, r)),
            other => Err(Error::Internal(format!(
                "the signature of {id} resolved selector {other:?}"
            ))),
        })
        .collect()
}

// ===== verbs and facades =====

/// `reference generate`: gather, then build the export as data.
pub fn reference_generate(
    cfg: &Config,
    repo_root: &Path,
    request: &ReferenceRequest,
) -> Result<ReferenceExport, Error> {
    let inputs = gather_reference_inputs(cfg, repo_root, request)?;
    reference_export(request, &inputs)
}

/// `reference check`.
pub fn reference_check(
    cfg: &Config,
    repo_root: &Path,
    request: &ReferenceRequest,
) -> Result<ReferenceCheck, Error> {
    let Some(inputs) = gather_if_fresh(cfg, repo_root, request)? else {
        return Ok(ReferenceCheck {
            schema_version: REFERENCE_PAGE_SCHEMA_VERSION.to_string(),
            output_dir: request.output_dir.clone(),
            findings: vec![ReferenceFinding {
                kind: ReferenceFindingKind::LedgerStale,
                path: None,
                regions: Vec::new(),
                detail: "the committed registry or index is not fresh".into(),
            }],
        });
    };
    reference_check_with(request, &inputs)
}

/// `reference model`: every page model, in spec order. A stale ledger is
/// [`Error::Stale`].
pub fn reference_model(
    cfg: &Config,
    repo_root: &Path,
    request: &ReferenceRequest,
) -> Result<ReferenceModel, Error> {
    let export = reference_generate(cfg, repo_root, request)?;
    Ok(ReferenceModel {
        schema_version: REFERENCE_PAGE_SCHEMA_VERSION.to_string(),
        pages: export.models,
    })
}

/// The canonical model document.
pub fn reference_model_document(model: &ReferenceModel) -> Result<String, Error> {
    read_document(model, Versioning::Preexisting("schemaVersion"))
}

/// The canonical check document.
pub fn reference_check_document(check: &ReferenceCheck) -> Result<String, Error> {
    read_document(check, Versioning::Preexisting("schemaVersion"))
}

#[derive(Serialize)]
struct PagesDocument<'a> {
    pages: Vec<PageFile<'a>>,
    remove: &'a [String],
}

#[derive(Serialize)]
struct PageFile<'a> {
    path: &'a str,
    content: &'a str,
}

/// JSON facade for [`reference_generate`] (spec 166 2): `{ "pages": [{ "path",
/// "content" }], "remove": [...] }`. Writes nothing.
pub fn reference_pages_json(
    config_json: &str,
    repo_root: &str,
    request_json: &str,
) -> Result<String, Error> {
    let (cfg, request) = facade_args(config_json, request_json)?;
    let export = reference_generate(&cfg, Path::new(repo_root), &request)?;
    canonical_json::to_string(&PagesDocument {
        pages: export
            .pages
            .iter()
            .map(|(path, content)| PageFile { path, content })
            .collect(),
        remove: &export.remove,
    })
}

/// JSON facade for [`reference_check`].
pub fn reference_check_json(
    config_json: &str,
    repo_root: &str,
    request_json: &str,
) -> Result<String, Error> {
    let (cfg, request) = facade_args(config_json, request_json)?;
    reference_check_document(&reference_check(&cfg, Path::new(repo_root), &request)?)
}

fn facade_args(config_json: &str, request_json: &str) -> Result<(Config, ReferenceRequest), Error> {
    let cfg: Config = serde_json::from_str(config_json)
        .map_err(|e| Error::Usage(format!("invalid config JSON: {e}")))?;
    let request: ReferenceRequest = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid reference request: {e}")))?;
    Ok((cfg, request))
}

fn sha256(bytes: &[u8]) -> String {
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
    fn free_text_cannot_break_a_cell_or_form_a_delimiter() {
        assert_eq!(
            esc("a | b <!-- spec-spine:end generated id=\"header\" --> & \\\r\nc"),
            "a \\| b &lt;!-- spec-spine:end generated id=\"header\" --&gt; &amp; \\\\ c"
        );
    }

    #[test]
    fn code_spans_and_fences_outgrow_their_backtick_runs() {
        assert_eq!(code("a"), "`a`");
        assert_eq!(code("a``b"), "```a``b```");
        assert_eq!(code("`a"), "`` `a ``");
        assert_eq!(fence("x", "rust"), "```rust\nx\n```\n");
        assert_eq!(fence("```\n", ""), "````\n```\n````\n");
    }

    #[test]
    fn a_delimiter_inside_a_fence_is_content() {
        let page = format!(
            "{}\n````\n<!-- spec-spine:end generated id=\"a\" -->\n````\n{}\n\n{}\n{}\n",
            delimiter(true, false, "a"),
            delimiter(false, false, "a"),
            delimiter(true, true, "notes"),
            delimiter(false, true, "notes")
        );
        let regions = parse_page(page.as_bytes()).unwrap();
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].body.len(), 3);
    }

    #[test]
    fn a_resolution_is_never_filed_under_another_symbol() {
        let symbols: BTreeSet<String> = ["crate::a", "crate::b"].map(String::from).into();
        let resolution = |id: &str| Resolution {
            identity: id.into(),
            projection: ContentProjection::Signature,
            selector: ContentSelector::Symbol {
                id: id.into(),
                projection: Some(ContentProjection::Signature),
                required: true,
            },
            outcome: Err((
                spec_spine_types::ContentOmissionReason::MissingContent,
                String::new(),
            )),
        };
        let paired = pair_resolutions(
            &symbols,
            vec![resolution("crate::a"), resolution("crate::b")],
        )
        .unwrap();
        assert_eq!(paired[1].0, "crate::b");
        for short_or_swapped in [
            vec![resolution("crate::a")],
            vec![resolution("crate::b"), resolution("crate::a")],
        ] {
            assert!(matches!(
                pair_resolutions(&symbols, short_or_swapped),
                Err(Error::Internal(_))
            ));
        }
    }

    #[test]
    fn a_committed_page_is_read_only_up_to_the_page_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let page = tmp.path().join("001-a.md");
        fs::write(&page, "x".repeat(8)).unwrap();
        assert_eq!(read_committed_page(&page, "p", 8).unwrap().len(), 8);
        match read_committed_page(&page, "p", 7) {
            Err(Error::Refused(m)) => assert!(m.starts_with("limit-exceeded:"), "{m}"),
            other => panic!("expected limit-exceeded, got {:?}", other.map(|b| b.len())),
        }
    }

    #[test]
    fn malformed_pages_say_why() {
        assert!(parse_page(b"stray\n").is_err());
        assert!(parse_page(&[0xff]).is_err());
        let two = format!(
            "{a}\n{b}\n\n{a}\n{b}\n",
            a = delimiter(true, true, "notes"),
            b = delimiter(false, true, "notes")
        );
        assert!(parse_page(two.as_bytes()).is_err());
        let indented = format!(
            "{}\n  <!-- spec-spine:x -->\n{}\n",
            delimiter(true, true, "notes"),
            delimiter(false, true, "notes")
        );
        assert!(parse_page(indented.as_bytes()).is_err());
    }
}
