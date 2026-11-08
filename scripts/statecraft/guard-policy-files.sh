#!/bin/sh
# Rendered by Statecraft from profile github-actions-rust revision 28.
# The policy-isolation guard (spec 044 section 3.3): a pull request that
# changes a dependency or supply-chain policy file and anything else must be
# split, so a dependency change is never reviewed inside a feature.
#
# usage: guard-policy-files.sh <base> <head>
#
# The changed paths are those between the merge base of the two revisions and
# the head, never between the two tips, so a change present on both sides
# cannot cancel out. Each path falls in exactly one class:
#
#   policy     deny.toml, .cargo/audit.toml, anything under supply-chain/, any
#              Cargo.toml or Cargo.lock at any depth, spec-spine.toml and
#              docs/adoption/spec-spine.md;
#   generated  anything under .statecraft/derived/, and
#              .statecraft/environment.json, which their own freshness checks
#              guard; the guard ignores them;
#   version    (revision 27, spec 071) a Cargo.toml or Cargo.lock present at
#              both revisions whose only change is the version of a workspace
#              package: Cargo.toml's [package] or [workspace.package] version,
#              or the version in an inline table that also names a path;
#              Cargo.lock's version of a [[package]] with no source. A release
#              bump moves no dependency, so the guard ignores it too.
#              (revision 28, spec 072) Also spec-spine.toml, present at both
#              revisions, when its only change is required_version and the
#              head pins the version the head's root Cargo.toml declares for
#              the workspace: a repository that pins itself;
#   spec       (revision 28, spec 072) a spec.md whose front matter names,
#              quoted, a policy path this change also changes: the spec that
#              owns that file, which the coupling gate requires beside it.
#              Ignored, so a policy change and its owning spec are one pull
#              request;
#   other      every remaining path.
#
# Exit 0 when policy and other paths did not both change; 1 when they did,
# listing each class's paths; 2 when the guard cannot decide (bad usage, a
# revision that does not resolve, no merge base). It never fetches, so it
# cannot hang or half-decide.
#
# The exit and merge-base rule are adapted from preloop-ci's
# supply-chain/guard-policy-files.sh. That part: MIT License, Copyright (c)
# 2026 Preloop contributors. Permission is hereby granted, free of charge, to
# any person obtaining a copy of this software and associated documentation
# files (the "Software"), to deal in the Software without restriction,
# including without limitation the rights to use, copy, modify, merge,
# publish, distribute, sublicense, and/or sell copies of the Software, and to
# permit persons to whom the Software is furnished to do so, subject to the
# following conditions: The above copyright notice and this permission notice
# shall be included in all copies or substantial portions of the Software.
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
set -eu

# The family exit contract (revision 7), with this guard's own table: 2 is
# also bad usage here (spec 044 section 3.3), because every way the guard
# cannot decide is one refusal. A command `set -e` stopped on is an operation
# that broke, reported as 4; never 0.
leave() {
  trap - EXIT
  exit "$1"
}
trap 'rc=$?; if [ "$rc" -ne 0 ]; then echo "guard-policy-files.sh: a command failed (exit $rc); reported as failed (4)" >&2; exit 4; fi' EXIT

undecided() {
  echo "guard-policy-files.sh: $1; the guard cannot decide, and fails closed" >&2
  leave 2
}

[ "$#" -eq 2 ] || undecided "usage: guard-policy-files.sh <base> <head>"
for rev in "$1" "$2"; do
  case "$rev" in
    "" | -*) undecided "usage: guard-policy-files.sh <base> <head>, each a revision (got '$rev')" ;;
  esac
done
git rev-parse --is-inside-work-tree > /dev/null 2>&1 || undecided "not inside a git work tree"
base=$(git rev-parse --verify --quiet "$1^{commit}") || undecided "the base revision $1 does not resolve (was it fetched?)"
head=$(git rev-parse --verify --quiet "$2^{commit}") || undecided "the head revision $2 does not resolve (was it fetched?)"
merge_base=$(git merge-base "$base" "$head") || undecided "$1 and $2 have no merge base"

# No renames: a renamed path counts as both its old and its new path, so a
# move into or out of a policy path is seen on both sides.
changed=$(git -c core.quotePath=true diff --no-renames --name-only "$merge_base" "$head") \
  || undecided "cannot list the paths changed between $merge_base and $head"

# Revision 27 (spec 071): each Cargo.toml or Cargo.lock is version-only when
# both revisions hold it and they are equal once every workspace package's
# version is masked. Anything else in the file, a dependency's version
# included, keeps it policy. A path git had to quote stays policy.
mask_versions() {
  case "$1" in
    */Cargo.lock | Cargo.lock)
      awk '
        function flush(  i, l) {
          for (i = 0; i < n; i++) {
            l = buf[i]
            if (pkg && !src && l ~ /^version[ \t]*=[ \t]*"[^"]*"[ \t]*$/) sub(/"[^"]*"/, "\"*\"", l)
            print l
          }
          n = 0; src = 0; pkg = 0
        }
        /^\[/ { flush(); pkg = ($0 == "[[package]]") }
        /^source[ \t]*=/ { src = 1 }
        { buf[n++] = $0 }
        END { flush() }
      ' ;;
    *)
      awk '
        {
          l = $0
          if (l ~ /^[ \t]*\[/) {
            t = l; gsub(/[ \t]/, "", t); sub(/#.*/, "", t); table = t
          } else if ((table == "[package]" || table == "[workspace.package]") && l ~ /^[ \t]*version[ \t]*=[ \t]*"[^"]*"[ \t]*$/) {
            sub(/"[^"]*"/, "\"*\"", l)
          } else if (l ~ /\{/ && l ~ /[{, \t]path[ \t]*=/ && l ~ /[{, \t]version[ \t]*=[ \t]*"/) {
            gsub(/version[ \t]*=[ \t]*"[^"]*"/, "version = \"*\"", l)
          }
          print l
        }
      ' ;;
  esac
}
version_only=$(printf '%s\n' "$changed" | while IFS= read -r p; do
  case "$p" in
    (\"*) continue ;;
    (Cargo.toml | */Cargo.toml | Cargo.lock | */Cargo.lock) ;;
    (*) continue ;;
  esac
  git cat-file -e "$merge_base:$p" 2> /dev/null || continue
  git cat-file -e "$head:$p" 2> /dev/null || continue
  was=$(git show "$merge_base:$p" | mask_versions "$p")
  now=$(git show "$head:$p" | mask_versions "$p")
  if [ "$was" = "$now" ]; then printf '%s\n' "$p"; fi
done)

# Revision 28 (spec 072): a repository that pins itself. spec-spine.toml is
# version-only when both revisions hold it, it is unchanged once its
# required_version line is masked, and the head pins exactly the version the
# head's root Cargo.toml declares in [workspace.package] or [package]. A
# consumer's pin moves to another tool's version, so it stays policy.
workspace_version() {
  git show "$head:Cargo.toml" 2> /dev/null | awk '
    /^[ \t]*\[/ { t = $0; gsub(/[ \t]/, "", t); sub(/#.*/, "", t); table = t; next }
    (table == "[workspace.package]" || table == "[package]") && /^[ \t]*version[ \t]*=[ \t]*"[^"]*"[ \t]*$/ {
      v = $0; sub(/^[^"]*"/, "", v); sub(/".*$/, "", v); print v; exit
    }
  '
}
mask_pin() {
  awk '{ l = $0; if (l ~ /^[ \t]*required_version[ \t]*=[ \t]*"[^"]*"[ \t]*$/) sub(/"[^"]*"/, "\"*\"", l); print l }'
}
pinned_version() {
  awk '/^[ \t]*required_version[ \t]*=[ \t]*"[^"]*"[ \t]*$/ { v = $0; sub(/^[^"]*"/, "", v); sub(/".*$/, "", v); sub(/^=/, "", v); print v; exit }'
}
if printf '%s\n' "$changed" | grep -qx 'spec-spine.toml' \
  && git cat-file -e "$merge_base:spec-spine.toml" 2> /dev/null \
  && git cat-file -e "$head:spec-spine.toml" 2> /dev/null; then
  was=$(git show "$merge_base:spec-spine.toml" | mask_pin)
  now=$(git show "$head:spec-spine.toml" | mask_pin)
  own=$(workspace_version)
  pin=$(git show "$head:spec-spine.toml" | pinned_version)
  if [ "$was" = "$now" ] && [ -n "$own" ] && [ "$pin" = "$own" ]; then
    version_only=$(printf '%s\nspec-spine.toml' "$version_only")
  fi
fi

# Revision 28 (spec 072): the owning spec. A changed spec.md is ignored when
# its front matter (the lines between its opening --- and the next) names,
# in double or single quotes, a policy path this change changes and does not
# ignore as version-only. The match is exact text; a claim written any other
# way leaves the spec.md other, which fails closed into the split.
policy_paths=$(printf '%s\n' "$changed" | VERSION_ONLY="$version_only" awk '
  BEGIN { k = split(ENVIRON["VERSION_ONLY"], v, "\n"); for (i = 1; i <= k; i++) if (v[i] != "") vo[v[i]] = 1 }
  NF == 0 || /^"/ || ($0 in vo) { next }
  { n = split($0, part, "/"); name = part[n] }
  $0 == "deny.toml" || $0 == ".cargo/audit.toml" || /^supply-chain\// || name == "Cargo.toml" ||
    name == "Cargo.lock" || $0 == "spec-spine.toml" || $0 == "docs/adoption/spec-spine.md" { print }
')
owning_specs=$(printf '%s\n' "$changed" | while IFS= read -r p; do
  [ -n "$policy_paths" ] || break
  case "$p" in
    (\"*) continue ;;
    (spec.md | */spec.md) ;;
    (*) continue ;;
  esac
  git cat-file -e "$head:$p" 2> /dev/null || continue
  if git show "$head:$p" | POLICY="$policy_paths" awk '
      BEGIN { k = split(ENVIRON["POLICY"], q, "\n") }
      NR == 1 { if ($0 != "---") exit; next }
      $0 == "---" { closed = 1; exit }
      { for (i = 1; i <= k; i++) if (q[i] != "" && (index($0, "\"" q[i] "\"") || index($0, "\047" q[i] "\047"))) found = 1 }
      END { exit (closed && found) ? 0 : 1 }
    '; then
    printf '%s\n' "$p"
  fi
done)

# One pass over the paths, so the guard is linear in their number. A path git had to quote is unquoted only enough to
# classify it: the class depends on its directory prefix and its final name.
rc=0
printf '%s\n' "$changed" | VERSION_ONLY="$version_only" OWNING="$owning_specs" awk '
  BEGIN {
    k = split(ENVIRON["VERSION_ONLY"], v, "\n"); for (i = 1; i <= k; i++) if (v[i] != "") vo[v[i]] = 1
    k = split(ENVIRON["OWNING"], w, "\n"); for (i = 1; i <= k; i++) if (w[i] != "") os[w[i]] = 1
  }
  NF == 0 { next }
  {
    p = $0
    if (p ~ /^".*"$/) p = substr(p, 2, length(p) - 2)
    n = split(p, part, "/")
    name = part[n]
    if (p ~ /^\.statecraft\/derived\// || p == ".statecraft/environment.json") {
      class = "generated"
    } else if ($0 in vo) {
      class = "version"
    } else if ($0 in os) {
      class = "spec"
    } else if (p == "deny.toml" || p == ".cargo/audit.toml" || p ~ /^supply-chain\// ||
               name == "Cargo.toml" || name == "Cargo.lock" ||
               p == "spec-spine.toml" || p == "docs/adoption/spec-spine.md") {
      class = "policy"
    } else {
      class = "other"
    }
    count[class]++
    # Listed exactly as git printed it, quoted where git quoted it, so a
    # name with a newline or a control character cannot forge a line here.
    list[class] = list[class] "  " $0 "\n"
  }
  END {
    printf "policy-isolation: %d policy, %d generated (ignored), %d other, %d version-only (ignored), %d owning spec (ignored)\n", count["policy"], count["generated"], count["other"], count["version"], count["spec"]
    if (count["policy"] > 0 && count["other"] > 0) {
      printf "this pull request changes policy paths and other paths together; split it, so the dependency or policy change is its own pull request\n"
      printf "policy:\n%s", list["policy"]
      printf "other:\n%s", list["other"]
      mixed = 1
    }
    exit (mixed ? 1 : 0)
  }
' || rc=$?
case "$rc" in
  0) leave 0 ;;
  1) leave 1 ;;
  *) echo "guard-policy-files.sh: classifying the paths failed (exit $rc); reported as failed (4)" >&2; leave 4 ;;
esac
