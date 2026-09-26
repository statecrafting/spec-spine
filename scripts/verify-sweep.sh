#!/usr/bin/env bash
# Spec: specs/089-nothing-reruns-a-merged-acceptance/spec.md
#
# verify-sweep.sh: run the whole corpus's declared acceptance against one
# trusted, already-merged revision, in an isolated checkout, and account for
# every spec.
#
# The gap this closes is spec 083 §1.3: `verify` is the one verb that executes
# what the corpus declares, so it sits outside the gate chain deliberately and
# nothing in CI runs it. A `## Verification` block can therefore be red for
# months while every gate stays green, which is exactly what specs 084-110 each
# had to repair one spec at a time. Spec 083 §4 named the remedy ("a periodic
# sweep a maintainer runs, not a gate step") and left naming its home as its
# own work. This is that home.
#
#   WHO RUNS IT   a maintainer by hand, and since spec 099 the acceptance
#                 workflow: on the default branch after a merge, and nightly.
#                 Never a pull request, never a driven session's gate, never a
#                 hook. It executes what the corpus declares, so it runs only
#                 against a revision already merged into the trusted ref, and
#                 spec 099 3.1 keeps it off every event that could carry one
#                 that is not.
#   WHEN          before cutting a release, and after merging any spec that
#                 carries `amends` or `amends_verification` (the crossing that
#                 silently staled the five blocks 106-110 repaired).
#   AGAINST WHAT  a revision that is already an ancestor of the trusted ref.
#                 A PR branch is a stranger's code; this script refuses one.
#
# Every selected spec gets exactly one of five outcomes, and only two of them
# are success:
#
#   passed         declared acceptance, ran, every command exited 0
#   failed         declared acceptance, ran, a command exited non-zero
#   not-declared   no verify:cli commands, and not on the legacy ledger
#   exempt         on the closed legacy ledger below
#   not-run        no verdict: the verb could not answer, or the run timed out
#
# `not-declared` and `not-run` are NOT success. Missing acceptance never counts
# as passing; that is the whole point of accounting for every spec rather than
# reporting a pass rate over the ones that happened to declare something.
#
# Exit: 0 every selected spec is passed or exempt
#       1 any spec is failed, not-declared or not-run
#       3 the sweep refused to run (usage, untrusted revision, bad ledger, I/O)
#
# THE RELEASE VERDICT (spec 119). Every row also carries the spec's lifecycle,
# read through `registry list --json`, and a second outcome judged only over
# what the corpus says is built. `implementation: complete` or `n-a`, or an
# absent key on a spec that is not a draft (spec 042), is an implemented
# obligation, and its release outcome is its outcome above, whatever its
# `status`: an implemented draft that fails still fails. `pending`,
# `in-progress`, `deferred`, or an absent key on a draft, is `pending`: its
# block still runs and its outcome above is still reported, but it is neither
# success nor failure of the release, and it is never `exempt`. A lifecycle the
# sweep cannot read, and a plan it cannot read, are `not-run`.
#
# Both verdicts are always in the report. `--release` makes the exit code the
# release verdict's (0 when every implemented spec is passed or exempt, 1
# otherwise); without it the exit code is the corpus verdict above, unchanged.
#
# Reads the corpus only through `spec-spine` (registry list, verify), per
# AGENTS.md "Governed artifact reads".

set -u

readonly PROG="verify-sweep.sh"

# --- the closed legacy ledger ---------------------------------------------
#
# Specs 000 through 047 were filed before `verify:cli` existed: spec 043 built
# the verb, and spec 093, filed immediately before it, is the first spec in this
# corpus to carry a block. Those 48 specs are tracked debt, not a standing
# exception, and this ledger is how the debt is tracked.
#
# The ledger is CLOSED AT ORDINAL 043 (`LEDGER_CLOSED_AT` below) and the sweep
# refuses an entry at or above it. That is what keeps an exemption from silently
# covering a future spec: there is no predicate here that a spec filed tomorrow
# could satisfy. A post-048 spec with no acceptance is `not-declared`, the sweep
# exits 1, and the answer is to write the block.
#
# RETIREMENT POLICY. The ledger only shrinks. When a spec below gains a
# `## Verification` block (its own, or one carried for it under
# `amends_verification`), its line here is deleted in the same change. The sweep
# enforces that half mechanically: an entry whose spec declares acceptance is a
# stale exemption and the sweep refuses (exit 3) until the line is removed. So
# the debt cannot be paid and then silently re-incurred, and an entry cannot
# hide a block that exists.
#
# One id per line, `#` comments and blank lines ignored. `--exempt-file` reads
# the same grammar from a file instead.
readonly LEDGER_CLOSED_AT=43
legacy_ledger() {
  cat <<'LEDGER'
# Filed before spec 043 built `verify`; no acceptance was declarable. Five of
# the original 48 were removed by spec 095's collapse and are gone from here.
# Specs 136 to 139 carried acceptance for 39 more under amends_verification
# and deleted their lines. Spec 149 gave 006 an offline acceptance and deleted
# its line. The three below remain, each with its reason.
# exempt: tier-1 constitution, whose claims the whole gate chain and determinism.yml test (139 3.3)
000-spec-spine-bootstrap
# exempt: SBOM and SLSA provenance are produced only by a tag-triggered release run (139 3.3)
019-release-supply-chain-artifacts
# exempt: a process rule on pull-request edits; its descendants 082, 083, 095 and 118 carry acceptance (139 3.3)
037-amendment-authoring
LEDGER
}

usage() {
  cat <<USAGE
usage: $PROG [options]

  --rev <rev>           revision to test (default: HEAD)
  --trusted-ref <ref>   <rev> must be an ancestor of this (default: the
                        repository's origin/HEAD, else origin/main)
  --repo <dir>          repository to sweep (default: the current directory)
  --only <id,...>       sweep only these spec ids, as a full 'NNN-slug' or the
                        3-digit ordinal 'NNN' (the short form spec-spine itself
                        resolves; '49' is not one, '049' is)
  --out <dir>           run directory for the worktree, logs and report
                        (default: a new directory per run,
                        \${XDG_CACHE_HOME:-\$HOME/.cache}/spec-spine/sweeps/
                        <shortsha>-<UTC time>-<pid>)
  --release             exit with the release verdict: judged over the specs
                        whose implementation is built, with pending work
                        reported and not counted (spec 119)
  --exempt-file <path>  read the exemption ledger from this file instead of
                        the one built into this script
  --timeout <seconds>   per-spec limit, enforced by this script
                        (default: 900; 0 disables)
  --keep-tree           do not remove the isolated worktree when finished
  -h, --help            this

  \$SPEC_SPINE_BIN       an already-built binary to drive the sweep with. When
                        unset, the sweep builds one from <rev> inside the
                        isolated worktree, which is the only binary whose
                        version provably matches the revision under test.
USAGE
}

die() { printf '%s: %s\n' "$PROG" "$*" >&2; exit 3; }
say() { printf '%s\n' "$*" >&2; }

rev="HEAD"
trusted_ref=""
repo="."
only=""
out=""
exempt_file=""
timeout_s=900
keep_tree=0
release=0

while [ $# -gt 0 ]; do
  case "$1" in
    --rev)          shift; [ $# -gt 0 ] || die "--rev needs a value"; rev="$1" ;;
    --trusted-ref)  shift; [ $# -gt 0 ] || die "--trusted-ref needs a value"; trusted_ref="$1" ;;
    --repo)         shift; [ $# -gt 0 ] || die "--repo needs a value"; repo="$1" ;;
    --only)         shift; [ $# -gt 0 ] || die "--only needs a value"; only="$1" ;;
    --out)          shift; [ $# -gt 0 ] || die "--out needs a value"; out="$1" ;;
    --exempt-file)  shift; [ $# -gt 0 ] || die "--exempt-file needs a value"; exempt_file="$1" ;;
    --timeout)      shift; [ $# -gt 0 ] || die "--timeout needs a value"; timeout_s="$1" ;;
    --keep-tree)    keep_tree=1 ;;
    --release)      release=1 ;;
    -h|--help)      usage; exit 0 ;;
    *)              usage >&2; die "unknown argument: $1" ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || die "git is not on PATH"
command -v python3 >/dev/null 2>&1 || die "python3 is not on PATH (the report writer needs it)"
case "$timeout_s" in *[!0-9]*|"") die "--timeout takes a whole number of seconds: $timeout_s" ;; esac

root=$(git -C "$repo" rev-parse --show-toplevel 2>/dev/null) || die "not a git repository: $repo"
# Physical paths on both sides of the containment test below: on macOS
# `rev-parse` answers /private/var while a `cd`+`pwd` answers /var, and a
# guard comparing the two forms would wave through an --out inside the repo.
root=$(cd "$root" && pwd -P)

# --- the trust boundary (spec 083 4) --------------------------------------
#
# `verify` runs what the corpus declares. Sweeping the whole corpus runs what
# every spec declares, so the revision has to be one the maintainer already
# trusts. "Already merged into the trusted ref" is the mechanical form of that,
# and it is why this script cannot be pointed at a PR branch by accident. The
# override is an explicit `--trusted-ref`, which is visible in the report.
if [ -z "$trusted_ref" ]; then
  trusted_ref=$(git -C "$root" symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null || echo "origin/main")
fi
git -C "$root" rev-parse --verify --quiet "$trusted_ref^{commit}" >/dev/null \
  || die "trusted ref '$trusted_ref' does not resolve; fetch it, or name one with --trusted-ref"
sha=$(git -C "$root" rev-parse --verify --quiet "$rev^{commit}") \
  || die "revision '$rev' does not resolve"
git -C "$root" merge-base --is-ancestor "$sha" "$trusted_ref" \
  || die "refusing: $rev ($(git -C "$root" rev-parse --short "$sha")) is not an ancestor of $trusted_ref.
  The sweep executes what every spec in the corpus declares, so it runs only
  against a revision that is already merged. Name a merged revision with
  --rev, or, if you genuinely trust another line of history, name it with
  --trusted-ref."

short=$(git -C "$root" rev-parse --short "$sha")
# The default run directory is outside the operating system's temporary tree
# and new for every run (spec 119). Under $TMPDIR, macOS's dirhelper deletes
# files older than three days at 03:35 each day, and a build script's copied
# outputs keep their crate archive's timestamps (2006, for tree-sitter). A
# sweep that crossed 03:35 reported 26 blocks failed that were not; the purge
# is the inferred cause, the deletion itself was not observed
# (docs/release-candidate-0.22.0.md 12). A fixed default also meant a rerun
# cleared the previous run's evidence.
if [ -z "$out" ]; then
  cache_root="${XDG_CACHE_HOME:-}"
  if [ -z "$cache_root" ]; then
    [ -n "${HOME:-}" ] || die "cannot choose a default --out: neither XDG_CACHE_HOME nor HOME is set; name one with --out"
    cache_root="$HOME/.cache"
  fi
  out="$cache_root/spec-spine/sweeps/$short-$(date -u +%Y%m%dT%H%M%SZ)-$$"
fi
# Canonicalized WITHOUT creating anything: the containment check below has to
# be able to refuse before a single directory exists inside the repository.
# Resolving by `mkdir -p` the parent first, then `cd`+`pwd`, refused
# `--out <repo>/new/run` only after `<repo>/new` had already been created.
# So walk up to the deepest ancestor that does exist, canonicalize that, and
# re-attach the part that does not.
probe="$out"
suffix=""
while [ ! -d "$probe" ]; do
  case "$probe" in /|.|"") break ;; esac
  suffix="$(basename "$probe")${suffix:+/$suffix}"
  probe="$(dirname "$probe")"
done
[ -d "$probe" ] || die "cannot resolve --out: $out"
out="$(cd "$probe" && pwd -P)${suffix:+/$suffix}"
case "$out" in
  "$root"|"$root"/*) die "--out must be outside the repository ($root): the sweep must never
  write into the tree the gate judges" ;;
esac
# A named --out under a purged temporary tree is honoured, and said (spec 119).
if [ "$(uname -s 2>/dev/null)" = "Darwin" ]; then
  for tmp_root in "${TMPDIR:-}" /private/tmp /private/var/tmp; do
    [ -n "$tmp_root" ] && [ -d "$tmp_root" ] || continue
    tmp_root=$(cd "$tmp_root" && pwd -P) || continue
    case "$out" in
      "$tmp_root"|"$tmp_root"/*)
        say "$PROG: warning: --out is under $tmp_root, which macOS purges of files older
  than three days (dirhelper, daily at 03:35). Build-script outputs keep their
  crate's old timestamps, so a run crossing 03:35 can lose them mid-sweep."
        break ;;
    esac
  done
fi

# The run directory is cleared before use, so it must be one this script is
# entitled to delete: absent, empty, or a directory a previous sweep created
# (marked below). Without this, a slipped `--out ~/work` is a `rm -rf` of
# someone's home, and `--out /` would reach here as `//`.
#
# The marker is written at creation, not at completion, so a run that died
# half-way is still re-runnable at the same --out. Keying on the report instead
# would refuse exactly the directory a maintainer is retrying.
readonly RUN_MARKER=".verify-sweep-run"
if [ -e "$out" ]; then
  if [ ! -d "$out" ]; then
    die "--out exists and is not a directory: $out"
  elif [ -f "$out/$RUN_MARKER" ] || [ -z "$(ls -A "$out" 2>/dev/null)" ]; then
    :
  else
    die "refusing to clear $out: it is neither empty nor a directory this
  script created (no $RUN_MARKER). Name a new --out, or remove it yourself."
  fi
fi
rm -rf "$out" || die "cannot clear $out"
mkdir -p "$out/logs" "$out/tmp" || die "cannot create $out"
: > "$out/$RUN_MARKER" || die "cannot create $out"
tree="$out/tree"

cleanup() {
  if [ "$keep_tree" -eq 0 ] && [ -d "$tree" ]; then
    git -C "$root" worktree remove --force "$tree" >/dev/null 2>&1 || true
  fi
  git -C "$root" worktree prune >/dev/null 2>&1 || true
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# --- isolation ------------------------------------------------------------
#
# The sweep never runs in the maintainer's checkout. A `## Verification` block
# is arbitrary shell: it may write fixtures, regenerate artifacts, or leave
# residue, and doing that to the working tree someone is mid-change in is not
# acceptable for a routine maintenance run. A detached worktree at <rev> gives
# each sweep its own tree, its own `target/`, and a `git` that can restore it.
git -C "$root" worktree add --detach "$tree" "$sha" >/dev/null 2>&1 \
  || die "cannot create an isolated worktree at $tree"

# --- the binary -----------------------------------------------------------
if [ -n "${SPEC_SPINE_BIN:-}" ]; then
  ss="$SPEC_SPINE_BIN"
  case "$ss" in /*) ;; *) ss="$(cd "$(dirname "$ss")" && pwd -P)/$(basename "$ss")" ;; esac
  [ -x "$ss" ] || die "\$SPEC_SPINE_BIN is not executable: $ss"
  binary_origin="SPEC_SPINE_BIN"
else
  # Built from <rev>, inside the worktree. This is the only binary whose version
  # provably corresponds to the corpus under test, and it is also the binary the
  # blocks themselves reach for: they invoke `target/release/spec-spine`
  # relative to the repository root, which here is the worktree.
  say "$PROG: building spec-spine from $short ..."
  ( cd "$tree" && cargo build --release --locked ) >"$out/logs/_build.log" 2>&1 \
    || die "cargo build --release --locked failed at $short; see $out/logs/_build.log"
  ss="$tree/target/release/spec-spine"
  binary_origin="built from $short"
fi
ss_version=$("$ss" --version 2>/dev/null) || die "$ss does not answer --version"

# --- the per-spec limit ---------------------------------------------------
#
# One block that hangs must not cost the verdict on every other spec. The limit
# is enforced here rather than by delegating to `timeout(1)`, which stock macOS
# does not ship: a limit that silently does not exist on half the machines that
# run this is worse than none, because the report would still claim one.
#
# `set -m` puts the child in its own process group so the whole tree a block
# spawned (a `cargo`, an `sh -c`, whatever they started) is signalled, not just
# the `verify` process that is their parent.
#
# Exit 124 is returned for an exceeded limit, matching timeout(1)'s convention.
run_limited() { # <seconds, 0 = no limit> <log> <spec-tmp> <spec-id>
  local _limit="$1" _log="$2" _tmp="$3" _id="$4" _pid _waited _grace _rc
  if [ "$_limit" -le 0 ]; then
    ( cd "$tree" && TMPDIR="$_tmp" "$ss" verify "$_id" ) >"$_log" 2>&1
    return $?
  fi
  set -m
  ( cd "$tree" && TMPDIR="$_tmp" "$ss" verify "$_id" ) >"$_log" 2>&1 &
  _pid=$!
  set +m
  _waited=0
  while kill -0 "$_pid" 2>/dev/null; do
    if [ "$_waited" -ge "$_limit" ]; then
      # Signalling the group is the contract; a platform that did not give the
      # job one is a degraded run, not a silent one, because whatever the block
      # spawned then outlives the kill.
      if ! kill -TERM "-$_pid" 2>/dev/null; then
        printf '\n[verify-sweep] no process group for this job; signalling the process only, so anything it spawned may survive\n' >> "$_log"
        kill -TERM "$_pid" 2>/dev/null || true
      fi
      # Grace, proportional to how long it actually takes to die rather than a
      # flat wait every time.
      _grace=0
      while kill -0 "$_pid" 2>/dev/null && [ "$_grace" -lt 2 ]; do
        sleep 1
        _grace=$((_grace + 1))
      done
      kill -KILL "-$_pid" 2>/dev/null || kill -KILL "$_pid" 2>/dev/null
      wait "$_pid" 2>/dev/null
      _rc=$?
      # It may have finished on its own in the window between the liveness
      # check and the signal. A spec that actually reached a verdict keeps it;
      # reporting it `not-run` because the clock ran out a moment later would
      # be the sweep inventing an outcome it did not observe.
      case "$_rc" in
        0|1) return "$_rc" ;;
      esac
      printf '\n[verify-sweep] killed after %ss\n' "$_limit" >> "$_log"
      return 124
    fi
    sleep 1
    _waited=$((_waited + 1))
  done
  wait "$_pid"
  return $?
}

# --- selection ------------------------------------------------------------
corpus=$("$ss" --repo "$tree" registry list --ids-only) \
  || die "cannot read the corpus at $short (registry list)"
[ -n "$corpus" ] || die "the corpus at $short is empty"

# The lifecycle, through the same governed read (spec 119). The CLI has already
# deserialized the ledger; this parses its answer, not the ledger files.
lifecycle_json=$("$ss" --repo "$tree" registry list --json) \
  || die "cannot read the corpus lifecycle at $short (registry list --json)"
printf '%s' "$lifecycle_json" | python3 -c '
import json, sys
for s in json.load(sys.stdin)["items"]:
    print("\t".join([s["id"], s.get("status") or "-", s.get("implementation") or "-"]))
' > "$out/lifecycle.tsv" || die "cannot parse the corpus lifecycle at $short (registry list --json)"

# Sets lc_status, lc_impl and lc_class (implemented | pending | unknown).
# Returned through globals: the one caller, emit_row, reads all three right
# after the call and before anything else could call this again.
lifecycle_of() {
  local _line
  _line=$(awk -F'\t' -v id="$1" '$1 == id { print; exit }' "$out/lifecycle.tsv")
  if [ -z "$_line" ]; then
    lc_status="-"; lc_impl="-"; lc_class="unknown"; return
  fi
  lc_status=$(printf '%s\n' "$_line" | cut -f2)
  lc_impl=$(printf '%s\n' "$_line" | cut -f3)
  case "$lc_impl" in
    complete|n-a) lc_class="implemented" ;;
    pending|in-progress|deferred) lc_class="pending" ;;
    -)
      # Spec 042: an absent key defers to `status`.
      case "$lc_status" in
        draft) lc_class="pending" ;;
        approved|superseded|retired) lc_class="implemented" ;;
        *) lc_class="unknown" ;;
      esac ;;
    *) lc_class="unknown" ;;
  esac
}

# Appends one row: the seven corpus fields, then the lifecycle and the release
# outcome. <plan-read> is 0 when `verify --plan` itself could not answer.
emit_row() { # id outcome commands exit failure dirty secs plan-read
  local _rel
  lifecycle_of "$1"
  if [ "$8" -eq 0 ] || [ "$lc_class" = "unknown" ]; then
    _rel="not-run"
  elif [ "$lc_class" = "pending" ]; then
    _rel="pending"
  else
    _rel="$2"
  fi
  case "$_rel" in
    failed|not-declared|not-run) n_rel_bad=$((n_rel_bad + 1)) ;;
    pending) n_rel_pending=$((n_rel_pending + 1)) ;;
  esac
  # A row that is not written is refused rather than lost: the exit code is
  # counted here and the report is written from the file, and the two must
  # describe the same run.
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$1" "$2" "$3" "$4" "$5" "$6" "$7" "$lc_status" "$lc_impl" "$lc_class" "$_rel" >> "$rows" \
    || die "cannot record the result for $1 in $rows"
}

selected="$corpus"
selection="all"
if [ -n "$only" ]; then
  selection="$only"
  picked=""
  for want in $(printf '%s' "$only" | tr ',' ' '); do
    hit=""
    for id in $corpus; do
      case "$id" in "$want"|"$want"-*) hit="$id" ;; esac
    done
    [ -n "$hit" ] || die "--only names no spec in the corpus at $short: $want
  Ids are a full \`NNN-slug\` or the 3-digit ordinal \`NNN\`, which is the short
  form spec-spine resolves everywhere (spec 015): \`049\`, not \`49\`."
    # A spec selected twice would be run twice and counted twice, so the report
    # would say the corpus is larger than it is. `--only 001,001` selects one.
    case "
$picked" in *"
$hit
"*) continue ;; esac
    picked="$picked$hit
"
  done
  selected=$(printf '%s' "$picked")
fi

# --- the ledger, validated before anything runs ---------------------------
if [ -n "$exempt_file" ]; then
  [ -f "$exempt_file" ] || die "--exempt-file not found: $exempt_file"
  ledger=$(sed -e 's/#.*//' -e 's/[[:space:]]*$//' "$exempt_file" | grep -v '^$' || true)
  ledger_origin="$exempt_file"
else
  ledger=$(legacy_ledger | sed -e 's/#.*//' -e 's/[[:space:]]*$//' | grep -v '^$' || true)
  ledger_origin="built into $PROG"
fi

declared_p() { # spec id -> 0 when it declares at least one verify:cli command
  n=$("$ss" --repo "$tree" verify "$1" --plan 2>/dev/null | grep -c . | tr -d ' ')
  [ "${n:-0}" -gt 0 ]
}

for id in $ledger; do
  ord=$(printf '%s' "$id" | sed -n 's/^\([0-9][0-9]*\)-.*$/\1/p')
  [ -n "$ord" ] || die "ledger entry is not an <ordinal>-<slug> id: $id"
  # The ledger is closed: nothing filed at or after the ordinal where the
  # mechanism arrived can be exempted, so no future spec is covered by it.
  [ "$((10#$ord))" -lt "$LEDGER_CLOSED_AT" ] \
    || die "ledger entry '$id' is at or above the closed ordinal $LEDGER_CLOSED_AT.
  The legacy ledger covers only the specs filed before \`verify\` existed. A
  spec filed after it declares acceptance; it is not exempted."
  found=""
  for c in $corpus; do [ "$c" = "$id" ] && found=1; done
  [ -n "$found" ] || die "ledger entry '$id' names no spec in the corpus at $short.
  A dangling exemption hides nothing and accounts for nothing; remove the line."
  if declared_p "$id"; then
    die "ledger entry '$id' declares acceptance: the exemption is stale.
  The ledger only shrinks. Delete the line; the sweep will run the block."
  fi
done

exempt_p() {
  for e in $ledger; do [ "$e" = "$1" ] && return 0; done
  return 1
}

# --- the sweep ------------------------------------------------------------
#
# Serial, and it never stops. A failing block is one spec's result; the run
# continues so that one red block cannot cost the verdict on the other 111.
started_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
rows="$out/rows.tsv"
: > "$rows"
n_passed=0; n_failed=0; n_notdecl=0; n_exempt=0; n_notrun=0
n_rel_bad=0; n_rel_pending=0

for id in $selected; do
  plan_out=$("$ss" --repo "$tree" verify "$id" --plan 2>/dev/null)
  plan_rc=$?
  if [ "$plan_rc" -ne 0 ]; then
    # The verb could not answer what this spec declares, so neither can the
    # sweep. Reporting that as `not-declared` would claim a fact about the
    # spec's acceptance that was never established.
    emit_row "$id" not-run 0 "$plan_rc" "no verdict: verify --plan exited $plan_rc" 0 0 0
    n_notrun=$((n_notrun + 1))
    say "  NOT-RUN       $id (verify --plan exit $plan_rc)"
    continue
  fi
  # A ledger entry excuses absent acceptance, not an unreadable document.
  # The plan must answer before an exemption can count as success (3.3).
  if exempt_p "$id"; then
    emit_row "$id" exempt 0 0 "" "" 0 1
    n_exempt=$((n_exempt + 1))
    say "  exempt        $id"
    continue
  fi
  total=$(printf '%s' "$plan_out" | grep -c . | tr -d ' ')
  if [ "${total:-0}" -eq 0 ]; then
    emit_row "$id" not-declared 0 0 "" "" 0 1
    n_notdecl=$((n_notdecl + 1))
    say "  NOT-DECLARED  $id"
    continue
  fi

  log="$out/logs/$id.log"
  spec_tmp="$out/tmp/$id"
  rm -rf "$spec_tmp" && mkdir -p "$spec_tmp"
  t0=$(date +%s)
  # Each spec gets its own TMPDIR. Blocks in this corpus build fixture corpora
  # under ${TMPDIR}/ssNNN; a shared TMPDIR would let one spec's residue decide
  # another spec's outcome, which is the opposite of an accountable result.
  run_limited "$timeout_s" "$log" "$spec_tmp" "$id"
  code=$?
  t1=$(date +%s)
  secs=$((t1 - t0))
  rm -rf "$spec_tmp"

  fail_cmd=""
  case "$code" in
    0) outcome="passed";  n_passed=$((n_passed + 1)); say "  passed        $id (${secs}s)" ;;
    1) outcome="failed";  n_failed=$((n_failed + 1))
       fail_cmd=$(grep -m1 'FAILED at command' "$log" | tr '\t' ' ')
       say "  FAILED        $id (${secs}s) ${fail_cmd:-see $log}" ;;
    124)
       outcome="not-run"; n_notrun=$((n_notrun + 1))
       fail_cmd="no verdict: the block exceeded the ${timeout_s}s per-spec limit and was killed"
       say "  NOT-RUN       $id (exceeded ${timeout_s}s)" ;;
    *) outcome="not-run"; n_notrun=$((n_notrun + 1))
       fail_cmd="verify exited $code (no verdict; 2=stale, 3=I/O, parse, schema or config)"
       say "  NOT-RUN       $id (verify exit $code)" ;;
  esac

  # Fixture hygiene: a block that dirtied the tree is recorded and the tree is
  # restored, so the next spec is judged against <rev> and not against the
  # leftovers of the last one. Ignored paths (target/, build-meta.json) are
  # deliberately kept: rebuilding them for every spec would cost hours.
  dirty=0
  tree_status=$(git -C "$tree" status --porcelain) \
    || die "cannot inspect the isolated worktree after $id"
  tree_head=$(git -C "$tree" rev-parse --verify HEAD) \
    || die "cannot resolve the isolated worktree revision after $id"
  if [ -n "$tree_status" ] || [ "$tree_head" != "$sha" ] \
    || git -C "$tree" symbolic-ref -q HEAD >/dev/null 2>&1; then
    dirty=1
    # Restore both the index and tracked files from the immutable revision.
    # A path checkout would preserve staged edits; a clean status alone would
    # miss a block that committed them. Detach so no branch is moved here.
    git -C "$tree" checkout --detach --force "$sha" >/dev/null 2>&1 \
      || die "cannot restore the isolated worktree to $sha after $id"
    git -C "$tree" clean -fdq >/dev/null 2>&1 \
      || die "cannot clean the isolated worktree after $id"
    tree_status=$(git -C "$tree" status --porcelain) \
      || die "cannot inspect the restored worktree after $id"
    [ -z "$tree_status" ] \
      || die "the isolated worktree is still dirty after restoring $id"
    say "                (block left the tree dirty; restored)"
  fi

  emit_row "$id" "$outcome" "$total" "$code" "$fail_cmd" "$dirty" "$secs" 1
done

ended_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# --- the report -----------------------------------------------------------
#
# Enough to reproduce a finding without the run directory: the revision, the
# binary that produced it, the selection, the ledger that was applied, and per
# spec the outcome, the command that failed and the log that holds its output.
#
# Written in one pass so the JSON and the markdown cannot disagree about what
# the run found, and so quoting is done by a JSON encoder rather than by hand.
SWEEP_OUT="$out" \
SWEEP_ROWS="$rows" \
SWEEP_SHA="$sha" \
SWEEP_SHORT="$short" \
SWEEP_TRUSTED="$trusted_ref" \
SWEEP_VERSION="$ss_version" \
SWEEP_ORIGIN="$binary_origin" \
SWEEP_SELECTION="$selection" \
SWEEP_LEDGER="$ledger_origin" \
SWEEP_CLOSED_AT="$LEDGER_CLOSED_AT" \
SWEEP_TIMEOUT="$timeout_s" \
SWEEP_STARTED="$started_at" \
SWEEP_ENDED="$ended_at" \
SWEEP_MODE="$([ "$release" -eq 1 ] && echo release || echo corpus)" \
python3 - <<'PY' || die "cannot write the report to $out"
import json, os, pathlib

env = os.environ
out = pathlib.Path(env["SWEEP_OUT"])
fields = ("id", "outcome", "commands", "exitCode", "failure", "leftTreeDirty", "seconds",
          "status", "implementation", "lifecycleClass", "releaseOutcome")
specs = []
for line in pathlib.Path(env["SWEEP_ROWS"]).read_text().splitlines():
    if not line.strip():
        continue
    parts = line.split("\t")
    parts += [""] * (len(fields) - len(parts))
    row = dict(zip(fields, parts))
    row["commands"] = int(row["commands"] or 0)
    row["exitCode"] = int(row["exitCode"] or 0)
    row["leftTreeDirty"] = row["leftTreeDirty"] == "1"
    row["seconds"] = int(row["seconds"] or 0)
    # Spec 119: what the ledger says about the spec, as read, and the outcome
    # the release verdict counts. `-` is an absent key, reported as null.
    status, impl = row.pop("status"), row.pop("implementation")
    row["lifecycle"] = {
        "status": None if status in ("", "-") else status,
        "implementation": None if impl in ("", "-") else impl,
        "class": row.pop("lifecycleClass") or "unknown",
    }
    # Cited only when it exists. A spec whose `verify --plan` failed never
    # reached a run, so there is no log for it, and pointing the reader at a
    # path that is not there is worse than saying there is nothing to read.
    log = out / "logs" / f"{row['id']}.log"
    row["log"] = f"logs/{row['id']}.log" if log.is_file() else None
    specs.append(row)

counts = {k: 0 for k in ("passed", "failed", "not-declared", "exempt", "not-run")}
for s in specs:
    counts[s["outcome"]] = counts.get(s["outcome"], 0) + 1
release_counts = {k: 0 for k in ("passed", "failed", "not-declared", "exempt", "not-run", "pending")}
for s in specs:
    release_counts[s["releaseOutcome"]] = release_counts.get(s["releaseOutcome"], 0) + 1
BAD = ("failed", "not-declared", "not-run")
corpus_clean = not any(counts[k] for k in BAD)
release_clean = not any(release_counts[k] for k in BAD)
timeout = int(env["SWEEP_TIMEOUT"])

report = {
    # 1.1.0 (spec 119): additive. `counts`, the five outcomes and the exit
    # code without --release mean what they meant in 1.0.0.
    "schemaVersion": "1.1.0",
    "mode": env["SWEEP_MODE"],
    "verdicts": {
        "corpus": "clean" if corpus_clean else "not-clean",
        "release": "clean" if release_clean else "not-clean",
    },
    "releaseCounts": release_counts,
    "tool": "verify-sweep",
    "revision": env["SWEEP_SHA"],
    "revisionShort": env["SWEEP_SHORT"],
    "trustedRef": env["SWEEP_TRUSTED"],
    "binaryVersion": env["SWEEP_VERSION"],
    "binaryOrigin": env["SWEEP_ORIGIN"],
    "selection": env["SWEEP_SELECTION"],
    "ledgerOrigin": env["SWEEP_LEDGER"],
    "ledgerClosedAt": int(env["SWEEP_CLOSED_AT"]),
    "timeoutSeconds": timeout or None,
    "startedAt": env["SWEEP_STARTED"],
    "endedAt": env["SWEEP_ENDED"],
    "counts": counts,
    "specs": specs,
}
(out / "sweep.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")

not_passing = [s for s in specs if s["outcome"] in ("failed", "not-declared", "not-run")]
md = [
    f"# verification sweep: {env['SWEEP_SHORT']}",
    "",
    f"- revision: `{env['SWEEP_SHA']}` (ancestor of `{env['SWEEP_TRUSTED']}`)",
    f"- binary: `{env['SWEEP_VERSION']}` ({env['SWEEP_ORIGIN']})",
    f"- selection: {env['SWEEP_SELECTION']}",
    f"- ledger: {env['SWEEP_LEDGER']}, closed at ordinal {env['SWEEP_CLOSED_AT']}",
    f"- per-spec limit: {str(timeout) + 's' if timeout else 'none (--timeout 0)'}",
    f"- ran: {env['SWEEP_STARTED']} .. {env['SWEEP_ENDED']}",
    "",
    "| outcome | count |",
    "|---|---|",
]
md += [f"| {k} | {counts[k]} |" for k in ("passed", "failed", "not-declared", "exempt", "not-run")]
md += ["", f"**{'NOT CLEAN' if not_passing else 'CLEAN'}**: "
       f"{len(not_passing)} of {len(specs)} selected specs are not passing.", ""]
pending = [s for s in specs if s["releaseOutcome"] == "pending"]
release_bad = [s for s in specs if s["releaseOutcome"] in BAD]
exit_from = "release" if env["SWEEP_MODE"] == "release" else "corpus"
md += [
    "## Release verdict",
    "",
    "Judged over the specs whose implementation is built (`complete`, `n-a`, or",
    "absent on a non-draft). Pending work runs and is reported above, and is",
    "counted here as `pending`: neither success nor failure of the release.",
    f"This run's exit code is the **{exit_from}** verdict.",
    "",
    "| release outcome | count |",
    "|---|---|",
]
md += [f"| {k} | {release_counts[k]} |" for k in ("passed", "failed", "not-declared", "exempt", "not-run", "pending")]
md += ["", f"**{'CLEAN' if release_clean else 'NOT CLEAN'}**: "
       f"{len(release_bad)} implemented or unreadable specs are not passing; "
       f"{len(pending)} pending.", ""]
if release_bad:
    md += ["### Not passing the release", ""]
    for s in release_bad:
        lc = s["lifecycle"]
        md.append(f"- **{s['id']}** {s['releaseOutcome']} "
                  f"(status {lc['status'] or 'absent'}, implementation "
                  f"{lc['implementation'] or 'absent'}, {lc['class']})")
    md.append("")
if pending:
    md += ["### Pending, not a release obligation", "",
           "Listed with the outcome their block produced, so nothing is hidden.", ""]
    for s in pending:
        lc = s["lifecycle"]
        md.append(f"- {s['id']}: status {lc['status'] or 'absent'}, implementation "
                  f"{lc['implementation'] or 'absent'}; block {s['outcome']}")
    md.append("")
if not_passing:
    md += ["## Not passing", ""]
    for s in not_passing:
        line = f"- **{s['id']}** {s['outcome']}"
        if s["failure"]:
            line += f" : {s['failure']}"
        if s["log"]:
            line += f" (`{s['log']}`)"
        md.append(line)
    md.append("")
dirty = [s["id"] for s in specs if s["leftTreeDirty"]]
if dirty:
    md += ["## Blocks that left the worktree dirty", "",
           "Restored before the next spec ran; recorded because a block that writes",
           "to the tree it is judged in is a defect in that block.", ""]
    md += [f"- {i}" for i in dirty] + [""]
(out / "sweep.md").write_text("\n".join(md))
PY

say ""
say "$PROG: $short  passed=$n_passed failed=$n_failed not-declared=$n_notdecl exempt=$n_exempt not-run=$n_notrun"
if [ "$n_rel_bad" -eq 0 ]; then rel_verdict="clean"; else rel_verdict="NOT CLEAN"; fi
say "$PROG: release verdict: $rel_verdict  (not passing=$n_rel_bad pending=$n_rel_pending)"
say "$PROG: report $out/sweep.md  (json: $out/sweep.json, logs: $out/logs/)"

if [ "$release" -eq 1 ]; then
  [ "$n_rel_bad" -eq 0 ] || exit 1
  exit 0
fi
[ "$((n_failed + n_notdecl + n_notrun))" -eq 0 ] || exit 1
exit 0
