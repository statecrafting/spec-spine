#!/bin/sh
# Rendered by Statecraft from profile github-actions-rust revision 28.
# The action pin check (spec 044 section 3.4): every `uses:` in a workflow
# names code that cannot change under it. A step or job may use
#
#   ./<path>                                a local action or workflow;
#   docker://<image>@sha256:<64 hex>        an image by digest; or
#   <owner>/<repo>[/<path>]@<40 hex> # <ref>
#                                           an action by commit digest, with
#                                           the ref it was taken from as a
#                                           comment.
#
# A tag, a branch, a short digest or a missing comment is a finding naming
# the file and line. The check is offline: it never asks a host whether the
# comment matches the digest, which is left to review.
#
# usage: verify-action-pins.sh [<directory>]   (default .github/workflows)
#
# Exit 0 when every use is pinned; 1 with a finding per unpinned use; 2 when
# the directory is absent; 3 for bad usage.
#
# The pin rule is adapted from preloop-ci's scripts/verify-action-pins.py.
# That part: MIT License, Copyright (c) 2026 Preloop contributors. Permission
# is hereby granted, free of charge, to any person obtaining a copy of this
# software and associated documentation files (the "Software"), to deal in the
# Software without restriction, including without limitation the rights to
# use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
# of the Software, and to permit persons to whom the Software is furnished to
# do so, subject to the following conditions: The above copyright notice and
# this permission notice shall be included in all copies or substantial
# portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY
# OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
# OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN
# NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
# DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
# OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
# OR OTHER DEALINGS IN THE SOFTWARE.
set -eu

# The family exit contract (revision 7): 0 ok, 1 finding, 2 refused, 3 usage,
# 4 failed. A command `set -e` stopped on is reported as 4.
leave() {
  trap - EXIT
  exit "$1"
}
trap 'rc=$?; if [ "$rc" -ne 0 ]; then echo "verify-action-pins.sh: a command failed (exit $rc); reported as failed (4)" >&2; exit 4; fi' EXIT

if [ "$#" -gt 1 ]; then
  echo "usage: verify-action-pins.sh [<directory>]" >&2
  leave 3
fi
dir="${1:-.github/workflows}"
if [ ! -d "$dir" ]; then
  echo "verify-action-pins.sh: $dir is not a directory; nothing was checked" >&2
  leave 2
fi

files=$(find "$dir" -type f \( -name '*.yml' -o -name '*.yaml' \) | LC_ALL=C sort)
if [ -z "$files" ]; then
  echo "action pins: no workflow under $dir"
  leave 0
fi

# Each `uses:` line, in a step (`- uses:`) or a job (`uses:`), with its value
# unquoted and its trailing comment kept apart. awk exits 1 with a file's
# findings; every file is read before the verdict, so all of them are named.
rc=0
printf '%s\n' "$files" | {
found=0
while IFS= read -r f; do
  awk '
    /^[[:space:]]*(-[[:space:]]+)?uses:[[:space:]]/ {
      line = $0
      sub(/^[[:space:]]*(-[[:space:]]+)?uses:[[:space:]]+/, "", line)
      comment = ""
      c = index(line, "#")
      if (c > 0) {
        comment = substr(line, c + 1)
        line = substr(line, 1, c - 1)
      }
      sub(/[[:space:]]+$/, "", line)
      gsub(/^["\047]|["\047]$/, "", line)
      sub(/^[[:space:]]+/, "", comment)
      sub(/[[:space:]]+$/, "", comment)
      why = ""
      if (line ~ /^\.\//) {
        next
      } else if (line ~ /^docker:\/\//) {
        if (line !~ /@sha256:[0-9a-f]+$/ || length(substr(line, index(line, "@sha256:") + 8)) != 64)
          why = "a docker image must be named by @sha256:<64 hex>"
      } else if (line !~ /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+(\/[^@[:space:]]+)?@[^@[:space:]]+$/) {
        why = "not owner/repo[/path]@<ref>"
      } else {
        ref = substr(line, index(line, "@") + 1)
        if (ref !~ /^[0-9a-f]+$/ || length(ref) != 40)
          why = "pinned by `" ref "`, not by a 40-hex commit digest"
        else if (comment == "")
          why = "a digest pin needs a `# <ref>` comment naming what it was taken from"
      }
      if (why != "") {
        printf "%s:%d: %s: %s\n", FILENAME, FNR, line, why
        bad = 1
      }
    }
    END { exit (bad ? 1 : 0) }
  ' "$f" || {
    [ "$?" -eq 1 ] || exit 4
    found=1
  }
done
[ "$found" -eq 0 ] || exit 1
} || rc=$?
case "$rc" in
  0)
    echo "action pins: every use under $dir is pinned"
    leave 0
    ;;
  1) leave 1 ;;
  *)
    echo "verify-action-pins.sh: reading the workflows failed (exit $rc); reported as failed (4)" >&2
    leave 4
    ;;
esac
