// Spec: specs/169-declared-obligation-traceability/spec.md
//! Declared obligation traceability (spec 169): an authored relation from one
//! qualified obligation (spec 106) to one typed target.
//!
//! A relation records only what an author declared (§3.1). Its resolution
//! state (§3.6) says whether the target binds under a closed structural
//! contract, never that anything ran, passed, or was accepted (§3.8).
//!
//! The authored shape and the compiled registry shape are one type, as for
//! [`crate::Impact`]: the compiler normalizes every spec reference in place
//! (spec 015) before the record is built.

use serde::{Deserialize, Serialize};

use crate::{ContentSelector, Unit};

/// The six relation kinds (§3.1). There is no seventh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TraceRelationKind {
    ImplementedBy,
    TestedBy,
    EnforcedBy,
    DocumentedBy,
    ProducedBy,
    ConsumedBy,
}

impl TraceRelationKind {
    /// The authored spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ImplementedBy => "implemented-by",
            Self::TestedBy => "tested-by",
            Self::EnforcedBy => "enforced-by",
            Self::DocumentedBy => "documented-by",
            Self::ProducedBy => "produced-by",
            Self::ConsumedBy => "consumed-by",
        }
    }
}

/// An interface target's side (§3.2): `producer` pairs with `produced-by`,
/// `consumer` with `consumed-by`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterfaceRole {
    Producer,
    Consumer,
}

impl InterfaceRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Producer => "producer",
            Self::Consumer => "consumer",
        }
    }
}

/// One typed target (§3.2). The kinds are closed, and an unknown member of
/// any of them is malformed frontmatter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TraceTarget {
    /// One exact authority unit, expected to be claimed by `spec`.
    Unit { spec: String, unit: Unit },
    /// One selected-content test selector (spec 155 §3.3), by its id.
    Test { selector: String },
    /// One qualified obligation of kind `invariant`.
    Invariant { obligation: String },
    /// One selected-content selector of any kind but `test`.
    Documentation { selector: ContentSelector },
    /// A producer or consumer interface: exactly one of the local form
    /// (`spec` plus `unit`) or the cross-corpus form (`corpus` plus `spec`,
    /// naming one of the declaring spec's `interface_references`).
    Interface {
        role: InterfaceRole,
        spec: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit: Option<Unit>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        corpus: Option<String>,
    },
}

impl TraceTarget {
    /// The target kind's authored spelling.
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Unit { .. } => "unit",
            Self::Test { .. } => "test",
            Self::Invariant { .. } => "invariant",
            Self::Documentation { .. } => "documentation",
            Self::Interface { .. } => "interface",
        }
    }

    /// Whether §3.2's table allows `relation` with this target kind.
    pub fn allows(&self, relation: TraceRelationKind) -> bool {
        use TraceRelationKind as R;
        matches!(
            (self, relation),
            (Self::Unit { .. }, R::ImplementedBy)
                | (Self::Test { .. }, R::TestedBy)
                | (Self::Invariant { .. }, R::EnforcedBy)
                | (Self::Documentation { .. }, R::DocumentedBy)
                | (Self::Interface { .. }, R::ProducedBy | R::ConsumedBy)
        )
    }
}

/// One declared relation (§3.1): exactly one source obligation, one
/// relation kind and one target.
///
/// `deny_unknown_fields`: a misspelt member is refused as malformed
/// frontmatter rather than silently dropped, because a dropped `withdrawn`
/// would resurrect a retired relation (§3.7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceDeclaration {
    /// Unique within the declaring spec, in the obligation id grammar
    /// (spec 106 §3.3). Qualified as `<declaring-spec>#trace:<id>`.
    pub id: String,
    /// A qualified `<spec>#<obligation-id>` reference; the registry carries
    /// its spec half as the full id.
    pub obligation: String,
    pub relation: TraceRelationKind,
    pub target: TraceTarget,
    /// Withdrawn in place: the identity stays occupied (§3.7). Omitted when
    /// false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub withdrawn: bool,
}
