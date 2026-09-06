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
      - run: spec-spine compile --check     # validation + registry freshness
      - run: spec-spine index check         # staleness gate (exit 2 if stale)
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

Both freshness steps (`compile --check` and `index check`) compare against
**committed** artifacts, so this job assumes you commit `.derived/` as described
in §3. If you gitignore the derived tree instead, drop both `--check`/`check`
forms and run plain `spec-spine compile` and `spec-spine index` to build the
artifacts in-job: with nothing committed to compare against, the freshness gates
would report every shard missing and fail permanently.

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
| `index.extra_hashed_inputs` | globs folded into the staleness content hash, beyond the always-hashed core | `["standards/**", ".github/workflows/**"]` |
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
extra_hashed_inputs = ["standards/**", ".github/workflows/**", "schemas/**"]
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
- CI runs `compile` → `index check` → `lint` → `couple` on every PR.
