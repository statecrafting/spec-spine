# Windows test findings

Spec 134 runs the workspace test suite on `windows-latest` as the `test
(windows)` check. A test that does not run there is gated or quarantined in its
own source, and each one is listed here. The workflow never skips a test.

Every entry below still runs on Linux in the `build · test · clippy` job on
every change, so what is excluded is Windows coverage of that test, not the
test itself.

| Id | Where | Gate | Why it does not run on Windows | Exit |
|---|---|---|---|---|
| WF-1 | `crates/spec-spine-core/tests/ai_review_policy.rs` | `#![cfg(unix)]` | executes the `ai-review` workflow's own `run:` bash scalars, which run on `ubuntu-latest` only | none planned: the workflow is Linux-only |
| WF-2 | `crates/spec-spine-core/tests/gate_binary.rs` | `#![cfg(unix)]` | drives the root `Makefile` with `make` and POSIX shell stand-ins | none planned: the governed gate runs on Linux |
| WF-3 | `crates/spec-spine-core/tests/harness_hooks.rs` | `#![cfg(unix)]` | runs this repository's `.claude/settings.json` hook bodies through `sh` with executable stand-ins | none planned: the harness is POSIX shell |
| WF-4 | `crates/spec-spine-core/tests/commit_boundary.rs` | `#![cfg(unix)]` | runs `.githooks/pre-commit` through `sh` | none planned |
| WF-5 | `crates/spec-spine-core/tests/reader_identity.rs` | `#![cfg(unix)]` | runs `scripts/reader-identity.sh` and executable shims | none planned |
| WF-6 | `crates/spec-spine-core/tests/gate.rs`, `a_guarded_recipe_skips_when_absent_and_fails_when_the_command_fails` | `#[cfg(unix)]` | runs a Makefile recipe through `sh` | none planned |

Tests already gated to Unix before spec 134 (symbolic links, permission bits,
process groups) are gated because the behavior they assert is Unix behavior;
they are not quarantines and are not listed.

## Fixed or quarantined after the first Windows run

| Id | Where | Status | Finding |
|---|---|---|---|
| WF-7 | `cli.rs`: `delta_classifies_under_the_merge_base_not_the_checked_out_head`, `delta_prose_says_what_not_required_does_not_mean`; `couple.rs`: `no_deletion_builds_no_prior_snapshot` | **fixed** in spec 143; quarantine lifted | The fixtures commit shards to a scratch repository with no `.gitattributes`; git on the Windows runner (`core.autocrlf=true`) checked them out with CRLF endings, and the byte comparison of spec 086 reported every shard `modified`. An adopter on Windows with `core.autocrlf=true` saw the same false staleness. The freshness comparisons (registry shards, index shards, the inputs sidecar) now read a committed file whose only difference is CRLF for LF as the same text; any other difference, a lone CR included, is still drift. `core/tests/line_endings.rs` converts a committed tree to CRLF on every platform, and the three tests run on Windows again. |
| WF-8 | `crates/spec-spine-core/src/compact.rs` (retire and historical plan paths) | **fixed** in spec 134 | `Path::is_absolute` is false on Windows for `/etc/passwd` (no drive prefix), so the plan-path refusal failed open there. The check is now lexical (`/` or `\` prefix, or absolute), as spec 128 decides `derived_dir`. Covered by `retire.rs`'s existing cases on both platforms. |
