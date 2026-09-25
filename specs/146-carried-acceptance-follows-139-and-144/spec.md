---
id: "146-carried-acceptance-follows-139-and-144"
title: "Carried acceptance follows 139 and 144"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  The pre-release sweep of `e440b714` failed two approved specs' acceptance,
  both invalidated by specs merged this cycle rather than by a defect. Spec
  046's block lints this corpus from a scratch root whose `specs` is a link to
  this repository's `specs/`, a link that resolves outside the scratch root, which
  spec 144 now refuses (exit 2). Spec 089's block asserts that the built-in
  legacy ledger reports spec 011 `exempt`, and spec 137 retired 011's line when it
  gave 011 an executable acceptance. This spec carries both blocks under
  `amends_verification`, unchanged except that 046's scratch root copies the
  corpus instead of linking it, and 089 asks the ledger about 019, which stays
  exempt by design (139 3.3).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "046-depends-on-ordinal-monotonicity"
  - "089-nothing-reruns-a-merged-acceptance"
  - "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
  - "144-a-repository-path-is-one-type"
amends_verification:
  - "046-depends-on-ordinal-monotonicity"
  - "089-nothing-reruns-a-merged-acceptance"
amends:
  - "046-depends-on-ordinal-monotonicity"
  - "089-nothing-reruns-a-merged-acceptance"
---

# 146: Carried acceptance follows 139 and 144

## 1. Purpose

Measured by `scripts/verify-sweep.sh --rev e440b714` (the 0.27.0 bump on
`main`), before any 0.27.0 release was cut: `passed=140 failed=2`.

| Spec | Failing command | Cause |
|---|---|---|
| 046 | the scratch-root lint of this corpus (command 5) | the scratch root's `specs` is a link to this repository's `specs/`, outside the scratch root; spec 144 §3.4 refuses a read through such a link (exit 2) |
| 089 | the built-in-ledger check (command 47) | it asserts `[('011-index-hash-slices', 'exempt')]`; spec 137 gave 011 carried acceptance and deleted its ledger line, so the sweep reports it `passed` |

Neither rule changed: 046's convention still holds on this corpus, and 089's
built-in ledger still answers and is still closed, live and shrink-only. Each
block tested its rule through a detail a later spec legitimately moved.

## 2. Territory

This spec establishes nothing and edits no amended spec's file beyond the
superseded-acceptance note spec 082 §3.4 requires.

## 3. Behavior

### 3.1 046's acceptance copies the corpus

`spec-spine verify 046` MUST run this spec's block. 046's commands are carried
unchanged except the scratch-root line, which copies `specs/` (`cp -R`) instead
of linking it, so spec 144's link rule does not refuse the read.

### 3.2 089's acceptance asks about a spec that stays exempt

`spec-spine verify 089` MUST run this spec's block. 089's commands are carried
unchanged except the built-in-ledger pair, which runs `--only 019` and expects
`[('019-release-supply-chain-artifacts', 'exempt')]`. 019 is one of the four
entries spec 139 §3.3 keeps exempt by design, because its acceptance is the
release run itself.

### 3.3 The amended specs say so

046 and 089 each carry spec 082 §3.4's superseded-acceptance note naming this
spec above their own block.

## 4. Out of scope

**Why the sweep did not catch this before the merges.** 137's pre-merge check
ran 089's command that sweeps `--only 011` and saw exit 0; it did not run the
next command, which read that sweep's report. 144's review did not run 046's
block. Both were caught by the pre-release sweep, which is the check spec 089
exists to provide.

## 5. Resolved decisions

**D-1 (2026-09-25): carry, do not edit.** Both specs are approved, so their
blocks are replaced through `amends_verification` (spec 082), as 084 and 132
did, not edited in place.

**D-2 (2026-09-25): 019, not 000.** Any of the four remaining entries would
do. 019 is a single-spec sweep whose outcome is fixed by the ledger alone; 000
is the constitution, and pinning its exemption into another spec's acceptance
adds a reader of it for no gain.

## Verification

```verify:cli
# ---- carried for 046-depends-on-ordinal-monotonicity (amends_verification), the scratch root copies the corpus (3.1) ----
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test lint --locked
# 3.1: the knob exists and parses.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs" && printf '[lint]\nrequire_ordinal_monotonic_depends_on = true\n' > "$tmp/spec-spine.toml" && target/release/spec-spine --repo "$tmp" lint
# 3.1: and a config written before this spec still parses, reading the default.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs" && printf '[layout]\nspecs_dir = "specs"\n' > "$tmp/spec-spine.toml" && target/release/spec-spine --repo "$tmp" lint
# 4: this corpus is asked, not assumed. A scratch root copies `specs/` so the
# repository's own `spec-spine.toml` is neither read nor written; the knob is on
# only inside the scratch root. Zero diagnostics means the convention holds.
tmp=$(mktemp -d) && cp -R "$PWD/specs" "$tmp/specs" && printf '[lint]\nrequire_ordinal_monotonic_depends_on = true\n' > "$tmp/spec-spine.toml" && target/release/spec-spine --repo "$tmp" lint | grep -q '^lint: 0 error'
# 3.4: a lint is not a validation. The committed shards are byte-identical and
# the corpus this repository ships still lints clean with the knob off.
target/release/spec-spine compile --check
target/release/spec-spine lint --fail-on-warn
# ---- carried for 089-nothing-reruns-a-merged-acceptance (amends_verification), the built-in-ledger pair asks about 019 (3.2) ----
cargo build --release --locked
# --- the fixture corpus: repo inside, scratch outside (3.6) ---
rm -rf "${TMPDIR:-/tmp}/ss112" && mkdir -p "${TMPDIR:-/tmp}/ss112/repo/specs"
printf '%s\n' '[layout]' 'specs_dir = "specs"' 'derived_dir = ".derived"' > "${TMPDIR:-/tmp}/ss112/repo/spec-spine.toml"
for s in 000-legacy 001-green 002-red 003-silent 004-slow 005-dirty 006-after-dirty 007-ghost; do mkdir -p "${TMPDIR:-/tmp}/ss112/repo/specs/$s" && printf '%s\n' '---' "id: \"$s\"" "title: \"Sweep fixture $s\"" 'status: draft' 'implementation: pending' 'created: "2026-09-17"' 'summary: >' '  A sweep fixture.' '---' '' "# $s" > "${TMPDIR:-/tmp}/ss112/repo/specs/$s/spec.md"; done
printf '%s\n' '' '## Verification' '' '```verify:cli' 'true' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/001-green/spec.md"
printf '%s\n' '' '## Verification' '' '```verify:cli' 'true' 'false' 'true' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/002-red/spec.md"
printf '%s\n' '' '## Verification' '' '```verify:cli' 'sleep 120' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/004-slow/spec.md"
printf '%s\n' '' '## Verification' '' '```verify:cli' 'touch residue.txt' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/005-dirty/spec.md"
printf '%s\n' '' '## Verification' '' '```verify:cli' 'test ! -e residue.txt' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/006-after-dirty/spec.md"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss112/repo" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss112/repo" index >/dev/null
git -C "${TMPDIR:-/tmp}/ss112/repo" init -q -b main . && git -C "${TMPDIR:-/tmp}/ss112/repo" add -A && git -C "${TMPDIR:-/tmp}/ss112/repo" -c user.email=a@b -c user.name=t commit -qm fixture
# 3.3: a spec the committed registry still lists whose document is gone. Its
# acceptance cannot be read, which is not the same fact as having none.
git -C "${TMPDIR:-/tmp}/ss112/repo" rm -rq specs/007-ghost && git -C "${TMPDIR:-/tmp}/ss112/repo" -c user.email=a@b -c user.name=t commit -qm ghost
git -C "${TMPDIR:-/tmp}/ss112/repo" checkout -q -b side && printf '\n<!-- unmerged -->\n' >> "${TMPDIR:-/tmp}/ss112/repo/specs/003-silent/spec.md" && git -C "${TMPDIR:-/tmp}/ss112/repo" add -A && git -C "${TMPDIR:-/tmp}/ss112/repo" -c user.email=a@b -c user.name=t commit -qm side && git -C "${TMPDIR:-/tmp}/ss112/repo" checkout -q main
printf '000-legacy\n' > "${TMPDIR:-/tmp}/ss112/exempt.txt"
# --- 3.3, 3.5, 3.6: one run, five outcomes, and it never stops ---
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/run" --timeout 5 >/dev/null 2>&1; test $? -eq 1
# 3.3: every spec is accounted for, and the four non-passing kinds are distinct
# from success. `006-after-dirty` passing is the witness that the tree was
# restored after `005-dirty` wrote to it (3.6, D-7).
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));o={s['id']:s['outcome'] for s in d['specs']};assert o=={'000-legacy':'exempt','001-green':'passed','002-red':'failed','003-silent':'not-declared','004-slow':'not-run','005-dirty':'passed','006-after-dirty':'passed','007-ghost':'not-run'}, o"
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));assert d['counts']=={'passed':3,'failed':1,'not-declared':1,'exempt':1,'not-run':2}, d['counts']"
# 3.3: the two `not-run` reasons are distinct and both are recorded. A spec
# whose plan cannot be read MUST NOT be reported as declaring no acceptance.
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));s={x['id']:x for x in d['specs']};assert '--plan' in s['007-ghost']['failure'] and s['007-ghost']['exitCode']!=0;assert 'limit' in s['004-slow']['failure']"
# 3.8: the evidence a finding is reproduced from.
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));assert len(d['revision'])==40;assert d['trustedRef']=='main';assert d['binaryVersion'].startswith('spec-spine ');assert d['selection']=='all';assert d['ledgerClosedAt']==43"
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));s={x['id']:x for x in d['specs']};assert 'FAILED at command 2' in s['002-red']['failure'];assert s['002-red']['exitCode']==1;assert s['002-red']['log']=='logs/002-red.log';assert s['002-red']['commands']==3"
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));s={x['id']:x for x in d['specs']};assert s['004-slow']['exitCode']==124 and 'limit' in s['004-slow']['failure'];assert s['005-dirty']['leftTreeDirty'] is True;assert s['001-green']['leftTreeDirty'] is False"
test -s "${TMPDIR:-/tmp}/ss112/run/logs/002-red.log"
# 3.8: every cited log exists, and a spec that never ran cites none.
python3 -c "import json,os;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));s={x['id']:x for x in d['specs']};assert s['007-ghost']['log'] is None;assert all(os.path.isfile('${TMPDIR:-/tmp}/ss112/run/'+x['log']) for x in d['specs'] if x['log'])"
grep -qF 'logs/002-red.log' "${TMPDIR:-/tmp}/ss112/run/sweep.md"
! grep -qF 'logs/007-ghost.log' "${TMPDIR:-/tmp}/ss112/run/sweep.md"
grep -q 'not-declared' "${TMPDIR:-/tmp}/ss112/run/sweep.md"
# 3.6: the worktree is removed and the swept repository is left clean.
test -f "${TMPDIR:-/tmp}/ss112/run/sweep.json" && test ! -e "${TMPDIR:-/tmp}/ss112/run/tree"
test -f "${TMPDIR:-/tmp}/ss112/run/sweep.json" && test -z "$(git -C "${TMPDIR:-/tmp}/ss112/repo" status --porcelain)"
# --- 3.5: a narrowed selection, by short ordinal, is recorded ---
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/one" --only 001 >/dev/null 2>&1; test $? -eq 0
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/one/sweep.json'));assert [s['id'] for s in d['specs']]==['001-green'];assert d['selection']=='001'"
# 3.5: an unpadded ordinal is refused the way the tool refuses it, and the
# refusal names the form it wants rather than only reporting a miss.
# 3.5: a spec named twice is selected once.
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/dup" --only 001,001,001 >/dev/null 2>&1; test $? -eq 0
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/dup/sweep.json'));assert [s['id'] for s in d['specs']]==['001-green'], d['specs'];assert d['counts']['passed']==1"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/x" --only 1 >/dev/null 2>&1; test $? -eq 3
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/x" --only 1 2>&1 | grep -q '3-digit ordinal'
target/release/spec-spine verify 49 >/dev/null 2>&1; test $? -eq 1
# --- 3.4: the ledger refuses, before anything is executed, in all four ways ---
# Closed: nothing at or above the closing ordinal can be exempted, so no future
# spec is covered by it.
printf '050-too-late\n' > "${TMPDIR:-/tmp}/ss112/e-closed.txt"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-closed.txt" --out "${TMPDIR:-/tmp}/ss112/x" >/dev/null 2>&1; test $? -eq 3
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-closed.txt" --out "${TMPDIR:-/tmp}/ss112/x" 2>&1 | grep -q 'closed ordinal 43'
# Only shrinks: an entry whose spec declares acceptance is a stale exemption.
printf '001-green\n' > "${TMPDIR:-/tmp}/ss112/e-stale.txt"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-stale.txt" --out "${TMPDIR:-/tmp}/ss112/x" >/dev/null 2>&1; test $? -eq 3
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-stale.txt" --out "${TMPDIR:-/tmp}/ss112/x" 2>&1 | grep -q 'exemption is stale'
# Live: an entry naming no spec in the corpus accounts for nothing.
printf '030-gone\n' > "${TMPDIR:-/tmp}/ss112/e-dangling.txt"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-dangling.txt" --out "${TMPDIR:-/tmp}/ss112/x" 2>&1 | grep -q 'names no spec in the corpus'
# Enumerated and closed: the ledger this repository actually ships carries no id
# at or above the closing ordinal, so no future spec can enter it (D-2). The
# count is deliberately not asserted: the ledger shrinks as the debt is retired,
# and a pinned size would refuse the retirement 3.4 exists to allow.
test -f scripts/verify-sweep.sh && ! grep -qE '^0(4[3-9]|[5-9][0-9])-|^[1-9][0-9][0-9]?-' scripts/verify-sweep.sh
grep -qE '^0[0-4][0-9]-' scripts/verify-sweep.sh
# And it holds against the real corpus: running the BUILT-IN ledger over a
# merged revision of this repository passes all four checks of 3.4 (closed,
# live, only-shrinking, enumerated) and reports a legacy spec as `exempt`
# rather than `not-declared`. A single `--only` keeps this to a second. This
# line needs `origin/main` present in the checkout, which a maintainer's clone
# has and a remote-less mirror does not; 3.1 makes that the only context this
# block runs in.
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --rev origin/main --trusted-ref origin/main --only 019 --out "${TMPDIR:-/tmp}/ss112/builtin" >/dev/null 2>&1; test $? -eq 0
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/builtin/sweep.json'));assert d['ledgerOrigin']=='built into verify-sweep.sh';assert [(s['id'],s['outcome']) for s in d['specs']]==[('019-release-supply-chain-artifacts','exempt')]"
# --- 3.7: the trust boundary is mechanical, and its override is visible ---
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --rev side --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/x" >/dev/null 2>&1; test $? -eq 3
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --rev side --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/x" 2>&1 | grep -q 'is not an ancestor of main'
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --rev side --trusted-ref side --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/side" --timeout 5 >/dev/null 2>&1; test $? -eq 1
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/side/sweep.json'));assert d['trustedRef']=='side'"
# --- 3.6: the report can never be written inside the repository under test ---
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/repo/inside" >/dev/null 2>&1; test $? -eq 3
test -d "${TMPDIR:-/tmp}/ss112/repo/specs" && test ! -e "${TMPDIR:-/tmp}/ss112/repo/inside"
# ... and it refuses before creating anything, including when the named parents
# do not exist yet (the case the `--out <repo>/inside` line above cannot reach,
# since its parent is the repository root and already exists).
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/repo/newsub/run" >/dev/null 2>&1; test $? -eq 3
test ! -e "${TMPDIR:-/tmp}/ss112/repo/newsub"
# --- 3.6: the run directory is cleared before use, so it must be one the
# sweep is entitled to delete. A slipped --out at a directory holding anything
# else is refused and its contents survive.
mkdir -p "${TMPDIR:-/tmp}/ss112/precious" && touch "${TMPDIR:-/tmp}/ss112/precious/keepme"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/precious" >/dev/null 2>&1; test $? -eq 3
test -f "${TMPDIR:-/tmp}/ss112/precious/keepme"
# --- 3.1, 3.2: the sweep is a caller; it changes nothing about the tool ---
# It is not a subcommand, it is in no skill's gate floor, and it is on no
# pull-request leg. Spec 099 3.6 corrected the last assertion in place: since
# that spec the sweep runs from `.github/workflows/acceptance.yml`, on the
# default branch after a merge and on a schedule, and never on a pull request.
# The `test -f` guard is load-bearing: a bare `! grep` on a file that is not
# there passes, and would assert nothing if the workflow were deleted.
! target/release/spec-spine --help 2>&1 | grep -qE '^[[:space:]]+sweep'
! grep -rqF 'verify-sweep' .claude/skills/
! grep -qF 'verify-sweep' .github/workflows/ci.yml
test -f .github/workflows/acceptance.yml && ! grep -qE '^[[:space:]]*(pull_request|pull_request_target|merge_group):' .github/workflows/acceptance.yml
# It reads the corpus through the governed verbs only.
grep -qF 'registry list --ids-only' scripts/verify-sweep.sh
grep -qF 'verify "$1" --plan' scripts/verify-sweep.sh
test -f scripts/verify-sweep.sh && ! grep -qE '(jq|awk|sed)[^|]*\.derived' scripts/verify-sweep.sh
# --- 3.9: the runbook names the sweep and both of its triggers ---
grep -qF 'scripts/verify-sweep.sh' docs/releasing.md
grep -qF 'amends_verification' docs/releasing.md
rm -rf "${TMPDIR:-/tmp}/ss112"
```
