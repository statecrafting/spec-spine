---
id: statecraft
title: Statecraft and spec-spine
sidebar_position: 6
---

# Statecraft and spec-spine

spec-spine is a governance engine. It compiles a markdown spec corpus into a
typed, hash-verifiable ledger, indexes the code against it, classifies a diff
and refuses drift. It does not set up your repository, and it does not ship a
development environment.

Setting up a project is the Statecraft CLI's job. Statecraft is the sole
distributor and initializer of the managed development environment: project
onboarding, the agent harness, the reusable workflows and the per-agent
delivery adapters that put those workflows in front of whichever agent runtime
you use. Statecraft calls spec-spine; it does not reimplement it.

## Who owns what

| Owner | Surface |
|---|---|
| Statecraft CLI | project onboarding and initialization, the managed development environment, the agent harness, reusable workflows and their per-agent delivery adapters |
| spec-spine | governance semantics: `compile`, `index`, `check`, `lint`, `couple`, `registry`, `verify`, `attest`, `verify-attestation`, `config`, `delta`; the deterministic engine; governance starter content, produced as data |

This is a boundary, not a layering. Two consequences follow from it and they
are both visible in the product:

- There is no `spec-spine init`. The command has been removed along with its
  `--force` and `--with-kit` flags, and no verb replaces it under another name.
  `spec-spine init` now fails as an unknown subcommand (exit 3).
- spec-spine no longer distributes an agent harness. The skills, agents, rules,
  hooks, `Makefile` and CI workflow it used to write are environment, and the
  environment has an owner. Statecraft holds them globally, under
  `~/.statecraft/`.

Nothing about the engine's own distribution changes. `spec-spine` is still
installable from crates.io, npm, PyPI and `install.sh`, and every governance
verb still works on a repository that Statecraft never touched.

## Local-first, with no account

A solo developer can use every local Statecraft CLI and spec-spine capability
with no Statecraft platform account, no login and no hosted connection. Local
initialization, execution, approvals, policy, eligibility, evidence, recovery
and governance verification are all local. Authenticating to a model provider
is a separate question from platform authentication.

No part of spec-spine's deterministic engine depends on platform
authentication, on a hosted service, or on a global Statecraft installation
being present.

## The managed layout

A project Statecraft initializes looks like this:

```
AGENTS.md                 yours, with one Statecraft-inserted first line:
                          @.statecraft/AGENTS.md
.statecraft/AGENTS.md     Statecraft-generated project instructions
.statecraft/derived/      compiled governance artifacts, committed
.statecraft/state/        tooling's own working files, ignored
spec-spine.toml           repository root
specs/ standards/spec/    repository root
```

The bridge into a root `AGENTS.md` is a single first line. The rest of that
file is yours and is preserved.

`.statecraft/` as a whole is **not** runtime state. Only `.statecraft/state/`
and the derived tree's `build-meta.json` are excluded from version control.
Anything else under `.statecraft/` is an ordinary governed file: the ownership
walk sees it, `index coverage` reports it, and the coupling gate judges it like
a file anywhere else in the tree.

That distinction matters because `.statecraft/derived/` holds the committed
governance ledger. A rule that ignored the whole directory would put the ledger
outside version control and outside every check that reads it.

The paths above are configuration, not a new default. For a repository that
configures nothing, spec-spine's `derived_dir` is still `.derived`. See
[Configuration](configuration.md) for the `[layout]` keys.

## The producer contract

spec-spine retains exactly one function for a consumer that wants governance
starter content:

```rust
spec_spine_core::scaffold_init_json(config_json: &str) -> Result<String, Error>
```

It takes a serialized `Config` and returns a serialized `Scaffold`
(`files: [ScaffoldFile]`). It produces data. It never writes anything; the
caller decides what to put on disk.

### What it emits

| File | Content |
|---|---|
| `spec-spine.toml` | the starter configuration, reflecting the config it was given |
| `<standards_dir>/constitution.md` | the tier-2 constitution |
| `<standards_dir>/contract.md` | the normative summary |
| `<standards_dir>/templates/spec-template.md` | the authoring template |
| `<standards_dir>/templates/constitution-template.md` | the constitution template |
| `<specs_dir>/000-bootstrap/spec.md` | the bootstrap spec |
| `.gitignore` | an exclusion fragment for transient metadata and runtime state |

That is the whole list. It emits no `AGENTS.md`, no `CLAUDE.md`, no `.claude/`,
`.codex/` or `.agents/`, no skills, agents, hooks or MCP configuration, no CI
workflow and no `Makefile`.

The `.gitignore` entry is content for the consumer to reconcile, not permission
to replace an existing file. It comes back with `append: true` and an
`appendMarker`, and the writer is Statecraft's.

### What it guarantees

`scaffold_init_json` is a pure function of its argument. It does not write to
the filesystem, read the environment, discover or launch Statecraft, spawn a
process, open a network connection, read a clock, register anything or activate
an agent. The library does not require the Statecraft CLI to be installed, and
it behaves identically with an empty or absent home directory.

### The layout it is given

Every emitted path honors `config.layout` and is relative to the repository
root, never to the configuration file's directory. A non-default value for any
layout key produces a coherent scaffold rather than a default one.

The four values Statecraft passes are:

```toml
[layout]
specs_dir     = "specs"
standards_dir = "standards/spec"
derived_dir   = ".statecraft/derived"
state_dir     = ".statecraft/state"
```

`scaffold_init` and the `Scaffold` / `ScaffoldFile` types stay exported for a
consumer linking the library directly; the JSON facade is a projection of them.
See the [API Reference](api-reference.md).
