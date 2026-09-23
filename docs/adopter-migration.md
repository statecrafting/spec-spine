# Adopter migration: the realignment release

**Status: shipped in `v0.23.0` (2026-09-23).** This was written on 2026-09-21
for a `v0.22.0` tag that was never published; the 0.22.0 candidate was frozen
and the owner chose to publish `v0.23.0` directly after `v0.21.0`. Every change
below is in `v0.23.0`, which also carries the additive expansion line
(`docs/consumer-integration-expansion.md`). Where this page says "this
release", read `v0.23.0`.

Read this if you pin `spec-spine` and upgrade past `v0.21.0`.

**Two changes in this release can move a verdict**, so read §9 before you
upgrade a CI pipeline. Everything else is the realignment, which affects
creating a repository rather than judging one.

## 1. What changed, in one paragraph

spec-spine now ships a **governance engine and nothing else**. The project
initializer and the vendored agent harness are gone: no `spec-spine init`, no
`--with-kit`, no `kit/`. The library keeps one pure producer of governance
starter content, which the Statecraft CLI consumes. Spec ids were renumbered.
Two behavioral corrections ride along and are described in §9: the coupling
gate now judges a deleted path against the snapshot it lived in, and it refuses
rather than passing when it cannot read the history a verdict depends on.
Nothing else about compile, index, check, lint, couple, verify or attest
changed for a repository that was not calling `init`.

## 2. Removed, with what to do instead

| Gone | What it did | What to do |
|---|---|---|
| `spec-spine init` | created a corpus, config, standards and templates | See §3. There is deliberately no replacement verb in this binary |
| `spec-spine init --with-kit` | installed the agent harness into `.claude/` and friends | See §5 |
| `kit/` in this repository | the vendored harness source | Your existing copy keeps working. See §5 |
| `.agents/`, `.codex/` | generated projections of that harness | Your copies are yours; nothing here regenerates them |
| `website/` | the documentation site | `docs/adoption-guide.md` and `docs/api.md` carry what only existed there |

**There is no deprecation window and no shim.** That was a deliberate choice
(spec 092 D-1): the removed surface is an initializer, and two initializers with
different opinions about the same paths is a worse failure than an absent one.

**Nothing you already have stops working.** These removals affect creating a new
governed repository and installing a harness. An existing corpus, its committed
derived trees and every gate verb are unaffected.

## 3. The producer contract, which is preserved

The scaffolding capability was not deleted; it was reduced to a pure function
and frozen as a contract:

```
spec_spine_core::scaffold_init_json(config_json: &str) -> Result<String, Error>
```

It returns `{ "files": [ScaffoldFile] }` and it **writes nothing**: no
filesystem writes, no environment reads, no process launches, no network, no
timestamps. It emits governance starter content only: `spec-spine.toml`, the
constitution, the contract, the authoring templates, the bootstrap spec, and a
`.gitignore` fragment returned with `append: true` and an `appendMarker` for the
consumer to reconcile.

It emits no `AGENTS.md`, no `CLAUDE.md`, no `.claude/`, no skills, agents,
hooks or MCP configuration, no CI workflow and no `Makefile`.

If you automated `spec-spine init`, call this function (or its JSON facade) and
write the files yourself. That is the whole of the migration for that case.

## 4. Spec ids moved

Spec 095 collapsed 27 specs into three and renumbered the survivors so the
ordinals are contiguous. The corpus is `000` through `104` in this candidate.

**A bare pre-collapse ordinal now resolves to a different document**, which is
worse than a dangling one. `docs/corpus-map.md` is the map, both directions:
every removed id to the spec that answers for it, and every old ordinal to its
new one.

This matters to you if you cite spec-spine ordinals in your own specs, your
`CLAUDE.md`, your skills or your review comments. Citations in git history,
merged pull requests and release notes were deliberately not rewritten.

## 5. Your local harness copy: the supported interim

If you ran `init --with-kit`, you have a `.claude/` tree (skills, agents, rules,
`settings.json`) copied from a kit that no longer exists upstream.

**It keeps working, and it is yours.** Nothing in this release touches it, reads
it, or requires it. It is ordinary governed content in your repository: your
specs claim it, your `extra_hashed_inputs` hash it, and your gate judges it,
exactly as before.

**What you no longer get is upstream revisions of it.** There is no kit to
re-copy from and no verb that refreshes it. A fix that used to arrive as "adopt
the new kit revision" now has to be applied by hand, in your copy, or not at
all.

**The intended replacement is a globally delivered harness from the Statecraft
CLI.** Be careful about how much weight you put on that sentence today:

- It is **specified**, in `statecraft-cli` spec `002-environment-lifecycle`
  §§3.13, 3.14, 3.23: one content-addressed harness under the product home,
  adapters pointing at it, no copy in any repository, and delivery evaluated
  rather than assumed.
- That spec is `approved` with `implementation: in-progress`, and one of its
  sections is specified and not implemented.
- **No Statecraft release is claimed here, and no migration path is offered.**
  spec-spine's own repository has not been enrolled and still runs its own local
  harness, for exactly the reason you should: removing a local harness before
  its replacement is concretely available leaves a repository with no
  development loop and no hook enforcement.

So the supported interim is: **keep your copy, maintain it yourself, and change
nothing on the strength of an announcement.** When a Statecraft delivery is
available, its own documentation will say what enrolling costs.

## 6. The layout question

This repository moved its own compiled artifacts to `.statecraft/derived/` and
its ungoverned state root to `.statecraft/state/`, because it is governed under
Statecraft's managed layout.

**That is configuration, not a product default and not a deprecation.**
`derived_dir` still defaults to `.derived`, and a corpus that configures nothing
gets `.derived` exactly as before. Five of the six repositories that pin
spec-spine are on `.derived` today and none of them needs to move.

One improvement does reach you wherever your derived root is: the gate now adds
the **configured** `derived_dir` to its bypass floor itself. A repository whose
derived root was not `.derived` previously had every regenerated shard judged as
source, which made the change that recomputes the ledger a `C-001` refusal and,
under `require_ownership`, a `C-002` unclaimed file as well. If you configured a
non-default derived root and worked around that, the workaround is no longer
needed.

## 7. Who this applies to, measured

Six repositories pin spec-spine, read 2026-09-21:

| Repository | Pin | `derived_dir` | Local harness copy |
|---|---|---|---|
| `hqgit` | `>=0.18.0`, workflow `v0.18.0` | `.derived` | yes |
| `aicortex` | `>=0.20.0`, workflow `v0.20.0` | `.derived` | yes |
| `rahi` | **no `required_version`**; `Makefile` default `0.20.0` | `.derived` | yes |
| `claude-observatory` | `0.15.0`, archived | `.derived` | archived |
| `statecraft` | `=0.20.0` | `.derived` | |
| `statecraft-cli` | `=0.20.0` | `.statecraft/derived` | |

Two things worth acting on independently of this release:

- **`rahi` has no `[meta] required_version`.** The `0.20.0` in its `Makefile`
  is a **tool selection**: it decides which binary the local loop downloads or
  runs. `[meta] required_version` is a **floor**: it decides which binaries the
  corpus consents to be governed by, and it is checked on every run, whoever
  invoked the binary and however they obtained it. The two answer different
  questions and neither substitutes for the other: a developer with an older
  binary on `PATH`, a CI job that resolves the tool some other way, and a
  consumer calling the library directly all bypass a `Makefile` and none of
  them bypasses the floor. Without the floor, a binary older than the corpus
  expects fails in whatever way the missing feature happens to fail, with
  nothing naming the cause. Setting it is one line.
- **`claude-observatory` is three releases behind and archived.** It is not
  covered by this note; treat it as out of support rather than as an adopter to
  migrate.

## 8. Upgrade checklist

1. Read `docs/corpus-map.md` if you cite spec-spine ordinals anywhere.
2. Search your automation for `spec-spine init` and `--with-kit`. If you find
   either, §3 is your replacement.
3. Set `[meta] required_version` if you have not (spec 055).
4. Leave `derived_dir` alone unless you have your own reason to move it (§6).
5. Keep your `.claude/` copy and plan to maintain it yourself (§5).
6. Read §9 and check your CI clone depth before upgrading a pipeline.
7. Run your gate.

## 9. The two verdict-affecting changes

These are the only changes in this release that can make your gate answer
differently on an unchanged repository. Both concern **deletions**.

### 9.1 A deleted path is judged against the snapshot it lived in (spec 100)

**What was wrong.** A change that deleted a file and withdrew its claim in the
owning spec's own frontmatter, in one commit, was refused `C-001`. The gate
resolved the deleted path's owners from the head index, where the claim was
already gone, so the answer fell through to the package manifest floor, which
had nothing true to say about a file it never claimed. The remedies the refusal
printed did not work: editing the floor spec would have written something untrue
into an approved document, and an `extends` edge naming a vanished path raises
`I-004` on the next `index`.

**What happens now.** A deleted path resolves its owners against the snapshot
preceding the segment that recorded the deletion: the merge base for the
committed range, `HEAD` for the working-tree diff under
`--include-uncommitted`. Additions and modifications are untouched and still
resolve at head.

**What this means for you.** Some `C-001` refusals on removals will stop. No
new refusal is introduced by this half: an owner surviving at the prior
snapshot still refuses, a floor-only deletion still refuses, and a co-owner
that was never edited is still named.

The snapshot is **reconstructed** by compiling and indexing that commit's
exported tree. It is not read from that commit's committed derived shards, so a
stale or missing historical ledger cannot affect your verdict, and a
repository that never committed its derived tree is unaffected.

### 9.2 History the gate cannot read is a refusal, not a pass (spec 100 §3.5)

**Check your CI clone depth before upgrading.** This is the one change that can
turn a passing job red.

The gate reads history: its diff is a three-dot range, and a deleted path needs
the snapshot it lived in. When that history cannot be read, the gate now exits
`3` with a message saying it judged nothing and naming the remedy. It does not
substitute another revision, does not ignore a historical corpus that fails
validation, and does not treat an unreadable range as an empty diff.

Exit `3` is the code spec 005 already assigns to an IO or parse failure, so no
new exit code is introduced.

**Who is affected.** A job whose checkout cannot reach the merge base. In
practice that is `actions/checkout` left at its default `fetch-depth: 1`. If
your gate job already sets `fetch-depth: 0`, nothing changes for you.

```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0   # the gate reads history; a depth-1 clone cannot
```

**Why this is not a fallback.** The previous behavior in these states would
have been to answer from the head tree alone, which makes a gate's verdict
depend on clone depth with nothing on the screen to say so. A gate that reports
a pass it did not compute is worse than one that says it could not judge.

**The cost is bounded by laziness.** A snapshot is built only when the change
actually deletes something, so a deletion-free pull request pays nothing and
still runs in a shallow clone.

### 9.3 The verdict envelope is `0.5.0`

`couple`'s `--json` report gained a `deletions` block naming, per deleted path,
which snapshot resolved its owners (`merge-base`, `head-commit` or `head-tree`)
and whether the path was absent from it. The block is **omitted when empty**,
so every input that produced a verdict before this release still produces the
same payload bytes. Only the envelope's `schemaVersion` string moves, from
`0.4.0` to `0.5.0`, which is the additive MINOR rule
[`docs/schema-versioning.md`](schema-versioning.md) states. A consumer pinning
`0.x` on the MAJOR is unaffected; a consumer asserting the exact string needs
one edit.

### 9.4 The library API is additive

`couple_with` and `couple_with_scope` keep their exact signatures and their
exact behavior. They hold one snapshot and therefore resolve deletions at head,
which is the old behavior, retained deliberately: an overlay or binding calling
them sees no change at all.

They do **not** carry the correction, and the report labels every deletion they
judged `head-tree` so that is visible rather than assumed. To get §9.1's
behavior from the library, call `couple` (or `couple_snapshots`,
`couple_with_prior`), or pass `priorRoots` to `couple_json`. See
[`docs/api.md`](api.md).
