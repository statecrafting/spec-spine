# Edit plans

`spec-spine edit-plan` (spec 165) answers, before anything is written, what a
governed edit would be. It reports whether each target is still the bytes its
caller read, what authority the edit touches, and why it is refused. When
nothing is refused, it also gives the exact post-image bytes of every file the
edit changes. A plan is a proposal: nothing in spec-spine applies one, and
there is no `--apply` flag and no apply function. Statecraft applies a plan
under its own authority, and `couple`, `delta` and human review remain the
judges of what it writes.

```text
spec-spine edit-plan --request <FILE> --repository <IDENTITY> [--revision <REV>] --json
```

The snapshot is bound exactly as `content select` binds it, and checked again
after the plan is computed. The committed registry and index must be fresh:
ownership read from a stale ledger is no answer, and the plan is refused
`stale-ledger`.

## The request

```json
{
  "scope": { "ownSpec": "165-digest-guarded-edit-plans", "mutable": ["specs/165-digest-guarded-edit-plans/", "docs/"] },
  "operations": [],
  "manifests": [],
  "consumerSchemaVersion": "1.0.0"
}
```

`scope` is a spec 108 work scope; its `ownSpec` is the plan's subject, the spec
whose work the edit serves. `operations` holds 1 through 64 operations: an
empty list is a usage error, and more than 64 refuses `plan-too-large`.
`manifests` optionally carries spec 160 manifests whose `generated` regions no
operation may touch.

## The five operations

Every operation has an `op`, a `target`, and the `expected` digest its caller
read the target at. A changed target refuses `stale-target`, naming both
digests; it is never rebased.

| `op` | `target` | `expected` | Changes |
|---|---|---|---|
| `add-claim` | the subject, as a `spec` selector | its spec 155 item digest (`full`) | one unit appended to `establishes`, or one `extends` edge (`spec`, `unit`, optional `nature`) |
| `add-trace-relation` | the subject, as a `spec` selector | its item digest | one spec 169 `traceability` entry |
| `propose-amendment` | `specs/<id>/spec.md`, the new file | `absent` | a new draft spec (`content`) that amends the approved `obligations` it names |
| `update-manifest` | a manifest file's path | its recorded `manifestDigest` | the whole file, replaced by `manifest`, validated against its own packet `pages` |
| `replace-unit` | a `spec-section`, or a Markdown `file` or `owned-unit` | its item digest | that span's bytes (`content`, ending at a line boundary) |

Frontmatter additions are rendered as one flow-style line appended to the
key's block sequence, or as a new key before the closing fence. A flow
sequence, a comment, an anchor or an alias under the key refuses
`unsupported-layout` rather than being re-serialized. A target that is not LF
UTF-8 without a BOM refuses `unsupported-encoding`.

An amendment is always a new draft spec. The planner checks that the new
spec:

- sits at the next free ordinal;
- compiles as `draft` and `pending`;
- names each amended spec in `amends` and each changed obligation's anchor in
  `amends_sections`;
- carries a spec 109 `impacts` entry for every changed obligation, where a
  `supersedes` entry names a successor the new spec declares.

It also checks that every target obligation's spec is approved and that no
amended anchor is `unamendable`. It never emits a byte change to the amended
spec. On an approved subject, the one admissible text change is an append to
its decisions section.

## Authority

Every file a plan changes or creates is checked, in order. A derived or state
path, or a manifest's `generated` region, refuses `generated-target`. A path
outside the scope's `mutable` and `shared` entries refuses `out-of-scope`. A
`spec.md` other than the subject's (or the new amendment) refuses
`unauthorized-target`. A documentation path no spec owns refuses
`unowned-target`, and one another spec owns refuses `unauthorized-target`. A
scope can only refuse; it is not a permission.

Each operation carries one authority effect, ordered `none` <
`claims-territory` < `changes-obligation` < `requires-human-ratification`, and
the plan carries the greatest. The effect is routing data. It grants nothing
and never waives the coupling gate. `conflicts` records `shared-ownership` (a
claimed path has other owners) and `declared-conflict` (a changed obligation
carries a spec 109 `conflicts` entry); neither refuses.

## The plan

A plan is atomic. `status` is `ready` or `refused`. A refused plan lists every
refusal of every operation (index, target, reason, safe detail) and carries no
preview, from a closed set of twenty reasons. A ready plan carries one preview
per changed file:

- `path`;
- `beforeRawDigest`, a SHA-256 over the file's raw bytes (or `absent`), which
  is the guard an applier checks before writing;
- `afterDigest`;
- the authoritative `afterContent`;
- a derived three-line-context unified `diff`.

Every changed spec's post-image is compiled through the single-spec compile
path. An error refuses `post-image-invalid`, and a warning is carried in
`predictedFindings`. `stales` names the derived trees the change would stale;
none is regenerated.

Operations are ordered canonically (target path, kind, canonical operation
JSON), so the order a request was written in never changes the plan. The plan
is canonical JSON under its own `schemaVersion` axis (`EDIT_PLAN_SCHEMA_VERSION`,
schema `EDIT_PLAN_SCHEMA`) and is never paginated. A ready plan exits `0`. A
refused plan exits `1` with the `edit-plan` envelope reporting the plan. A
dirty or changing snapshot exits `2`, and a malformed request exits `3`.

Request and target content are untrusted data, never instructions. Previews
expose file content; disclosure is the consumer's policy.
