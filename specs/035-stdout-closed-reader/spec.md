---
id: "035-stdout-closed-reader"
title: "A closed reader is not an error: stdout writes stop panicking the CLI"
status: approved
kind: "tooling"
created: "2026-09-03"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "001-compile-registry"
  - "002-registry-query"
establishes:
  - "crates/spec-spine-cli/src/out.rs"
extends:
  # A crate-wide output-mechanism change, so every edge names the crate's floor
  # owner (001) rather than each command's semantic owner: no command's meaning
  # changes, only the call it makes to write a line.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_attest.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_compile.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_init.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_lint.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_registry.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/verify_attestation.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
summary: >
  Exit codes are a stable contract: 0 ok, 1 validation failure, 2 stale, 3 I/O.
  The CLI could exit 101. `println!` unwraps its write, so a reader that stops
  early made the process panic: `spec-spine registry list --json | head` printed
  a Rust backtrace and exited 101, and under `set -o pipefail` that failed the
  surrounding script. Piping into `head`, `less` or `grep -q` is ordinary use,
  not an error condition. This spec routes every CLI stdout write through one
  helper that classifies the write instead of unwrapping it: a closed reader
  ends the process 0, because the consumer got what it asked for, and a genuine
  I/O failure reports and exits 3, which is the code the contract already
  assigns to I/O. Stderr keeps `eprintln!`: diagnostics are small, and a broken
  stderr has nowhere left to report itself. The fix is not `unsafe` SIGPIPE
  restoration, which `forbid(unsafe_code)` rules out workspace-wide, and not a
  panic hook, which would match on a message string the standard library is
  free to change.
---

# 035: A closed reader is not an error

## 1. Purpose

`CLAUDE.md` and `docs/design/00-architecture.md` both state the exit codes as a
stable contract: `0` ok, `1` validation failure / not found / drift, `2` stale,
`3` I/O / parse / schema / config. `main.rs` maps them in exactly one place, via
`Error::exit_code()`.

The CLI could nonetheless exit `101`, the Rust panic code, through a path that
never reached that mapping:

```
$ spec-spine registry list --json | head -c 50
thread 'main' panicked at library/std/src/io/stdio.rs:1165:9:
failed printing to stdout: Broken pipe (os error 32)
$ echo ${PIPESTATUS[0]}
101
```

`println!` unwraps its write. When the reader stops early the write fails with
`BrokenPipe`, and the macro panics. Under `set -o pipefail`, a shell pipeline
that reads part of the output fails.

Reading part of a stream is not an error. `head`, `less`, `grep -q` and a
terminal pager all do it, and every well-behaved Unix producer treats it as a
normal end. The failure was also self-inflicted in a specific way: this is the
same argument spec 033's `V-014` review turned on, where a panic was rejected
precisely because 101 sits outside the documented contract. The CLI was doing on
its most ordinary path what the library had just refused to do on an unreachable
one.

## 2. Territory

`out.rs`, new: the two write entry points and their shared classification.
`main.rs`: the `outln!` and `out!` macros and the module declaration. The eight
command modules and `verify_attestation.rs`: their stdout call sites, including
the two `print!` block writes in `cmd_index.rs` that emit the rendered
projections. `tests/cli.rs`: the end-to-end acceptance test.

## 3. Behavior

### 3.1 Two entry points, one classifier, three outcomes

The CLI writes stdout two ways, and both must be covered. Most sites emit one
formatted line (`println!`). The rendered projections, `index render` and
`index coverage`, build their whole output as a single string that already ends
in a newline and emit it with `print!`, which panics identically. A line-only
helper would have left those two sites unguarded while the rule read as though
they were covered.

So there are two entry points, `line` (appends a newline) and `block` (writes
verbatim), sharing one classifier. No CLI stdout write uses `println!` or
`print!` directly. Both entry points classify the write rather than unwrapping
it:

| outcome | when | process |
|---|---|---|
| written | the write succeeded | continue |
| reader gone | `ErrorKind::BrokenPipe` | exit `0` |
| failed | any other I/O error | report on stderr, exit `3` |

`0` for a closed reader is the shell convention: `producer | head` succeeds.
`3` for a real failure is the code the contract already assigns to I/O, so the
mapping in `main.rs` stays the single authority on what each code means.

### 3.2 Why not the two shorter fixes

**Restoring the default `SIGPIPE` disposition** is the usual remedy and is
unavailable here: it requires an `unsafe` libc call, and `unsafe` is
`forbid`-en workspace-wide in `Cargo.toml [workspace.lints]`. Pulling in a
third-party crate to perform the same `unsafe` behind a safe wrapper would honor
the letter of that lint while evading its intent, and would add a dependency to
the determinism surface for one signal.

**A panic hook, or `catch_unwind` in `main`,** would have to recognize the
broken-pipe panic by its message text. That string belongs to the standard
library and can change without notice, so the guard would fail silently on a
toolchain bump. It also leaves `println!` panicking and merely intercepts the
result.

Routing the writes is more code, and it is the version that cannot rot: the
`BrokenPipe` error kind is a stable API.

### 3.3 Stderr is deliberately unchanged

`eprintln!` stays at every diagnostic site. Stderr output is small and rarely
piped, and a process whose stderr has gone has no remaining channel to report
that fact, so the classification would have nothing useful to do.

### 3.4 Determinism and output are unaffected

Bytes written are identical: the helper writes the same formatted line to the
same handle. No artifact, hash or emitted JSON changes, so the determinism gate
and every golden test are untouched. Only the behavior when a write *fails*
changes.

### 3.5 Tests (minimum)

1. Classification is pure and tested directly: `Ok` is written, `BrokenPipe` is
   reader-gone, other kinds are failures.
2. End to end: the binary emitting more than a pipe buffer, whose reader closes
   after 32 bytes, exits `0`, does not exit `101`, and prints no panic to
   stderr.
3. The block path (`index render`, `index coverage`) is covered by
   construction rather than by a pipe-breaking test: it shares the classifier
   with the line path, and its output on any corpus small enough to build in a
   test fits inside a pipe buffer, so such a test could not fail. What is
   asserted instead is the property that makes the guarantee total: no
   `print!` or `println!` remains at a CLI stdout site.

## 4. Out of scope

- **Stderr classification**, per §3.3.
- **Buffered or streaming output.** The helper writes line by line, as the
  macros it replaces did. Making large queries stream is a separate performance
  question this spec does not open.
- **The exit-code contract itself.** This spec brings a path that escaped the
  contract back under it; it adds no code and redefines none.
