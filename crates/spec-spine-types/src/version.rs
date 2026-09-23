//! Schema-version constants: library-owned, started fresh at `0.1.0`.
//!
//! These are deliberately decoupled from the reference repos' version lines
//! (OAP registry `2.2.0`, index `3.0.0`). They are compile-time constants; in
//! Phase 2/3 the conformance tests assert that emitted JSON validates against the
//! embedded schema of the matching version, so a mismatch fails the build rather
//! than at runtime.
//!
//! Versioning policy (see `docs/design/00-architecture.md` §7 and, in Phase 5,
//! `docs/schema-versioning.md`): MINOR = additive only; MAJOR = breaking, and
//! loaders reject an unknown MAJOR. Under `0.x`, MINOR may break (SemVer `0.x`).

/// `specVersion` emitted in the registry, carried by each registry shard.
/// `0.2.0`: declared extra-frontmatter values widen to arbitrary JSON (spec 012).
/// `0.3.0`: structured/partial `supersedes` items (spec 018); full supersession
/// stays a bare string, so a full-only corpus is byte-identical.
/// `1.0.0`: **MAJOR** (spec 022). The committed registry is sharded per-spec
/// under `by-spec/<id>.json`; the single `registry.json` is no longer emitted.
/// The aggregate view (validation, content hash) is recomputed on read. Loaders
/// reject an unknown MAJOR, so a 0.x reader cannot misread a 1.x shard tree.
/// `1.1.0`: additive MINOR (spec 026). A `references` provenance item may carry
/// an optional `derived_at` ISO-8601 timestamp; the registry format gains an
/// emittable field, so the minor bumps. The permissive shard schema is unchanged
/// and a corpus that declares no `derived_at` emits byte-identical record bodies.
/// `1.2.0`: additive MINOR (spec 063). A unit payload may carry an optional
/// `planned: true`, declaring territory a spec intends to own and has not
/// written yet. Follows the precedent 028 set: additive, no MAJOR, loaders that
/// know `1.x` keep working. `planned` is serialized only when true, and a
/// written `planned: false` normalizes to absent, so a corpus that uses no
/// planned units emits byte-identical shards across this change and needs no
/// re-index. `Config` and the DTOs derive `deny_unknown_fields`, so a binary
/// predating this spec meets the key with a parse error and exits 3 rather than
/// silently ignoring a claim about territory: the fail-closed direction, and the
/// reason the flag is a typed field rather than a convention in a comment.
/// `1.3.0`: additive `amendsVerification` (spec 082). A spec that amends
/// another may declare that its own `## Verification` block replaces the
/// amended spec's, so `verify <amended-id>` runs the replacement. Absent on
/// every existing spec, so the field is omitted from every existing shard and
/// only `specVersion` is restamped; `shardHash` is over `spec.md`'s bytes and
/// does not move.
/// `1.4.0`: additive `obligations` and `sectionDigests` (spec 106). A spec may
/// declare obligations; every spec's record carries a digest per body section.
/// Unlike 1.3.0, every existing shard gains `sectionDigests` (every spec has
/// headings), so every shard's bytes change; `shardHash` still does not, since
/// it is over `spec.md`. A binary predating this spec meets the member with a
/// parse error and exits 3, the fail-closed direction 1.2.0 chose.
/// `1.5.0`: additive `impacts` and `conflicts` (spec 109). A spec may declare
/// its relation to, or a knowing disagreement with, another spec's obligation
/// (spec 106); `obligation` is normalized to its full qualified form. Absent
/// on every existing spec, so only `specVersion` is restamped and no
/// `shardHash` moves (it is over `spec.md`'s bytes). A binary predating this
/// spec meets either member with a parse error and exits 3, the same
/// fail-closed direction 1.2.0 and 1.4.0 chose.
/// `1.6.0`: additive `interfaceReferences` (spec 110). A spec may declare a
/// cross-corpus interface reference: a citation of another repository's spec,
/// pinned to its content hash and optionally its section digests. Absent on
/// every existing spec, so only `specVersion` is restamped and no `shardHash`
/// moves (it is over `spec.md`'s bytes). A binary predating this spec meets
/// the member with a parse error and exits 3, the same fail-closed direction
/// 1.2.0, 1.4.0 and 1.5.0 chose.
/// `1.7.0`: reserved for spec 114, built concurrently on another branch and
/// not present in this history yet.
/// `1.8.0`: additive `moves` (spec 111). A spec may declare a relocation,
/// split, merge or removal of a path it once owned, `answered_by`'s spec half
/// normalized to its full id (spec 111 §3.2, by way of spec 015's short-id
/// resolution). Absent on every existing spec, so only `specVersion` is
/// restamped and no `shardHash` moves (it is over `spec.md`'s bytes). A
/// binary predating this spec meets the member with a parse error and exits
/// 3, the same fail-closed direction 1.2.0, 1.4.0, 1.5.0 and 1.6.0 chose.
pub const REGISTRY_SCHEMA_VERSION: &str = "1.8.0";

/// `schemaVersion` emitted in the codebase index, carried by each index shard.
/// `0.2.0`: additive `build.sliceHashes` (spec 011).
/// `0.3.0`: additive `directory`/`crate`/`module` resolved-unit kinds (spec 016).
/// `1.0.0`: **MAJOR** (spec 022). The committed index is sharded per-spec under
/// `by-spec/<id>.json` and per-package under `by-package/<slug>.json`; the single
/// `index.json` is no longer emitted. The aggregate view (orphans, untraced code,
/// content hash) is recomputed on read; staleness is per-shard.
/// `1.1.0`: additive (spec 023). The resolver downgrades an unresolved unit to a
/// non-blocking `W-001` (draft/pending owning) or `W-002` (non-owning reference)
/// warning instead of a hard error; the `warnings` tier and free-form diagnostic
/// `code` already exist, so no schema-file edit is needed.
pub const INDEX_SCHEMA_VERSION: &str = "1.1.0";

/// `schemaVersion` emitted in `build-meta.json` (the non-deterministic artifact).
pub const BUILD_META_SCHEMA_VERSION: &str = "0.1.0";

/// `schemaVersion` carried by a per-spec attestation (spec 039).
///
/// Independent of the registry, index and corpus-attestation versions: an
/// external consumer pins the shape of the evidence it verifies without pinning
/// the ledger it was derived from, which is the point of a bundle format
/// crossing a trust boundary.
pub const SPEC_ATTESTATION_SCHEMA_VERSION: &str = "0.1.0";

/// `schemaVersion` carried by the `--json` verdict envelope (spec 034).
///
/// Independent of the registry and index versions on purpose: a consumer pins
/// the shape of the verdict it parses without pinning the ledger it reads.
/// MINOR is additive, which includes adding an `error.kind` token or a `verb`
/// token (a consumer's existing branches still match); MAJOR is breaking,
/// which includes renaming or removing one (they stop matching).
///
/// 0.2.0 added the `verify` verb (spec 043); 0.3.0 added `compile.spec`
/// (spec 049); 0.4.0 added `delta` (spec 071); 0.5.0 added `couple`'s
/// `deletions` block, which names the snapshot that answered for each deleted
/// path (spec 100 §3.7). Each is the additive case this doc-comment names, and
/// each followed the same reasoning rather than reopening it.
///
/// 0.5.0 is additive in the strict sense the policy requires: the block is
/// omitted when empty, so every input that produced a verdict before spec 100
/// still produces the same payload bytes.
///
/// 0.6.0 (spec 113): `couple`'s report gains `waivers`, each declared waiver
/// evaluated with what it cleared, and `unattachedWaiverLines`. Both are
/// omitted when empty, so a run that declares no waiver keeps its payload
/// bytes, and no existing member changed meaning.
pub const VERDICT_SCHEMA_VERSION: &str = "0.6.0";

/// `schemaVersion` carried by a change-classification report (spec 071).
///
/// On its own axis, like the per-spec attestation: a consumer that stores what a
/// change was classified as pins the shape of that record without pinning the
/// envelope it arrived in or the ledger it was classified against.
pub const DELTA_SCHEMA_VERSION: &str = "0.1.0";

/// `schemaVersion` carried by every read document (spec 074): the JSON a read
/// verb, or the facade function behind it, emits when it answers a question
/// rather than rendering a verdict.
///
/// On its own axis, starting at `0.x` as `DELTA_SCHEMA_VERSION` did. It versions
/// the shape of the answer, not the artifacts the answer is about, so it does
/// not move when `REGISTRY_SCHEMA_VERSION` or `INDEX_SCHEMA_VERSION` does. One
/// constant for every read document: per-verb axes would always move together.
///
/// `0.2.0` (spec 102): additive. Each `registry plan` ready entry carries the
/// spec's `status`, verbatim. No member moved and none was removed.
///
/// `0.3.0` (spec 106): additive. A new read document, the `obligation` answer
/// (`registry obligation --json`, `query_json` `op: "obligation"`). No member
/// of an existing document moved.
///
/// `0.4.0` (spec 107): additive. A new read document, the resolved closure
/// (`registry closure --json`, `closure_json`). No member of an existing
/// document moved.
///
/// `0.5.0` (spec 109): additive. A new read document, the impact set
/// (`registry impacts --json`, `query_json` `op: "impacts"`). No member of an
/// existing document moved.
///
/// `0.6.0` (spec 110): additive. A new read document, the interface report
/// (`interface verify --json`, `interface_verify_json`). No member of an
/// existing document moved.
///
/// `0.7.0` (spec 108): additive. Two new read documents, the scope evaluation
/// (`scope evaluate --json`, `scope_json`) and the scope comparison (`scope
/// compare --json`, `scope_compare_json`). No member of an existing document
/// moved.
///
/// `0.8.0` (spec 111): additive. A new read document, the move lookup
/// (`registry moves [<path>] --json`, `query_json` `op: "moves"`), and the
/// flattened move list it answers with when no path is given. No member of an
/// existing document moved.
pub const READ_SCHEMA_VERSION: &str = "0.8.0";

/// `schemaVersion` of an authority snapshot (spec 070): its own axis, defined
/// beside the DTO it versions and re-exported here with the others.
pub use crate::snapshot::SNAPSHOT_SCHEMA_VERSION;

/// The `spec-spine.toml` config schema version (optional `config_version` key).
pub const CONFIG_VERSION: &str = "0.1.0";

/// Parse a `MAJOR.MINOR.PATCH` string into its numeric components.
///
/// Returns `None` if the string is not three dot-separated non-negative integers.
/// Used by loaders to reject an unknown MAJOR.
pub fn parse_semver(v: &str) -> Option<(u64, u64, u64)> {
    let mut parts = v.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}
