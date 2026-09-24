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
