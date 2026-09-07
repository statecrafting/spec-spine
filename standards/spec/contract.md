# spec-spine contract (normative summary)

A one-page operational summary of the bootstrap spec
(`specs/000-spec-spine-bootstrap/spec.md`), for quick reference. The bootstrap
spec and the constitution are authoritative; where this summary is terser, they
govern.

## Inputs (authored truth: markdown only)

- `specs/NNN-slug/spec.md`: one spec per directory; directory name equals `id`;
  `NNN` is a unique three-digit ordinal.
- `standards/spec/`: this constitution, this contract, and templates.
- `spec-spine.toml`: the repo's configuration (all keys optional; absent ⇒
  conventional single-Cargo-workspace defaults).

## Outputs (machine truth: compiler-owned JSON, read via the typed consumer only)

Since spec 024 both views are committed as **per-unit shard trees**; the
aggregate view is recomputed from the shard set on read, never committed.

- `<derived_dir>/spec-registry/by-spec/<id>.json`: spec-as-source shards (output of `compile`).
- `<derived_dir>/codebase-index/by-spec/<id>.json` and `.../by-package/<slug>.json`:
  code-as-source shards (output of `index`).
- `<derived_dir>/**/build-meta.json`: wall-clock metadata (the only
  non-deterministic artifact; gitignored; excluded from determinism/golden checks).

## Required frontmatter

`id`, `title`, `status` (`draft`/`approved`/`superseded`/`retired`), `created`
(`YYYY-MM-DD`), and `summary`. Everything else is optional.

## Typed edges (8; `references` is the only non-owning one)

`establishes`, `extends`, `refines`, `supersedes`, `amends`, `co_authority`,
`constrains`, `references`. `origin` is a bootstrap marker, not an edge.

## Amendment authoring

An `amends` edge is declared **once, in the amending spec's frontmatter**. The
amended spec's `spec.md` is not edited to record that it has been amended: its
text is the contract as it stood, and rewriting it to mention a successor is how
history stops being queryable (constitution V). The inbound view is a compiled
read, `spec-spine registry relationships <amended-id>`, which reports
`amended_by (incoming)`. See spec 040.

## Amending the constitution

`standards/spec/constitution.md` is tier 2 and is **not** changed by an `amends`
edge: `amends` resolves to spec ids and the constitution is not a spec. An
approved ordinary spec changes it by claiming the affected text as a section
unit of that file (`establishes` a new principle, `refines` one it tightens with
a named `aspect`, `co_authority` on one genuinely shared) and by contradicting
no `specs/000` `unamendable` anchor. The constitution is a standing statement,
so unlike an amended `spec.md` it is edited in place. The file is on the gate's
bypass floor, so the claim is a ledger fact rather than a `C-001` refusal. See
spec 043.

## Authority units

`file` (bare string shorthand; trailing slash ⇒ subtree), `section`
(`{file, anchor}`), `symbol` (`{id}`, resolved by the indexer), and `directory`/
`crate`/`module` (added by spec 017 as an additive minor, no schema-file edit).
Symbol resolution covers Rust + TS in v1; Python is deferred.

## Lifecycle as scheduling

Two frontmatter keys decide whether a spec is offered as work and how strictly
its claims are held. `status` is `draft` / `approved` / `superseded` / `retired`;
`implementation` is `pending` / `in-progress` / `complete` / `n-a` / `deferred`,
or absent.

| `status` | `implementation` | schedulable | unresolved unit is |
|---|---|---|---|
| `draft` | absent, `pending`, `in-progress` | yes | `W-001` warning |
| `approved` | `pending`, `in-progress` | yes | `W-001` warning |
| `approved` | absent | no (settled, spec 045) | error |
| any | `complete` | no | error (spec 041) |
| any | `n-a`, `deferred` | no | takes its answer from `status` |
| `superseded`, `retired` | any | no | takes its answer from `status` |

Three sentences make the table usable.

**`approved` plus `pending` is a work order.** It is the state a specify-first
corpus lives in for months, and it is the state `spec-spine registry plan`
offers as ready.

**`draft` is never a claim about code.** A draft's unresolved units are expected,
which is why they warn (`W-001`) instead of refusing. Spec 044 defines that
window.

**An absent `implementation` is not a third value.** It defers to `status` (spec
045): on a `draft` it reads as `pending`, and on anything ratified it reads as
settled. That is what keeps a bootstrap spec owning no code from being offered
as ready forever, and it is why `n-a` exists for a record spec that is ratified
and owns nothing.

The specs behind the rows are 041 (completion held to claims), 044 (in-progress
is in flight) and 045 (absent implementation defers to status). This page is a
summary; those are the argument.

## Extra keys

`frontmatter.extra_known_keys` in `spec-spine.toml` declares frontmatter keys
this corpus recognizes beyond the grammar. A declared key stops the conformance
lint warning about it, and its value is preserved verbatim into the registry as
`extraFrontmatter` (spec 013), so a consumer can read it.

The config lists the names and records nothing about what they mean. An adopter
who declares keys should write down their semantics, in their own constitution
or contract, next to the rest of what governs the corpus.

## The gate chain

`compile` → `index` → `lint` → `couple`: the coupling gate refuses a merge where
a claimed code unit and its owning spec disagree, unless a scoped
`Spec-Drift-Waiver:` reason is present in the PR body. Configured bypass prefixes
(docs, lockfiles, `.derived/`) are exempt.

## Determinism

Pure function of `(config, file contents)` → byte-identical output; the ledger is
diffable and mechanically mergeable; staleness is detected by content-hash
comparison alone.
