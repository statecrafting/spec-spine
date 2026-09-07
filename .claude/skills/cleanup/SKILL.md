---
name: cleanup
description: "Run dead-code and duplicate-code detection across the source surface with one read-only analyzer agent, investigate each finding in context, and return categorized recommendations that respect spec ownership."
allowed-tools: Agent, Read, Bash, Glob, Grep, Edit
---

# /cleanup: Cleanup Analysis

## Purpose

Spawn one analyzer sub-agent that runs dead-code and duplicate-code
detection across the source surface `AGENTS.md` or `CLAUDE.md` names,
reads each finding in context, and returns a structured report. Optional
detectors are used when available and skipped visibly when not.

## Usage

```
/cleanup              # dead code + duplicates
/cleanup dead-code
/cleanup duplicates
```

## Execution

### Step 1: parse arguments

`$ARGUMENTS` selects detectors; default is both. Valid tokens: `dead-code`,
`duplicates`.

### Step 2: spawn the analyzer

Use the `Agent` tool (type `explorer`, read-only) with this prompt, passing
the selected detectors and the source surface:

---

You are a cleanup analyzer. Analyze and report; change nothing.

**Detectors to run:** [selected]
**Source surface:** [directories]

**A. Dead code.** Per language present, prefer the stack's own detector
and fall back visibly:

| Language | Detector | Fallback |
|---|---|---|
| Rust | `cargo clippy --workspace --all-targets --locked -- -W dead_code -W unused`, `cargo udeps` (nightly) | each crate's `[dependencies]` against its `use` lines; a file under `src/` no `mod` or `use` path references |
| TypeScript | `npx --no-install knip --no-exit-code` | an exported symbol no import references |
| Python | `vulture` | a module nothing imports |
| Go | `go vet ./...`, `staticcheck ./...` | an unexported identifier with one definition and no use |

Count the linter's unused findings separately from orphan files.

**B. Duplicates.** A duplicate detector if installed (`jscpd`, `simian`,
`cpd`); otherwise surface near-identical public function signatures
across the surface (`grep -rn "^pub fn\|^export function\|^def \|^func "`
then `sort | uniq -d`). Treat results as hints.

**C. Investigate every finding** by reading the source before
categorizing.

**D. Categorize.**

Keep (false positives): anything under the derived directory (compiler
output); generated code; trait or interface implementations reached only
through dynamic dispatch (the seams the project's rules name); public API
of a library consumed by another package; test fixtures and builders;
fuzz targets; workflow and hook scripts; entry points and registries; any
never-touch artefact the path-scoped rules name.

Safe to remove: private items the linter flags as never used with no
suppression; dependencies with zero usage in their package; files no spec
claims and nothing references (check `spec-spine index coverage` first).

Needs review: exported items flagged unused inside their package; files
recently added (`git log` shows planned work); ambiguous dependency usage
(a build script, a feature gate).

Duplicates by priority: high (more than 15 lines of logic), medium (10 to
15 lines of utilities), low (under 10 lines or test setup, keep).

**E. Return exactly this report:**

```markdown
## Cleanup Analysis Report

### Dead Code
#### Safe to remove
| Item | Type | Location | Owning spec | Reason |
#### Needs review
| Item | Type | Location | Owning spec | Context |
#### Keeping
| Item | Reason |

### Duplicate Code
#### High priority
- **[description]** ([N lines]): locations; recommendation
#### Medium priority
#### Keep as-is

### Detectors
- linter unused findings: N
- dead-code detector: ran / skipped: reason
- dependency audit: ran / skipped: reason
- duplicate detector: ran / skipped: reason

### Summary
- N safe to remove, N need review, N duplicate blocks, N confirmed intentional
```

Rules: read code before categorizing; be conservative; name the owning
spec of every path via `spec-spine registry show <id> --json` and
`spec-spine index coverage` (never parse `.derived/`); never recommend
removing a never-touch artefact, a fuzz target, or a seam implementation;
do not explore the codebase for problems beyond what the detectors find;
do not create any files; make no changes.

---

### Step 3: present the report.

### Step 4: offer next steps

Ask whether to remove the safe items, walk the review items, or keep the
report. Removing a spec-claimed path is a change to that spec's territory:
if the owner is the spec being implemented, its `establishes` list drops
the path in the same change; if the owner is a shipped spec, that is an
amendment recorded in a new spec (an `amends` edge), never an edit to the
shipped spec itself. Say so before removing anything.

## Project layer

Read from `AGENTS.md` or `CLAUDE.md`: the source surface. Read from
`.claude/rules/`: the dynamic-dispatch seams and never-touch artefacts.
Nothing here is edited per project.
