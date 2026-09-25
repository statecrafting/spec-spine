# spec-spine API reference

> Stable surface: `spec-spine-core` (the engine) over `spec-spine-types` (the
> data substrate). **The library API, not the CLI, is the contract bindings
> wrap.** This document describes the public Rust API and the JSON-in/JSON-out
> facade. For the CLI surface see [adoption-guide.md](adoption-guide.md); for the
> design rationale see [design/00-architecture.md](design/00-architecture.md);
> for what each result proves, and how to bind it to a revision, see
> [authority-evidence.md](authority-evidence.md).

spec-spine turns a markdown spec corpus into a typed, hash-verifiable authority
ledger. Two views are emitted deterministically and joined at PR time:

- **`registry.json`**: the *spec-as-source* view (compiler output).
- **`index.json`**: the *code-as-source* view (indexer output), with a
  content-hash staleness mechanism.

Every artifact-producing function is a **pure function of `(Config, file
contents)`**: no ambient clock, no environment reads, and **no `git`** (the CLI
parses the diff and passes a typed `DiffInput` in). Same inputs ⇒ byte-identical
output. The one wall-clock value (`build-meta.json.builtAt`) is written by the
CLI and excluded from determinism/golden checks.

---

## 1. Crates

| Crate | Role | Depend on it when… |
|---|---|---|
| `spec-spine-types` | DTOs, frontmatter grammar, `Config`, schema-version constants, embedded JSON Schemas, the `Error` enum | you only need the data shapes (e.g. an overlay reading the registry) |
| `spec-spine-core` | the engine: `compile` / `index` / `lint` / `couple` / query + the governance `scaffold_init` producer + the JSON facade | you are embedding the engine, building over the artifacts, or initializing a corpus from your own tooling |
| `spec-spine-cli` | the thin `spec-spine` multi-call binary | you want the command-line tool (`cargo install spec-spine-cli`) |

`spec-spine-core` re-exports the whole type substrate, so a Rust caller can
depend on `spec-spine-core` alone and reach everything via
`spec_spine_core::types::*` (or the flattened re-exports `Registry`,
`CodebaseIndex`, `Config`, `Error`, …).

```toml
[dependencies]
spec-spine-core = "0.3"
```

---

## 2. The five capabilities + freshness

Each is a pure function of `(Config, on-disk inputs under repo_root)`:

```rust
use std::path::Path;
use spec_spine_core::{
    compile, index, lint, couple, couple_with, parse_waiver,
    check_index_freshness, check_slice_freshness,
    DiffInput, DiffFile, Waiver,
};

pub fn compile(cfg: &Config, repo_root: &Path) -> Result<CompileOutcome, Error>;
pub fn index  (cfg: &Config, repo_root: &Path) -> Result<IndexOutcome,   Error>;
pub fn lint   (cfg: &Config, repo_root: &Path) -> Result<LintReport,     Error>;

// Coupling takes an already-parsed diff + optional waiver; it loads the
// committed registry + index from `derived_dir` itself.
pub fn couple(cfg: &Config, repo_root: &Path,
              diff: &DiffInput, waiver: Option<&Waiver>) -> Result<CoupleReport, Error>;

// Lower-level form for callers that already hold the artifacts (overlays, tests).
// COMPATIBILITY: it holds one snapshot, so a deleted path's owners resolve at
// head. See "Deletions and the prior snapshot" below.
pub fn couple_with(cfg: &Config, registry: &Registry, index: &CodebaseIndex,
                   diff: &DiffInput, waiver: Option<&Waiver>) -> Result<CoupleReport, Error>;

// Spec 100: the forms that judge a deleted path where it lived. `couple_snapshots`
// is the IO form the CLI calls; `couple_with_prior` is its pure counterpart.
pub fn couple_snapshots(cfg: &Config, repo_root: &Path, diff: &DiffInput,
                        waiver: Option<&Waiver>,
                        prior: &PriorSnapshots<'_>) -> Result<CoupleReport, Error>;
pub fn couple_with_prior(cfg: &Config, registry: &Registry, index: &CodebaseIndex,
                         scope: &GovernedScope, prior: &PriorSnapshots<'_>,
                         diff: &DiffInput, waiver: Option<&Waiver>) -> Result<CoupleReport, Error>;

// Reconstruct a prior snapshot from an exported tree's own source bytes.
// `cfg` is that tree's configuration, not the run's.
pub fn prior_ownership_from_root(cfg: &Config, root: &Path) -> Result<PriorOwnership, Error>;

// Cheap staleness check: does the aggregate index's contentHash (folded from
// the committed by-spec / by-package shards) match the current inputs?
pub fn check_index_freshness(cfg: &Config, repo_root: &Path) -> Result<Freshness, Error>;

// Per-slice staleness (spec 011): `name` is a configured `[index.slices]` key.
pub fn check_slice_freshness(cfg: &Config, repo_root: &Path, name: &str) -> Result<Freshness, Error>;

// Ownership coverage (spec 029): which source files no spec specifically
// claims. Freshness-guarded like `couple` (a stale index is Error::Stale);
// the pure form takes an already-loaded index and a path listing.
pub fn coverage     (cfg: &Config, repo_root: &Path) -> Result<CoverageReport, Error>;
pub fn coverage_with(cfg: &Config, index: &CodebaseIndex, files: &[String]) -> CoverageReport;
pub fn classify(index: &CodebaseIndex, path: &str) -> Ownership;   // Specific | FloorOnly(ids) | Unowned
pub fn in_coverage_universe(cfg: &Config, index: &CodebaseIndex, path: &str) -> bool;
pub fn enumerate_source_files(cfg: &Config, repo_root: &Path, index: &CodebaseIndex) -> Vec<String>;
```

Returned outcomes carry both the typed struct and the canonical bytes the CLI
writes, so overlays and tests need not re-parse:

```rust
pub struct CompileOutcome { pub registry: Registry,       pub json: String, pub validation_passed: bool }
pub struct IndexOutcome   { pub index:    CodebaseIndex,   pub json: String }  // hash: index.build.content_hash
pub enum   Freshness      { Fresh, Stale { expected: String, actual: String } }
```

### Coupling input

The gate never shells out; the caller passes a parsed diff:

```rust
pub struct DiffInput { pub files: Vec<DiffFile> }
pub struct DiffFile  { pub path: String, pub hunks: Vec<LineSpan>,    // empty hunks ⇒ whole-file change
                       pub deleted: bool }                            // `+++ /dev/null`; C-002 skips it (spec 029)
pub struct Waiver    { pub reason: String }

// Build a Waiver from a PR body using the configured keyword:
pub fn parse_waiver(cfg: &Config, pr_body: &str) -> Option<Waiver>;

// Spec 113: every waiver a body declares, with its lifecycle lines. The first
// declaration's reason is what parse_waiver returns.
pub fn parse_waivers(cfg: &Config, pr_body: &str) -> WaiverSet;
pub struct WaiverSet { pub declarations: Vec<WaiverDeclaration>, pub unattached: Vec<String> }
pub struct WaiverDeclaration {           // values kept as written; a malformed one fails its check
    pub reason: String, pub paths: Option<Vec<String>>,       // None = unscoped
    pub until: Option<String>, pub since: Option<String>, pub max_uses: Option<String>,
    pub repeated: Vec<String>,           // single-valued keys declared twice
}
impl WaiverDeclaration { pub fn id(&self) -> String; }        // "sha256:...", the use-count key
// The caller's inputs. The library reads no clock and runs no git: an input left
// out leaves its check `not-evaluated`, never satisfied, and a missing count is not zero.
pub struct WaiverInputs {
    pub as_of: Option<String>,                   // YYYY-MM-DD
    pub ancestry: BTreeMap<String, bool>,        // declared -Since: -> is an ancestor of head
    pub uses: BTreeMap<String, u64>,             // waiver id -> prior clearing runs
}
pub fn couple_snapshots_waived(cfg: &Config, repo_root: &Path, diff: &DiffInput,
                               waivers: &WaiverSet, inputs: &WaiverInputs,
                               prior: &PriorSnapshots<'_>) -> Result<CoupleReport, Error>;
pub fn couple_with_prior_waived(cfg: &Config, registry: &Registry, index: &CodebaseIndex,
                                scope: &GovernedScope, prior: &PriorSnapshots<'_>,
                                diff: &DiffInput, waivers: &WaiverSet,
                                inputs: &WaiverInputs) -> Result<CoupleReport, Error>;
// CoupleReport gains `waivers: Vec<WaiverOutcome>` (id, reason, scoped, paths,
// checks [{check, declared, input?, outcome, detail?}], effective, clears
// [{code, path}]) and `unattached_waiver_lines`, both omitted from JSON when
// empty, and `uncleared()`. Every older entry point is one unscoped waiver.

// Mechanical dependency-only auto-waiver (spec 005 §3.5; cargo + workflow
// classes added by spec 027), used when `coupling.auto_waive_dependency_only`
// is set and no PR-body waiver is present. dependency_only_waiver dispatches
// per manifest class; is_dependency_manifest is the CLI pre-filter predicate:
pub struct FileContents { pub path: String, pub base: String, pub head: String }
pub fn dependency_only_waiver(files: &[FileContents]) -> Option<Waiver>;
pub fn is_dependency_manifest(path: &str) -> bool;      // package.json | Cargo.toml | workflow yaml
pub fn dependency_only_change(base: &str, head: &str) -> bool;           // package.json text
pub fn cargo_dependency_only_change(base: &str, head: &str) -> bool;     // Cargo.toml text
pub fn workflow_dependency_only_change(base: &str, head: &str) -> bool;  // workflow yaml text
pub fn is_package_json(path: &str) -> bool;
pub fn is_cargo_toml(path: &str) -> bool;
pub fn is_workflow_yaml(path: &str) -> bool;

// Is a path covered by the bypass floor (+ configured `coupling.bypass_prefixes`)?
pub fn is_bypassed_path(cfg: &Config, index: &CodebaseIndex, path: &str) -> bool;
```

`couple` returns a `CoupleReport` **even on drift**: drift is data, not an
`Error`, so the JSON facade can return the structured report. Map it to an exit
code with `report.has_blocking_drift()` (the CLI does exactly this → exit 1).
`DEFAULT_BYPASS_PREFIXES` is exported so callers can see the always-applied
bypass floor that `coupling.bypass_prefixes` adds to.

---

### Deletions and the prior snapshot (spec 100)

A deleted path is judged against the snapshot it lived in, not against the tree
the deletion produced: the claim that authorizes a removal is the one the
removal withdraws, and at head it is already gone. There are two prior
snapshots and the segment decides which answers.

| Deletion recorded in | Answered by |
|---|---|
| the committed range `merge-base...head` | the merge base |
| the working-tree diff `git diff HEAD` (`--include-uncommitted`) | HEAD |

A snapshot is **reconstructed** by compiling and indexing that commit's
exported tree, never read from its committed derived shards: the shards are
evidence somebody wrote and can be stale, and the corpus source is immutable.
It is built only when a deletion needs one, so a change with no deletion pays
nothing and works in a shallow clone.

**A required snapshot that cannot be obtained is a refusal**, exit `2` (exit
`3` before 0.26.0), naming
the cause and the remedy. The gate does not substitute another revision, does
not ignore corrupt evidence, and does not treat a history it could not read as
an empty diff. A path that is simply **absent** from a snapshot that *was* read
is a different fact: the answer is that nobody owned it there, the owner set is
empty, and the report records the absence.

The report carries, for each deleted path examined, which snapshot answered:
`merge-base`, `head-commit`, or `head-tree`. The block is omitted when empty.

**Which entry points carry this.** `couple` (through `couple_snapshots`), the
CLI, and `couple_json` with `priorRoots` judge deletions at the prior snapshot.
`couple_with` and `couple_with_scope` are **compatibility** entry points: they
hold one snapshot, resolve deletions at head, and label every such deletion
`head-tree` in the report. Their behavior is unchanged and preserved
deliberately; preserving it is not the same as providing the guarantee, and
they should not be described as if it were.

## 3. Config load + init scaffolding

```rust
use spec_spine_types::{Config, load_config};

// Parse and validate a spec-spine.toml. Clean Error::Config on malformed input,
// never a panic. An absent file ⇒ use Config::default() (a working single-Cargo-
// workspace default with specs/ at the root).
pub fn load_config(toml_src: &str) -> Result<Config, Error>;

// The governance scaffold returns files-as-data; the CONSUMER writes them
// (spec 092: the Statecraft CLI, since there is no `spec-spine init`). Pure:
// no writes, no environment reads, no process launches, no clock. It produces
// spec-spine.toml, the constitution, the contract, the two templates, the
// bootstrap spec and a .gitignore fragment, and no agent or environment
// artifact of any kind.
pub fn scaffold_init(cfg: &Config) -> Result<Scaffold, Error>;
// Spec 131: the same, under options. `ScaffoldOptions::default()` is
// byte-identical to `scaffold_init`; `pin_exact_version: true` emits an active
// `[meta]` table with `required_version = "=<this producer's version>"`.
pub fn scaffold_init_opts(cfg: &Config, options: &ScaffoldOptions)
    -> Result<Scaffold, Error>;
pub struct ScaffoldOptions { pub pin_exact_version: bool }
pub struct Scaffold     { pub files: Vec<ScaffoldFile> }
pub struct ScaffoldFile { pub rel_path: String, pub contents: String, pub overwrite: bool,
                          pub executable: bool, pub append: bool,
                          pub append_marker: Option<String> }
```

Every `Config` sub-struct is `#[serde(default, deny_unknown_fields)]`: a
misspelled knob is a loud `Error::Config`, not a silently-ignored setting. See
[adoption-guide.md](adoption-guide.md) §Config for the full knob table.

---

## 4. The overlay seam: typed read-only loaders

These are the public functions an external **overlay** crate depends on to read
a generic artifact and emit its own enriched sibling (`*-<overlay>.json`) without
forking the core:

```rust
use spec_spine_core::{load_registry, load_index};

pub fn load_registry(bytes: &[u8]) -> Result<Registry,      Error>;  // rejects unknown MAJOR schema
pub fn load_index   (bytes: &[u8]) -> Result<CodebaseIndex, Error>;  // rejects unknown MAJOR schema
```

See [overlay-contract.md](overlay-contract.md) for the full extensibility
contract and [schema-versioning.md](schema-versioning.md) for what "rejects
unknown MAJOR" means.

---

## 5. Typed query layer

Read-only queries over a loaded `Registry`:

```rust
use spec_spine_core::{list, list_ids, show, status_report, relationships, ListFilter};

pub fn list        (registry: &Registry, filter: &ListFilter)  -> Vec<&SpecRecord>;
pub fn list_ids    (registry: &Registry, filter: &ListFilter)  -> Vec<&str>;  // idsOnly projection (spec 009)
pub fn show        (registry: &Registry, id: &str)             -> Result<&SpecRecord, Error>;
pub fn status_report(registry: &Registry)                      -> StatusReport;
pub fn relationships(registry: &Registry, id: &str)            -> Result<RelationshipView, Error>;
```

Authority-by-unit resolves over the **index**, where the compiler has already
pre-flattened the registry's edges into resolved units (so "who owns unit X" is a
set-membership lookup, not a runtime graph walk):

```rust
use spec_spine_core::authorities;
use spec_spine_core::types::Unit;

pub fn authorities(index: &CodebaseIndex, unit: &Unit) -> Vec<String>;  // → owning spec ids
```

---

## 6. The `Error` enum → exit codes

A single, stable, `#[non_exhaustive]` enum. The CLI is the **only** place that
maps an `Error` to a process exit code. Since spec 132 the mapping is the
family exit contract shared with the Statecraft CLI:

| Variant | Meaning | Exit | `error.kind` |
|---|---|---|---|
| `Error::Validation(Vec<Violation>)` | compile validation failed | **1** | `validation` |
| `Error::NotFound(String)` | spec id / view / path not found | **1** | `not-found` |
| `Error::Stale { expected, actual }` | committed ledger out of date | **1** | `stale` |
| `Error::Parse(String)` | authored content (frontmatter, a corpus document) that does not parse | **1** | `validation` |
| `Error::Config(String)` | malformed `spec-spine.toml`, or a config JSON the loader refuses | **2** | `config` |
| `Error::Refused(String)` | a precondition or policy not met and nothing done: a version pin not satisfied, a containment refusal (specs 126-128) | **2** | `refused` |
| `Error::Usage(String)` | an argument combination or request document the verb rejects | **3** | `usage` |
| `Error::Io(String)` | filesystem / git / process failure | **4** | `io` |
| `Error::Schema(String)` | a tool-produced artifact (a committed shard, an attestation) that fails to parse or fails its schema/version check | **4** | `schema` |
| `Error::Internal(String)` | a defect in the tool itself, e.g. a value that will not serialize | **4** | `internal` |

`Error::Refused` and `Error::Usage` are new in spec 132; `Error::Internal` is
also new, taking cases that previously mapped `Error::Schema` to a
serialization failure that was not really a schema problem. Before spec 132,
`Config`/`Io`/`Parse`/`Schema` all mapped to exit `3`, `Stale` to `2`, and
there was no `Refused`, `Usage` or `Internal` variant; a `Parse` failure
carried `error.kind: "parse"`, which the closed set no longer has.

Coupling **drift** is *not* an `Error` variant; it is carried in the
`CoupleReport` and mapped to exit **1** (`error.kind` equivalent `drift`) by
the CLI. `Error::exit_code(&self) -> u8` and `Error::outcome(&self) ->
&'static str` are the authoritative mapping.

Per-subcommand exit-code table: see
[design/00-architecture.md](design/00-architecture.md) §6.

---

## 7. The JSON-in / JSON-out facade (the FFI seam)

Every top-level operation has a `&str → Result<String, Error>` facade function.
This is the seam napi / pyo3 / cgo will wrap (see
[bindings-plan.md](bindings-plan.md)); in Rust it returns a typed `Error`, which
the binding layer maps to the same envelope `--json` already writes (spec 132:
`schemaVersion`, `tool`, `verb`, `outcome`, `exitCode`, `summary`,
`report`/`error`).

```rust
pub fn compile_json        (config_json: &str, repo_root: &str) -> Result<String, Error>;
pub fn index_json          (config_json: &str, repo_root: &str) -> Result<String, Error>;
pub fn lint_json           (config_json: &str, repo_root: &str) -> Result<String, Error>;
pub fn check_freshness_json(config_json: &str, repo_root: &str) -> Result<String, Error>;
pub fn check_registry_freshness_json(config_json: &str, repo_root: &str) -> Result<String, Error>;
pub fn coverage_json       (config_json: &str, repo_root: &str) -> Result<String, Error>;
pub fn verify_plan_json    (config_json: &str, repo_root: &str, spec_id: &str) -> Result<String, Error>;
pub fn couple_json         (request_json: &str)                 -> Result<String, Error>;
pub fn delta_json          (request_json: &str)                 -> Result<String, Error>;
pub fn query_json          (request_json: &str)                 -> Result<String, Error>;
pub fn closure_json        (config_json: &str, repo_root: &str, request_json: &str) -> Result<String, Error>;
pub fn scope_json          (config_json: &str, repo_root: &str, scope_json: &str) -> Result<String, Error>;
pub fn scope_compare_json  (a_json: &str, b_json: &str)          -> Result<String, Error>;
pub fn interface_verify_json(request_json: &str)                -> Result<String, Error>;
pub fn render_json         (config_json: &str, index_json: &str) -> Result<String, Error>;
pub fn orphans_json        (index_json: &str)                    -> Result<String, Error>;
pub fn load_config_json    (toml_src: &str)                     -> Result<String, Error>;
pub fn scaffold_init_json  (config_json: &str)                  -> Result<String, Error>;
pub fn scaffold_init_opts_json(config_json: &str, options_json: &str) -> Result<String, Error>;
pub fn attest_json         (config_json: &str, repo_root: &str, with_coupling: bool) -> Result<String, Error>;
pub fn attest_spec_json    (config_json: &str, repo_root: &str, spec_id: &str)       -> Result<String, Error>;
pub fn verify_attestation_json     (request_json: &str)         -> Result<String, Error>;
pub fn verify_spec_attestation_json(request_json: &str)         -> Result<String, Error>;
```

- `config_json` is a JSON object matching `Config`; `"{}"` ⇒ `Config::default()`.
  Since spec 129 it, and the `config` member of every request that carries
  one, is held to the rules `load_config` applies to a `spec-spine.toml`
  (the `[index.slices]` grammar, spec 128's `derived_dir` rule, spec 036's
  `state_dir` rule), through the public `spec_spine_types::validate_config`.
  A configuration the loader would refuse is `Error::Config` (exit 2, exit 3
  before 0.26.0) with the
  loader's message, before the entry reads anything. `scaffold_init` and
  `scaffold_init_json` refuse it too, and escape every value they write into
  the starter `spec-spine.toml`. `[meta] required_version` is not checked
  here: only the CLI knows the version it runs as.
- `scaffold_init_opts_json` (spec 131) takes the options as a JSON
  object: `{"pinExactVersion": true}` returns the scaffold with an active
  `[meta]` table and `required_version = "=<this producer's version>"`, so the
  written repository is judged only by the spec-spine release that produced
  it. `"{}"` or `{"pinExactVersion": false}` returns exactly what
  `scaffold_init_json` returns. An unknown option key or options that are not
  JSON is `Error::Config`, so a request for a pin never yields an unpinned
  file silently. Only `spec-spine.toml`'s `[meta]` block differs; every other
  byte and file is the same.
- `closure_json` request (spec 107): `{ "specs"?: [id], "sections"?: [{ "spec",
  "anchor" }], "obligations"?: ["<spec-id>#<obligation-id>"], "rationale"?:
  string }`, at least one member named, unknown members refused. The answer is
  a read document (read schema `0.4.0`): `members`, each tagged `kind` (`spec`
  with `contentHash`, `section` with `digest`, `obligation` with its whole
  resolved member: `obligationKind`, `text`, `anchor`, `inputs` when non-empty,
  `withdrawn` when true, and `sectionDigest`, every one of which its piece
  digests), sorted by kind then identity; `digest`, the one hash
  construction over one piece per member (`spec:<id>`, `section:<id>#<anchor>`,
  `obligation:<id>#<obligation-id>`), so reordering, repeating or short-naming
  a member changes nothing and editing a named member's content changes it; and
  `rationale`, carried and never digested. It checks registry freshness first
  and refuses a stale ledger (exit 1, exit 2 before 0.26.0); an empty or
  unqualified request is exit 3; every unresolved reference is named in one
  exit-1 refusal. A closure lives
  in the consumer's record; this only resolves one, and no gate reads it.
- `scope_json` request (spec 108): a `ScopeRequest` document, `{ "id"?:
  string, "ownSpec": id, "mutable"?: [path], "shared"?: [{ "path", "with":
  [id] }], "readOnly"?: [path] }`, at least one path in total, no absolute
  path or `..` segment, a non-empty `with` for every `shared` entry, and no
  path (or subtree and a path inside it) named under two roles; unknown
  members refused (exit 3). It checks index freshness first and refuses a
  stale index (exit 1, exit 2 before 0.26.0); an unresolved `ownSpec` or `with` is one exit-1
  refusal naming every one. The answer is a read document (read schema
  `0.7.0`): `ownSpec` and `indexHash` (the committed index's aggregate content
  hash), `entries`, one per declared path with its resolved `role`, `owners`
  (the specs the committed index says own it, by the same function `index
  owner` uses) and, for a `shared` entry, resolved `with`, and `findings`,
  sorted by path then code: `S-001` unowned, `S-002` undeclared crossing (a
  `mutable` path another spec owns), `S-003` sharing mismatch (a `shared`
  path whose owners disagree with its declared `with`). It exits 0 whether or
  not it found anything: a report, not a gate. The library form is
  `evaluate_scope(&Config, &Registry, &CodebaseIndex, &ScopeRequest)`
  (pure), and `evaluate(&Config, repo_root, &ScopeRequest)` adds the
  freshness guard and the committed-ledger read. A scope lives in the
  consumer's record; this only evaluates one, and no gate reads it.
- `scope_compare_json` request (spec 108): two `ScopeRequest` documents (the
  same shape `scope_json` takes), validated the same way but never resolved
  against a ledger: comparison is a pure function of the two documents. The
  answer is a read document naming each scope's `id` and `ownSpec` as
  declared, and `conflicts`: every overlapping pair of declared paths (equal,
  or one a subtree containing the other) whose roles conflict per spec 108
  §3.5's table (`both-mutable`, `mutable-shared`, `changed-under-read`; two
  `shared` or two `readOnly` entries never conflict). Exits 0 whether or not
  any conflict is found. The library form is `compare_scopes(&ScopeRequest,
  &ScopeRequest)`.
- `interface_verify_json` request (spec 110): `{ "registry": "<registry.json
  text>", "exports": { "<corpus>": { "<spec-id>": { "path", "text" } } },
  "spec"?: id }`, unknown members refused (exit 3). `path` is the cited spec's
  repo-relative path **in the exporting corpus** (it is part of the hash
  construction), `text` its `spec.md` bytes. No filesystem: a binding supplies
  the exported texts it already holds. The answer is a read document (read
  schema `0.6.0`), the same one `interface verify --json` prints:
  `references`, each `{ "declaredBy", "corpus", "spec", "digest",
  "observedDigest"?, "outcome", "sections"? }` with `outcome` one of `current`,
  `sections-current`, `stale`, `missing`, `unverified`, and each section
  `{ "anchor", "digest", "observedDigest"?, "outcome" }` (`current`, `stale`,
  `missing`); sorted by `(declaredBy, corpus, spec)`; and `summary`, a count
  per outcome. `current` and `sections-current` hold; the other three do not.
  An unknown `spec` is exit 1. The library form is
  `verify_interface_references(&Registry, &Exports, Option<&str>)`, and
  `interface_verify(&Config, repo_root, &BTreeMap<corpus, dir>, Option<&str>)`
  adds the committed-ledger read (a stale registry is exit 1, exit 2 before
  0.26.0, refused before any export is read) and `load_export`, which reads
  only `<specs_dir>/<id>/spec.md` for the referenced ids and refuses (exit 2)
  a path that resolves outside the export root, a containment refusal
  distinct from an unreadable export directory (I/O, exit 4). Nothing
  fetches, and nothing writes a pin: an observed digest is reported for a
  human to copy.
- `query_json` request: `{ "registry": "<registry.json text>", "op":
  "list" | "show" | "status-report" | "relationships" | "plan" |
  "obligation" | "impacts" | "moves", "id"?: string,
  "status"?: string, "idsOnly"?: bool, "nonzeroOnly"?: bool, "target"?: string,
  "declaredBy"?: string, "path"?: string }` (the projection fields, spec 009,
  default to `false`). Every answer is a **read document**
  (spec 074): an object with sorted keys and `schemaVersion` =
  `READ_SCHEMA_VERSION`; `list` (with or without `idsOnly`) carries its array
  under `items`. `obligation` (spec 106) takes `id` as a qualified
  `<spec-id>#<obligation-id>` and returns `{ "spec", "specPath",
  "obligation", "sectionDigest", "schemaVersion" }`; an unqualified `id` is a
  usage error (exit 3; `error.kind` was `parse`, now `usage`), never resolved
  against a spec. It carries no
  `contentHash`, because registry text has none; the CLI's `registry
  obligation` adds it from the committed shard, as `show` does.

  `impacts` (spec 109) inverts every declared impact and conflict against
  spec 106's obligations, from either side, filtered by `target` (a spec id,
  every obligation it declares, or a qualified `<spec-id>#<obligation-id>`
  reference) and `declaredBy` (a spec id), composed by intersection; with
  neither, every declaration in the corpus. The answer is `{ "impacts": [{
  "declaredBy", "target", "nature", "successor"?, "note"?, "targetWithdrawn"
  }], "conflicts": [{ "declaredBy", "target", "reason", "resolution",
  "settledBy"?, "targetWithdrawn" }], "schemaVersion" }`, both arrays sorted
  by `(target, declaredBy)`. `target` and `declaredBy` in each entry are full
  ids; `target` is the full qualified obligation reference regardless of the
  short form a filter or an author used. A `target` carrying `#` that is not a
  qualified reference (an empty or blank half, or a second `#`) is a usage
  error (exit 3; `error.kind` was `parse`, now `usage`), never resolved
  against a spec; a target spec, target obligation or declaring spec that
  does not exist is `NotFound` (exit 1).
  None of these three is answered as an empty result: an empty `impacts` and
  `conflicts` pair means every filter resolved and nothing matched, never that
  a filter failed to resolve.

  `moves` (spec 111) looks up `path` against every declared move in the
  committed registry, following declared chains, never a similarity or
  content signal (§3.6), and changes no verdict (§3.5). With `path`, the
  answer is `{ "outcome": "unmapped" | "resolved" | "ambiguous" | "cycle",
  "path", ... }`: `unmapped` carries only `path`; `resolved` adds `hops`
  (each `{ "from", "to"?, "kind", "declaredBy": [...], "answeredBy"? }`) and
  `terminals` (each `{ "path"?, "answeredBy"? }`); `ambiguous` adds `at` (the
  path the declarations disagreed about) and `candidates` (every disagreeing
  declaration, none picked); `cycle` adds `chain`, the repeated path naming
  the loop's start and end. Without `path`, the answer is every declaration
  flattened to one entry per branch/source and sorted by `(from, to,
  declaredBy)`: `{ "items": [{ "from", "to"?, "kind", "declaredBy",
  "answeredBy"? }], "schemaVersion" }`. The library form is
  `lookup_move(&Registry, &str) -> MoveLookup` and
  `flattened_moves(&Registry) -> Vec<MoveEntry>`; the CLI's `registry moves`
  exits `0` on `unmapped`/`resolved` and `1` on `ambiguous`/`cycle`, the
  lookup refusing to guess rather than an error.

  `plan` (spec 035) returns `{ "ready": [{ "id", "status", "title" }],
  "blocked": [{ "id", "blockedBy": [{ "id", "state" }] }], ...,
  "schemaVersion" }`. A ready entry's `status` (spec 102, read schema `0.2.0`)
  is the spec's `status` as the registry records it, verbatim, so a consumer
  can apply its approval rule from this one document.

  **What membership of `ready` means** (spec 101). It is a **scheduling**
  answer: every `depends_on` target is satisfied and the spec is itself
  schedulable. It is **not an approval**, not a permission to execute, and not
  a claim that any human has read the spec.

  Approval is not a partition key. `status` is consulted to exclude
  `superseded` and `retired` (spec 035 section 3.1), and, only when a spec
  declares no `implementation`, to read the absent key as `pending` on a draft
  and as settled on anything ratified (spec 042). So a `status: draft` spec
  appears on `ready` as soon as its dependencies are met. That is by design,
  not a defect, and it is what lets a repository whose cadence is
  draft-then-build and one whose cadence is ratify-then-build read the same
  document.

  The approval rule therefore belongs to the consumer, applied **on top of**
  `plan`; a ready entry's `status` is the value that rule reads. This
  repository's own `/next` does exactly that: it drops a draft from the ready
  set and reports it as awaiting approval (spec 093). A consumer
  that treats `ready` as a work queue without adding such a rule is reading the
  document correctly and reaching a conclusion the document does not support.
- `couple_json` request: `{ "config"?: Config, "repoRoot": string, "diff":
  DiffInput, "waiver"?: { "reason": string }, "waivers"?: [WaiverDeclaration],
  "prBody"?: string, "waiverInputs"?: { "asOf"?: string, "ancestry"?: { commit:
  bool }, "uses"?: { waiverId: number } }, "priorRoots"?: { "mergeBase"?:
  string, "headCommit"?: string, "worktreeDeletions"?: [string] } }`.
  At most one of `waiver`, `waivers` and `prBody` (spec 113); `prBody` is
  parsed with the configured keyword exactly as the CLI parses `--pr-body`.
  `waiverInputs` is the only source of a lifecycle input: the facade reads no
  clock and runs no git.
  `priorRoots` (spec 100) names exported trees in exactly the sense
  `delta_json` takes `baseRoot` and `headRoot`; each is compiled and indexed
  under **its own** `spec-spine.toml`. Absent, deletions resolve at head, which
  is the compatibility behavior and not the correction. **Each side is
  independent**: supplying `mergeBase` without `headCommit` (or the reverse)
  is accepted, and the side you did not supply resolves at head and is labelled
  `head-tree` in the report. The library does not guess what a partial set
  meant, and the CLI never sends one: it exports exactly the sides the run's
  deletions need.
- `delta_json` (spec 071) request: `{ "config"?: Config, "baseRoot": string,
  "headRoot": string, "changed": [string], "commits": { "base", "mergeBase",
  "head" } }`. The two roots are exported trees; `config` is the merge base's
  (absent, it is read from `<baseRoot>/spec-spine.toml`). Returns the
  `DeltaReport`: every changed path with its classes under the base's rules,
  per-class `counts`, and `priorPolicy`. A record, never a refusal; a stale
  committed index at the base is `Error::Stale`. `priorPolicy.required: false`
  means only that no structural class changed, not that the change is safe,
  correct or approved.
- `check_freshness_json` returns `{ "fresh": bool, "expected"?, "actual"? }`.
  `check_registry_freshness_json` (spec 028) returns the same shape for the
  committed registry shards; staleness only, the validation verdict rides on
  `compile_json`.
- `attest_json` (spec 021) and `attest_spec_json` (spec 039) return
  `{ "attestation": <CorpusAttestation | SpecAttestation>, "attestationHash":
  "<hex>" }`. Both are pure: no key, no clock (signing is a CLI post-pass). A
  failing verdict still yields a payload; attestation is a record, not a gate.
- **Two digests of one `spec.md`, two names (spec 077).** A `SpecAttestation`'s
  `specSourceHash` is SHA-256 over the file's normalized bytes (BOM stripped,
  CRLF and CR folded to LF) with **no path prefix**: the value an ordinary
  digest of the normalized file reproduces, reported only by `attest --spec`.
  `registry show`'s `contentHash` is the committed registry shard's
  `shardHash`: SHA-256 over the repo-relative POSIX path, a NUL byte, then the
  same normalized bytes. The two are never equal for one file, and
  `registry show` does not report the unframed one, because it reads the
  committed ledger and never recomputes (spec 048 §3.2).
- `verify_attestation_json` / `verify_spec_attestation_json` request:
  `{ "config"?: Config, "repoRoot": string, "attestation": <...> }`; they return
  `{ "outcome": "match" }`, `{ "outcome": "versionMismatch", "expected",
  "actual" }`, or `{ "outcome": "contentMismatch", "differences": [...] }`.
- The CLI's `--json` verdict envelope (spec 034) wraps these payloads verbatim
  under `report`, versioned by `VERDICT_SCHEMA_VERSION`; see
  `specs/034-machine-readable-verdicts/spec.md`.
- `coverage_json` (spec 029) returns the `CoverageReport` as a read document
  (spec 074, so it also carries `schemaVersion`): `sourceFiles`,
  `claimedFiles`, the sorted `floorOnlyFiles` / `unclaimedFiles` lists, and
  per-package counts. A stale committed index is `Error::Stale`, not a report.
- `verify_plan_json` (spec 043) returns a spec's `VerifyPlan`: the `verify:cli`
  commands its `## Verification` section declares, in document order, plus the
  fence tags it declined (`skipped`). `spec_id` accepts the short form (spec
  015). **The plan is all the library produces**: running the commands is the
  caller's act, never the engine's, which is the same seam that keeps `git` on
  the CLI side of `couple`. A library with no shell can still read the plan.
- `render_json` (spec 010) takes `config_json` and the aggregate index JSON
  text and returns the markdown projection (a JSON-encoded string).
  `orphans_json` (spec 010) takes only the index JSON text and returns the
  orphaned-spec ids under `items` in a read document (spec 074).

All emitted JSON is **pretty-printed with sorted keys, LF line endings, and a
trailing newline** (diffability over compactness; see
[design/00-architecture.md](design/00-architecture.md) §10.1). For the read
documents this holds because they all go through one emitter,
`spec_spine_core::read_document` (spec 074); the facades that return compact
JSON (`lint_json`, `couple_json` and the other verdict payloads) are the
exception, and the CLI's envelope around them is sorted and pretty.

---

## 8. Binding-readiness invariants (what the boundary guarantees)

These hold across the public surface and are what make the library safe to wrap
from another language:

- Owned, `serde`-serializable plain-data DTOs: **no lifetimes, generics, or
  trait objects** at the boundary.
- A single `Error` enum with stable, documented variants → stable exit codes.
- **No `process::exit`, no `println!`-for-data, no `panic!`-on-user-input** inside
  the library. Those live only in `spec-spine-cli`.
- **No `unsafe`** anywhere (`unsafe_code = "forbid"` workspace-wide).
- Pure functions of `(Config, file bytes)`; no ambient clock/env in core.
