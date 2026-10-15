//! Context-packet delta request, delta, fallback and candidate types (spec
//! 164).
//!
//! A delta relates two complete spec 159 page sets of one request: the
//! predecessor a caller holds and the target spec 159 assembles on a named
//! snapshot. It names every member change in a closed vocabulary, carries
//! content only for members the target presents, and reproduces the target
//! byte for byte when applied. Its own schema axis is
//! [`crate::CONTEXT_PACKET_DELTA_SCHEMA_VERSION`], distinct from spec 071's
//! `DELTA_SCHEMA_VERSION`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    ContentProjection, PacketInterface, PacketMember, PacketOmission, PacketOmissionReason,
    PacketProducer, PacketRequest, PacketRequirement, PacketSnapshot,
};

/// Most predecessor pages one delta request carries (spec 164 3.2).
pub const DELTA_MAX_PREDECESSOR_PAGES: usize = 512;
/// Most documentation manifests one delta request carries (spec 164 3.2).
pub const DELTA_MAX_MANIFESTS: usize = 64;
/// Most bytes a canonical delta may hold; a larger one falls back as
/// `delta-too-large` (spec 164 3.10).
pub const DELTA_MAX_BYTES: usize = 4_194_304;

/// One delta request (spec 164 3.2). Unknown members are refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextDeltaRequest {
    /// The spec 159 request, with `continuation` null.
    pub request: PacketRequest,
    /// The complete predecessor page set, as the exact strings the caller
    /// holds, in page order.
    pub predecessor_pages: Vec<String>,
    /// Spec 160 documentation manifests, as exact strings.
    #[serde(default)]
    pub manifests: Vec<String>,
    /// The delta axis version the caller reads.
    pub consumer_schema_version: String,
}

/// Why a delta request fell back to a full packet (spec 164 3.3), in row
/// order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeltaFallbackReason {
    PredecessorUnverifiable,
    PredecessorIncomplete,
    RepositoryChanged,
    SchemaVersionChanged,
    BudgetChanged,
    RequestChanged,
    DeltaTooLarge,
}

/// The six closed change kinds (spec 164 3.4), in table row order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MemberChange {
    InterfaceChanged,
    Changed,
    Removed,
    NewlyUnresolved,
    NewlyOmitted,
    NewlyAvailable,
}

/// What an entry or candidate follows from (spec 164 3.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeBasis {
    /// Compared digests or records.
    Digest,
    /// An `unavailable` interface on either side.
    InterfaceUnknown,
    /// A changed closure digest, for a member omitted on both sides.
    ClosureChanged,
    /// A manifest not bound to the predecessor.
    UnboundManifest,
    /// Every manifest, on any fallback.
    Fallback,
}

impl ChangeBasis {
    /// The wire name, which is also the candidate sort key (spec 164 3.9).
    pub fn name(self) -> &'static str {
        match self {
            ChangeBasis::Digest => "digest",
            ChangeBasis::InterfaceUnknown => "interface-unknown",
            ChangeBasis::ClosureChanged => "closure-changed",
            ChangeBasis::UnboundManifest => "unbound-manifest",
            ChangeBasis::Fallback => "fallback",
        }
    }
}

/// A key's state in one page set (spec 164 3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeltaMemberState {
    Present,
    Omitted,
    Absent,
}

/// Which parts of a `changed` record differ (spec 164 3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangedField {
    Content,
    Span,
    Requirement,
    Origins,
}

/// A spec 159 canonical member key: the spec 155 identity plus projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemberKey {
    pub identity: String,
    pub projection: ContentProjection,
}

/// One member change (spec 164 3.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeltaEntry {
    pub key: MemberKey,
    pub change: MemberChange,
    pub basis: ChangeBasis,
    pub prior_state: DeltaMemberState,
    /// The predecessor item digest, when the key was present.
    pub prior_item_digest: Option<String>,
    /// The predecessor omission reason, when the key was omitted.
    pub prior_reason: Option<PacketOmissionReason>,
    /// The target omission reason, for `newly-unresolved` and
    /// `newly-omitted`.
    pub reason: Option<PacketOmissionReason>,
    /// For `changed`: which of content, span, requirement and origins differ,
    /// each once and in declaration order. Empty when the difference lies
    /// elsewhere in the record (D-19). A repeated field is refused on read, as
    /// the schema's `uniqueItems` refuses it.
    #[serde(deserialize_with = "unique_fields")]
    pub fields: BTreeSet<ChangedField>,
    /// For `interface-changed`: both interfaces, so a spec 110 pin holder can
    /// match either digest.
    pub prior_interface: Option<PacketInterface>,
    pub interface: Option<PacketInterface>,
    /// The complete target member, for `changed`, `interface-changed` and
    /// `newly-available`; null otherwise.
    pub record: Option<PacketMember>,
}

/// Read `fields`, refusing a repeated entry rather than collapsing it.
fn unique_fields<'de, D>(deserializer: D) -> Result<BTreeSet<ChangedField>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let listed = Vec::<ChangedField>::deserialize(deserializer)?;
    let set: BTreeSet<ChangedField> = listed.iter().copied().collect();
    if set.len() != listed.len() {
        return Err(serde::de::Error::custom("`fields` repeats an entry"));
    }
    Ok(set)
}

/// Every change kind's count, zeros included (spec 164 3.6).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeltaCounts {
    pub interface_changed: usize,
    pub changed: usize,
    pub removed: usize,
    pub newly_unresolved: usize,
    pub newly_omitted: usize,
    pub newly_available: usize,
}

/// One target page's layout (spec 164 3.6): its envelope, member order and
/// complete omissions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeltaPageLayout {
    /// Every spec 159 top-level field except `members`, `omissions` and
    /// `closure`, byte-equal to the page's, `packetDigest` included.
    pub envelope: serde_json::Value,
    /// The page's member keys, in order.
    pub members: Vec<MemberKey>,
    /// The page's complete, ordered omission records.
    pub omissions: Vec<PacketOmission>,
}

/// The predecessor a delta names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeltaPredecessor {
    pub snapshot: PacketSnapshot,
    pub page_digests: Vec<String>,
}

/// The target a delta reproduces.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeltaTarget {
    pub snapshot: PacketSnapshot,
    pub page_digests: Vec<String>,
    /// The target closure document, or null when the predecessor's is reused.
    pub closure: Option<serde_json::Value>,
    pub pages: Vec<DeltaPageLayout>,
}

/// What invalidation candidates cover. Closed: packet members only (D-8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CandidateCoverage {
    PacketMembers,
}

/// One manifest input whose declared identity changed (spec 164 3.9). A
/// whole-manifest candidate carries null `inputId`, `key` and `change`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InvalidationCandidate {
    pub manifest_digest: String,
    pub input_id: Option<String>,
    pub requirement: Option<PacketRequirement>,
    pub key: Option<MemberKey>,
    pub change: Option<MemberChange>,
    pub basis: ChangeBasis,
    pub uses: Vec<String>,
}

/// The candidate list and its coverage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InvalidationCandidates {
    pub coverage: CandidateCoverage,
    pub items: Vec<InvalidationCandidate>,
}

/// A delta document (spec 164 3.6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextPacketDelta {
    pub schema_version: String,
    pub producer: PacketProducer,
    pub producer_changed: bool,
    pub request_digest: String,
    pub predecessor: DeltaPredecessor,
    pub target: DeltaTarget,
    pub changes: Vec<DeltaEntry>,
    pub counts: DeltaCounts,
    pub invalidation_candidates: InvalidationCandidates,
    pub delta_digest: String,
}

/// The predecessor a fallback names: only the supplied pages' digests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FallbackPredecessor {
    pub page_digests: Vec<String>,
}

/// The target a fallback names: only its snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FallbackTarget {
    pub snapshot: PacketSnapshot,
}

/// An explicit full-packet fallback (spec 164 3.3): a reason, no content and
/// no changes. The caller runs spec 159 with the same request and snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextPacketFallback {
    pub schema_version: String,
    pub reason: DeltaFallbackReason,
    pub request_digest: String,
    pub predecessor: FallbackPredecessor,
    pub target: FallbackTarget,
    pub invalidation_candidates: InvalidationCandidates,
    pub delta_digest: String,
}

/// What `context delta` answers: a delta or a fallback, discriminated by the
/// document's `outcome` member, so a document's outcome and its shape cannot
/// disagree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum ContextDeltaResult {
    Delta(Box<ContextPacketDelta>),
    Fallback(ContextPacketFallback),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wire name of a closed-vocabulary value, which serializes as a
    /// JSON string.
    fn wire<T: Serialize + std::fmt::Debug>(value: T) -> String {
        match serde_json::to_value(&value) {
            Ok(serde_json::Value::String(name)) => name,
            other => panic!("{value:?} does not serialize as a string: {other:?}"),
        }
    }

    fn schema_enum(pointer: &str) -> Vec<String> {
        let schema: serde_json::Value =
            serde_json::from_str(crate::CONTEXT_PACKET_DELTA_SCHEMA).unwrap();
        schema
            .pointer(pointer)
            .unwrap_or_else(|| panic!("no {pointer}"))
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn changed_fields_are_unique_and_written_in_declaration_order() {
        let entry = |fields: &str| {
            format!(
                r#"{{"key":{{"identity":"file:a","projection":"full"}},"change":"changed","basis":"digest","priorState":"present","priorItemDigest":null,"priorReason":null,"reason":null,"fields":{fields},"priorInterface":null,"interface":null,"record":null}}"#
            )
        };
        let read: DeltaEntry =
            serde_json::from_str(&entry(r#"["origins","content","span"]"#)).unwrap();
        assert_eq!(
            serde_json::to_value(&read).unwrap()["fields"],
            serde_json::json!(["content", "span", "origins"])
        );
        assert!(
            serde_json::from_str::<DeltaEntry>(&entry(r#"["content","content"]"#)).is_err(),
            "a repeated field is refused, as uniqueItems refuses it"
        );
    }

    #[test]
    fn the_outcome_member_decides_the_shape() {
        let fallback = ContextDeltaResult::Fallback(ContextPacketFallback {
            schema_version: "1.0.0".into(),
            reason: DeltaFallbackReason::BudgetChanged,
            request_digest: "sha256:r".into(),
            predecessor: FallbackPredecessor {
                page_digests: vec!["sha256:p".into()],
            },
            target: FallbackTarget {
                snapshot: crate::PacketSnapshot {
                    repository: "r".into(),
                    revision: "1".repeat(40),
                    tree: "2".repeat(40),
                    dirty_state: crate::ContentDirtyState::CleanExport,
                },
            },
            invalidation_candidates: InvalidationCandidates {
                coverage: CandidateCoverage::PacketMembers,
                items: Vec::new(),
            },
            delta_digest: "sha256:d".into(),
        });
        let mut value = serde_json::to_value(&fallback).unwrap();
        assert_eq!(value["outcome"], "fallback");
        assert_eq!(
            serde_json::from_value::<ContextDeltaResult>(value.clone()).unwrap(),
            fallback
        );
        value["outcome"] = "delta".into();
        assert!(
            serde_json::from_value::<ContextDeltaResult>(value).is_err(),
            "a fallback's shape under outcome delta is not a delta"
        );
    }

    #[test]
    fn a_request_refuses_unknown_members() {
        let text = r#"{"request":{"closure":{"root":"001"},"consumerSchemaVersion":"1.1.0"},"predecessorPages":["{}"],"consumerSchemaVersion":"1.0.0"}"#;
        let request: ContextDeltaRequest = serde_json::from_str(text).unwrap();
        assert!(request.manifests.is_empty());
        let extra = text.replacen('{', r#"{"extra":1,"#, 1);
        assert!(serde_json::from_str::<ContextDeltaRequest>(&extra).is_err());
    }

    #[test]
    fn the_schema_names_every_closed_vocabulary_the_types_serialize() {
        let changes = [
            MemberChange::InterfaceChanged,
            MemberChange::Changed,
            MemberChange::Removed,
            MemberChange::NewlyUnresolved,
            MemberChange::NewlyOmitted,
            MemberChange::NewlyAvailable,
        ];
        assert_eq!(
            schema_enum("/$defs/entry/properties/change/enum"),
            changes.map(wire).to_vec()
        );
        let bases = [
            ChangeBasis::Digest,
            ChangeBasis::InterfaceUnknown,
            ChangeBasis::ClosureChanged,
            ChangeBasis::UnboundManifest,
            ChangeBasis::Fallback,
        ];
        assert_eq!(
            schema_enum("/$defs/entry/properties/basis/enum"),
            bases.map(wire).to_vec()
        );
        for basis in bases {
            assert_eq!(basis.name(), wire(basis));
        }
        let reasons = [
            DeltaFallbackReason::PredecessorUnverifiable,
            DeltaFallbackReason::PredecessorIncomplete,
            DeltaFallbackReason::RepositoryChanged,
            DeltaFallbackReason::SchemaVersionChanged,
            DeltaFallbackReason::BudgetChanged,
            DeltaFallbackReason::RequestChanged,
            DeltaFallbackReason::DeltaTooLarge,
        ];
        assert_eq!(
            schema_enum("/$defs/fallback/properties/reason/enum"),
            reasons.map(wire).to_vec()
        );
        let fields = [
            ChangedField::Content,
            ChangedField::Span,
            ChangedField::Requirement,
            ChangedField::Origins,
        ];
        assert_eq!(
            schema_enum("/$defs/entry/properties/fields/items/enum"),
            fields.map(wire).to_vec()
        );
    }
}
