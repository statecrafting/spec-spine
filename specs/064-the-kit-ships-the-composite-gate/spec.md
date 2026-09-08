---
id: "064-the-kit-ships-the-composite-gate"
title: "The kit ships the composite gate and the merge driver"
status: approved
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "020-derived-artifact-merge-driver"
  - "029-claude-code-skill-kit"
  - "024-index-sharding"
  - "051-harness-runs-the-verbs-it-ships"
extends:
  - { spec: "029-claude-code-skill-kit", unit: "kit/", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/README.md", nature: additive }
  - { spec: "020-derived-artifact-merge-driver", unit: ".githooks/", nature: additive }
  # This repository adopts the kit's Makefile as its own CI entry point (3.4),
  # and the new artifacts join the hashed-input set so a change to the gate
  # adopters copy stales the ledger (spec 057).
  - { spec: "021-release-supply-chain-artifacts", unit: ".github/workflows/ci.yml", nature: additive }
  - { spec: "057-claimed-but-unwitnessed", unit: "spec-spine.toml", nature: additive }
establishes:
  # Created by this spec (2), so claimed by it.
  - "kit/Makefile"
  - "kit/govern.yml"
  - "kit/.gitattributes-stanza"
  - "kit/.githooks/enable-merge-driver.sh"
  - "kit/.githooks/merge-derived-index.sh"
  - "crates/spec-spine-core/tests/kit_gate.rs"
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: ".github/workflows/ci.yml" }, role: exemplar }
summary: >
  The kit ships a session harness and no build. Every adopter therefore wrote
  the same two things by hand. First a composite gate: a Makefile whose language
  targets are guarded on a manifest probe so the whole thing is green on a
  code-free tree, and a CI workflow carrying the `has_cargo` job-output pattern
  that three adopters each rediscovered after GitHub rejected `hashFiles` in a
  job-level `if`. Second, nothing at all for the committed shard trees: rahi
  runs one spec per PR with committed shards and has no merge driver, and the
  kit's README never says the words. This spec ships `kit/Makefile`,
  `kit/govern.yml`, `kit/.githooks/` and the `.gitattributes` stanza, and holds
  them to the rule spec 046 and 051 established for everything else in the kit:
  what the kit ships, this repository runs.
---

# 064: The kit ships the composite gate and the merge driver

## 1. Purpose

`kit/` is a session harness: rules, agents, skills, hooks, an `AGENTS.md`. It
tells an agent how to work in a governed repository. It says nothing about how
that repository is built or how its artifacts are merged, and adopters found out
that those are not optional.

**The composite gate.** Every adopter reinvented it, and each hit the same two
problems. A gate that runs `cargo test` fails on a corpus with no Rust, which is
the state three of the four repositories are in, so each guarded its language
targets on a manifest probe. And each then discovered that GitHub rejects
`hashFiles` in a job-level `if`, so each arrived at the same workaround: a
probe job emitting `has_cargo` as a job output that later jobs condition on.
hqgit hit it, then rahi, then aicortex. Three repositories paying the same
half-day for the same undocumented platform behavior is the audit's clearest
signal of a missing artifact.

**The merge driver.** Spec 020 built one and spec 024 narrowed the case it is
for: since sharding, two PRs touching different specs write disjoint files, and
the driver remains for the rare same-shard conflict. It lives in `.githooks/`
with a `.gitattributes` stanza and a per-clone enable script. None of it is in
`kit/`, and `kit/README.md` never uses the words "merge driver". rahi runs one
spec per PR with committed shards, which is precisely the workflow the driver
exists for, and has none of it.

The rule this repository has already adopted twice applies here. Spec 046 found
three kit hooks that wrote when they should read, and the reason they shipped
that way was that this repository had no `.claude/settings.json` and never
exercised them. Spec 051 made the harness run the verbs it ships. A kit
`Makefile` this repository does not use would be a third instance of the same
mistake, so this spec does not ship one unexercised.

These are items 1 and 2 of the adopter audit's ranked backlog for the kit.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `kit/` | 029 | four new files under it |
| `kit/README.md` | 029 | the composite gate and the driver, documented |
| `.githooks/` | 020 | the scripts become the kit's source |

The implementing change creates `kit/Makefile`, `kit/govern.yml`,
`kit/.githooks/` and `kit/.gitattributes-stanza`, and claims them in this spec,
which is one of the two always-legitimate mid-build edits. A new
`crates/spec-spine-core/tests/kit_gate.rs` is created and claimed the same way.

## 3. Behavior

### 3.1 `kit/Makefile`

The kit MUST ship a `Makefile` with the composite targets every adopter wrote:

- `gate`: the governed loop in order, `compile --check`, `index check`,
  `lint --fail-on-warn`, `couple`. Read-only throughout, because a gate that
  writes repairs what it is meant to judge (spec 046).
- `refresh`: the writing half, `compile` then `index`, for the session that has
  edited a spec and can commit the result.
- `test`, `build`, `fmt`, `clippy`: language targets, each **guarded on a
  manifest probe** so they are no-ops on a tree that has no such manifest.
- `verify SPEC=<id>`: `spec-spine verify <id>` (spec 049).

Two variables, both overridable, both with the defaults the adopters converged
on: `SPEC_SPINE ?= spec-spine` and `BASE ?= origin/main`. The first is what lets
a repository that builds its own binary point at it, which spec 051 established
is the correct resolution order for a self-governing repository.

The guard MUST be a file probe (`test -f Cargo.toml`), not a command probe. A
tree with `cargo` installed and no `Cargo.toml` is the specify-first case, and
probing for the tool answers the wrong question.

### 3.2 `kit/govern.yml`

The kit MUST ship a CI workflow carrying the `has_cargo` job-output pattern:

```yaml
jobs:
  probe:
    outputs:
      has_cargo: ${{ steps.p.outputs.has_cargo }}
    steps:
      - id: p
        run: test -f Cargo.toml && echo "has_cargo=true" >> "$GITHUB_OUTPUT" || echo "has_cargo=false" >> "$GITHUB_OUTPUT"
  build:
    needs: probe
    if: needs.probe.outputs.has_cargo == 'true'
```

with a comment stating why: **GitHub rejects `hashFiles` in a job-level `if`**,
so the probe must be a job whose output later jobs condition on. That sentence
is the artifact. Three adopters spent time rediscovering it, and a workflow that
carries the pattern without the reason will be "simplified" back into the broken
form by the next person who reads it.

The workflow MUST write the PR body to a file under `$RUNNER_TEMP` and pass it
with `--pr-body`, which is how a body containing a waiver line reaches the gate
without shell quoting hazards. It MUST run `couple` only on `pull_request`, since
the gate compares a base to a head.

### 3.3 The merge driver moves into the kit

`kit/.githooks/` MUST carry the driver and the enable script, and the kit MUST
carry the `.gitattributes` stanza registering it on the shard globs.

The kit's copy is the **source**, and this repository's `.githooks/` is
generated from it or asserted equal to it. One direction, tested, because two
hand-maintained copies of a shell script that regenerates committed artifacts is
how they diverge silently.

`kit/README.md` MUST say "merge driver", say that it is opt-in per clone, say
that sharding (spec 024) already removes the common conflict so the driver is for
the same-shard case, and say that it never replaces the `index check` staleness
gate. rahi's situation is the test of this text: an adopter running one spec per
PR with committed shards should be able to read the README and know whether they
need it.

### 3.4 This repository runs what the kit ships

`tests/kit_gate.rs` MUST assert, at compile time against the checked-in files,
that:

- every `spec-spine` invocation in `kit/Makefile` and `kit/govern.yml` is a verb
  this binary has, and every gate-path invocation is read-only, which is the
  same assertion `tests/kit_hooks.rs` makes for the hooks (spec 046);
- `kit/.githooks/` and this repository's `.githooks/` agree;
- the gate chain in `kit/Makefile` is the chain `AGENTS.md` lists, in order,
  which is the assertion spec 051 established for the skills.

**Decision, 2026-09-07: the chain's order is AGENTS.md's, not the summary's.**
§3.1 lists the gate as "`compile --check`, `index check`, `lint --fail-on-warn`,
`couple`", and the first implementation followed that. The test in §3.4 refused
it: `AGENTS.md` runs `lint --fail-on-warn` **before** `index check
--fail-on-unresolved`, and it also carries `index coverage --fail-on-untraced`,
which §3.1's four-item summary omits. The Makefile follows `AGENTS.md`, which is
the authority §3.4 names, and §3.1's list is read as naming the verbs rather
than fixing their order. That the assertion caught a summary written two
paragraphs above it is the argument for having it.

And this repository MUST adopt `kit/Makefile` itself, as the entry point its own
CI and its own skills call. That is the point of the test and the lesson of
specs 046 and 051: the kit's artifacts are exercised here or they ship broken.

The existing `.github/workflows/ci.yml` keeps its structure. It is a
self-governing repository's workflow with a determinism matrix and release
concerns that a generic `govern.yml` should not carry; what changes is that its
gate steps call the Makefile targets rather than open-coding the same commands,
so the chain has one definition.

## 4. Out of scope

**Replacing `ci.yml`.** §3.4. `govern.yml` is the adopter's starting point, not
this repository's workflow.

**A non-GitHub CI template.** The `has_cargo` finding is GitHub-specific and so
is the artifact. An adopter on other CI takes the Makefile, which is portable,
and writes their own invocation.

**Enabling the merge driver anywhere.** It stays opt-in per clone, exactly as
spec 020 left it. Shipping it in the kit makes it available; it does not
register it.

**Language targets beyond Rust and npm.** The manifest probes cover what the
indexer discovers. A Python or Go target would be guessing at a toolchain the
tool does not model.

## 5. Verification

Each line is one command (spec 049 §3.2). Every assertion fails against
pre-064 state: none of the four artifacts existed.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test kit_gate --locked
# 3.1-3.3: the kit ships all four artifacts.
test -f kit/Makefile
test -f kit/govern.yml
test -d kit/.githooks
test -f kit/.gitattributes-stanza
# 3.2: the workflow carries the finding, not just the pattern, so the next
# reader does not simplify it back into the form GitHub rejects.
grep -q 'hashFiles' kit/govern.yml
grep -q 'has_cargo' kit/govern.yml
# 3.1: the language targets probe for a manifest, not for a tool.
grep -q 'test -f Cargo.toml' kit/Makefile
# 3.3: the README answers rahi's question about the driver.
grep -q 'merge driver' kit/README.md
grep -q 'opt-in per clone' kit/README.md
# 3.4: this repository runs the gate it ships, read-only, and the tree it
# judged is unchanged afterwards.
make -f kit/Makefile gate SPEC_SPINE=target/release/spec-spine BASE=HEAD
target/release/spec-spine compile --check
target/release/spec-spine index check
# 3.4: and its CI calls that same target rather than open-coding the chain.
grep -q 'make -f kit/Makefile gate' .github/workflows/ci.yml
```
