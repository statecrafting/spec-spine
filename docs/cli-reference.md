# CLI reference

Every verb the `spec-spine` binary exposes, its flags, what it decides, and the
exit code it returns. Written against `spec-spine 0.21.0`; `spec-spine --help`
and `spec-spine <verb> --help` are the authority if the two ever disagree.

This document is part of the D-3 retention of the deleted documentation site
(`docs/design/09-disposition-2026-09-21.md` section 8). The site's `cli/`
section was the only place a per-verb flag and exit-code reference existed. It
is brought current here rather than copied: `init` is gone (spec 092), and
`check`, `delta`, `compact`, `config show`, `index owner`, `index diagnostics`
and the `--json` envelope all postdate the deleted pages.

## Exit codes

Mapped in exactly one place, `crates/spec-spine-types/src/error.rs`'s
`Error::exit_code()`, applied by `crates/spec-spine-cli/src/main.rs`. Since
spec 132 this is the **family contract** shared with the Statecraft CLI: five
codes, ordered by severity, and the numeric order is the severity order.

| Code | Outcome | Meaning |
|---|---|---|
| `0` | `ok` | the verb did what was asked and found nothing |
| `1` | `finding` | validation failure, drift, staleness, an unresolved claim, not found, or authored content that does not parse |
| `2` | `refused` | a precondition or policy was not met and nothing was done: invalid configuration, a `[meta] required_version` pin the binary does not satisfy, a containment refusal (specs 126-128) |
| `3` | `usage` | a clap argument error, an argument combination a verb rejects, or a malformed request document the caller supplied |
| `4` | `failed` | I/O, a tool-produced artifact that fails to parse or fails its schema, an internal serialization failure |

Before spec 132 (releases through 0.25.x) the contract was `0` ok / `1`
validation-failure-or-not-found-or-drift / `2` stale / `3`
I/O-parse-schema-config-or-usage. Staleness moved from `2` to `1`; a
containment or config refusal moved from `3` to `2`; I/O and schema failures
moved from `3` to `4`. Ask `spec-spine --version` before believing an exit
code: a binary built before 0.26.0 answers under the old contract.

`delta` is the one verb with no refusal of its own: it exits `0` whenever a
report was produced, whatever the report says.

## Global flags

| Flag | Meaning |
|---|---|
| `--repo <DIR>` | Repository root. Defaults to the current directory. Accepted by every verb and subcommand. |
| `-V`, `--version` | Print the version and exit `0`. Every released binary answers it. |

## The machine-readable verdict (`--json`)

The verbs that render a *verdict* accept `--json` (spec 034). The flag changes
what is written and never what is decided: `outcome` and `exitCode` always
agree with the code the process returns without it, and `outcome` is derived
from `exitCode`, never passed separately.

```json
{
  "schemaVersion": "1.0.0",
  "tool": "spec-spine",
  "verb": "couple",
  "outcome": "finding",
  "exitCode": 1,
  "summary": "couple: finding",
  "report": { "...": "the verb's facade payload" }
}
```

`report` and `error` are mutually exclusive and exactly one is present. A
failure carries `error` instead:

```json
{ "kind": "stale", "message": "...", "violations": [] }
```

`kind` is a closed lowercase set to branch on: `validation`, `stale`,
`not-found`, `drift`, `refused`, `config`, `io`, `schema`, `usage`,
`internal`. `message` carries no stability promise; branch on `outcome` and
`kind`, not on `summary` or `message`. `violations` is present only when
`kind` is `validation`.

Before spec 132 (schema `0.6.0` and earlier) the header carried `ok` (a
boolean) instead of `outcome` and `summary`, had no `tool` member, and `kind`
had no `refused`, `usage`, `internal` or `drift`, but did have `parse`:
authored content that fails to parse now reports `kind: validation`.

`verb` is a stable dotted command path, not the argv spelling:

| `verb` | Command |
|---|---|
| `compile.check` | `compile --check` |
| `compile.spec` | `compile --spec <id>` |
| `index.check` | `index check`, including `--slice` |
| `check` | `check` (both trees, spec 062) |
| `lint` | `lint` |
| `couple` | `couple` |
| `attest` | `attest` |
| `verify-attestation` | `verify-attestation` |
| `verify` | `verify <id>` |
| `delta` | `delta` |

`check` is its own verb and not a variant of either primitive: a consumer
branching on `compile.check` must not silently receive it.

The read-only query verbs (`registry *`, `index render|orphans|diagnostics|owner|coverage`,
`config show`) have spoken JSON since spec 009 and emit their document
directly, not wrapped in this envelope.

## compile

```
spec-spine compile [--check | --spec <ID>] [--fail-on-warn] [--json]
```

Compiles `specs/*/spec.md` into the deterministic registry shard tree under
`<derived_dir>/spec-registry/by-spec/<id>.json`.

| Flag | Meaning |
|---|---|
| `--check` | Verify the committed shards match the corpus, writing nothing. Exit `1` if stale (exit `2` before 0.26.0). |
| `--spec <ID>` | Validate exactly one spec and write nothing (spec 049). Accepts the short id (`056`). Incompatible with `--check`. |
| `--fail-on-warn` | Exit `1` on any warning-tier violation (spec 064). Accepted on every form; changes only the exit code, never `validation.passed` and never an emitted byte. |
| `--json` | Requires `--check` or `--spec`. The writing form's verdict is deliberately not machine-readable (spec 034 section 4). |

The bare writing form is for authors. A gate calls `--check` or `check`,
because a gate must never repair the tree it is judging.

A shard is named after its spec's frontmatter `id`. If any id would not make
one plain file name (empty, a leading `.`, a trailing `.` or space, or
containing `/`, `\`, `:` or NUL, which covers `../` traversal and absolute
paths; or a reserved Windows device name such as `CON`, `NUL.tar` or `COM1`, on
every platform), the writing form refuses (exit `2`, a containment refusal
under spec 132; exit `3` before 0.26.0) before it writes, prunes or creates
anything, including the `spec-registry/` directory itself on a first build,
and names the file name and directory (specs 126 and 127). This outranks the
exit `1` the same id's `V-012` would earn. The refusal is about the file name,
not the id grammar: an id that fails `V-012` but is a plain name (`001-Foo`)
is still written and still exits `1`. `compile --check` reports the offending
id without writing anything.

The writing form also refuses, with exit `2` (exit `3` before 0.26.0) and
nothing written, when any path it would write, create, remove or prune passes
through a symbolic link below the repository root, or meets an existing
component of the wrong kind (a file where a directory belongs), and names
that path (spec 127). It does not replace the link. The repository root
itself, and its ancestors, may be links. Not covered: another process
replacing a path while the verb runs, a configured `derived_dir` that points
outside the repository, and Windows links or junctions, which were not
measured.

## check

```
spec-spine check [--fail-on-unresolved] [--fail-on-warn] [--json]
```

Both freshness reads in one verb (spec 062): are the committed registry shards
and the committed index shards current? It compiles in memory and compares
without writing. The exit code is the higher of the two halves' codes,
numerically (spec 132, amending spec 062's `3`-then-`1`-then-`2`-then-`0`
order), and the two report lines under it say which tree answered what.

| Flag | Forwarded to | Meaning |
|---|---|---|
| `--fail-on-unresolved` | the index half | Exit `1` when the committed index records any unresolved-unit diagnostic (`W-001` / `W-002`). |
| `--fail-on-warn` | the compile half | Exit `1` on any warning-tier violation. |

Neither implies the other and either may be passed alone. This is the only form
CI can call: the self-governance job runs `check` in place of `compile` and
`index`.

## registry

Read-only queries over the compiled registry. Every subcommand takes `--json`.

| Subcommand | Answers |
|---|---|
| `registry list [--status S] [--ids-only]` | The corpus, optionally filtered by status. `--ids-only` prints bare ids, one per line (a JSON string array with `--json`). |
| `registry show <ID>` | One spec. Accepts the short id (`016`). |
| `registry status-report [--nonzero-only]` | Counts by status. `--nonzero-only` omits zero counts; the total still covers the corpus. |
| `registry relationships <ID>` | A spec's relationship neighborhood: the typed edges in and out. |
| `registry obligation <SPEC>#<ID>` | One declared obligation (spec 106): its kind, text, anchor and inputs, its section's digest, and the spec's content hash. The spec half accepts the short id; an unqualified id exits `3`, an unknown one `1`. |
| `registry closure --request <FILE\|->` | Resolve a context closure (spec 107): every named spec, section and obligation with its identity, and one order-independent digest over them. Refuses a stale registry with exit `1` (exit `2` before 0.26.0). |
| `registry impacts [--target <REF>] [--declared-by <SPEC>]` | Every declared impact and conflict against spec 106's obligations (spec 109), inverted so the target side can see them. `--target` is a spec id or a qualified `<spec-id>#<obligation-id>` reference; `--declared-by` is a spec id; both filters compose by intersection. An unqualified `#` reference exits `3`; an unknown spec or obligation exits `1`. |
| `registry moves [<PATH>]` | Look up a path against every declared move (spec 111): a relocation, split, merge or removal a spec declared in its own frontmatter. Without a path, lists every declaration, flattened and sorted. Answers from the committed registry, like `show`; changes no verdict and infers nothing. Exits `0` on `unmapped`/`resolved`, `1` on `ambiguous`/`cycle` (the lookup refusing to guess, not an error). |
| `registry plan [--next]` | Which specs can be worked on now and what blocks the rest (spec 035). `--next` prints only the first ready spec; an empty ready set is `(nothing ready)` at exit `0`, not a failure. |

`plan`'s `ready` set is a **scheduling** answer, not an approval. See
`docs/api.md` for what membership does and does not mean.

## interface

```
spec-spine interface verify [--export <CORPUS>=<DIR>]... [--spec <ID>] [--json]
```

Recompute every declared cross-corpus interface reference (spec 110) against a
local checkout of each cited corpus. Each `--export` names a corpus and a
directory the caller asserts is its repository root; only the referenced
`spec.md` files under its specs directory are read. Nothing is fetched and
nothing is written.

| Outcome | Meaning | Exit |
|---|---|---|
| `current` | the cited spec's content hash equals the pin | `0` |
| `sections-current` | the spec moved, and every pinned section is unchanged | `0` |
| `stale` | the spec moved and no section is pinned, or a pinned section moved or vanished | `1` |
| `missing` | the export has no such spec | `1` |
| `unverified` | no `--export` was supplied for the corpus | `1` |

A stale committed registry exits `1` (exit `2` before 0.26.0) before any
export is read. A malformed `--export` or a corpus named twice exits `3`
(usage); an unreadable export directory exits `4` (I/O; exit `3` before
0.26.0); an export path that escapes its root (a symlink out) exits `2`
(refused). `--spec` accepts the short id; an unknown one exits `1`. To pin a reference,
run `spec-spine registry show <id> --json` in the cited corpus and copy
`contentHash` (and any `sectionDigests` entry), each prefixed `sha256:`.

## scope

```
spec-spine scope evaluate --scope <FILE|-> [--json]
spec-spine scope compare <A> <B> [--json]
```

Evaluate or compare a declared work scope (spec 108): a consumer's document
naming which paths one piece of work expects to change alone (`mutable`),
alongside named other specs (`shared`, each with a non-empty `with` list of
specs), and only reads (`readOnly`). A scope lives in the consumer's record,
never in spec frontmatter; spec-spine only evaluates and compares one.

| Subcommand | Answers |
|---|---|
| `scope evaluate --scope <FILE\|->` | Resolves each declared path's owners against the committed index and reports where the declaration and the ownership disagree: `S-001` unowned, `S-002` undeclared crossing (a `mutable` path another spec owns), `S-003` sharing mismatch (a `shared` path whose owners disagree with its declared `with`). `readOnly` paths are reported with no finding. Refuses a stale committed index with exit `1` (exit `2` before 0.26.0) before any path is resolved. A malformed document (unknown member, an absolute path or one carrying `..`, no path at all, one path under two roles) exits `3`; an unresolved `ownSpec` or `with` exits `1` naming every one. Otherwise exits `0` whether or not it found anything: a report, not a gate. |
| `scope compare <A> <B>` | Reports every overlapping declared path (equal, or one a subtree containing the other) whose roles conflict: `both-mutable`, `mutable-shared`, `changed-under-read`. Two `shared` or two `readOnly` entries never conflict. Reads no ledger and exits `0` whether or not a conflict is found; the same malformed-document refusal (exit `3`) applies to each document. |

Either `--scope`, or one of `A` / `B`, may be `-` for stdin. Nothing here
locks, reserves, excludes or permits anything: `couple`, `check`, `lint` and
`index coverage` never read a scope.

## index

```
spec-spine index [--repo DIR]
```

Builds the code-as-source index into `<derived_dir>/codebase-index/{by-spec,by-package}/`.

Per-spec shards are named after the spec's `id`, with the same refusal as
`compile`: an id that is not one plain file name exits `2` (a containment
refusal, spec 132; exit `3` before 0.26.0) before anything under
`codebase-index/` is written, pruned or created, the directory itself
included (spec 126). So does a symbolic link below the repository root on any
path it writes, removes or prunes, `slices.json` and both shard directories
included (spec 127).

| Subcommand | Answers |
|---|---|
| `index check [--slice NAME] [--fail-on-unresolved] [--json]` | Is the committed index current? Exit `1` if stale (exit `2` before 0.26.0). `--slice` gates one named `[index.slices]` group instead of the whole shard set; naming an undeclared slice is usage, exit `3` (it was a config error, also `3`, before 0.26.0). `--fail-on-unresolved` exits `1` on any `W-001` / `W-002`. |
| `index render` | The committed index as markdown. A projection; it never recomputes. |
| `index orphans [--json]` | Specs the committed index records as owning nothing resolvable. |
| `index diagnostics [--json]` | The diagnostics the committed index records (spec 044). Recomputes nothing and never refuses; the refusal lives on `check`. Empty output means none. |
| `index owner <PATH> [--json]` | Which specs own one path, and how (spec 048). Calls the coupling gate's own owner derivation, so this answer and a `C-001` decision cannot disagree. The path need not exist on disk. |
| `index coverage [--fail-on-untraced] [--paths-from FILE] [--json]` | Which source files no spec specifically claims (spec 029). `--fail-on-untraced` exits `1` unless every source file has a specific owning spec. `--paths-from` supplies the candidate paths instead of asking git (spec 078), which is the git-free route. |

Bare `index coverage` refuses nothing. The ratchet is the flag.

## lint

```
spec-spine lint [--fail-on-warn] [--fail-on-info] [--json]
```

Corpus conformance (the `L-` codes). Bare `lint` reports; `--fail-on-warn`
exits `1` on any warning-tier diagnostic, `--fail-on-info` on any info-tier
one.

## couple

```
spec-spine couple [--base BASE] [--head HEAD] [--pr-body FILE]
                  [--paths-from FILE] [--include-uncommitted]
                  [--waiver-as-of YYYY-MM-DD] [--waiver-uses ID=N]... [--json]
```

The PR-time gate: refuse code that drifts from its owning spec. The diff it
reads is `git diff --no-color -U0 --no-renames base...head`, a three-dot range,
so the "before" side is `merge-base(base, head)`.

| Flag | Meaning |
|---|---|
| `--base <BASE>` | Base ref. Default `origin/main`. |
| `--head <HEAD>` | Head ref. Default `HEAD`. |
| `--pr-body <FILE>` | The waiver source, as a file path. Falls back to `$SPEC_SPINE_PR_BODY`. |
| `--paths-from <FILE>` | Override the diff: newline-delimited changed paths, whole-file authority, no hunk data. |
| `--include-uncommitted` | Also judge the index and working tree, so a pre-commit run sees the change being committed (spec 081). Only valid when `--head` resolves to `HEAD`, and never with `--paths-from`. |
| `--waiver-as-of <YYYY-MM-DD>` | The date a declared `-Until:` is judged against (spec 113). Never read from the clock: without it an expiry is reported `not-evaluated`. A value that is not a date exits `3`. |
| `--waiver-uses <ID>=<N>` | How many runs the waiver with that `id` has already cleared, excluding this one (spec 113); repeatable. The count is yours: nothing records or consumes a use. Without it a declared `-Max-Uses:` is `not-evaluated`, never zero. |

Refusal codes: `C-001` (a path changed without an authoring edit to any owning
spec) and `C-002` (a changed source file no spec claims, when
`[coupling] require_ownership` is on).

A `Spec-Drift-Waiver:` line in the PR body clears the run. It is a human
instrument; an agent never writes one on its own authority. A dependency-only
manifest bump self-clears through `[coupling] auto_waive_dependency_only`.

A waiver may narrow itself with lines of its own under it (spec 113), each
naming the keyword without its colon plus a suffix:

```
Spec-Drift-Waiver: mechanical version bump
Spec-Drift-Waiver-Paths: npm/package.json, py/pyproject.toml
Spec-Drift-Waiver-Until: 2026-12-31
Spec-Drift-Waiver-Since: a3d5213d
Spec-Drift-Waiver-Max-Uses: 1
```

`-Paths:` clears only the listed paths (an entry ending in `/` is a subtree);
without it a waiver clears everything, as it always has, and is reported
unscoped. `-Until:` is judged against `--waiver-as-of`, `-Since:` by asking git
whether the commit is an ancestor of `--head`, and `-Max-Uses:` against
`--waiver-uses`. Each declared check is reported `satisfied`, `failed` or
`not-evaluated`; a waiver with a failed check clears nothing. Several waivers
may be declared, and each violation is cleared by the first effective waiver
whose scope covers it. The report says which waiver cleared which violation,
and `--json` carries it as `report.waivers`. A waiver reported `effective` is
one whose declared checks did not fail; it says nothing about who approved it.

## delta

```
spec-spine delta [--base BASE] [--head HEAD] [--json]
```

Classifies every path a change touches under the **merge base's** rules (spec
071): implementation, requirement, relocation, verification, authority,
lifecycle, constitutional, policy, derived, bypassed, unowned or unknown. A path
may carry several classes.

`relocation` (spec 142) replaces `requirement` for a `spec.md` whose body only
lost sections that another spec declares it received (`relocates`) and that
arrived unchanged, compared with the source as it was at the merge base.
`relocations` lists every declared relocation the change touches, with
`proven` and, when it is false, a `reason`. An unproven relocation leaves the
change a `requirement`.

The configuration and the index that classify are the merge base's, so a change
cannot reclassify itself by editing `spec-spine.toml`.

`priorPolicy.required` is true when any path carries requirement, verification,
authority, lifecycle, constitutional, policy or unknown. `required: false`
means only that no structural class above changed; it is not a statement that
the change is safe, correct or approved. `delta` refuses nothing.

## verify

```
spec-spine verify <ID> [--plan] [--json]
```

Runs a spec's declared acceptance: the `verify:cli` commands under its
`## Verification` heading, in order, stopping at the first failure. `<ID>`
accepts the short form (`049`).

`--plan` prints the commands and runs none of them. Reading the plan before
executing it is the safety affordance for the one verb that runs what the
corpus declares, which is why `verify` is deliberately outside the gate chain:
the chain runs against branches whose contents are, in the general case, a
stranger's.

Under `--json`, stdout is reserved for the envelope and the child commands'
output goes to stderr (spec 090; filed pre-collapse as 118, see docs/corpus-map.md).

## attest

```
spec-spine attest [--spec ID] [--with-coupling] [--snapshot]
                  [--sign --key PATH] [--key-id ID] [--json]
```

Emits a reproducible attestation into `<derived_dir>/attestation/`.

| Flag | Meaning |
|---|---|
| `--spec <ID>` | Scope to one spec (spec 039), writing `<derived>/attestation/by-spec/<id>.json`. Accepts the short id. |
| `--with-coupling` | Also record the coupling verdict. |
| `--snapshot` | Emit an authority snapshot instead (spec 070), writing `<derived>/attestation/snapshot.json`: which inputs were read, what they hashed to, whether the committed ledger matches the recompute, and every spec's territory digest. Cannot combine with `--spec` or `--with-coupling`. |
| `--sign` | Produce a detached Ed25519 seal over the attestation hash. |
| `--key <PATH>` | The signing key (32-byte seed, raw or hex). Required with `--sign`. |
| `--key-id <ID>` | Override the seal's key id. Defaults to the hex public key. |

Exit `0` means an attestation was written. It is a record, not a gate.

With `--spec`, the file is named after the resolved spec id. An id the corpus
declares but that is not one plain file name (see `compile`) exits `2` (exit
`3` before 0.26.0) before `by-spec/` is created and before the attestation or
its seal is written (spec 126). Every form refuses the same way (exit `2`)
when the attestation, the snapshot, the seal or a directory above them is a
symbolic link below the repository root; the seal is made first, so a refused
seal leaves the attestation unwritten too (spec 127).

The document goes to the file, not to stdout: stdout carries a summary, and
redirecting it publishes prose rather than the attestation.

## verify-attestation

```
spec-spine verify-attestation [--spec ID] [--snapshot] [--recompute]
                              [--signature --public-key PATH] [--seal PATH]
                              [--attestation PATH] [--json]
```

| Flag | Meaning |
|---|---|
| `--recompute` | Re-read the corpus and check it reproduces the attestation. No key needed. |
| `--signature` | Check the detached seal against a supplied public key. |
| `--public-key <PATH>` | The Ed25519 public key (32 bytes, raw or hex). Required with `--signature`. |
| `--seal <PATH>` | The detached seal. Defaults to the attestation's sibling `.sig`. |
| `--attestation <PATH>` | The attestation file. Defaults to `<derived>/attestation/attestation.json`. |
| `--spec <ID>` | Verify the per-spec attestation for this id (spec 039). |
| `--snapshot` | Verify the authority snapshot (spec 070) instead. |

The verifier checks the bytes it was given (spec 068). A re-canonicalized copy
fails: unknown members, reformatting and duplicate keys are all refused, and a
payload whose schema MAJOR is unsupported is refused while a MINOR ahead of the
verifier verifies.

## compact

```
spec-spine compact --plan-file FILE [--plan] [--force] [--map-out FILE]
```

Rewrites the corpus under an authored compaction plan (spec 096): which specs
leave and which spec answers for each, whether the survivors' ordinals are
compacted, and which paths retire. `--plan` prints the map and the per-form
report and writes nothing; `--force` rewrites over a dirty tree; `--map-out`
defaults to `docs/corpus-map.md`.

## config show

```
spec-spine config show [--json]
```

The effective configuration: every default resolved, and the built-in bypass
floor merged with the adopter's list and attributed to its source. See
`docs/configuration.md` for what each key means.

## There is no `init`

The deleted site documented `spec-spine init` and a `--with-kit` flag. Both
were removed by spec 092: this repository ships a governance **engine**, not a
development environment. The retained producer is the library function
`spec_spine_core::scaffold_init_json`, a pure function that returns governance
starter files as data and writes nothing. The Statecraft CLI is the sole
initializer of a managed development environment.
