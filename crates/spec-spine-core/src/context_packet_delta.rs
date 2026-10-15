//! Context-packet deltas (spec 164).
//!
//! A delta relates a caller's complete predecessor page set to the page set
//! spec 159 assembles for the same request on a target snapshot. Every member
//! difference is one of six closed change kinds; records the target presents
//! travel in full, and per-page layouts reproduce the target byte for byte.
//! Reuse is decided by record equality, never by trusting the predecessor
//! (D-2), so applying a delta to the bytes it was computed from yields the
//! true target, and applying it to any other bytes refuses.
//!
//! Pure over its arguments and the named export: no Git, no clock, no
//! environment, no session state. The CLI binds the snapshot.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};
use spec_spine_types::{
    CONTEXT_PACKET_DELTA_SCHEMA_VERSION, CONTEXT_PACKET_SCHEMA_VERSION, CandidateCoverage,
    ChangeBasis, ChangedField, Config, ContentProjection, ContentSnapshot, ContextDeltaRequest,
    ContextDeltaResult, ContextPacket, ContextPacketDelta, ContextPacketFallback, DELTA_MAX_BYTES,
    DELTA_MAX_MANIFESTS, DELTA_MAX_PREDECESSOR_PAGES, DeltaCounts, DeltaEntry, DeltaFallbackReason,
    DeltaMemberState, DeltaPageLayout, DeltaPredecessor, DeltaTarget, DocumentationManifest, Error,
    FallbackPredecessor, FallbackTarget, InvalidationCandidate, InvalidationCandidates,
    MemberChange, MemberKey, PacketInterfaceKind, PacketMember, PacketOmission,
    PacketOmissionReason, PacketOrigin, PacketRequirement, PacketSnapshot, parse_semver,
};

use crate::context_packet::{context_packet_pages, normalized_request_digest};
use crate::documentation_manifest::{check_structure, parse_documentation_manifest};
use crate::{Versioning, canonical_json, context_packet_document, packet_digest, read_document};

/// The page fields a layout's envelope leaves out (spec 164 3.6).
const LAYOUT_EXCLUDED: [&str; 3] = ["members", "omissions", "closure"];

/// A key in canonical order: identity, then projection name, bytewise (spec
/// 159 3.10).
type Key = (String, String);

fn key_of(identity: &str, projection: ContentProjection) -> Key {
    (identity.to_string(), projection_name(projection))
}

fn member_key(key: &Key, projection: ContentProjection) -> MemberKey {
    MemberKey {
        identity: key.0.clone(),
        projection,
    }
}

/// One page set's members and omissions by key.
struct PageSet {
    present: BTreeMap<Key, PacketMember>,
    omitted: BTreeMap<Key, PacketOmission>,
}

impl PageSet {
    fn of(pages: &[ContextPacket]) -> Self {
        let mut present = BTreeMap::new();
        let mut omitted = BTreeMap::new();
        for page in pages {
            for m in &page.members {
                present.insert(key_of(&m.identity, m.projection), m.clone());
            }
            for o in &page.omissions {
                omitted.insert(key_of(&o.identity, o.projection), o.clone());
            }
        }
        PageSet { present, omitted }
    }

    fn projection(&self, key: &Key) -> Option<ContentProjection> {
        self.present
            .get(key)
            .map(|m| m.projection)
            .or_else(|| self.omitted.get(key).map(|o| o.projection))
    }
}

/// Compute the delta from `request.predecessor_pages` to the page set spec 159
/// assembles for `request.request` on `snapshot` (spec 164 3.2 to 3.10).
///
/// Returns a delta or an explicit fallback. A malformed request, an
/// unparseable page or an invalid manifest is [`Error::Usage`]; an
/// unsupported consumer schema is [`Error::Refused`]; the target assembly's
/// own refusals pass through; a delta that fails its own reproduction check
/// is [`Error::Internal`] and is never returned.
pub fn context_packet_delta(
    cfg: &Config,
    repo_root: &Path,
    request: &ContextDeltaRequest,
    snapshot: &ContentSnapshot,
    producer_build: Option<&str>,
) -> Result<ContextDeltaResult, Error> {
    validate_delta_request(request)?;
    let manifests = parse_manifests(&request.manifests)?;
    let target_snapshot = PacketSnapshot {
        repository: snapshot.repository.clone(),
        revision: snapshot.revision.clone(),
        tree: snapshot.tree.clone(),
        dirty_state: snapshot.dirty_state,
    };
    let request_digest = normalized_request_digest(&request.request)?;

    // Row 1: every page parses under a context-packet MAJOR this build reads.
    let values = parse_page_values(&request.predecessor_pages)?;
    let page_digests = values
        .iter()
        .map(recomputed_digest)
        .collect::<Result<Vec<_>, _>>()?;
    let fallback = |reason| {
        fallback_result(
            reason,
            &request_digest,
            &page_digests,
            &target_snapshot,
            &manifests,
        )
    };

    // Rows 2 to 7, in order; the first that holds decides.
    if values
        .iter()
        .zip(&page_digests)
        .any(|(v, d)| v.get("packetDigest").and_then(|p| p.as_str()) != Some(d.as_str()))
    {
        return fallback(DeltaFallbackReason::PredecessorUnverifiable);
    }
    if !one_complete_set(&values) {
        return fallback(DeltaFallbackReason::PredecessorIncomplete);
    }
    let first = &values[0];
    if first
        .pointer("/snapshot/repository")
        .and_then(|r| r.as_str())
        != Some(target_snapshot.repository.as_str())
    {
        return fallback(DeltaFallbackReason::RepositoryChanged);
    }
    if first.get("schemaVersion").and_then(|v| v.as_str()) != Some(CONTEXT_PACKET_SCHEMA_VERSION) {
        return fallback(DeltaFallbackReason::SchemaVersionChanged);
    }
    let budget = |name: &str| {
        first
            .pointer(&format!("/request/{name}"))
            .and_then(|v| v.as_u64())
    };
    if budget("maxBytes") != Some(request.request.max_bytes as u64)
        || budget("maxItems") != Some(request.request.max_items as u64)
    {
        return fallback(DeltaFallbackReason::BudgetChanged);
    }
    if first.get("requestDigest").and_then(|v| v.as_str()) != Some(request_digest.as_str()) {
        return fallback(DeltaFallbackReason::RequestChanged);
    }

    // The predecessor is now on this build's exact schema, so it must read
    // back as typed pages.
    let predecessor = values
        .into_iter()
        .enumerate()
        .map(|(i, v)| {
            serde_json::from_value::<ContextPacket>(v).map_err(|e| {
                Error::Usage(format!(
                    "predecessor page {} is not a context packet: {e}",
                    i + 1
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let target = context_packet_pages(cfg, repo_root, &request.request, snapshot, producer_build)?;
    let delta = build_delta(&predecessor, &page_digests, &target, &manifests)?;

    let document =
        context_packet_delta_document(&ContextDeltaResult::Delta(Box::new(delta.clone())))?;
    if document.len() > DELTA_MAX_BYTES {
        return fallback(DeltaFallbackReason::DeltaTooLarge);
    }

    // The reproduction law (3.7), checked on this build's own output before
    // anything is emitted.
    let expected = target
        .iter()
        .map(context_packet_document)
        .collect::<Result<Vec<_>, _>>()?;
    match context_packet_apply(&request.predecessor_pages, &delta) {
        Ok(pages) if pages == expected => Ok(ContextDeltaResult::Delta(Box::new(delta))),
        Ok(_) => Err(Error::Internal(
            "the delta does not reproduce its target".into(),
        )),
        Err(e) => Err(Error::Internal(format!(
            "the delta does not reproduce its target: {e}"
        ))),
    }
}

/// Whether the delta's target omits a required member: spec 159's
/// `incomplete`, which the CLI reports as a finding (spec 164 3.12).
pub fn delta_target_incomplete(delta: &ContextPacketDelta) -> bool {
    delta
        .target
        .pages
        .iter()
        .any(|p| p.envelope.get("completeness").and_then(|c| c.as_str()) == Some("incomplete"))
}

/// Apply `delta` to the predecessor pages it names (spec 164 3.7) and return
/// every reproduced target page as its canonical document, in order. All
/// pages or none: a predecessor whose digests are not the ones the delta
/// names, or a page that does not reproduce its recorded digest, is
/// [`Error::Refused`]; a malformed delta or page is [`Error::Usage`].
pub fn context_packet_apply(
    predecessor_pages: &[String],
    delta: &ContextPacketDelta,
) -> Result<Vec<String>, Error> {
    check_delta_schema(&delta.schema_version)?;
    if predecessor_pages.is_empty() || predecessor_pages.len() > DELTA_MAX_PREDECESSOR_PAGES {
        return Err(Error::Usage(format!(
            "apply takes 1 through {DELTA_MAX_PREDECESSOR_PAGES} predecessor pages"
        )));
    }
    let values = parse_page_values(predecessor_pages)?;
    // Step 1: the predecessor must be the bytes the delta was computed from.
    let digests = values
        .iter()
        .map(recomputed_digest)
        .collect::<Result<Vec<_>, _>>()?;
    if digests != delta.predecessor.page_digests {
        return Err(Error::Refused(
            "predecessor-mismatch: the predecessor pages are not the ones the delta names".into(),
        ));
    }
    let predecessor = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            serde_json::from_value::<ContextPacket>(v.clone()).map_err(|e| {
                Error::Usage(format!(
                    "predecessor page {} is not a context packet: {e}",
                    i + 1
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let malformed = |why: String| Error::Usage(format!("malformed delta: {why}"));
    if delta.target.pages.is_empty() || delta.target.pages.len() != delta.target.page_digests.len()
    {
        return Err(malformed(
            "the target names a different number of layouts and page digests".into(),
        ));
    }

    // Step 2: the member map, restamped, with every carried record in place.
    let mut map: BTreeMap<Key, PacketMember> = BTreeMap::new();
    for page in &predecessor {
        for m in &page.members {
            map.insert(
                key_of(&m.identity, m.projection),
                restamp(m, &delta.target.snapshot),
            );
        }
    }
    let mut carried: BTreeSet<Key> = BTreeSet::new();
    let mut entry_keys: BTreeSet<Key> = BTreeSet::new();
    for entry in &delta.changes {
        let key = key_of(&entry.key.identity, entry.key.projection);
        if !entry_keys.insert(key.clone()) {
            return Err(malformed(format!(
                "change entry '{}' is repeated",
                entry.key.identity
            )));
        }
        let carries = matches!(
            entry.change,
            MemberChange::Changed | MemberChange::InterfaceChanged | MemberChange::NewlyAvailable
        );
        match (&entry.record, carries) {
            (Some(record), true) => {
                if key_of(&record.identity, record.projection) != key {
                    return Err(malformed(format!(
                        "change entry '{}' carries another member's record",
                        entry.key.identity
                    )));
                }
                map.insert(key.clone(), record.clone());
                carried.insert(key);
            }
            (None, false) => {}
            _ => {
                return Err(malformed(format!(
                    "change entry '{}' carries a record its change kind does not",
                    entry.key.identity
                )));
            }
        }
    }

    // Step 3: assemble each page from its layout, the map and the closure.
    let closure = match &delta.target.closure {
        Some(c) => c.clone(),
        None => serde_json::to_value(&predecessor[0].closure)
            .map_err(|e| Error::Internal(e.to_string()))?,
    };
    let mut used: BTreeSet<Key> = BTreeSet::new();
    let mut pages = Vec::with_capacity(delta.target.pages.len());
    for (i, layout) in delta.target.pages.iter().enumerate() {
        let n = i + 1;
        let Some(envelope) = layout.envelope.as_object() else {
            return Err(malformed(format!("layout {n}'s envelope is not an object")));
        };
        if LAYOUT_EXCLUDED.iter().any(|f| envelope.contains_key(*f)) {
            return Err(malformed(format!(
                "layout {n}'s envelope carries members, omissions or closure"
            )));
        }
        let mut members = Vec::with_capacity(layout.members.len());
        for k in &layout.members {
            let key = key_of(&k.identity, k.projection);
            if !used.insert(key.clone()) {
                return Err(malformed(format!(
                    "member '{}' is laid out twice",
                    k.identity
                )));
            }
            let record = map.get(&key).ok_or_else(|| {
                malformed(format!(
                    "layout {n} names member '{}', which neither side carries",
                    k.identity
                ))
            })?;
            members.push(record.clone());
        }
        let mut page = envelope.clone();
        page.insert(
            "members".into(),
            serde_json::to_value(&members).map_err(|e| Error::Internal(e.to_string()))?,
        );
        page.insert(
            "omissions".into(),
            serde_json::to_value(&layout.omissions).map_err(|e| Error::Internal(e.to_string()))?,
        );
        page.insert("closure".into(), closure.clone());
        let packet: ContextPacket = serde_json::from_value(serde_json::Value::Object(page))
            .map_err(|e| malformed(format!("layout {n} does not make a context packet: {e}")))?;
        pages.push(packet);
    }
    if let Some(unused) = carried.iter().find(|k| !used.contains(*k)) {
        return Err(malformed(format!(
            "change entry '{}' carries a record no layout uses",
            unused.0
        )));
    }

    // Steps 4 and 5: every page reproduces its recorded digest, or nothing is
    // written.
    let mut out = Vec::with_capacity(pages.len());
    for (i, page) in pages.iter().enumerate() {
        let digest = packet_digest(page)?;
        if digest != page.packet_digest || digest != delta.target.page_digests[i] {
            return Err(Error::Refused(format!(
                "page-not-reproduced: page {} does not reproduce its recorded packetDigest",
                i + 1
            )));
        }
        out.push(context_packet_document(page)?);
    }
    Ok(out)
}

/// Write a delta or fallback as its canonical document: sorted keys, two-space
/// indent, LF and one trailing newline.
pub fn context_packet_delta_document(result: &ContextDeltaResult) -> Result<String, Error> {
    read_document(result, Versioning::Preexisting("schemaVersion"))
}

/// Parse a delta document for `apply`. A fallback, or anything else that is
/// not a delta, is a usage error.
pub fn parse_delta(text: &str) -> Result<ContextPacketDelta, Error> {
    match serde_json::from_str::<ContextDeltaResult>(text) {
        Ok(ContextDeltaResult::Delta(delta)) => Ok(*delta),
        Ok(ContextDeltaResult::Fallback(_)) => {
            Err(Error::Usage("a fallback cannot be applied".into()))
        }
        Err(e) => Err(Error::Usage(format!("invalid context-packet delta: {e}"))),
    }
}

fn validate_delta_request(request: &ContextDeltaRequest) -> Result<(), Error> {
    check_delta_schema(&request.consumer_schema_version)?;
    if request.request.continuation.is_some() {
        return Err(Error::Usage(
            "a delta request's packet request carries no continuation: a delta is atomic".into(),
        ));
    }
    if !(1..=DELTA_MAX_PREDECESSOR_PAGES).contains(&request.predecessor_pages.len()) {
        return Err(Error::Usage(format!(
            "predecessorPages must hold 1 through {DELTA_MAX_PREDECESSOR_PAGES} pages"
        )));
    }
    if request.manifests.len() > DELTA_MAX_MANIFESTS {
        return Err(Error::Usage(format!(
            "manifests must hold at most {DELTA_MAX_MANIFESTS} documents"
        )));
    }
    Ok(())
}

/// A version on the delta axis this build serves: same MAJOR, MINOR not above
/// its own.
fn check_delta_schema(version: &str) -> Result<(), Error> {
    let v = parse_semver(version).ok_or_else(|| {
        Error::Usage(format!(
            "context-packet-delta schema '{version}' is not MAJOR.MINOR.PATCH"
        ))
    })?;
    let ours = parse_semver(CONTEXT_PACKET_DELTA_SCHEMA_VERSION).expect("own version parses");
    if v.0 != ours.0 || v.1 > ours.1 {
        return Err(Error::Refused(format!(
            "unsupported-schema: context-packet-delta schema {version} is not served by \
             {CONTEXT_PACKET_DELTA_SCHEMA_VERSION}"
        )));
    }
    Ok(())
}

fn parse_manifests(texts: &[String]) -> Result<Vec<DocumentationManifest>, Error> {
    texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            parse_documentation_manifest(text)
                .and_then(|m| check_structure(&m))
                .map_err(|e| Error::Usage(format!("manifest {}: {e}", i + 1)))
        })
        .collect()
}

/// Row 1 (spec 164 3.3): each page is a JSON object under a context-packet
/// MAJOR this build reads, with a string `packetDigest`. Anything else is a
/// usage error naming the page's position.
fn parse_page_values(pages: &[String]) -> Result<Vec<serde_json::Value>, Error> {
    let ours = parse_semver(CONTEXT_PACKET_SCHEMA_VERSION)
        .expect("own version parses")
        .0;
    pages
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let bad = |why: &str| {
                Error::Usage(format!(
                    "predecessor page {} does not parse as a context packet: {why}",
                    i + 1
                ))
            };
            let value: serde_json::Value =
                serde_json::from_str(text).map_err(|_| bad("not JSON"))?;
            let major = value
                .get("schemaVersion")
                .and_then(|v| v.as_str())
                .and_then(parse_semver)
                .map(|v| v.0);
            if major != Some(ours) {
                return Err(bad("not a context-packet schema MAJOR this build reads"));
            }
            if !value.get("packetDigest").is_some_and(|d| d.is_string()) {
                return Err(bad("no packetDigest"));
            }
            Ok(value)
        })
        .collect()
}

/// `sha256:` over the canonical page with `packetDigest` absent: spec 159's
/// packet digest, recomputed over the bytes the caller supplied rather than
/// a re-rendering of them.
fn recomputed_digest(value: &serde_json::Value) -> Result<String, Error> {
    let mut value = value.clone();
    if let Some(map) = value.as_object_mut() {
        map.remove("packetDigest");
    }
    digest_of(&value)
}

/// Row 3 (spec 164 3.1, D-12): one complete page set. Every page shares one
/// `requestDigest`, snapshot and `closureDigest`; every page but the last
/// carries a continuation and the last carries none; member keys strictly
/// increase within and across pages; and every page carries the same
/// omissions, none of them also a member.
fn one_complete_set(values: &[serde_json::Value]) -> bool {
    let first = &values[0];
    let mut last: Option<Key> = None;
    let mut members: BTreeSet<Key> = BTreeSet::new();
    for (i, page) in values.iter().enumerate() {
        for field in ["requestDigest", "snapshot", "closureDigest", "omissions"] {
            if page.get(field).is_none() || page.get(field) != first.get(field) {
                return false;
            }
        }
        let terminal = i + 1 == values.len();
        let has_continuation = page.get("continuation").is_some_and(|c| !c.is_null());
        if has_continuation == terminal {
            return false;
        }
        let Some(list) = page.get("members").and_then(|m| m.as_array()) else {
            return false;
        };
        for member in list {
            let Some(key) = value_key(member) else {
                return false;
            };
            if last.as_ref().is_some_and(|prev| prev >= &key) {
                return false;
            }
            last = Some(key.clone());
            members.insert(key);
        }
    }
    let Some(omissions) = first.get("omissions").and_then(|o| o.as_array()) else {
        return false;
    };
    let mut omitted = BTreeSet::new();
    for o in omissions {
        match value_key(o) {
            Some(key) if !members.contains(&key) && omitted.insert(key.clone()) => {}
            _ => return false,
        }
    }
    true
}

fn value_key(v: &serde_json::Value) -> Option<Key> {
    Some((
        v.get("identity")?.as_str()?.to_string(),
        v.get("projection")?.as_str()?.to_string(),
    ))
}

fn build_delta(
    predecessor: &[ContextPacket],
    predecessor_digests: &[String],
    target: &[ContextPacket],
    manifests: &[DocumentationManifest],
) -> Result<ContextPacketDelta, Error> {
    let before = PageSet::of(predecessor);
    let after = PageSet::of(target);
    let first = &target[0];

    let mut keys: BTreeSet<Key> = BTreeSet::new();
    keys.extend(before.present.keys().cloned());
    keys.extend(before.omitted.keys().cloned());
    keys.extend(after.present.keys().cloned());
    keys.extend(after.omitted.keys().cloned());

    let mut changes = Vec::new();
    for key in &keys {
        if let Some(entry) = classify(key, &before, &after, &first.snapshot) {
            changes.push(entry);
        }
    }

    let mut counts = DeltaCounts::default();
    for e in &changes {
        *match e.change {
            MemberChange::InterfaceChanged => &mut counts.interface_changed,
            MemberChange::Changed => &mut counts.changed,
            MemberChange::Removed => &mut counts.removed,
            MemberChange::NewlyUnresolved => &mut counts.newly_unresolved,
            MemberChange::NewlyOmitted => &mut counts.newly_omitted,
            MemberChange::NewlyAvailable => &mut counts.newly_available,
        } += 1;
    }

    let mut pages = Vec::with_capacity(target.len());
    let mut target_digests = Vec::with_capacity(target.len());
    for page in target {
        let mut envelope =
            serde_json::to_value(page).map_err(|e| Error::Internal(e.to_string()))?;
        if let Some(map) = envelope.as_object_mut() {
            for field in LAYOUT_EXCLUDED {
                map.remove(field);
            }
        }
        pages.push(DeltaPageLayout {
            envelope,
            members: page
                .members
                .iter()
                .map(|m| MemberKey {
                    identity: m.identity.clone(),
                    projection: m.projection,
                })
                .collect(),
            omissions: page.omissions.clone(),
        });
        target_digests.push(page.packet_digest.clone());
    }

    // The predecessor's closure document is reused only when it is the
    // target's exactly: an equal digest beside a different document is a
    // forged predecessor, and record equality, not trust, decides (D-2).
    let closure = if predecessor[0].closure == first.closure {
        None
    } else {
        Some(first.closure.clone())
    };
    let closure_changed = predecessor[0].closure_digest != first.closure_digest;

    let candidates = candidates(
        manifests,
        &predecessor[0].request_digest,
        predecessor_digests,
        &changes,
        closure_changed.then_some((&before, &after)),
    )?;

    let mut delta = ContextPacketDelta {
        schema_version: CONTEXT_PACKET_DELTA_SCHEMA_VERSION.to_string(),
        producer: first.producer.clone(),
        producer_changed: predecessor[0].producer != first.producer,
        request_digest: first.request_digest.clone(),
        predecessor: DeltaPredecessor {
            snapshot: predecessor[0].snapshot.clone(),
            page_digests: predecessor_digests.to_vec(),
        },
        target: DeltaTarget {
            snapshot: first.snapshot.clone(),
            page_digests: target_digests,
            closure,
            pages,
        },
        changes,
        counts,
        invalidation_candidates: InvalidationCandidates {
            coverage: CandidateCoverage::PacketMembers,
            items: candidates,
        },
        delta_digest: String::new(),
    };
    delta.delta_digest = delta_digest(&ContextDeltaResult::Delta(Box::new(delta.clone())))?;
    Ok(delta)
}

/// One key's entry under the 3.4 transition table, read top to bottom; `None`
/// when the key's state and record are unchanged.
fn classify(
    key: &Key,
    before: &PageSet,
    after: &PageSet,
    target_snapshot: &PacketSnapshot,
) -> Option<DeltaEntry> {
    let projection = after
        .projection(key)
        .or_else(|| before.projection(key))
        .expect("a key comes from one of the sets");
    let prior_state = if before.present.contains_key(key) {
        DeltaMemberState::Present
    } else if before.omitted.contains_key(key) {
        DeltaMemberState::Omitted
    } else {
        DeltaMemberState::Absent
    };
    let entry = |change, basis| DeltaEntry {
        key: member_key(key, projection),
        change,
        basis,
        prior_state,
        prior_item_digest: before.present.get(key).map(|m| m.item.digest.clone()),
        prior_reason: before.omitted.get(key).map(|o| o.reason),
        reason: None,
        fields: BTreeSet::new(),
        prior_interface: None,
        interface: None,
        record: None,
    };

    if let Some(target) = after.present.get(key) {
        if let Some(prior) = before.present.get(key) {
            let prior = restamp(prior, target_snapshot);
            let unknown = |i: &Option<spec_spine_types::PacketInterface>| {
                i.as_ref()
                    .is_none_or(|i| i.kind == PacketInterfaceKind::Unavailable)
            };
            let whole_signature = prior.item.digest == target.item.digest
                && matches!(
                    target.projection,
                    ContentProjection::Full | ContentProjection::Signature
                );
            let interface_unknown =
                (unknown(&prior.interface) || unknown(&target.interface)) && !whole_signature;
            if interface_unknown || prior.interface != target.interface {
                let basis = if interface_unknown {
                    ChangeBasis::InterfaceUnknown
                } else {
                    ChangeBasis::Digest
                };
                return Some(DeltaEntry {
                    prior_interface: prior.interface.clone(),
                    interface: target.interface.clone(),
                    record: Some(target.clone()),
                    ..entry(MemberChange::InterfaceChanged, basis)
                });
            }
            if &prior != target {
                return Some(DeltaEntry {
                    fields: changed_fields(&prior, target),
                    record: Some(target.clone()),
                    ..entry(MemberChange::Changed, ChangeBasis::Digest)
                });
            }
            return None;
        }
        return Some(DeltaEntry {
            record: Some(target.clone()),
            ..entry(MemberChange::NewlyAvailable, ChangeBasis::Digest)
        });
    }
    if let Some(target) = after.omitted.get(key) {
        if before.omitted.get(key) == Some(target) {
            return None;
        }
        let change = if is_resolution_reason(target.reason) {
            MemberChange::NewlyUnresolved
        } else {
            MemberChange::NewlyOmitted
        };
        return Some(DeltaEntry {
            reason: Some(target.reason),
            ..entry(change, ChangeBasis::Digest)
        });
    }
    (prior_state != DeltaMemberState::Absent)
        .then(|| entry(MemberChange::Removed, ChangeBasis::Digest))
}

/// Spec 164 3.4: the reasons a selector failed to resolve. Every other reason
/// is a selection reason.
fn is_resolution_reason(reason: PacketOmissionReason) -> bool {
    matches!(
        reason,
        PacketOmissionReason::Missing
            | PacketOmissionReason::Removed
            | PacketOmissionReason::Withdrawn
            | PacketOmissionReason::Unresolved
            | PacketOmissionReason::Ambiguous
    )
}

fn changed_fields(prior: &PacketMember, target: &PacketMember) -> BTreeSet<ChangedField> {
    let mut fields = BTreeSet::new();
    if prior.item.content != target.item.content {
        fields.insert(ChangedField::Content);
    }
    if prior.item.span != target.item.span {
        fields.insert(ChangedField::Span);
    }
    if prior.requirement != target.requirement {
        fields.insert(ChangedField::Requirement);
    }
    if prior.origins != target.origins {
        fields.insert(ChangedField::Origins);
    }
    fields
}

/// A predecessor member with the four snapshot fields its item repeats
/// replaced by the target's (spec 164 3.4).
fn restamp(member: &PacketMember, snapshot: &PacketSnapshot) -> PacketMember {
    let mut m = member.clone();
    m.item.repository = snapshot.repository.clone();
    m.item.revision = snapshot.revision.clone();
    m.item.tree = snapshot.tree.clone();
    m.item.dirty_state = snapshot.dirty_state;
    m
}

/// Spec 164 3.8, 3.9: the manifest inputs whose declared identities changed.
/// `closure` is the two sets when the closure digest changed.
fn candidates(
    manifests: &[DocumentationManifest],
    predecessor_request_digest: &str,
    predecessor_digests: &[String],
    changes: &[DeltaEntry],
    closure: Option<(&PageSet, &PageSet)>,
) -> Result<Vec<InvalidationCandidate>, Error> {
    let by_key: BTreeMap<Key, &DeltaEntry> = changes
        .iter()
        .map(|e| (key_of(&e.key.identity, e.key.projection), e))
        .collect();
    let mut out = Vec::new();
    for m in manifests {
        let bound = m.packet.request_digest == predecessor_request_digest
            && m.packet.page_digests == predecessor_digests;
        if !bound {
            out.push(whole_manifest(m, ChangeBasis::UnboundManifest));
            continue;
        }
        let inputs = m
            .inputs
            .iter()
            .map(|i| {
                (
                    &i.input_id,
                    &i.identity,
                    i.projection,
                    i.requirement,
                    &i.uses,
                )
            })
            .chain(m.omissions.iter().map(|o| {
                (
                    &o.input_id,
                    &o.identity,
                    o.projection,
                    PacketRequirement::Optional,
                    &o.uses,
                )
            }));
        for (input_id, identity, projection, requirement, uses) in inputs {
            let key = key_of(identity, projection);
            let candidate = |change, basis| InvalidationCandidate {
                manifest_digest: m.manifest_digest.clone(),
                input_id: Some(input_id.clone()),
                requirement: Some(requirement),
                key: Some(MemberKey {
                    identity: identity.clone(),
                    projection,
                }),
                change,
                basis,
                uses: uses.clone(),
            };
            if let Some(entry) = by_key.get(&key) {
                out.push(candidate(Some(entry.change), entry.basis));
            }
            if let Some((before, after)) = closure {
                let closure_omitted = |set: &PageSet| {
                    set.omitted
                        .get(&key)
                        .is_some_and(|o| o.origins.contains(&PacketOrigin::Closure))
                };
                if closure_omitted(before) && closure_omitted(after) {
                    out.push(candidate(None, ChangeBasis::ClosureChanged));
                }
            }
        }
    }
    Ok(sort_candidates(out))
}

fn whole_manifest(m: &DocumentationManifest, basis: ChangeBasis) -> InvalidationCandidate {
    InvalidationCandidate {
        manifest_digest: m.manifest_digest.clone(),
        input_id: None,
        requirement: None,
        key: None,
        change: None,
        basis,
        uses: Vec::new(),
    }
}

/// Sort by `manifestDigest`, then `inputId` with null first, then basis, and
/// drop exact duplicates (spec 164 3.9). The order never depends on manifest
/// order.
fn sort_candidates(mut items: Vec<InvalidationCandidate>) -> Vec<InvalidationCandidate> {
    items.sort_by(|a, b| {
        a.manifest_digest
            .as_bytes()
            .cmp(b.manifest_digest.as_bytes())
            .then_with(|| {
                a.input_id
                    .as_deref()
                    .map(str::as_bytes)
                    .cmp(&b.input_id.as_deref().map(str::as_bytes))
            })
            .then_with(|| a.basis.name().cmp(b.basis.name()))
            .then_with(|| {
                canonical_json::to_string(a)
                    .unwrap_or_default()
                    .cmp(&canonical_json::to_string(b).unwrap_or_default())
            })
    });
    items.dedup();
    items
}

fn fallback_result(
    reason: DeltaFallbackReason,
    request_digest: &str,
    page_digests: &[String],
    target_snapshot: &PacketSnapshot,
    manifests: &[DocumentationManifest],
) -> Result<ContextDeltaResult, Error> {
    let items = sort_candidates(
        manifests
            .iter()
            .map(|m| whole_manifest(m, ChangeBasis::Fallback))
            .collect(),
    );
    let mut fallback = ContextPacketFallback {
        schema_version: CONTEXT_PACKET_DELTA_SCHEMA_VERSION.to_string(),
        reason,
        request_digest: request_digest.to_string(),
        predecessor: FallbackPredecessor {
            page_digests: page_digests.to_vec(),
        },
        target: FallbackTarget {
            snapshot: target_snapshot.clone(),
        },
        invalidation_candidates: InvalidationCandidates {
            coverage: CandidateCoverage::PacketMembers,
            items,
        },
        delta_digest: String::new(),
    };
    fallback.delta_digest = delta_digest(&ContextDeltaResult::Fallback(fallback.clone()))?;
    Ok(ContextDeltaResult::Fallback(fallback))
}

/// `sha256:` over the canonical document with `deltaDigest` absent (spec 164
/// 3.6).
fn delta_digest(result: &ContextDeltaResult) -> Result<String, Error> {
    let mut value = serde_json::to_value(result).map_err(|e| Error::Internal(e.to_string()))?;
    if let Some(map) = value.as_object_mut() {
        map.remove("deltaDigest");
    }
    digest_of(&value)
}

fn digest_of<T: Serialize>(value: &T) -> Result<String, Error> {
    let text = canonical_json::to_string(value)?;
    Ok(format!(
        "sha256:{}",
        Sha256::digest(text.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}

fn projection_name(projection: ContentProjection) -> String {
    serde_json::to_value(projection)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}
