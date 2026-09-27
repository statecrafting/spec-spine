#!/usr/bin/env bash
# Spec: specs/156-statecraft-profile-10-governs-this-repository/spec.md
#
# Check repository-authored files, pull request text, and commit messages for
# the two publication hazards that this repository refuses. Generated derived
# shards are excluded from tree and range scans. NUL-delimited inventories keep
# filenames containing spaces intact.
#
# Exit 0 clean, 1 finding, 2 refused, 3 usage, 4 failed.

if [ -z "${BASH_VERSION:-}" ]; then
  echo "check-authored-content.sh requires bash; run it through its executable shebang" >&2
  exit 3
fi

set -uo pipefail

status=0
mode=tree

usage() {
  echo "usage: check-authored-content.sh [--text FILE...] [--range BASE HEAD] [--self-test]" >&2
  exit 3
}

if [ "${1:-}" = "--self-test" ]; then
  [ "$#" -eq 1 ] || usage
  me=$(cd "$(dirname "$0")" && pwd)/$(basename "$0")
  tmp=$(mktemp -d "${TMPDIR:-/tmp}/spec-spine-authored.XXXXXX") || exit 4
  trap 'rm -rf "$tmp"' EXIT
  failed=0

  expect() {
    want=$1
    shift
    "$me" "$@" >/dev/null 2>&1
    got=$?
    if [ "$got" -ne "$want" ]; then
      echo "self-test: expected exit $want, got $got: $*" >&2
      failed=1
      return 1
    fi
    return 0
  }

  printf '%s\n' 'plain authored text' > "$tmp/clean file.txt"
  printf '\342\200\224\n' > "$tmp/forbidden-dash.txt"
  printf '%s%s\n' 'HTTPS://CODEX.AI/' 'CODE/SESSION_example' > "$tmp/forbidden-url.txt"
  : > "$tmp/empty.txt"
  expect 0 --text "$tmp/clean file.txt"
  expect 1 --text "$tmp/forbidden-dash.txt"
  expect 1 --text "$tmp/forbidden-url.txt"
  expect 0 --text "$tmp/empty.txt"
  expect 3 --text
  expect 3 --unknown

  mkdir "$tmp/range repo" || exit 4
  (
    cd "$tmp/range repo" || exit 4
    git init -q
    git config user.email self-test@example.invalid
    git config user.name self-test
    printf '%s\n' clean > "file with spaces.txt"
    git add "file with spaces.txt"
    git commit -qm base
    "$me" --range HEAD HEAD >/dev/null 2>&1
  ) || {
    echo "self-test: expected exit 0 for an empty range" >&2
    failed=1
  }

  [ "$failed" -eq 0 ] || exit 1
  echo "check-authored-content: self-test passed"
  exit 0
fi

files=()
case "${1:-}" in
  "")
    root=$(git rev-parse --show-toplevel 2>/dev/null) || {
      echo "check-authored-content: not inside a git work tree" >&2
      exit 2
    }
    cd "$root" || exit 4
    while IFS= read -r -d '' file; do
      case "$file" in
        .statecraft/derived/*) continue ;;
      esac
      [ -f "$file" ] && files+=("$file")
    done < <(git ls-files -z --cached --others --exclude-standard) || exit 4
    ;;
  --text)
    mode=text
    shift
    [ "$#" -gt 0 ] || usage
    for file in "$@"; do
      [ -f "$file" ] || {
        echo "check-authored-content: no such file: $file" >&2
        exit 3
      }
      files+=("$file")
    done
    ;;
  --range)
    mode=range
    [ "$#" -eq 3 ] || usage
    root=$(git rev-parse --show-toplevel 2>/dev/null) || {
      echo "check-authored-content: not inside a git work tree" >&2
      exit 2
    }
    cd "$root" || exit 4
    if ! git rev-parse --verify "${2}^{commit}" >/dev/null 2>&1 \
      || ! git rev-parse --verify "${3}^{commit}" >/dev/null 2>&1; then
      echo "check-authored-content: the range endpoints must name commits" >&2
      exit 3
    fi
    while IFS= read -r -d '' file; do
      case "$file" in
        .statecraft/derived/*) continue ;;
      esac
      [ -f "$file" ] && files+=("$file")
    done < <(git diff --name-only -z "$2...$3" --) || exit 4
    ;;
  *) usage ;;
esac

dash=$(printf '\342\200\224')
url_a='https://codex.ai/'
url_b='code/session_'

for ((i = 0; i < ${#files[@]}; i++)); do
  file=${files[$i]}
  if LC_ALL=C grep -nI -F -- "$dash" "$file" > "${TMPDIR:-/tmp}/spec-spine-authored-dash.$$"; then
    echo "U+2014 is refused in $file:" >&2
    sed 's/^/  /' "${TMPDIR:-/tmp}/spec-spine-authored-dash.$$" >&2
    status=1
  elif [ "$?" -gt 1 ]; then
    echo "check-authored-content: could not scan $file" >&2
    rm -f "${TMPDIR:-/tmp}/spec-spine-authored-dash.$$"
    exit 4
  fi
  rm -f "${TMPDIR:-/tmp}/spec-spine-authored-dash.$$"

  if LC_ALL=C grep -niI -F -- "${url_a}${url_b}" "$file" > "${TMPDIR:-/tmp}/spec-spine-authored-url.$$"; then
    echo "Codex session URLs are refused in $file:" >&2
    sed 's/^/  /' "${TMPDIR:-/tmp}/spec-spine-authored-url.$$" >&2
    status=1
  elif [ "$?" -gt 1 ]; then
    echo "check-authored-content: could not scan $file" >&2
    rm -f "${TMPDIR:-/tmp}/spec-spine-authored-url.$$"
    exit 4
  fi
  rm -f "${TMPDIR:-/tmp}/spec-spine-authored-url.$$"
done

if [ "$status" -eq 0 ]; then
  echo "check-authored-content: ${#files[@]} authored file(s) clean ($mode)"
fi
exit "$status"
