#!/bin/sh
# Rendered by Statecraft from profile github-actions-rust revision 11.
# A managed file: `statecraft doctor` names an edit to it. Installs the exact
# spec-spine release this repository pins into .tooling/bin, and refuses a
# range or an absent pin rather than resolving one.
set -eu

# The family exit contract (revision 7): 0 ok, 2 refused (no exact pin, a
# precondition the operator supplies), 4 failed (the install or its version
# read broke). Every deliberate non-zero exit goes through `leave`; a command
# `set -e` stops on is reported as 4 by the EXIT trap, whatever its own code.
leave() {
  trap - EXIT
  exit "$1"
}
trap 'rc=$?; if [ "$rc" -ne 0 ]; then echo "install-spec-spine.sh: a command failed (exit $rc); reported as failed (4)" >&2; exit 4; fi' EXIT

pin=$(awk '
  /^[[:space:]]*\[/ { section = $0; gsub(/[[:space:]]/, "", section); next }
  section == "[meta]" && /^[[:space:]]*required_version[[:space:]]*=/ { print; exit }
' spec-spine.toml 2>/dev/null | sed 's/^[^=]*=[[:space:]]*"\(.*\)"[[:space:]]*$/\1/')

case "$pin" in
  =[0-9]*.[0-9]*.[0-9]*) version="${pin#=}" ;;
  "")
    echo "install-spec-spine.sh: spec-spine.toml [meta] carries no required_version; this profile requires an exact pin (=X.Y.Z)" >&2
    leave 2 ;;
  *)
    echo "install-spec-spine.sh: required_version \"$pin\" is not an exact pin (=X.Y.Z); this profile refuses a range" >&2
    leave 2 ;;
esac

bin=.tooling/bin/spec-spine
if [ -x "$bin" ] && [ "$("$bin" --version 2>/dev/null)" = "spec-spine $version" ]; then
  echo "spec-spine $version is already installed at $bin"
  exit 0
fi
if ! command -v cargo > /dev/null 2>&1; then
  echo "install-spec-spine.sh: cargo is not on PATH, so spec-spine $version cannot be installed" >&2
  leave 2
fi
if ! cargo install spec-spine-cli --version "=$version" --locked --root .tooling; then
  echo "install-spec-spine.sh: cargo install of spec-spine $version failed" >&2
  leave 4
fi
if ! "$bin" --version; then
  echo "install-spec-spine.sh: the installed $bin does not answer --version" >&2
  leave 4
fi
