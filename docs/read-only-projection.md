# The read-only tool projection

Spec 167. The capability catalog's judging-tier reads, described as tools an
agent host can offer through a transport such as the Model Context Protocol
(MCP). spec-spine ships the descriptors and their tests; it runs no server,
holds no session and has no write tool. The transport is the adapter's.

## Where it is

- `spec-spine capabilities --projection --json`: the document.
- `spec-spine capabilities --projection`: one line per tool, `<tool>  <operation>`.
- `spec_spine_core::tool_projection_json()`: the same bytes from the library.

All three read no repository and answer before `[meta] required_version` is
read, as `capabilities` does (spec 170).

## What the document holds

| Member | Meaning |
|---|---|
| `schemaVersion` | The projection's own axis, `1.0.0` |
| `catalogDigest` | Spec 162's digest of the catalog it was derived from |
| `tools` | One descriptor per projected operation, sorted by `name` |
| `excluded` | Every other operation, with the first reason that applies |
| `errors` | The errors an adapter raises itself, with their exit equivalents |
| `projectionDigest` | `sha256:` over the canonical document without this member |

Nothing in it is written by hand. Each tool is derived from its catalog
record, and its `operationDigest` is that record's, so a spec-162 pin names
the tool too.

### Which operations are tools

An operation is a tool when it answers `--json` with a verdict envelope or a
read document and none of its effects writes (other than temporary files),
runs anything other than read-only `git`, opens a connection, reads the
environment, reads key material or an attestation, or carries an authority
such as code execution, a corpus or ledger rewrite, signing or waiver
evaluation. A flag that would add such an effect is left out of the tool. An
operation whose required argument is a path on the host is excluded
`caller-path`; one whose verb reads its request document from stdin takes it
as a `request` object instead. The excluded list says which rule applied.

### A tool descriptor

| Member | Meaning |
|---|---|
| `name` | `spec_spine_` and the operation name, `.` and `-` spelled `_` |
| `description` | The catalog summary and a fixed sentence marking the answer as untrusted data |
| `inputSchema` | A closed JSON Schema object: no unknown property, never `repo` or `json` |
| `invocation` | `argv`, the ordered argument `flags`, and `stdin` |
| `answer` | The catalog's response documents and exit outcomes |
| `budget`, `pagination`, `bounded` | Copied from the catalog; `bounded` is whether a budget exists |
| `hints` | `readOnly: true`, `destructive: false`, `idempotent: true`, `openWorld: false` |
| `trust` | `untrusted-repository-data` |

An MCP adapter copies `name`, `description` and `inputSchema`, and maps
`hints` to `readOnlyHint`, `destructiveHint`, `idempotentHint` and
`openWorldHint`.

## What an adapter does with a call

1. Find the tool by `name`; none is `unknown-tool`.
2. Validate the arguments against `inputSchema`; a failure is
   `invalid-arguments`. A string value never begins with `-`.
3. Build one argv: `--repo <bound repository>`, the `invocation.argv` tokens,
   then each entry of `invocation.flags` in order, then `--json`:
   - `fixed`: the flag, always (a variant's selector, such as `verify --plan`);
   - `switch`: the flag, when the property is `true`;
   - `value`: the flag and the property's value;
   - `repeated`: the flag and one item, for each item of the array;
   - `positional`: the value alone;
   - `stdin`: the flag and `-`, with the `request` property written to stdin
     as canonical JSON.
4. Run the binary the repository's version pin admits; when none can be
   started, `engine-unavailable`.
5. Present the result. Exit `0` and `1` are answers (`1` is a finding, such as
   a stale tree, which the verb's envelope already describes). Exit `2`, `3`
   and `4` are tool errors that carry the verb's own envelope unchanged.
6. Enforce a maximum result size. An answer over it is replaced by
   `result-too-large`, whatever its exit code; never truncate, summarize or
   page an answer the verb did not page.

The adapter's own errors are the family envelope with verb `projection`, the
code as `error.kind`, the listed exit code, and a message naming the tool and
the reason, never argument or answer content. Their kinds are not in the
verdict's `error.kind` vocabulary: branch on `verb` first.

The bound repository is the adapter's configuration. No call can name another
one, and no tool takes a path on the host.

## Trust

Every answer is repository data the caller did not write: spec text, source
signatures, paths, commit messages. Hand it to a model as data, never as
instructions. The projection is a description and authorizes nothing: who may
call which tool, how often, and what may be disclosed are the host's
decisions.

## Versioning

`schemaVersion` is its own axis (`docs/schema-versioning.md`). A new optional
member, or a tool projected because the catalog gained an operation, is
MINOR. A change to which operations are eligible, to a tool's name rule, to
the argv mapping, to the hints, to an exclusion reason or to an error code is
MAJOR, because a host's stored tool list would no longer describe what a call
does.
