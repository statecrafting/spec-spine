# The launcher

Governed by [spec 188](../specs/188-a-repository-pin-selects-the-engine/spec.md).

The launcher is a small executable, separate from the engine, that finds a
repository, reads the engine release it pins, resolves an installed engine of
exactly that release, verifies its digest, and executes it. It implements no
governance verb and interprets no specification. It depends on no engine crate.

The crate is `crates/spec-spine-launcher/`, released with its own version
(currently `0.1.0`, unrelated to the engine's). Its binary target is named
`spec-spine-launcher`, because `spec-spine-cli` already owns the `spec-spine`
target in this workspace. A distribution channel installs it under the name
`spec-spine`; which channel does, and how a machine moves from a globally
installed engine to the launcher, is a later spec (188 section 4). This page
calls the executable `spec-spine`, as a user will.

## What it reads

| Source | What is read |
|---|---|
| The repository | The directory named by `--repo`, else by `SPEC_SPINE_REPO`, else the nearest ancestor of the working directory holding `spec-spine.toml`. |
| The pin | `[meta] required_version` in `spec-spine.toml`, read by a reader that ignores every other table and key. It must be exact: `=X.Y.Z` (an optional `-prerelease` is allowed). Any other form is refused, exit 2. |
| The lock | `spec-spine.lock` beside `spec-spine.toml`, when present. No engine reads it. |
| The user's launcher configuration | `launcher.toml` (see below), for the acquisition policy only. |

The launcher reads nothing else from the repository. Outside a repository it
runs no engine and says so, exit 2, except that `--version` and `--help` report
the launcher's own.

## The lock

`spec-spine.lock` is committed. It is TOML with one table:

```toml
[engine]
release = "0.29.0"                 # must equal the pin
[engine.digests]
"aarch64-apple-darwin" = "sha256:..."
"x86_64-unknown-linux-gnu" = "sha256:..."
```

- A lock whose `release` differs from the pin is refused, exit 2, naming both.
- Each digest is the SHA-256 of the **engine executable** for that target, as
  `sha256:` followed by 64 lowercase hex digits. It is not the digest of the
  release archive (spec 188 D-5).
- A lock that records no digest for the host's target is refused, exit 2.
- The lock is written only by `spec-spine launcher lock`, never as a side effect.

A repository without a lock is supported. Its engines are verified against the
release's published digest, and every resolution reports `lock: absent`.

## Resolution order

The first candidate that answers decides. None falls through to a different
release, and `PATH` is never consulted.

1. **Override.** `SPEC_SPINE_ENGINE`, an absolute path. It must answer
   `--version` with the pinned release and, when a lock is present, match the
   lock's digest for the host's target. Otherwise it is refused, with no
   fallback.
2. **Project tool directory.** When `.bin/spec-spine` exists in the
   repository, that file, under the same two conditions. The directory is
   fixed; the lock does not name it. A
   file that is present but fails them is refused rather than skipped (D-13).
3. **User store.** `<data root>/engines/<release>/<target>/<sha256>/spec-spine`.
   Entries are immutable and named by the SHA-256 of the executable. When a lock
   is present only the entry it names is eligible. Without a lock, exactly one
   entry may exist; several are refused as ambiguous.
4. **Nothing else.** If no candidate answers, acquisition runs when the policy
   allows it; otherwise the invocation is refused, exit 2, naming the release,
   the target, the lock state and `spec-spine launcher install`.

Every resolution digests the file it is about to execute. A locked repository's
override and tool directory are digested before anything is run. A store entry
whose bytes no longer match its name is renamed to a sibling
`<sha256>.quarantined-<n>` and the invocation is refused; `launcher resolve`
reports the same refusal but leaves the entry in place, because it writes
nothing. A candidate that answers `launcher --version` is a launcher and is
refused, exit 4.

### The data root

| Platform | Store root (`engines/` lives under it) | Configuration file |
|---|---|---|
| any, with `SPEC_SPINE_HOME` set | `$SPEC_SPINE_HOME` | `$SPEC_SPINE_HOME/launcher.toml` |
| Linux and other Unix | `$XDG_DATA_HOME/spec-spine`, else `~/.local/share/spec-spine` | `$XDG_CONFIG_HOME/spec-spine/launcher.toml`, else `~/.config/spec-spine/launcher.toml` |
| macOS | `~/Library/Application Support/spec-spine` | `~/Library/Application Support/spec-spine/launcher.toml` |
| Windows | `%LOCALAPPDATA%\spec-spine` | `%APPDATA%\spec-spine\launcher.toml` |

## Acquisition policy

Acquisition inside an ordinary invocation happens only under a policy set
outside the repository. In precedence order:

1. `SPEC_SPINE_FROZEN=1` forces `never`, above everything else.
2. `--acquire=never|auto` on the invocation (the launcher removes this flag; the
   engine never sees it).
3. `SPEC_SPINE_ACQUIRE=never|auto`.
4. `acquire = "never"|"auto"` in the user's `launcher.toml`. Other keys there
   are ignored.
5. The default, `never`.

No file in a repository can enable acquisition. A missing engine under `never`
is refused, exit 2, and nothing is downloaded.

`spec-spine launcher install` prepares the pinned engine explicitly and does not
depend on the policy: naming it is the consent (D-13). `launcher install
--build` is not supported yet and is refused, exit 2.

Acquisition downloads `spec-spine-v<release>-<target>.tar.gz` (`.zip` on
Windows) and its `.sha256` sidecar, the names `release.yml` publishes, from
`<base>/v<release>/`. The base is
`https://github.com/statecrafting/spec-spine/releases/download` unless
`SPEC_SPINE_RELEASE_BASE` names another: an `http(s)` URL (fetched with `curl`),
a `file://` URL, or a plain directory path with the same layout. The archive is
checked against its sidecar and extracted with `tar` (a `.zip` needs a `tar`
that reads zip, or `unzip`). The extracted engine is then digested; when a lock
exists its digest must match, otherwise the install is refused naming both
digests and nothing is placed. Placement is one rename, under a per-artifact
lock file (`.install.lock` in the release and target directory, created
exclusively). A concurrent installer waits and then uses the winner's entry. A
lock file older than ten minutes is treated as abandoned.

Publisher identity (the build-provenance attestation of spec 019) is not
verified; which trust root to use is the owner's decision (spec 188 D-4). The
`trust` member of every resolution says what was relied on.

## `launcher` verbs

Every launcher verb lives under `spec-spine launcher`. Every other invocation is
passed to the selected engine unchanged.

| Invocation | Effect |
|---|---|
| `launcher resolve [--json]` | Answers which engine would run. Writes and downloads nothing, whatever the policy. |
| `launcher install` | Resolves, and if nothing answers, acquires. Prints `installed <path>` or `present <path>`. |
| `launcher lock` | Fetches each published target's archive, digests the engine inside, and writes `spec-spine.lock`. Places nothing in the store. |
| `launcher --version` | `spec-spine-launcher <version>`. |

### `launcher resolve --json`

One envelope, on the launcher's own schema axis (`schemaVersion` `0.1.0`).
Header members, in sorted order like every family document: `exitCode`,
`outcome`, `report` or `error`, `schemaVersion`, `summary`, `tool`
(`spec-spine-launcher`), `verb` (`launcher.resolve`). The engine's identity is
carried under `report`:

| Member | Value |
|---|---|
| `repo` | the repository root |
| `release` | the pinned release |
| `target` | the target triple |
| `path` | the engine's absolute path; a consumer executes this itself |
| `digest` | `sha256:` and the SHA-256 of that file, verified now |
| `rule` | `override`, `tool-dir` or `store` |
| `lock` | `matched` or `absent` |
| `trust` | `lock` or `published-digest` |

When there is no `report`, `error` carries `kind` and `message`. Kinds used:
`not-found` (exit 1), `config` and `refused` (exit 2), `usage` (exit 3), `io`
and `internal` (exit 4).

## Execution

The engine runs with the original arguments, working directory, environment and
standard streams, plus `SPEC_SPINE_LAUNCHED=<launcher version>`. On Unix the
launcher replaces its process with the engine's; elsewhere it waits and exits
with the engine's status. The engine's exit status is the invocation's.

The launcher refuses, exit 4, to run when `SPEC_SPINE_LAUNCHED` is already set.
When it found the repository somewhere other than the working directory (an
ancestor, or `SPEC_SPINE_REPO`) and no `--repo` was given, it prepends
`--repo <root>` so the engine judges the repository the launcher chose the
engine for (D-13).

## Exit codes

The family contract (spec 132).

| Code | Meaning here |
|---|---|
| 0 | Resolved, installed, or the engine's own success |
| 1 | `launcher resolve`: the pinned engine is not installed |
| 2 | Refused: not in a repository, inexact pin, lock disagrees, digest mismatch, engine not installed under policy `never`, unsupported target, bad configuration |
| 3 | Usage |
| 4 | Failed: an I/O error, recursion, or an engine that is itself a launcher |

For any invocation that reaches an engine, the engine's exit status is returned
unchanged, so an engine exit of 1, 2, 3 or 4 is the invocation's.

## Environment variables

| Variable | Effect |
|---|---|
| `SPEC_SPINE_REPO` | Names the repository (the `--repo` flag wins). |
| `SPEC_SPINE_ENGINE` | Absolute path of an override engine. |
| `SPEC_SPINE_ACQUIRE` | `never` or `auto`. |
| `SPEC_SPINE_FROZEN` | `1` forces policy `never`. |
| `SPEC_SPINE_HOME` | Replaces the data root and the configuration directory. |
| `SPEC_SPINE_RELEASE_BASE` | Where release archives come from. |
| `SPEC_SPINE_LAUNCHED` | Set for the child; refused when already set. |
