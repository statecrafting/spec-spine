---
id: "097-governed-scope-is-declared-not-inferred"
title: "Governed scope is declared, not inferred"
status: draft
kind: "tooling"
created: "2026-09-15"
summary: >
  The ownership ratchet sees a file only if all four conjuncts of
  `coverage.rs::in_coverage_universe` hold: the extension is in `SOURCE_EXTS`,
  no path component is a resolver exclusion, the file lies inside a discovered
  package, and no bypass prefix matches. Measured on 2026-09-15: this
  repository tracks 572 files and the ratchet can see 89. Nineteen tracked
  files carry a `SOURCE_EXTS` extension and are invisible on the package
  conjunct alone, and eleven of those nineteen are files `[index]
  extra_hashed_inputs` already names, so a change to one restales all 101
  shards while `C-002` cannot ask who owns it. Seven of the nineteen already
  carry a valid `// Spec:` claim header that nothing ever reads. The two lists
  disagree about what this repository governs. This spec adds an opt-in declared scope: named
  path patterns that join the coverage universe whatever their extension and
  wherever they sit, with the tracked-file enumeration done by the CLI and
  passed in, because the core has no git. Empty by default, so no adopter's
  verdict changes on upgrade.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "005-coupling-gate"
  - "032-ownership-coverage"
  - "054-effective-config-is-a-governed-read"
  - "057-claimed-but-unwitnessed"
  - "094-a-claim-below-the-header-window-is-not-silent"
extends:
  # 3.2: the config table. `config.rs` is 000's floor territory; 032 already
  # carries the `[coupling] require_ownership` key that this one sits beside.
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  # 3.3 to 3.5: the universe, the classifier's input, and the report.
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-core/src/coverage.rs", nature: additive }
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-types/src/coverage.rs", nature: additive }
  # 3.6: the CLI enumerates tracked files and passes them in, the way 005
  # passes `DiffInput`.
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  # 3.7: effective config prints the declared scope like every other key.
  - { spec: "054-effective-config-is-a-governed-read", unit: "crates/spec-spine-cli/src/cmd_config.rs", nature: additive }
  # 3.8: the tests.
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-core/tests/coverage.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/tests/couple.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 097: Governed scope is declared, not inferred

## 1. Purpose

### 1.1 Four conjuncts, and only one of them is about code

`coverage.rs::in_coverage_universe` decides whether the ownership ratchet may
ask who owns a path:

```rust
has_source_ext(path)
    && !has_excluded_component(path, &cfg.index.resolver_exclusions)
    && index.packages.iter().any(|p| package_contains(&p.path, path))
    && !is_bypassed_path(cfg, index, path)
```

`C-002` fires only inside that set (`couple.rs` line 164), and
`index coverage` reports only that set, which is what makes the report predict
the gate. The third conjunct is the one nobody declared: a file is governable
because a Cargo or npm manifest happens to sit above it. That is a fact about
packaging, not about what this repository holds itself to.

### 1.2 What the conjuncts exclude here

Measured on 2026-09-15 at `0.19.0`, against this repository:

| Set | Count |
|---|---|
| Tracked files | 572 |
| In the coverage universe (`index coverage`) | 89 |
| Tracked, `SOURCE_EXTS` extension, **outside every discovered package** | 19 |
| Of those 19, named by `[index] extra_hashed_inputs` | 11 |
| Of those 19, **already carrying a valid `// Spec:` claim header** | 7 |

The nineteen are not incidental files. They are the three git hooks this
repository ships, the three the kit ships, `install.sh`, the release scripts
(`scripts/bump_version.py`, `scripts/gen-kit-embedded.py`,
`scripts/verify-spec.sh`), the PyPI shim's Python, and the website's
TypeScript. `scripts/gen-kit-embedded.py` is the generator behind
`kit_embedded.rs`, which `tests/scaffold.rs` pins; a change to it changes what
`init --with-kit` writes into an adopter's repository, and no spec is asked to
account for it.

The eleven are the sharp half. `spec-spine.toml` already declares them as
governance inputs, so editing one restales all 101 committed shards, and the
committing session must regenerate the whole ledger. The repository therefore
says, in one list, that these files are governance; and in another, that they
are not code, so nobody need own them. One of those two statements is wrong.

The seven are sharper still. These files already **claim themselves**, in the
documented way, and the claim is inert:

```
.githooks/enable-hooks.sh             # Spec: specs/090-.../spec.md
.githooks/enable-merge-driver.sh      # Spec: 020-derived-artifact-merge-driver
.githooks/merge-derived-index.sh      # Spec: 020-derived-artifact-merge-driver
kit/.githooks/enable-hooks.sh         # Spec: specs/090-.../spec.md
kit/.githooks/enable-merge-driver.sh  # Spec: 020-derived-artifact-merge-driver
kit/.githooks/merge-derived-index.sh  # Spec: 020-derived-artifact-merge-driver
py/scripts/smoke_test.sh              # Spec: specs/008-python-distribution/spec.md
```

Each header satisfies the recognizer spec 094 §3.2 declares, and each names a
spec that exists. `scan_comment_headers` never reads any of them, because it
walks discovered packages and these files are in none. An author followed the
documented mechanism exactly and got nothing, with no diagnostic saying so. The
only reason these headers have any effect at all is oblique: `kit/.githooks/`
is embedded verbatim into `kit_embedded.rs`, which **is** inside a package, so
the copies inside the generated Rust file are the ones the scanner reads (spec
094 §3.6 measures them there).

That is the finding this spec rests on. The question is not whether markdown
should be code. It is that seven files state their owner, in the format the
documentation gives, and the ratchet is looking somewhere else.

### 1.3 Why a longer extension list does not fix it

The obvious move, adding `md`, `yml`, `toml` to `SOURCE_EXTS`, fails on the
third conjunct and would misfire on the first. `AGENTS.md`, `CLAUDE.md`,
`.github/workflows/*.yml` and `spec-spine.toml` sit at the repository root or in
directories no package contains, so a longer extension list still cannot see
them. Meanwhile `SOURCE_EXTS` is shared with `scan_comment_headers` (spec 094
§3.2), so widening it also widens what the claim scanner reads, which is a
second change wearing the first one's clothes. Forty-four tracked governance
files fail the extension conjunct today; adding extensions reaches none of them
while changing the recognizer for all of them.

The rule the corpus needs is not "which extensions are code". It is "which
paths this repository governs", and that is a declaration, not an inference.

## 2. Territory

This spec adds a `[coverage]` table to `Config`, widens the coverage universe by
that table, adds one member to the coverage report naming what entered through
it, and adds the CLI-side tracked-file enumeration the widened universe needs.
It changes no committed artifact's schema: the scope is a config input and the
report is computed on read.

## 3. Behavior

### 3.1 Opt-in, and silent when unset

`[coverage] governed_scope` is empty by default. With it empty, every verb MUST
behave byte-for-byte as it does today: the same universe, the same coverage
counts, the same `C-002` set. An adopter upgrading past this spec and changing
no configuration MUST see no verdict change anywhere.

This is the same posture spec 032 took for `require_ownership` and spec 030 for
`auto_waive_dependency_only`: a corpus turns a ratchet on when it has retired
the debt the ratchet would refuse, never on upgrade.

### 3.2 The declaration

```toml
[coverage]
# Paths governed regardless of extension or package membership.
governed_scope = ["AGENTS.md", ".github/workflows/*.yml", "scripts/*", ".githooks/*"]
# Carved back out of the above, for generated or vendored files.
governed_scope_exclusions = ["kit_embedded.rs"]
```

Both are glob patterns matched against repo-relative POSIX paths, with the same
glob semantics `extra_hashed_inputs` uses, **including its trap**: `dir/**`
matches directories and therefore no files, and `dir/**/*` is what matches
files. Spec 069 fixed the shipped default for that reason and spec 079 found the
same form in `[index.slices]`; a third list with the same semantics MUST say so
in the same place, in the commented default `init` writes.

`governed_scope_exclusions` MUST be applied after `governed_scope` and MUST NOT
be able to remove a file that is in the universe for another reason. It carves
out of this spec's addition only; it is not a second bypass list.

### 3.3 What the declaration changes

A path matching `governed_scope` and not matching `governed_scope_exclusions`
MUST enter the coverage universe, bypassing the extension conjunct and the
package conjunct. The other two conjuncts still apply, and deliberately:

- **`resolver_exclusions` still wins.** A declared scope reaching into `target/`
  or `node_modules/` is a mistake, and honoring it would make the report
  machine-dependent.
- **The bypass floor still wins.** `docs/`, `.github/`, `README.md` and the rest
  of `couple.rs::DEFAULT_BYPASS_PREFIXES` are bypassed for coupling, and a
  file that is bypassed but counted as unclaimed debt is a coverage figure that
  can never reach 100% (the reasoning in `coverage.rs`'s own comment on
  `in_coverage_universe`). A corpus that wants `.github/workflows/` governed
  therefore has one honest move and this spec does not shortcut it: the file
  must stop being bypassed, which is a change to `[coupling] bypass_prefixes`
  the corpus makes deliberately, in the same commit, and which `config show`
  prints. §5 D-2 records why the alternative was rejected.

The consequence is worth stating plainly: of §1.2's nineteen files, the ones
under `docs/` or `.github/` are not reachable by this spec alone, and the git
hooks, `install.sh`, `scripts/` and `py/` are.

### 3.4 How a governed-scope file is claimed

The three ways a file is claimed (spec 032 §3.1, CLAUDE.md's "how a file gets
claimed") are unchanged, but only two of them reach every governed-scope file:

1. an `establishes` unit, or a unit carried on an `extends` edge: available for
   any path;
2. a package manifest floor: available only inside a package, so a
   governed-scope file outside every package can never be `FloorOnly`. It is
   `Specific` or it is `Unowned`, and the `C-002` message for it MUST NOT name a
   floor it does not have;
3. a `// Spec:` comment header: available only where the file's syntax has a
   line comment **and** its extension is in `SOURCE_EXTS`, because the scanner
   walks `SOURCE_EXTS` files inside packages (spec 094 §3.2). This spec does
   **not** widen the scanner. A `.md` or `.yml` file in the governed scope is
   claimed from frontmatter or it is not claimed.

That asymmetry MUST be documented rather than designed around. Widening the
claim scanner is a change to what claims, which spec 094 §4 reserves for a spec
that measures it first.

### 3.5 The report says which files entered this way

`CoverageReport` MUST gain one additive member, `declaredScopeFiles`, listing
the paths that are in the universe because of `governed_scope` and would not
otherwise be, sorted, omitted when empty. Omitted-when-empty is what keeps §3.1
true for a corpus that never sets the key.

The prose form of `index coverage` MUST report them on their own line, counted
and separable from the package totals, because their denominator is not a
package: a reader comparing "89/89 claimed" before and after must be able to
see which of the new files came from where.

The classifier MUST NOT change. A governed-scope file is `Specific` or
`Unowned` by exactly the rules in §3.4, and `--fail-on-untraced` refuses on
unclaimed files as it already does. This spec adds files to the denominator; it
adds no new refusal code and no new refusal condition.

### 3.6 The enumeration is the CLI's, not the core's

The universe is currently enumerated by walking package directories. A declared
scope can name a path in no package, so the walk must start from the repository
root, and a root walk meets `.git/`, untracked scratch files, build output and
whatever else a working tree holds. The filter that makes a root walk honest is
"is this file tracked", and that is a git question.

**The core MUST NOT run git.** This is the workspace invariant every
artifact-producing function already keeps. The CLI MUST enumerate tracked files
(`git ls-files -z`) and pass the list into the core as typed data, exactly as it
parses `git diff` into `DiffInput` for spec 005 and `git diff --name-only -z`
into `changed_path_names` for spec 088.

The core function MUST accept the list as an argument and MUST remain a pure
function of `(Config, file contents, file list)`. When no list is supplied (a
library caller, or a corpus not under git), the governed scope MUST be resolved
against a root walk filtered by `resolver_exclusions` alone, and the report MUST
say which of the two enumerations it used, because the two can differ and a
consumer comparing reports across them is comparing different denominators.

### 3.7 `config show` prints it

Both keys MUST appear in `config show`, like every other effective-config key
(spec 054). A declared scope that a corpus cannot read back is a governance
input nobody can audit.

### 3.8 The tests

`crates/spec-spine-core/tests/coverage.rs` MUST cover:

- the empty default: a fixture corpus's report is byte-identical with the key
  absent and with it set to `[]`;
- a file outside every package entering the universe through `governed_scope`;
- a file with no `SOURCE_EXTS` extension entering the same way;
- `governed_scope_exclusions` removing one of the above, and **failing** to
  remove a file that is in the universe for another reason;
- a `resolver_exclusions` path and a bypassed path staying out despite matching
  `governed_scope` (§3.3);
- a governed-scope file outside every package classifying as `Unowned` rather
  than `FloorOnly`, with the `C-002` message naming no floor (§3.4);
- `declaredScopeFiles` omitted when empty and populated otherwise (§3.5).

`crates/spec-spine-core/tests/couple.rs` MUST assert that with
`require_ownership` on, a changed governed-scope file with no claim is `C-002`,
and that the same file with a claim is not.

## 4. Out of scope

**Turning it on in this repository.** This spec ships the mechanism with an
empty default. Populating `[coverage] governed_scope` here is a separate change
that must first give the unclaimed files of §1.2 an owner, because the moment
the key is set they become unclaimed debt and
`index coverage --fail-on-untraced` refuses in CI. Seven of the nineteen are
already claimed and would arrive owned; the remaining twelve need a claim, and
most can take a `// Spec:` header in the same change, since they are shell and
Python. That change also edits `spec-spine.toml`, which restales all 101
shards. Doing both in one PR would mix a mechanism nobody can review against a
ledger regeneration nobody can read.

**Widening `SOURCE_EXTS`** (§1.3), and **widening the comment-header scanner**
(§3.4). Both are changes to what claims; this spec changes only what is asked.

**Overriding the bypass floor** (§3.3, D-2).

**A `governed_scope` for the registry or the index hashes.** `[index]
extra_hashed_inputs` already names what stales the ledger. The two lists serve
different questions ("does a change here invalidate the ledger" against "must a
spec own this") and a corpus may reasonably answer them differently for one
path. §1.2 measures that they currently disagree by accident, not by intent;
this spec gives a corpus the means to make them agree, and does not merge them.

**Reporting the disagreement between the two lists.** A verb that named every
`extra_hashed_inputs` match outside the coverage universe would be useful and is
a smaller spec than this one. It is not this one.

## 5. Resolved decisions

**D-1 (2026-09-15). A declared scope, not a longer extension list.** §1.3. The
extension list is a conjunct of four and shares its definition with the claim
scanner, so widening it fails to reach the governance files at the repository
root and changes the recognizer as a side effect. The wave-B rider that prompted
this spec was phrased as "should the ratchet reach tracked files outside
`SOURCE_EXTS`"; the measured answer is that the extension is not the binding
constraint, the package conjunct is.

**D-2 (2026-09-15). The bypass floor is not overridable by the scope.** §3.3. A
`governed_scope` entry that beat the floor would let one key silently un-bypass
`docs/` or `.github/` for the coupling gate, and the gate's bypass list is the
thing an adopter reads to know what is exempt. A corpus that wants a bypassed
path governed edits the bypass list, where the change is visible in
`config show` and in review. The cost is that this spec alone does not reach
`.github/workflows/`, which §3.3 states rather than hides.

**D-3 (2026-09-15). Git stays in the CLI.** §3.6. The alternative was to let
the core shell out for `git ls-files`, which is one line and breaks the
invariant that every artifact-producing function is a pure function of config
and file contents. The fallback for a non-git corpus is specified rather than
left to fail, and the report names which enumeration ran, because a silent
fallback would change a coverage denominator without changing a number anyone
could point at.

**D-4 (2026-09-15). The claim scanner is not widened with the universe.** §3.4.
A governed `.md` file cannot claim itself with a comment header, and this spec
accepts the asymmetry rather than resolving it, because resolving it means
deciding what a claim looks like in markdown, YAML and TOML, which is a design
question with its own spec.

## Verification

Each line is one command. The lines asserting the config keys, the report member
and the CLI enumeration fail against pre-097 code, because none of the three
exists; those are the fail-first evidence. The two `index coverage` lines assert
§3.1, that an unset key changes nothing, and they pass before and after by
design: they are the regression pin, not the evidence. The `cargo test` lines
are **not** fail-first; the cases in §3.8 do not exist at the parent commit, so
the suites pass vacuously.

```verify:cli
# 3.2: both keys exist in the config model.
grep -qF 'governed_scope' crates/spec-spine-types/src/config.rs
grep -qF 'governed_scope_exclusions' crates/spec-spine-types/src/config.rs
# 3.5: the report names what entered through the scope.
grep -qE 'declared_scope_files|declaredScopeFiles' crates/spec-spine-types/src/coverage.rs
# 3.6: the CLI enumerates, the core does not.
grep -qF 'ls-files' crates/spec-spine-cli/src/cmd_index.rs
! grep -rqF 'Command::new("git")' crates/spec-spine-core/src/
# 3.7: the effective config prints it.
target/release/spec-spine config show | grep -q 'governed_scope'
# 3.1: unset here, so this corpus is unchanged and reports no declared scope.
target/release/spec-spine index coverage --fail-on-untraced
target/release/spec-spine index coverage --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d.get("declaredScopeFiles", []) == [], d["declaredScopeFiles"]'
# 3.3, 3.4, 3.5, 3.8: the cases.
cargo test -p spec-spine-core --test coverage --locked
cargo test -p spec-spine-core --test couple --locked
```
