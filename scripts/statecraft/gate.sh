#!/bin/sh
# Rendered by Statecraft from profile github-actions-rust revision 13.
# The one definition of this repository's gate: `make gate` and `make code`
# run it locally, and CI runs the same script, so the two cannot drift. Only
# the repository-local .bin/spec-spine is used; a spec-spine elsewhere
# on PATH never answers for this repository.
#
# In CI this script is read at the base commit and run against the candidate's
# tree (revision 5, rule 1), so a candidate never judges itself with its own
# gate. BASE_SHA, when set, names that base: the declared authored-content
# script is read there too, and the commit walk judges every commit with this
# copy. A local run names no base and runs the working tree's copies.
set -eu

# The family exit contract (revision 7): 0 ok, 1 finding, 2 refused (a
# precondition the operator supplies was not met, and nothing was judged), 3
# usage, 4 failed (an operation that was attempted broke). Every deliberate
# non-zero exit goes through `leave`. Anything else that ends this script, a
# command `set -e` stopped on, is an operation that broke: the EXIT trap
# reports it as 4 whatever that command's own code was, so no child's code
# leaves this script untranslated.
leave() {
  trap - EXIT
  exit "$1"
}
trap 'rc=$?; if [ "$rc" -ne 0 ]; then echo "gate.sh: a command failed (exit $rc) and stopped the gate; reported as failed (4)" >&2; exit 4; fi' EXIT

SS=.bin/spec-spine
BASE_SHA="${BASE_SHA:-}"
case "$BASE_SHA" in 0000000000000000000000000000000000000000) BASE_SHA="" ;; esac
# This script's own path, for the commit walk, which judges each commit with
# the copy that is running (the base's in CI), never the commit's own.
SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"

# The project's governance parameters (revision 4, and FAIL_ON_UNRESOLVED from
# revision 9, spec 010), rendered from its setup block. Each default keeps a
# revision-3 project's behaviour except the base rule, which is a new refusal.
DEFAULT_BRANCH='main'
ENFORCE_COVERAGE=true
AUTHORED_CONTENT='scripts/check-authored-content.sh'
AUTHORED_CONTENT_TEXT=true
GATE_EACH_COMMIT=true
REQUIRE_SIGNED_COMMITS=true
REQUIRE_DEFAULT_BASE=true
FAIL_ON_UNRESOLVED=true
REQUIRE_RATIFIED=false

usage() {
  echo "usage: gate.sh governance|code|couple|couple-group|base|text|commits|pin" >&2
  leave 3
}

[ "$#" -eq 1 ] || usage
MODE=$1

# An input the mode is given through the environment; unset or empty is a
# usage error (3), never the shell's own code for an unset parameter.
need_env() {
  for name in "$@"; do
    eval "value=\${$name:-}"
    if [ -z "$value" ]; then
      echo "gate.sh $MODE needs $name" >&2
      leave 3
    fi
  done
}

# A missing spec-spine is a prerequisite the operator supplies, so it refuses
# (2): nothing was attempted, and spec 002 section 3.23 translates an absent
# binary to a refusal.
need_spec_spine() {
  if [ ! -x "$SS" ]; then
    echo "gate.sh: $SS is not installed; run scripts/statecraft/install-spec-spine.sh" >&2
    leave 2
  fi
}

# spec-spine's own exit codes, translated into the family contract. The
# table is chosen by the release the repository pins, so one rendering serves
# both sides of the 0.26.0 adoption. Before 0.26.0: 0 ok, 1 a validation
# failure or coupling drift, 2 stale, 3 an I/O, parse, schema, config or usage
# error, which includes a pin this binary does not satisfy. From 0.26.0
# (spec-spine's 132), the family contract itself: 0 ok, 1 finding (stale
# included), 2 refused (a pin not met, a containment refusal), 3 usage, 4
# failed. A usage error from the gate's own fixed invocation is the gate
# failing, so 3 reads as 4 under both tables.
# Decided once, the first time spec-spine answers, into SS_TABLE (026 or 025);
# POSIX sh has no local variables, so the working name is removed after use.
SS_TABLE=""
ss_family() {
  if [ -z "$SS_TABLE" ]; then
    SS_PIN_READ=$(pin_of spec-spine.toml)
    [ -n "$SS_PIN_READ" ] || SS_PIN_READ=$("$SS" --version 2>/dev/null | sed -n 's/^spec-spine \([0-9][0-9.]*\).*/\1/p')
    SS_TABLE=025
    case "$SS_PIN_READ" in
      0.[0-9].*|0.1[0-9].*|0.2[0-5].*|"") ;;
      *) SS_TABLE=026 ;;
    esac
    unset SS_PIN_READ
  fi
  [ "$SS_TABLE" = 026 ]
}
spec_spine() {
  ss_rc=0
  "$SS" "$@" || ss_rc=$?
  if ss_family; then
    case "$ss_rc" in
      0) return 0 ;;
      1) ss_to=1; ss_why="found the corpus does not pass, or a stale committed tree" ;;
      2) ss_to=2; ss_why="refused (a pin not met, or a containment refusal)" ;;
      3) ss_to=4; ss_why="refused the gate's own invocation as a usage error" ;;
      4) ss_to=4; ss_why="could not do its work (I/O, internal, schema)" ;;
      *) ss_to=4; ss_why="answered a code this gate does not know" ;;
    esac
  else
    case "$ss_rc" in
      0) return 0 ;;
      1) ss_to=1; ss_why="found the corpus does not pass" ;;
      2) ss_to=1; ss_why="found a stale committed tree" ;;
      3) ss_to=4; ss_why="did not perform the read (I/O, parse, schema, config or usage)" ;;
      *) ss_to=4; ss_why="answered a code this gate does not know" ;;
    esac
  fi
  echo "gate.sh: spec-spine $* $ss_why (spec-spine exit $ss_rc, gate exit $ss_to)" >&2
  leave "$ss_to"
}

# The declared authored-content script is the project's, and its codes are
# not this family's: any non-zero answer is a finding (1).
run_authored() {
  ac_rc=0
  "$AC" "$@" || ac_rc=$?
  if [ "$ac_rc" -ne 0 ]; then
    echo "gate.sh: $AUTHORED_CONTENT found a violation (its exit $ac_rc, gate exit 1)" >&2
    leave 1
  fi
}

# One cargo verb. A verb that ran and did not pass is a finding (1): cargo's
# own codes do not separate a failing test from a failing tool (101 is both),
# and the candidate is what it ran on.
cargo_verb() {
  c_rc=0
  cargo "$@" || c_rc=$?
  if [ "$c_rc" -ne 0 ]; then
    echo "gate.sh: cargo $1 did not pass (cargo exit $c_rc, gate exit 1)" >&2
    leave 1
  fi
}

# A declared authored-content script is required: deleting it, or dropping
# its executable bit, refuses (2) rather than passing silently.
need_authored_content() {
  if [ ! -f "$AUTHORED_CONTENT" ]; then
    echo "gate.sh: governance.authored_content names $AUTHORED_CONTENT, which is absent" >&2
    leave 2
  fi
  if [ ! -x "$AUTHORED_CONTENT" ]; then
    echo "gate.sh: governance.authored_content names $AUTHORED_CONTENT, which is not executable" >&2
    leave 2
  fi
}

# The declared authored-content script, as it exists at the base (revision 5,
# rule 1): the candidate's copy cannot weaken the check that judges it. Sets AC
# to the path to run. With no base named (a local run) the working tree's copy
# runs; a base that carries no such file is the adoption, where the
# candidate's copy runs, and it is said.
authored_content() {
  if [ -n "$BASE_SHA" ]; then
    if ! git cat-file -e "${BASE_SHA}^{commit}" 2>/dev/null; then
      echo "gate.sh: cannot read the base commit $BASE_SHA" >&2
      leave 2
    fi
    entry=$(git ls-tree "$BASE_SHA" -- "$AUTHORED_CONTENT")
    if [ -n "$entry" ]; then
      case "$entry" in
        100755\ *) ;;
        *)
          echo "gate.sh: governance.authored_content names $AUTHORED_CONTENT, which is not executable at the base $BASE_SHA" >&2
          leave 2
          ;;
      esac
      AC=$(mktemp "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/statecraft-authored-content.XXXXXX")
      git show "$BASE_SHA:$AUTHORED_CONTENT" > "$AC"
      chmod 755 "$AC"
      echo "gate.sh: $AUTHORED_CONTENT read at the base $BASE_SHA"
      return 0
    fi
    echo "gate.sh: the base carries no $AUTHORED_CONTENT; the candidate's copy runs (adoption)"
  fi
  need_authored_content
  AC="$(pwd)/$AUTHORED_CONTENT"
}

# The exact pin a spec-spine.toml states, read as install-spec-spine.sh reads
# it; empty when there is none.
pin_of() {
  awk '
    /^[[:space:]]*\[/ { section = $0; gsub(/[[:space:]]/, "", section); next }
    section == "[meta]" && /^[[:space:]]*required_version[[:space:]]*=/ { print; exit }
  ' "$1" 2>/dev/null | sed -n 's/^[^=]*=[[:space:]]*"=\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)"[[:space:]]*$/\1/p'
}

# Revision 12 (spec 024): ratification is a merge condition. Every path in
# the file $1 names is looked up with the pinned spec-spine at the
# candidate's tree (`index owner`, the coupling gate's own derivation), and a
# path a `draft` spec owns is a finding (1). The pull request that moves the
# spec to approved passes here; ci-gate requires the owner's exception for it.
ratified() {
  if [ "$REQUIRE_RATIFIED" != true ]; then
    echo "gate.sh: governance.require_ratified is false; a path a draft spec owns is not refused"
    return 0
  fi
  if ! command -v jq > /dev/null 2>&1; then
    echo "gate.sh: jq is not on PATH; the ratification check reads spec-spine's JSON with it" >&2
    leave 2
  fi
  rt="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
  spec_spine registry list --json > "$rt/statecraft-registry.json"
  jq -r '(.items // [])[] | select(.status == "draft") | .id' "$rt/statecraft-registry.json" > "$rt/statecraft-drafts"
  if [ ! -s "$rt/statecraft-drafts" ]; then
    echo "no spec is draft: every changed path's owner is ratified"
    return 0
  fi
  unratified=0
  while IFS= read -r path; do
    [ -n "$path" ] || continue
    spec_spine index owner "$path" --json > "$rt/statecraft-owner.json"
    for id in $(jq -r '(.owners // [])[].specId' "$rt/statecraft-owner.json" | sort -u); do
      if grep -qxF -- "$id" "$rt/statecraft-drafts"; then
        echo "gate.sh: $path is owned by $id, which is draft: the owner ratifies it (status: approved) in this pull request, with the owner exception, before this change merges" >&2
        unratified=1
      fi
    done
  done < "$1"
  if [ "$unratified" -ne 0 ]; then
    leave 1
  fi
  echo "every changed path's owning specs are ratified"
}

# The pull request a merge-queue entry was built from, by its group ref.
queue_pr() {
  printf '%s' "$1" | sed -n 's|.*/pr-\([0-9][0-9]*\)-[0-9a-f]*$|\1|p'
}

case "$MODE" in
  governance)
    need_spec_spine
    spec_spine check --fail-on-warn
    spec_spine lint --fail-on-warn
    if [ "$ENFORCE_COVERAGE" = true ]; then
      spec_spine index coverage --fail-on-untraced
    else
      # Reported, not enforced (governance.enforce_coverage is false): a new
      # project's own sources are unclaimed until it writes the specs that
      # claim them.
      spec_spine index coverage
    fi
    if [ "$FAIL_ON_UNRESOLVED" = true ]; then
      spec_spine index check --fail-on-unresolved
    else
      # Reported, not refused (governance.fail_on_unresolved is false, spec
      # 010): a corpus that approves a spec before building it has an
      # unresolved claim by design. `index check` still runs, so a stale or
      # invalid index still fails, and each unresolved claim is named.
      spec_spine index check
    fi
    if [ -n "$AUTHORED_CONTENT" ]; then
      authored_content
      run_authored
    else
      echo "gate.sh: no authored-content script is declared (governance.authored_content), so none runs"
    fi
    ;;
  code)
    # A workspace with no member crates yet judges nothing and says so, the
    # guard the hand-written CI this profile replaced had: every
    # `cargo --workspace` verb refuses a virtual manifest with no members.
    # `metadata --no-deps` resolves nothing, so it answers on such a manifest.
    # Whitespace is removed before matching, so the test does not depend on
    # how cargo's serializer spaces its JSON. No cargo refuses (2); a read
    # that fails stops the gate as failed (4).
    if ! command -v cargo > /dev/null 2>&1; then
      echo "gate.sh: cargo is not on PATH; install the toolchain rust-toolchain.toml names" >&2
      leave 2
    fi
    meta=$(cargo metadata --no-deps --format-version 1) || leave 4
    meta=$(printf '%s' "$meta" | tr -d ' \t\r\n')
    case "$meta" in
      *'"workspace_members":[]'*)
        echo "gate.sh: the workspace has no member crates yet; build, test, clippy and fmt judge nothing"
        exit 0
        ;;
    esac
    cargo_verb build --workspace --locked
    cargo_verb test --workspace --locked
    cargo_verb clippy --workspace --all-targets --locked -- -D warnings
    cargo_verb fmt --all --check
    ;;
  pin)
    # For the CI caches: the pin as a step output.
    version=$(pin_of spec-spine.toml)
    if [ -z "$version" ]; then
      echo "gate.sh: spec-spine.toml [meta] states no exact required_version (=X.Y.Z)" >&2
      leave 2
    fi
    echo "version=$version"
    ;;
  base)
    # A stacked pull request merges into another branch and is never judged
    # against the default branch (revision 4, rule 5).
    if [ "$REQUIRE_DEFAULT_BASE" != true ]; then
      echo "gate.sh: governance.require_default_base is false; the base is not judged"
      exit 0
    fi
    need_env BASE_REF
    if [ "$BASE_REF" != "$DEFAULT_BRANCH" ]; then
      echo "gate.sh: this pull request's base is '$BASE_REF', not the default branch $DEFAULT_BRANCH: open it off $DEFAULT_BRANCH, or merge the one below it first" >&2
      leave 1
    fi
    echo "the base is the default branch $DEFAULT_BRANCH"
    ;;
  text)
    # The pull request's title and body, which become the merge commit's
    # message: from the event on pull_request, through the API on
    # merge_group, whose event carries neither (revision 4, rule 3).
    if [ "$AUTHORED_CONTENT_TEXT" != true ]; then
      echo "gate.sh: governance.authored_content_text is false; the title and body are not judged"
      exit 0
    fi
    authored_content
    tmp="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
    text="$tmp/statecraft-pr-text.txt"
    case "${EVENT_NAME:-}" in
      pull_request)
        printf '%s\n%s\n' "${PR_TITLE:-}" "${PR_BODY:-}" > "$text"
        ;;
      merge_group)
        need_env GROUP_HEAD_REF REPO
        pr=$(queue_pr "$GROUP_HEAD_REF")
        if [ -z "$pr" ]; then
          echo "gate.sh: cannot read the pull request number from $GROUP_HEAD_REF" >&2
          leave 3
        fi
        gh api "repos/$REPO/pulls/$pr" --jq '.title, (.body // "")' > "$text"
        ;;
      *)
        echo "gate.sh: text judges a pull_request or merge_group event, not '${EVENT_NAME:-}'" >&2
        leave 3
        ;;
    esac
    run_authored --text "$text"
    ;;
  commits)
    # Every commit in the change's base..head, not only its head (revision 4,
    # rules 3 and 4): under merge commits each one lands on the default
    # branch. A step of the governance job, never a job of its own, so it
    # cannot be skipped into a green gate.
    if [ "$GATE_EACH_COMMIT" != true ] && [ "$REQUIRE_SIGNED_COMMITS" != true ] && [ "$AUTHORED_CONTENT_TEXT" != true ]; then
      echo "gate.sh: no per-commit check is enabled (governance.gate_each_commit, governance.require_signed_commits, governance.authored_content_text)"
      exit 0
    fi
    need_env BASE_SHA HEAD_SHA
    if [ "$REQUIRE_SIGNED_COMMITS" = true ]; then
      need_env REPO
    fi
    if [ "$AUTHORED_CONTENT_TEXT" = true ]; then
      authored_content
    fi
    tmp="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
    here=$(pwd)
    head_pin=$(pin_of spec-spine.toml)
    commits=$(git rev-list --reverse "$BASE_SHA..$HEAD_SHA")
    echo "judging $(printf '%s\n' "$commits" | grep -c . || true) commit(s) in $BASE_SHA..$HEAD_SHA"
    if [ "$GATE_EACH_COMMIT" = true ]; then
      echo "each commit's tree is judged by the running gate.sh ($SELF), never by the commit's own copy"
    fi
    fail=0
    for c in $commits; do
      short=$(git rev-parse --short "$c")
      if [ "$REQUIRE_SIGNED_COMMITS" = true ]; then
        # GitHub's verification, the one branch protection's signed-commits
        # rule reads.
        verified=$(gh api "repos/$REPO/commits/$c" --jq '.commit.verification.verified' 2>/dev/null) || verified=unreadable
        if [ "$verified" != true ]; then
          echo "gate.sh: $short is not signed with a key GitHub verifies ($verified)" >&2
          fail=1
        fi
      fi
      if [ "$AUTHORED_CONTENT_TEXT" = true ]; then
        msg="$tmp/statecraft-msg-$short.txt"
        git log -1 --format=%B "$c" > "$msg"
        if ! "$AC" --text "$msg"; then
          echo "gate.sh: $short's message breaks the authored-content rules" >&2
          fail=1
        fi
      fi
      if [ "$GATE_EACH_COMMIT" = true ]; then
        # The commit's own tree and the spec-spine release its own
        # spec-spine.toml pins, judged by the running gate.sh: the base's in
        # CI, never the commit's own (revision 5, rule 1).
        wt="$tmp/statecraft-commit-$short"
        git worktree add -q --detach "$wt" "$c"
        pin=$(pin_of "$wt/spec-spine.toml")
        bin=""
        log="$tmp/statecraft-gate-$short.log"
        : > "$log"
        if [ -z "$pin" ]; then
          # Written to the log too, which the refusal below prints: the gate
          # never ran for this commit, and the log must say why.
          echo "gate.sh: $short's spec-spine.toml states no exact pin, so no gate can judge it" | tee "$log" >&2
        elif [ "$pin" = "$head_pin" ] && [ -x "$SS" ]; then
          bin="$here/$SS"
        else
          root="$tmp/statecraft-spec-spine-$pin"
          [ -x "$root/bin/spec-spine" ] || cargo install spec-spine-cli --version "=$pin" --locked --root "$root"
          bin="$root/bin/spec-spine"
        fi
        script="$SELF"
        # The binary is copied into the commit's own tree, never linked
        # (revision 11, spec 023): a link from the worktree resolves outside
        # it, and spec-spine refuses to read a repository through a link that
        # leaves it (its spec 144, from 0.28.0). The copy is a regular file
        # inside the worktree, and it goes when the worktree does.
        contained="$wt/.bin/spec-spine"
        if [ -n "$bin" ] && mkdir -p "$wt/.bin" && rm -f "$contained" \
          && cp "$bin" "$contained" && chmod 755 "$contained" \
          && [ -f "$contained" ] && [ ! -L "$contained" ] \
          && (cd "$wt" && sh "$script" governance && cargo fmt --all --check) > "$log" 2>&1; then
          echo "$short: the gate and the format check pass at its own tree"
        else
          echo "gate.sh: $short fails the gate or the format check at its own tree" >&2
          cat "$log" 2>/dev/null || true
          fail=1
        fi
        git worktree remove --force "$wt"
      fi
    done
    if [ "$fail" -ne 0 ]; then
      echo "gate.sh: a commit in $BASE_SHA..$HEAD_SHA was refused; rebuild the branch" >&2
      leave 1
    fi
    echo "every commit in $BASE_SHA..$HEAD_SHA passes"
    ;;
  couple)
    # Pull requests only, with the event's two frozen endpoints: a three-dot
    # diff whose merge base is the pull request's own fork point.
    need_spec_spine
    need_env BASE_SHA HEAD_SHA
    body="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/statecraft-pr-body.txt"
    printf '%s' "${PR_BODY:-}" > "$body"
    spec_spine couple --base "$BASE_SHA" --head "$HEAD_SHA" --pr-body "$body"
    git -c core.quotePath=false diff --name-only "$BASE_SHA...$HEAD_SHA" > "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/statecraft-pr-changed"
    ratified "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/statecraft-pr-changed"
    ;;
  couple-group)
    # A merge-queue entry (revision 3): the group's own endpoints, which are
    # the entry's change on the speculative base it lands on. A waiver is read
    # from the entry's pull request and honoured only when the group changes
    # no path that pull request does not change; otherwise the group is
    # judged with no waiver, so a waiver never covers more than it was
    # granted for.
    need_spec_spine
    need_env BASE_SHA HEAD_SHA GROUP_HEAD_REF REPO
    tmp="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
    pr=$(queue_pr "$GROUP_HEAD_REF")
    if [ -z "$pr" ]; then
      echo "gate.sh: cannot read the pull request number from $GROUP_HEAD_REF" >&2
      leave 3
    fi
    # Each read is its own command, never the left side of a pipe: this shell
    # has no pipefail, and a read that failed into `sort` would be an empty
    # list the waiver rule then trusts (revision 7).
    git diff --name-only "$BASE_SHA" "$HEAD_SHA" > "$tmp/statecraft-group-paths.raw"
    sort -u "$tmp/statecraft-group-paths.raw" > "$tmp/statecraft-group-paths"
    gh api --paginate "repos/$REPO/pulls/$pr/files" \
      --jq '.[] | .filename, (.previous_filename // empty)' > "$tmp/statecraft-pr-paths.raw"
    sort -u "$tmp/statecraft-pr-paths.raw" > "$tmp/statecraft-pr-paths"
    body="$tmp/statecraft-pr-body.txt"
    extra=$(comm -23 "$tmp/statecraft-group-paths" "$tmp/statecraft-pr-paths")
    if [ -z "$extra" ]; then
      gh api "repos/$REPO/pulls/$pr" --jq '.body // ""' > "$body"
      echo "the group changes only #$pr's paths: its waiver, if any, applies"
    else
      : > "$body"
      echo "the group changes paths #$pr does not, so no waiver applies:"
      printf '%s\n' "$extra"
    fi
    spec_spine couple --base "$BASE_SHA" --head "$HEAD_SHA" --pr-body "$body"
    ratified "$tmp/statecraft-group-paths"
    ;;
  *)
    usage
    ;;
esac
