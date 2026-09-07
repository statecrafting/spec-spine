---
id: "062-a-version-pin-the-cli-can-check"
title: "A version pin the CLI can check"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "000-spec-spine-bootstrap"
  - "006-init-scaffold"
  - "054-effective-config-is-a-governed-read"
extends:
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/tests/config.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "docs/schema-versioning.md" }, role: context }
summary: >
  A repository is governed by whichever spec-spine binary happens to be on the
  path, and nothing anywhere says which one it should be. rahi states 0.11.0 in
  three files that must agree by hand. All four governed repositories are two to
  four releases behind the fixes their own experience motivated, and rahi cannot
  see spec 044, which was written for rahi's exact state, because it is pinned
  below it. This spec adds `[meta] required_version`, a semver requirement the
  CLI checks on every run, refusing with a message that names both the
  requirement and the running version. The schema versions already tell a loader
  whether it can read an artifact; this tells an operator whether they are
  running the tool the corpus was governed by, which is a different question and
  currently has no answer at all.
---

# 062: A version pin the CLI can check

## 1. Purpose

Everything in this system is versioned except the thing doing the work.

`REGISTRY_SCHEMA_VERSION`, `INDEX_SCHEMA_VERSION`, `VERDICT_SCHEMA_VERSION`,
`SPEC_ATTESTATION_SCHEMA_VERSION` and `CONFIG_VERSION` are compile-time
constants, asserted against embedded schemas so a DTO drift fails the build, and
`docs/schema-versioning.md` explains how a loader reacts to an unknown MAJOR.
That machinery answers "can this reader read this artifact". It does not answer
"is this the tool this repository is governed by", and nothing does.

The consequences the audit measured:

- **rahi states `0.11.0` in three files that must agree by hand**, none of which
  the tool reads. Nothing detects a fourth place that disagrees.
- **All four repositories run two to four releases behind** the fixes their own
  experience motivated. Not by decision: nothing told them.
- **rahi cannot see spec 044.** 044 was written for rahi's exact `approved` plus
  `in-progress` state, by an audit of rahi, and rahi is pinned below the release
  that ships it. The corpus that motivated a fix cannot get the fix, and nothing
  in either repository says so.
- **A pin is not just about features.** The built-in bypass floor is a constant
  compiled into the binary. Two versions can judge the same diff differently,
  and spec 054 makes the floor readable but cannot make an old binary's floor
  match a new one's.

The distinction worth stating plainly: a schema version protects a **reader**
from an artifact it cannot parse. A version pin protects an **operator** from
running a different tool than their repository was governed by. The second is
the one CI needs and the one this spec adds.

This is item 16 of the adopter audit's ranked backlog for the tool.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-types/src/config.rs` | 000 | the `[meta]` table and the check |
| `crates/spec-spine-types/tests/config.rs` | 000 | its acceptance |
| `crates/spec-spine-cli/src/main.rs` | 001 | enforcement, and the verbs exempt from it |
| `crates/spec-spine-core/src/scaffold.rs` | 006 | the scaffolded key |

## 3. Behavior

### 3.1 `[meta] required_version`

`Config` MUST gain a `[meta]` table with one optional key, `required_version`,
holding a semver **requirement** string (`"0.15.0"`, `">=0.15, <0.16"`,
`"^0.15"`). Absent means unpinned, which is every existing repository, and the
default.

A requirement rather than an exact version, because both intentions are
legitimate. An adopter reproducing a byte-identical ledger pins exactly. An
adopter who wants fixes but not a MAJOR writes a range. A single exact string
would force the first on everybody and produce a pin that is bumped without
being thought about.

The key is a **config** key rather than a `.spec-spine-version` sidecar. One
file already governs the repository, `spec-spine config show` (spec 054) reports
it with everything else, and a sidecar would be a second configuration surface
with its own discovery rules for one scalar.

`CONFIG_VERSION` does not move. An added optional table with a default is the
additive case that constant exists to make safe.

### 3.2 Every verb checks it

When `required_version` is present and the running binary's version does not
satisfy it, the CLI MUST refuse before doing any work:

```
spec-spine: this repository requires spec-spine >=0.15, <0.16
            (spec-spine.toml [meta] required_version); running 0.11.0.
            Install the required version, or change the pin deliberately.
```

Exit `3`. It is a configuration the tool cannot honour, which is what `3` means,
and it is emphatically not `1` (nothing was validated), not `2` (nothing is
stale), and not `0`.

The refusal MUST name three things: the requirement, the running version, and
that the pin lives in `spec-spine.toml`. An operator seeing only "version
mismatch" has to go find all three.

**Exempt verbs.** `--version` and `--help` MUST NOT be refused. They are how an
operator finds out what they are running, and refusing them would make the
diagnostic unavailable at exactly the moment it is needed. `init` MUST NOT be
refused either: it scaffolds a repository that does not have a configuration
yet, and in a repository that does, `init` is the verb an operator reaches for
when things are wrong.

Every other verb, including every read, is refused. A read from a mismatched
binary is the quiet failure this spec exists to prevent: `registry plan` from an
old binary answers a question about a corpus it may misunderstand, and answers
it confidently.

### 3.3 An old binary refuses too, for the wrong reason

`Config` carries `deny_unknown_fields` on every table, so a binary released
before this spec rejects `[meta]` as an unknown field and exits `3`:

```
config error: TOML parse error at line 1, column 2
unknown field `meta`, expected one of `manifest`, `domains`, ...
```

This is worth stating rather than discovering. The pin is enforced even by
binaries that predate it, which is the property that makes it usable at all: an
adopter adding the key today is protected from every older binary immediately,
not only from future ones. The message is poor, naming a parse error rather than
a version mismatch, and it cannot be improved: the binary producing it was built
before the key existed.

`docs/schema-versioning.md` MUST record this, because an operator who hits it
will otherwise read "unknown field `meta`" as a corrupt config rather than as an
out-of-date binary.

### 3.4 The scaffold writes it, commented out

`config_toml` MUST emit the key commented out, with the running version as the
example:

```toml
[meta]
# Pin the spec-spine version this repository is governed by. Uncomment to
# refuse a binary that does not satisfy it. A pin is not only about features:
# the coupling gate's built-in bypass floor is compiled into the binary, so two
# versions can judge the same diff differently.
# required_version = "0.15.0"
```

Commented out is the right default for the same reason the key is optional: a
scaffold that pinned a new repository to the version that happened to scaffold
it would create the exact stale-pin problem this spec is about, on day one, by
accident. Spec 061 §3.2 sets the rule this follows: a key that only makes sense
once the adopter has decided something is emitted commented out with an example.

### 3.5 It does not check the schema versions

`required_version` constrains the **binary**. It says nothing about
`REGISTRY_SCHEMA_VERSION` or `INDEX_SCHEMA_VERSION`, which the committed shards
carry and the loaders already enforce by rejecting an unknown MAJOR.

The two are deliberately separate axes and `docs/schema-versioning.md` already
says so about package version and schema version. A repository can pin a binary
without pinning a schema (the ordinary case) or find its shards rejected by a
binary that satisfies its pin (a MAJOR bump within a permissive range, which is
the pin being too loose, and the loader is right to refuse).

## 4. Out of scope

**Bumping any adopter's pin.** The audit's §5 records that as each repository's
own follow-up. This spec gives them a place to write it down.

**Installing or fetching the required version.** The refusal names what is
needed; it does not download anything. A tool that installed itself on a version
mismatch would be doing the largest possible thing in response to a config
mismatch.

**Warning on a satisfied-but-old pin.** "You are on 0.15.0 and 0.18.0 exists" is
a network question, and every verb in this system is a pure function of local
inputs. That property is worth more than the notification.

**Pinning per verb.** One binary, one pin.

## 5. Verification

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-types --test config --locked
# A satisfiable pin is silent. Until this ships, `[meta]` is an unknown table
# and every one of these exits 3.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs" \
  && printf '[meta]\nrequired_version = ">=0.1"\n' > "$tmp/spec-spine.toml" \
  && target/release/spec-spine --repo "$tmp" lint
# An unsatisfiable pin refuses, and the refusal names the requirement.
printf '[meta]\nrequired_version = "=0.0.1"\n' > "$tmp/spec-spine.toml"
target/release/spec-spine --repo "$tmp" lint 2>&1 | grep -q '0.0.1'
! target/release/spec-spine --repo "$tmp" lint 2>/dev/null
# --version stays available under an unsatisfiable pin: it is the diagnostic.
target/release/spec-spine --repo "$tmp" --version
# The scaffold writes the key, commented out.
tmp2=$(mktemp -d) && target/release/spec-spine --repo "$tmp2" init >/dev/null \
  && grep -q 'required_version' "$tmp2/spec-spine.toml"
```
