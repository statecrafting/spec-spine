---
id: "054-effective-config-is-a-governed-read"
title: "The effective configuration is a governed read"
status: approved
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "005-coupling-gate"
  - "009-coupling-floor-claim-precedence"
  - "037-machine-readable-verdicts"
extends:
  # A new `config show` verb. Spec 002 owns the read-only query surface and its
  # `--json` shape; it scopes itself to the compiled registry and says nothing
  # about reading configuration, so this is a sibling verb rather than a change
  # to what 002 requires. 002's spec.md is not edited (spec 040).
  - { spec: "002-registry-query", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  # The bypass floor and the waiver keyword are computed here today, privately.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  # Both crate roots list their public surface by name, and three new types and
  # one new function join those lists (2.1).
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
establishes:
  # Created by this spec (2), so claimed by it.
  - "crates/spec-spine-cli/src/cmd_config.rs"
  - "crates/spec-spine-cli/tests/config.rs"
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "docs/adoption-guide.md" }, role: context }
summary: >
  `spec-spine.toml` is only half the configuration a coupling decision is made
  from. The other half is `DEFAULT_BYPASS_PREFIXES`, a thirteen-entry constant
  compiled into the binary, additive to the adopter's list and impossible to
  read from outside the process. claude-observatory, which judges other
  repositories, hand-parses each target's TOML and documents in its spec 016
  D-3 that the built-in floor is simply unknowable to it. That is a consumer
  reimplementing half a contract and knowing it has the other half wrong. This
  spec adds `spec-spine config show [--json]`: the effective configuration,
  every default resolved, with the bypass floor rendered as the merged list the
  gate actually matches against and each entry attributed to the built-in floor
  or to the adopter's file. It reads and reports; it decides nothing.
---

# 054: The effective configuration is a governed read

## 1. Purpose

Every read in this system is meant to go through a typed consumer. The
constitution's principle II says machine truth is read only through the
`spec-spine` binary or the `spec-spine-core` library, and
`.claude/rules/governed-artifact-reads.md` says the same to every agent working
in an adopting repository. The reason is always the same: a hand-rolled parser
encodes today's assumptions and then goes quietly wrong when the contract moves.

Configuration is the one input that has no such consumer. `spec-spine.toml` is
authored TOML, and an adopter reading it back gets exactly what they wrote,
which is not what the tool uses. Two things are missing from the file:

**Defaults.** Every table is `#[serde(default)]`, so the absent keys are the
common case. `index.resolver_exclusions` is six directory names nobody wrote
down. `layout.cargo_workspace` is `Cargo.toml`. A file with three keys in it
configures thirty.

**The built-in bypass floor.** `couple.rs::DEFAULT_BYPASS_PREFIXES` is thirteen
entries compiled into the binary: `.github/`, `docs/`, `README.md`,
`CHANGELOG.md`, `LICENSE`, `CODEOWNERS`, `.gitignore`, `.gitattributes`,
`standards/spec/constitution.md`, `.derived/`, and three lockfile globs. The
adopter's `coupling.bypass_prefixes` is **additive** to it and cannot remove an
entry. So the list the gate matches against is a merge of a constant nobody
outside the process can see and a list the adopter wrote, and only the second
half is in the file.

claude-observatory is the case that makes this concrete. It is an orchestrator:
it judges repositories other than itself, so it must reason about a target's
coupling configuration without being that target. It hand-parses the target's
TOML, and its spec 016 D-3 records the conclusion honestly: the built-in floor
is unknowable to it. It is not a bug in claude-observatory. It is a fact about
what the tool exposes, written down by the only consumer that hit it.

The answer is the one the corpus already gives everywhere else: a verb. This is
item 5 of the adopter audit's ranked backlog for the tool.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-cli/src/main.rs` | 002 | the `config` subcommand |
| `crates/spec-spine-core/src/couple.rs` | 005 | the floor becomes readable as data |
| `crates/spec-spine-types/src/config.rs` | 000 | the effective-config DTO |

A new `crates/spec-spine-cli/src/cmd_config.rs` and its acceptance test are
created by the implementing change and claimed there, per
`.claude/rules/adversarial-prompt-refusal.md`: adding a file you created to the
spec you are implementing is one of the two always-legitimate mid-build edits.

Nothing about the gate's decision is in this spec's territory. `couple.rs`
changes only in that a value it already computes becomes reachable; the
predicate that consumes it is untouched.

### 2.1 Two crate roots

**Decision, 2026-09-07.** Both crate roots re-export their public surface by
name, so `BypassEntry`, `BypassSource`, `EffectiveConfig` and
`EffectiveCouplingConfig` join `spec-spine-types`'s list and
`effective_bypass_prefixes` joins `spec-spine-core`'s. Declared as `extends`
edges. Neither line changes behavior; they are the difference between a type
being nameable by a consumer and being reachable only through its module path.

## 3. Behavior

### 3.1 `spec-spine config show`

A new subcommand, `config`, with one action, `show`, taking `--json`.

It loads the repository's configuration exactly as every other verb does, fills
in every default, and prints it. It reads no committed artifact, so it works on
a repository that has never compiled, which is the state an adopter is in when
they most need to see what their config resolved to.

Exit `0` on success. A malformed `spec-spine.toml` is the existing
`Error::Config`, exit `3`, unchanged: this verb reports a configuration, and a
file that is not a configuration is the same failure it has always been.

`config` is a **read**. It MUST NOT write, MUST NOT create a missing
`spec-spine.toml`, and MUST NOT emit a "suggested" file. Scaffolding is
`init`'s job and has been since spec 006.

### 3.2 The bypass floor is reported merged and attributed

The report's coupling section MUST carry the bypass floor as the merged, ordered
list the gate matches against, with each entry attributed to its source:

```
coupling:
  waiver_keyword: "Spec-Drift-Waiver:"
  require_ownership: true
  auto_waive_dependency_only: true
  bypass_prefixes:
    .github/                          (built-in)
    docs/                             (built-in)
    ...
    **/README.md                      (spec-spine.toml)
```

Attribution is the load-bearing part and the reason this is not just a TOML
echo. A consumer needs to answer two different questions: "is this path
bypassed" (the merged list) and "did the adopter ask for that" (the source). An
unattributed merge answers only the first, and an orchestrator deciding whether
a target's configuration is unusual needs the second.

The order MUST be the order the gate evaluates: built-in floor first, then the
adopter's entries, each in its declared order. A consumer reimplementing the
match against this list gets the same answer as the gate, in the same order,
and that is the whole point of publishing it.

An entry appearing in both lists MUST be reported once, attributed to both. It
is legal and harmless (the match is an `or`), and reporting it twice would
suggest a duplicate the adopter should remove when there is nothing to fix.

### 3.3 The report is data first

`--json` MUST emit the effective configuration as a single JSON object, sorted
keys, pretty-printed, LF, trailing newline, exactly as every other emitted JSON
in this system.

It MUST NOT be the spec 037 verdict envelope. Spec 037 versions verdicts, and a
verdict is what a **gate** returns: `ok`, `exitCode`, and a report of what was
decided. `config show` decides nothing and cannot fail a gate, so wrapping it in
an envelope would put a permanently-`true` `ok` field on a verb that has no
notion of passing. It is a query, and it takes `--json` the way `registry show`
and `index coverage` take it: the object itself.

The prose form is a rendering of the same object and carries no fact the JSON
lacks.

**Decision, 2026-09-07: the report's keys are snake_case.** Every other emitted
JSON in this system is camelCase, and those are machine artifacts where that is
the house style. This document is a mirror of an authored TOML file, and a
consumer holding it beside `spec-spine.toml` should be reading the same token in
both: `bypass_prefixes` in the report is `bypass_prefixes` in the file. The
alternative shapes are both worse. All-camelCase renames every key the adopter
wrote, which is a translation table the verb exists to avoid; a camelCase
envelope over snake_case tables mixes the two in one document. §3.4 already
names the field `config_version` in prose, and this decision is the rest of that
choice made explicit rather than left to the serializer.

### 3.4 What it reports is what the tool would use

The reported object MUST be the `Config` value the verbs in this process would
consume, with defaults resolved and no field omitted for being defaulted.

Omitting defaulted fields would rebuild the exact problem this verb exists to
solve: a consumer would see a short document and have to know which absences
mean which values, which is reading `spec-spine.toml` again with extra steps.
The verb is worth having precisely because it is exhaustive.

It MUST include `config_version`, reporting `CONFIG_VERSION`, so a consumer
pinning against this shape has the shape's version in the document rather than
having to infer it from the binary's `--version`.

### 3.5 Determinism

The output is a pure function of `(config file contents, binary version)`. No
clock, no environment, no filesystem walk beyond reading the config file that
every verb already reads. The same repository and the same binary produce
byte-identical bytes.

## 4. Out of scope

**`couple --explain-bypass`.** The audit offered it as the alternative shape:
given a diff, say which paths the gate bypassed and why. It is a good verb and
it answers a different question, about a specific run rather than about the
configuration. Making the configuration readable first is the smaller change and
the one the blocked consumer actually needs; per-path explanation can be built
on it later without redoing anything here.

**Changing the floor, or making it removable.** The built-in list stays exactly
as it is, and adopter entries stay additive. Spec 009 settled the interaction
between the floor and an explicit claim, and this spec neither reopens it nor
depends on how it was settled. This verb makes the floor visible; it does not
make it negotiable.

**A `config check` or config lint.** Validating that an adopter's configuration
is sensible (a `bypass_prefixes` entry matching nothing, a `resolver_exclusions`
directory that does not exist) is a real idea and a different one. `show`
reports; it does not judge.

**Reading another repository's configuration without its binary.** `--repo`
already points every verb at another tree, so an orchestrator with the CLI can
read any target it can see. What it cannot do is read a target governed by a
*different version* of spec-spine and learn that version's floor. That is a
version-pinning problem, and spec 062 is where it belongs.

## 5. Verification

Every assertion fails against pre-054 code: `config` is an unknown subcommand
and clap refuses it with exit 2.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-cli --test config --locked
# 3.1: the verb exists and reports.
target/release/spec-spine config show
# 3.2: the built-in floor is in the report, which is the fact
# claude-observatory recorded as unknowable to it (its spec 016 D-3), and each
# entry says where it came from.
target/release/spec-spine config show --json | python3 -c 'import json,sys; e=json.load(sys.stdin)["coupling"]["bypass_prefixes"]; by={b["prefix"]:b["sources"] for b in e}; assert by[".derived/"]==["built-in"], by; assert by["**/README.md"]==["spec-spine.toml"], by'
# 3.2: merged in the order the gate evaluates, floor first.
target/release/spec-spine config show --json | python3 -c 'import json,sys; e=[b["prefix"] for b in json.load(sys.stdin)["coupling"]["bypass_prefixes"]]; assert e[0]==".github/", e; assert e[-1]=="**/README.md", e'
# 3.3: a query, not a spec 037 verdict envelope.
! target/release/spec-spine config show --json | grep -q '"exitCode"'
# 3.4: exhaustive, defaults resolved, and the shape carries its own version.
target/release/spec-spine config show --json | python3 -c 'import json,sys; c=json.load(sys.stdin); assert c["config_version"]=="0.1.0", c; assert c["index"]["resolver_exclusions"][0]=="target", c'
# 3.1: it reports on a corpus that never compiled, and writes nothing.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs" && target/release/spec-spine --repo "$tmp" config show > /dev/null && test ! -e "$tmp/spec-spine.toml"
# 3.5: a pure function of the config file and the binary.
test "$(target/release/spec-spine config show --json | shasum)" = "$(target/release/spec-spine config show --json | shasum)"
```
