#!/usr/bin/env bash
# Spec: specs/090-a-hook-bound-to-a-tool-route-misses-the-work/spec.md
#
# One-command, idempotent enablement of this repository's committed git hooks
# in THIS clone. Run once per clone: `core.hooksPath` lives in per-clone
# `.git/config`, which is not committed, so each clone (you may keep several)
# enables it separately. Worktrees created off a clone inherit the clone's
# config, so one run covers every worktree under it.
#
#   ./.githooks/enable-hooks.sh
#
# Disable:
#   git config --unset core.hooksPath
#
# What it turns on is `.githooks/pre-commit` (spec 090): a refusal, never a
# repair, at the one boundary no tool route can bypass. Until this script is
# run the hook is inert bytes in the tree.
#
# NOTE: `core.hooksPath` replaces the hook search path wholesale. Any script
# you keep in `.git/hooks/` stops running once this is set. Move it into
# `.githooks/` first if you rely on it.

set -eu

root="$(git rev-parse --show-toplevel)"
cd "$root"

if [ ! -d .githooks ]; then
  echo "no .githooks/ in $root" >&2
  exit 3
fi

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit 2>/dev/null || true

echo "core.hooksPath = $(git config core.hooksPath)"
echo "enabled: $(ls .githooks | tr '\n' ' ')"
echo "disable with: git config --unset core.hooksPath"
