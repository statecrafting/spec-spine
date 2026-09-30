---
id: "188-a-repository-pin-selects-the-engine"
title: "A repository's pin selects the engine that runs"
status: draft
kind: "distribution"
created: "2026-09-30"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "006-distribution"
  - "019-release-supply-chain-artifacts"
  - "132-one-exit-contract-for-the-family"
  - "170-a-consumer-is-served-answers-not-access"
summary: >
  Adds a launcher: a small executable, separate from the engine, that a user
  or a consumer invokes as `spec-spine`. It finds the repository, reads the
  engine pin from `spec-spine.toml` with a tolerant reader, resolves an
  installed engine of exactly that release, verifies its digest, and executes
  it with the original arguments, working directory, streams and exit status.
  Engines are stored once per user, content addressed, and a repository may
  instead declare a local tool directory. A committed `spec-spine.lock`, which
  no engine reads, records each target's engine digest. Acquisition of a
  missing engine happens only when the user or CI enabled it, never because a
  repository asked. A consumer asks `launcher resolve --json` and executes the
  absolute path it returns. The launcher never judges a repository.
establishes:
  - { kind: directory, path: "crates/spec-spine-launcher/" }
  - { kind: file, path: "docs/launcher.md" }
extends:
  # D-14: the claim of docs/launcher.md puts the document into `[index] extra_hashed_inputs`.
  - { spec: "092-the-engine-ships-governance-not-an-environment", unit: { kind: file, path: "spec-spine.toml" }, nature: additive }
references:
  - { unit: { kind: file, path: "specs/092-the-engine-ships-governance-not-an-environment/spec.md" }, role: "section 1.4: the engine's own distribution stays here; the managed environment is Statecraft's" }
  - { unit: { kind: file, path: "specs/170-a-consumer-is-served-answers-not-access/spec.md" }, role: "section 3.1: a repository is judged only by a binary its own pin admits" }
  - { unit: { kind: file, path: "install.sh" }, role: "the existing download, digest and attestation path the launcher's acquisition follows" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "The launcher is a separate executable whose contract is limited to finding the repository, reading the pin and the lock, resolving, verifying and executing an engine; it implements no governance verb."
    anchor: "3-1-launcher-and-engine-are-separate"
  - id: "R-2"
    kind: requirement
    text: "The launcher reads `[meta] required_version` with a reader that ignores every other key, so a newer configuration never prevents it from selecting the engine that understands it."
    anchor: "3-2-what-the-launcher-reads"
  - id: "R-3"
    kind: requirement
    text: "The engine that runs is the exact release the pin names, verified against the lock's digest when a lock is present; a missing release is never replaced by another."
    anchor: "3-4-resolution"
  - id: "R-4"
    kind: requirement
    text: "Acquisition happens only under a policy the user or CI set; a repository's files can select a release but can never enable a download."
    anchor: "3-5-acquisition"
  - id: "R-5"
    kind: requirement
    text: "`launcher resolve --json` answers the engine's absolute path, release, target, digest and the rule that chose it, in the family envelope, and writes and downloads nothing."
    anchor: "3-7-a-consumer-asks-for-the-resolution"
  - id: "I-1"
    kind: invariant
    text: "Execution preserves arguments, working directory, standard streams and exit status, and the launcher never executes itself or another launcher."
    anchor: "3-6-execution"
  - id: "V-1"
    kind: verification
    text: "Fixtures with two repositories pinning different releases, a newer configuration key, a corrupted store entry, a digest that disagrees with the lock, concurrent installs, and a frozen policy prove resolution, verification, refusal and exit-status preservation."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-launcher/tests/resolve.rs"
      - "crates/spec-spine-launcher/tests/execute.rs"
      - "crates/spec-spine-launcher/tests/acquire.rs"
intent:
  goal: "Invoking `spec-spine` in any repository runs the engine that repository pins, with no manual installation step per project and no second resolver in any consumer."
  non_goals:
    - "a governance verb, a daemon, or a background update"
    - "changing which name each distribution channel installs, which is its own spec"
---

# 188: A repository's pin selects the engine that runs

## 1. Purpose

A repository states which engine judges it: `[meta] required_version` in
`spec-spine.toml` (spec 055). Nothing selects that engine. Today:

- `install.sh`, npm, PyPI and crates.io each install one `spec-spine`. Whichever
  was installed last answers for every repository on the machine.
- The engine refuses a pin it does not meet, so the wrong engine refuses rather
  than judging. The user then installs the right one by hand, per repository or
  globally, and the next repository with a different pin repeats it.
- The pin check needs the whole configuration. `Config` rejects unknown keys
  (`deny_unknown_fields` throughout `crates/spec-spine-types/src/config.rs`),
  and `check_version_pin` in `crates/spec-spine-cli/src/main.rs` returns `Ok`
  when the configuration does not load, leaving the verb to fail. So an older
  engine facing a newer repository reports a configuration error, never "wrong
  version", and cannot say which engine would understand it.
- Consumers each invent a resolver. The Statecraft CLI has four ways of
  choosing an executable, measured in its draft spec 028.

Spec 092 section 1.4 keeps the engine's own distribution here while the managed
environment is Statecraft's. Selecting and delivering the engine a repository
pins is engine distribution, so it belongs here, and one mechanism here means no
consumer carries its own.

Something must read the pin before the pinned engine runs. That is the launcher.
It is a proposal, and it authorizes no implementation.

## 2. Territory

A new crate, `crates/spec-spine-launcher/`, planned, and its reference page
`docs/launcher.md`. The engine crates are unchanged: an engine invoked directly
behaves exactly as it does today. Which distribution channel installs the
launcher under the name `spec-spine` is a later spec that amends 006 and 007.

## 3. Behavior

### 3.1 Launcher and engine are separate

The launcher is its own executable, built from its own crate and released with
its own version. It depends on no engine crate. Its whole contract is sections
3.2 to 3.7: find the repository, read the pin and the lock, resolve, verify,
execute. It implements no governance verb and interprets no specification.

It is separate so that the behavior of selection does not depend on which engine
happens to be on `PATH`, and so that its contract can change far more slowly
than the engine's.

Every launcher verb lives under one reserved namespace, `spec-spine launcher
...`. No engine may define a top-level verb named `launcher`. Every other
invocation is passed to the selected engine unchanged.

### 3.2 What the launcher reads

- **The repository.** The nearest ancestor of the working directory holding
  `spec-spine.toml`, or the directory `--repo`/`SPEC_SPINE_REPO` names. Outside
  any repository, the launcher runs no engine and says so, except that
  `--version` and `--help` report the launcher's own.
- **The pin.** `[meta] required_version`, read by a reader that ignores every
  other table and key. A launcher therefore never refuses a configuration
  because an engine newer than itself added a key. The pin must be exact
  (`=X.Y.Z`) for the launcher to select. Any other form is refused, exit 2,
  naming the pin and the remedy.
- **The lock.** `spec-spine.lock` beside `spec-spine.toml`, when present
  (section 3.3).

No engine reads the lock, and the launcher reads nothing else from the
repository.

### 3.3 The lock

`spec-spine.lock` is committed. It is TOML with one table:

```toml
[engine]
release = "0.29.0"                 # must equal the pin
[engine.digests]
"aarch64-apple-darwin" = "sha256:..."
"x86_64-unknown-linux-gnu" = "sha256:..."
```

It lives in its own file because a key added to `spec-spine.toml` is refused by
every engine older than the key. A lock whose `release` differs from the pin is
refused, exit 2, naming both. The lock is written only by `spec-spine launcher
lock`, an explicit verb that records the digests of an engine release for every
target that release publishes. It is never written as a side effect.

A repository without a lock is supported. Its engines are verified against the
release's published digests (section 3.5), and every resolution reports that no
lock bound them.

### 3.4 Resolution

Candidates, in order. The first that answers decides, and none falls through to
a different release:

1. **Override.** `SPEC_SPINE_ENGINE`, an absolute path. It must answer
   `--version` with the pinned release and, when a lock is present, match the
   lock's digest for this target; otherwise it is refused with no fallback.
2. **Project tool directory.** The engine at `.bin/spec-spine` in the
   repository, when that file exists. It must answer the pinned release and,
   when a lock is present, match the lock's digest; otherwise it is refused
   with no fallback. The directory is fixed, not configured. A project uses it
   for hermetic or offline checkouts, and it is where Statecraft's setup
   profile installs the pinned engine.
3. **User store.** `<data dir>/spec-spine/engines/<release>/<target>/<sha256>/spec-spine`,
   where `<data dir>` is the platform's per-user data directory. Entries are
   immutable and content addressed. When a lock is present, only the entry
   named by its digest is eligible.
4. **Nothing else.** `PATH` is never consulted: the launcher is on it. When no
   candidate answers, acquisition (section 3.5) runs if the policy allows it;
   otherwise the invocation is refused, exit 2, naming the release, the target,
   the lock state and `spec-spine launcher install`.

Every resolution digests the file it is about to execute and compares it with
the digest it was selected by. A mismatch in the store quarantines the entry and
refuses. It is never repaired silently.

### 3.5 Acquisition

`spec-spine launcher install` prepares the pinned engine explicitly. Automatic
acquisition inside an ordinary invocation happens only under a policy set
outside the repository:

| Source | Values |
|---|---|
| `--acquire=<policy>` on the invocation | `never`, `auto` |
| `SPEC_SPINE_ACQUIRE` | `never`, `auto` |
| the user's launcher configuration (platform config directory) | `never`, `auto` |
| default | `never` |

`SPEC_SPINE_FROZEN=1` forces `never` above every other source; CI and a
consumer's managed sessions set it. No file in a repository can enable
acquisition. A repository selects a release; it never decides that cloning it
and running a read downloads an executable.

Acquisition downloads the release archive for this target into a temporary
directory in the store, verifies it, then renames it into place in one step
under a per-artifact lock. Concurrent installs of the same artifact never write
the same path. Verification requires the lock's digest when a lock is present,
and otherwise the release's published digest; the source of trust used is
recorded in every resolution. Whether publisher identity is additionally
verified, and against which root, is the owner's decision under section 5.

An unsupported target is refused, naming the target and the supported ones.
Building from source is only ever an explicit `launcher install --build`.

### 3.6 Execution

The selected engine runs with the original arguments, working directory,
environment and standard streams. On Unix the launcher replaces its process
with the engine's. Elsewhere it waits and exits with the engine's exit status,
forwarding interruption. The engine's exit status is the invocation's exit
status, so the family exit contract (spec 132) is the engine's, unchanged.

The launcher sets `SPEC_SPINE_LAUNCHED=<launcher version>` for the child. It
refuses, exit 4, to execute a file that is itself a launcher (it answers
`launcher --version`), and refuses when `SPEC_SPINE_LAUNCHED` is already set,
so it cannot recurse.

### 3.7 A consumer asks for the resolution

`spec-spine launcher resolve --json` answers one family envelope (spec 034,
spec 132) with its own schema axis starting at `0.1.0`. The members below are
carried under the envelope's `report`, as every other `--json` verb carries its
payload:

| Member | Value |
|---|---|
| `repo` | the repository root |
| `release` | the pinned release |
| `target` | the target triple |
| `path` | the engine's absolute path |
| `digest` | the SHA-256 of that file, verified now |
| `rule` | `override`, `tool-dir` or `store` |
| `lock` | `matched`, `absent` |
| `trust` | `lock`, `published-digest` |

It writes nothing and downloads nothing, whatever the policy. Exit 0 when
resolved, 1 when the pinned engine is not installed, 2 when refused (an inexact
pin, a lock that disagrees, a digest mismatch), 3 on usage. A consumer executes
`path` itself, so the identity it records is the identity it runs.

### 3.8 Observable negative cases

| Case | Required behavior |
|---|---|
| Two repositories pinning different releases | Each runs its own release; neither changes the other's store entry |
| A repository whose `spec-spine.toml` carries a key the launcher's release never heard of | The pinned engine is selected and runs |
| An engine on `PATH` of another release | Never consulted |
| The pinned release is not installed, policy `never` | Refused, exit 2, naming `launcher install`; nothing downloaded |
| The same, policy `auto`, `SPEC_SPINE_FROZEN=1` | Refused as above |
| A store entry whose bytes changed | Quarantined and refused |
| A lock digest that disagrees with the downloaded archive | Refused, both digests named; nothing installed |
| Two concurrent installs of one artifact | One writes, the other waits and uses it |
| The engine exits 1 | The invocation exits 1 |
| `SPEC_SPINE_ENGINE` names a launcher | Refused, exit 4 |

## 4. Out of scope

Each is its own spec:

- Which name npm, PyPI, crates.io and `install.sh` install the launcher under,
  and how a machine moves from a globally installed engine to the launcher
  (amends 006 and 007).
- Garbage collection of the store.
- A published list of revoked releases and the launcher's refusal of them.
- Any consumer's adoption; the Statecraft CLI's is its own draft spec 028.

## 5. Resolved decisions

**D-1 (2026-09-30, a separate executable).** Rejected: dispatch built into every
engine release. The launcher that ran would then be whichever engine is on
`PATH`, with that release's dispatch rules, and a release older than this spec
could not dispatch at all.

**D-2 (2026-09-30, the lock is its own file).** Rejected: a `[launcher]` or
`[layout]` key in `spec-spine.toml`. Every engine older than the key would refuse
the configuration, so a repository pinning such an engine could not use it.

**D-3 (2026-09-30, the repository cannot enable acquisition).** A repository
choosing both a release and that it is downloaded would let any clone run an
executable its user never chose. Official releases bound what can arrive, but an
old official release with a known defect is still an official release.

**D-4 (2026-09-30, trust roots are the owner's).** This spec records which
verification each resolution relied on (`trust`) and requires the lock's digest
whenever a lock exists. Adding publisher-identity verification (the existing
build-provenance attestation of spec 019, or a signed release manifest) chooses
a trust root, which is reserved to the owner.

**D-5 (2026-09-30, the digest is of the engine executable).** The lock's
per-target digest, and the `<sha256>` directory name in the store, are the
SHA-256 of the engine executable's bytes, not of the release archive. The
archive's digest names a container that is discarded after extraction, and the
lock must be checkable against the file that will run. Acquisition therefore
verifies twice: the downloaded archive against the release's published `.sha256`
sidecar (as `install.sh` does), then, after extraction, the engine against the
lock's digest when a lock exists. A mismatch at either step names both digests
and installs nothing. Rejected: locking the archive digest, which would let a
store entry drift from its name unnoticed once the archive is gone.

**D-6 (2026-09-30, acquisition follows `install.sh`).** Acquisition downloads with
a `curl` subprocess and extracts with a `tar` subprocess, so the launcher links no
HTTP client and no archive library. The release source base defaults to
`https://github.com/statecrafting/spec-spine/releases/download` and
`SPEC_SPINE_RELEASE_BASE` overrides it with an `http(s)` URL, a `file://` URL or a
plain directory path, all laid out as `<base>/v<release>/<archive>`, so tests run
with no network. Archive names are those `.github/workflows/release.yml`
produces: `spec-spine-v<release>-<target>.tar.gz` (`.zip` on Windows), each with
`<archive>.sha256`. Zip extraction uses `tar` where it reads zip and `unzip`
otherwise; where neither does, Windows acquisition is refused as unsupported
rather than approximated. A target whose archive the release does not publish
(a local file that is absent, or an HTTP error from `curl -f`) reads as not
published: `launcher lock` records the targets that are published, and `launcher
install` refuses naming the supported targets.

**D-7 (2026-09-30, where per-user state lives).** `SPEC_SPINE_HOME` replaces both
roots: the store is `$SPEC_SPINE_HOME/engines/...` and the user's configuration is
`$SPEC_SPINE_HOME/launcher.toml`. Otherwise the platform's per-user directories
are computed by hand under a `spec-spine` subdirectory: `XDG_DATA_HOME` or
`~/.local/share`, and `XDG_CONFIG_HOME` or `~/.config`, on Linux and other Unix;
`~/Library/Application Support` on macOS; `%LOCALAPPDATA%` and `%APPDATA%` on
Windows. Rejected: a directories crate, for two lookups the launcher can state.

**D-8 (2026-09-30, quarantine).** A store entry whose bytes no longer match the
digest it is named by is renamed to a sibling `<sha256>.quarantined-<n>` (the
lowest free `n`) and the invocation is refused. The entry is kept for inspection,
never deleted and never repaired. `launcher resolve` writes nothing, so it reports
the same refusal and leaves the entry where it is.

**D-9 (2026-09-30, the per-artifact install lock).** The lock is an exclusive create
(`create_new`) of `.install.lock` in the `<release>/<target>` directory, beside
the entries. The artifact is the release archive for a target, whose engine digest
is not known before extraction, so the lock is keyed by release and target rather
than by the entry name. The installer that creates the file writes; the others
poll (50 ms, at most 180 s), re-resolving on each poll, and use the winner's entry
when it appears. A lock file older than ten minutes is treated as abandoned and
removed. The entry is staged in a scratch directory in the same parent and placed
by one rename.

**D-10 (2026-09-30, packaging of the crate).** The crate is a workspace member with
`publish = false` for now: which channel ships the launcher is out of scope under
section 4. It carries its own version, `0.1.0`, not the engine's workspace
version (section 3.1: released with its own version). Its binary target is named
`spec-spine-launcher`, because a target named `spec-spine` would collide with
`spec-spine-cli`'s in the same workspace target directory. A channel installs it
under the name `spec-spine`; `docs/launcher.md` says so. That renaming is the
later spec section 2 already names.

**D-11 (2026-09-30, `launcher install --build`).** `launcher install --build` is
refused in this implementation, exit 2, saying it is not supported yet. Building
from source is only ever explicit (section 3.5), and refusing it leaves that
sentence true; the build path (toolchain discovery, a source checkout at the
pinned release, a digest for a locally built engine that no published sidecar
attests) is a separate piece of work and a trust question for the owner.

**D-12 (2026-09-30, the envelope of `launcher resolve --json`).** Header members
and ordering are spec 132 section 3.4's: `schemaVersion` (`0.1.0`, the launcher's
own axis), `tool`, `verb`, `outcome`, `exitCode`, `summary`, then exactly one of
`report` and `error`, written as canonical JSON (sorted keys, 2-space indent, LF,
trailing newline). `tool` is `spec-spine-launcher`, not `spec-spine`, because a
family consumer branches on `tool` before `verb` and this document is not the
engine's; the `verb` is `launcher.resolve`. The launcher defines this envelope
locally and depends on no engine crate. A resolution that does not succeed also
writes an envelope on stdout, with `error` in place of `report` and the exit code
of section 3.7: `not-found` for exit 1, `config` or `refused` for 2, `usage` for
3. `digest` is written as `sha256:` followed by the hex digest, the spelling the
lock uses, so a consumer can compare the two without converting.

**D-13 (2026-09-30, resolution and invocation details section 3 left open).**
(a) A project tool directory whose engine is absent falls through to the store; one
that is present but fails the release or digest check is refused, not skipped,
because a stale hermetic directory is a fault worth hearing about. (b) With no
lock and more than one store entry for a release and target, the launcher refuses
as ambiguous and names `launcher lock`; it does not choose. (c) A lock that
records no digest for the host's target is refused. (d) The digest of an override
or tool-directory candidate is checked against a lock before the file is run for
its `--version` and `launcher --version` probes. (e) `launcher install` is
explicit consent and does not depend on the acquisition policy, `SPEC_SPINE_FROZEN`
included; the policy governs only acquisition inside an ordinary invocation.
(f) `--acquire` is the launcher's own flag and is removed from the arguments passed
on; `--repo` is shared with the engine and is passed on. (g) When the launcher
found the repository somewhere other than the working directory (an ancestor, or
`SPEC_SPINE_REPO`) and the invocation carries no `--repo`, it passes `--repo
<root>` first, because the engine defaults its repository to the working
directory and would otherwise judge a different tree than the one it was selected
for. When the working directory is the root, or `--repo` was given, the arguments
are passed exactly as received. (h) On Unix the launcher does not intercept
signals: it replaces its process, so interruption is the engine's own. Elsewhere it
waits and returns the engine's status.

**D-14 (2026-09-30, the reference page is a hashed input).** `docs/launcher.md` is a
claimed file that no span backs, so lint `L-008` requires it to be in some
content hash. It joins `[index] extra_hashed_inputs` in `spec-spine.toml`, as
`docs/overlay-contract.md` did under spec 112 D-3, and this spec declares an
`additive` `extends` edge on that unit of spec 092. Only the one entry is added.

**D-15 (2026-09-30, the project tool directory is `.bin/`).** The owner decided
that neither spec-spine nor Statecraft uses `.tooling/bin`, and that the
repository-local engine lives in `.bin/`. The lock's optional `tool_dir` key is
removed rather than defaulted: a configurable directory is one more thing two
tools must agree on, and Statecraft's resolution (its spec 028) and setup
profile look in one fixed place. `.bin/spec-spine` is a candidate with or
without a lock; without one it is put to the pin alone, as the override is. A
`.tooling/bin/spec-spine` left in a checkout is not a candidate. This
repository's own gate still installs into `.tooling/bin` because its gate
script and workflow are rendered by Statecraft's setup profile, and they move
to `.bin/` when this repository adopts the profile revision that renders them
so. Until then this spec changes only the launcher.

## Verification

The `V-1` inputs are the three test files below. They use stub engines of two
releases in a temporary store, a local release fixture built with `tar` for
acquisition, and no network.

```verify:cli
cargo test -p spec-spine-launcher --test resolve --locked
cargo test -p spec-spine-launcher --test execute --locked
cargo test -p spec-spine-launcher --test acquire --locked
cargo test -p spec-spine-launcher --bins --locked
```
