---
id: "196-yaml-parsing-moves-off-unsafe-libyaml"
title: "YAML parsing moves off unsafe-libyaml"
status: draft
kind: "tooling"
created: "2026-10-12"
summary: >
  Every YAML read in the engine (spec frontmatter, compaction plans, the
  delta classifier, the dependency-only waiver, workflow manifests) goes
  through `serde_yaml` 0.9, which is deprecated and parses with
  `unsafe-libyaml`, a c2rust translation of libyaml that RUSTSEC-2024-0449
  marks unmaintained. `deny.toml` ignores that advisory by id so `cargo-deny`
  can pass; this spec removes the reason for the ignore. The engine moves to a
  YAML crate written in safe Rust, the committed registry and index shard trees
  stay byte-identical, every refusal the current parser produces on authored
  frontmatter is kept, and the ignore entry is deleted in the same change.
implementation: pending
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "135-the-toolchain-floor-and-dependencies-are-checked"
establishes:
  # 3.3: the refusal and acceptance parity table, run against both parsers.
  - { kind: file, path: "crates/spec-spine-types/tests/yaml_parity.rs", planned: true }
  # 3.4: workflow parity for the dependency-only waiver and the manifest projection.
  - { kind: file, path: "crates/spec-spine-core/tests/yaml_workflow_parity.rs", planned: true }
extends:
  # The frontmatter reader: the parse every registry shard is built from.
  - { spec: "012-declared-extra-frontmatter-passthrough", unit: "crates/spec-spine-types/src/frontmatter.rs", nature: corrective }
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: "crates/spec-spine-core/src/compact.rs", nature: corrective }
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-core/src/delta.rs", nature: corrective }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/dep_only.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/manifest.rs", nature: corrective }
  # Tests that call the old crate directly, on the spec that owns each file.
  - { spec: "111-a-move-is-a-reviewed-mapping", unit: "crates/spec-spine-types/src/moves.rs", nature: corrective }
  - { spec: "109-impact-and-conflict-are-declared", unit: "crates/spec-spine-types/src/impact.rs", nature: corrective }
  - { spec: "013-edge-paths-grammar-sugar", unit: "crates/spec-spine-types/tests/grammar.rs", nature: corrective }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: "crates/spec-spine-core/tests/gate.rs", nature: corrective }
  - { spec: "091-an-unclassified-review-failure-blocks-the-merge", unit: "crates/spec-spine-core/tests/ai_review_policy.rs", nature: corrective }
  # The dependency swap.
  - { spec: "135-the-toolchain-floor-and-dependencies-are-checked", paths: ["crates/spec-spine-types/Cargo.toml", "crates/spec-spine-core/Cargo.toml"] }
references:
  - { unit: { kind: file, path: "deny.toml" }, role: context }
---

# 196: YAML parsing moves off unsafe-libyaml

## 1. Purpose

### 1.1 The parser is unmaintained

`serde_yaml` 0.9.34 is published as `+deprecated`, and its parser is
`unsafe-libyaml` 0.2.11, a machine translation of libyaml's C into unsafe
Rust. Its repository was archived on 2024-03-25. RUSTSEC-2024-0449 records it
as unmaintained and names no safe upgrade. The advisory began failing
`cargo-deny` on 2026-10-12, which fails `ci-gate` on every pull request.

`deny.toml` now ignores RUSTSEC-2024-0449 by id. Spec 135 §3 fails the check
on vulnerability and unsoundness notices, and an unmaintained notice is
neither, so the ignore stays within that spec. What it leaves is a parser
nobody fixes, sitting under the one input every hashed artifact is derived
from. The workspace forbids `unsafe` in its own code, and its most exposed
parse, of authored text from any pull request, runs through unsafe code no one
maintains.

### 1.2 Where YAML is read

| site | what it parses |
|---|---|
| `crates/spec-spine-types/src/frontmatter.rs` | every spec's frontmatter, then the typed `Frontmatter` and the declared extra keys |
| `crates/spec-spine-core/src/compact.rs` | a compaction plan |
| `crates/spec-spine-core/src/delta.rs` | a spec's frontmatter at base and head, for classification |
| `crates/spec-spine-core/src/dep_only.rs` | a workflow at base and head, compared except for `uses:` refs |
| `crates/spec-spine-core/src/manifest.rs` | workflow manifests, projected to JSON for hashing |

Five more files call `serde_yaml` directly in tests: the unit tests inside
`crates/spec-spine-types/src/moves.rs` and `crates/spec-spine-types/src/impact.rs`,
and the integration tests `crates/spec-spine-types/tests/grammar.rs`,
`crates/spec-spine-core/tests/gate.rs` and
`crates/spec-spine-core/tests/ai_review_policy.rs`.

## 2. Territory

The five engine files are `extends` crossings on their owners, and the two
crate manifests are `extends` crossings on 135, which set the dependency
policy. The root `Cargo.toml` (the `[workspace.dependencies]` line) is claimed
by no spec and hashed by none; claiming it would oblige an
`extra_hashed_inputs` glob (L-008), so it stays unclaimed, as it is today.
`deny.toml` is referenced rather than claimed, as in 135: a claimed file must
be hashed (L-008), and a policy edit should not restamp every shard. The five files
whose tests call the old crate are `extends` crossings on the spec that
establishes each (111, 109, 094, 091); `grammar.rs` has no establisher, so its
crossing is on 013, the edge-grammar spec that first extended it.

## 3. Behavior

### 3.1 The parser

The workspace MUST NOT depend on `serde_yaml` or `unsafe-libyaml`, directly or
transitively, in any feature combination (`[graph] all-features = true`). The
replacement MUST:

- be written in safe Rust, with no C or c2rust parser below it;
- build at the declared `rust-version` with `--locked`;
- carry a license on `deny.toml`'s allow-list, or a change to that list that
  is argued in this spec's decisions;
- add no `cargo-deny` advisory.

The choice of crate is made at build and recorded as a decision.
`serde-saphyr`, which RUSTSEC-2024-0449 itself suggests, is the first
candidate.

### 3.2 The registry and index do not move

After the swap, `spec-spine check` MUST report both trees fresh against the
shards committed before it, with no shard regenerated. The same is true across
the four determinism triples: `determinism.yml` passes unchanged.

### 3.3 Every refusal is kept

Frontmatter the current parser refuses MUST stay refused, with the same
diagnostic code and exit code, and frontmatter it accepts MUST compile to the
same record. The build adds a fixture table,
`crates/spec-spine-types/tests/yaml_parity.rs`, run against both parsers before
the old one is removed and against the new one after, covering at least:

- duplicate mapping keys;
- the YAML 1.1 words (`yes`, `no`, `on`, `off`, `y`, `n`) as keys and values;
- `~`, `null`, and an empty value;
- numbers that differ between YAML 1.1 and 1.2 (`0o17`, `017`, `1e3`, `.inf`,
  `.nan`, `+1`, `1_000`);
- anchors, aliases, and merge keys (`<<`);
- explicit tags (`!!str`, `!custom`), which `yaml_to_extra` refuses today;
- a second document in the frontmatter block;
- non-ASCII keys and values, and a byte-order mark.

Where the two parsers disagree on a case no corpus file uses, the build
decides which behavior to keep and records the decision, rather than calling
the difference parity.

### 3.4 Workflows

`dep_only.rs` and `manifest.rs` MUST treat a GitHub Actions workflow exactly
as before: the dependency-only waiver clears the same pull requests, and the
hashed projection of each manifest is unchanged.
`crates/spec-spine-core/tests/yaml_workflow_parity.rs` pins both over every
workflow in `.github/workflows/` and over fixtures for these cases:

- `on:` as a top-level key, in its scalar, sequence and mapping forms. YAML
  1.1 reads the bare word as boolean `true`; this is the known hazard, and
  `tests/gate.rs` already reads it both ways.
- `uses:` references pinned to a full commit SHA, to a tag, and to a branch,
  with and without a trailing `# vX.Y.Z` comment.
- Anchors, aliases and merge keys (`<<: *defaults`) inside a workflow.
- Multi-line `run:` blocks in literal (`|`), folded (`>`) and chomping
  (`|-`, `>+`) forms.

### 3.5 The ignore goes

The same change MUST delete the RUSTSEC-2024-0449 entry and its comment from
`deny.toml`, and `cargo deny check` MUST pass without it.

## 4. Out of scope

**Writing YAML.** The engine emits JSON only. A test that renders YAML
(`ai_review_policy.rs`) may move to any serializer, or compare parsed values
instead.

**A YAML dialect change.** Frontmatter does not move to YAML 1.2 semantics for
cases the corpus never uses unless a decision under §3.3 says so.

## 5. Resolved decisions

None yet.

## Verification

```verify:cli
# 3.1: neither crate is in the locked tree, no manifest depends on the old
# one, and no Rust line outside a `//` comment names it, so a comment line
# may still mention the old crate.
! grep -q '^name = "serde_yaml"$' Cargo.lock
! grep -q '^name = "unsafe-libyaml"$' Cargo.lock
! grep -q 'serde_yaml' Cargo.toml crates/*/Cargo.toml
! grep -rh --include='*.rs' 'serde_yaml' crates | grep -qvE '^[[:space:]]*//'
# 3.5: the ignore entry is gone.
! grep -q 'RUSTSEC-2024-0449' deny.toml
# 3.2: the committed trees are fresh under the new parser.
cargo build --release --locked -p spec-spine-cli
target/release/spec-spine check
# 3.3 and 3.4: the parity tables run and pass. The build records each
# target's exact test count here (spec 153).
cargo test -p spec-spine-types --locked --test yaml_parity
cargo test -p spec-spine-core --locked --test yaml_workflow_parity
```
