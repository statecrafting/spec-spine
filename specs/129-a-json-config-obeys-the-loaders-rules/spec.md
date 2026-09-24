---
id: "129-a-json-config-obeys-the-loaders-rules"
title: "A configuration passed as JSON obeys the loader's rules"
status: approved
kind: "tooling"
created: "2026-09-24"
summary: >
  `load_config` refuses a `derived_dir` that leaves the repository (128), a
  `state_dir` that is the repository root or overlaps a governed root (036),
  and a malformed `[index.slices]` entry (011). The JSON facade deserialized
  its configuration with serde alone and ran none of those rules, at all 22
  entries that take one, so a binding could hand the engine a configuration
  no `spec-spine.toml` can hold. `scaffold_init_json`, the entry Statecraft
  links, returned `Ok` with a `spec-spine.toml` that every verb then refuses
  with exit 3, and it wrote each value between literal quotes, so a `"` broke
  the file and a `\` changed the value. The rules now live in one public
  function, `validate_config`, which the loader and every facade entry call,
  and the scaffold refuses what the loader refuses and escapes what it
  writes. Checks that need the filesystem or the running binary stay where
  they are.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "036-declared-state-dir"
  - "092-the-engine-ships-governance-not-an-environment"
  - "128-a-derived-tree-stays-in-its-repository"
# 3.3: 092 3.3 says a non-default layout value MUST produce a coherent
# scaffold. A value `load_config` refuses now produces a refusal instead.
amends: ["092-the-engine-ships-governance-not-an-environment"]
amends_sections: ["3.3"]
establishes:
  - { kind: file, path: "crates/spec-spine-core/tests/config_facade.rs" }
extends:
  # 3.1: one validator, public, run by the loader.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/config.rs", nature: corrective }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  # 3.1: every facade entry that takes a configuration runs it.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: corrective }
  # 3.3: the scaffold refuses and escapes.
  - { spec: "092-the-engine-ships-governance-not-an-environment", unit: "crates/spec-spine-core/src/scaffold.rs", nature: corrective }
  # 3.5: the facade documentation.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "docs/api.md", nature: additive }
references:
  - { unit: { kind: file, path: "specs/128-a-derived-tree-stays-in-its-repository/spec.md" }, role: context }
  - { unit: { kind: file, path: "specs/036-declared-state-dir/spec.md" }, role: context }
  - { unit: { kind: file, path: "docs/configuration.md" }, role: context }
intent:
  goal: "a configuration the engine accepts as JSON is one a `spec-spine.toml` could hold, refused with the same message where the loader would refuse it"
  non_goals:
    - "confining `specs_dir`, `standards_dir` or any layout key 128 left unchecked (4)"
    - "read-side link handling, concurrent path swaps, or any claim about Windows links and junctions (4)"
    - "checking `[meta] required_version` outside the CLI, which alone knows the version it runs as (3.4)"
    - "validating a `Config` a Rust caller builds and passes to a non-facade function other than `scaffold_init` (4)"
---

# 129: A configuration passed as JSON obeys the loader's rules

## 1. Purpose

### 1.1 Inventory of the facade, on `main` at `8446e773`

Every `pub fn *_json` in `crates/spec-spine-core/src/lib.rs`, by how its
configuration arrives:

| How the configuration arrives | Entries | Rules run before this spec |
|---|---|---|
| `config_json` argument, through `config_from_json` | `compile_json`, `closure_json`, `scope_json`, `index_json`, `lint_json`, `check_json`, `check_freshness_json`, `check_registry_freshness_json`, `coverage_json`, `verify_plan_json`, `render_json`, `scaffold_init_json`, `compact_json`, `attest_json`, `attest_spec_json`, `attest_snapshot_json` (16) | none |
| a `config` member of the request | `coverage_inventory_json`, `couple_json`, `delta_json` (optional), `verify_snapshot_attestation_json`, `verify_spec_attestation_json`, `verify_attestation_json` (6) | none |
| TOML text or a tree's own `spec-spine.toml` | `load_config_json`; `delta_json` without `config`; `couple_json`'s `priorRoots` | all of `load_config`'s |
| no configuration | `query_json`, `interface_verify_json`, `scope_compare_json`, `orphans_json` | not applicable |

For the 22 entries in the first two rows, a `derived_dir` of `../outside`, a
`state_dir` of `.`, and a slice named `Api` were each accepted: every entry
went on to its own work and failed, if at all, for a reason that had nothing
to do with the configuration (`tests/config_facade.rs`, run on the unchanged
source: five refusal tests fail, the two controls pass).

### 1.2 Measured through the entry Statecraft links

`scaffold_init_json` returned `Ok` for each configuration below. Its files
were written to an empty repository and `spec-spine compile` (0.24.0 source,
`8446e773`) was run there:

| Configuration passed | `scaffold_init_json` | `compile` on its output |
|---|---|---|
| `derived_dir = "../outside"` | `Ok` | exit 3: the `derived_dir` rule (128) |
| `state_dir = "."` | `Ok` | exit 3: the `state_dir` rule (036) |
| `metadata_namespace = "a\"b"` | `Ok` | exit 3: TOML parse error; the `"` ended the string |
| `specs_dir = "sp\\tx"` | `Ok` | exit 3: the file loads with `specs_dir` = `sp<TAB>x`, and no such directory exists |
| Statecraft's configuration (1.3) | `Ok` | exit 0 |

A producer returned, without complaint, a configuration file its own
consumer's every verb refuses. The last two rows are a second defect in the
same place: the template wrote each value between literal quotes, unescaped.

### 1.3 Statecraft's call shape

Statecraft CLI (`statecraft-home/src/producer.rs`, `config_json()`, measured at
statecraft-cli `fa000c6`, which pins `spec-spine-core =0.23.0`) passes every
layout key explicitly: `specs_dir = "specs"`, `derived_dir =
".statecraft/derived"`, `standards_dir = "standards/spec"`, `schemas_dir =
"standards/schemas"`, `cargo_workspace = "Cargo.toml"`, `npm_workspaces =
["package.json", "pnpm-workspace.yaml"]`, empty standalone lists, and
`state_dir = ".statecraft/state"`. Every rule accepts it. The scaffold this
spec's build returns for it, and for `"{}"`, is byte-identical to the one
`8446e773` returns.

## 2. Territory

- `crates/spec-spine-types/src/config.rs` (extends 000): `validate_config`, the
  one place the rules are listed; `load_config` calls it.
- `crates/spec-spine-types/src/lib.rs` (extends 000): exports it.
- `crates/spec-spine-core/src/lib.rs` (extends 001): every entry in 1.1's
  first two rows calls it.
- `crates/spec-spine-core/src/scaffold.rs` (extends 092): `scaffold_init`
  calls it, and the template escapes every value it writes.
- `crates/spec-spine-core/tests/config_facade.rs` (establishes): every entry,
  every rule, the controls, the Statecraft call shape and the escaping.
- `docs/api.md` (extends 057): the facade section says so.

## 3. Behavior

### 3.1 One validator, run by the loader and by every facade entry

`spec_spine_types::validate_config(&Config) -> Result<()>` MUST run exactly the
rules `load_config` runs after parsing: 011's `[index.slices]` grammar, 128's
`derived_dir` rule and 036's `state_dir` rule, in that order. `load_config`
MUST call it rather than list the rules itself.

Each of the 22 entries in 1.1's first two rows MUST call it on the
configuration it deserialized, before it reads a file, parses another input
that could fail for another reason, or does any other work, and MUST return
its error unchanged: `Error::Config`, exit 3, with the message `load_config`
gives for the same value. `delta_json` validates a configuration it was given;
without one it loads the base tree's `spec-spine.toml`, which already runs the
rules. A request that is not well-formed JSON is still refused as it was
(`Error::Parse` for a request, `Error::Config` for a `config_json` argument),
before the configuration inside it can be checked.

### 3.2 What stays accepted

Every configuration `load_config` accepts is accepted at every entry,
unchanged: `"{}"`, `.derived`, `.statecraft/derived`, `./.derived/`, an empty
`derived_dir` and `.` (128 D-3), a segment containing dots, a
`.statecraft/state` state root, a well-formed slice, and Statecraft's full
configuration (1.3). No signature, response shape, schema axis or exit-code
mapping changes.

### 3.3 The scaffold refuses what the loader refuses, and escapes what it writes (amends 092 3.3)

`scaffold_init`, and so `scaffold_init_json`, MUST refuse a configuration
`validate_config` refuses, with its error. 092 3.3's "a non-default value for
any of them MUST produce a coherent scaffold" now applies to the values the
loader accepts; for a value it refuses, the answer is that refusal, because
the only scaffold it could return is a `spec-spine.toml` every verb refuses.

Every value the starter `spec-spine.toml` carries MUST be written as a TOML
basic string escaped so that it loads back as given: `"`, `\`, and every
control character are escaped. A value the template quotes inside a comment
is escaped the same way, without the quotes, so a line break in it cannot end
the comment. The output for a value with none of those characters is
byte-identical to what it was; that is every value in 1.3.

`scaffold_init_json` stays a pure function of its argument (092 3.2): the
check reads the argument and nothing else.

### 3.4 Checks that are not the facade's

These stay where they are, and the facade does not gain them:

- **`[meta] required_version`** (055). It compares the configuration against
  the version of the binary judging it, which only the CLI knows; a library
  caller chose its version when it linked. The facade still does not check it.
- **The checked writer** (127, 128 3.3). Containment of each path written
  below the derived root needs the path and the root; it stays in `shard.rs`,
  unchanged, and still guards a Rust caller that builds a `Config` without
  either door. `validate_config` does not replace it.
- **Anything about the filesystem**: whether a directory exists, what a link
  resolves to, what a tree's own `spec-spine.toml` says. No rule in
  `validate_config` reads a path, so an entry that reads no file (`render_json`,
  `scaffold_init_json`) still reads none.
- **An absent `spec-spine.toml` means the defaults.** That is how the CLI and
  `tree_config` read a repository; the facade has no file, and `"{}"` already
  means the defaults.

### 3.5 Documentation

`docs/api.md`'s facade section MUST say that a `config_json` argument or a
`config` member is held to the loader's rules and refused as a configuration
error, and that `required_version` is not checked there.

## 4. Out of scope, and the limits of the claim

- **`specs_dir`, `standards_dir` and the other layout keys** are checked by
  no rule, at either door, so neither door refuses one. The scaffold still
  returns paths built from `specs_dir` and `standards_dir` as given, so a
  value like `../x` yields a `relPath` outside the repository; Statecraft's
  closed contract set (its `producer.rs`) is what declines to write it today.
  Confining those keys is 128 4's open question and stays open.
- **The bootstrap spec and templates** carry `metadata_namespace` in YAML and
  prose, unescaped. Only the `spec-spine.toml` template is escaped here.
- **Rust callers of the non-facade API** (`compile(&Config, ..)` and the rest)
  are not validated; `validate_config` is public for them, and 3.4's writer
  check still guards what they write.
- **Read-side links, concurrent path swaps, Windows links and junctions, and
  a device-named `derived_dir` segment**: as 128 4. Nothing here measures or
  claims them.
- **The CLI's behavior on a base whose `derived_dir` escapes** is unchanged.
  Measured for the recovery procedure `docs/configuration.md` now documents:
  on the pull request that corrects it, `delta` exits 3, and `couple` exits 3
  only when the diff deletes a path (it reads the base snapshot only then,
  spec 100 3.4); a correcting diff that deletes nothing gets `couple` exit 0.
  128 D-4 first said both refuse, true of `couple` only with a deletion; it
  was corrected to this measurement before its ratification.

## 5. Resolved decisions

**D-1 (2026-09-24): one function, not a second copy.** The loader and the
facade call the same function, and the tests compare the facade's message
with the loader's for the same value, so a rule added to one door is in the
other and a divergent wording fails a test.

**D-2 (2026-09-24): every entry, including the ones that read no file.**
`render_json` and `scaffold_init_json` read nothing, and the rules read
nothing, so applying them costs those entries no purity. Exempting them would
make "the facade accepts what a `spec-spine.toml` can hold" true of some
entries and not others, and the tests enumerate all 22 so an entry added
later without the rule fails there.

**D-3 (2026-09-24): amend 092 rather than read it as silent.** 092 3.3 says a
non-default layout value MUST produce a coherent scaffold, and it names no
exception. A refusal is not a scaffold, so this is a change to what 092 says,
declared as an amendment rather than recorded as a decision.

**D-4 (2026-09-24): escape rather than refuse a `"` or `\`.** Each is a legal
character in a value the loader accepts (a `\` in `specs_dir` included), so
refusing it would reject configurations the loader holds. Escaping makes the
scaffold's file load back as given, which is what 054's round-trip test
already assumed.

**D-5 (2026-09-24): compatibility.** A binding that passed a configuration the
loader refuses now gets `Error::Config` (exit 3) where it used to get an
answer. No known consumer does: Statecraft passes 1.3's configuration, which
every rule accepts, and its scaffold is byte-identical. The CLI is unchanged:
it has always loaded through `load_config`.

## Verification

```verify:cli
# 3.1 to 3.3: every entry refuses every rule's cases with the loader's
# message, the accepted configurations and Statecraft's call shape pass, and
# every scaffolded value reads back as given.
cargo test -p spec-spine-core --test config_facade --locked
# The loader's own cases, now through validate_config.
cargo test -p spec-spine-types --test config --locked
# 054's scaffold round-trip and 092's producer boundary still hold.
cargo test -p spec-spine-core --test scaffold --locked
```
