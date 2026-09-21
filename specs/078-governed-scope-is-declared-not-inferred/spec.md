---
id: "078-governed-scope-is-declared-not-inferred"
title: "Governed scope is declared, not inferred"
status: approved
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
  verdict changes on upgrade. The claim scanner is untouched: a scoped file is
  claimed from spec frontmatter, so those seven headers stay inert and this
  spec says so rather than implying otherwise.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "005-coupling-gate"
  - "008-coupling-floor-claim-precedence"
  - "029-ownership-coverage"
  - "047-effective-config-is-a-governed-read"
  - "050-claimed-but-unwitnessed"
  - "075-a-claim-below-the-header-window-is-not-silent"
extends:
  # 3.2: the config table. `config.rs` is 000's floor territory; 032 already
  # carries the `[coupling] require_ownership` key that this one sits beside.
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  # 3.3 to 3.5: the universe, the classifier's input, and the report.
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-core/src/coverage.rs", nature: additive }
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-types/src/coverage.rs", nature: additive }
  # 3.2, 3.6: the config table and the inventory types are re-exported.
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  # 3.3: the gate reads the same universe, so the inventory reaches the
  # `C-002` arm too; the bypass predicate itself is untouched (D-2).
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: additive }
  # 3.6: the facade takes the inventory the CLI would have passed, and must
  # answer what the CLI answers (`coverage_json`).
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # 3.6: the CLI enumerates tracked files and passes them in, the way 005
  # passes `DiffInput`; `index coverage` gains `--paths-from` for the
  # git-free route, which is a flag in the clap tree.
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  # 3.2, 3.7: effective config prints the declared scope like every other key,
  # and `init` writes the commented default beside it.
  - { spec: "047-effective-config-is-a-governed-read", unit: "crates/spec-spine-cli/src/cmd_config.rs", nature: additive }
  - { spec: "092-the-engine-ships-governance-not-an-environment", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "092-the-engine-ships-governance-not-an-environment", unit: "crates/spec-spine-core/tests/scaffold.rs", nature: additive }
  # 3.8: the tests, core and CLI. The CLI half is not optional: the
  # enumeration, its failure mode and the facade/CLI agreement are all
  # invisible to a core-only classification test.
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-core/tests/coverage.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/tests/couple.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
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
.githooks/enable-merge-driver.sh      # Spec: 094-one-gate-and-the-boundaries-it-holds
.githooks/merge-derived-index.sh      # Spec: 094-one-gate-and-the-boundaries-it-holds
kit/.githooks/enable-hooks.sh         # Spec: specs/090-.../spec.md
kit/.githooks/enable-merge-driver.sh  # Spec: 094-one-gate-and-the-boundaries-it-holds
kit/.githooks/merge-derived-index.sh  # Spec: 094-one-gate-and-the-boundaries-it-holds
py/scripts/smoke_test.sh              # Spec: specs/007-python-distribution/spec.md
```

Each header satisfies the recognizer spec 095 §3.2 declares, and each names a
spec that exists. `scan_comment_headers` never reads any of them, because it
walks discovered packages and these files are in none. An author followed the
documented mechanism exactly and got nothing, with no diagnostic saying so. The
only reason these headers have any effect at all is oblique: `kit/.githooks/`
is embedded verbatim into `kit_embedded.rs`, which **is** inside a package, so
the copies inside the generated Rust file are the ones the scanner reads (spec
095 §3.6 measures them there).

That is the finding this spec rests on. The question is not whether markdown
should be code. It is that the ratchet is looking somewhere else entirely: at
package membership, which nobody declared, while seven authors wrote a claim in
the format the documentation gives and got silence.

What this spec does with that finding is bounded, and §3.4 states the bound:
it lets a corpus declare these files governed, so `C-002` can ask who owns
them. It does **not** make the seven headers readable. Under this spec those
files are claimed in spec frontmatter or they are unclaimed debt; reaching the
headers where they sit is a separate spec (§4).

### 1.3 Why a longer extension list does not fix it

The obvious move, adding `md`, `yml`, `toml` to `SOURCE_EXTS`, fails on the
third conjunct and would misfire on the first. `AGENTS.md`, `CLAUDE.md`,
`.github/workflows/*.yml` and `spec-spine.toml` sit at the repository root or in
directories no package contains, so a longer extension list still cannot see
them. Meanwhile `SOURCE_EXTS` is shared with `scan_comment_headers` (spec 095
§3.2), so widening it also widens what the claim scanner reads, which is a
second change wearing the first one's clothes. Forty-four tracked governance
files fail the extension conjunct today; adding extensions reaches none of them
while changing the recognizer for all of them.

The rule the corpus needs is not "which extensions are code". It is "which
paths this repository governs", and that is a declaration, not an inference.

## 2. Territory

This spec adds a `[coverage]` table to `Config`, widens the coverage universe by
that table, adds two members to the coverage report (what entered through the
table, and which enumeration produced the file list), and adds the CLI-side
tracked-file enumeration the widened universe needs. The universe is read by
`coverage.rs` and by `couple.rs`'s `C-002` arm, and reached through both the
CLI and the `coverage_json` facade, so all four are territory: a change that
threads an inventory into the classifier and not into the gate would make the
report stop predicting the gate, which is the property spec 029 built. `init`'s
commented default and the CLI tests are here for the same reason (§3.2, §3.8).

It changes no committed artifact's schema: the scope is a config input and the
report is computed on read.

## 3. Behavior

### 3.1 Opt-in, and silent when unset

`[coverage] governed_scope` is empty by default. With it empty, the **answers**
MUST NOT move: the same coverage universe, the same per-file classifications,
the same coverage counts and prose, the same `C-002` set and the same exit code
from every gate verb. An adopter upgrading past this spec and changing no
configuration MUST see no verdict change anywhere.

"The answers" is narrower than "every byte", deliberately, and the difference is
declared here rather than discovered by a golden test. Two outputs do change
with the key unset: `config show` gains the two keys (§3.7), because an
effective config that omits a key nobody set is an effective config nobody can
audit; and `init` writes the commented default (§3.2) into a new
`spec-spine.toml`. A corpus pinning the bytes of either sees a diff, and both
are additive. Nothing that renders a verdict, counts a file or classifies
ownership changes at all, and `declaredScopeFiles` and the enumeration member
(§3.5) are absent from the report entirely while the scope is empty.

This is the same posture spec 029 took for `require_ownership` and spec 027 for
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
files. Spec 058 fixed the shipped default for that reason and spec 065 found the
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
- **Scope membership does not override a bypass, and does not disturb what
  already does.** `docs/`, `.github/`, `README.md` and the rest of
  `couple.rs::DEFAULT_BYPASS_PREFIXES` stay bypassed for a file that merely
  matches `governed_scope`: a file that is bypassed but counted as unclaimed
  debt is a coverage figure that can never reach 100% (the reasoning in
  `coverage.rs`'s own comment on `in_coverage_universe`). What already
  overrides a bypass is spec 008's explicit-claim precedence, which
  `is_bypassed_path` applies before consulting either list, and this spec
  leaves it exactly as it is: a resolved, ownership-bearing unit claim covering
  the path beats the built-in floor and the adopter's list alike, so a workflow
  file a spec explicitly claims is governed today and stays governed. An
  **unclaimed** file under a bypass prefix stays bypassed, before and after
  this spec. The one part of the bypass conjunct no claim reaches either is the
  declared state root (`[layout] state_dir`, spec 036), which is bypassed
  unconditionally; a scope entry certainly does not reach it.

The consequence is worth stating plainly: of §1.2's nineteen files, the ones
under `docs/` or `.github/` are not reachable **by a scope entry** alone. The
route that exists for them is the one that already exists for every bypassed
path: an explicit unit claim in spec frontmatter, which overrides the bypass
under spec 008. What no route reaches is an **unclaimed** file under a built-in
prefix, and making those require ownership is a change to the exemption policy
itself, which §5 D-2 records as a separate spec's work. The git hooks,
`install.sh`, `scripts/` and `py/` are reachable by a scope entry, because
nothing bypasses them.

### 3.4 How a governed-scope file is claimed

The three ways a file is claimed (spec 029 §3.1, CLAUDE.md's "how a file gets
claimed") are unchanged, and only the first of them reaches every
governed-scope file:

1. an `establishes` unit, or a unit carried on an `extends` edge: available for
   any path;
2. a package manifest floor: available only inside a package, so a
   governed-scope file outside every package can never be `FloorOnly`. It is
   `Specific` or it is `Unowned`, and the `C-002` message for it MUST NOT name a
   floor it does not have;
3. a `// Spec:` comment header: available only where the file's syntax has a
   line comment **and** its extension is in `SOURCE_EXTS` **and** the file lies
   inside a discovered package, because that is where the scanner walks (spec
   095 §3.2). This spec does **not** widen the scanner.

The consequence MUST be stated exactly, because §1.2's most striking measurement
is on the wrong side of it: **the seven inert headers stay inert after this
spec.** A governed-scope file outside every package is claimed by an
`establishes` unit or an `extends`-carried unit in spec frontmatter, or it is
not claimed, whatever comment it carries. A corpus adopting the scope writes
those frontmatter claims; it does not get them for free from headers already in
the files. The same holds for extension: a `.md` or `.yml` file in the governed
scope is claimed from frontmatter or not at all.

That asymmetry MUST be documented rather than designed around. Widening the
claim scanner is a change to what claims, which spec 095 §4 reserves for a spec
that measures it first, and which this spec deliberately does not carry: a
mechanism spec that also moved the claim boundary would ship two changes under
one review.

### 3.5 The report says which files entered this way

`CoverageReport` MUST gain two additive members. Their omission is keyed to the
**configured scope**, never to what the scope matched:

- when `governed_scope` is empty (absent or `[]`), both members MUST be
  omitted, which is what keeps §3.1 true for a corpus that never sets the key;
- when `governed_scope` is non-empty, both members MUST be present, and a
  scope that matched nothing emits `declaredScopeFiles: []` beside its
  `enumeration`. Omitting them on an empty match would drop the provenance in
  exactly the case §3.6 needs it: a supplied-empty inventory and a walk that
  found nothing are different answers, and only `enumeration` tells them apart.

The two members:

- `declaredScopeFiles`: the paths that are in the universe because of
  `governed_scope` and would not otherwise be, sorted.
- `enumeration`: which of §3.6's three enumerations produced the file list, as
  one of `tracked`, `supplied` or `walk`. A denominator that can come from
  three places MUST say which one it came from, or two reports that disagree by
  five files look like a corpus that changed.

The prose form of `index coverage` MUST report them on their own line, counted
and separable from the package totals, because their denominator is not a
package: a reader comparing "89/89 claimed" before and after must be able to
see which of the new files came from where.

The classifier MUST NOT change. A governed-scope file is `Specific` or
`Unowned` by exactly the rules in §3.4, and `--fail-on-untraced` refuses on
unclaimed files as it already does. This spec adds files to the denominator; it
adds no new refusal code and no new refusal condition.

### 3.6 The enumeration is the CLI's, not the core's, and it names itself

The universe is currently enumerated by walking package directories. A declared
scope can name a path in no package, so the walk must start from the repository
root, and a root walk meets `.git/`, untracked scratch files, build output and
whatever else a working tree holds. The filter that makes a root walk honest is
"is this file tracked", and that is a git question.

**The core MUST NOT run git.** This is the workspace invariant every
artifact-producing function already keeps. The CLI MUST enumerate the inventory
and pass it into the core as typed data, exactly as it parses `git diff` into
`DiffInput` for spec 005 and `git diff --name-only -z` into `changed_path_names`
for spec 071.

#### Three cases, not two

The core function MUST accept the inventory as an argument and MUST remain a
pure function of `(Config, file contents, inventory)`. The argument MUST
distinguish an **absent** inventory from a **supplied empty** one:

| Inventory | Files the scope may match | Reported `enumeration` |
|---|---|---|
| supplied, non-empty | exactly those paths | the provenance the caller declared |
| supplied, empty | none: the caller has said there is nothing to govern | the same |
| absent | a filesystem walk from the repository root | `walk` |

A supplied inventory carries its **provenance**, which the core copies into the
report and never infers: `tracked` when the CLI enumerated it from git, and
`supplied` when it came from an explicit path list. The core cannot tell the two
apart by looking at a list of strings, and a report that guessed would name the
wrong denominator in exactly the case a reader is trying to explain.

Modelling the argument as a plain list collapses the middle row into the
bottom one, and they are different answers: a caller that supplies an empty
list has answered the question, and silently walking the tree instead
substitutes a different denominator for the one it gave. An `Option`-shaped
argument (or any type that carries the distinction, provenance included) is
therefore required, and §3.8 pins the two apart.

#### The walk

The walk MUST exclude, in addition to `resolver_exclusions`: `.git/`, because it
is not corpus and its contents are machine-specific; and the declared state root
(`[layout] state_dir`), because spec 036 bypasses it unconditionally and a walk
that enumerated it would put paths in the denominator that the gate refuses to
look at. It follows no symlink out of the repository root.

#### Tracked, added, deleted

When the CLI enumerates from git it MUST use `git ls-files -z --cached --others
--exclude-standard` and MUST drop paths that do not exist in the working tree
(`--deleted` entries, and index entries whose file is gone):

- **added**: a new file that `.gitignore` does not exclude is in the inventory
  before it is staged. Otherwise `index coverage` reads clean locally and CI
  refuses the same tree, which is the lag this repository's own harness exists
  to remove.
- **ignored**: an **untracked** file that the ignore rules match is never in
  the inventory; `--exclude-standard` is what keeps build output and editor
  droppings out. A **tracked** file stays in the inventory whether or not an
  ignore rule matches it: `--cached` lists every index entry and
  `--exclude-standard` applies only to `--others`, and that is the right
  answer, because a file git tracks is corpus however `.gitignore` reads.
- **deleted or missing**: a path git still lists but the tree no longer holds is
  dropped. The scope governs files there are to read; a coverage row for a file
  that does not exist is a row nobody can act on.

#### A git failure is an error, not a fallback

If the CLI runs git and git fails (absent binary, not a repository, non-zero
exit), it MUST report that failure as an I/O error (exit 3) naming the remedy.
It MUST NOT fall back to the walk, because a silent fallback changes the
denominator without changing any message a reader could point at. The walk is
the **library** caller's case (an absent inventory), never a repair for a
command that failed.

The git-free route through the CLI is the explicit path list: `couple
--paths-from FILE` already is one, and this spec MUST NOT weaken it, so a list
supplied that way is the inventory and no git enumeration overrides it.
`index coverage` MUST gain the same `--paths-from FILE` flag, because with a
scope set the enumeration now decides its denominator too, and a corpus not
under git otherwise has no way to run the verb at all.

### 3.7 `config show` prints it

Both keys MUST appear in `config show`, like every other effective-config key
(spec 047). A declared scope that a corpus cannot read back is a governance
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
- both `declaredScopeFiles` and `enumeration` omitted when `governed_scope` is
  empty, and both present when it is set: populated for a scope that matches,
  and `declaredScopeFiles: []` with `enumeration` retained for a scope that
  matches nothing (§3.5);
- an **absent** inventory resolving by walk, and a **supplied empty** inventory
  matching nothing, asserted as two different reports (§3.6);
- the walk excluding `.git/` and the declared state root (§3.6).

`crates/spec-spine-core/tests/couple.rs` MUST assert that with
`require_ownership` on, a changed governed-scope file with no claim is `C-002`,
and that the same file with a claim is not; and that an unclaimed changed file
under a built-in bypass prefix matching `governed_scope` is **not** refused,
while the same file with an explicit unit claim is governed exactly as spec 008
already makes it (§3.3, D-2).

`crates/spec-spine-cli/tests/cli.rs` MUST cover what no core test can see:

- `index coverage --paths-from` resolving against the supplied list, with the
  reported `enumeration` naming it;
- a git enumeration failure exiting 3 rather than reporting a walk (§3.6);
- the git enumeration's four membership cases in a scratch repository (§3.6):
  a tracked file an ignore rule matches is **retained**, an untracked ignored
  file is **excluded**, an unstaged untracked addition is **included**, and a
  tracked file missing from the working tree is **dropped**;
- the `coverage_json` facade and the CLI answering the same report for the same
  inventory, the pairing spec 050 §3.3 requires, since the facade is the half a
  binding consumes;
- `config show` and a scaffolded `spec-spine.toml` carrying the keys (§3.1,
  §3.7).

## 4. Out of scope

**Turning it on in this repository.** This spec ships the mechanism with an
empty default. Populating `[coverage] governed_scope` here is a separate change
that must first give every file it names an owner, because the moment the key is
set they become unclaimed debt and `index coverage --fail-on-untraced` refuses
in CI. **All** of them need a frontmatter claim, the seven of §1.2 included:
their `// Spec:` headers are still inert on the far side of this spec (§3.4),
so nothing arrives owned by having a header already. That change also edits
`spec-spine.toml`, which restales all 101 shards. Doing both in one PR would mix
a mechanism nobody can review against a ledger regeneration nobody can read.

**Widening `SOURCE_EXTS`** (§1.3), and **widening the comment-header scanner**
(§3.4). Both are changes to what claims; this spec changes only what is asked.
Reaching the seven inert headers is a follow-up spec of its own, and is not
smuggled into this one.

**Making an unclaimed file under a built-in bypass prefix require ownership**
(§3.3, D-2). That is a change to the exemption policy: whether the floor in
`DEFAULT_BYPASS_PREFIXES` can be subtracted from at all, and by what. Spec 008's
explicit-claim precedence is untouched here and remains the route that exists.

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

**D-2 (2026-09-15). Scope membership does not override a bypass, and the
override that exists is spec 008's, untouched.** §3.3. A `governed_scope` entry
that beat the bypass verdict would let one key silently un-bypass `docs/` or
`.github/` for the coupling gate, and the bypass list is the thing an adopter
reads to know what is exempt. The first draft of this decision then offered an
escape hatch that does not exist: it said a corpus wanting `.github/workflows/`
governed edits `[coupling] bypass_prefixes`. It cannot.
`effective_bypass_prefixes` unions the adopter's list with
`DEFAULT_BYPASS_PREFIXES` and nothing subtracts from the floor, so an explicitly
empty adopter list still reports `.github/` as built-in. The route that does
exist is the one spec 008 built and `is_bypassed_path` consults first: an
explicit, resolved, ownership-bearing unit claim covering the path beats both
lists, so a workflow file a spec claims is governed today, with or without this
spec. What has no route is making an **unclaimed** file under a built-in prefix
require ownership, and that is a change to the exemption policy rather than
something a scope key may reach around. The cost is that this spec alone does
not reach unclaimed `.github/workflows/`, which §3.3 states rather than hides.

**D-3 (2026-09-15). Git stays in the CLI, and the enumeration is explicit in
all three directions.** §3.6. The alternative was to let the core shell out for
`git ls-files`, which is one line and breaks the invariant that every
artifact-producing function is a pure function of config and file contents.
Three consequences are decided rather than left to the build: an absent
inventory and a supplied empty one are different arguments with different
answers; a git failure is an error naming its remedy, never a quiet demotion to
the walk; and the report names which enumeration ran, because a coverage
denominator that can come from three places and says nothing is a number two
readers will read differently. `--paths-from` is the git-free route and this
spec keeps it, extending it to `index coverage`, which now has a denominator
that depends on the enumeration too.

**D-5 (2026-09-15). The unset-key promise is about answers, not bytes.** §3.1.
The first draft promised every verb would be "byte-for-byte" unchanged while
§3.7 added two keys to `config show` and §3.2 added a commented default to what
`init` writes, which is a contradiction a build would have had to resolve by
guessing. The promise that matters to an adopter is that nothing classified,
counted or refused moves; the two additive output changes are named where they
happen.

**D-4 (2026-09-15). The claim scanner is not widened with the universe.** §3.4.
A governed `.md` file cannot claim itself with a comment header, and this spec
accepts the asymmetry rather than resolving it, because resolving it means
deciding what a claim looks like in markdown, YAML and TOML, which is a design
question with its own spec.

**D-6 (2026-09-15). Ignore rules bind only untracked files, and omission is
keyed to the configured scope.** §3.5, §3.6. A pre-build review found two
places where the text contradicted itself. First, §3.6 prescribed `git ls-files
--cached --others --exclude-standard` and also said an ignored file is never in
the inventory, but `--exclude-standard` filters only `--others`, so a tracked
file an ignore rule matches is listed (reproduced in a scratch repository). The
command is right and the sentence was wrong: a tracked file is corpus, so it is
retained, and only untracked ignored files are excluded. Second, §3.8 said the
new report members are omitted "when empty" while §3.5 tied omission to an
empty scope. Keying omission to an empty match would drop `enumeration` exactly
when a supplied-empty inventory must be told apart from a walk that found
nothing, so both members are omitted only when `governed_scope` is empty, and a
set scope that matches nothing emits `declaredScopeFiles: []` with its
`enumeration`. §3.8 gained CLI cases for the four git membership cases.


**D-7 (2026-09-15). Five choices the text left open, settled at build.**

- **Scope membership is a filesystem glob, intersected with the inventory.**
  §3.2 requires `extra_hashed_inputs`' semantics, trap included, and those are
  the semantics of `shard::glob_files` walking the tree, not of a pattern
  matched against a string (a string matcher would let `dir/**` match files).
  So the patterns are globbed exactly as the hashed inputs are, the exclusions
  are subtracted, and the result is restricted to the inventory. A listed path
  that does not exist matches nothing.
- **The gate needs no enumeration.** `couple` resolves the scope from the same
  globs and does not ask git: every path it judges is in the diff, which is
  tracked by construction or is the caller's own `--paths-from` list, so no
  inventory could remove one. `cmd_couple.rs` is therefore unchanged; its
  `extends` edge stays for the territory it names.
- **The CLI enumerates only while the scope is set.** With `governed_scope`
  empty, `index coverage` runs no git and ignores `--paths-from`, so a corpus
  that never sets the key needs neither, which is §3.1's promise.
- **A file only the scope brings in counts toward the totals and toward no
  package**, including one that happens to lie inside a package, so each
  package's line reads as it did and the prose names those files on their own
  `declared scope` line (§3.5).
- **The facade is a new function, `coverage_inventory_json`**, taking
  `{ config?, repoRoot, inventory? }`. `coverage_json` keeps its signature and
  answers for an absent inventory, which is the library caller's walk (§3.6).

## Verification

Each line is one command. The lines asserting the config keys, the report member
and the CLI enumeration fail against pre-097 code, because none of the three
exists, and so does the `--paths-from` line, because `index coverage` has no
such flag today; those are the fail-first evidence. The two `index coverage`
lines assert §3.1, that an unset key changes nothing, and they pass before and
after by design: they are the regression pin, not the evidence. The `cargo test`
lines are **not** fail-first; the cases in §3.8 do not exist at the parent
commit, so the suites pass vacuously.

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
# 3.6: the git-free route exists at the verb whose denominator now depends on
# the enumeration.
target/release/spec-spine index coverage --help | grep -q 'paths-from'
# 3.1: unset here, so this corpus is unchanged and reports neither new member.
target/release/spec-spine index coverage --fail-on-untraced
target/release/spec-spine index coverage --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert "declaredScopeFiles" not in d, d["declaredScopeFiles"]; assert "enumeration" not in d, d["enumeration"]'
# 3.3, 3.4, 3.5, 3.8: the cases.
cargo test -p spec-spine-core --test coverage --locked
cargo test -p spec-spine-core --test couple --locked
# 3.6, 3.8: the enumeration, its failure mode, and the facade/CLI agreement.
cargo test -p spec-spine-cli --test cli --locked
```
