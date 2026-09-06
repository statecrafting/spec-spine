//! Ledger-seal DTOs (spec 023): the `CorpusAttestation` payload and the detached
//! `LedgerSeal` envelope.
//!
//! Plain data only: no crypto and no clock. The `CorpusAttestation` is a pure
//! function of `(config, file contents)` (built in `spec-spine-core::attest`),
//! and the `LedgerSeal` carries the non-reproducible signing identity and time,
//! populated by the CLI. Keeping the timestamp and key id in the detached seal,
//! never in the payload, is what lets the attested fact stay reproducible while
//! the act of attesting carries its own identity. Field names serialize
//! `camelCase`, matching the registry and index wire.

use serde::{Deserialize, Serialize};

/// The `schemaVersion` emitted in a [`CorpusAttestation`]. Library-owned,
/// started fresh at `0.1.0`, on its own axis (independent of the registry and
/// index schema lines). MINOR = additive; MAJOR = breaking, and loaders reject
/// an unknown MAJOR (see `docs/schema-versioning.md`).
pub const ATTESTATION_SCHEMA_VERSION: &str = "0.1.0";

/// The tool identity recorded in an attestation: the reproducibility anchor
/// (spec 023 FR-005). A `--recompute` verify is meaningful only under the same
/// `version`; a different version is a distinct, named outcome, never a false
/// content mismatch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolStamp {
    pub name: String,
    pub version: String,
}

/// The compile verdict: did structural validation pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileVerdict {
    pub ok: bool,
}

/// The lint verdict: pass/fail plus a hash over the canonical findings, so a
/// change in the findings set is detectable on recompute even when `ok` is
/// unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LintVerdict {
    pub ok: bool,
    pub findings_hash: String,
}

/// The coupling verdict, present only under `attest --with-coupling` (spec 023
/// FR-002): specs and code are in sync (every claimed unit resolves, no blocking
/// resolver diagnostic). `index_hash` is the code-as-source content hash;
/// `join_hash` binds the registry and index hashes into one handle over the
/// joined claim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoupleVerdict {
    pub ok: bool,
    pub index_hash: String,
    pub join_hash: String,
}

/// The verdicts an attestation freezes. `couple` is present iff the attestation
/// was built with coupling scope; its absence (not a silent default) marks the
/// narrower spec-corpus-only claim (FR-002).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdicts {
    pub compile: CompileVerdict,
    pub lint: LintVerdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub couple: Option<CoupleVerdict>,
}

/// A reproducible, pure-function attestation over the spec corpus state (spec
/// 023). No clock, no env, no key: re-running `attest` on an unchanged corpus at
/// the same `tool.version` yields a byte-identical payload. It is the signed,
/// archival form of a verdict spec-spine already computes (`compile`, `lint`,
/// optionally `couple`) and otherwise discards into a CI log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorpusAttestation {
    pub schema_version: String,
    pub tool: ToolStamp,
    /// SHA-256 over what spec-spine read: the path-sorted, normalized corpus
    /// inputs (the registry's own input content hash). Content-derived rather
    /// than a git SHA, so the payload stays git-agnostic.
    pub inputs_manifest_hash: String,
    /// SHA-256 over the canonical `registry.json` bytes.
    pub registry_hash: String,
    pub verdicts: Verdicts,
}

/// One owning unit of a spec, with the content hash of what it resolved to
/// (spec 042).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestedUnit {
    pub unit: crate::unit::Unit,
    /// SHA-256 over the normalized bytes of the resolved location, or `None`
    /// when the unit resolves to nothing. Absent rather than an empty string:
    /// a hash of nothing and a hash that happens to be zero must not read alike.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

/// A spec's own lifecycle, as declared at attestation time (spec 042).
///
/// `implementation` is omitted **only** when the key is absent from the spec's
/// frontmatter. Every declared value is carried, `n-a` included: an absent key
/// means nobody stated an intention, `n-a` means someone stated that none
/// applies, and collapsing the second into the first would lose the more
/// informative of the two.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestedLifecycle {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<String>,
}

/// The resolution verdict: does every owning unit this spec claims resolve to
/// an existing location (spec 042).
///
/// Records the **fact**, never the indexer's severity tier for it: an in-flight
/// spec whose phantom unit is only a `W-001` still attests `ok: false`. Tying
/// the flag to the tier would make an attestation report "resolution ok" for a
/// unit that does not exist, which is the one thing this payload exists to make
/// impossible, and would make the record depend on the subject's lifecycle
/// rather than on the corpus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionVerdict {
    pub ok: bool,
}

/// The verdicts a [`SpecAttestation`] freezes.
///
/// There is no `couple` verdict: coupling is a property of a diff between two
/// revisions, not of a spec at one revision, and spec 023 already carries the
/// corpus-scoped version for consumers that want it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecVerdicts {
    pub compile: CompileVerdict,
    pub resolution: ResolutionVerdict,
    pub lint: LintVerdict,
}

/// A reproducible attestation scoped to one spec (spec 042).
///
/// Spec 023's `CorpusAttestation` answers "was the corpus sound at this
/// revision". The unit of work in a governed build is one spec, and this answers
/// the same question about one spec's territory: its own source hash, every
/// owning unit it claims with the content hash of what that resolved to, and the
/// verdicts restricted to it.
///
/// Pure function of `(config, file contents)`, exactly as 023: no clock, no
/// environment, no git. The wall-clock instant and the signer identity live in
/// the detached [`LedgerSeal`], so the attested fact stays reproducible while
/// the act of attesting is dated and attributed.
///
/// **No lint rule may consume one of these.** The payload records `lint.ok`, so
/// a rule conditioned on an attestation would be grading its own output. See
/// spec 042 3.4.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecAttestation {
    pub schema_version: String,
    pub tool: ToolStamp,
    pub spec_id: String,
    /// SHA-256 over the spec's own normalized `spec.md` bytes.
    pub spec_source_hash: String,
    pub lifecycle: AttestedLifecycle,
    /// The spec's **owning** units in the registry's canonical order.
    ///
    /// Non-owning `references` units are excluded: spec 034 settled that a cited
    /// file is not a claimed one, and an attestation of territory must not
    /// assert authority the gate does not.
    pub units: Vec<AttestedUnit>,
    pub verdicts: SpecVerdicts,
}

/// The detached Ed25519 seal over an attestation (spec 023 FR-003). Produced
/// only by `attest --sign`. It carries its own non-reproducible identity and
/// time, kept OUT of the pure payload so the attested fact stays reproducible
/// while the act of attesting is dated and attributed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerSeal {
    /// The signature algorithm; `"ed25519"` in v1.
    pub alg: String,
    /// An opaque signer identifier (e.g. a public-key fingerprint).
    pub key_id: String,
    /// RFC3339 wall-clock instant the seal was produced (CLI-populated).
    pub signed_at: String,
    /// Lowercase-hex Ed25519 signature over the 32-byte attestation hash.
    pub sig: String,
}
