#!/usr/bin/env bash
# verify-spec.sh <spec-id>: run a spec's `verify:cli` blocks locally.
#
# This is what an orchestrator's verify stage runs after merge, in a clean
# checkout of the merged sha: every non-comment, non-blank line inside a
# ```verify:cli fence under the spec's `## Verification` heading (a numbered
# `## 5. Verification` also counts), from the
# repository root, in order, stopping at the first non-zero exit.
#
#   passed          exit 0   every command exited 0
#   FAILED at N     exit c   command N exited c; later commands did not run
#   not-declared    exit 0   no `## Verification` section, or no verify:cli
#                            commands in it (an honest zero, not a pass)
#   no such spec    exit 2   (also: usage error)
#
# `verify:browser` blocks are counted and skipped: only an orchestrator with a
# browser stage drives those. The script reads the spec markdown, never
# `.derived/`, and runs commands through `sh -c` from the repo root, so a
# command may reference `spec-spine`, `make`, `cargo`, or anything on PATH.
set -u

id="${1:-}"
if [ -z "$id" ]; then
  echo "usage: scripts/verify-spec.sh <spec-id>" >&2
  exit 2
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
specs_dir="${SPECS_DIR:-specs}"
spec="$root/$specs_dir/$id/spec.md"
if [ ! -f "$spec" ]; then
  echo "verify: no such spec: $spec" >&2
  exit 2
fi

# The Verification section: from the heading (numbered or not) to the next H2.
section="$(awk '
  /^## ([0-9]+\. )?Verification[[:space:]]*$/ { on = 1; next }
  on && /^## / { exit }
  on { print }
' "$spec")"

if [ -z "$section" ]; then
  echo "verify: $id: not-declared (no ## Verification section)"
  exit 0
fi

# Fenced blocks: tag on the opening line, body until a bare closing fence.
commands="$(printf '%s\n' "$section" | awk '
  /^```verify:cli[[:space:]]*$/ { inblock = 1; next }
  /^```/ { inblock = 0; next }
  inblock { print }
')"

browser_count="$(printf '%s\n' "$section" | grep -c '^```verify:browser' || true)"
if [ "${browser_count:-0}" -gt 0 ]; then
  echo "verify: $id: $browser_count verify:browser block(s) are driven by the orchestrator; skipped here"
fi

ran=0
while IFS= read -r line; do
  trimmed="${line#"${line%%[![:space:]]*}"}"
  case "$trimmed" in
    ""|\#*) continue ;;
  esac
  ran=$((ran + 1))
  echo "[verify] \$ $trimmed"
  (cd "$root" && sh -c "$trimmed")
  code=$?
  echo "[verify] exit $code"
  if [ "$code" -ne 0 ]; then
    echo "verify: $id: FAILED at command $ran" >&2
    exit "$code"
  fi
done <<EOF
$commands
EOF

if [ "$ran" -eq 0 ]; then
  echo "verify: $id: not-declared (Verification section holds no verify:cli commands)"
  exit 0
fi
echo "verify: $id: passed ($ran command(s))"
