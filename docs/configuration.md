# Configuration reference

Every key of `spec-spine.toml`, its default, and what turning it on changes.
Written against `spec-spine 0.21.0`; `spec-spine config show` prints the
effective values for a repository, with the bypass floor merged and attributed
to its source.

The model is `spec_spine_types::Config`. Every table is
`deny_unknown_fields`, so a misspelled key is a config error at exit `3`
rather than a silently ignored line.

This document is part of the D-3 retention of the deleted documentation site
(`docs/design/09-disposition-2026-09-21.md` section 8), which was the only
place a full key reference existed. Defaults and semantics are re-read from the
current source, not copied: `[lint]`, `[coverage]`, `[meta]` and `state_dir`
all postdate the deleted page.

## `[meta]`

| Key | Default | Meaning |
|---|---|---|
| `required_version` | absent | The spec-spine version this repository is governed by, as a semver **requirement**: `"0.15.0"`, `">=0.15, <0.16"`, `"=0.15.0"`. Absent means unpinned. Checked on every run. |

A requirement rather than an exact version because both intentions are
legitimate: reproducing a byte-identical ledger pins with `=`, wanting fixes
but not a MAJOR writes a range. An adopter with no pin gets no version check at
all, so a binary/corpus mismatch there is silent.

## `[layout]`

Path conventions. Nothing in the engine hardcodes `specs/` or `.derived/`.

| Key | Default | Meaning |
|---|---|---|
| `specs_dir` | `specs` | Where `NNN-slug/spec.md` lives. |
| `derived_dir` | `.derived` | The compiled shard trees. Whatever it is set to, the gate adds it to the bypass floor itself (spec 092 section 3.8). It must name a directory inside the repository: an absolute path, a `..` segment, a `\` or a `:` is a configuration error (exit 3) at every verb (spec 128). |
| `standards_dir` | `standards/spec` | Constitution, contract, templates. |
| `schemas_dir` | `standards/schemas` | Adopter-side schemas. |
| `cargo_workspace` | `Cargo.toml` | The root Cargo workspace manifest. |
| `npm_workspaces` | `["package.json", "pnpm-workspace.yaml"]` | Manifests that *declare* workspace members. The indexer reads member globs from whichever exists. |
| `standalone_rust_workspaces` | `[]` | Crates outside the root Cargo workspace. |
| `standalone_npm_packages` | `[]` | npm packages outside the declared workspaces. |
| `state_dir` | `""` (none) | A declared, ungoverned tool-state root (spec 036). |

`.derived` is the product default and is **not** deprecated. This repository
and `statecraft-cli` set `derived_dir = ".statecraft/derived"` because they run
under Statecraft's managed layout; that is their configuration, not a change to
what an unconfigured corpus gets.

`state_dir` is *declared* and *ungoverned*: the gates recognize it, so `couple`
bypasses it, `coverage` excludes it, the resolver does not walk it, and it
contributes to no content hash. spec-spine never reads or writes inside it,
which is what keeps the purity invariant. Empty means no root is declared and
every behavior keyed on it is inert; there is deliberately no default, because
silently bypassing a real path on upgrade would change what the gate refuses.

Matching is separator-aware for both roots: a root of `state` covers `state`
and `state/journal.db` and never `stateful/x`.

### Recovering from a base whose `derived_dir` escapes

Since spec 128, a committed `derived_dir` that is absolute, has a `..`
segment, or contains `\` or `:` is a configuration error (exit 3) at every
verb. Older binaries wrote the derived tree wherever such a value pointed, so
a repository can have one committed on its default branch. The pull request
that corrects it is judged against that base, and two verbs read the base's
configuration:

| Verb on the correcting pull request | Exit | Why |
|---|---|---|
| `check`, `lint`, `index coverage` (read head only) | as usual | head's configuration loads |
| `couple`, when the diff deletes no path | 0 | no deletion asks for the base snapshot (spec 100 3.4) |
| `couple`, when the diff deletes any path | 3 | the merge-base snapshot's configuration cannot load |
| `delta` | 3 | it always classifies by the merge-base's rules |

Measured with a build of spec 129's branch, whose CLI behaves here exactly as
spec 128's does. A `Spec-Drift-Waiver:` does not
change either exit 3: a waiver clears drift, and this is not drift. Do not use
one, and do not run an older binary to get a pass, since an older binary is
what writes outside the repository.

The procedure:

1. **Confirm the cause at the base.** `spec-spine config show` on the default
   branch exits 3 and names `layout.derived_dir`. If it names anything else,
   this procedure does not apply.
2. **Account for what was written outside.** An older binary created, and
   pruned `*.json` files in, the directory the value pointed at. That
   directory is outside the repository; inspect it and decide what to remove
   by hand. No spec-spine verb touches it any more.
3. **Open a correcting pull request that deletes nothing.** Change
   `derived_dir` to a relative path of plain segments, run `spec-spine
   compile` and `spec-spine index`, and commit the regenerated tree, which is
   all additions. Put no other change in it, and in particular no deletion:
   with none, `couple` judges the diff normally and exits 0 when it is clean.
4. **If a required check runs `delta`, it stays at exit 3 on that pull
   request.** The repository's owner merges it by an explicit decision, with
   the `config show` output and the diff's file list recorded in the pull
   request body. That is a human override of one check on one pull request,
   not a waiver, and it is the owner's to make; a session or an agent never
   makes it. This repository's gate runs no `delta`, so here step 3 suffices.
5. **Make every other change after the merge.** Deletions and any other work
   go in a later pull request, whose base configuration now loads, so
   `couple` and `delta` judge it as usual (measured: both exit 0).

## `[manifest]`

| Key | Default | Meaning |
|---|---|---|
| `metadata_namespace` | `spec-spine` | Drives both `[package.metadata.<ns>].spec` in Cargo and `"<ns>": {"spec": "NNN-slug"}` in `package.json`. |

This is the package-floor link: a manifest key names the spec that answers for
a whole package. A floor is not a specific claim, which is what
`[coupling] require_ownership` turns on the difference between.

## `[index]`

| Key | Default | Meaning |
|---|---|---|
| `extra_hashed_inputs` | `["standards/**/*", ".github/workflows/**/*"]` | Globs folded into the staleness content hash, beyond the always-hashed core (every `spec.md`, the discovered manifests, and `spec-spine.toml`). |
| `resolver_exclusions` | `["target", "node_modules", ".derived", "dist", "build", ".next"]` | Directory **names** pruned from symbol and section resolution walks. |
| `[index.slices]` | `{}` | Named glob groups (spec 011), each emitted as a `build.sliceHashes` entry and gated by `index check --slice <name>`. Names match `[a-z0-9][a-z0-9-]*`. |

Glob semantics, with the trap stated: `dir/**` matches directories and so
matches no files; `dir/**/*` is what matches files. A glob that matches nothing
is `L-010`, and `lint --fail-on-warn` is in most gate chains, so a dead glob is
a refusal rather than a quiet no-op.

Slices are independent of the global hash: listing a file in a slice does
**not** fold it into `contentHash`.

Adding a pattern to `extra_hashed_inputs` restales every shard, so it is a
deliberate and expensive edit. Keep the patterns narrow: a bare `dir/**/*` over
a directory that can hold `.DS_Store` makes shard hashes machine-dependent.

## `[coverage]`

The ownership ratchet's universe is otherwise inferred: a source extension
inside a discovered package. That leaves out governance files at the root,
scripts and hooks in no package, and anything whose extension is not code.

| Key | Default | Meaning |
|---|---|---|
| `governed_scope` | `[]` | Paths that join the coverage universe regardless of extension or package (spec 078), and so join `index coverage` and `C-002`. |
| `governed_scope_exclusions` | `[]` | Carved back out of `governed_scope`, applied after it. Removes only what the scope added, never a file in the universe for another reason. |

Both are globs over repo-relative POSIX paths with `extra_hashed_inputs`
semantics. A resolver exclusion and a bypass prefix both still win: the scope
adds paths to the universe, it does not lift them out of an exemption.

Empty by default so no verdict moves on upgrade.

## `[coupling]`

| Key | Default | Meaning |
|---|---|---|
| `bypass_prefixes` | `[]` | **Additional** exempt paths, on top of the always-applied built-in floor. |
| `waiver_keyword` | `Spec-Drift-Waiver:` | The PR-body waiver keyword; the free-text reason follows the colon. |
| `auto_waive_dependency_only` | `false` | Self-clear a diff whose only changes are dependency version strings in standard manifest tables. |
| `require_ownership` | `false` | The ownership ratchet: a changed source file inside a discovered package that no spec **specifically** claims becomes `C-002` instead of being skipped. |

`bypass_prefixes` is additive and can never remove a floor entry. Match rules:
a trailing `/` is a directory prefix, a leading `**/` is a tail suffix
anywhere, anything else is an exact file. The default is empty because the
floor is the single built-in source; restating it in config would imply it was
overridable.

`auto_waive_dependency_only` is fail-closed: anything beyond a version string
(a new package, a `scripts` edit, spec-binding metadata) refuses the
auto-waiver. It is the one route a Dependabot-class pull request has, since it
can edit neither a spec nor a PR body.

`require_ownership`: *specifically* means a resolved ownership-bearing unit
(file, section, symbol, directory, crate, module) or a `// Spec:` comment
header. A manifest floor alone does not count, because it covers a whole
package regardless of what anyone has thought about. Off by default: `C-001`
holds for any corpus the day it is written, but full coverage is a state a
repository has to reach first, and `index coverage` reports the distance.

## `[lint]`

Opt-in conformance conventions (spec 046).

| Key | Default | Meaning |
|---|---|---|
| `require_ordinal_monotonic_depends_on` | `false` | Emit `L-007` for a `depends_on` entry that does not name a lower ordinal than the spec declaring it. |
| `unwitnessed_allowed` | `[]` | Globs naming claimed paths this corpus deliberately leaves out of every content hash, suppressing `L-008` for them (spec 050). |

An unwitnessed claim is a real gap and a legitimate state. `unwitnessed_allowed`
makes an exception explicit, never invisible: `index check` still counts every
unwitnessed claim and reports how many of the total the list covers.

## `[frontmatter]`

| Key | Default | Meaning |
|---|---|---|
| `extra_known_keys` | `[]` | Keys this adopter recognizes, suppressing the lint's unknown-key warning. They still overflow into `extra_frontmatter`. |

## `[domains]` and `[kind]`

| Key | Default | Meaning |
|---|---|---|
| `allowed` | `[]` | A closed enum for the optional `domain` (resp. `kind`) frontmatter field. |

Empty means the field is free text and no enum check runs. Non-empty makes it
a closed enum: the value, **when present**, must be a member, and a non-member
is a `V`-error. Field absence is always allowed.

## `[branding]`

| Key | Default | Meaning |
|---|---|---|
| `compiler_id` | `spec-spine` | Stamped into the registry's emitted `build` metadata. |
| `indexer_id` | `spec-spine` | Stamped into the index's emitted `build` metadata. |

## `[provenance.uri_schemes]`

An open registry mapping a provenance kind to a URI scheme.

| Key | Default |
|---|---|
| `knowledge` | `knowledge://` |
| `code-fingerprint` | `fingerprint://` |

Open rather than closed: an adopter adds their own kinds by adding entries.

## A worked example

This repository's own `spec-spine.toml` is the largest worked example in the
tree and is worth reading next to this table: it sets the managed layout, turns
the ratchet on, folds its governance files into the content hash, and pins its
own required version.
