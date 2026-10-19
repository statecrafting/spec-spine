# Reference documentation

`spec-spine reference` (spec 166) generates one Markdown page per exported
spec, a pure function of declared inputs, and checks committed pages against
a fresh in-memory generation. A page restates what a spec declares, owns and
relates to. It does not say that code is correct, that a test ran, or that
anything was accepted or released. Tutorials, navigation and other narrative
documentation stay authored and outside this product.

```text
spec-spine reference generate --request <FILE> [--json]
spec-spine reference check --request <FILE> [--json]
spec-spine reference model --request <FILE> --json
```

## The request

```json
{
  "schemaVersion": "1.0.0",
  "outputDir": "docs/reference",
  "specs": { "select": "all" },
  "traceability": true,
  "catalog": "docs/reference/catalog.json",
  "limits": { "maxPages": 1024, "maxPageBytes": 524288, "maxSignatures": 256 }
}
```

- `specs` is `{ "select": "all" }` or `{ "ids": [...] }` with full ids only.
- `outputDir` is a normalized repository-relative directory. It may not lie
  inside the configured derived or state directory, or cross a symbolic link.
- `traceability` supplies the spec 169 `relations` field.
- `catalog` names a committed copy of a spec 162 catalog (the `catalog`
  member of `capabilities --json`). The running binary's own catalog is never
  read, so page bytes do not depend on the generator's version.
- `limits` may only lower its defaults.

## A page

Every page is `<outputDir>/<spec-id>.md` and holds twelve regions, always
present and always in this order:

| Region | Source |
|---|---|
| `header` | the registry record |
| `lifecycle` | the registry record |
| `requirements` | the spec's obligations |
| `ownership` | its owning units by edge, each unit's index resolution, and its non-owning references |
| `signatures` | the spec 155 `signature` projection of each owned symbol, or its omission reason |
| `relations` | spec 169 relations whose source obligation is this spec's |
| `operations` | catalog operations whose `governedBy` names this spec, with their effects and examples |
| `evidence` | verification inputs, and the `verify:cli` commands, declared and not executed |
| `limitations` | the spec's non-goals and a fixed limitation statement |
| `changes` | outgoing amends and supersedes edges, and incoming amends, supersedes and extends |
| `inputs` | the identities and digests the page was generated from |
| `notes` | human-authored text |

An empty field reads `None declared.`, and a source the request does not
supply reads `Not supplied.`. Neither is ever written as untested or
undocumented. Operations bind only through the catalog's own `governedBy`,
never by path or ownership.

Each region sits between two whole-line delimiters:

```text
<!-- spec-spine:begin generated id="lifecycle" -->
<!-- spec-spine:end generated id="lifecycle" -->
```

The parser recognizes a delimiter only outside fenced blocks. Every generated
fence is one backtick longer than any run inside it, and free text is escaped,
so no generated value can end a region or break a table. `notes` uses
`authored` in place of `generated` and is always the last region. Generation
copies its bytes forward verbatim (after LF normalization) and rewrites every
other region. A notes region holding a line that reads as a delimiter, an
oversized notes region, or a missing or repeated one refuses generation
without writing anything.

Pages carry no timestamp, revision, hostname or generator version. They are
UTF-8, LF, with no trailing whitespace and one trailing newline, and they are
byte-identical across platforms for equal inputs.

## Generating and checking

`reference generate` needs a fresh ledger; a stale one is a finding (exit 1)
and nothing is written. It builds every page and checks every limit before
writing any. A page-pattern file whose spec is no longer exported is an
orphan: one with empty notes is removed, and one with notes refuses (exit 2)
so human text is never deleted. A limit excess refuses the whole export.

`reference check` never writes. It exits 0 when every page is current, and 1
with these findings, ordered by path then kind:

- `page-missing`;
- `page-changed`, naming the differing regions;
- `page-orphaned`;
- `page-malformed`;
- `ledger-stale`.

Findings name paths, regions and digests, never page content. `--json` writes
the check document on success, and the verdict envelope with the check as its
report on a finding.

`reference model --json` emits every page model as one document. That
document and the check result are on the `reference-page` schema axis
(`REFERENCE_PAGE_SCHEMA`).

## Examples

A page renders each catalog example's `id`, `fixture`, `argv`, `stdin`,
`exitCode` and `stdoutIncludes` exactly as declared. No reference verb runs
an example. Running them is a separate acceptance step against the built
binary.
