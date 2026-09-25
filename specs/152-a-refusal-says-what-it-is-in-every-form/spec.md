---
id: "152-a-refusal-says-what-it-is-in-every-form"
title: "A refusal says what it is in every form"
status: draft
kind: "tooling"
created: "2026-09-25"
summary: >
  Three consumer-visible inconsistencies survived 0.27.0, each documented in
  its handoff with a workaround. `check --json` and `index check --json` still
  report an unresolved claim as `fresh: false` with a `stale shard` line, where
  the human output and every other verb say validation, not staleness (spec
  145). `index coverage`, `index owner`, `scope evaluate` and `registry show`
  print nothing on stdout under `--json` when they refuse, so a consumer gets
  an empty document where the verdict verbs give an error envelope. And a
  compact plan path refused for a `:` is told to drop `.`, `..` and a leading
  slash, none of which it has. This spec makes the index half of the JSON
  freshness report answer drift alone and carry the claims separately, gives
  every `--json` verb an error envelope on failure, and makes the compact
  refusal name the rule the path broke.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "096-compaction-is-a-verb-not-a-session"
  - "132-one-exit-contract-for-the-family"
  - "144-a-repository-path-is-one-type"
  - "145-an-unresolved-claim-is-not-called-stale"
amends:
  - "145-an-unresolved-claim-is-not-called-stale"
extends:
  # 3.1 the freshness report's JSON halves
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: corrective }
  - { spec: "062-one-name-one-freshness-verb", unit: "crates/spec-spine-cli/src/cmd_check.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: corrective }
  # 3.2 the failure path of every --json verb
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: corrective }
  - { spec: "034-machine-readable-verdicts", unit: "crates/spec-spine-types/src/verdict.rs", nature: additive }
  # 3.3 the compact refusal
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: "crates/spec-spine-core/src/compact.rs", nature: corrective }
  # 3.4 the version and its history
  - { spec: "022-index-sharding", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "docs/schema-versioning.md", nature: additive }
---

# 152: A refusal says what it is in every form

## 1. Purpose

Measured on the 0.27.0 binary (`d78fb09a`), 2026-09-25, with a complete spec
claiming an absent file (`src/missing.rs`), compiled and indexed.

### 1.1 The JSON freshness report still calls a claim stale

`check` prints `codebase-index: UNRESOLVED CLAIM: 1 unresolved claim(s) over 1
spec(s), which is not staleness`, and `couple --json` reports
`error.kind: "validation"` with the claim as a violation. `check --json`
reports the same corpus as:

```json
"index": {
  "actual": "1 stale shard(s):\n  blocking-diagnostics by-spec/001-a.json",
  "expected": "1 shard(s) matching the corpus",
  "fresh": false,
  ...
}
```

`index check --json` reports the same object as its `report`. The report reads
the folded verdict (`IndexFreshnessReport::freshness`), which re-enters each
blocking claim as a `blocking-diagnostics <shard>` line; 145 split the verbs'
refusal and left the report folded.

### 1.2 Four reads print nothing when they refuse under `--json`

| Invocation | exit | stdout | stderr |
|---|---|---|---|
| `index coverage --json` | 1 | empty | `validation failed` and the claim |
| `index owner src/missing.rs --json` | 1 | empty | same |
| `scope evaluate --scope <file> --json` | 1 | empty | same |
| `registry show 999 --json` | 1 | empty | `not found: spec '999'` |

The verdict verbs (`check`, `couple`, `delta`, `lint`, ...) write an error
envelope on stdout on every failure (spec 034 §3.3, spec 132); `main.rs`
writes it only for the verbs `json_verb()` names, and the reads are not among
them.

### 1.3 The compact refusal names rules the path did not break

A plan retiring `C:rules/one.md` exits 2 with:

```
spec-spine: config error: compact: the plan retires `C:rules/one.md`; write it as the corpus spells it, with no `.` or `..` component and no leading slash
```

The path has none of those. It is refused by spec 144's path rule for its
`:`, which the message does not say.

## 2. Territory

- `crates/spec-spine-core/src/index.rs`: the freshness report's JSON form.
- `crates/spec-spine-cli/src/cmd_check.rs`, `cmd_index.rs`: its rendering.
- `crates/spec-spine-cli/src/main.rs`, `crates/spec-spine-types/src/verdict.rs`:
  the failure envelope for every `--json` verb.
- `crates/spec-spine-core/src/compact.rs`: the plan-path refusal.
- `crates/spec-spine-types/src/version.rs`, `docs/schema-versioning.md`: the
  verdict version (D-1).

## 3. Behavior

### 3.1 The index half answers drift; claims are their own member (amends 145)

In the `check --json` report's `index` object, and in the `index check --json`
report:

- `fresh` MUST be `true` exactly when no committed shard differs from the
  recompute, whatever the corpus's claims; `expected` and `actual` MUST name
  drifted shards only, and no `blocking-diagnostics` line;
- a new member `unresolvedClaims` MUST list each unresolved claim as a
  violation (`code`, `path`, `message`, `severity`), in the words and order
  the verbs' validation refusal uses (145 §3.1), and MUST be omitted when there
  is none, so a report on a corpus without unresolved claims is byte-identical
  to 0.27.0's.

`exitCode` and `outcome` are unchanged for every input: an unresolved claim is
still `1`, `finding`. A shard that drifted is still `fresh: false`, and when a
corpus has both, both are reported.

### 3.2 Every `--json` verb answers a failure with an envelope

When any verb invoked with `--json` fails, it MUST write the spec 132 error
envelope on stdout (`outcome`, `exitCode`, `error.kind`, and `error.violations`
for a validation failure), with `verb` naming the invocation in the existing
dotted form (`index.coverage`, `index.owner`, `scope.evaluate`,
`registry.show`, and likewise for every other read that takes `--json`), and
nothing on stderr. A successful read's output is unchanged: it is still the
bare read document, not an envelope (docs/schema-versioning.md, "A read is not
a verdict").

### 3.3 The compact refusal names the rule

A compact plan path refused by spec 144's rule MUST be refused with that rule's
own reason (`repo_path_problem`), for example:

```
spec-spine: config error: compact: the plan retires `C:rules/one.md`, which contains a ':', a drive, drive-relative or stream form on Windows (spec 144); write it as the corpus spells it: a relative path of plain segments
```

A `.` segment, which 144's rule accepts and compact refuses, keeps its own
reason. The exit code stays 2.

### 3.4 The verdict version moves (D-1)

The verdict schema version MUST move and `docs/schema-versioning.md` MUST
record what changed and why a consumer reading `fresh` alone was wrong
before and after.

## 4. Out of scope

- The human output of every verb: 145 fixed it, and 151 tests it at all seven.
- `registry list`, `registry plan`, `config show` and the other reads on
  success: unchanged.
- `index diagnostics` and `index render`, which list a claim and exit 0 by
  design (they report, they do not refuse).

## 5. Resolved decisions

**D-1 (open, owner): MINOR or MAJOR.** 3.1 changes the value of `fresh` for
one input (an unresolved claim alone, `false` to `true`) and adds a member;
3.2 adds output where there was none. The exit code, `outcome` and every other
input's bytes are unchanged, and the documented contract is that a consumer
decides on `exitCode`/`outcome` ("`--json` changes what is written and never
what is decided"). **Recommended: MINOR, verdict `1.1.0`,** with the handoff
naming the one reader it breaks: a consumer that gated on `fresh` alone and
ignored `exitCode`. MAJOR `2.0.0` would make every 1.x reader (Statecraft
included) refuse every envelope for a change that moves no verdict.

## Verification

```verify:cli
# Self-contained: every line below uses the release build.
cargo build --release --locked
# 3.1: an unresolved claim alone reads fresh, with the claim in its own member,
# in both `check --json` and `index check --json`; both still exit 1.
sh -c 'T="${TMPDIR:-/tmp}/ss152a"; B="$PWD/target/release/spec-spine"; rm -rf "$T" && mkdir -p "$T/specs/001-a" "$T/src" && printf -- "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/missing.rs\"\n---\n# a\n" > "$T/specs/001-a/spec.md" && "$B" --repo "$T" compile >/dev/null 2>&1 && "$B" --repo "$T" index >/dev/null 2>&1 || exit 1; "$B" --repo "$T" check --json > "$T.c" 2>/dev/null; c=$?; "$B" --repo "$T" index check --json > "$T.i" 2>/dev/null; i=$?; python3 -c "import json,sys; c=json.load(open(sys.argv[1])); i=json.load(open(sys.argv[2])); ok=lambda r: r[\"fresh\"] is True and [(v[\"code\"], v[\"path\"]) for v in r[\"unresolvedClaims\"]] == [(\"I-004\", \"src/missing.rs\")] and \"stale shard\" not in json.dumps(r); sys.exit(0 if c[\"exitCode\"] == 1 and i[\"exitCode\"] == 1 and ok(c[\"report\"][\"index\"]) and ok(i[\"report\"]) else 1)" "$T.c" "$T.i" 2>/dev/null; r=$?; rm -rf "$T" "$T.c" "$T.i"; test $c -eq 1 && test $i -eq 1 && test $r -eq 0'
# 3.1: a drifted shard is still stale, and a report without claims has no claims member (a control that holds on 0.27.0).
sh -c 'T="${TMPDIR:-/tmp}/ss152b"; B="$PWD/target/release/spec-spine"; rm -rf "$T" && mkdir -p "$T/specs/001-a" "$T/src" && printf "pub fn a() {}\n" > "$T/src/a.rs" && printf -- "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/a.rs\"\n---\n# a\n" > "$T/specs/001-a/spec.md" && "$B" --repo "$T" compile >/dev/null 2>&1 && "$B" --repo "$T" index >/dev/null 2>&1 && mkdir -p "$T/specs/002-b" && sed "s/001-a/002-b/" "$T/specs/001-a/spec.md" > "$T/specs/002-b/spec.md" || exit 1; "$B" --repo "$T" check --json > "$T.c" 2>/dev/null; c=$?; python3 -c "import json,sys; x=json.load(open(sys.argv[1]))[\"report\"][\"index\"]; sys.exit(0 if x[\"fresh\"] is False and \"unresolvedClaims\" not in x else 1)" "$T.c"; r=$?; rm -rf "$T" "$T.c"; test $c -eq 1 && test $r -eq 0'
# 3.2: the four reads of 1.2 each write an error envelope on stdout under --json.
sh -c 'T="${TMPDIR:-/tmp}/ss152c"; B="$PWD/target/release/spec-spine"; rm -rf "$T" "$T.s" && mkdir -p "$T/specs/001-a" "$T/src" && printf -- "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/missing.rs\"\n---\n# a\n" > "$T/specs/001-a/spec.md" && printf "{\"id\": \"W-1\", \"ownSpec\": \"001\", \"mutable\": [\"src/missing.rs\"]}\n" > "$T.s" && "$B" --repo "$T" compile >/dev/null 2>&1 && "$B" --repo "$T" index >/dev/null 2>&1 || exit 1; r=0; for a in "index.coverage/validation/index coverage --json" "index.owner/validation/index owner src/missing.rs --json" "scope.evaluate/validation/scope evaluate --scope $T.s --json" "registry.show/not-found/registry show 999 --json"; do verb=${a%%/*}; rest=${a#*/}; kind=${rest%%/*}; args=${rest#*/}; "$B" --repo "$T" $args > "$T.o" 2>/dev/null; rc=$?; python3 -c "import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if d[\"outcome\"] == \"finding\" and d[\"exitCode\"] == 1 and d[\"verb\"] == sys.argv[2] and d[\"error\"][\"kind\"] == sys.argv[3] else 1)" "$T.o" "$verb" "$kind" 2>/dev/null && test $rc -eq 1 || { echo "$verb: exit $rc, no $kind envelope on stdout"; r=1; }; done; rm -rf "$T" "$T.s" "$T.o"; exit $r'
# 3.3: the compact refusal names the ':' rule, and still exits 2.
sh -c 'T="${TMPDIR:-/tmp}/ss152d"; B="$PWD/target/release/spec-spine"; G="git -C $T -c user.name=v -c user.email=v@v -c commit.gpgsign=false"; rm -rf "$T" && mkdir -p "$T/specs/000-a" "$T/rules" "$T/docs" && printf "r\n" > "$T/rules/one.md" && printf "see \140rules/one.md\140\n" > "$T/docs/note.md" && printf -- "---\nid: \"000-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\n---\n# a\n\nBody.\n" > "$T/specs/000-a/spec.md" && printf "retire:\n  - path: \"C:rules/one.md\"\n    kind: file\n    forms:\n      citation: \"x\"\n" > "$T/plan.yaml" && $G init -q && $G add -A && $G commit -qm base || exit 1; "$B" --repo "$T" compact --plan-file "$T/plan.yaml" --plan > /dev/null 2> "$T.e"; rc=$?; grep -q "C:rules/one.md.*a drive, drive-relative or stream form on Windows" "$T.e"; g=$?; test $g -eq 0 || cat "$T.e"; rm -rf "$T" "$T.e"; test $rc -eq 2 && test $g -eq 0'
# 3.4: the conformance test still holds every envelope to the embedded schema.
cargo test -p spec-spine-core --locked --test conformance
```
