#!/bin/sh
# Rendered by Statecraft from profile github-actions-rust revision 29.
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
#   crate      (revision 29, spec 073) a Cargo.toml in a directory the merge
#              base holds nothing in, whose every table is one a crate needs
#              and whose every dependency inherits from the workspace or is a
#              path to a workspace crate; and a Cargo.lock whose only change,
#              after revision 27's mask, is such a crate's sourceless entry
#              and the dependency lists that name it. A new crate that adds
#              no package is one pull request; any registry or git package
#              change keeps the file policy;
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
tmp=""
leave() {
  [ -z "$tmp" ] || rm -rf "$tmp"
  trap - EXIT
  exit "$1"
}
trap 'rc=$?; [ -z "$tmp" ] || rm -rf "$tmp"; if [ "$rc" -ne 0 ]; then echo "guard-policy-files.sh: a command failed (exit $rc); reported as failed (4)" >&2; exit 4; fi' EXIT

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

# Revision 29 (spec 073): a new workspace crate that adds no dependency. A
# Cargo.toml is new-crate when the merge base holds nothing in its directory,
# its every table is one a crate needs, and its every dependency line inherits
# from the workspace or is a path to a workspace crate (section 3.1). A
# Cargo.lock is new-crate when, after revision 27's mask, its header and every
# block with a source are unchanged, no sourceless block is removed, each
# existing sourceless block differs at most in its dependencies list, and each
# added block is sourceless and named by a new-crate manifest (section 3.2).
# Anything else keeps the file policy; the comparison is textual and never
# runs cargo.
check_manifest() {
  # Prints "name <package>" and one "path <directory>" per path dependency;
  # exits 1 when a table, a key or a dependency line is not admitted.
  DIR="$1" awk '
    function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
    function bad() { rejected = 1; exit 1 }
    function deptable(t) {
      if (t == "[dependencies]" || t == "[dev-dependencies]" || t == "[build-dependencies]") return 1
      return t ~ /^\[target\..+\.(dependencies|dev-dependencies|build-dependencies)\]$/
    }
    function resolve(rel,  n, i, seg, out, m, k) {
      if (rel == "" || rel ~ /^\// || rel ~ /\\/) bad()
      m = 0
      n = split(ENVIRON["DIR"], seg, "/")
      for (i = 1; i <= n; i++) if (seg[i] != "" && seg[i] != ".") out[++m] = seg[i]
      n = split(rel, seg, "/")
      for (i = 1; i <= n; i++) {
        if (seg[i] == "" || seg[i] == ".") continue
        if (seg[i] == "..") { if (m == 0) bad(); m--; continue }
        out[++m] = seg[i]
      }
      if (m == 0) bad()
      k = out[1]
      for (i = 2; i <= m; i++) k = k "/" out[i]
      return k
    }
    function dep(l,  name, inner, n, i, kv, key, val, keys, path) {
      if (l ~ /^[A-Za-z0-9_-]+[ \t]*\.[ \t]*workspace[ \t]*=[ \t]*true$/) return
      if (l !~ /^[A-Za-z0-9_-]+[ \t]*=[ \t]*\{.*\}$/) bad()
      inner = l; sub(/^[^{]*\{/, "", inner); sub(/\}$/, "", inner)
      if (inner ~ /[{}]/) bad()
      # A features array is the one value that may hold a comma.
      while (match(inner, /\[[^][]*\]/)) {
        if (substr(inner, RSTART + 1, RLENGTH - 2) !~ /^[ \t]*("[^"]*"[ \t]*,[ \t]*)*("[^"]*"[ \t]*,?[ \t]*)?$/) bad()
        inner = substr(inner, 1, RSTART - 1) "@array" substr(inner, RSTART + RLENGTH)
      }
      if (inner ~ /[][]/) bad()
      n = split(inner, kv, ",")
      for (key in keys) delete keys[key]
      for (i = 1; i <= n; i++) {
        if (trim(kv[i]) == "" && i == n) continue
        if (index(kv[i], "=") == 0) bad()
        key = trim(substr(kv[i], 1, index(kv[i], "=") - 1))
        val = trim(substr(kv[i], index(kv[i], "=") + 1))
        if (key in keys) bad()
        if (key == "workspace") { if (val != "true") bad() }
        else if (key == "features") { if (val != "@array") bad() }
        else if (key == "optional" || key == "default-features") { if (val != "true" && val != "false") bad() }
        else if (key == "path" || key == "version" || key == "package") { if (val !~ /^"[^"]*"$/) bad() }
        else bad()
        keys[key] = val
      }
      if ("workspace" in keys) {
        if (("path" in keys) || ("version" in keys) || ("package" in keys)) bad()
        return
      }
      if (!("path" in keys)) bad()
      path = keys["path"]; path = substr(path, 2, length(path) - 2)
      paths[++np] = resolve(path)
    }
    {
      l = trim($0)
      if (l == "" || l ~ /^#/) next
      if (l ~ /^\[/) {
        t = l; sub(/#.*/, "", t); gsub(/[ \t]/, "", t)
        if (t == "[package]" || t == "[package.metadata]" || t ~ /^\[package\.metadata\.[^][]+\]$/ ||
            t == "[lib]" || t == "[[bin]]" || t == "[[test]]" || t == "[[bench]]" || t == "[[example]]" ||
            t == "[features]" || t == "[lints]" || t == "[badges]" || deptable(t)) { table = t; next }
        bad()
      }
      if (table == "") bad()
      if (deptable(table)) { dep(l); next }
      if (table == "[package]") {
        if (l ~ /^links[ \t]*[.=]/) bad()
        if (l ~ /^name[ \t]*=[ \t]*"[^"]*"$/ && pkg == "") { pkg = l; sub(/^[^"]*"/, "", pkg); sub(/"$/, "", pkg) }
      }
    }
    END {
      if (rejected) exit 1
      if (pkg == "") exit 1
      print "name " pkg
      for (i = 1; i <= np; i++) print "path " paths[i]
    }
  '
}
lock_blocks() {
  # The masked lockfile as records: one "header" record, then one per
  # [[package]] block, each prefixed by its kind (sourced or local) and name.
  # A dependencies list is moved to a record of its own, so a local block
  # can be compared without it.
  mask_versions "$1" | awk '
    function esc(s) { gsub(/\\/, "\\\\", s); gsub(/\t/, "\\t", s); return s "\\n" }
    function flush() {
      if (!inpkg) return
      # A sourced block keeps its dependencies; a local one is compared
      # without them.
      printf "%s\t%s\t%s\n", (src ? "sourced" : "local"), name, (src ? body deplist : body)
      inpkg = 0; body = ""; deplist = ""; name = ""; src = 0
    }
    BEGIN { header = "" }
    /^\[\[package\]\][ \t]*$/ { flush(); inpkg = 1; next }
    /^\[/ { flush(); other = 1 }
    {
      if (!inpkg) { header = header esc($0); next }
      if (deps) { if ($0 ~ /^\][ \t]*$/) deps = 0; deplist = deplist esc($0); next }
      if ($0 ~ /^dependencies[ \t]*=[ \t]*\[/) { if ($0 !~ /\][ \t]*$/) deps = 1; deplist = deplist esc($0); next }
      if ($0 ~ /^[ \t]*$/) next
      if ($0 ~ /^source[ \t]*=/) src = 1
      if ($0 ~ /^name[ \t]*=[ \t]*"[^"]*"[ \t]*$/) { name = $0; sub(/^[^"]*"/, "", name); sub(/".*$/, "", name) }
      body = body esc($0)
    }
    END { flush(); printf "header\t\t%s\n", header }
  '
}
new_crate=""
new_names=""
candidates=$(printf '%s\n' "$changed" | while IFS= read -r p; do
  case "$p" in
    (\"* | Cargo.toml) continue ;;
    (*/Cargo.toml) ;;
    (*) continue ;;
  esac
  git cat-file -e "$merge_base:$p" 2> /dev/null && continue
  git cat-file -e "$head:$p" 2> /dev/null || continue
  d=${p%/Cargo.toml}
  [ -z "$(git ls-tree -r --name-only "$merge_base" -- "$d/")" ] || continue
  printf '%s\n' "$p"
done)
if [ -n "$candidates" ]; then
  while IFS= read -r p; do
    out=$(git show "$head:$p" | check_manifest "${p%/Cargo.toml}") || continue
    ok=1
    deps=$(printf '%s\n' "$out" | sed -n 's/^path //p')
    while IFS= read -r d; do
      [ -n "$d" ] || continue
      t="$d/Cargo.toml"
      if printf '%s\n' "$candidates" | grep -qxF "$t"; then continue; fi
      was=$(git rev-parse --verify --quiet "$merge_base:$t") || { ok=0; break; }
      now=$(git rev-parse --verify --quiet "$head:$t") || { ok=0; break; }
      [ "$was" = "$now" ] || { ok=0; break; }
    done <<DEPS
$deps
DEPS
    [ "$ok" = 1 ] || continue
    new_crate=$(printf '%s\n%s' "$new_crate" "$p")
    new_names=$(printf '%s\n%s' "$new_names" "$(printf '%s\n' "$out" | sed -n 's/^name //p')")
  done <<CANDIDATES
$candidates
CANDIDATES
fi
locks=$(printf '%s\n' "$changed" | VERSION_ONLY="$version_only" awk '
  BEGIN { k = split(ENVIRON["VERSION_ONLY"], v, "\n"); for (i = 1; i <= k; i++) if (v[i] != "") vo[v[i]] = 1 }
  /^"/ || ($0 in vo) { next }
  $0 == "Cargo.lock" || /\/Cargo\.lock$/ { print }
')
while IFS= read -r p; do
  [ -n "$p" ] || continue
  git cat-file -e "$merge_base:$p" 2> /dev/null || continue
  git cat-file -e "$head:$p" 2> /dev/null || continue
  [ -n "$tmp" ] || tmp=$(mktemp -d)
  git show "$merge_base:$p" | lock_blocks "$p" > "$tmp/was"
  git show "$head:$p" | lock_blocks "$p" > "$tmp/now"
  if NAMES="$new_names" awk -F '\t' '
      BEGIN { k = split(ENVIRON["NAMES"], v, "\n"); for (i = 1; i <= k; i++) if (v[i] != "") fresh[v[i]] = 1 }
      FNR == NR {
        if ($1 == "header") wh = $3
        else if ($1 == "sourced") ws[$3]++
        else { if ($2 in wl) dup = 1; wl[$2] = $3 }
        next
      }
      {
        if ($1 == "header") nh = $3
        else if ($1 == "sourced") ns[$3]++
        else { if ($2 in nl) dup = 1; nl[$2] = $3 }
      }
      END {
        if (dup || wh != nh) exit 1
        for (b in ws) if (ns[b] != ws[b]) exit 1
        for (b in ns) if (ns[b] != ws[b]) exit 1
        for (n in wl) if (!(n in nl) || nl[n] != wl[n]) exit 1
        for (n in nl) if (!(n in wl) && !(n in fresh)) exit 1
        exit 0
      }
    ' "$tmp/was" "$tmp/now"; then
    new_crate=$(printf '%s\n%s' "$new_crate" "$p")
  fi
done <<LOCKS
$locks
LOCKS

# Revision 28 (spec 072): the owning spec. A changed spec.md is ignored when
# its front matter (the lines between its opening --- and the next) names,
# in double or single quotes, a policy path this change changes and does not
# ignore as version-only or, since revision 29, as new-crate. The match is exact text; a claim written any other
# way leaves the spec.md other, which fails closed into the split.
policy_paths=$(printf '%s\n' "$changed" | VERSION_ONLY="$version_only" NEW_CRATE="$new_crate" awk '
  BEGIN {
    k = split(ENVIRON["VERSION_ONLY"], v, "\n"); for (i = 1; i <= k; i++) if (v[i] != "") vo[v[i]] = 1
    k = split(ENVIRON["NEW_CRATE"], v, "\n"); for (i = 1; i <= k; i++) if (v[i] != "") vo[v[i]] = 1
  }
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
printf '%s\n' "$changed" | VERSION_ONLY="$version_only" OWNING="$owning_specs" NEW_CRATE="$new_crate" awk '
  BEGIN {
    k = split(ENVIRON["NEW_CRATE"], c, "\n"); for (i = 1; i <= k; i++) if (c[i] != "") nc[c[i]] = 1
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
    } else if ($0 in nc) {
      class = "crate"
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
    printf "policy-isolation: %d policy, %d generated (ignored), %d other, %d version-only (ignored), %d owning spec (ignored), %d new crate (ignored)\n", count["policy"], count["generated"], count["other"], count["version"], count["spec"], count["crate"]
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
