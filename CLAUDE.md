# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

spec-spine turns a markdown spec corpus into a typed, hash-verifiable authority
ledger and **refuses code that drifts from its owning spec** at PR time. It is a
three-crate Rust workspace publishing an installable library + CLI, plus npm and
PyPI shims that ship the prebuilt binary.

Two documents outrank this one for their subjects. Read
`docs/design/00-architecture.md` for the design (crate layout, full `Config`,
public API, exit codes, schema plan, and the provenance of every ported
algorithm). Read **`AGENTS.md`** for how work is actually done here: it is the
cross-agent authority (Claude Code, Codex CLI, Cursor, Copilot, via the
AGENTS.md standard), it holds the session protocol `/prime` executes, and its
"Working the backlog" section is the operating loop. **The gate chain is
defined there, not here**, and `harness_skills.rs` asserts every skill's inlined
gate floor is a subset of that list, so a step added to `AGENTS.md` reaches
every skill. The executable form of that list is the root `Makefile`'s `gate`
target (spec 094); `.claude/` itself is spec 093's.

## Commands

The toolchain is pinned in `rust-toolchain.toml` (channel `1.92.0`); MSRV is
`1.85` / edition 2024. Always pass `--locked`; CI does, and the committed
`Cargo.lock` is part of the determinism contract (tree-sitter grammars are
pinned exact).

```sh
cargo build --workspace --locked
cargo test  --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check

# A single test / one test file:
cargo test -p spec-spine-core --test couple            # one integration test file
cargo test -p spec-spine-core compile::                # tests matching a path
cargo test --workspace emitted_registry_conforms       # one test by name

# tree-sitter sits behind the default-on `symbol-resolution` feature (spec 025).
# CI builds and tests without it, and asserts the grammars are absent:
cargo test -p spec-spine-core --no-default-features --locked
```

The governed loop runs through the **in-tree binary** (`target/release/spec-spine`,
or `cargo run -p spec-spine-cli --`), never `npx spec-spine`: the npm and PyPI
distributions are for adopters.

```sh
cargo build --release -p spec-spine-cli                # what the harness drives
```

**Run the gate from `AGENTS.md`'s fenced list, not from here.** That block is
the one spelling, flags included, and `harness_skills.rs` asserts CI's flags are
a subset of it, so the two cannot drift. The executable form of that list is the
root `Makefile`'s `gate` target, which CI calls rather than restates (spec 092
§3.6). Nothing tests a copy in this file, which
is why there is no longer one: an earlier revision restated the chain and
silently dropped `--fail-on-unresolved` and `--fail-on-warn` from `check` and
`--fail-on-untraced` from `index coverage`, leaving three refusals off a list
that still read like the gate.

What the individual verbs are, as a reference rather than a sequence:

```sh
spec-spine compile          # -> .statecraft/derived/spec-registry/by-spec/<id>.json shards
spec-spine index            # -> .statecraft/derived/codebase-index/{by-spec,by-package}/ shards
spec-spine check            # BOTH freshness reads in one verb; never writes
spec-spine lint             # corpus conformance (L- codes)
spec-spine index coverage   # which source files no spec specifically claims
spec-spine couple           # the PR-time drift gate
spec-spine registry plan    # the ready set: workable now vs blocked
spec-spine index diagnostics  # unresolved-unit W-001 / W-002
spec-spine index owner <path> # which specs own one path, and how (spec 048)
spec-spine verify <id>      # run a spec's `## Verification` block
```

The `--fail-on-*` flags are what turn a read into a refusal: `lint
--fail-on-warn`, `check --fail-on-unresolved --fail-on-warn`, `index coverage
--fail-on-untraced`. Bare `check` exits 1 on a stale tree or a validation
failure (the report lines say which); what the flags add are the
unresolved-unit and warning-tier refusals. Bare `index coverage` refuses nothing at all.

Exit codes are the family contract shared with Statecraft (spec 132): `0` ok ·
`1` finding (validation, drift, stale, unresolved, not found, authored content
that does not parse) · `2` refused (invalid config, a version pin not met, a
containment refusal; nothing done) · `3` usage · `4` failed (I/O, internal, a
tool-produced artifact that fails its schema). They are mapped in exactly one
place (`Error::exit_code()` in `spec-spine-types`, applied by
`crates/spec-spine-cli/src/main.rs`). Verdict-rendering verbs take `--json` (spec 034), which
changes what is written and never what is decided.

**Ask `spec-spine --version` before believing any exit code.** The binary in
`target/release/` is whatever was last built, not necessarily this checkout, and
a binary predating a flag cannot report on it. `spec-spine.toml` sets
`[meta] required_version` to this repository's own package version (spec 061
§3.8, raised with each bump by spec 124) as a floor for exactly this failure;
`scripts/bump_version.py` deliberately does not touch that key. The version
string alone does not identify a build: `scripts/reader-identity.sh <binary>
[<checkout>]` records its digest, its checkout's revision and the schema axes
it actually emits.

## Architecture

Three crates, strict one-directional dependency `types → core → cli`:

- **`spec-spine-types`**, the plain-data substrate: `Config` (the
  `spec-spine.toml` model), the frontmatter grammar, the typed-edge and
  authority-unit vocabulary, registry/index/verdict/attestation DTOs,
  schema-version `const`s, the embedded JSON Schemas (`schemas/*.schema.json`,
  `include_str!`'d so the crate is self-contained), and the `Error` enum.
  Everything is owned, serde, with no lifetimes/generics/trait-objects at the
  boundary, so the same types back both the engine and future FFI bindings.
- **`spec-spine-core`**, the engine. One module per capability (`compile`,
  `index`, `query`, `lint`, `couple`, `coverage`, `verify`, `attest`,
  `diagnostics`, `render`, `scaffold`) plus internal `canonical_json` / `hash` /
  `markdown` / `sections` / `symbols` / `shard` / `manifest` / `pathutil` /
  `dep_only`. **The library API is the stable surface bindings wrap, not the
  CLI.**
- **`spec-spine-cli`**, a thin clap wrapper. One `cmd_*.rs` per subcommand.

**Invariants that shape every change to core (do not violate without updating
the design doc):**

- Every artifact-producing function is a **pure function of `(Config, file
  contents)`**: no ambient clock, no env reads, **no `git`**. The CLI parses
  `git diff` and passes a typed `DiffInput` in; the library never shells out.
  The only wall-clock value (`build-meta.json`'s `builtAt`) is written by the
  CLI and excluded from determinism/golden tests.
- **`unsafe` is `forbid`-en** workspace-wide (`Cargo.toml [workspace.lints]`).
- Core is IO-light and panic-free on user input: malformed config/frontmatter
  yields a clean `Error`, never a panic. The governance **scaffold** returns
  files-as-data (`Scaffold`); the consumer writes them. Since spec 092 that
  consumer is the Statecraft CLI, not a `spec-spine init` command: there is no
  `init` verb, and `scaffold_init_json` is a pure producer of governance
  starter content (config, constitution, contract, templates, bootstrap spec,
  a `.gitignore` fragment) and nothing else.
- The **JSON-in/JSON-out facade** in `core/src/lib.rs` (`compile_json`,
  `query_json`, `couple_json`, …) is the FFI seam. Keep it `&str → Result<String,
  Error>` and additive.
- **`verify` is the one verb that executes** rather than reads, so it is
  deliberately outside the gate chain: the chain runs against PR branches whose
  contents are, in the general case, a stranger's.

### The authority model

Each `specs/NNN-slug/spec.md` declares, in YAML frontmatter, **typed edges** to
other specs and the **authority units** it owns. Eight edges; `references` is
the only non-owning one (the coupling gate ignores it): `establishes`,
`extends`, `refines`, `supersedes`, `amends`, `co_authority`, `constrains`,
`references`. `origin.retroactive` is a bootstrap marker, not an edge. Units
resolve to code as `file` (bare string = file shorthand; trailing `/` =
subtree), `section` (`{file, anchor}`), or `symbol` (`{id}`, resolved by
tree-sitter, Rust + TS in v1; Python deferred). `directory`/`crate`/`module`
kinds shipped in spec 016. A unit may carry `planned: true` (spec 063) to
declare territory before it is written.

Two lifecycle axes, independent: `status` (draft / approved / superseded /
retired) and `implementation` (pending / in-progress / complete / n-a /
deferred). The ratify PR flips only the first.

Two views, joined by the gate: `compile` emits the spec-as-source registry;
`index` emits the code-as-source index. Since spec 022 both are stored
**sharded** (one committed file per authority unit); the aggregate view is
recomputed from the shard set on read, never committed. `index coverage`
(spec 029) reports, per source file, whether a spec specifically claims it, only
a manifest floor covers it, or nothing does; `[coupling] require_ownership` is
**on** in this repo, so an unclaimed changed source file is a `C-002` refusal,
and CI runs `index coverage --fail-on-untraced`. The coupling clearance
algorithm (amends-awareness, the strict-expansion guard, waiver parsing, bypass
matching) is ported behaviorally intact from OAP. Modules cite their source;
preserve the cited semantics when editing `couple.rs`.

### How a file gets claimed (three ways, and a common misreading)

1. **`establishes`** in a spec's frontmatter: the direct claim.
2. **An `extends` edge carrying a unit.** `extends` is defined in `edges.rs` as
   *"adds surface to a predecessor"*. The unit does **not** need to appear in
   the target spec's territory, and often does not: a large minority of
   unit-carrying `extends` edges name a target that never established the unit.
   A quarter of tracked source files have no establisher anywhere, most of the
   `spec-spine-types` crate among them, yet `index coverage --fail-on-untraced`
   passes. An extends-carried unit is a first-class claim.
   **Do not "fix" these.** A validation requiring an `extends` unit to appear in
   its target's territory would refuse the repository's ownership model.
3. **A `// Spec: specs/<id>/spec.md` comment header** in the **first 16 lines**
   of the file, for the extensions in `coverage.rs::SOURCE_EXTS` (rs, ts, tsx,
   js, jsx, go, py, sh). The scanner (`index.rs::scan_comment_headers`) strips
   one `//` or `#` marker and then requires `Spec:`, so `.py` and `.sh` claim
   with `#`, and `//!` does not claim at all (the `!` is left in the way). It
   claims exactly the file it sits in, never a subtree, and needs no
   frontmatter edit anywhere. This is how spec 029's coverage debt was retired
   without touching the tier-1 bootstrap spec.

## Determinism is the central claim

Emitted JSON is sorted-key, pretty-printed (2-space), LF, trailing newline.
Content hashes are SHA-256 over `<repo-relative-POSIX-path>\0<normalized-bytes>`
sorted by path, where normalization strips the BOM and converts CRLF/CR to LF
(`.gitattributes` also enforces LF on checkout). Each shard self-describes a
`shardHash`; the aggregate content hash is the fold of those.
`.github/workflows/determinism.yml` proves the registry + index **shard trees**
are **byte-identical across four release triples** (it folds every shard's path
and content into one tree digest; incl. tree-sitter symbol line-spans), not just
locally. If you change emission, expect that gate to be the real test.

## Self-governance (dogfood): why `.statecraft/derived/` is committed

This repo runs its own gates against its own corpus in CI (`.github/workflows/ci.yml`
`self_governance` job). Consequences:

- The `.statecraft/derived/spec-registry/by-spec/` and `.statecraft/derived/codebase-index/{by-spec,by-package}/`
  shard trees are **committed** (only `build-meta.json` is gitignored). After any
  change that affects them, regenerate and commit: `spec-spine compile` then
  `spec-spine index`. CI runs `check` rather than `compile`/`index`, because a
  gate must never repair the tree it is judging; it fails (exit 1, a `STALE`
  report line) if any committed shard is stale.
- **Editing a governance file restales every shard.** `[index] extra_hashed_inputs`
  in `spec-spine.toml` folds `AGENTS.md`, `CLAUDE.md`, `spec-spine.toml` itself,
  `Makefile`, the workflows and the embedded schemas into one
  global scalar. A one-line edit to any of them means regenerating and committing
  the whole index. Adding a pattern there is expensive and deliberate; the
  patterns are narrow on purpose (a bare `.claude/**/*` would fold in
  `.DS_Store` and make shard hashes machine-dependent).
- Editing code under a path owned by a spec generally requires also editing that
  spec's `spec.md` (or adding a `Spec-Drift-Waiver:` line to the PR body, which
  is a human instrument an agent never self-approves). The bypass floor (docs,
  lockfiles, `.derived/`, per `couple.rs::DEFAULT_BYPASS_PREFIXES`, plus the
  **configured** `[layout] derived_dir` which the gate adds to the floor itself
  (spec 092 §3.8, which is why `.statecraft/derived/` needs no config entry),
  extended by `spec-spine.toml [coupling] bypass_prefixes`) exempts non-code
  paths. A
  dependency-only manifest bump self-clears via `auto_waive_dependency_only`
  (specs 005/027), which is why Dependabot PRs are mergeable.
- **Merge conflicts on the committed artifacts** are rare since sharding (spec
  022): two PRs touching different specs write disjoint files. The opt-in
  per-clone merge driver handles the same-shard case:
  `./.githooks/enable-merge-driver.sh` (build the binary first). It is inert
  until registered and never replaces the staleness gate.

When working on a feature, add or amend the governing spec under `specs/` in the
same change. `standards/spec/` holds the constitution + contract + templates;
`specs/000-spec-spine-bootstrap` is tier-1 (its `unamendable` anchors are
non-overridable). New specs are filed as the next `NNN-slug` directory, born
`status: draft`.

**Never edit an approved spec to make your code pass.** The sanctioned move is
an `amends` edge declared in the *new* spec's frontmatter, which records the
change without touching the amended file (spec 037). See
`AGENTS.md` "Adversarial prompt refusal". Two edits are always legitimate
for the spec you are implementing: claiming a file you created, and recording a
dated decision the spec was silent on.

## The boundary with Statecraft (spec 092)

This repository ships a **governance engine**, not a development environment.
The Statecraft CLI is the sole distributor and initializer of the managed
development environment: project onboarding, the agent harness, the reusable
workflows and their per-agent delivery adapters are all its. spec-spine owns
compile, index, classify, couple, lint, verify, attest, and the governance
starter templates it produces **as data**.

Consequences for working here:

- There is **no `spec-spine init`** and no `--with-kit`. Do not add a verb that
  recreates them.
- The retained producer is `spec_spine_core::scaffold_init_json`, a pure
  function of its argument. It writes nothing, reads no environment, launches
  nothing, and emits no `AGENTS.md`, `CLAUDE.md`, `.claude/`, skills, agents,
  hooks, MCP configuration, CI workflow or `Makefile`. Keep it that way; the
  Statecraft CLI is implementing against it.
- `kit/`, `kit_embedded.rs`, the kit generators and the `.agents/` / `.codex/`
  projections are gone. `.claude/` is **this repository's own** development
  instruction, not a distribution source: it is what a session here loads, and
  it stays until Statecraft's global delivery concretely replaces it.
- This repository is governed under the managed layout: `[layout] derived_dir`
  is `.statecraft/derived` (committed) and `state_dir` is `.statecraft/state`
  (ignored). `.statecraft/` as a whole is neither; a file under it that is in
  neither root is ordinary governed territory.
- `docs/design/07-statecraft-realignment-2026-09.md` holds the boundary, the
  producer contract and the disposition of the earlier harness plans.
- **Spec ids moved.** Spec 095 collapsed 27 specs into three and renumbered the
  survivors so the ordinals are contiguous (000 through 095). A citation written
  before that, in git history, a merged pull request or an adopter repository,
  names an ordinal that has moved or a document that is gone.
  **`docs/corpus-map.md` is the map**, both directions. Read it before trusting
  a spec number you did not get from `registry list`.

## Schema & release versioning (two decoupled axes)

- **Schema versions** (`registry`, `index`, `verdict`, read documents,
  `build-meta`, `config` and the rest) are compile-time `const`s in
  `spec-spine-types/src/version.rs`; `docs/schema-versioning.md` lists each
  axis's current value and history. The conformance test
  (`core/tests/conformance.rs`) asserts emitted JSON validates against the
  embedded schema of that version: a DTO/schema drift fails the **build**.
  MINOR = additive only; MAJOR = breaking (loaders reject an unknown MAJOR). An
  additive registry field is a MINOR bump that restamps every shard's
  `specVersion` while leaving `shardHash` alone (the hash is over `spec.md`
  source bytes). See `docs/schema-versioning.md`.
- **Package version** lives in three files (`Cargo.toml`, `npm/package.json`,
  `py/pyproject.toml`) and must agree at release time. Bump them in lockstep with
  `scripts/bump_version.py <x.y.z>` (`--check` verifies agreement). This is
  **independent** of schema versions: a release can ship without a schema change.
  Maintainer release runbook: `docs/releasing.md`.

The npm (`npm/`) and PyPI (`py/`) directories are binary-distribution shims (specs
006/008): they ship the prebuilt binary per platform, assembled from release
archives at publish time. The platform packages and binaries are never committed
(`.gitignore`). Don't hand-edit generated platform packages.
