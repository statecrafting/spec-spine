//! # spec-spine-types
//!
//! The typed data substrate for spec-spine: configuration, the spec frontmatter
//! grammar, the authority-unit and typed-edge vocabulary, the registry DTOs,
//! schema-version constants, and the stable [`Error`] enum.
//!
//! Everything here is plain, owned, `serde`-serializable data (no lifetimes,
//! generics, or trait objects at the public boundary) so the same types back
//! both the `spec-spine-core` engine and future FFI bindings.
//!
//! See `docs/design/00-architecture.md` for the design and the provenance of
//! the ported semantics.
//!
//! ## Layout
//! - [`config`]: the `spec-spine.toml` model ([`Config`]).
//! - [`coverage`]: the ownership-coverage report DTOs (spec 032).
//! - [`frontmatter`]: the authored grammar ([`Frontmatter`], [`parse_frontmatter`]).
//! - [`unit`] / [`edges`]: the authority-unit and relationship vocabulary.
//! - [`registry`]: the compiled spec-as-source DTOs ([`Registry`]).
//! - [`verify`]: declared-acceptance plan and report shapes (spec 049).
//! - [`version`]: schema-version constants.
//! - [`error`]: the [`Error`] enum and its exit-code contract.

pub mod attest;
pub mod codebase;
pub mod config;
pub mod coverage;
pub mod edges;
pub mod error;
pub mod frontmatter;
pub mod registry;
pub mod schema;
pub mod unit;
pub mod verdict;
pub mod verify;
pub mod version;

// --- curated public prelude (the names callers reach for most) ---

pub use attest::{
    ATTESTATION_SCHEMA_VERSION, AttestedLifecycle, AttestedUnit, CompileVerdict, CorpusAttestation,
    CoupleVerdict, LedgerSeal, LintVerdict, ResolutionVerdict, SpecAttestation, SpecVerdicts,
    ToolStamp, Verdicts,
};
pub use codebase::{
    CodebaseIndex, Diagnostic, Diagnostics, ImplementingPath, IndexBuild, IndexPackageShard,
    IndexSpecShard, LineSpan, PackageKind, PackageRecord, ResolvedLocation, ResolvedUnit,
    SourceField, TraceMapping, TraceSource, Traceability,
};
pub use config::{
    AllowlistConfig, BrandingConfig, Config, CouplingConfig, FrontmatterConfig, IndexConfig,
    LayoutConfig, ManifestConfig, ProvenanceConfig, load_config,
};
pub use coverage::{CoverageReport, PackageCoverage};
pub use edges::{
    CoAuthorityItem, ConstrainItem, ExtendItem, Origin, Provenance, ReferenceItem, RefineItem,
    SupersedeItem, SupersedeScope, SupersedeScoped,
};
pub use error::{Error, Result};
pub use frontmatter::{
    Frontmatter, FrontmatterIssue, Implementation, KNOWN_KEYS, Risk, Status, parse_frontmatter,
    parse_frontmatter_with, split_frontmatter,
};
pub use registry::{
    Build, BuildMeta, Registry, RegistrySpecShard, Severity, SpecRecord, ValidationReport,
    Violation,
};
pub use schema::{
    BUILD_META_SCHEMA, INDEX_PACKAGE_SHARD_SCHEMA, INDEX_SCHEMA, INDEX_SPEC_SHARD_SCHEMA,
    REGISTRY_SCHEMA, REGISTRY_SPEC_SHARD_SCHEMA,
};
pub use unit::Unit;
pub use verdict::{Verdict, VerdictError, error_kind};
pub use verify::{SkippedBlocks, VerifyFailure, VerifyOutcome, VerifyPlan, VerifyReport};
pub use version::{
    BUILD_META_SCHEMA_VERSION, CONFIG_VERSION, INDEX_SCHEMA_VERSION, REGISTRY_SCHEMA_VERSION,
    SPEC_ATTESTATION_SCHEMA_VERSION, VERDICT_SCHEMA_VERSION, parse_semver,
};
