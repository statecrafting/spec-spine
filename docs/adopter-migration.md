# Adopter migration: the realignment release

**Status: prepared 2026-09-21, unreleased.** Nothing described here has shipped
in a tagged build yet. The latest release is `v0.21.0`; every change below is on
the default branch and will reach adopters in the next tag. This document is
written before that tag so the release notes can point at it rather than
restate it.

Read this if you pin `spec-spine` and upgrade past `v0.21.0`.

## 1. What changed, in one paragraph

spec-spine now ships a **governance engine and nothing else**. The project
initializer and the vendored agent harness are gone: no `spec-spine init`, no
`--with-kit`, no `kit/`. The library keeps one pure producer of governance
starter content, which the Statecraft CLI consumes. Spec ids were renumbered.
Nothing about compile, index, check, lint, couple, verify or attest changed in
this release for a repository that was not calling `init`.

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
ordinals are contiguous, `000` through `099` as of this writing.

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

- **`rahi` has no `[meta] required_version`.** Spec 055's floor check cannot
  fire, so a binary older than the corpus expects fails in whatever way the
  missing feature happens to fail, with nothing naming the cause. Setting a pin
  is one line.
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
6. Run your gate. Nothing in this release changes a verdict for a corpus that
   was not calling `init`.
