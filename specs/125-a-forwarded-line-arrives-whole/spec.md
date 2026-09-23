---
id: "125-a-forwarded-line-arrives-whole"
title: "A forwarded line arrives whole"
status: draft
kind: "tooling"
created: "2026-09-23"
summary: >
  `verify` forwards a command's stdout and stderr to its own stderr from two
  threads, one write per read. A read ends wherever the pipe's contents did,
  which is mid-line whenever the command writes faster than the drain, so a
  line of one stream could be spliced by a chunk of the other. Spec 090's own
  flood test counts intact lines and caught it intermittently: the post-merge
  acceptance sweep at `0bf9ff78` failed twice, in two different specs' runs of
  the same suite, with 8191 of 8192 lines intact. Each drain now delivers whole
  lines only, holding a partial line until its newline arrives or a bound is
  reached. Order between the streams is still not promised.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "090-the-verdict-is-the-only-thing-on-stdout"
extends:
  # 3.1 to 3.3: the drain, and its tests beside it.
  - { spec: "090-the-verdict-is-the-only-thing-on-stdout", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_verify.rs" }, nature: additive }
references:
  - { unit: { kind: file, path: "crates/spec-spine-cli/tests/verify_streams.rs" }, role: context }
---

# 125: A forwarded line arrives whole

## 1. Purpose

### 1.1 Measured, on the default branch

The `Acceptance` push leg on `main` at `0bf9ff78` (#318) swept 64 specs. It
failed, and its one re-run failed again, each time in a different spec:

| Attempt | Failing spec | At | Test |
|---|---|---|---|
| 1 | `043-verify-declared-acceptance` | command 4, `cargo test -p spec-spine-cli` | `json_forwards_more_than_a_pipe_buffer_without_deadlocking`: `left: 8191, right: 8192`, "every forwarded line must arrive" |
| 2 | `044-index-diagnostics-reach-a-gate` | command 3, `cargo test -p spec-spine-cli` | the same test, the same assertion |

The test and the code it exercises were last changed on 2026-09-20; #318
touched neither. On the same commit the `CI` workflow ran the same test and
passed. Both sweeps ran the whole CLI suite several times (in every spec whose
block runs it) and lost one line once, in whichever run happened to lose it.

### 1.2 Why a line was lost

The command in that test writes 8192 lines of about 70 bytes to each stream.
`drain_to_stderr` read each pipe in 16 KiB chunks and wrote each chunk to the
parent's stderr as it was read, under the stderr lock. The child's writes are
whole lines, but a read ends wherever the pipe's buffered contents did: when
the child outpaces the drain, a 16 KiB read cuts a line. The other thread's
chunk is then written between the two halves, and one line of each stream is
spliced into the other. Counting lines by prefix finds 8191.

Spec 090 §3.2 promises no ordering between the two streams, and that stays
true. What its test assumed, and the implementation never provided, is that a
line arrives as one line.

## 2. Territory

Extends spec 090's claim on `cmd_verify.rs`, where the drain and its unit tests
live.

## 3. Behavior

### 3.1 Every delivery is whole lines

Each drain MUST deliver to the parent's stderr only whole lines, as one write
under the stderr lock per delivery. The tail of a read that does not end a line
is held and delivered with the rest of its line. Two streams drained
concurrently can therefore interleave only between lines.

### 3.2 Bounds, and what arrives at the end

Spec 090 §3.2's memory bound holds. A held tail longer than 16 KiB is
delivered as it stands, so a command writing without newlines is forwarded in
bounded pieces rather than accumulated, and one delivery never exceeds 32 KiB.
At end of file, or when the pipe cannot be read, a held tail is delivered as
it stands. Draining past a failed delivery (spec 090 D-3) and the three drain
outcomes (D-5) are unchanged.

### 3.3 Evidence

The drain's loop takes its destination as a parameter, so its line discipline
is tested without a process: a line split across reads is delivered whole; a
last line with no newline arrives at end of file; a line three times the bound
arrives in bounded pieces with every byte in order; a failed delivery still
drains to end of file; and two streams drained concurrently into one
destination, with every read ending mid-line, keep all 4096 lines intact.
Spec 090's end-to-end flood test is unchanged and now asserts a property the
forwarder holds rather than one it usually happened to.

## 4. Out of scope

- **Ordering between the streams.** Spec 090 §3.2 promises none, and neither
  does this.
- **The non-`--json` path.** It inherits the child's streams and forwards
  nothing.
- **Changing the flood test.** It asserted the right property; the
  implementation now keeps it.

## 5. Resolved decisions

**D-1 (2026-09-23, build: fail-first by mutation).** With `drain_lines`
delivering each read as it arrives, which is the behavior this replaces, three
of the five unit tests fail (the split line, the final unterminated line, and
the two-stream splice), deterministically: the splice test's reads are 7
bytes. The bound and the failed-delivery cases pass both ways, because the
previous forwarder already kept them. With this change, all five pass, as do
the 19 cases of `verify_streams.rs`.

**D-2 (2026-09-23, the post-merge verdict at `0bf9ff78`).** The two failed
attempts are preserved as the `Acceptance` run's history (run `35822753272`,
attempts 1 and 2) and their artifacts. The run is not re-run again to obtain a
green result: the finding is this spec, and the next merge's sweep is its
evidence.

## Verification

Written to fail against the tree this spec is filed on: `drain_lines` does not
exist, and the unit tests are absent.

```verify:cli
cargo build --release --locked
# 3.1 to 3.3: the line discipline, deterministic.
cargo test -p spec-spine-cli --bin spec-spine --locked cmd_verify::tests::
# Spec 090's end-to-end stream contract, unchanged.
cargo test -p spec-spine-cli --test verify_streams --locked
```
