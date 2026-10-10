//! Repository-scoped context packets (spec 159).
//!
//! A packet binds a spec 107 closure and spec 155 selected-content members to
//! one caller-supplied repository snapshot. It reuses both identities: a member
//! is keyed by its selected-content identity and projection, and the closure
//! is carried with its own digest. What this module adds is composition only:
//! requirement, origin, omissions, warnings, completeness, an integrity-checked
//! continuation and two digests.
//!
//! Pure over its arguments and the named export: no Git, no network, no
//! execution, no writes. The CLI binds the snapshot; core never infers its
//! own build identity (D-5).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spec_spine_types::{
    CONTEXT_PACKET_SCHEMA_VERSION, Config, ContentCompleteness, ContentOmissionReason,
    ContentProjection, ContentSelector, ContentSnapshot, ContextPacket, Error,
    PACKET_MAX_BYTES_LIMIT, PACKET_MAX_ITEMS_LIMIT, PACKET_OPAQUE_LIMIT, PacketMember,
    PacketMemberRequest, PacketOmission, PacketOmissionReason, PacketOrigin, PacketProducer,
    PacketRequest, PacketRequirement, PacketSnapshot, PacketWarning, PacketWarningCode,
    parse_semver,
};

use crate::closure::{ClosureMember, ClosureRequest, SectionRef};
use crate::content::{Resolution, Selected, content_item, resolve_selectors, validate_snapshot};
use crate::{Versioning, canonical_json, read_document};

/// The build identity recorded when the binding supplies none (spec 159 3.5).
pub const UNVERIFIED_BUILD: &str = "unverified";

/// One canonical member: its key, the requests that named it, and what its
/// selector resolved to.
struct Member {
    identity: String,
    projection: ContentProjection,
    requirement: PacketRequirement,
    origins: Vec<PacketOrigin>,
    selector: Option<ContentSelector>,
    outcome: Outcome,
}

enum Outcome {
    Selected(Selected),
    Omitted(PacketOmissionReason, String),
}

/// The continuation's signed-by-checksum payload (spec 159 3.10).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContinuationPayload {
    schema_major: u64,
    snapshot_digest: String,
    request_digest: String,
    closure_digest: String,
    max_bytes: usize,
    max_items: usize,
    last_identity: String,
    last_projection: ContentProjection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContinuationToken {
    payload: ContinuationPayload,
    checksum: String,
}

/// Assemble one context-packet page from the export at `repo_root`.
///
/// `producer_build` is the binding's `sha256:<hex>` build digest, or `None`
/// to record the build as `unverified` (D-5). A packet whose required member
/// is omitted is still returned, with `completeness: incomplete`; deciding
/// that it is a finding is the caller's (spec 159 3.8). Refusals return no
/// packet: a malformed request is [`Error::Usage`], an unsupported consumer
/// schema, mismatched closure or stale continuation is [`Error::Refused`], a
/// stale ledger is [`Error::Stale`], and a closure reference that does not
/// resolve is [`Error::NotFound`].
pub fn context_packet(
    cfg: &Config,
    repo_root: &Path,
    request: &PacketRequest,
    snapshot: &ContentSnapshot,
    producer_build: Option<&str>,
) -> Result<ContextPacket, Error> {
    validate_request(request)?;
    validate_snapshot(snapshot)?;
    let build = match producer_build {
        None => UNVERIFIED_BUILD.to_string(),
        Some(b) if is_sha256(b) => b.to_string(),
        Some(_) => {
            return Err(Error::Usage(
                "producer build must be sha256:<64 lowercase hex>".into(),
            ));
        }
    };

    let normalized = normalize(request);
    let request_digest = digest_of(&normalized)?;
    let packet_snapshot = PacketSnapshot {
        repository: snapshot.repository.clone(),
        revision: snapshot.revision.clone(),
        tree: snapshot.tree.clone(),
        dirty_state: snapshot.dirty_state,
    };
    let snapshot_digest = digest_of(&packet_snapshot)?;

    let resolved = resolve_packet_closure(cfg, repo_root, request)?;
    let closure_document =
        serde_json::to_value(&resolved).map_err(|e| Error::Internal(e.to_string()))?;
    let closure_digest = resolved.digest.clone();

    let start = match &request.continuation {
        None => None,
        Some(token) => {
            let payload = decode_continuation(token)?;
            let expected = ContinuationPayload {
                last_identity: payload.last_identity.clone(),
                last_projection: payload.last_projection,
                ..continuation_payload(&snapshot_digest, &request_digest, &closure_digest, request)
            };
            if payload != expected {
                return Err(Error::Refused(
                    "stale-continuation: the continuation is bound to another request, \
                     snapshot, closure, schema major or budget"
                        .into(),
                ));
            }
            Some((payload.last_identity, payload.last_projection))
        }
    };

    let members = collect_members(
        cfg,
        repo_root,
        &resolved.members,
        &normalized.members,
        request,
    )?;

    let mut omissions = Vec::new();
    let mut pageable = Vec::new();
    for member in members {
        match member.outcome {
            Outcome::Omitted(reason, detail) => omissions.push(PacketOmission {
                identity: member.identity,
                projection: member.projection,
                requirement: member.requirement,
                origins: member.origins,
                reason,
                detail,
            }),
            Outcome::Selected(ref selected) if selected.content.len() > request.max_bytes => {
                omissions.push(PacketOmission {
                    detail: format!(
                        "member is {} bytes, above maxBytes {}",
                        selected.content.len(),
                        request.max_bytes
                    ),
                    identity: member.identity,
                    projection: member.projection,
                    requirement: member.requirement,
                    origins: member.origins,
                    reason: PacketOmissionReason::OversizedMember,
                })
            }
            Outcome::Selected(_) => pageable.push(member),
        }
    }

    let mut page = Vec::new();
    let mut bytes = 0usize;
    let mut continuation = None;
    let mut last: Option<(String, ContentProjection)> = None;
    for member in pageable.into_iter().filter(|m| match &start {
        None => true,
        Some(after) => key_cmp((&m.identity, m.projection), (&after.0, after.1)).is_gt(),
    }) {
        let Outcome::Selected(selected) = &member.outcome else {
            unreachable!("only selected members are pageable")
        };
        let size = selected.content.len();
        if page.len() == request.max_items || bytes + size > request.max_bytes {
            let (identity, projection) = last.clone().expect("a page holds at least one member");
            continuation = Some(encode_continuation(ContinuationPayload {
                last_identity: identity,
                last_projection: projection,
                ..continuation_payload(&snapshot_digest, &request_digest, &closure_digest, request)
            })?);
            break;
        }
        bytes += size;
        last = Some((member.identity.clone(), member.projection));
        let selector = member
            .selector
            .as_ref()
            .expect("a selected member has a selector");
        page.push(PacketMember {
            item: content_item(
                &member.identity,
                member.projection,
                selector,
                selected,
                snapshot,
            ),
            identity: member.identity,
            projection: member.projection,
            requirement: member.requirement,
            origins: member.origins,
        });
    }

    omissions.sort_by(|a, b| {
        key_cmp((&a.identity, a.projection), (&b.identity, b.projection))
            .then(a.reason.cmp(&b.reason))
    });
    let mut warnings = Vec::new();
    if build == UNVERIFIED_BUILD {
        warnings.push(PacketWarning {
            code: PacketWarningCode::UnverifiedProducerBuild,
            identity: None,
            projection: None,
        });
    }
    for o in &omissions {
        let unsupported = matches!(
            o.reason,
            PacketOmissionReason::Withdrawn
                | PacketOmissionReason::UnsupportedSelector
                | PacketOmissionReason::UnsupportedProjection
                | PacketOmissionReason::NonText
        );
        if unsupported && o.origins.contains(&PacketOrigin::Closure) {
            warnings.push(PacketWarning {
                code: PacketWarningCode::ClosureMemberUnsupported,
                identity: Some(o.identity.clone()),
                projection: Some(o.projection),
            });
        }
        if o.requirement == PacketRequirement::Optional {
            warnings.push(PacketWarning {
                code: PacketWarningCode::OptionalMemberOmitted,
                identity: Some(o.identity.clone()),
                projection: Some(o.projection),
            });
        }
    }
    warnings.sort_by(|a, b| {
        a.code.cmp(&b.code).then_with(|| {
            let ka = a.identity.as_deref().unwrap_or_default();
            let kb = b.identity.as_deref().unwrap_or_default();
            ka.cmp(kb)
                .then_with(|| proj_name(a.projection).cmp(&proj_name(b.projection)))
        })
    });

    let completeness = if omissions
        .iter()
        .any(|o| o.requirement == PacketRequirement::Required)
    {
        ContentCompleteness::Incomplete
    } else if continuation.is_some() {
        ContentCompleteness::Partial
    } else {
        ContentCompleteness::Complete
    };

    let mut packet = ContextPacket {
        schema_version: CONTEXT_PACKET_SCHEMA_VERSION.to_string(),
        producer: PacketProducer {
            package: "spec-spine".into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build,
        },
        snapshot: packet_snapshot,
        request: normalized,
        request_digest,
        closure: closure_document,
        closure_digest,
        members: page,
        omissions,
        warnings,
        completeness,
        continuation,
        packet_digest: String::new(),
    };
    packet.packet_digest = packet_digest(&packet)?;
    Ok(packet)
}

/// Write one packet page as its canonical document: sorted keys, two-space
/// indent, LF and one trailing newline, under its own `schemaVersion`.
pub fn context_packet_document(packet: &ContextPacket) -> Result<String, Error> {
    read_document(packet, Versioning::Preexisting("schemaVersion"))
}

/// JSON facade for [`context_packet`] (spec 159 2). The facade has no build
/// identity of its own to supply, so its packets record `unverified`. A
/// returned document may be `incomplete`: read `completeness`.
pub fn context_packet_json(
    config_json: &str,
    repo_root: &str,
    request_json: &str,
    snapshot_json: &str,
) -> Result<String, Error> {
    let cfg: Config = serde_json::from_str(config_json)
        .map_err(|e| Error::Usage(format!("invalid config JSON: {e}")))?;
    let request: PacketRequest = serde_json::from_str(request_json)
        .map_err(|e| Error::Usage(format!("invalid context-packet request: {e}")))?;
    let snapshot: ContentSnapshot = serde_json::from_str(snapshot_json)
        .map_err(|e| Error::Usage(format!("invalid content snapshot: {e}")))?;
    let packet = context_packet(&cfg, Path::new(repo_root), &request, &snapshot, None)?;
    context_packet_document(&packet)
}

/// The packet digest: `sha256:` over the canonical packet with `packetDigest`
/// absent and `continuation` present (spec 159 3.10).
pub fn packet_digest(packet: &ContextPacket) -> Result<String, Error> {
    let mut value = serde_json::to_value(packet).map_err(|e| Error::Internal(e.to_string()))?;
    if let Some(map) = value.as_object_mut() {
        map.remove("packetDigest");
    }
    digest_of(&value)
}

/// The `sha256:<hex>` build identity of one build's bytes, for a binding that
/// can read its own executable or library (spec 159 D-5).
pub fn build_digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(&Sha256::digest(bytes)))
}

fn validate_request(request: &PacketRequest) -> Result<(), Error> {
    let version = parse_semver(&request.consumer_schema_version).ok_or_else(|| {
        Error::Usage(format!(
            "consumerSchemaVersion '{}' is not MAJOR.MINOR.PATCH",
            request.consumer_schema_version
        ))
    })?;
    let ours = parse_semver(CONTEXT_PACKET_SCHEMA_VERSION).expect("own version parses");
    if version.0 != ours.0 || version.1 > ours.1 {
        return Err(Error::Refused(format!(
            "unsupported-schema: consumer schema {} is not served by context-packet schema {}",
            request.consumer_schema_version, CONTEXT_PACKET_SCHEMA_VERSION
        )));
    }
    let closure = &request.closure;
    match (&closure.root, &closure.digest, &closure.document) {
        (Some(root), None, None) if !root.trim().is_empty() => {}
        (None, Some(digest), Some(_)) if !digest.trim().is_empty() => {}
        (None, Some(_), None) => {
            return Err(Error::Usage(
                "closure.digest needs the closure document it names".into(),
            ));
        }
        _ => {
            return Err(Error::Usage(
                "closure names exactly one of root or digest (with its document)".into(),
            ));
        }
    }
    if !(1..=PACKET_MAX_BYTES_LIMIT).contains(&request.max_bytes) {
        return Err(Error::Usage(format!(
            "maxBytes must be from 1 through {PACKET_MAX_BYTES_LIMIT}"
        )));
    }
    if !(1..=PACKET_MAX_ITEMS_LIMIT).contains(&request.max_items) {
        return Err(Error::Usage(format!(
            "maxItems must be from 1 through {PACKET_MAX_ITEMS_LIMIT}"
        )));
    }
    for (name, value) in [
        ("rationale", &request.rationale),
        ("workIdentity", &request.work_identity),
    ] {
        if value
            .as_ref()
            .is_some_and(|v| v.len() > PACKET_OPAQUE_LIMIT)
        {
            return Err(Error::Usage(format!(
                "{name} must be at most {PACKET_OPAQUE_LIMIT} bytes"
            )));
        }
    }
    if request.members.iter().any(|m| !m.selector.required()) {
        return Err(Error::Usage(
            "a packet member's requirement is set by `requirement`; leave the selector's \
             `required` unset"
                .into(),
        ));
    }
    Ok(())
}

/// The request with its members in canonical order, exact duplicates removed,
/// and no continuation: what `requestDigest` hashes.
fn normalize(request: &PacketRequest) -> PacketRequest {
    let mut keyed: BTreeMap<String, PacketMemberRequest> = BTreeMap::new();
    for member in &request.members {
        let key = canonical_json::to_string(member).unwrap_or_default();
        keyed.insert(key, member.clone());
    }
    PacketRequest {
        members: keyed.into_values().collect(),
        continuation: None,
        ..request.clone()
    }
}

fn resolve_packet_closure(
    cfg: &Config,
    repo_root: &Path,
    request: &PacketRequest,
) -> Result<crate::closure::ResolvedClosure, Error> {
    let closure = &request.closure;
    if let Some(root) = &closure.root {
        return crate::closure::closure(
            cfg,
            repo_root,
            &ClosureRequest {
                specs: vec![root.clone()],
                ..ClosureRequest::default()
            },
        );
    }
    let digest = closure.digest.as_deref().unwrap_or_default();
    let document = closure.document.as_ref().expect("validated");
    let closure_request = closure_request_of(document)?;
    let resolved = crate::closure::closure(cfg, repo_root, &closure_request)?;
    let recomputed = serde_json::to_value(&resolved).map_err(|e| Error::Internal(e.to_string()))?;
    if resolved.digest != digest || &recomputed != document {
        return Err(Error::Refused(format!(
            "closure-mismatch: the supplied closure document does not recompute to digest \
             {digest} against this snapshot"
        )));
    }
    Ok(resolved)
}

/// Read back the request a resolved closure document was made from: its
/// members' identities and its rationale.
fn closure_request_of(document: &serde_json::Value) -> Result<ClosureRequest, Error> {
    let bad = |why: &str| Error::Usage(format!("invalid closure document: {why}"));
    let members = document
        .get("members")
        .and_then(|m| m.as_array())
        .ok_or_else(|| bad("members must be an array"))?;
    let text = |m: &serde_json::Value, key: &str| -> Result<String, Error> {
        m.get(key)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| bad(&format!("a member needs a string '{key}'")))
    };
    let mut request = ClosureRequest::default();
    for member in members {
        match member.get("kind").and_then(|k| k.as_str()) {
            Some("spec") => request.specs.push(text(member, "spec")?),
            Some("section") => request.sections.push(SectionRef {
                spec: text(member, "spec")?,
                anchor: text(member, "anchor")?,
            }),
            Some("obligation") => request.obligations.push(format!(
                "{}#{}",
                text(member, "spec")?,
                text(member, "id")?
            )),
            _ => return Err(bad("a member's kind is spec, section or obligation")),
        }
    }
    request.rationale = match document.get("rationale") {
        None => None,
        Some(v) => Some(
            v.as_str()
                .map(str::to_string)
                .ok_or_else(|| bad("rationale must be a string"))?,
        ),
    };
    Ok(request)
}

/// Every requested member, closure-derived and additional, resolved and
/// collapsed by canonical key (spec 159 3.3, 3.4), in canonical order.
fn collect_members(
    cfg: &Config,
    repo_root: &Path,
    closure_members: &[ClosureMember],
    additional: &[PacketMemberRequest],
    request: &PacketRequest,
) -> Result<Vec<Member>, Error> {
    let mut selectors: Vec<(ContentSelector, PacketRequirement, PacketOrigin)> = Vec::new();
    let mut members: BTreeMap<(String, String), Member> = BTreeMap::new();
    // Closure members are the declared context: each is required, and each
    // becomes the most specific selector that keeps its identity. A withdrawn
    // obligation has no current text to select.
    for member in closure_members {
        let selector = match member {
            ClosureMember::Spec { spec, .. } => ContentSelector::Spec {
                spec: spec.clone(),
                projection: None,
                required: true,
            },
            ClosureMember::Section { spec, anchor, .. } => ContentSelector::SpecSection {
                spec: spec.clone(),
                anchor: anchor.clone(),
                projection: None,
                required: true,
            },
            ClosureMember::Obligation {
                spec,
                id,
                withdrawn,
                ..
            } => {
                if *withdrawn {
                    add_member(
                        &mut members,
                        Member {
                            identity: format!("obligation:{spec}#{id}"),
                            projection: ContentProjection::Full,
                            requirement: PacketRequirement::Required,
                            origins: vec![PacketOrigin::Closure],
                            selector: None,
                            outcome: Outcome::Omitted(
                                PacketOmissionReason::Withdrawn,
                                format!("obligation '{spec}#{id}' is withdrawn"),
                            ),
                        },
                    );
                    continue;
                }
                ContentSelector::Obligation {
                    obligation: format!("{spec}#{id}"),
                    projection: None,
                    required: true,
                }
            }
        };
        selectors.push((selector, PacketRequirement::Required, PacketOrigin::Closure));
    }
    for m in additional {
        selectors.push((m.selector.clone(), m.requirement, PacketOrigin::Additional));
    }

    let plain: Vec<ContentSelector> = selectors.iter().map(|(s, _, _)| s.clone()).collect();
    let resolutions = resolve_selectors(
        cfg,
        repo_root,
        &plain,
        ContentProjection::Full,
        request.max_bytes,
    )?;
    for (resolution, (_, requirement, origin)) in resolutions.into_iter().zip(selectors) {
        let Resolution {
            identity,
            projection,
            selector,
            outcome,
        } = resolution;
        let outcome = match outcome {
            Ok(selected) => Outcome::Selected(selected),
            Err((reason, detail)) => Outcome::Omitted(omission_reason(&selector, reason), detail),
        };
        add_member(
            &mut members,
            Member {
                identity,
                projection,
                requirement,
                origins: vec![origin],
                selector: Some(selector),
                outcome,
            },
        );
    }
    Ok(members.into_values().collect())
}

/// Insert `m`, or collapse it into the member already holding its key:
/// `required` wins, and origins are kept in canonical order (spec 159 3.4).
fn add_member(members: &mut BTreeMap<(String, String), Member>, m: Member) {
    let key = (m.identity.clone(), proj_name(Some(m.projection)));
    match members.get_mut(&key) {
        Some(existing) => {
            if m.requirement == PacketRequirement::Required {
                existing.requirement = PacketRequirement::Required;
            }
            for o in m.origins {
                if !existing.origins.contains(&o) {
                    existing.origins.push(o);
                }
            }
            existing.origins.sort();
        }
        None => {
            members.insert(key, m);
        }
    }
}

/// Map a spec 155 omission onto the packet vocabulary (D-9). Content that is
/// absent at a structural selector did not resolve; absent anywhere else it is
/// missing. History is never read, so nothing is called `removed`.
fn omission_reason(
    selector: &ContentSelector,
    reason: ContentOmissionReason,
) -> PacketOmissionReason {
    match reason {
        ContentOmissionReason::MissingContent => match selector {
            ContentSelector::Symbol { .. }
            | ContentSelector::Module { .. }
            | ContentSelector::OwnedUnit { .. } => PacketOmissionReason::Unresolved,
            _ => PacketOmissionReason::Missing,
        },
        ContentOmissionReason::UnsupportedSelector => PacketOmissionReason::UnsupportedSelector,
        ContentOmissionReason::UnsupportedProjection => PacketOmissionReason::UnsupportedProjection,
        ContentOmissionReason::BinaryContent => PacketOmissionReason::NonText,
        ContentOmissionReason::ItemExceedsByteBudget => PacketOmissionReason::OversizedMember,
    }
}

fn continuation_payload(
    snapshot_digest: &str,
    request_digest: &str,
    closure_digest: &str,
    request: &PacketRequest,
) -> ContinuationPayload {
    let major = parse_semver(CONTEXT_PACKET_SCHEMA_VERSION)
        .expect("own version parses")
        .0;
    ContinuationPayload {
        schema_major: major,
        snapshot_digest: snapshot_digest.to_string(),
        request_digest: request_digest.to_string(),
        closure_digest: closure_digest.to_string(),
        max_bytes: request.max_bytes,
        max_items: request.max_items,
        last_identity: String::new(),
        last_projection: ContentProjection::Full,
    }
}

fn encode_continuation(payload: ContinuationPayload) -> Result<String, Error> {
    let checksum = digest_of(&payload)?;
    let token = ContinuationToken { payload, checksum };
    let compact = serde_json::to_value(&token)
        .and_then(|v| serde_json::to_string(&v))
        .map_err(|e| Error::Internal(e.to_string()))?;
    Ok(base64url_encode(compact.as_bytes()))
}

fn decode_continuation(token: &str) -> Result<ContinuationPayload, Error> {
    let malformed = || Error::Usage("malformed continuation".into());
    let bytes = base64url_decode(token).ok_or_else(malformed)?;
    let token: ContinuationToken = serde_json::from_slice(&bytes).map_err(|_| malformed())?;
    if digest_of(&token.payload)? != token.checksum {
        return Err(malformed());
    }
    Ok(token.payload)
}

fn digest_of<T: Serialize>(value: &T) -> Result<String, Error> {
    let text = canonical_json::to_string(value)?;
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    Ok(format!("sha256:{}", hex(&hasher.finalize())))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|h| {
        h.len() == 64
            && h.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    })
}

/// A projection's wire name, the second half of a member key.
fn proj_name(projection: Option<ContentProjection>) -> String {
    projection
        .and_then(|p| serde_json::to_value(p).ok())
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// Bytewise UTF-8 order of identity, then projection name (spec 159 3.10).
fn key_cmp(a: (&String, ContentProjection), b: (&String, ContentProjection)) -> std::cmp::Ordering {
    a.0.as_bytes()
        .cmp(b.0.as_bytes())
        .then_with(|| proj_name(Some(a.1)).cmp(&proj_name(Some(b.1))))
}

const BASE64URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Unpadded base64url (RFC 4648 section 5).
fn base64url_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, b)| acc | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..=chunk.len() {
            out.push(BASE64URL[((n >> (18 - 6 * i)) & 63) as usize] as char);
        }
    }
    out
}

/// Decode unpadded base64url; `None` on any byte outside the alphabet, an
/// impossible length, or non-zero trailing bits (so one payload has exactly
/// one spelling).
fn base64url_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 4 == 1 {
        return None;
    }
    let value = |c: u8| BASE64URL.iter().position(|a| *a == c).map(|p| p as u32);
    let mut out = Vec::with_capacity(text.len() / 4 * 3 + 2);
    for chunk in text.as_bytes().chunks(4) {
        let mut n = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            n |= value(*c)? << (18 - 6 * i);
        }
        let produced = chunk.len() - 1;
        for i in 0..produced {
            out.push((n >> (16 - 8 * i)) as u8);
        }
        let used_bits = 8 * produced;
        let mask = (1u32 << (24 - used_bits)) - 1;
        if n & mask != 0 {
            return None;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64url_round_trips_every_tail_length() {
        for len in 0..10 {
            let bytes: Vec<u8> = (0..len).map(|i| (i * 37 + 11) as u8).collect();
            let encoded = base64url_encode(&bytes);
            assert!(!encoded.contains('='));
            assert_eq!(base64url_decode(&encoded), Some(bytes));
        }
    }

    #[test]
    fn base64url_refuses_a_second_spelling_and_foreign_bytes() {
        assert_eq!(base64url_encode(b"a"), "YQ");
        assert_eq!(base64url_decode("YR"), None, "non-zero trailing bits");
        assert_eq!(base64url_decode("Y"), None, "impossible length");
        assert_eq!(
            base64url_decode("YQ=="),
            None,
            "padding is not the alphabet"
        );
        assert_eq!(base64url_decode("Y+"), None);
    }
}
