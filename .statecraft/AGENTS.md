# Statecraft: managed project instructions

This file is **managed by Statecraft**. It is rewritten on upgrade and removed
on removal, and its ownership is recorded in `.statecraft/environment.json`.
Write your own instructions in the repository's own `AGENTS.md`, which
Statecraft never owns and never rewrites.

## The area

| Path | Committed | Holds |
|---|---|---|
| `.statecraft/environment.json` | yes | The environment declaration: pins, managed-file ownership, tracked modifications, and the project block. |
| `.statecraft/AGENTS.md` | yes | This file. |
| `.statecraft/derived/` | yes | spec-spine's compiled artifacts. |
| `.statecraft/state/` | no | Runtime state. Ignored. |

`.statecraft/` as a whole is never ignored. Ignoring it would take the
declaration, these instructions and the compiled artifacts out of version
control in one line, and those three are what make the project governed.

## Reading the governed artifacts

Read `.statecraft/derived/` only through `spec-spine` subcommands. A typed read
fails at the deserializer with a clean error; an ad-hoc parse silently encodes a
stale assumption.

## Four acts, never inferred from one another

Registration makes a project visible. Qualification is a read-only verdict.
Arming consents to the project being driven. An execution posture is a separate
consent again. None of them implies the next.

## Where a value came from

`statecraft-cli config show <path>` answers per key, with the layer that
supplied it and every layer that constrained it. A key no layer supplies is
unknown, and unknown is not success: do not substitute a default for it.
