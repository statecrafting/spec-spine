# Adopting spec-spine

> Take any conventional repo from zero to spec-governed: **install →
> `spec-spine init` → annotate manifests → wire CI.** No source edits to the
> library; every project-specific assumption is a `spec-spine.toml` knob (see
> §Config). For the design rationale see
> [design/00-architecture.md](design/00-architecture.md); for the programmatic
> API see [api.md](api.md).

spec-spine compiles a markdown spec corpus into a typed, hash-verifiable
authority ledger and refuses code that drifts from its owning spec. Adoption is
four steps.

---

## 1. Install

### From crates.io (recommended)

```sh
cargo install spec-spine-cli      # yields a `spec-spine` binary on your PATH
```

### Prebuilt binary (no Rust toolchain)

```sh
curl -fsSL https://raw.githubusercontent.com/statecrafting/spec-spine/main/install.sh | sh
```

The script detects your platform/arch, downloads the matching release archive
and its `.sha256` sidecar, verifies the checksum, and drops `spec-spine` on your
`PATH`. Pin a version with `SPEC_SPINE_VERSION=vX.Y.Z` (a published release tag) and a target dir with
`SPEC_SPINE_BIN_DIR=~/.local/bin`.

### From source

```sh
cargo install --path crates/spec-spine-cli
```

Verify any install:

```sh
spec-spine --version
spec-spine --help
```

---

## 2. Scaffold the corpus: `spec-spine init`

Run at your repo root:

```sh
spec-spine init            # skips files that already exist
spec-spine init --force    # overwrite existing files
```

`init` writes a starter governance corpus:

| Path | What it is |
|---|---|
| `spec-spine.toml` | your config: every knob defaulted, ready to edit |
| `standards/spec/constitution.md` | tier-2 durable principles |
| `standards/spec/contract.md` | the normative summary |
| `standards/spec/templates/spec-template.md` | template for new specs |
| `standards/spec/templates/constitution-template.md` | template for the constitution |
| `specs/000-bootstrap/spec.md` | the hand-authored bootstrap spec (tier 1) |
| `.claude/rules/orchestrator-rules.md` | execute-in-order / write-output / stop-at-checkpoints |
| `.claude/rules/governed-artifact-reads.md` | read `.derived/**` only via `spec-spine`, never ad-hoc `jq` |
| `.claude/rules/adversarial-prompt-refusal.md` | the prompt-time refusal rule (coherence guard) |

Then compile the corpus and confirm it is well-formed:

```sh
spec-spine compile          # → .derived/spec-registry/by-spec/ shards
spec-spine lint             # corpus conformance
spec-spine registry list    # see your specs
```

Write your first real specs under `specs/NNN-slug/spec.md` using the template,
declaring the **authority units** each spec owns (file / section / symbol /
directory / crate / module) in its frontmatter edges.

---

## 3. Annotate manifests: link code to specs

Three linkage directions connect code ↔ spec; the gate joins all three. The two
you author directly:

**Manifest key** (crate / package → spec). The TOML/JSON key is your
`manifest.metadata_namespace` (default `spec-spine`):

```toml
# Cargo.toml
[package.metadata.spec-spine]
spec = "001-my-capability"
```

```json
// package.json
{ "spec-spine": { "spec": "001-my-capability" } }
```

**Comment header** (file → spec), a doc-comment at file root:

```rust
// Spec: specs/001-my-capability/spec.md
```

The third direction, **spec edges**, is the `unit:` declarations inside each
spec's frontmatter (`establishes` / `extends` / `refines` / `supersedes` /
`amends` / `co_authority` / `constrains` / `references`; `references` is the
only non-owning edge, ignored by the coupling gate). See the bootstrap spec and
the template for the grammar.

Build the code-as-source view and commit it:

```sh
spec-spine index            # → .derived/codebase-index/{by-spec,by-package}/ shards
git add .derived/           # committed so the staleness + coupling checks can compare
```

> **Why commit `.derived/`?** Determinism makes the committed registry/index a
> reliable baseline. Both artifacts are stored **sharded** (one file per
> authority unit; spec 024), so two PRs touching different specs/packages write
> disjoint files and never conflict. The staleness check (`spec-spine index
> check`) recomputes each shard's hash (and the shard set) and compares it to the
> committed shards; the coupling gate joins the committed registry + index
> (assembled from shards) against the PR diff. `build-meta.json` (the sole
> wall-clock artifact) is the one file you `.gitignore`.

---

## 4. Wire CI: the coupling gate

The gate runs at PR time and refuses a changed, owned path whose owning spec was
*not* also edited (exit 1). Run it against the PR's merge base:

```yaml
# .github/workflows/spec-spine.yml
name: spec-spine
on: pull_request
permissions:
  contents: read
jobs:
  govern:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0          # full history so the gate can diff the merge base
      - run: cargo install spec-spine-cli   # or download the prebuilt binary
      # `compile --check` validates the frontmatter AND proves the committed
      # registry shards match the corpus, without writing (exit 1 invalid,
      # 2 stale). Never run it after a plain `spec-spine compile` in the same
      # job: it would compare the shards against files that run just overwrote
      # and pass unconditionally.
      - run: spec-spine check               # both trees: validation + freshness
      - run: spec-spine lint --fail-on-warn
      - name: Coupling gate
        env:
          PR_BODY: ${{ github.event.pull_request.body }}
        run: |
          set -euo pipefail
          printf '%s' "${PR_BODY:-}" > /tmp/pr-body.txt
          spec-spine couple \
            --base "${{ github.event.pull_request.base.sha }}" \
            --head HEAD \
            --pr-body /tmp/pr-body.txt
```

The freshness read (`spec-spine check`, or either primitive alone) compares against
**committed** artifacts, so this job assumes you commit `.derived/` as described
in §3. If you gitignore the derived tree instead, drop both `--check`/`check`
forms and run plain `spec-spine compile` and `spec-spine index` to build the
artifacts in-job: with nothing committed to compare against, the freshness gates
would report every shard missing and fail permanently.

**Refusing warnings (`--fail-on-warn`, spec 077).** `compile` emits one
warning-tier code, `V-010`, for a `depends_on` naming a spec that does not
exist. The tier is deliberate: a corpus that files specs forward must be able to
name a dependency filed after the spec that names it, so escalation is the
caller's decision. `compile --fail-on-warn` and `check --fail-on-warn` make it
exit `1`; `check`'s form forwards into the compile half, mirroring
`--fail-on-unresolved` into the index half, and is the one to use in CI, which
runs `check` rather than the primitives. The flag changes the exit code only:
the shards written are byte-identical with and without it.

Leave it off while migrating a corpus that legitimately points forward, and turn
it on once the graph is closed. Bare `compile` is unchanged either way, so
upgrading the binary cannot break an existing job.

This repo dogfoods exactly this pattern; see
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

### Waivers

When a drift is deliberate and reviewed, add a line to the PR body using the
configured keyword (default `Spec-Drift-Waiver:`):

```
Spec-Drift-Waiver: refactor moves helper out of the owned section; behavior unchanged
```

The waiver is global to the run and downgrades violations to warnings.

### Coverage: "is everything specified?"

The gate above refuses drift in code a spec claims; it says nothing about code
no spec claims. `spec-spine index coverage` (spec 032) answers that, per
source file, against the committed index:

```sh
spec-spine index coverage                      # the report (exit 2 if the index is stale)
spec-spine index coverage --json               # the same as a CoverageReport
spec-spine index coverage --fail-on-untraced   # exit 1 unless every source file is claimed
```

A file is **specifically claimed** when a resolved unit (file, section,
symbol, directory, crate, module) or a `// Spec:` comment header covers it.
A file only a manifest floor (`[package.metadata.<ns>].spec`) covers is
**floor-only**: the floor is the right safety net for drift and a useless
coverage signal, so it counts as debt here. Anything else is **unclaimed**.
The universe is source files inside discovered packages, minus
`resolver_exclusions` and bypassed paths; prose, manifests, workflows and
config are never counted.

To turn coverage into a PR-time ratchet, set `[coupling] require_ownership =
true`: a changed floor-only or unclaimed source file is then a `C-002`
violation (waivable like `C-001`; deletions are exempt). The report predicts
the gate exactly, both read one classifier over one universe. The intended
sequence: read the report, flip the flag to stop new debt, drive the lists to
empty, then add `--fail-on-untraced` to CI to defend the state.

---

## Config: `spec-spine.toml`

An **absent file yields a working default** for a single-Cargo-workspace repo
with `specs/` at the root. Every knob below is optional and traces to a real
divergence observed across the reference repos. Every sub-table is
`deny_unknown_fields`: a typo is a loud `config error`, not a silent no-op.

| Knob | Purpose | Default |
|---|---|---|
| `manifest.metadata_namespace` | the Cargo `[package.metadata.<ns>].spec` / package.json `"<ns>".spec` key | `"spec-spine"` |
| `domains.allowed` | closed enum for the optional `domain` field; **empty ⇒ disabled** (free-text) | `[]` |
| `kind.allowed` | closed enum for the optional `kind` field; symmetric with `domains` | `[]` |
| `layout.specs_dir` / `derived_dir` / `standards_dir` / `schemas_dir` | path conventions, never hardcoded | `specs` / `.derived` / `standards/spec` / `standards/schemas` |
| `layout.state_dir` | one repo-relative directory for the state of tools built around spec-spine (spec 039); bypassed by `couple`, excluded from `coverage`, never resolved or hashed, never read or written by spec-spine; must not overlap `specs_dir` / `derived_dir` | `""` (nothing declared) |
| `layout.cargo_workspace` | root Cargo workspace manifest | `Cargo.toml` |
| `layout.npm_workspaces` | manifests that *declare* npm/pnpm workspace members | `["package.json", "pnpm-workspace.yaml"]` |
| `layout.standalone_rust_workspaces` / `standalone_npm_packages` | crates/packages outside the root workspace | `[]` |
| `index.extra_hashed_inputs` | globs folded into the staleness content hash, beyond the always-hashed core. **Keep the trailing `/*`**: `dir/**` matches directories and therefore no files | `["standards/**/*", ".github/workflows/**/*"]` |
| `index.resolver_exclusions` | dir names pruned from symbol/section walks | `["target","node_modules",".derived","dist","build",".next"]` |
| `index.slices` | named glob groups, each emitted as a `build.sliceHashes` entry and gated by `index check --slice <name>`; names match `[a-z0-9][a-z0-9-]*`, each list non-empty. Independent of the global `contentHash` | `{}` |
| `branding.compiler_id` / `indexer_id` | ids stamped in emitted `build` metadata | `"spec-spine"` |
| `coupling.bypass_prefixes` | **additions** to the built-in bypass floor (additive; cannot remove a floor entry) | `[]` |
| `coupling.waiver_keyword` | the PR-body waiver keyword | `"Spec-Drift-Waiver:"` |
| `coupling.require_ownership` | the ownership ratchet (spec 032): a changed source file inside a package that no spec **specifically** claims (a resolved unit or a `// Spec:` header; a manifest floor alone does not count) is a `C-002` violation. Read `spec-spine index coverage` first; turn on to stop new debt | `false` |
| `coupling.auto_waive_dependency_only` | when `true` and no PR-body waiver is present, mechanically self-waives PRs where every non-bypassed changed path is a recognized dependency manifest with only version-pin changes: a `package.json` dependency table, a `Cargo.toml` dependency version, or a claimed `.github/workflows/*.yml` `uses:` action ref (the dependabot-class path); fail-closed on anything more (spec 005 §3.5, extended by spec 030) | `false` |
| `provenance.uri_schemes` | open kind→scheme map for provenance URIs | `{ knowledge = "knowledge://", code-fingerprint = "fingerprint://" }` |
| `frontmatter.extra_known_keys` | recognized frontmatter keys added without forking the types crate | `[]` |

A non-default example (a repo whose namespace is `acme` with a closed domain
enum and an extra standalone crate):

```toml
[manifest]
metadata_namespace = "acme"

[domains]
allowed = ["app", "platform", "tooling"]

[layout]
standalone_rust_workspaces = ["apps/desktop/src-tauri"]

[index]
extra_hashed_inputs = ["standards/**/*", ".github/workflows/**/*", "schemas/**/*"]
```

The bypass floor (always applied, cannot be removed) covers `.github/`, `docs/`,
`README.md`, `CHANGELOG.md`, `LICENSE`, `CODEOWNERS`, `.gitignore`,
`.gitattributes`, `standards/spec/constitution.md`, `.derived/`, `**/Cargo.lock`,
`**/package-lock.json`, and `**/pnpm-lock.yaml`. Your `coupling.bypass_prefixes`
adds to it (other lockfiles, e.g. `yarn.lock`, are not covered by default).
Match rules: a trailing
`/` is a directory prefix, a leading `**/` is a tail-suffix match anywhere, and
anything else is an exact path.

---

## A note on OAP-style adopters

A repo with domain-specific output (compliance reports, factory artifacts, a
Claude config-hash gate) adopts spec-spine as **generic core + its own overlay
crate**, *not* as a drop-in. The generic core deliberately omits that machinery
(see [design/00-architecture.md](design/00-architecture.md) §10.4); the overlay
reads the committed registry/index shard trees (`by-spec/`, `by-package/`) via
the public loaders and emits its own sibling artifact. See
[overlay-contract.md](overlay-contract.md).

---

## Definition of done (for your repo)

- `spec-spine init` scaffolded the corpus; `spec-spine compile` and
  `spec-spine lint` are clean.
- Your crates/packages carry `[package.metadata.<ns>].spec` (or the package.json
  equivalent), and `spec-spine index` maps them to specs.
- `.derived/` is committed (except `build-meta.json`).
- CI runs `check` → `lint` → `couple` on every PR.

## If your corpus has no code yet

Three of the four repositories governed by spec-spine ratify the whole corpus
before writing a line of code, and stay that way for months.
[docs/specify-first.md](specify-first.md) says what the counts, the warnings and
the gates mean in that mode, and what to run first.

## Bypass and hashing are independent

Two mechanisms decide what happens to a path, and they are **separate axes**.

**Bypassed** means the coupling gate will not refuse a change to it. The
built-in floor plus your `[coupling] bypass_prefixes`; `spec-spine config show`
prints the merged list.

**Hashed** means a change to it stales shards. `spec-spine.toml`, every spec's
`spec.md`, each discovered package's manifest, the span-backing files of
resolved units, and everything matched by `[index] extra_hashed_inputs`.

A path can be on both, neither, or either, and neither implies the other:

- `docs/` here is **bypassed and not hashed**: a documentation edit raises no
  `C-001`, and stales nothing.
- `standards/**/*` here is **bypassed and hashed**: an edit to the constitution
  raises no `C-001`, and does stale every shard.

Bypassed answers "will the gate refuse this change"; hashed answers "does this
change make the ledger stale". They are different questions, and reading one as
the other is how a governance file ends up outside both.

> **Watch the glob form** in `extra_hashed_inputs`. `dir/**` matches
> **directories**, so it hashes no files; you want `dir/**/*`. This repository
> carried `["standards/**", ".github/workflows/**"]` for a long time, matching
> nothing, until spec 057's predicate found it. So did the shipped default
> behind it, until spec 069.

> **Upgrading across spec 069.** `[index] extra_hashed_inputs` shipped a default
> that matched no files. It is fixed. If you did not override the key, your next
> `spec-spine index` will rewrite every shard once, because the standards tree
> and the workflow directory are entering the content hash for the first time.
> Commit the result. Nothing about what staleness *means* has changed; a surface
> that was silently outside the ledger is now inside it.

> **If your own `spec-spine.toml` carries `standards/**` or
> `.github/workflows/**`** (spec 074 3.9), those entries match **no files**, and
> upgrading does not change them: the value is yours, not the default, and spec
> 069 only fixed the default. **The absence of a restale on upgrade is therefore
> not evidence that you were unaffected.** It is the opposite: your patterns
> never contributed a byte to any content hash, so there was nothing to move.
> Rewrite them as `standards/**/*` and `.github/workflows/**/*`, run
> `spec-spine index` once, and commit the result.
>
> Every repository scaffolded before v0.16.0 is in this cohort, because
> `scaffold.rs` emitted the default's value into the file. Since spec 074,
> `spec-spine lint` names the pattern for you: `L-010` refuses any
> `extra_hashed_inputs` entry ending in `/**`.

> **Upgrading across spec 075.** Two things are renamed, and one is added.
>
> The session skill is **`/prime`**, not `/init`. Claude Code ships its own
> `/init`, which generates a CLAUDE.md: a one-time, repository-level operation
> that *writes*, where the kit's is per-session and only reports. The kit was
> shadowing a built-in and inverting its meaning. **There is no `/init` alias**,
> deliberately: an alias keeps shadowing for the whole deprecation window, and
> skills are copied files, so an adopter who does not refresh keeps their old
> copy regardless. **The failure mode if you refresh the skills but keep a
> customized `AGENTS.md`** that still says `/init` is "skill not found", which
> is loud and instantly diagnosable rather than silent. Rename the reference.
>
> **`spec-spine check`** is new and additive: it runs both freshness reads and
> reports each tree separately, so the protocol asks one question with one verb.
> `compile --check` and `index check` are unchanged, keep their flags and their
> contracts, and remain the right call when you regenerated only one tree.

> **Upgrading across spec 073.** A GitHub Actions workflow now folds into the
> content hash as its **governance projection**: the parsed document with the
> pinned ref of every `uses:` reference removed and the action path kept. A
> Dependabot action bump therefore stales nothing, while a changed action, an
> unpin, a `run:` / `with:` / `env:` / `if:` edit, an added step and a changed
> trigger all still do. If you hash your workflows, your next `spec-spine
> index` rewrites every shard once, exactly as the npm and cargo projections
> did before it. Commit the result. No schema version changes: only a hash
> value moves.
>
> If you seal your ledger (spec 023), a **corpus attestation created before
> this change** was computed over hashes from the previous rule. Re-attest
> after re-indexing. `verify-attestation --recompute` compares the tool version
> before it compares content, so a pre-073 attestation reports
> `VersionMismatch` with the remedy in the message, never a content mismatch:
> this reads as an upgrade, not as tampering.

## Directory units claim recursively

A `file` unit with a **trailing slash** is a subtree claim:

```yaml
establishes:
  - "crates/spec-spine-core/src/"
```

Every file under it counts as **specifically claimed** — for `index coverage`,
and for `C-002` when `[coupling] require_ownership` is on. That is the intended
instrument for retiring coverage debt across a directory: claim the subtree,
rather than enumerating its files and re-enumerating them every time one is
added.

The trailing slash is load-bearing. Without it the same string is an exact file
claim and matches nothing, since no file is named `src`.
