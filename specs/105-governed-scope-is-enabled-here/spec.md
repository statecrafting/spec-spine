---
id: "105-governed-scope-is-enabled-here"
title: "Governed scope is enabled here"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "078-governed-scope-is-declared-not-inferred"
  - "100-a-deleted-path-is-judged-where-it-lived"
summary: >
  Spec 078 added `[coverage] governed_scope` so a repository can bring its
  governance files under the ownership ratchet, and the repository that wrote
  it never turned it on. The measured cost is four unclaimed paths. They are
  claimed, the scope is declared over exactly the files this repository already
  folds into its content hash, and what stays exempt is stated rather than
  implied.
establishes:
  - "CLAUDE.md"
  - "scripts/bump_version.py"
  - "standards/spec/templates/constitution-template.md"
extends:
  - spec: "011-index-hash-slices"
    unit: { kind: file, path: "crates/spec-spine-types/schemas/build-meta.schema.json" }
    nature: additive
  - spec: "061-shipped-is-not-the-same-as-working"
    unit: { kind: file, path: "spec-spine.toml" }
    nature: additive
references:
  - unit: { kind: file, path: "crates/spec-spine-core/src/coverage.rs" }
    role: "context"
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
---

# 105: Governed scope is enabled here

## 1. Purpose

Spec 078 exists because the ownership ratchet's universe is inferred: a source
extension inside a discovered package. That leaves out the governance files at
the root, the scripts and hooks in no package, and anything whose extension is
not code. `[coverage] governed_scope` declares them in, and this repository,
which wrote the spec, has it set to `[]`.

The consequence is narrow and real: `AGENTS.md`, the `Makefile`, the hooks, the
scripts and the schemas are folded into the content hash, so editing one
restales every shard, and none of them is subject to `C-002`. A governance file
can be added here today with no spec claiming it and nothing will say so.

Note 09 D-2 rules that the gaps are closed first and the scope enabled after
spec 100, in its own spec. This is that spec.

### 1.1 The measured cost, re-measured

Note 09 §3.2 measured three unclaimed paths against `4ab1b31e`. Re-measured
2026-09-21 against this branch with `index owner`, one path at a time, over
every file the `[index] extra_hashed_inputs` globs actually match:

| Path | Unit claims |
|---|---|
| `CLAUDE.md` | 0 |
| `scripts/bump_version.py` | 0 |
| `standards/spec/templates/constitution-template.md` | 0 |
| `crates/spec-spine-types/schemas/build-meta.schema.json` | 0 |

**Four, not three.** Note 09's table recorded the
`crates/spec-spine-types/schemas/*.json` group as fully claimed; five of the
six are, each by the spec that introduced the shape it describes, and
`build-meta.schema.json` is not. D-2's instruction to re-measure rather than
assume the list still exhaustive is the only reason this is not being
discovered by the gate after the switch was thrown.

Two paths that a naive reading adds to that list and that do **not** belong on
it are the unclaimed `.github/workflows/ai-pr-review.yml` and
`determinism.yml`. See §3.4.

## 2. Territory

`[coverage] governed_scope` in `spec-spine.toml`, and the four previously
unclaimed governance files.

It claims no engine code. `coverage.rs` is carried as a `references` unit,
which is non-owning: spec 078 built the mechanism and this spec only
configures it.

Three of the four gaps are claimed by this spec directly, which is coherent
rather than opportunistic: this spec's subject *is* that every governance file
this repository hashes also has an owner, so owning the residue is its
territory. The fourth, `build-meta.schema.json`, is claimed by an `extends`
edge onto spec 011, which already owns three of its siblings, because a schema
belongs with the specs that describe its shape and not with the spec that
noticed it was unowned.

## 3. Behavior

### 3.1 The scope is declared over what is already hashed

`[coverage] governed_scope` MUST be the set of governance files this
repository already folds into its content hash, and nothing wider. The two
lists answer the same question, "which files outside a package govern this
repository", and letting them diverge would mean a file whose edit restales
every shard but which the ratchet has never heard of, or the reverse.

The declared scope is therefore the `[index] extra_hashed_inputs` globs, minus
the entries §3.4 shows are unreachable.

Glob semantics are `extra_hashed_inputs`'s, trap included: `dir/**` matches
directories and therefore no files, and `dir/**/*` is what matches files
(spec 069). A glob matching nothing is `L-010` and `lint --fail-on-warn` is in
the gate, so a dead entry here is a refusal rather than a silent no-op.

### 3.2 The four gaps are closed before the switch is thrown

Enabling the scope with a gap open would put the gate in a red state on
purpose, which note 09 D-2 explicitly rejects as option (b). Each of §1.1's
four paths MUST carry an ownership-bearing claim in the same change that
declares the scope.

`scripts/bump_version.py` could be claimed by a `# Spec:` comment header
(spec 075's window; `.py` claims with `#`), which needs no frontmatter edit
anywhere. It is claimed by frontmatter here instead, deliberately: a comment
header claims exactly the file it sits in and says nothing to a reader of the
corpus about *why* the file is governed, and the reason is this spec.

### 3.3 Bypass semantics are not weakened

Note 09 D-2 attaches this as a condition of its own ruling, and it is restated
as a requirement rather than a footnote.

This spec MUST NOT remove or narrow any entry of the built-in bypass floor,
MUST NOT add an entry to `[coupling] bypass_prefixes`, and MUST NOT change the
precedence between the floor, the configured additions, the spec 008 explicit
claim override, the declared state root or the configured derived root. Those
are spec 005 §3.5, spec 008, spec 036 and spec 092 §3.8 and they are untouched.

The scope **adds paths to the coverage universe**. It does not lift a path out
of an exemption, and the evaluation order is unchanged: bypass first, ratchet
second.

If closing a gap appears to require loosening a bypass, that is the signal to
stop and re-scope rather than to loosen it.

### 3.4 What stays exempt, stated plainly

"Governed scope enabled" reads as "everything is covered". It is not, and the
gap between the two is where a later reader gets a wrong answer. With the scope
on, all of the following remain outside `C-002`:

| Still exempt | Why |
|---|---|
| everything under the built-in bypass floor, including `.github/`, `docs/`, `README.md`, `**/README.md`, the lockfiles and the configured `derived_dir` | the floor is evaluated **before** the ratchet, so a floor path never reaches `C-002` however the scope is written |
| everything under `[coupling] bypass_prefixes` | same evaluation order; additive to the floor |
| everything under `[layout] state_dir` | declared ungoverned (spec 036), excluded from classification by construction |
| anything matched by `[index] resolver_exclusions` | pruned before the universe is built |
| anything listed in `[coverage] governed_scope_exclusions` | carved back out after the scope is applied |
| a **deleted** path | `C-002` exempts deletions (spec 029), scope or no scope |
| every path in neither a discovered package nor the declared scope | the scope is an allowlist, not a wildcard |

Two concrete consequences worth naming, because both look like gaps and are
not:

- **The two unclaimed workflow files stay unclaimed and stay green.**
  `.github/` is on the floor, so `ai-pr-review.yml` and `determinism.yml` never
  reach `C-002` and adding `.github/workflows/*.yml` to the scope would be a
  dead entry. They are not a debt this spec closes and this spec MUST NOT
  pretend otherwise.
- **`docs/*.md` entries are likewise unreachable**, for the same reason. The
  hashed docs are governed by being hashed and by their owning specs, not by
  the ratchet.

### 3.5 `.claude/` is included, and leaves with the harness

The `.claude/` governance globs already in the hash are in the scope. Every one
of them is already claimed (specs 061, 062, 064, 092, 093), so including them
costs nothing now and closes the hole where a new skill or agent could be added
here with no spec.

`.claude/settings.local.json` is **not** matched: the glob is the exact path
`.claude/settings.json`. That file is the user's own, claimed by no spec, and
note 09 §4.4 keeps it out of the harness retirement for the same reason.

When `.claude/`'s harness content is retired (note 09 §4.2's conditions, none
of which is satisfied today), the three `.claude/` entries leave
`extra_hashed_inputs` and this scope together. That is one more line for §4.3's
surgery list and is recorded here so it is not discovered then.

### 3.6 Nothing about the mechanism changes

This spec MUST NOT change `coverage.rs`, the `C-002` message, the
`--fail-on-untraced` flag, `index coverage --paths-from`, or spec 078's
evaluation order. It is a configuration change plus four claims.

## 4. Out of scope

- **Any change to spec 078's mechanism.** §3.6.
- **Weakening a bypass.** §3.3.
- **Claiming the two unclaimed workflows.** §3.4: they are exempt by the floor
  and a claim would be theatre.
- **Widening the scope past what is hashed.** §3.1. A repository-wide scope is
  a different decision with a different cost, and nothing measured asks for it.
- **The `.claude/` retirement.** §3.5 records the line it adds; it performs
  nothing.

## 5. Resolved decisions

**D-1 (2026-09-21, build: the re-measure changed the answer).** See §1.1. Note
09 §3.2's three-path list was taken against `4ab1b31e` and was one short:
`crates/spec-spine-types/schemas/build-meta.schema.json` is the only one of the
six embedded schemas with no unit claim, and the note's table recorded that
group as clean. D-2's instruction to re-measure rather than assume is what
caught it, and it is the reason the switch did not go on with a `C-002` waiting
behind it.

**D-2 (2026-09-21, build: measured effect, 97 to 136).** With the scope on,
the coverage universe grows from 97 files to 136, of which 39 come from the
declared scope, and all 136 are specifically claimed. The four gaps were closed
in the same change, so the ratchet is green from the moment it is armed.

**D-3 (2026-09-21, build: the growth assertion compares against a fixed base,
not a constant).** A hardcoded `> 99` would go stale on the next branch that
adds a source file, and worse, a scope whose globs all matched nothing would
still satisfy `--fail-on-untraced`. The acceptance runs `index coverage` twice
with the same binary on the same tree, once with an empty `--paths-from`
inventory, which spec 078 §3.6 defines as matching nothing, and asserts the
with-scope universe is strictly larger. That is a comparison the scope has to
earn.

**D-4 (2026-09-21, build: `bump_version.py` is claimed by frontmatter, and the
comment header was tried and reverted).** A `# Spec:` header is the cheaper
claim and was written first. It has to sit inside the first sixteen lines,
which put it above the shebang and broke the script as an executable. Moving it
below the shebang would have worked, but the frontmatter claim was the right
one anyway for the reason §3.2 gives, so the header came out rather than being
relocated.

**D-5 (2026-09-21, build: every `python3 -c` in the acceptance is one line).**
`verify:cli` commands run under `sh -c`, where a newline inside a double-quoted
argument ends the command and the remainder is parsed as shell. A multi-line
assertion failed at exit 2 with a shell syntax error, which reads like a
governance failure and is not one.

**D-6 (2026-09-22, carried onto `main` and re-measured).** The build was
written on a pre-integration branch (`a48a1691`) that never merged. It is
carried onto `main` at `c6380932` as the spec and the one `spec-spine.toml`
block, unchanged, and re-measured there rather than trusted. With the scope on
and this spec's claims removed, `index coverage` reports three unclaimed paths
(`CLAUDE.md`, `scripts/bump_version.py`,
`standards/spec/templates/constitution-template.md`) and one floor-only path,
`crates/spec-spine-types/schemas/build-meta.schema.json`, which is covered only
by its package manifest. §1.1's "0 unit claims" for that file still holds; the
coverage report files it under the floor rather than as unclaimed. All four
claims are still needed for every file in the grown universe to be
specifically claimed. The universe grows from 100 to 140 files, 40 of them from
the declared scope (D-2 measured 97 to 136, 39, before specs 117 to 121 added
files). `[index] extra_hashed_inputs` has not changed since the build, so §3.1's
list is still exactly that set minus the floor paths. Re-measured again after the expansion
wave merged (`main` at `6e123d2e`): 108 to 148, still 40 from the scope, all
148 specifically claimed.

**D-7 (2026-09-22, the scope adds a refusal, measured).** A larger universe
is not the goal; a governance file that can no longer be added unowned is. On
this tree, with a new `scripts/zz-probe.sh` present, regenerated and listed in
a `--paths-from` inventory of every tracked file,
`index coverage --fail-on-untraced` exits 1 naming the probe with the scope
declared, and exits 0 on the same tree with the scope removed. The probe is
not in the acceptance because it has to write into the committed tree and
regenerate the ledger; it is recorded here and in the pull request.

## Verification

Behavioral where it can be. The load-bearing assertion is that the ratchet,
with the scope on, reports full coverage over a universe that has actually
grown: a scope that matched nothing would also pass `--fail-on-untraced`, so
the case count is compared against the pre-scope figure.

Written to fail against the tree this spec is filed on: `governed_scope` is
empty and three of the four paths have no owner.

```verify:cli
# Each `python3 -c` below is a SINGLE line on purpose: `verify:cli` commands
# run under `sh -c`, where an embedded newline inside a double-quoted argument
# ends the command and the rest is parsed as shell.
#
# 3.1: the scope is declared and non-empty.
./target/release/spec-spine config show --json > /tmp/spec105-config.json
python3 -c "import json; c=json.load(open('/tmp/spec105-config.json')); assert c['coverage']['governed_scope'], 'governed_scope is empty'"
# 3.2: each measured gap now has an ownership-bearing claim. `index owner`
# prints '(no spec owns this path)' for an unowned path, so a 'unit' line is
# the positive half of the assertion.
./target/release/spec-spine index owner CLAUDE.md | grep -q 'unit'
./target/release/spec-spine index owner scripts/bump_version.py | grep -q 'unit'
./target/release/spec-spine index owner standards/spec/templates/constitution-template.md | grep -q 'unit'
./target/release/spec-spine index owner crates/spec-spine-types/schemas/build-meta.schema.json | grep -q 'unit'
# 3.1: the universe actually grew, compared against a FIXED BASE rather than a
# hardcoded count. `--paths-from` with an empty inventory is spec 078's
# git-free route and makes the declared scope match nothing, so the same binary
# on the same tree yields the without-scope universe. A scope whose globs
# matched nothing would pass `--fail-on-untraced` while leaving these two
# numbers equal, which is exactly the vacuous pass this compares against.
: > /tmp/spec105-empty-inventory.txt
./target/release/spec-spine index coverage --json > /tmp/spec105-with-scope.json
./target/release/spec-spine index coverage --paths-from /tmp/spec105-empty-inventory.txt --json > /tmp/spec105-no-scope.json
python3 -c "import json; w=json.load(open('/tmp/spec105-with-scope.json'))['sourceFiles']; n=json.load(open('/tmp/spec105-no-scope.json'))['sourceFiles']; assert w > n, f'the declared scope added nothing: {n} -> {w}'; print(f'universe: {n} -> {w}')"
# 3.2 and 3.6: the ratchet is green over the grown universe, which is the whole
# point of closing the gaps before throwing the switch.
./target/release/spec-spine index coverage --fail-on-untraced
# 3.3: the bypass floor is unchanged. Every built-in entry is still present and
# still attributed to the built-in source, and the adopter list still holds
# exactly the one entry it held.
python3 -c "import json; c=json.load(open('/tmp/spec105-config.json')); e={x['prefix']: x['sources'] for x in c['coupling']['bypass_prefixes']}; missing=[p for p in ['.github/','docs/','README.md','.gitignore','**/Cargo.lock'] if p not in e or 'built-in' not in e[p]]; assert not missing, f'left the floor: {missing}'; adopter=sorted(p for p,s in e.items() if 'built-in' not in s); assert adopter == ['**/README.md'], f'the adopter bypass list changed: {adopter}'"
# 3.4: a floor path stays exempt with the scope on, asserted rather than
# argued: the unclaimed workflows are not reported as coverage debt.
./target/release/spec-spine index coverage --json > /tmp/spec105-cov.json
python3 -c "import json; c=json.load(open('/tmp/spec105-cov.json')); u=[str(x) for x in c['unclaimedFiles']]; bad=[x for x in u if '.github/' in x]; assert not bad, f'a floor path reached the ratchet: {bad}'; assert not u, f'unclaimed: {u}'"
# The governed loop over the corpus this spec is part of.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
