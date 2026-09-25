# Security policy

## Reporting a vulnerability

Report a suspected vulnerability privately through GitHub's private
vulnerability reporting for this repository:

<https://github.com/statecrafting/spec-spine/security/advisories/new>

Please do not open a public issue, pull request or discussion for a suspected
vulnerability. A private report reaches the maintainers only; they will
acknowledge it, work on a fix in a private advisory, and credit the reporter in
the published advisory unless asked not to.

Include what you can of:

- the affected version (`spec-spine --version`) and distribution channel
  (crates.io, npm, PyPI, `install.sh` or a release archive);
- the input that triggers it (a spec corpus, `spec-spine.toml`, a diff, an
  attestation) reduced as far as possible;
- what happens and what you expected.

## Supported versions

Fixes are made on the latest release. There is no backport line for earlier
versions; upgrade to the newest release to receive a fix.

## Scope

In scope: the `spec-spine` CLI and the `spec-spine-core` and
`spec-spine-types` libraries, the npm and PyPI distribution shims, `install.sh`,
and the release workflow's supply-chain artifacts (checksums, SBOMs, provenance
attestations).

`spec-spine verify` executes the commands a spec's `## Verification` block
declares, by design; running it on an untrusted corpus runs untrusted commands.
That is documented behavior, not a vulnerability. The gate chain (`check`,
`lint`, `couple`, `index coverage`) reads and never executes.
