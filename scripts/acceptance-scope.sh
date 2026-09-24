#!/usr/bin/env bash
# Spec: specs/099-a-merged-acceptance-is-asked-again/spec.md
#
# acceptance-scope.sh: which specs' declared acceptance could a revision range
# have invalidated?
#
# Prints one spec id per line, sorted and deduplicated, for
# `verify-sweep.sh --only`. Two sources, both read through the governed verbs:
#
#   1. every spec `spec-spine index owner <path>` names for a path the range
#      changed, of any ownership kind;
#   2. every spec whose own `spec.md` the range changed, since a spec that
#      edits its own document is the most likely to have edited its own block.
#
# An empty answer is a true answer and exits 0: a range that changed only
# bypassed paths invalidates nothing, and the caller decides what to do with
# "nothing". The refusals (exit 3) are for a question that could not be asked:
# a range that does not resolve, a missing binary, a stale index.
#
# This script computes a SELECTION. It runs no acceptance and executes nothing
# the corpus declares; `verify-sweep.sh` does that, behind its own trust check.

set -u

readonly PROG="acceptance-scope.sh"

usage() {
  cat <<USAGE
usage: $PROG --base <rev> --head <rev> [options]

  --base <rev>     the revision the range starts at (exclusive)
  --head <rev>     the revision the range ends at (default: HEAD)
  --repo <dir>     repository to read (default: the current directory)
  --bin <path>     the spec-spine binary (default: \$SPEC_SPINE_BIN, else
                   target/release/spec-spine under the repository root)
  -h, --help       this

Exit: 0 a selection was computed (possibly empty)
      3 the question could not be asked (usage, bad revision, no binary,
        a verb that refused)
USAGE
}

die() { printf '%s: %s\n' "$PROG" "$*" >&2; exit 3; }

base=""; head_rev="HEAD"; repo="."; bin="${SPEC_SPINE_BIN:-}"

while [ $# -gt 0 ]; do
  case "$1" in
    --base) shift; [ $# -gt 0 ] || die "--base needs a value"; base="$1" ;;
    --head) shift; [ $# -gt 0 ] || die "--head needs a value"; head_rev="$1" ;;
    --repo) shift; [ $# -gt 0 ] || die "--repo needs a value"; repo="$1" ;;
    --bin)  shift; [ $# -gt 0 ] || die "--bin needs a value";  bin="$1" ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; die "unknown argument: $1" ;;
  esac
  shift
done

[ -n "$base" ] || { usage >&2; die "--base is required"; }

command -v git >/dev/null 2>&1 || die "git is not on PATH"
root=$(git -C "$repo" rev-parse --show-toplevel 2>/dev/null) || die "not a git repository: $repo"
root=$(cd "$root" && pwd -P)

[ -n "$bin" ] || bin="$root/target/release/spec-spine"
[ -x "$bin" ] || die "spec-spine binary not found or not executable: $bin"

git -C "$root" rev-parse --verify --quiet "$base^{commit}" >/dev/null \
  || die "base revision does not resolve: $base"
git -C "$root" rev-parse --verify --quiet "$head_rev^{commit}" >/dev/null \
  || die "head revision does not resolve: $head_rev"

# `--no-renames`: a rename is a change to both spellings, and the spec that owns
# either one has a stake in it.
changed=$(git -C "$root" diff --name-only --no-renames "$base" "$head_rev") \
  || die "could not diff $base..$head_rev"

specs_dir=$("$bin" config show --repo "$root" --json 2>/dev/null \
  | sed -n 's/.*"specs_dir"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)
[ -n "$specs_dir" ] || specs_dir="specs"

ids=""
while IFS= read -r path; do
  [ -n "$path" ] || continue
  case "$path" in
    "$specs_dir"/*/spec.md)
      id=${path#"$specs_dir"/}
      ids="$ids
${id%%/*}"
      continue
      ;;
  esac
  owners=$("$bin" index owner --repo "$root" --json "$path" 2>/dev/null) || {
    rc=$?
    # Spec 132: exit 1 is a stale ledger (a path with no owner is exit 0), and
    # 2, 3 and 4 a read that was refused or could not be made (before 0.26.0,
    # 2 was stale and 3 a failed read). Each is a question this script cannot
    # answer, and answering "nothing" would be a silent empty sweep.
    [ "$rc" -eq 1 ] && die "index is stale; the scope cannot be computed from it"
    die "'index owner $path' refused (exit $rc)"
  }
  ids="$ids
$(printf '%s' "$owners" | sed -n 's/.*"specId"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
done <<EOF
$changed
EOF

printf '%s\n' "$ids" | sed '/^$/d' | sort -u
