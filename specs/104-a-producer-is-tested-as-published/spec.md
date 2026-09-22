---
id: "104-a-producer-is-tested-as-published"
title: "A producer is tested as published"
status: approved
kind: "tooling"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "092-the-engine-ships-governance-not-an-environment"
summary: >
  The published spec-spine-core 0.21.0 emits AGENTS.md and a `.claude/rules/`
  tree; the current source under the same version string emits neither. Both
  states passed a green workspace suite, because "the workspace is green" and
  "the artifact a consumer installs behaves this way" are different claims and
  only the first was ever tested. The producer contract is asserted against the
  packaged crate, through its public facade, from outside the workspace, and
  that check gates a release.
establishes:
  - { kind: file, path: "scripts/verify-packaged-producer.sh" }
extends:
  - spec: "006-distribution"
    unit: { kind: file, path: "docs/releasing.md" }
    nature: additive
references:
  - unit: { kind: file, path: "crates/spec-spine-core/tests/scaffold.rs" }
    role: "context"
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
---

# 104: A producer is tested as published

## 1. Purpose

One version string names two different producers.

Measured 2026-09-21, directly, by building a consumer against each and calling
the public facade:

| | published `spec-spine-core` 0.21.0 | this source tree, also 0.21.0 |
|---|---|---|
| governance files emitted | 11 | 7 |
| `AGENTS.md` | emitted, 4,505 bytes | not emitted |
| `.claude/rules/*.md` | three files emitted | not emitted |
| `.gitignore` | whole-file write, no marker | `append: true` with an append marker |

Statecraft is implementing against the published artifact and receives the
first column. Spec 092 removed the agent harness from this engine and the
workspace suite asserts that removal in
`crates/spec-spine-core/tests/scaffold.rs`: twenty-one tests, including one
that names the exact file set and one that refuses any agent or environment
artifact. Every one of them is green.

They are green **against the workspace**. Nothing has ever exercised the
packaged crate a consumer actually installs, so the workspace suite could not
have caught this and did not.

### 1.1 What the defect actually is

Not the emitted content, which is correct here and was corrected deliberately.
The defect is that the correction shipped with no version of its own, so
`0.21.0` is the identity of two behaviors. The fix for that instance is a new
version, which is release work rather than a spec. What this spec fixes is the
absence of the check that would have made the mismatch visible before a
consumer found it.

## 2. Territory

One new script, `scripts/verify-packaged-producer.sh`, and the release runbook
step that calls it.

It claims no library code. The producer's behavior is spec 092's and is not
changed here: this spec adds an assertion about the packaged form of a contract
another spec owns.

## 3. Behavior

### 3.1 The check runs against the packaged crate, not the workspace

`scripts/verify-packaged-producer.sh` MUST:

1. `cargo package` the producer crates from the tree under test, which is the
   same operation a publish performs;
2. unpack the resulting `.crate` archives into a directory **outside** the
   workspace;
3. build a consumer crate that depends on the unpacked sources by path and on
   nothing in the workspace;
4. call `spec_spine_core::scaffold_init_json` and assert §3.2;
5. print the source commit, each package's file name and its SHA-256, and exit
   non-zero on any failed assertion.

Step 3 is the load-bearing one. A test inside the workspace reads the
workspace's sources however the package manifest is written, so it cannot see
an `exclude` rule, a missing `include` entry, or a path that does not survive
packaging. Those are exactly the ways a packaged crate diverges from the tree
it was cut from.

### 3.2 What the packaged producer must satisfy

Asserted through the public facade only. No private item is reached for, so the
check is a statement about the surface a consumer has.

1. **Exact allowed governance output.** The emitted `relPath` set equals, with
   no extra and no missing member: `spec-spine.toml`, the constitution, the
   contract, the two authoring templates, the bootstrap spec, and `.gitignore`.
   Equality, not containment: a containment check is what lets an extra file
   ride along unnoticed, which is this spec's entire subject.
2. **Absence of harness, root instruction, hook, CI and Makefile output.** No
   `AGENTS.md`, `CLAUDE.md`, `Makefile` or `.mcp.json`; nothing under
   `.claude/`, `.agents/`, `.codex/`, `.github/` or `.githooks/`. Checked by
   name **and** by content, because a harness that returns under a different
   filename is the same regression: no emitted file may contain a hook-event
   token (`SessionStart`, `PostToolUse`, `PreToolUse`) or describe a slash
   command.
3. **Append-marker behavior.** `.gitignore` is `append: true` with a non-empty
   `appendMarker`, and the marker appears in the appended contents so a second
   application is idempotent. Every other file is `append: false` with a null
   marker. No file is `overwrite: true`.
4. **Determinism and documented purity.** Two calls with the same argument
   return identical bytes, and a call made from an empty directory leaves that
   directory empty.
5. **Statecraft's requested layout.** With `derived_dir` and `state_dir` set to
   the managed roots, both reach the scaffolded `spec-spine.toml`, the
   scaffolded `.gitignore` follows the configured state root, and the emitted
   file set is unchanged.

### 3.3 The configuration argument is snake_case, and that is asserted

The facade's `Config` input deserializes **snake_case**, matching
`spec-spine.toml`, while every DTO the library emits is camelCase. A consumer
that assumes one casing throughout gets a config error at exit 3 on its first
call.

The check MUST assert that a camelCase key is **refused** rather than silently
ignored, which pins the asymmetry instead of leaving a consumer to find it. It
is asserted, not changed: `Config` is the TOML model, `deny_unknown_fields`
makes the mistake loud, and renaming the boundary is a breaking change to a
type another spec owns.

### 3.4 The check gates a release

`docs/releasing.md`'s pre-flight MUST carry the script as a checklist item,
alongside the existing `cargo package --workspace --locked` line. Packaging
already proves the crate builds from its packaged sources; this proves it
**behaves** correctly from them, which is a different question and the one that
was not being asked.

It is deliberately **not** in the gate chain. It shells out to `cargo package`
and compiles a consumer, so it is minutes rather than seconds, and the chain
runs on every pull request. Release preparation is the right cadence for a
check about what a release contains.

### 3.5 Evidence is recorded, and is not a publication claim

A run MUST record the source commit, each package's identity and SHA-256, and
the commands. A recorded packaged-candidate run says that an artifact built
from a named commit behaved correctly. It says nothing about whether anything
was published, and the record MUST NOT be worded as though it did.

## 4. Out of scope

- **Changing what the producer emits.** That is spec 092's, and it is correct.
- **Testing the packaged CLI or the npm and PyPI shims.** They ship prebuilt
  binaries assembled at publish time and have their own verification in specs
  006 and 007. This spec is about the library facade Statecraft consumes.
- **Publishing anything.** §3.5.
- **Renaming the facade's configuration casing.** §3.3.
- **A general "the package matches the workspace" property.** The producer is
  the surface with a named external consumer and a measured divergence; a
  wider claim is a different spec with its own evidence.

## 5. Resolved decisions

**D-1 (2026-09-21, build: the check is proven by running it against the
published crate).** An acceptance that only ever passes is not evidence. The
same consumer, built against `spec-spine-core = "=0.21.0"` from crates.io,
fails seven assertions and names the four extra files:

```
FAIL emits exactly the allowed governance file set
     extra:   [".claude/rules/adversarial-prompt-refusal.md",
               ".claude/rules/governed-artifact-reads.md",
               ".claude/rules/orchestrator-rules.md", "AGENTS.md"]
FAIL emits no AGENTS.md
FAIL emits nothing under .claude/
FAIL .gitignore is an append, not a whole-file write
FAIL the append carries a non-empty marker
FAIL the marker is inside the appended contents, so re-applying is idempotent
FAIL the managed layout emits the same file set and nothing more
```

The published release is the negative control this check exists for, which is
the strongest fail-first evidence available: not a mutated fixture, but the
actual regression, caught.

**D-2 (2026-09-21, build: no `spec-spine.toml` edge is needed).** The filed
draft declared an `extends` edge onto the configuration, expecting `L-008` to
demand a hashed input for the newly claimed script. It does not: `scripts/*.sh`
is already in `[index] extra_hashed_inputs`, so the claim is witnessed the
moment it is made and `lint --fail-on-warn` is clean. The edge was removed. An
ownership claim that buys nothing is not free: it makes the configuration look
like this spec's territory to everyone who reads the frontmatter afterwards.

**D-3 (2026-09-21, build: the consumer is patched to the packaged types crate,
not the workspace one).** The packaged `spec-spine-core` depends on
`spec-spine-types` by registry version, which would resolve to whatever is
published rather than to the candidate. `[patch.crates-io]` points it at the
unpacked packaged types crate, so nothing in the consumer reaches the registry
or the workspace and the two halves under test are both the packaged ones.

**D-4 (2026-09-21, build: absence is checked by content as well as by name).**
A name list refuses `AGENTS.md`. It does not refuse the same harness returned
as `INSTRUCTIONS.md`. The check therefore also refuses any emitted file whose
contents carry a hook-event token or describe a slash command, which is the
property that actually matters and the one a rename cannot evade.

## Verification

Behavioral. The script is the acceptance: it fails if the packaged producer
emits anything §3.2 forbids, and it is written so a missing assertion is loud
rather than silent.

Written to fail against the tree this spec is filed on: the script does not
exist, and the runbook does not call it.

```verify:cli
# 3.1: the check exists and is executable.
test -x scripts/verify-packaged-producer.sh
# 3.4: the release runbook calls it.
grep -q 'verify-packaged-producer.sh' docs/releasing.md
# 3.1 to 3.3: run it. It packages, unpacks outside the workspace, builds a
# consumer against the unpacked sources, and asserts every clause of 3.2 and
# 3.3. Any failure is a non-zero exit.
./scripts/verify-packaged-producer.sh
# The governed loop over the corpus this spec is part of.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
