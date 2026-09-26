---
id: "149-the-installer-is-tested"
title: "The installer is tested"
status: approved
kind: "tooling"
created: "2026-09-25"
summary: >
  Spec 006 is one of the four specs the pre-release sweep still reports
  `exempt`, because `install.sh`, the tag-gated release pipeline and npm
  publication "run only in a release" (139 3.3). Two of its three channels do
  not: `install.sh` is a shell script whose every decision (platform, tag,
  checksum, attestation policy, install directory) can be exercised offline
  against a local fixture release, and the npm shim already has an offline
  unit test and smoke test. This spec adds an offline test of `install.sh`,
  gives 006 an executable acceptance that runs it and the npm shim's tests,
  and removes 006's line from the legacy ledger. The release pipeline's
  publication steps stay unexercised by it, and it says so.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "006-distribution"
  - "089-nothing-reruns-a-merged-acceptance"
  - "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
amends:
  - "006-distribution"
  - "139-the-legacy-ledger-is-paid-verdicts-and-lifecycle"
amends_verification:
  - "006-distribution"
establishes:
  - { kind: file, path: "scripts/test-install.sh" }
extends:
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: corrective }
---

# 149: The installer is tested

## 1. Purpose

The sweep of `7ce4935f` ended `passed=143 failed=0 exempt=4`. The ledger's
reason for 006 is that its territory runs only in a release. That is true of
`release.yml`'s publish jobs, and not of `install.sh` or `npm/`:

- `install.sh` reaches the network only through `curl` or `wget`, and the
  platform only through `uname`. With both replaced by stubs on `PATH`, every
  branch runs offline.
- `npm/` already has `npm test` (the platform map) and `npm run smoke` (pack,
  install and launch through the shim), both offline, both run by the release
  qualification but by no acceptance block.

## 2. Territory

- `scripts/test-install.sh` (new): the offline installer test.
- `scripts/verify-sweep.sh`: 006's ledger line is removed.

## 3. Behavior

### 3.1 The installer runs against a fixture release

`scripts/test-install.sh` MUST build a fixture release directory holding one
archive (`spec-spine-<tag>-<triple>.tar.gz` containing a stub `spec-spine`
that prints a version) and its `.sha256` sidecar, put a `curl` stub on `PATH`
that serves those URLs from the directory (and fails any other), and run
`install.sh` with `SPEC_SPINE_VERSION`, `SPEC_SPINE_BIN_DIR` and
`SPEC_SPINE_SKIP_ATTESTATION=1` unless a case says otherwise. Each case runs
against a fresh `SPEC_SPINE_BIN_DIR` and asserts the installer's exit code. The
cases are:

1. a matching checksum installs the binary into `SPEC_SPINE_BIN_DIR`, exit 0,
   and the installed binary runs;
2. a sidecar that does not match refuses, exit 1, with `checksum mismatch` on
   stderr, and installs nothing;
3. an archive without the binary, with a sidecar that matches it (so the
   installer reaches extraction), refuses, exit 1, and installs nothing;
4. a `uname` stub reporting an unsupported OS refuses, exit 1, naming the OS;
5. a `uname` stub reporting an unsupported architecture refuses, exit 1,
   naming it;
6. `SPEC_SPINE_REQUIRE_ATTESTATION=1`, with `SPEC_SPINE_SKIP_ATTESTATION`
   unset and a `PATH` holding no `gh`, refuses, exit 1, and installs nothing;
7. `SPEC_SPINE_VERSION=latest` resolves the tag from the stubbed releases API
   response (`{"tag_name": "<tag>"}`) and installs that tag's archive, exit 0.

The script MUST print one line per case naming it and its result, and end with
exactly `install.sh: 7 of 7 cases passed` when all pass; it exits 0 only then,
and 1 otherwise.

The triple under test is the host's, from the same `uname` mapping the
installer uses, so the test runs on every sweep host.

### 3.2 006 has an executable acceptance

This spec's block replaces 006's (`amends_verification`): it runs
`scripts/test-install.sh`, `npm test` and `npm run smoke`.

### 3.3 The ledger shrinks (amends 139 3.3)

`006-distribution` MUST be removed from `legacy_ledger()`; `LEDGER_CLOSED_AT`
is unchanged (the ledger only shrinks). The sweep then reports 006 `passed`
or `failed`, never `exempt`.

## 4. Out of scope

- `release.yml`'s build, attestation and publish jobs, and the published
  channels: they run in a release and are witnessed by the release record's
  channel checks, not by acceptance.
- musl detection: it reads `/lib/ld-musl-*` and `ldd`, which a stub cannot
  redirect without editing the installer.

## 5. Resolved decisions

**D-1 (2026-09-25): the installer's PATH is built, not inherited.** 3.1 asks
for stubs on `PATH` and, in case 6, a `PATH` holding no `gh`; a host that has
`gh` or `wget` installed would otherwise reach the installer. Every case runs
`install.sh` under `env -i` with `PATH` set to the stub directory (plus the
case's `uname` stub, in cases 4 and 5) followed by a toolbox of symlinks to the
host tools the installer calls (`sh`, `cat`, `grep`, `sed`, `awk`, `tr`,
`mktemp`, `tar`, `gzip`, `chmod`, `mkdir`, `mv`, `rm`, `uname`, and whichever
of `sha256sum`, `shasum`, `openssl`, `perl`, `ldd`, `env` exist). No `gh` and
no `wget` are ever on it, and no host `SPEC_SPINE_*` variable leaks in. Case 6
also refuses to run if a `gh` appears in either directory.

**D-2 (2026-09-25): `SPEC_SPINE_INSTALLER` selects the installer under test.**
It defaults to the repository's `install.sh`; a mutation harness points it at
an edited copy so `install.sh` itself is never modified to prove a case can
fail. The seven mutations (install step removed, checksum comparison removed,
missing-binary guard removed, unsupported-OS arm falling through, unsupported
architecture arm falling through, required-attestation refusal removed, latest
resolution ignoring the API) each fail the case that guards it, and the script
exits 1.

**D-3 (2026-09-25): each refusal case also asserts its message.** Exit code and
an empty install directory alone cannot tell case 3's guard from `set -e`
failing at the following `chmod`, or cases 4 and 5's refusals from an unset
variable. So case 3 asserts `checksum verified` (the installer reached
extraction) and `archive did not contain spec-spine`, cases 4 and 5 assert the
named OS or architecture, case 6 asserts the unverified-attestation refusal,
and case 7 asserts the installer announced the resolved tag and installed that
tag's binary.

**D-4 (2026-09-25): the host triple is mapped in the test, and case 1 checks
the mapping.** The test repeats the installer's `uname` case arms. If the two
drifted, the fixture archive's name would not be the one the installer asks
for, and case 1 would fail on the download.

## Verification

```verify:cli
# 3.1: all seven cases, counted exactly.
sh -c 'sh scripts/test-install.sh > "${TMPDIR:-/tmp}/ss149.out" 2>&1; rc=$?; grep -qx "install.sh: 7 of 7 cases passed" "${TMPDIR:-/tmp}/ss149.out"; g=$?; rm -f "${TMPDIR:-/tmp}/ss149.out"; test $rc -eq 0 && test $g -eq 0'
# 3.2: the npm shim's offline unit test and pack-install-launch smoke test.
sh -c 'cd npm && npm test'
sh -c 'cd npm && npm run smoke'
# 3.3: the ledger no longer lists 006.
sh -c '! grep -qx "006-distribution" scripts/verify-sweep.sh'
```
