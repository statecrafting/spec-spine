//! Documentation manifests and their freshness (spec 160).
//!
//! A manifest records what one declared generator invocation says it read and
//! produced, bound to one complete spec 159 context-packet page set. This
//! module validates one against the pages and output bytes it names, and
//! evaluates its freshness against one current snapshot, current pages,
//! caller-supplied interface exports and external resolutions, and current
//! output bytes.
//!
//! Pure over its arguments, except [`read_documentation_outputs`], which reads
//! the named output files beneath a caller-named output root and nothing else.
//! Nothing here runs a generator or an example, opens a network connection,
//! reads Git, or judges whether documentation is true (spec 160 3.16).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};
use spec_spine_types::{
    Config, ContentCompleteness, ContentProjection, ContentSnapshot, ContextPacket,
    DOCUMENTATION_MANIFEST_SCHEMA_VERSION, DocumentationFreshness, DocumentationManifest, Error,
    ExternalResolution, FreshnessFinding, FreshnessReason, FreshnessStatus, InterfaceReference,
    MANIFEST_LABEL_LIMIT, MANIFEST_MAX_BYTES, MANIFEST_MAX_EXAMPLES,
    MANIFEST_MAX_EXTERNAL_REFERENCES, MANIFEST_MAX_INPUTS, MANIFEST_MAX_INTERFACES,
    MANIFEST_MAX_OUTPUTS, MANIFEST_MAX_REGIONS, ManifestOutput, ManifestRegion, Outcome,
    PacketOmissionReason, PacketRequirement, PacketSnapshot, RegionKind, parse_semver,
    valid_corpus_name, valid_digest,
};

use crate::interface::{Exports, verify_reference};
use crate::{Versioning, canonical_json, context_packet::packet_digest, hash, read_document};

/// The current bytes of each declared output, by output path. `None` is a
/// file that does not exist in the snapshot.
pub type OutputBytes = BTreeMap<String, Option<Vec<u8>>>;

/// The finding identity of the packet (spec 160 D-16).
const PACKET_IDENTITY: &str = "packet";

/// Parse a manifest document. The size limit is checked before the text is
/// deserialized, a continuation member is refused by name, and an unknown
/// member is a usage error (spec 160 3.2, 3.13).
pub fn parse_documentation_manifest(text: &str) -> Result<DocumentationManifest, Error> {
    if text.len() > MANIFEST_MAX_BYTES {
        return Err(Error::Usage(format!(
            "limit-exceeded: the manifest is {} bytes, above the {MANIFEST_MAX_BYTES}-byte limit",
            text.len()
        )));
    }
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| Error::Usage(format!("invalid documentation manifest: {e}")))?;
    if value.get("continuation").is_some() {
        return Err(Error::Usage(
            "continuation-unsupported: a documentation manifest is one atomic record".into(),
        ));
    }
    serde_json::from_value(value)
        .map_err(|e| Error::Usage(format!("invalid documentation manifest: {e}")))
}

/// The manifest with every collection in canonical order (spec 160 3.13).
/// Generator and example argument vectors keep their order.
pub fn canonical_manifest(manifest: &DocumentationManifest) -> DocumentationManifest {
    let mut m = manifest.clone();
    let by = |a: &str, b: &str| a.as_bytes().cmp(b.as_bytes());
    for i in &mut m.inputs {
        sort_ids(&mut i.uses);
    }
    m.inputs.sort_by(|a, b| by(&a.input_id, &b.input_id));
    for i in &mut m.interfaces {
        sort_ids(&mut i.uses);
        i.sections.sort_by(|a, b| by(&a.anchor, &b.anchor));
    }
    m.interfaces.sort_by(|a, b| by(&a.input_id, &b.input_id));
    for e in &mut m.external_references {
        sort_ids(&mut e.uses);
    }
    m.external_references
        .sort_by(|a, b| by(&a.input_id, &b.input_id));
    for o in &mut m.omissions {
        sort_ids(&mut o.uses);
    }
    m.omissions
        .sort_by(|a, b| by(&a.input_id, &b.input_id).then(a.reason.cmp(&b.reason)));
    for o in &mut m.outputs {
        for r in &mut o.regions {
            sort_ids(&mut r.input_ids);
        }
        o.regions.sort_by(|a, b| {
            (a.span.start_line, a.span.end_line)
                .cmp(&(b.span.start_line, b.span.end_line))
                .then_with(|| by(&a.region_id, &b.region_id))
        });
        for e in &mut o.examples {
            sort_ids(&mut e.input_ids);
        }
        o.examples.sort_by(|a, b| by(&a.example_id, &b.example_id));
    }
    m.outputs.sort_by(|a, b| by(&a.output_id, &b.output_id));
    m
}

/// `sha256:` over the canonical manifest with `manifestDigest` absent.
pub fn manifest_digest(manifest: &DocumentationManifest) -> Result<String, Error> {
    let mut value = serde_json::to_value(canonical_manifest(manifest))
        .map_err(|e| Error::Internal(e.to_string()))?;
    if let Some(map) = value.as_object_mut() {
        map.remove("manifestDigest");
    }
    digest_of(&value)
}

/// The canonical manifest document: sorted keys, LF, one trailing newline.
pub fn documentation_manifest_document(manifest: &DocumentationManifest) -> Result<String, Error> {
    read_document(
        &canonical_manifest(manifest),
        Versioning::Preexisting("schemaVersion"),
    )
}

/// The canonical freshness document.
pub fn documentation_freshness_document(result: &DocumentationFreshness) -> Result<String, Error> {
    read_document(result, Versioning::Preexisting("schemaVersion"))
}

/// Validate a manifest against the snapshot it was recorded at (spec 160
/// 3.1 to 3.9, 3.14): its own structure and digest, the complete page set it
/// names, and the output bytes read from `snapshot`. Returns the canonical
/// manifest.
///
/// Malformed members, duplicate ids, invalid digests or regions and limit
/// excesses are [`Error::Usage`]; a page set, input, output or snapshot that
/// does not match the declaration is [`Error::Refused`].
pub fn validate_documentation_manifest(
    manifest: &DocumentationManifest,
    pages: &[ContextPacket],
    snapshot: &PacketSnapshot,
    outputs: &OutputBytes,
) -> Result<DocumentationManifest, Error> {
    let m = validate_manifest_inputs(manifest, pages, snapshot)?;
    check_outputs(&m, outputs)?;
    Ok(m)
}

/// Everything [`validate_documentation_manifest`] checks except the output
/// bytes: structure and digest, the snapshot, the complete page set, and
/// every input and omission against those pages. The edit planner (spec 165
/// 3.7) validates a replacement manifest this way, because the outputs it
/// describes are what the change itself writes.
pub(crate) fn validate_manifest_inputs(
    manifest: &DocumentationManifest,
    pages: &[ContextPacket],
    snapshot: &PacketSnapshot,
) -> Result<DocumentationManifest, Error> {
    let m = check_structure(manifest)?;
    if snapshot != &m.packet.snapshot {
        return Err(Error::Refused(
            "snapshot-mismatch: the bound snapshot is not the manifest's recorded snapshot".into(),
        ));
    }
    check_pages(pages, &m.packet.snapshot, &m.packet.request_digest, true)?;
    if pages.len() != m.packet.page_digests.len() {
        return Err(Error::Refused(format!(
            "packet-mismatch: the manifest names {} pages and {} were supplied",
            m.packet.page_digests.len(),
            pages.len()
        )));
    }
    for (i, page) in pages.iter().enumerate() {
        if page.packet_digest != m.packet.page_digests[i]
            || page.completeness != m.packet.page_completeness[i]
            || page.schema_version != m.packet.schema_version
            || page.closure_digest != m.packet.closure_digest
        {
            return Err(Error::Refused(format!(
                "packet-mismatch: page {} is not the page the manifest records",
                i + 1
            )));
        }
    }
    let current = PacketView::of(pages);
    for input in &m.inputs {
        match current.state(&input.identity, input.projection) {
            MemberState::Present { digest, schema } => {
                if digest != input.digest || schema != input.schema_version {
                    return Err(Error::Refused(format!(
                        "input-mismatch: input '{}' does not carry the packet's digest and schema",
                        input.input_id
                    )));
                }
            }
            MemberState::Omitted(reason) if input.requirement == PacketRequirement::Optional => {
                return Err(Error::Refused(format!(
                    "input-omitted: optional input '{}' is omitted by the packet ({}); record \
                     it under omissions",
                    input.input_id,
                    reason_name(reason)
                )));
            }
            _ => {
                return Err(Error::Refused(format!(
                    "input-missing: input '{}' is not in the packet",
                    input.input_id
                )));
            }
        }
    }
    for o in &m.omissions {
        match current.omission(&o.identity, o.projection) {
            Some((reason, PacketRequirement::Optional)) if reason == o.reason => {}
            _ => {
                return Err(Error::Refused(format!(
                    "omission-mismatch: omission '{}' is not an optional packet omission with \
                     reason {}",
                    o.input_id,
                    reason_name(o.reason)
                )));
            }
        }
    }
    Ok(m)
}

/// Spec 160 3.7 to 3.9: every output exists in the snapshot with its recorded
/// digest, length and region coverage.
fn check_outputs(m: &DocumentationManifest, outputs: &OutputBytes) -> Result<(), Error> {
    for output in &m.outputs {
        let bytes = outputs
            .get(output.path.as_str())
            .and_then(|b| b.as_ref())
            .ok_or_else(|| {
                Error::Refused(format!(
                    "output-missing: output '{}' does not exist in the snapshot",
                    output.output_id
                ))
            })?;
        let text = output_text(bytes).ok_or_else(|| {
            Error::Refused(format!(
                "unsupported-output: output '{}' is not UTF-8 text",
                output.output_id
            ))
        })?;
        if sha256(text.as_bytes()) != output.digest || text.len() as u64 != output.byte_length {
            return Err(Error::Refused(format!(
                "output-mismatch: output '{}' does not have its recorded digest and length",
                output.output_id
            )));
        }
        let lines = Lines::of(&text);
        let last = output.regions.last().map_or(0, |r| r.span.end_line);
        if last != lines.count() {
            return Err(Error::Refused(format!(
                "undeclared-output-bytes: output '{}' has {} lines and its regions cover {}",
                output.output_id,
                lines.count(),
                last
            )));
        }
        for r in &output.regions {
            if lines.digest(r.span.start_line, r.span.end_line).as_deref() != Some(&r.digest) {
                return Err(Error::Refused(format!(
                    "region-mismatch: region '{}#{}' does not have its recorded digest",
                    output.output_id, r.region_id
                )));
            }
        }
    }
    Ok(())
}

/// Evaluate a manifest's freshness (spec 160 3.10, 3.11, 3.14) against one
/// current snapshot, its pages for the recorded request, interface exports,
/// external resolutions and current output bytes. Returns one finding per
/// difference; a result too large to hold every finding refuses rather than
/// truncating (3.12).
pub fn documentation_freshness(
    manifest: &DocumentationManifest,
    current_snapshot: &PacketSnapshot,
    pages: &[ContextPacket],
    exports: &Exports,
    resolutions: &[ExternalResolution],
    outputs: &OutputBytes,
) -> Result<DocumentationFreshness, Error> {
    let m = check_structure(manifest)?;
    check_snapshot(current_snapshot)?;
    check_pages(pages, current_snapshot, &m.packet.request_digest, false)?;
    if let Some(page) = pages.iter().find(|p| {
        parse_semver(&p.schema_version).map(|v| v.0)
            != parse_semver(&m.packet.schema_version).map(|v| v.0)
    }) {
        return Err(Error::Refused(format!(
            "unsupported-schema: a current page carries context-packet schema {}, another \
             major than the recorded {}",
            page.schema_version, m.packet.schema_version
        )));
    }
    let resolved = index_resolutions(resolutions)?;
    let mut findings = Vec::new();

    // A page digest binds its snapshot, so under another snapshot it differs
    // by construction and says nothing 3.14's `repositoryTreeChanged` does
    // not. The packet identity is compared at the recorded snapshot, where a
    // difference is a real one (spec 160 D-17).
    let current_pages: Vec<&str> = pages.iter().map(|p| p.packet_digest.as_str()).collect();
    if current_snapshot == &m.packet.snapshot && current_pages != m.packet.page_digests {
        findings.push(finding(
            FreshnessReason::Changed,
            PACKET_IDENTITY.into(),
            PacketRequirement::Required,
            None,
            None,
            Vec::new(),
        ));
    }

    let current = PacketView::of(pages);
    for input in &m.inputs {
        let id = format!("input:{}", input.input_id);
        let recorded = Some(input.digest.clone());
        match current.state(&input.identity, input.projection) {
            MemberState::Present { digest, .. } if digest == input.digest => {}
            MemberState::Present { digest, .. } => findings.push(finding(
                FreshnessReason::Changed,
                id,
                input.requirement,
                recorded,
                Some(digest.to_string()),
                input.uses.clone(),
            )),
            MemberState::Omitted(reason) => findings.push(finding(
                omission_finding(reason),
                id,
                input.requirement,
                recorded,
                None,
                input.uses.clone(),
            )),
            MemberState::Absent => findings.push(finding(
                FreshnessReason::Missing,
                id,
                input.requirement,
                recorded,
                None,
                input.uses.clone(),
            )),
        }
    }
    for o in &m.omissions {
        let id = format!("input:{}", o.input_id);
        let now = match current.state(&o.identity, o.projection) {
            MemberState::Present { digest, .. } => {
                findings.push(finding(
                    FreshnessReason::Changed,
                    id,
                    PacketRequirement::Optional,
                    None,
                    Some(digest.to_string()),
                    o.uses.clone(),
                ));
                continue;
            }
            MemberState::Omitted(reason) => reason,
            MemberState::Absent => PacketOmissionReason::Missing,
        };
        if now != o.reason {
            findings.push(finding(
                omission_finding(now),
                id,
                PacketRequirement::Optional,
                None,
                None,
                o.uses.clone(),
            ));
        }
    }
    for i in &m.interfaces {
        let reference = InterfaceReference {
            corpus: i.corpus.clone(),
            spec: i.spec.clone(),
            digest: i.digest.clone(),
            sections: i.sections.clone(),
            obtained: String::new(),
            rationale: None,
        };
        let result = verify_reference(&i.input_id, &reference, exports);
        let reason = match result.outcome {
            Outcome::Current => continue,
            Outcome::SectionsCurrent | Outcome::Stale => FreshnessReason::Changed,
            Outcome::Missing => FreshnessReason::Missing,
            Outcome::Unverified => FreshnessReason::Unverified,
        };
        findings.push(finding(
            reason,
            format!("input:{}", i.input_id),
            i.requirement,
            Some(i.digest.clone()),
            result.observed_digest,
            i.uses.clone(),
        ));
    }
    for e in &m.external_references {
        let key = (
            e.namespace.as_str(),
            e.schema_version.as_str(),
            e.identity.as_str(),
        );
        let (reason, current_digest) = match resolved.get(&key) {
            None => (FreshnessReason::Unverified, None),
            Some(d) if *d == e.digest => continue,
            Some(d) => (FreshnessReason::Changed, Some(d.to_string())),
        };
        findings.push(finding(
            reason,
            format!("input:{}", e.input_id),
            e.requirement,
            Some(e.digest.clone()),
            current_digest,
            e.uses.clone(),
        ));
    }
    for output in &m.outputs {
        output_findings(output, outputs.get(output.path.as_str()), &mut findings);
    }

    findings.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.identity.as_bytes().cmp(b.identity.as_bytes()))
    });
    let mut result = DocumentationFreshness {
        schema_version: DOCUMENTATION_MANIFEST_SCHEMA_VERSION.to_string(),
        manifest_digest: m.manifest_digest.clone(),
        recorded_snapshot: m.packet.snapshot.clone(),
        current_snapshot: current_snapshot.clone(),
        repository_tree_changed: m.packet.snapshot.tree != current_snapshot.tree,
        status: status_of(&findings),
        findings,
        freshness_digest: String::new(),
    };
    result.freshness_digest = freshness_digest(&result)?;
    let size = documentation_freshness_document(&result)?.len();
    if size > MANIFEST_MAX_BYTES {
        return Err(Error::Refused(format!(
            "result-too-large: the freshness result needs {size} bytes, above the \
             {MANIFEST_MAX_BYTES}-byte limit"
        )));
    }
    Ok(result)
}

/// `sha256:` over the canonical freshness result with `freshnessDigest`
/// absent (spec 160 3.13).
pub fn freshness_digest(result: &DocumentationFreshness) -> Result<String, Error> {
    let mut value = serde_json::to_value(result).map_err(|e| Error::Internal(e.to_string()))?;
    if let Some(map) = value.as_object_mut() {
        map.remove("freshnessDigest");
    }
    digest_of(&value)
}

/// Read each declared output beneath `output_root`, a normalized
/// repository-relative directory or `.` (spec 160 3.8). A path outside the
/// output root, through `.git`, into a generated-artifact root the caller did
/// not select, or out of the root through a link is [`Error::Refused`]. A file
/// that does not exist is `None`.
pub fn read_documentation_outputs(
    cfg: &Config,
    repo_root: &Path,
    output_root: &str,
    manifest: &DocumentationManifest,
) -> Result<OutputBytes, Error> {
    let root_components = if output_root == "." {
        Vec::new()
    } else {
        normalized_components(output_root).ok_or_else(|| {
            Error::Usage(format!(
                "output root '{output_root}' is not a normalized repository-relative directory"
            ))
        })?
    };
    let generated: Vec<Vec<&str>> = [
        cfg.layout.derived_dir.as_str(),
        cfg.layout.state_dir.as_str(),
    ]
    .into_iter()
    .filter_map(|d| normalized_components(d.trim_end_matches('/')))
    .collect();
    let root_dir = repo_root.join(if output_root == "." { "" } else { output_root });
    let root_real = std::fs::canonicalize(&root_dir).map_err(|e| {
        Error::Refused(format!(
            "path-escape: output root '{output_root}' does not resolve in the snapshot: {e}"
        ))
    })?;
    let repo_real = std::fs::canonicalize(repo_root)
        .map_err(|e| Error::Io(format!("read {}: {e}", repo_root.display())))?;
    if !root_real.starts_with(&repo_real) {
        return Err(Error::Refused(format!(
            "path-escape: output root '{output_root}' leaves the repository"
        )));
    }
    let mut out = OutputBytes::new();
    for output in &manifest.outputs {
        let path = output.path.as_str();
        let components = normalized_components(path)
            .ok_or_else(|| Error::Usage(format!("output path '{path}' is not normalized")))?;
        if !components.starts_with(&root_components) || components.len() == root_components.len() {
            return Err(Error::Refused(format!(
                "path-escape: output '{}' is not beneath output root '{output_root}'",
                output.output_id
            )));
        }
        if components.iter().any(|c| c.eq_ignore_ascii_case(".git")) {
            return Err(Error::Refused(format!(
                "path-escape: output '{}' is inside .git",
                output.output_id
            )));
        }
        if let Some(g) = generated
            .iter()
            .find(|g| components.starts_with(g) && !root_components.starts_with(g))
        {
            return Err(Error::Refused(format!(
                "generated-root: output '{}' is inside generated root '{}', which the output \
                 root does not select",
                output.output_id,
                g.join("/")
            )));
        }
        let target = repo_root.join(path);
        let bytes = match std::fs::symlink_metadata(&target) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(Error::Io(format!("read output '{path}': {e}"))),
            Ok(_) => {
                let real = std::fs::canonicalize(&target).map_err(|e| {
                    Error::Refused(format!(
                        "path-escape: output '{}' does not resolve: {e}",
                        output.output_id
                    ))
                })?;
                if !real.starts_with(&root_real) {
                    return Err(Error::Refused(format!(
                        "path-escape: output '{}' resolves outside output root '{output_root}'",
                        output.output_id
                    )));
                }
                if real.is_file() {
                    Some(
                        std::fs::read(&real)
                            .map_err(|e| Error::Io(format!("read output '{path}': {e}")))?,
                    )
                } else {
                    None
                }
            }
        };
        out.insert(path.to_string(), bytes);
    }
    Ok(out)
}

/// JSON facade for validation (spec 160 2): the manifest, a JSON array of
/// its packet pages, the snapshot the export at `repo_root` was bound to, and
/// the output root. Returns the canonical manifest document.
pub fn documentation_manifest_json(
    config_json: &str,
    repo_root: &str,
    manifest_json: &str,
    pages_json: &str,
    snapshot_json: &str,
    output_root: &str,
) -> Result<String, Error> {
    let cfg = parse_json::<Config>(config_json, "config")?;
    let manifest = parse_documentation_manifest(manifest_json)?;
    let pages = parse_json::<Vec<ContextPacket>>(pages_json, "context-packet pages")?;
    let snapshot = packet_snapshot(&parse_json::<ContentSnapshot>(snapshot_json, "snapshot")?);
    let outputs = read_documentation_outputs(&cfg, Path::new(repo_root), output_root, &manifest)?;
    let m = validate_documentation_manifest(&manifest, &pages, &snapshot, &outputs)?;
    documentation_manifest_document(&m)
}

/// JSON facade for freshness (spec 160 2). `exports_json` maps a corpus name
/// to its exported specs by id (each `{path, text}`); `resolutions_json` is
/// an array of external resolutions. Returns the canonical freshness
/// document, whatever its `status`.
#[allow(clippy::too_many_arguments)]
pub fn documentation_freshness_json(
    config_json: &str,
    repo_root: &str,
    manifest_json: &str,
    pages_json: &str,
    snapshot_json: &str,
    exports_json: &str,
    resolutions_json: &str,
    output_root: &str,
) -> Result<String, Error> {
    let cfg = parse_json::<Config>(config_json, "config")?;
    let manifest = parse_documentation_manifest(manifest_json)?;
    let pages = parse_json::<Vec<ContextPacket>>(pages_json, "context-packet pages")?;
    let snapshot = packet_snapshot(&parse_json::<ContentSnapshot>(snapshot_json, "snapshot")?);
    let exports = parse_json::<Exports>(exports_json, "interface exports")?;
    let resolutions = parse_json::<Vec<ExternalResolution>>(resolutions_json, "resolutions")?;
    let outputs = read_documentation_outputs(&cfg, Path::new(repo_root), output_root, &manifest)?;
    let result = documentation_freshness(
        &manifest,
        &snapshot,
        &pages,
        &exports,
        &resolutions,
        &outputs,
    )?;
    documentation_freshness_document(&result)
}

/// The packet half of a spec 155 snapshot: its identity without the binding.
pub fn packet_snapshot(snapshot: &ContentSnapshot) -> PacketSnapshot {
    PacketSnapshot {
        repository: snapshot.repository.clone(),
        revision: snapshot.revision.clone(),
        tree: snapshot.tree.clone(),
        dirty_state: snapshot.dirty_state,
    }
}

// ---------------------------------------------------------------- structure

/// Every check that needs only the manifest (spec 160 3.2 to 3.9, 3.13):
/// returns the canonical manifest, or a usage error naming the first defect.
pub(crate) fn check_structure(
    manifest: &DocumentationManifest,
) -> Result<DocumentationManifest, Error> {
    let m = canonical_manifest(manifest);
    match parse_semver(&m.schema_version) {
        Some((major, minor, _))
            if Some(major) == parse_semver(DOCUMENTATION_MANIFEST_SCHEMA_VERSION).map(|v| v.0)
                && Some(minor)
                    <= parse_semver(DOCUMENTATION_MANIFEST_SCHEMA_VERSION).map(|v| v.1) => {}
        _ => {
            return usage(format!(
                "unsupported-schema: documentation-manifest schema '{}' is not served by {}",
                m.schema_version, DOCUMENTATION_MANIFEST_SCHEMA_VERSION
            ));
        }
    }
    let regions: usize = m.outputs.iter().map(|o| o.regions.len()).sum();
    let examples: usize = m.outputs.iter().map(|o| o.examples.len()).sum();
    for (name, count, limit) in [
        (
            "inputs and omissions",
            m.inputs.len() + m.omissions.len(),
            MANIFEST_MAX_INPUTS,
        ),
        ("interfaces", m.interfaces.len(), MANIFEST_MAX_INTERFACES),
        (
            "external references",
            m.external_references.len(),
            MANIFEST_MAX_EXTERNAL_REFERENCES,
        ),
        ("outputs", m.outputs.len(), MANIFEST_MAX_OUTPUTS),
        ("regions", regions, MANIFEST_MAX_REGIONS),
        ("examples", examples, MANIFEST_MAX_EXAMPLES),
    ] {
        if count > limit {
            return usage(format!(
                "limit-exceeded: {count} {name}, above the limit of {limit}"
            ));
        }
    }

    // The packet (3.3).
    let p = &m.packet;
    if parse_semver(&p.schema_version).is_none() {
        return usage("packet.schemaVersion is not MAJOR.MINOR.PATCH".into());
    }
    check_snapshot(&p.snapshot)?;
    digest_field("packet.requestDigest", &p.request_digest)?;
    if p.closure_digest.len() != 64
        || !p
            .closure_digest
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return usage("packet.closureDigest is not 64 lowercase hex".into());
    }
    if p.page_digests.is_empty() {
        return usage("packet.pageDigests must name at least one page".into());
    }
    if p.page_completeness.len() != p.page_digests.len() {
        return usage("packet.pageCompleteness must have one entry per page".into());
    }
    let mut seen = BTreeSet::new();
    for (i, d) in p.page_digests.iter().enumerate() {
        digest_field("packet.pageDigests", d)?;
        if !seen.insert(d) {
            return usage(format!("duplicate-page: page digest {d} is repeated"));
        }
        let expected = if i + 1 == p.page_digests.len() {
            ContentCompleteness::Complete
        } else {
            ContentCompleteness::Partial
        };
        if p.page_completeness[i] != expected {
            return usage(format!(
                "incomplete-packet: page {} must be {}, the page chain terminates at the last \
                 page and no page may omit a required member",
                i + 1,
                if expected == ContentCompleteness::Complete {
                    "complete"
                } else {
                    "partial"
                }
            ));
        }
    }

    // Inputs of every kind share one id space (3.4 to 3.6).
    let mut input_ids: BTreeSet<&str> = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for i in &m.inputs {
        add_input(&mut input_ids, &i.input_id)?;
        nonempty("input identity", &i.identity)?;
        if parse_semver(&i.schema_version).is_none() {
            return usage(format!(
                "input '{}' schemaVersion is not MAJOR.MINOR.PATCH",
                i.input_id
            ));
        }
        digest_field("input digest", &i.digest)?;
        if i.label
            .as_ref()
            .is_some_and(|l| l.len() > MANIFEST_LABEL_LIMIT)
        {
            return usage(format!(
                "input '{}' label is longer than {MANIFEST_LABEL_LIMIT} bytes",
                i.input_id
            ));
        }
        if !keys.insert((i.identity.as_str(), i.projection)) {
            return usage(format!(
                "duplicate-input: '{}' names a selected-content identity and projection \
                 another input names",
                i.input_id
            ));
        }
    }
    for o in &m.omissions {
        add_input(&mut input_ids, &o.input_id)?;
        nonempty("omission identity", &o.identity)?;
        if !keys.insert((o.identity.as_str(), o.projection)) {
            return usage(format!(
                "duplicate-input: omission '{}' names an identity and projection already \
                 declared",
                o.input_id
            ));
        }
    }
    for i in &m.interfaces {
        add_input(&mut input_ids, &i.input_id)?;
        if !valid_corpus_name(&i.corpus) {
            return usage(format!("interface '{}' corpus name is invalid", i.input_id));
        }
        nonempty("interface spec", &i.spec)?;
        digest_field("interface digest", &i.digest)?;
        let mut anchors = BTreeSet::new();
        for s in &i.sections {
            nonempty("interface section anchor", &s.anchor)?;
            digest_field("interface section digest", &s.digest)?;
            if !anchors.insert(&s.anchor) {
                return usage(format!(
                    "duplicate-id: interface '{}' pins section '{}' twice",
                    i.input_id, s.anchor
                ));
            }
        }
    }
    let mut external_keys = BTreeSet::new();
    for e in &m.external_references {
        add_input(&mut input_ids, &e.input_id)?;
        nonempty("external namespace", &e.namespace)?;
        nonempty("external identity", &e.identity)?;
        if parse_semver(&e.schema_version).is_none() {
            return usage(format!(
                "external reference '{}' schemaVersion is not MAJOR.MINOR.PATCH",
                e.input_id
            ));
        }
        digest_field("external digest", &e.digest)?;
        if !external_keys.insert((&e.namespace, &e.schema_version, &e.identity)) {
            return usage(format!(
                "duplicate-input: external reference '{}' repeats another's identity",
                e.input_id
            ));
        }
    }

    // The generator (3.7).
    let g = &m.generator;
    nonempty("generator name", &g.name)?;
    if parse_semver(&g.version).is_none() {
        return usage("generator version is not MAJOR.MINOR.PATCH".into());
    }
    digest_field("generator buildDigest", &g.build_digest)?;
    digest_field("generator configurationDigest", &g.configuration_digest)?;

    // Outputs, regions and examples (3.8, 3.9).
    let mut use_ids: BTreeSet<&str> = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut cited: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for o in &m.outputs {
        if !valid_id(&o.output_id) || !use_ids.insert(&o.output_id) {
            return usage(format!(
                "duplicate-id: outputId '{}' is invalid or declared twice",
                o.output_id
            ));
        }
        if normalized_components(o.path.as_str()).is_none() {
            return usage(format!(
                "output '{}' path '{}' is not a normalized repository-relative path",
                o.output_id,
                o.path.as_str()
            ));
        }
        if !paths.insert(o.path.as_str()) {
            return usage(format!(
                "duplicate-path: output path '{}' is declared twice",
                o.path.as_str()
            ));
        }
        if !valid_media_type(&o.media_type) {
            return usage(format!("output '{}' mediaType is invalid", o.output_id));
        }
        digest_field("output digest", &o.digest)?;
        check_regions(o)?;
        for r in &o.regions {
            for i in &r.input_ids {
                cited.entry(i.as_str()).or_default().insert(&o.output_id);
            }
        }
        let regions: BTreeSet<&str> = o.regions.iter().map(|r| r.region_id.as_str()).collect();
        for e in &o.examples {
            if !valid_id(&e.example_id) || !use_ids.insert(&e.example_id) {
                return usage(format!(
                    "duplicate-id: exampleId '{}' is invalid or declared twice",
                    e.example_id
                ));
            }
            if !regions.contains(e.region_id.as_str()) {
                return usage(format!(
                    "example '{}' names region '{}', which output '{}' does not declare",
                    e.example_id, e.region_id, o.output_id
                ));
            }
            if e.arguments.is_empty() {
                return usage(format!(
                    "example '{}' has an empty argument vector",
                    e.example_id
                ));
            }
            if e.working_directory != "." && normalized_components(&e.working_directory).is_none() {
                return usage(format!(
                    "example '{}' workingDirectory is not '.' or a normalized relative path",
                    e.example_id
                ));
            }
            unique_ids("example inputIds", &e.input_ids)?;
            for d in [&e.stdout_digest, &e.stderr_digest].into_iter().flatten() {
                digest_field("example stream digest", d)?;
            }
            for i in &e.input_ids {
                cited.entry(i.as_str()).or_default().insert(&e.example_id);
            }
        }
    }

    // Citations are reciprocal and every cited id exists (3.8, D-13).
    let declared_uses: Vec<(&str, &Vec<String>)> = m
        .inputs
        .iter()
        .map(|i| (i.input_id.as_str(), &i.uses))
        .chain(m.omissions.iter().map(|o| (o.input_id.as_str(), &o.uses)))
        .chain(m.interfaces.iter().map(|i| (i.input_id.as_str(), &i.uses)))
        .chain(
            m.external_references
                .iter()
                .map(|e| (e.input_id.as_str(), &e.uses)),
        )
        .collect();
    for cited_input in cited.keys() {
        if !input_ids.contains(cited_input) {
            return usage(format!(
                "dangling-id: '{cited_input}' is cited but no input declares it"
            ));
        }
    }
    for (id, uses) in declared_uses {
        if uses.is_empty() {
            return usage(format!("input '{id}' uses nothing"));
        }
        unique_ids("uses", uses)?;
        if let Some(u) = uses.iter().find(|u| !use_ids.contains(u.as_str())) {
            return usage(format!(
                "dangling-id: input '{id}' uses '{u}', which is no declared output or example"
            ));
        }
        let declared: BTreeSet<&str> = uses.iter().map(String::as_str).collect();
        let actual = cited.get(id).cloned().unwrap_or_default();
        if declared != actual {
            return usage(format!(
                "citation-mismatch: input '{id}' uses and the regions and examples that cite it \
                 disagree"
            ));
        }
    }

    // The digest last: it is over the canonical form just checked (3.13).
    digest_field("manifestDigest", &m.manifest_digest)?;
    let expected = manifest_digest(&m)?;
    if expected != m.manifest_digest {
        return usage(format!(
            "manifest-digest-mismatch: manifestDigest is {}, the canonical manifest digests to \
             {expected}",
            m.manifest_digest
        ));
    }
    Ok(m)
}

/// Regions are ordered, start at line 1, abut without gap or overlap, and a
/// generated region cites an input (spec 160 3.8). The line count is checked
/// against bytes only where bytes are supplied.
fn check_regions(o: &ManifestOutput) -> Result<(), Error> {
    let mut ids = BTreeSet::new();
    let mut next = 1usize;
    for r in &o.regions {
        if !valid_id(&r.region_id) || !ids.insert(&r.region_id) {
            return usage(format!(
                "duplicate-id: region '{}' of output '{}' is invalid or declared twice",
                r.region_id, o.output_id
            ));
        }
        if r.span.start_line == 0 || r.span.end_line < r.span.start_line {
            return usage(format!(
                "invalid-region: region '{}#{}' has a malformed span",
                o.output_id, r.region_id
            ));
        }
        if r.span.start_line != next {
            return usage(format!(
                "invalid-region: region '{}#{}' starts at line {}; regions must abut from \
                 line 1 without gap or overlap (expected {next})",
                o.output_id, r.region_id, r.span.start_line
            ));
        }
        next = r.span.end_line + 1;
        digest_field("region digest", &r.digest)?;
        unique_ids("region inputIds", &r.input_ids)?;
        if r.kind == RegionKind::Generated && r.input_ids.is_empty() {
            return usage(format!(
                "invalid-region: generated region '{}#{}' cites no input",
                o.output_id, r.region_id
            ));
        }
    }
    if o.regions.is_empty() && o.byte_length != 0 {
        return usage(format!(
            "undeclared-output-bytes: output '{}' has {} bytes and no region",
            o.output_id, o.byte_length
        ));
    }
    Ok(())
}

/// A page set is one consistent, terminating, non-overlapping chain (spec 160
/// D-8): one snapshot, request and closure; a continuation on every page but
/// the last; member keys strictly increasing across pages. `complete` also
/// refuses a required omission, which manifest validation needs and freshness
/// reports instead.
fn check_pages(
    pages: &[ContextPacket],
    snapshot: &PacketSnapshot,
    request_digest: &str,
    complete: bool,
) -> Result<(), Error> {
    let refuse = |msg: String| Err(Error::Refused(msg));
    let Some(first) = pages.first() else {
        return Err(Error::Usage(
            "at least one context-packet page is required".into(),
        ));
    };
    let mut digests = BTreeSet::new();
    let mut last_key: Option<(String, String)> = None;
    for (i, page) in pages.iter().enumerate() {
        let n = i + 1;
        if packet_digest(page)? != page.packet_digest {
            return refuse(format!(
                "packet-integrity: page {n} does not recompute to its packetDigest"
            ));
        }
        if !digests.insert(&page.packet_digest) {
            return refuse(format!("duplicate-page: page {n} repeats an earlier page"));
        }
        if &page.snapshot != snapshot {
            return refuse(format!(
                "mixed-snapshot: page {n} was read from another snapshot"
            ));
        }
        if page.request_digest != request_digest {
            return refuse(format!(
                "request-mismatch: page {n} answers another request"
            ));
        }
        if page.closure_digest != first.closure_digest {
            return refuse(format!(
                "broken-continuation: page {n} carries another closure"
            ));
        }
        let terminal = n == pages.len();
        if page.continuation.is_some() == terminal {
            return refuse(format!(
                "broken-continuation: page {n} {} a continuation",
                if terminal { "carries" } else { "lacks" }
            ));
        }
        if complete
            && page
                .omissions
                .iter()
                .any(|o| o.requirement == PacketRequirement::Required)
        {
            return refuse(format!(
                "incomplete-packet: page {n} omits a required member"
            ));
        }
        for member in &page.members {
            let key = (member.identity.clone(), projection_name(member.projection));
            if let Some(prev) = &last_key
                && (prev.0.as_bytes(), prev.1.as_bytes()) >= (key.0.as_bytes(), key.1.as_bytes())
            {
                return refuse(format!(
                    "broken-continuation: page {n} repeats or reorders member '{}'",
                    member.identity
                ));
            }
            last_key = Some(key);
        }
    }
    Ok(())
}

fn check_snapshot(s: &PacketSnapshot) -> Result<(), Error> {
    crate::content::validate_snapshot(&ContentSnapshot {
        repository: s.repository.clone(),
        revision: s.revision.clone(),
        tree: s.tree.clone(),
        dirty_state: s.dirty_state,
        binding: spec_spine_types::ContentSnapshotBinding::CallerSupplied,
    })
}

// ---------------------------------------------------------------- freshness

enum MemberState<'a> {
    Present { digest: &'a str, schema: &'a str },
    Omitted(PacketOmissionReason),
    Absent,
}

/// Every member and omission of a page set, by canonical key.
struct PacketView<'a> {
    members: BTreeMap<(&'a str, ContentProjection), (&'a str, &'a str)>,
    omissions: BTreeMap<(&'a str, ContentProjection), (PacketOmissionReason, PacketRequirement)>,
}

impl<'a> PacketView<'a> {
    fn of(pages: &'a [ContextPacket]) -> Self {
        let mut members = BTreeMap::new();
        let mut omissions = BTreeMap::new();
        for page in pages {
            for m in &page.members {
                members.insert(
                    (m.identity.as_str(), m.projection),
                    (m.item.digest.as_str(), m.item.schema_version.as_str()),
                );
            }
            for o in &page.omissions {
                omissions
                    .entry((o.identity.as_str(), o.projection))
                    .or_insert((o.reason, o.requirement));
            }
        }
        PacketView { members, omissions }
    }

    fn state(&self, identity: &str, projection: ContentProjection) -> MemberState<'a> {
        if let Some((digest, schema)) = self.members.get(&(identity, projection)) {
            return MemberState::Present { digest, schema };
        }
        match self.omissions.get(&(identity, projection)) {
            Some((reason, _)) => MemberState::Omitted(*reason),
            None => MemberState::Absent,
        }
    }

    fn omission(
        &self,
        identity: &str,
        projection: ContentProjection,
    ) -> Option<(PacketOmissionReason, PacketRequirement)> {
        if self.members.contains_key(&(identity, projection)) {
            return None;
        }
        self.omissions.get(&(identity, projection)).copied()
    }
}

/// The finding a spec 159 omission reason decides (spec 160 3.10).
fn omission_finding(reason: PacketOmissionReason) -> FreshnessReason {
    use PacketOmissionReason as R;
    match reason {
        R::Missing => FreshnessReason::Missing,
        R::Removed => FreshnessReason::Removed,
        R::Withdrawn => FreshnessReason::Withdrawn,
        R::Unresolved => FreshnessReason::Unresolved,
        R::Ambiguous => FreshnessReason::Ambiguous,
        R::UnsupportedSelector | R::UnsupportedProjection | R::NonText => {
            FreshnessReason::Unsupported
        }
        R::OversizedMember | R::ItemBudget | R::ByteBudget => FreshnessReason::Unverified,
    }
}

/// One output's findings: whole-file change, and within a changed file, each
/// region that moved or changed (spec 160 3.10).
fn output_findings(
    output: &ManifestOutput,
    current: Option<&Option<Vec<u8>>>,
    findings: &mut Vec<FreshnessFinding>,
) {
    let id = format!("output:{}", output.output_id);
    let examples: Vec<String> = output
        .examples
        .iter()
        .map(|e| e.example_id.clone())
        .collect();
    let recorded = Some(output.digest.clone());
    let Some(Some(bytes)) = current else {
        findings.push(finding(
            FreshnessReason::Missing,
            id,
            PacketRequirement::Required,
            recorded,
            None,
            examples,
        ));
        return;
    };
    let Some(text) = output_text(bytes) else {
        findings.push(finding(
            FreshnessReason::Unsupported,
            id,
            PacketRequirement::Required,
            recorded,
            None,
            examples,
        ));
        return;
    };
    let digest = sha256(text.as_bytes());
    if digest == output.digest {
        return;
    }
    findings.push(finding(
        FreshnessReason::OutputChanged,
        id,
        PacketRequirement::Required,
        recorded,
        Some(digest),
        examples,
    ));
    let lines = Lines::of(&text);
    for r in &output.regions {
        let at_span = lines.digest(r.span.start_line, r.span.end_line);
        if at_span.as_deref() == Some(r.digest.as_str()) {
            continue;
        }
        let affects = region_examples(output, r);
        let id = format!("region:{}#{}", output.output_id, r.region_id);
        let (kind, current_digest) = if lines.find(r).is_some() {
            (FreshnessReason::RegionShifted, Some(r.digest.clone()))
        } else {
            (FreshnessReason::Changed, at_span)
        };
        findings.push(finding(
            kind,
            id,
            PacketRequirement::Required,
            Some(r.digest.clone()),
            current_digest,
            affects,
        ));
    }
}

fn region_examples(output: &ManifestOutput, region: &ManifestRegion) -> Vec<String> {
    output
        .examples
        .iter()
        .filter(|e| e.region_id == region.region_id)
        .map(|e| e.example_id.clone())
        .collect()
}

/// Precedence `incomplete`, `stale`, `unverified`, `fresh` (spec 160 3.11):
/// the most severe status any finding decides.
fn status_of(findings: &[FreshnessFinding]) -> FreshnessStatus {
    use FreshnessReason as R;
    let absent = |k: R| {
        matches!(
            k,
            R::Missing | R::Removed | R::Withdrawn | R::Unresolved | R::Ambiguous | R::Unsupported
        )
    };
    let mut status = FreshnessStatus::Fresh;
    for f in findings {
        let s = if absent(f.kind) && f.requirement == PacketRequirement::Required {
            FreshnessStatus::Incomplete
        } else if absent(f.kind)
            || matches!(f.kind, R::Changed | R::RegionShifted | R::OutputChanged)
        {
            FreshnessStatus::Stale
        } else {
            FreshnessStatus::Unverified
        };
        status = status.max(s);
    }
    status
}

fn index_resolutions(
    resolutions: &[ExternalResolution],
) -> Result<BTreeMap<(&str, &str, &str), &str>, Error> {
    let mut out = BTreeMap::new();
    for r in resolutions {
        if !valid_digest(&r.digest) {
            return Err(Error::Usage(format!(
                "external resolution for '{}' has an invalid digest",
                r.identity
            )));
        }
        let key = (
            r.namespace.as_str(),
            r.schema_version.as_str(),
            r.identity.as_str(),
        );
        if out.insert(key, r.digest.as_str()).is_some() {
            return Err(Error::Usage(format!(
                "duplicate-id: two external resolutions name '{}'",
                r.identity
            )));
        }
    }
    Ok(out)
}

fn finding(
    kind: FreshnessReason,
    identity: String,
    requirement: PacketRequirement,
    recorded_digest: Option<String>,
    current_digest: Option<String>,
    mut affects: Vec<String>,
) -> FreshnessFinding {
    sort_ids(&mut affects);
    FreshnessFinding {
        kind,
        identity,
        requirement,
        recorded_digest,
        current_digest,
        affects,
    }
}

// ---------------------------------------------------------------- text

/// An output's normalized text, or `None` when its bytes are not UTF-8
/// (spec 160 3.8: binary outputs are unsupported in v1).
fn output_text(bytes: &[u8]) -> Option<String> {
    std::str::from_utf8(bytes).ok().map(hash::normalize)
}

/// A normalized text split into lines, each keeping its terminator, so the
/// regions' bytes concatenate to exactly the text.
struct Lines<'a> {
    text: &'a str,
    /// Byte offset of each line's start, plus the text's end.
    bounds: Vec<usize>,
}

impl<'a> Lines<'a> {
    fn of(text: &'a str) -> Self {
        let mut bounds = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                bounds.push(i + 1);
            }
        }
        if *bounds.last().expect("nonempty") != text.len() {
            bounds.push(text.len());
        }
        Lines { text, bounds }
    }

    fn count(&self) -> usize {
        self.bounds.len() - 1
    }

    /// The digest of lines `start..=end`, when they exist.
    fn digest(&self, start: usize, end: usize) -> Option<String> {
        if start == 0 || end < start || end > self.count() {
            return None;
        }
        Some(sha256(
            &self.text.as_bytes()[self.bounds[start - 1]..self.bounds[end]],
        ))
    }

    /// The first start line at which a span of the region's length carries
    /// its digest.
    fn find(&self, region: &ManifestRegion) -> Option<usize> {
        let len = region.span.end_line - region.span.start_line;
        (1..=self.count().saturating_sub(len))
            .find(|s| self.digest(*s, s + len).as_deref() == Some(region.digest.as_str()))
    }
}

// ---------------------------------------------------------------- helpers

fn add_input<'a>(ids: &mut BTreeSet<&'a str>, id: &'a str) -> Result<(), Error> {
    if !valid_id(id) {
        return usage(format!(
            "inputId '{id}' does not match ^[A-Za-z][A-Za-z0-9._-]*$"
        ));
    }
    if !ids.insert(id) {
        return usage(format!("duplicate-id: inputId '{id}' is declared twice"));
    }
    Ok(())
}

fn usage<T>(msg: String) -> Result<T, Error> {
    Err(Error::Usage(msg))
}

fn sort_ids(ids: &mut [String]) {
    ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
}

fn unique_ids(name: &str, ids: &[String]) -> Result<(), Error> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(Error::Usage(format!("duplicate-id: {name} repeats '{id}'")));
        }
    }
    Ok(())
}

fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn valid_media_type(t: &str) -> bool {
    let part = |p: &str| {
        let mut chars = p.chars();
        chars
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && chars
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "!#$&^_.+-".contains(c))
    };
    t.split_once('/').is_some_and(|(a, b)| part(a) && part(b))
}

fn nonempty(name: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::Usage(format!("{name} must not be empty")));
    }
    Ok(())
}

fn digest_field(name: &str, value: &str) -> Result<(), Error> {
    if !valid_digest(value) {
        return Err(Error::Usage(format!(
            "{name} '{value}' is not sha256:<64 lowercase hex>"
        )));
    }
    Ok(())
}

/// The components of a normalized repository-relative path: no empty, `.` or
/// `..` component, no leading or trailing separator, no backslash.
fn normalized_components(path: &str) -> Option<Vec<&str>> {
    if path.is_empty() || path.contains('\\') || spec_spine_types::repo_path_problem(path).is_some()
    {
        return None;
    }
    let parts: Vec<&str> = path.split('/').collect();
    if parts
        .iter()
        .any(|p| p.is_empty() || *p == "." || *p == "..")
    {
        return None;
    }
    Some(parts)
}

fn reason_name(reason: PacketOmissionReason) -> String {
    serde_json::to_value(reason)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn projection_name(projection: ContentProjection) -> String {
    serde_json::to_value(projection)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn parse_json<T: serde::de::DeserializeOwned>(text: &str, what: &str) -> Result<T, Error> {
    serde_json::from_str(text).map_err(|e| Error::Usage(format!("invalid {what} JSON: {e}")))
}

fn digest_of<T: Serialize>(value: &T) -> Result<String, Error> {
    Ok(sha256(canonical_json::to_string(value)?.as_bytes()))
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(71);
    out.push_str("sha256:");
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_keep_terminators_and_concatenate() {
        let l = Lines::of("a\nb\nc");
        assert_eq!(l.count(), 3);
        assert_eq!(l.digest(1, 3).unwrap(), sha256(b"a\nb\nc"));
        assert_eq!(l.digest(2, 2).unwrap(), sha256(b"b\n"));
        assert_eq!(l.digest(3, 4), None);
        assert_eq!(Lines::of("").count(), 0);
        assert_eq!(Lines::of("x\n").count(), 1);
    }

    #[test]
    fn normalized_paths_refuse_every_alternate_spelling() {
        assert!(normalized_components("docs/a.md").is_some());
        for bad in [
            "", ".", "./a", "a/", "a//b", "a/./b", "a/../b", "/a", "a\\b",
        ] {
            assert!(normalized_components(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn media_types_and_ids() {
        assert!(valid_media_type("text/markdown"));
        assert!(!valid_media_type("Text/markdown"));
        assert!(!valid_media_type("text"));
        assert!(valid_id("a.b_c-1"));
        assert!(!valid_id("1a"));
        assert!(!valid_id(""));
    }
}
