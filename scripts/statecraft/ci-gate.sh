#!/usr/bin/env bash
# Rendered by Statecraft from profile github-actions-rust revision 28.
# The aggregate gate. It passes only when every job the TRUSTED policy names
# as required ended the way its event requires:
#
#   required          must be `success`; `skipped` is an unexpected skip
#   required-review   `success` carrying a review result (findings,
#                     no-findings, or a visible skipped:<class>)
#   owner-exception   required (success) for a pull request whose review
#                     returned findings, and for a release candidate whose
#                     review was skipped; otherwise inapplicable (revision 2)
#   rc-exception      revision 1's rule: the release-candidate case only
#   recorded-review   a merge-queue entry (revision 3): the review is not
#                     re-run, so the job must be `skipped`, and the verdict
#                     recorded for the entry's pull request at its head is
#                     what admits it, with the owner exception for findings
#   inapplicable      must be `skipped`, and the rule that admitted it is printed
#   post-merge        a push to the default branch where the merge queue is
#                     required (revision 24, spec 067): `skipped` or `success`,
#                     and the pushed commit must have passed ci-gate in a
#                     completed merge-queue run of statecraft-ci at that commit
#
# A required job missing from the needs record has vanished, and blocks. The
# required set is read from the policy at the base commit, never from the
# candidate, so a candidate cannot drop a job from the set that judges it.
#
# An authority change blocks unless the owner approves it (revision 5, rule 2):
# when the candidate's own changes (BASE...HEAD) touch the authority set the
# base's policy defines (its files, the policy itself, scripts/statecraft/*
# and the declared authored-content script), the owner exception must have
# succeeded for this run, or, in the merge queue, for the run recorded for the
# entry's pull request. The gate computes this itself and reads no job's
# claim about it, so a candidate workflow that skips the exception fails
# closed. On push it is reported: the change was approved on its pull request.
#
# A ratification needs the owner exception by the same rule (revision 12,
# spec 024): a changed spec.md whose frontmatter status is `approved` at the
# head and anything else, or absent, at the base. The gate reads both trees
# itself; the coupling steps refuse a changed path a `draft` spec still owns.
#
# Inputs, all from the environment: NEEDS_JSON (toJSON(needs)), EVENT_NAME,
# HEAD_SHA, BASE_SHA (the pull request base, or the push's previous head),
# HEAD_REF (the pull request head ref; empty on push). On merge_group also
# GROUP_HEAD_REF (the queue branch, `.../pr-<n>-<sha>`), REPO and GH_TOKEN,
# which read the recorded review through the API.
#
# The family exit contract (revision 7): 0 passed, 1 blocked (a finding), 2
# refused (a precondition the gate cannot judge past), 3 usage (an input is
# missing), 4 failed. A command that fails outside a test is an operation
# that broke: the ERR trap reports it as 4, whatever its own code was.
set -eEuo pipefail
trap 'echo "ci-gate: a command failed (exit $?) at line ${LINENO}; reported as failed (4)" >&2; exit 4' ERR

for input in NEEDS_JSON EVENT_NAME HEAD_SHA; do
  if [ -z "${!input:-}" ]; then
    echo "ci-gate.sh needs ${input}" >&2
    exit 3
  fi
done
BASE_SHA="${BASE_SHA:-}"
HEAD_REF="${HEAD_REF:-}"

POLICY=.statecraft/setup/github-actions-rust.json
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

say() {
  printf '%s\n' "$*"
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    printf '%s\n\n' "$*" >> "$GITHUB_STEP_SUMMARY"
  fi
}

# A precondition the gate cannot judge past: say why, and refuse (2).
stop() {
  say "BLOCK: $*"
  exit 2
}

# The trusted policy: the base's copy. A base without one is the adoption
# itself, the only case the candidate's copy is read, and it is said.
trusted=no
case "$BASE_SHA" in
  "" | 0000000000000000000000000000000000000000) ;;
  *)
    # Revision 12 (spec 024 3.6): only a readable base without the policy is
    # the adoption. A base that does not resolve to a commit (a shallow or
    # failed fetch, a force-pushed `before`, a bad ref) refuses, and the
    # candidate's policy never judges in its place.
    if ! git cat-file -e "${BASE_SHA}^{commit}" 2>/dev/null; then
      stop "cannot read the base commit ${BASE_SHA}; the policy is never taken from the candidate in its place"
    fi
    if git cat-file -e "${BASE_SHA}:${POLICY}" 2>/dev/null; then
      git show "${BASE_SHA}:${POLICY}" > "$work/policy.json"
      trusted=yes
    fi
    ;;
esac
if [ "$trusted" = yes ]; then
  say "policy: read at the base ${BASE_SHA}"
else
  if [ ! -f "$POLICY" ]; then
    stop "no policy at the base and none in the candidate (${POLICY})"
  fi
  cp "$POLICY" "$work/policy.json"
  say "policy: the base carries none, so the candidate's is read; this pull request adopts the gate and is an authority change"
fi

if ! jq -e --arg e "$EVENT_NAME" '.jobs | to_entries | all(.value | has($e))' "$work/policy.json" > /dev/null; then
  stop "the policy states no rule for event '${EVENT_NAME}'"
fi

pattern="$(jq -r '.review.release_branch_pattern // ""' "$work/policy.json")"
release_candidate=no
if [ -n "$pattern" ] && [ -n "$HEAD_REF" ]; then
  # shellcheck disable=SC2053
  if [[ "$HEAD_REF" == $pattern ]]; then
    release_candidate=yes
  fi
fi

review_result="$(printf '%s' "$NEEDS_JSON" | jq -r '.["ai-review"].outputs.result // ""')"
# Revision 26 (spec 068): true only when the review job read a review note for
# this head naming every finding, each low and other.
review_answered="$(printf '%s' "$NEEDS_JSON" | jq -r '.["ai-review"].outputs.answered // ""')"

# The authority set is the base's, and the candidate's changes are read from
# its fork point: three dots, as coupling reads them. Two would also list
# what the base changed after the branch was cut, which is not this
# candidate's authority change.
authority=no
if [ "$trusted" = yes ]; then
  {
    jq -r '.files[].path, (.parameters.authored_content // empty), (.parameters.code.script // empty)' "$work/policy.json"
    echo "$POLICY"
  } | sort -u > "$work/authority-set"
  git diff --name-only "${BASE_SHA}...${HEAD_SHA}" | sort -u > "$work/changed"
  {
    comm -12 "$work/authority-set" "$work/changed"
    grep '^scripts/statecraft/' "$work/changed" || true
    # Revision 6: every workflow file, rendered or not. A workflow the
    # profile does not render could otherwise report a check named ci-gate.
    grep '^\.github/workflows/' "$work/changed" || true
  } | sort -u > "$work/touched"
  if [ -s "$work/touched" ]; then
    authority=yes
  fi
fi

# Revision 12 (spec 024): the specs this candidate ratifies, read from the
# two trees without spec-spine and without any job's claim.
status_at() {
  if git cat-file -e "$1:$2" 2> /dev/null; then
    git show "$1:$2" > "$work/status-spec.md"
    awk 'NR == 1 && $0 != "---" { done = 1 } NR > 1 && $0 == "---" { done = 1 }
      !done && NR > 1 && /^status:/ { sub(/^status:[[:space:]]*/, ""); gsub(/["\047]/, ""); sub(/[[:space:]]+$/, ""); print; done = 1 }' \
      "$work/status-spec.md"
  fi
}
ratification=no
: > "$work/ratified"
case "$BASE_SHA" in
  "" | 0000000000000000000000000000000000000000) ;;
  *)
    git diff --name-only "${BASE_SHA}...${HEAD_SHA}" > "$work/changed-all"
    while IFS= read -r path; do
      case "$path" in spec.md | */spec.md) ;; *) continue ;; esac
      after="$(status_at "$HEAD_SHA" "$path")"
      before="$(status_at "$BASE_SHA" "$path")"
      if [ "$after" = approved ] && [ "$before" != approved ]; then
        printf '%s (%s -> approved)\n' "$path" "${before:-absent}" >> "$work/ratified"
      fi
    done < "$work/changed-all"
    if [ -s "$work/ratified" ]; then
      ratification=yes
    fi
    ;;
esac

blocked=0
recorded=no
post_merge=no
block() {
  say "BLOCK: $*"
  blocked=1
}

# Revision 3: the review recorded for a merge-queue entry's pull request.
# Sets pr, pr_head, pr_ref, recorded_result and recorded_exception, or blocks
# with the reason and returns 1. Nothing here writes.
recorded_review() {
  pr="$(printf '%s' "${GROUP_HEAD_REF:-}" | sed -n 's|.*/pr-\([0-9][0-9]*\)-[0-9a-f]*$|\1|p')"
  if [ -z "$pr" ]; then
    block "cannot read the queued pull request's number from '${GROUP_HEAD_REF:-}'"
    return 1
  fi
  if ! gh api "repos/${REPO}/pulls/${pr}" > "$work/pr.json" 2> "$work/pr.err"; then
    block "cannot read pull request #${pr}: $(cat "$work/pr.err")"
    return 1
  fi
  pr_head="$(jq -r '.head.sha // ""' "$work/pr.json")"
  pr_ref="$(jq -r '.head.ref // ""' "$work/pr.json")"
  local run
  run="$(gh api "repos/${REPO}/actions/runs?event=pull_request&head_sha=${pr_head}&status=completed&per_page=100" 2> /dev/null \
    | jq -r --arg h "$pr_head" '[.workflow_runs[]? | select(.name == "statecraft-ci" and .event == "pull_request" and .head_sha == $h and .status == "completed")] | max_by(.id) | .id // empty' 2> /dev/null || true)"
  if [ -z "$run" ]; then
    block "no recorded review: pull request #${pr} has no completed statecraft-ci run at its head ${pr_head}"
    return 1
  fi
  mkdir -p "$work/record"
  if ! gh run download "$run" --repo "$REPO" --name "statecraft-ai-review-${pr_head}" --dir "$work/record" > /dev/null 2>&1 \
    || [ ! -s "$work/record/ai-review-evidence.json" ]; then
    block "no recorded review: run ${run} of pull request #${pr} carries no evidence record for ${pr_head}"
    return 1
  fi
  if ! jq -e --argjson n "$pr" --arg h "$pr_head" '.subject.pullRequest == $n and .subject.head == $h' "$work/record/ai-review-evidence.json" > /dev/null 2>&1; then
    block "the recorded review names another pull request or head than #${pr} at ${pr_head}"
    return 1
  fi
  recorded_result="$(jq -r '.result // ""' "$work/record/ai-review-evidence.json")"
  gh api "repos/${REPO}/actions/runs/${run}/jobs?per_page=100" > "$work/recorded-jobs.json" 2> /dev/null || echo '{}' > "$work/recorded-jobs.json"
  recorded_exception="$(jq -r '[.jobs[]? | select(.name == "review-exception")][0].conclusion // "absent"' "$work/recorded-jobs.json" 2> /dev/null || echo absent)"
  recorded_gate="$(jq -r '[.jobs[]? | select(.name == "ci-gate")][0].conclusion // "absent"' "$work/recorded-jobs.json" 2> /dev/null || echo absent)"
  say "recorded review: #${pr} at ${pr_head}, run ${run}: ${recorded_result} (exception: ${recorded_exception})"
}

# Revision 24 (spec 067): GitHub fast-forwards the default branch to the head
# of a merge-queue entry that passed, so that entry's run and its ci-gate
# check run sit on the pushed commit. A post-merge push is admitted only by
# that record; a push that bypassed the queue was judged nowhere, and blocks.
# Nothing here writes.
queue_judged() {
  local run gate
  if ! gh api "repos/${REPO:-}/actions/runs?event=merge_group&head_sha=${HEAD_SHA}&status=completed&per_page=100" > "$work/queue-runs.json" 2> "$work/queue-runs.err"; then
    block "post-merge push: cannot read the merge-queue runs of ${HEAD_SHA}: $(cat "$work/queue-runs.err")"
    return 0
  fi
  run="$(jq -r --arg h "$HEAD_SHA" '[.workflow_runs[]? | select(.name == "statecraft-ci" and .event == "merge_group" and .head_sha == $h and .status == "completed")] | max_by(.id) | .id // empty' "$work/queue-runs.json" 2> /dev/null || true)"
  if [ -z "$run" ]; then
    block "post-merge push: ${HEAD_SHA} is the head of no completed merge-queue run of statecraft-ci, so it was judged nowhere"
    return 0
  fi
  if ! gh api "repos/${REPO:-}/actions/runs/${run}/jobs?per_page=100" > "$work/queue-jobs.json" 2> "$work/queue-jobs.err"; then
    block "post-merge push: cannot read the jobs of merge-queue run ${run}: $(cat "$work/queue-jobs.err")"
    return 0
  fi
  gate="$(jq -r '[.jobs[]? | select(.name == "ci-gate")][0].conclusion // "absent"' "$work/queue-jobs.json" 2> /dev/null || echo unreadable)"
  if [ "$gate" != success ]; then
    block "post-merge push: ci-gate in merge-queue run ${run} at ${HEAD_SHA} ended '${gate}'"
    return 0
  fi
  say "post-merge push: ${HEAD_SHA} passed ci-gate in merge-queue run ${run}; its checks are not re-run on the push"
}

# The required jobs and their rules, read into a file first: a read that
# failed inside a process substitution would be an empty set, and an empty
# set passes (revision 7).
jq -r --arg e "$EVENT_NAME" '.jobs | to_entries[] | select(.value.required) | [.key, .value[$e]] | @tsv' "$work/policy.json" > "$work/required"

while IFS=$'\t' read -r job rule; do
  result="$(printf '%s' "$NEEDS_JSON" | jq -r --arg j "$job" 'if has($j) then .[$j].result else "vanished" end')"
  if [ "$result" = vanished ]; then
    block "required job '${job}' is not in the needs record: it vanished"
    continue
  fi
  if [ "$rule" = rc-exception ] || [ "$rule" = owner-exception ]; then
    exception_rule="$rule"
    case "$review_result" in
      skipped:*)
        if [ "$release_candidate" = yes ]; then rule=required; else rule=inapplicable; fi ;;
      findings)
        if [ "$exception_rule" = owner-exception ] && [ "$review_answered" != true ]; then rule=required; else rule=inapplicable; fi ;;
      *) rule=inapplicable ;;
    esac
    if [ "$exception_rule" = owner-exception ] && [ "$review_result" = findings ] && [ "$review_answered" = true ]; then
      say "review: findings, every one low and other and answered by a review note for this head (spec 068)"
    fi
    # A findings verdict is the final approver's refusal: only the owner's
    # exception, approved for this run, lets the head through, unless every
    # finding is low and other and answered (revision 26, spec 068).
    if [ "$exception_rule" = owner-exception ] && [ "$review_result" = findings ] && [ "$review_answered" != true ] && [ "$result" != success ]; then
      block "the AI review returned findings and the owner exception '${job}' was not approved for this run (it ended '${result}')"
      continue
    fi
    # Revision 5, rule 2: a change to the gate is the owner's, so the
    # exception is required whatever the review said.
    if [ "$authority" = yes ] && [ "$EVENT_NAME" = pull_request ]; then
      if [ "$result" != success ]; then
        block "this candidate changes the authority set and the owner exception '${job}' was not approved for this run (it ended '${result}')"
        continue
      fi
      rule=required
    fi
    # Revision 12: a ratification is the owner's act, so the exception is
    # required for it too.
    if [ "$ratification" = yes ] && [ "$EVENT_NAME" = pull_request ]; then
      if [ "$result" != success ]; then
        block "this candidate ratifies a spec and the owner exception '${job}' was not approved for this run (it ended '${result}')"
        continue
      fi
      rule=required
    fi
  fi
  case "$rule" in
    required | required-review)
      case "$result" in
        success) ;;
        skipped)
          block "required job '${job}' was skipped where it applies (${EVENT_NAME}): an unexpected skip"
          continue ;;
        *)
          block "required job '${job}' ended '${result}'"
          continue ;;
      esac
      if [ "$rule" = required-review ]; then
        case "$review_result" in
          findings | no-findings | skipped:draft | skipped:fork | skipped:dependabot | skipped:oversized | skipped:transient)
            say "review: ${review_result}" ;;
          *)
            block "job '${job}' succeeded without a review result (got '${review_result}')"
            continue ;;
        esac
      fi
      say "ok: ${job} ${result}"
      ;;
    recorded-review)
      if [ "$result" != skipped ]; then
        block "job '${job}' is not re-run in the merge queue and must be skipped, but ended '${result}'"
        continue
      fi
      if ! recorded_review; then
        recorded=failed
        continue
      fi
      recorded=yes
      entry_rc=no
      if [ -n "$pattern" ] && [ -n "$pr_ref" ]; then
        # shellcheck disable=SC2053
        if [[ "$pr_ref" == $pattern ]]; then entry_rc=yes; fi
      fi
      case "$recorded_result" in
        no-findings) ;;
        findings)
          # Revision 26 (spec 068): the pull request's own ci-gate passed
          # with the exception skipped only when every finding was answered.
          if [ "$recorded_exception" != success ]; then
            if [ "$recorded_exception" = skipped ] && [ "$recorded_gate" = success ]; then
              say "the review recorded for #${pr} returned findings, every one answered by a review note when its ci-gate passed (spec 068)"
            else
              block "the review recorded for #${pr} returned findings and its owner exception was not approved (it ended '${recorded_exception}')"
              continue
            fi
          fi ;;
        skipped:draft | skipped:fork | skipped:dependabot | skipped:oversized | skipped:transient)
          if [ "$entry_rc" = yes ] && [ "$recorded_exception" != success ]; then
            block "the review recorded for release candidate #${pr} was ${recorded_result} and its owner exception was not approved"
            continue
          fi ;;
        *)
          block "the review recorded for #${pr} carries no review result (got '${recorded_result}')"
          continue ;;
      esac
      say "ok: ${job} skipped in the merge queue, admitted by the review recorded for #${pr} at ${pr_head}: ${recorded_result}"
      ;;
    post-merge)
      post_merge=yes
      case "$result" in
        success | skipped)
          say "ok: ${job} ${result} on a post-merge push" ;;
        *)
          block "job '${job}' ended '${result}' on a post-merge push" ;;
      esac
      ;;
    inapplicable)
      if [ "$result" = skipped ]; then
        say "ok: ${job} skipped, admitted because it is inapplicable on ${EVENT_NAME}"
      else
        block "job '${job}' is inapplicable on ${EVENT_NAME} and must be skipped, but ended '${result}'"
      fi
      ;;
    *)
      block "the policy names an unknown rule '${rule}' for '${job}'"
      ;;
  esac
done < "$work/required"

if [ "$post_merge" = yes ]; then
  queue_judged
fi

if [ "$release_candidate" = yes ]; then
  say "release candidate: ${HEAD_REF} matches ${pattern}"
fi

# An authority change is never silently accepted: it is named, and on a pull
# request or a queue entry it needs the owner's exception (revision 5, rule 2).
if [ "$authority" = yes ]; then
  say "authority change: this candidate changes the gate that judges it:"
  say "$(cat "$work/touched")"
  case "$EVENT_NAME" in
    pull_request)
      say "authority change: the owner exception is required for this run" ;;
    merge_group)
      # The review job is skipped in the queue, so the exception that counts
      # is the one recorded for the entry's pull request at its head.
      if [ "$recorded" = no ] && recorded_review; then
        recorded=yes
      fi
      if [ "$recorded" = yes ]; then
        if [ "$recorded_exception" = success ]; then
          say "authority change: admitted by the owner exception recorded for #${pr} at ${pr_head}"
        else
          block "the queued group changes the authority set and the owner exception recorded for #${pr} was not approved (it ended '${recorded_exception}')"
        fi
      fi
      # Otherwise the record could not be read, and recorded_review has
      # already blocked with its reason on every path that returns 1: an
      # unread record never admits an authority change. A second block here
      # would be dead, and the mutation test refuses dead blocks.
      ;;
    *)
      say "authority change: reported on ${EVENT_NAME}; it was approved on its pull request" ;;
  esac
elif [ "$trusted" = no ]; then
  say "authority change: this candidate adopts the gate; the base carries no policy, so it is reported"
fi

# A ratification is named, and needs the owner's exception on a pull request
# or a queue entry (revision 12, spec 024).
if [ "$ratification" = yes ]; then
  say "ratification: this candidate moves a spec to approved:"
  say "$(cat "$work/ratified")"
  case "$EVENT_NAME" in
    pull_request)
      say "ratification: the owner exception is required for this run" ;;
    merge_group)
      if [ "$recorded" = no ] && recorded_review; then
        recorded=yes
      fi
      if [ "$recorded" = yes ]; then
        if [ "$recorded_exception" = success ]; then
          say "ratification: admitted by the owner exception recorded for #${pr} at ${pr_head}"
        else
          block "the queued group ratifies a spec and the owner exception recorded for #${pr} was not approved (it ended '${recorded_exception}')"
        fi
      fi
      # An unread record has already blocked with its reason, as above.
      ;;
    *)
      say "ratification: reported on ${EVENT_NAME}; it was approved on its pull request" ;;
  esac
fi

# Revision 19 (spec 060): a blocked gate names what failed. Best effort and
# read-only: it never changes the verdict or the exit code. Job names, step
# names and log lines are the candidate's text, so they go only to the step
# summary inside a fence they cannot close, and to annotations with the
# workflow-command escapes; never raw to standard output.
annotate() {
  local title="$1" message="$2"
  message="${message//'%'/%25}"
  message="${message//$'\r'/%0D}"
  message="${message//$'\n'/%0A}"
  title="${title//'%'/%25}"
  title="${title//$'\r'/%0D}"
  title="${title//$'\n'/%0A}"
  title="${title//:/%3A}"
  title="${title//,/%2C}"
  printf '::error title=%s::%s\n' "$title" "$message"
}

failure_summary() (
  set +eE
  trap - ERR
  run="${GITHUB_RUN_ID:-}"
  attempt="${GITHUB_RUN_ATTEMPT:-1}"
  if [ -z "${REPO:-}" ] || [ -z "${GH_TOKEN:-}" ] || [ -z "$run" ]; then
    say "failure summary unavailable: REPO, GH_TOKEN or GITHUB_RUN_ID is not set"
    return 0
  fi
  if ! gh api "repos/${REPO}/actions/runs/${run}/attempts/${attempt}/jobs?per_page=100" > "$work/jobs.json" 2> /dev/null; then
    say "failure summary unavailable: the jobs of run ${run} could not be read"
    return 0
  fi
  if ! jq -r '.jobs[] | select(.conclusion == "failure" and .name != "ci-gate")
      | [.id, (.name | gsub("[\t\r\n]"; " ")),
         ([(.steps // [])[] | select(.conclusion == "failure") | .name][0] // "" | gsub("[\t\r\n]"; " "))]
      | @tsv' "$work/jobs.json" > "$work/failed.all" 2> /dev/null; then
    say "failure summary unavailable: the jobs record of run ${run} did not parse"
    return 0
  fi
  head -n 20 "$work/failed.all" > "$work/failed"
  if [ ! -s "$work/failed" ]; then
    say "failure summary: no other job of run ${run} ended failure"
    return 0
  fi
  : > "$work/report"
  annotations=0
  while IFS=$'\t' read -r id name step; do
    lines=""
    if gh api "repos/${REPO}/actions/jobs/${id}/logs" > "$work/log" 2> /dev/null; then
      lines="$(sed -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/^[0-9]\{4\}-[0-9-]*T[0-9:.]*Z //' "$work/log" \
        | grep -E '##\[error\]|error\[|error:|panicked at|FAILED|not ok|BLOCK:' \
        | cut -c1-300 | head -n 5)"
    else
      lines="(the log could not be read)"
    fi
    {
      printf 'job: %s\n' "$name"
      printf 'failed step: %s\n' "${step:-(none recorded)}"
      if [ -n "$lines" ]; then printf '%s\n' "$lines" | sed 's/^/  /'; fi
      printf '\n'
    } >> "$work/report"
    if [ "$annotations" -lt 10 ]; then
      first="$(printf '%s\n' "$lines" | head -n 1)"
      annotate "$name" "${first:-failed at step: ${step:-(none recorded)}}"
      annotations=$((annotations + 1))
    fi
  done < "$work/failed"
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    # A fence longer than any run of backticks in the report.
    longest="$(grep -o '`*' "$work/report" | awk '{ if (length($0) > m) m = length($0) } END { print m + 0 }')"
    fence="$(printf '%*s' "$(( longest < 3 ? 3 : longest + 1 ))" '' | tr ' ' '`')"
    {
      printf '### Failed jobs\n\n%s\n' "$fence"
      cat "$work/report"
      printf '%s\n\n' "$fence"
    } >> "$GITHUB_STEP_SUMMARY"
  fi
  say "failure summary: $(wc -l < "$work/failed" | tr -d ' ') failed job(s), listed in the step summary and as annotations"
)

if [ "$blocked" -ne 0 ]; then
  failure_summary || true
  say "ci-gate: blocked"
  exit 1
fi
say "ci-gate: passed"
