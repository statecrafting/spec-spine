//! Bounded selected-content request and response types (spec 155).

use serde::{Deserialize, Serialize};

use crate::{RepoPath, Unit};

fn default_projection() -> ContentProjection {
    ContentProjection::Full
}

fn default_required() -> bool {
    true
}

fn default_max_bytes() -> usize {
    262_144
}

fn default_max_items() -> usize {
    64
}

/// One bounded content request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentRequest {
    pub selectors: Vec<ContentSelector>,
    #[serde(default = "default_projection")]
    pub default_projection: ContentProjection,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_max_items")]
    pub max_items: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<ContentContinuation>,
}

/// A selector, with its projection and requiredness kept beside its identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ContentSelector {
    Spec {
        spec: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection: Option<ContentProjection>,
        #[serde(default = "default_required")]
        required: bool,
    },
    SpecSection {
        spec: String,
        anchor: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection: Option<ContentProjection>,
        #[serde(default = "default_required")]
        required: bool,
    },
    Obligation {
        obligation: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection: Option<ContentProjection>,
        #[serde(default = "default_required")]
        required: bool,
    },
    OwnedUnit {
        spec: String,
        unit: Unit,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection: Option<ContentProjection>,
        #[serde(default = "default_required")]
        required: bool,
    },
    File {
        path: RepoPath,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection: Option<ContentProjection>,
        #[serde(default = "default_required")]
        required: bool,
    },
    DirectoryMember {
        directory: RepoPath,
        member: RepoPath,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection: Option<ContentProjection>,
        #[serde(default = "default_required")]
        required: bool,
    },
    Symbol {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection: Option<ContentProjection>,
        #[serde(default = "default_required")]
        required: bool,
    },
    Module {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection: Option<ContentProjection>,
        #[serde(default = "default_required")]
        required: bool,
    },
    Test {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection: Option<ContentProjection>,
        #[serde(default = "default_required")]
        required: bool,
    },
}

impl ContentSelector {
    pub fn projection(&self, fallback: ContentProjection) -> ContentProjection {
        match self {
            Self::Spec { projection, .. }
            | Self::SpecSection { projection, .. }
            | Self::Obligation { projection, .. }
            | Self::OwnedUnit { projection, .. }
            | Self::File { projection, .. }
            | Self::DirectoryMember { projection, .. }
            | Self::Symbol { projection, .. }
            | Self::Module { projection, .. }
            | Self::Test { projection, .. } => projection.unwrap_or(fallback),
        }
    }

    pub fn required(&self) -> bool {
        match self {
            Self::Spec { required, .. }
            | Self::SpecSection { required, .. }
            | Self::Obligation { required, .. }
            | Self::OwnedUnit { required, .. }
            | Self::File { required, .. }
            | Self::DirectoryMember { required, .. }
            | Self::Symbol { required, .. }
            | Self::Module { required, .. }
            | Self::Test { required, .. } => *required,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentProjection {
    Full,
    Declaration,
    Signature,
    Documentation,
    Body,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentDirtyState {
    CleanWorkingTree,
    CleanExport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentSnapshotBinding {
    CallerSupplied,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentSnapshot {
    pub repository: String,
    pub revision: String,
    pub tree: String,
    pub dirty_state: ContentDirtyState,
    pub binding: ContentSnapshotBinding,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentContinuation {
    pub request_digest: String,
    pub snapshot_digest: String,
    pub next_identity: String,
    pub next_ordinal: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSpan {
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentCompleteness {
    Complete,
    Partial,
    Incomplete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentResolution {
    Resolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentOmissionReason {
    MissingContent,
    UnsupportedSelector,
    UnsupportedProjection,
    BinaryContent,
    ItemExceedsByteBudget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentItem {
    pub identity: String,
    pub repository: String,
    pub revision: String,
    pub tree: String,
    pub dirty_state: ContentDirtyState,
    pub selector: ContentSelector,
    pub requested_projection: ContentProjection,
    pub path: RepoPath,
    pub span: ContentSpan,
    pub digest: String,
    pub content: String,
    pub resolution: ContentResolution,
    pub completeness: ContentCompleteness,
    pub schema_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentOmission {
    pub identity: String,
    pub requested_projection: ContentProjection,
    pub required: bool,
    pub reason: ContentOmissionReason,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoalescedSelector {
    pub identity: String,
    pub requested_projection: ContentProjection,
    pub positions: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentResponse {
    pub repository: String,
    pub revision: String,
    pub tree: String,
    pub dirty_state: ContentDirtyState,
    pub completeness: ContentCompleteness,
    pub items: Vec<ContentItem>,
    pub omissions: Vec<ContentOmission>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coalesced_selectors: Vec<CoalescedSelector>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<ContentContinuation>,
}
