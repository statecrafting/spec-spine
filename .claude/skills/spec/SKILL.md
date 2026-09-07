---
name: spec
description: "Author a new spec from standards/spec/templates/spec-template.md at the next free ordinal, born status draft, validated with spec-spine compile and lint before it lands. Approval stays a human flip."
allowed-tools: Bash(spec-spine:*), Bash(git:*), Bash(mkdir:*), Bash(cp:*), Bash(mktemp:*), Read, Write, Edit, Glob, Grep
argument-hint: "[short title]"
---

# /spec: author a new spec

A new spec is a design change, not code. It is born `status: draft`, it
claims territory only in prose and typed edges, and it schedules nothing
until a human flips it to `approved` (`/next` never offers a draft). The
contract is `standards/spec/contract.md`; the shape is
`standards/spec/templates/spec-template.md`; the vocabulary of edges and
units is in the constitution and the contract.

## Step 1: gather the inputs

Ask for what is not in `$ARGUMENTS`; do not invent any of these:

- **title** (short, imperative) and a **slug** (kebab-case, from the
  title)
- **kind** and **domain**, when `spec-spine.toml` closes those enums
  (`[kind] allowed`, `[domains] allowed`); read the lists from the file,
  never from memory
- **risk**: `low`, `medium`, `high`, `critical` (critical touches a
  hashed byte, a trust decision, or a never-touch artefact)
- **depends_on**: existing ids only, checked against
  `spec-spine registry list --ids-only`. A dependency must be lower-numbered
  unless the project says otherwise.
- **summary** (one paragraph) and the **territory** (the files, sections,
  or symbols it will own), enough to fill `establishes` with real paths
- any extra frontmatter keys the project declares in `[frontmatter]
  extra_known_keys` (a wave, a phase, a platform)

## Step 2: pick the ordinal

```sh
spec-spine registry list --ids-only
```

The id is the next free three-digit ordinal after the highest in the
registry, joined to the slug: `NNN-slug`. Never derive it from
`ls specs/`: a stale checkout collides. Projects that partition ordinals
into ranges (waves) say so in `AGENTS.md`; follow that and refuse when
the range is full (renumbering is a human decision). The directory name
must equal the id.

## Step 3: create the file

```sh
mkdir specs/<id>
cp standards/spec/templates/spec-template.md specs/<id>/spec.md
```

Then edit `specs/<id>/spec.md`:

- Frontmatter: `id`, `title`, `status: draft`, `kind`, `created` (today,
  `YYYY-MM-DD`), `implementation: pending`, `risk`, `depends_on`,
  `summary`, and the project's extra keys. Replace the commented typed-edge
  examples with real `establishes` paths (the manifest, every source
  file, every test, fixtures as a subtree) and any `extends`,
  `constrains`, or `references` edges the territory needs. Drop the
  template's inline comments.
- Body: the numbered sections in template order (purpose, territory,
  behavior with MUST/SHOULD/MAY, functional requirements, acceptance
  criteria, out of scope, resolved decisions, `## Verification`). The
  Verification block holds the `verify:cli` commands that prove the
  acceptance criteria; `scripts/verify-spec.sh` runs them after merge, so
  each must exist once the spec is built.
- No em dash anywhere in the text.

CHECKPOINT when an edge touches another spec's territory (an `extends`,
`amends`, or `supersedes`): present the frontmatter before writing the
body, since the edge is a claim about someone else's unit.

The `PostToolUse` hook recompiles the registry after a spec edit; that is
expected.

## Step 4: validate

Compile and lint a copy so nothing is judged against a half-written
working tree, then the real tree:

```sh
T=$(mktemp -d) && cp -R spec-spine.toml standards specs "$T"/ \
  && spec-spine compile --repo "$T" && spec-spine lint --fail-on-warn --repo "$T"
```

`--fail-on-warn` is what the gate runs. If `compile` complains about a
path referenced from outside `specs/` (a `references` edge into a docs
directory), add that directory to the copy and re-run. Fix every
diagnostic in the real file, re-copy, re-run, until both exit 0. A
dependency cycle is refused by `compile` itself (`V-014`).

Then the real gate, which regenerates and checks the committed shards:

```sh
spec-spine compile && spec-spine index && spec-spine lint --fail-on-warn && spec-spine index check
spec-spine registry plan
```

`plan` shows where the new id sits: blocked on its dependencies, or ready
the moment a human approves it.

## Step 5: commit and report

On a feature branch named after the new id (`git switch -c <id>`),
`/commit` as `docs(<NNN>): draft spec <id>` with the derived shards staged
alongside, then `/ship` when the draft is ready for review.

Report the id, kind, risk, dependencies, the units claimed, and:

- `status: draft` is deliberate. Approval (`status: approved`) is a human
  flip made in the file after review; nothing in this skill or in a
  driven session performs it.
- If the project keeps a sequencing plan in a thesis spec, adding the new
  id there is a change to that spec and a human call; say so rather than
  editing it.

## Project layer

Read from `spec-spine.toml`: the closed enums and extra keys. Read from
`AGENTS.md`: any ordinal-range convention and the thesis spec, if one
exists. Nothing here is edited per project.
