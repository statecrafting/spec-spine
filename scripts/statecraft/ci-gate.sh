#!/usr/bin/env bash
# Rendered by Statecraft from profile github-actions-rust revision 13.
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

# The authority set is the base's, and the candidate's changes are read from
# its fork point: three dots, as coupling reads them. Two would also list
# what the base changed after the branch was cut, which is not this
# candidate's authority change.
authority=no
if [ "$trusted" = yes ]; then
  {
    jq -r '.files[].path, (.parameters.authored_content // empty)' "$work/policy.json"
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
  recorded_exception="$(gh api "repos/${REPO}/actions/runs/${run}/jobs?per_page=100" 2> /dev/null \
    | jq -r '[.jobs[]? | select(.name == "review-exception")][0].conclusion // "absent"' 2> /dev/null || echo absent)"
  say "recorded review: #${pr} at ${pr_head}, run ${run}: ${recorded_result} (exception: ${recorded_exception})"
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
        if [ "$exception_rule" = owner-exception ]; then rule=required; else rule=inapplicable; fi ;;
      *) rule=inapplicable ;;
    esac
    # A findings verdict is the final approver's refusal: only the owner's
    # exception, approved for this run, lets the head through.
    if [ "$exception_rule" = owner-exception ] && [ "$review_result" = findings ] && [ "$result" != success ]; then
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
          if [ "$recorded_exception" != success ]; then
            block "the review recorded for #${pr} returned findings and its owner exception was not approved (it ended '${recorded_exception}')"
            continue
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

if [ "$blocked" -ne 0 ]; then
  say "ci-gate: blocked"
  exit 1
fi
say "ci-gate: passed"
