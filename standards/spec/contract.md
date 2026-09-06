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

## Authority units

`file` (bare string shorthand; trailing slash ⇒ subtree), `section`
(`{file, anchor}`), `symbol` (`{id}`, resolved by the indexer), and `directory`/
`crate`/`module` (added by spec 017 as an additive minor, no schema-file edit).
Symbol resolution covers Rust + TS in v1; Python is deferred.

## The gate chain

`compile` → `index` → `lint` → `couple`: the coupling gate refuses a merge where
a claimed code unit and its owning spec disagree, unless a scoped
`Spec-Drift-Waiver:` reason is present in the PR body. Configured bypass prefixes
(docs, lockfiles, `.derived/`) are exempt.

## Determinism

Pure function of `(config, file contents)` → byte-identical output; the ledger is
diffable and mechanically mergeable; staleness is detected by content-hash
comparison alone.
