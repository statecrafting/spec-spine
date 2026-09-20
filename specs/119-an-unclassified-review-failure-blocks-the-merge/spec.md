---
id: "119-an-unclassified-review-failure-blocks-the-merge"
title: "An unclassified review failure blocks the merge"
status: draft
kind: "governance"
created: "2026-09-20"
implementation: complete
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "073-a-workflow-bump-is-not-a-governed-change"
  - "116-shepherd-reads-every-reviewer"
establishes:
  # 3.8: the policy test. It executes the workflow's own `run:` scalars under
  # offline stubs and asserts the class, the step outputs, the exit status and
  # the publication attempt. Its own file rather than more of `kit_gate.rs`,
  # because what it asserts is a property of the AI review job and every case
  # needs a stubbed `claude` and `gh` on PATH.
  - { kind: file, path: "crates/spec-spine-core/tests/ai_review_policy.rs" }
references:
  # The classifier this spec governs the behavior of. Non-owning on purpose
  # (spec 034, D-1): the file's own header records the decision not to put a
  # C-001 ratchet on every `CLAUDE_CLI_VERSION` or `DIFF_SIZE_CAP` edit, and
  # nothing in this defect requires reversing it.
  - { unit: { kind: file, path: ".github/workflows/ai-pr-review.yml" }, role: exemplar }
  # The terminal gate that reads the job's conclusion.
  - { unit: { kind: file, path: ".github/workflows/ci.yml" }, role: context }
summary: >
  The AI review job classifies a failed reviewer invocation into two outcomes,
  and its fall-through is a pass. On 2026-09-20 the provider returned "Your
  organization does not have access to Claude. Please login again or contact
  your administrator." and the job matched it against a token allowlist
  (`authentication_error`, `permission_error`, `unauthorized`, `forbidden`,
  and three more), found no match, set `review_status=api_failure`, exited 0,
  and `ci-gate` went green on PR #267 with no review. The refusal was prose and
  the allowlist reads tokens, so an access refusal took the transient-outage
  branch. Two further holes sit beside it: `review_status=ok` is set from exit
  status alone, so a reviewer that exits 0 having written nothing publishes an
  empty review the gate reads as reviewed, and the transient notice swallows a
  failed `gh pr comment` with a warning, so the one artifact that makes a skip
  visible can be absent while the gate still passes. This spec states the
  classification policy the job must implement: success and failure are
  separated by exit status before any text is read, refusal signals outrank
  transient signals inside a failed invocation, transient signals are
  recognized only in the contextual forms their emitting layer produces, and
  everything else blocks. It is defended by a test that executes the workflow's
  own script rather than a copy of its patterns, so inverting a blocking branch
  is caught even when its regexes are untouched.
---

# 119: An unclassified review failure blocks the merge

## 1. Purpose

### 1.1 What was observed

PR #267, CI run 35496672060, two attempts, both `success`. The `ai-review` job
captured, identically on both attempts:

```
claude exited 1; captured output follows:
Your organization does not have access to Claude. Please login again or contact your administrator.
```

The classification path is `.github/workflows/ai-pr-review.yml:283-297`. The
invocation exits 1; the allowlist at `:286` is

```
authentication_error|permission_error|invalid[^a-z]*(x-)?api[^a-z]*key|unauthorized|forbidden|oauth[^a-z]*(token)?[^a-z]*(invalid|expired|revoked|missing)
```

and the captured prose matches none of it. Control falls to `:293-297`, which
writes `review_status=api_failure`, emits a `::warning::`, and leaves the step
exit status at 0. The notice step posts "AI Code Review: SKIPPED (Claude API
failure)", `ci-gate` reads the job conclusion as `success` (`ci.yml:176-183`),
and the PR merges with no review.

Two failures 4.5 minutes apart on different runners produced the same message.
That is repetition, not proof of persistence, and it does not rule out a
temporary provider-side problem. The logs establish that an organization-access
refusal **was returned**. They do not establish why. This spec asserts nothing
about the cause, and no requirement below depends on one.

### 1.2 The misclassification

> The classifier routes an access refusal to the transient-outage branch,
> because its only test for a refusal is a token allowlist that natural-language
> refusals do not match.

That is true regardless of why the refusal was issued, and it would be true if
the refusal had been transient. The correction is not "recognize this one
sentence". It is to invert the default: an invocation failure the job cannot
positively classify as transient must block, because a pass that cannot say why
it passed is a silent green.

### 1.3 Two adjacent holes, in scope

**`review_status=ok` is set from exit status alone.** `:280` sets it inside the
`if` branch of the invocation, and nothing reads `/tmp/review.md`. The post step
at `:301-304` carries a comment stating that it "must run only when Run AI
Review succeeded AND actually produced a review, so a partial/empty review is
never posted", and its condition is `success() && steps.diff.outputs.skip !=
'true' && steps.aireview.outputs.review_status == 'ok'`. No conjunct tests the
body. A reviewer exiting 0 having written nothing posts a comment headed
`## AI Code Review` with an empty body, and the gate reads it as reviewed. The
prose asserts a guard the code does not implement.

**A failed notice publication is swallowed.** `:345` ends the transient notice
with `|| echo "::warning::could not post API-failure notice comment"`. The one
artifact that distinguishes a visible skip from a silent green can therefore be
absent while the job still exits 0.

### 1.4 What this spec does not fix

Whatever caused the organization-access refusal is account-side. It is not a
repository change, it is not proposed here, and landing this spec does not
restore provider access. Conversely, restoring provider access does not close
this defect: the misclassification is in the code and survives the incident.

## 2. Territory

This spec **establishes** one new file:

- `crates/spec-spine-core/tests/ai_review_policy.rs`, the policy test of 3.8.
  Declared `planned: true` while the spec was filed, and claimed outright once
  the build wrote it. It lives inside the
  `spec-spine-core` package and is hashed as package source, so L-008 is
  satisfied with no new `[index] extra_hashed_inputs` glob.

This spec **references, and does not claim**:

- `.github/workflows/ai-pr-review.yml`, role `exemplar`. The classification
  behavior of section 3 is a behavior *of* this file, and section 3 is binding
  on it, but the file is not a unit of this spec. See D-1.
- `.github/workflows/ci.yml`, role `context`. The `ci-gate` aggregation that
  turns a job conclusion into a merge decision.

A `references` edge is the corpus's only non-owning edge (spec 034): the
coupling gate ignores it. Naming the workflow here records that this spec is
about it, and claims nothing.

### 2.1 What coupling and a behavioral test each protect

These are different protections and this spec deliberately takes only the
second. Stating the difference is part of the territory, because the choice
leaves a real hole and the hole should not be discovered later.

**The coupling gate protects co-change.** C-001 asks one question: when a
spec-claimed path changed, did that spec's `spec.md` change in the same PR? It
is indifferent to what either change says. A dated decision entry paired with a
regex rewrite that inverts the policy satisfies it completely. It catches the
edit that forgot the spec; it cannot catch the edit that remembered the spec and
broke the rule.

**A behavioral test protects the rule.** The test of 3.8 executes the real
classification control flow over fixed inputs and asserts the resulting class,
the step outputs, the exit status, and the publication attempt. Inverting a
blocking branch turns it red whatever the accompanying spec edit says and
whatever the regexes look like. It runs inside `cargo test --workspace`, which
`ci-gate` already requires.

**The consequence.** Under this spec a contributor may edit
`.github/workflows/ai-pr-review.yml` with no spec edit and no waiver. What they
may not do is edit it into a state 3.8 refuses. Behavior that 3.8 does not
assert is protected by neither mechanism, which makes 3.8's fixture coverage the
real boundary of this spec rather than a test detail.

## 3. Behavior

### 3.1 Success and failure are separated before any text is read

The job MUST decide between the success branch and the failed-invocation branch
on the reviewer invocation's **exit status alone**, before any diagnostic
pattern is consulted.

```
structural skip?            -> class 1, SKIP, gate green        (3.2)
secret unset?               -> class 2, FAIL                    (3.3)
invoke -> rc, review body (stdout), diagnostics (stderr)

rc == 0:
    review body has a non-whitespace character  -> class 3, PASS   (3.4)
    otherwise                                   -> class 6, FAIL   (3.7)
rc != 0:
    diagnostics match a refusal signal (3.5)    -> class 4, FAIL
    diagnostics match a transient signal (3.6)  -> class 5, SKIP, gate green
    otherwise, including empty diagnostics      -> class 6, FAIL   (3.7)
```

Classes 4, 5 and 6-on-failure MUST be unreachable when `rc == 0`. A review whose
own text discusses authentication, a 403 handler, billing code, or any other
term that appears in 3.5 or 3.6 MUST be class 3. There MUST be no pattern that
can reclassify a successful invocation as a provider failure.

The job MUST read stdout as a review only when `rc == 0`, and as diagnostics
only when `rc != 0`. When `rc != 0` the two streams are both diagnostics and MAY
be concatenated, as `:284` does today, because there is then no review to
confuse them with.

The step's exit status MUST be 1 for classes 2, 4 and 6, and 0 for classes 1, 3
and 5, subject to 3.4 and 3.6's publication conditions.

### 3.2 Class 1, structural skip, unchanged

Decided from event and PR metadata before the CLI runs: the event is not
`pull_request`, the PR is a draft, the PR is from a fork, the PR is from
Dependabot, or the reviewable diff exceeds `DIFF_SIZE_CAP`. These MUST keep the
behavior they have today, including their notices and, for the fork and
Dependabot notices, their `|| echo "::warning::..."` fallbacks. Section 5.1 of
this spec states why. No new comment requirement is imposed on a skip where the
job never starts.

### 3.3 Class 2, secret unset, unchanged

`[ -z "$CLAUDE_CODE_OAUTH_TOKEN" ]` on the path where the secret is expected to
exist MUST emit `::error::` and exit 1, as it does today at `:235-238`.

### 3.4 Class 3, a review was produced and published

`rc == 0` AND the review body contains at least one non-whitespace character.
The job MUST publish the review comment, and a failed publication MUST fail the
step. The post step's condition MUST additionally test the body, so that the
guard its own comment at `:301-304` describes is the guard it implements.

**What class 3 establishes, and what it does not.** It establishes output
presence: the reviewer ran, exited 0, wrote something, and the something was
published. It does **not** establish that the output is a review, that the
review is correct, that it read the diff, or that its findings are sound. No
text in this repository may describe a class 3 green as semantic proof of review
quality.

### 3.5 Class 4, access refusal

`rc != 0` AND the diagnostics carry an explicit refusal of access or credit.

The category name describes the **observed signal**, not a cause. The spec text,
the annotation, and any comment the job posts MUST say that the provider refused
the invocation and MUST NOT assert why. "The provider refused this invocation" is
sayable. "Your subscription lapsed", "the account is out of credit" and "the
seat was revoked" are not: the logs do not establish any of them.

Recognized signals, matched case-insensitively over the whole diagnostic text:

- the CLI's own status form for a refusal status: `API Error: 401`,
  `API Error: 402`, `API Error: 403`
- a JSON error-type field naming a refusal type, allowing optional whitespace
  around the colon and either quote style: `"type": "authentication_error"`,
  `"type": "permission_error"`
- explicit refusal prose: `does not have access`, `no longer has access`,
  `not authorized`, `unauthorized`, `access denied`, `permission denied`,
  `forbidden`, `contact your administrator`, `please log in again`,
  `please login`, `invalid api key`, `invalid x-api-key`, and `oauth token`
  adjacent to one of `invalid`, `expired`, `revoked`, `missing`
- explicit credit refusal: `credit balance is too low`, `payment required`,
  `insufficient credit`

The following MUST NOT be recognized on their own: `seat`, `billing`,
`subscription`, `entitlement`, `upgrade your plan`, and a bare `/login`. Each is
a topic word rather than a refusal, and each appears in ordinary prose about the
subject it names. A refusal is recognized by refusal language. `please log in
again` and `please login` are kept because they are imperatives addressed to the
operator, and they are the second sentence of the captured message.

Class 4 MUST emit `::error::` and exit 1. It MUST NOT post a PR comment: a
failing job is already visible, and a comment asserting a cause is exactly what
this section forbids.

### 3.6 Class 5, recognized transient

`rc != 0` AND the diagnostics carry a transient signal **in the contextual form
its emitting layer produces**. A bare HTTP number and a bare generic phrase are
NOT transient signals: three digits match a line number quoted back in an error
or a path in a diff, and `fetch failed` alone says nothing about what failed.

- **Provider status**, only in the CLI's own status form: `API Error: 429`,
  `API Error: 500`, `API Error: 502`, `API Error: 503`, `API Error: 504`,
  `API Error: 529`.
- **Provider error type**, only as the JSON error-type field, with optional
  whitespace around the colon and either quote style:
  `"type": "overloaded_error"`, `"type": "rate_limit_error"`,
  `"type": "api_error"`. The bare token in prose MUST NOT match.
- **Network layer**, only as one of `ECONNRESET`, `ETIMEDOUT`, `ENOTFOUND`,
  `EAI_AGAIN`, `ECONNREFUSED` appearing in the same diagnostic line as one of
  `fetch failed`, `socket hang up`, `request to`, or the provider host
  `api.anthropic.com`.

There is no timeout signal. See 3.6.1.

Bare `quota exceeded` is in neither 3.5 nor 3.6 and is therefore class 6. The
phrase reads as a rate limit in one product and an entitlement cap in another,
and nothing in the diagnostics settles which. Neither an account cause nor a
transient cause may be inferred from it. Diagnostics carrying `quota exceeded`
**and** a recognized signal are classified by the recognized signal.

Class 5 MUST emit `::warning::`, publish the skip notice, and exit 0 **only if
the notice published**. A failed `gh pr comment` on this step MUST fail the
step. The `|| echo "::warning::could not post API-failure notice comment"`
swallow at `:345` MUST be removed. The notice is the entire difference between a
visible skip and a silent green; a notice that failed to post is not a visible
skip.

#### 3.6.1 A timeout is not a recognized transient signal

A diagnostic containing `timeout`, `timed out` or `deadline exceeded` is NOT a
transient signal, and neither is a bare exit status. Exit `124` and exit `137`
are conventionally produced by `timeout(1)`, but they are ordinary statuses a
child process can return on its own: `124` is whatever the reviewer chose to
exit with, and `137` is `128 + SIGKILL`, which the OOM killer, a supervisor, or
the child itself can produce. Neither status, on its own, establishes that a
bound was exceeded rather than that the invocation failed for an unknown
reason, so neither may be read as evidence of a transient condition. Under 3.7
they are class 6 and block.

The only timeout-shaped evidence this policy recognizes is a network errno in
the contextual form of 3.6, which covers `ETIMEDOUT` at the transport layer.
That signal is produced by the transport, not by the child's choice of status.

**No per-invocation wrapper is added.** The bound that exists is the job-level
`timeout-minutes: 10`. It cancels the whole job, produces no classification, and
is read by `ci-gate` as `cancelled`, which already fails the gate
(`ci.yml:179-183`). That outcome is blocking and is preserved unchanged. An
earlier draft of this spec required the build to add a `timeout(1)` wrapper and
an `AI_REVIEW_TIMEOUT` bound, and to treat exit `124` as sufficient timeout
evidence; both requirements are withdrawn by D-6 and D-10, because the status
the fixture would assert on is one the child can return without any wrapper
having fired.

### 3.7 Class 6, unclassified, blocks

Everything the branches above do not positively classify:

- `rc != 0` with diagnostics matching neither 3.5 nor 3.6
- `rc != 0` with empty or whitespace-only diagnostics
- `rc == 0` with an empty or whitespace-only review body

Class 6 MUST emit `::error::` and exit 1. This is the inversion the spec exists
for. Today's fall-through passes, which is how the captured refusal became
green, and it is also how a non-zero exit with no captured output passes today
with the excerpt `claude CLI exited N with no captured output`. No evidence is
not evidence of an outage.

### 3.8 The policy test executes the workflow's own script

`crates/spec-spine-core/tests/ai_review_policy.rs` MUST assert 3.1 through 3.7
by **running the workflow's own `run:` scalars**, not by extracting its patterns
and rebuilding a classifier. A rebuilt classifier asserts a property of its own
copy: the two drift, and the copy stays green while the workflow rots. It also
cannot observe exit statuses, step outputs or publication, which is where this
defect lives.

The harness MUST:

1. Parse `.github/workflows/ai-pr-review.yml` with `serde_yaml`, as
   `kit_gate.rs::workflow_steps` already parses `kit/govern.yml`.
2. Locate a step by its `name` and take its `run:` scalar verbatim. Only the
   steps a case actually exercises are selected: the classification step and
   the publication step its outputs select. Steps a case does not reach are not
   executed and are not emulated.
3. Execute each selected step's `run:` scalar under the shell and
   error-handling semantics the runner applies to that step. A step that
   declares no `shell:` key runs on a GitHub-hosted Linux runner as
   `bash -e {0}`, so the harness MUST execute it as `bash -e <script>`: the
   `-e` is part of what is under test, because a branch that leaves a non-zero
   status uncaught fails the step on the runner and must fail it here. A step
   that declares a `shell:` key MUST be executed under that shell's documented
   flags, and a `shell:` value the harness does not implement MUST panic naming
   the step rather than falling back to a default.
4. Resolve, from fixture context and from actual classifier outputs, exactly
   the environment values and conditions the selected steps need. Job-level
   `env:` is applied first and the step's `env:` over it. A plain scalar is
   taken verbatim. A `${{ }}` expression is resolved from the fixture's PR
   context (`github.*`), from a synthetic non-credential literal
   (`secrets.*`), or from the output the classification step actually wrote
   (`steps.<id>.outputs.<name>`) rather than from a value the fixture asserts
   it ought to have written. The harness additionally sets `GITHUB_OUTPUT` and
   `GITHUB_ENV` to temp files it parses back, `RUNNER_TEMP` and the review
   scratch root inside the per-case `tempfile::TempDir`, and
   `CLAUDE_CODE_OAUTH_TOKEN` to a non-credential literal (unset for the class 2
   case).
5. Fail loudly on any relevant expression it does not support. An unsupported
   `${{ }}` form in an `env:` value a selected step needs, an unsupported `if:`
   form on a step a case must decide, or an unsupported `shell:` on a step a
   case must run, MUST panic naming the step, the key and the expression. A
   silent default here would let a new condition go unevaluated while the suite
   stayed green, which is the failure mode 2.1 says the test is the only
   defense against.

   **This is not a GitHub Actions emulator and MUST NOT become one.** Contexts
   the selected steps do not read, `uses:` steps, matrix and strategy,
   `continue-on-error`, expression functions beyond the ones the job contains,
   and job-level `if:` evaluation are all out of scope. The supported surface is
   defined by what this job actually writes; anything past it is rule 5's loud
   failure, not a feature to add.
6. Cover publication by evaluating each publication step's `if:` against the
   outputs the classification step produced and the fixture's context, then
   executing the selected step's `run:` under the same stubs and the same shell
   semantics. Exactly one publication step may be selected per case, or none;
   more than one is a harness failure.
7. Assert, per fixture: the classification step's exit status, the
   `review_status` output and `api_rc` / `api_excerpt` where set, which
   publication step was selected and its exit status, whether the `gh` stub was
   invoked and with what body, the resulting job outcome, and whether the
   annotation is `::error::` or `::warning::`.
8. Make no provider call, no GitHub call, and no network call, write nothing
   outside its TempDir, and use no credential. The stubs are the only `claude`
   and `gh` on `PATH`, so a real call fails the case rather than escaping it.

**A prerequisite the build must carry.** The step writes `/tmp/review.md`,
`/tmp/review.err` and `/tmp/review-input.txt` as absolute paths, so two parallel
cases executing it would collide. The build MUST parameterize the review scratch
root through an environment variable defaulting to the present behavior
(`AI_REVIEW_TMP`), so each case is isolated and the workflow's runtime behavior
is unchanged.

**The job outcome is derived, not asserted separately.** A GitHub job fails when
any step fails. The harness therefore computes the job outcome from the
classification step's status and the selected publication step's status, and the
matrix states all three, because the defect this spec exists for is precisely a
step that succeeded while the thing it was supposed to make visible did not
happen.

#### 3.8.1 Fixture matrix

`rc` is the reviewer invocation's exit status; `body` is its stdout; `diag` is
its stderr. The three result columns are distinct and are asserted separately:
**Clf** is the exit status of the `Run AI Review` classification step, **Pub**
is the publication step the outputs select and its exit status, and **Job** is
the resulting job outcome, which fails if either step failed. A case whose
classification succeeded and whose publication failed MUST still fail the job.

| # | rc | body | diag | Class | `review_status` | Clf | Pub | Job |
|---|---|---|---|---|---|---|---|---|
| 1 | 1 | empty | `Your organization does not have access to Claude. Please login again or contact your administrator.` (verbatim, PR #267) | 4 | unset | 1, `::error::` | none | FAIL |
| 2 | 1 | empty | `API Error: 401 {"type":"authentication_error","message":"invalid x-api-key"}` | 4 | unset | 1, `::error::` | none | FAIL |
| 3 | 1 | empty | `API Error: 403 {"type":"permission_error","message":"..."}` | 4 | unset | 1, `::error::` | none | FAIL |
| 4 | 1 | empty | `Credit balance is too low` | 4 | unset | 1, `::error::` | none | FAIL |
| 5 | 1 | empty | `API Error: 529 {"type":"overloaded_error","message":"Overloaded"}` | 5 | `api_failure` | 0, `::warning::` | notice, 0 | PASS |
| 6 | 1 | empty | `API Error: 429 {"type":"rate_limit_error","message":"..."}` | 5 | `api_failure` | 0, `::warning::` | notice, 0 | PASS |
| 7 | 1 | empty | `API Error: 503 Service Unavailable` | 5 | `api_failure` | 0, `::warning::` | notice, 0 | PASS |
| 8 | 1 | empty | `fetch failed: ETIMEDOUT connecting to api.anthropic.com` | 5 | `api_failure` | 0, `::warning::` | notice, 0 | PASS |
| 9 | 1 | empty | `fetch failed` (no errno, no host) | 6 | unset | 1, `::error::` | none | FAIL |
| 10 | 1 | empty | `529` (bare number, no `API Error:` form) | 6 | unset | 1, `::error::` | none | FAIL |
| 11 | 1 | empty | `quota exceeded` (bare) | 6 | unset | 1, `::error::` | none | FAIL |
| 12 | 1 | empty | `quota exceeded` and `API Error: 429 {"type":"rate_limit_error"}` | 5 | `api_failure` | 0, `::warning::` | notice, 0 | PASS |
| 13 | 1 | empty | `API Error: 529 overloaded_error` and `Your organization does not have access` | 4 | unset | 1, `::error::` | none | FAIL |
| 14 | 1 | empty | `Segmentation fault` | 6 | unset | 1, `::error::` | none | FAIL |
| 15 | 1 | empty | empty | 6 | unset | 1, `::error::` | none | FAIL |
| 16 | 1 | empty | whitespace only | 6 | unset | 1, `::error::` | none | FAIL |
| 17 | 1 | empty | `The request timed out` (no mechanical evidence) | 6 | unset | 1, `::error::` | none | FAIL |
| 18 | 124 | empty | empty | 6 | unset | 1, `::error::` | none | FAIL |
| 19 | 137 | empty | empty | 6 | unset | 1, `::error::` | none | FAIL |
| 20 | 124 | empty | `The operation timed out after 300 seconds` | 6 | unset | 1, `::error::` | none | FAIL |
| 28 | 1 | empty | `ETIMEDOUT` (errno alone, no transport context) | 6 | unset | 1, `::error::` | none | FAIL |
| 29 | 1 | empty | `ETIMEDOUT` and `fetch failed` on separate lines | 6 | unset | 1, `::error::` | none | FAIL |
| 21 | 0 | `## Findings\n\nNone.\n` | empty | 3 | `ok` | 0 | review, 0 | PASS |
| 22 | 0 | empty | empty | 6 | unset | 1, `::error::` | none | FAIL |
| 23 | 0 | whitespace only | empty | 6 | unset | 1, `::error::` | none | FAIL |
| 24 | 0 | `## Findings\n\nThe new handler returns 403 on an expired OAuth token and the billing path is unauthorized without a seat check.` | empty | 3 | `ok` | 0 | review, 0 | PASS |
| 25 | 1 | empty | `API Error: 529 {"type":"overloaded_error"}`; `gh` stub exits 1 | 5 | `api_failure` | **0** | notice, **1** | **FAIL** |
| 26 | 0 | `## Findings\n\n...`; `gh` stub exits 1 | empty | 3 | `ok` | **0** | review, **1** | **FAIL** |
| 27 | not invoked | | `CLAUDE_CODE_OAUTH_TOKEN` unset | 2 | unset | 1, `::error::` | none | FAIL |

Row 24 is 3.1's guarantee: its review text carries `403`, `OAuth token`,
`unauthorized`, `billing` and `seat`, and is class 3 because `rc` is 0. Rows 12
and 13 are the two precedence rules of 3.9. Rows 9, 28 and 29 are the three cases of 3.6's
contextual-form rule: a transport phrase with no errno, an errno with no
transport phrase, and both present but on separate lines each fail to classify,
because only their conjunction on one line is a signal. Rows 18, 19 and 20 are the negative
timeout fixtures of 3.6.1: `124` and `137` are statuses a child can return on
its own, so without a recognized diagnostic signal they block, and row 20 adds
timeout-sounding prose to show that the combination does not rescue them
either. Rows 25 and 26 are the publication conditions of 3.4 and 3.6, and they
are the rows that carry this spec's central distinction: the classification step
exits 0 in both, and the job still fails, because publication failed.

#### 3.8.2 The inversion regression check

A case MUST detect the substitution of a passing verdict into a blocking branch
**even when that branch's patterns are unchanged**.

It MUST take the classification step's `run:` scalar, rewrite the class 4
branch's terminator from a failing exit to a passing one, leaving every regex
byte untouched, execute the mutant against fixture row 1 under the same stubs,
and assert that the observed exit status diverges from the policy's required
exit status. The case passes only when that divergence is detected.

What this establishes: the suite's refusal is anchored on observed exit status
and outputs, not on the presence of a pattern. A test that grepped for the
regexes would pass the mutant. The mutation MUST be applied to an in-memory copy;
the test MUST NOT write the workflow on disk.

### 3.9 Precedence

In force order:

1. **Structural skip outranks everything.** Class 1 is decided before the CLI
   runs. With no invocation there is no exit status and no diagnostics, and
   classes 3 to 6 do not apply.
2. **Secret unset outranks invocation.** Class 2, on the path where the secret
   is expected to exist.
3. **Exit status selects the branch.** `rc == 0` is the success branch, classes
   3 and 6. `rc != 0` is the failed-invocation branch, classes 4, 5 and 6. No
   diagnostic pattern crosses between them.
4. **Within a failed invocation, refusal outranks transient.** Diagnostics
   matching both 3.5 and 3.6 are class 4 and the job fails. A 529 that also says
   "does not have access" is not an outage that happens to mention
   organizations. The safe reading of an ambiguous signal is the one that cannot
   produce a silent green.
5. **Absence of evidence is class 6.** Empty or whitespace-only diagnostics at
   `rc != 0` fail; an empty or whitespace-only body at `rc == 0` fails.
6. **Matching is over the whole diagnostic text**, not the last line, so a
   transient retry banner followed by a final refusal reaches class 4 by rule 4.

### 3.10 Acceptance

| AC | Requirement | Evidence |
|---|---|---|
| AC-1 | The policy test exists and passes | `cargo test -p spec-spine-core --test ai_review_policy` |
| AC-2 | The test executes the real workflow, not a copy | the test file names `.github/workflows/ai-pr-review.yml` |
| AC-3 | The inversion regression case exists and passes | the named case runs and reports one pass |
| AC-4 | The transient notice no longer swallows a publication failure | the swallow string is absent from the workflow |
| AC-5 | The fork and Dependabot annotation fallbacks are preserved | both fallback strings are present |
| AC-6 | No timeout wrapper and no `AI_REVIEW_TIMEOUT` bound are introduced | neither string appears in the workflow |
| AC-7 | The review scratch root is parameterized | the workflow names `AI_REVIEW_TMP` |
| AC-8 | The corpus still validates and the gate still holds | `lint --fail-on-warn`, `check` |
| AC-9 | A publication failure fails the job even when classification succeeded | the named publication-failure cases run and pass |
| AC-10 | The post step refuses an empty review the classifier let through | the named post-step-condition case runs and passes |

AC-1, AC-2, AC-3, AC-4, AC-7, AC-9 and AC-10 fail against the tree this spec is
filed on. AC-5, AC-6 and AC-8 pass now and after: they are preservation assertions,
and they go red only if the build breaks something it was told to keep. AC-6 is
a preservation assertion in the same sense: after D-10 the wrapper is something
the build must *not* add, so the assertion holds on the filed tree and must
still hold on the built one.

## 4. Out of scope

- **Why the provider refused access.** Account-side, unestablished by the logs,
  and outside any repository change. See 1.4.
- **Claiming `.github/workflows/ai-pr-review.yml`.** D-1.
- **Retry or backoff.** A recognized transient skips visibly; it is not retried
  in-job. Re-running the job is the operator's move and is what the notice says.
- **Any judgement of review quality.** 3.4 establishes output presence only.
  Whether the reviewer's findings are sound is a human question this spec does
  not touch, and spec 116's reviewer-reading discipline is where that lives.
- **The other workflows.** `ci.yml` and `determinism.yml` keep their present
  classification behavior, which is exit-status-only and has no text branch.
- **A per-invocation timeout wrapper.** Withdrawn by D-10. The job-level
  `timeout-minutes: 10` and its blocking `cancelled` outcome are preserved.
- **A general GitHub Actions expression evaluator.** 3.8 rule 5. The harness
  supports the forms this job contains and panics on anything else.
- **The kit.** Adopters do not ship this workflow; nothing under `kit/` changes.
- **Changing what `ci-gate` does with a `skipped` job.** Unchanged (`ci.yml:176-183`).

## 5. Resolved decisions

`D-1 (2026-09-20, the workflow is referenced, not established).` Three
mechanisms were available: establish the workflow file, establish a policy test
and reference the workflow, or place the rule in the standards tier. The second
is taken. The workflow's own header at `:1-3` records the existing decision not
to claim the file, and nothing in this defect requires reversing it: claiming it
would put a C-001 ratchet on every `CLAUDE_CLI_VERSION` and `DIFF_SIZE_CAP`
edit. The decisive argument is in 2.1: C-001 only demands that *some* edit land
in the owning spec, and a dated decision entry satisfies it while changing no
requirement, so file ownership would not in fact defend the classification rule,
while a behavioral test does. The standards tier is declined because the
anti-silent-green property is already downstream of spec 000's `refusal-rule`
anchor; an ordinary spec implements it and does not need constitutional text.

`D-2 (2026-09-20, bare "quota exceeded" blocks).` It is in neither recognized
list. The phrase reads as a rate limit in one product and an entitlement cap in
another; the diagnostics do not settle which, and neither an account cause nor a
transient cause may be inferred from it. Under 3.7 an unclassified failure
blocks, and that is the fail-safe reading.

`D-3 (2026-09-20, the category names the signal, never the cause).` Class 4 is
"access refusal", not "account-state refusal" or "expired subscription". The
observable fact is that the provider refused the invocation. Everything past that
is inference the logs do not support, and a diagnostic that asserts a cause sends
an operator to the wrong place.

`D-4 (2026-09-20, broad account keywords removed).` An earlier draft of this
policy recognized `seat`, `billing`, `subscription`, `entitlement` and
`upgrade your plan` as refusal signals. They are topic words. Combined with the
earlier draft's rule that text outranked exit code, a successful review
discussing billing code would have been classified as a provider failure. 3.1
removes the crossing structurally and 3.5 removes the keywords; both corrections
are kept, because either alone leaves the other half of the mistake in place.

`D-5 (2026-09-20, transient signals are contextual, not bare).` An earlier draft
claimed its transient list held only machine tokens, HTTP numbers and POSIX
errnos. That was false: `fetch failed`, `socket hang up` and
`internal server error` are natural-language phrases, and a bare number matches
any three digits anywhere in the diagnostics. 3.6 requires each signal in the
form its emitting layer produces.

`D-6 (2026-09-20, a timeout needs mechanical evidence).` An earlier draft listed
"an explicit timeout of the CLI invocation" as a transient signal without saying
how it would be detected, which was unimplementable: no timeout wrapper exists
and the job-level bound cancels rather than classifies. The finding stands: a
timeout-sounding diagnostic is not evidence of a timeout. The remedy the same
draft proposed, a `timeout(1)` wrapper whose exit `124` would be read as
evidence, is withdrawn by D-10 later the same day.

`D-7 (2026-09-20, the test runs the workflow's script).` An earlier draft
proposed extracting the classifier's regexes into the test and running them over
fixtures. Declined: that asserts a property of the test's own copy, the two
drift, and it cannot see exit statuses, outputs or publication. 3.8 executes the
step's `run:` scalar verbatim under offline stubs, which costs a small YAML step
harness and an `if:` evaluator and buys an assertion about the thing that
actually runs in CI.

`D-8 (2026-09-20, job-level skips keep their present notices).` The notice
condition of 3.6 applies only to the recognized-transient skip. A non-`pull_request`
event and a draft PR never start the job, so there is no step to post from; a
fork and a Dependabot PR carry a read-only `GITHUB_TOKEN`, so hard-failing on a
comment that cannot post would red-gate every external contribution and every
dependency bump for a permission reason unrelated to the review. Their
`::warning::` annotation keeps the skip visible in the checks UI and is
preserved.

`D-9 (2026-09-20, blocking merges while access is refused is intended).` Under
3.5 the captured refusal fails the job and `ci-gate` fails with it, so while
provider access is refused this repository's PRs do not merge on a green gate.
That is the cost side of 3.7's inversion and it is accepted: a gate that cannot
review is a gate that should not pass. The escape is a human decision, not an
automatic one.

**Scope of D-9.** It governs class 4 and class 6 only, and it is not a statement
that every path to a green gate now requires a completed review. The exceptions
this spec deliberately retains are unaffected: a class 1 structural skip (3.2, a
non-`pull_request` event, a draft PR, a fork PR, a Dependabot PR, an oversized
diff) still passes with its visible notice, and a class 5 recognized transient
(3.6) still passes when its notice published. D-9 says that an unrecognized
failure is not one of those exceptions; it does not withdraw them. Read without
this qualification, D-9's "a gate that cannot review is a gate that should not
pass" would contradict 3.2, 3.6 and D-8, all of which are retained.

`D-10 (2026-09-20, no timeout wrapper; exit 124 and 137 are not evidence).` The
draft required the build to add a `timeout(1)` wrapper with an
`AI_REVIEW_TIMEOUT` bound and carried a fixture treating exit `124` with empty
diagnostics as a recognized transient. Withdrawn. `124` and `137` are ordinary
statuses a child process returns on its own: `124` is whatever the reviewer
chose to exit with, `137` is `128 + SIGKILL` and is produced by the OOM killer,
a supervisor, or the child itself. A fixture asserting that `124` means "a
wrapper timed out" would assert something the status does not establish, and a
wrapper added to make that fixture true would add a mechanism the defect did not
call for. The existing job-level `timeout-minutes: 10` and its blocking
`cancelled` outcome are preserved. Bare exit codes and timeout-sounding text
alike are class 6, and 3.8.1 rows 18, 19 and 20 assert that negatively.

`D-11 (2026-09-20, the test harness is bounded, not general).` 3.8 rule 3
requires the selected steps to run under the runner's own shell and
error-handling semantics (`bash -e` for a step declaring no `shell:`), and rule
4 requires their environment values and conditions to be resolved from fixture
context and from actual classifier outputs rather than from values the fixture
wishes had been written. Rule 5 makes every unsupported *relevant* expression a
loud panic. The alternative considered and declined was a general Actions
emulator: it is a large surface with its own bugs, and every line of it is a
place where the harness could diverge from the runner while staying green. The
bound is "what this job contains"; past that, the harness refuses.

`D-12 (2026-09-20, classification, publication and job outcome are three
results).` The original matrix carried one "Step exit" column and a prose
Publication column, which conflated the classification step's status with the
job's. That conflation is the defect itself in miniature: `review_status=ok`
with a failed `gh pr comment` is exactly a step that succeeded while the thing
it existed to make visible did not happen. 3.8.1 now states **Clf**, **Pub**
and **Job** separately, and rows 25 and 26 are the cases where classification
succeeds, publication fails, and the job MUST fail.

`D-13 (2026-09-20, the inversion acceptance command must not hide a failure).`
The filed acceptance line piped `cargo test` into `grep -q`. A pipeline's status
is the last command's, so a compilation error or a failing case followed by a
matching line anywhere in the output would have been read as a pass, and a
`cargo` binary that never ran the named case could still satisfy the grep. The
line now assigns the output through a command substitution, which carries
`cargo`'s own exit status, and only then greps the captured text for a
`test result: ok.` summary with a non-zero pass count. Both facts are required
and neither can mask the other. This is the same family as the finding recorded
in spec 114's build: a successful filter is not a successful command.

`D-14 (2026-09-20, registry readiness is dependency readiness).` `spec-spine
registry plan` lists this spec as ready. That verdict means only that its
`depends_on` edges (073 and 116) resolve to specs whose implementation is
complete; it is not an approval, a ratification, or a human decision that the
work should proceed. This spec is `status: draft` and stays `draft` through its
build, per this repository's build-before-ratification loop. Nothing in the
ready set authorizes a build, and nothing in this spec should be read as
claiming it does.

`D-15 (2026-09-20, two fixture gaps the build's mutation probes found).` The
matrix as filed could not fail on two real regressions, and both are closed
here rather than left to be discovered later. First, class 4 and class 6 are
observationally identical in exit status, annotation level and publication, so
deleting every refusal pattern would have left rows 1 to 4 green: they would
simply have fallen through to class 6 and blocked for the other reason. The
suite therefore asserts the distinguishing phrase of each class's annotation,
which is the only thing that separates them for an operator. Second, 3.6's
contextual rule has two halves and only one was fixtured: row 9 is a transport
phrase with no errno, and nothing tested an errno with no transport phrase, so
dropping the conjunction would have gone unnoticed. Row 28 supplies it.

`D-16 (2026-09-20, the post step's body conjunct needed its own case).` 3.4
requires the post step's `if:` to test the review body, and the build added the
`review_nonempty` conjunct. A mutation probe then removed that conjunct and the
whole suite stayed green: no fixture can reach it, because the classifier's own
guard (3.7) means `review_status=ok` is never written for an empty body, so the
conjunct is defense in depth that nothing exercised. That is the same defect
shape as D-15's two gaps, and it is closed the same way rather than left for
later. The case reconstructs the pre-119 classifier in memory, exactly as 3.8.2
reconstructs an inverted branch, so that `review_status=ok` is set with no
`review_nonempty` output, and asserts the post step is still not selected and
nothing is published. Removing the conjunct turns that one case red and no
other, which is what makes it an assertion about the conjunct rather than about
the classifier. The mutation is in memory; the workflow on disk is never
written.

`D-17 (2026-09-20, the same-line rule has three cases, not two).` D-15 closed
3.6's contextual-form rule by adding row 28, on the reading that the rule has
two halves: a transport phrase without an errno, and an errno without a
transport phrase. That reading was incomplete. The rule 3.6 states is that the
errno and the context phrase appear **in the same diagnostic line**, so the
third case is both present on *different* lines, and no row covered it. The
implementation pipes one grep into another precisely so that the second reads
only the lines the first emitted, and it is correct; what was missing was
anything that would notice if it stopped being. Rewriting the branch as two
independent full-text greps leaves rows 9 and 28 green and classifies the
two-line input as transient, which is a silent green reached without touching a
single pattern byte. Row 29 supplies the case, and the mutant it was written
against fails on it.

## Verification

Each line runs from the repository root as its own `sh -c`, so no shell state
carries between lines. AC-1 through AC-4, AC-7, AC-9 and AC-10 are red against the
tree this spec is filed on; AC-5, AC-6 and AC-8 are preservation assertions that are
green before and after.

The inversion line does not pipe `cargo` into a filter. A pipeline reports its
last command's status, so `cargo ... | grep -q` would report a passing grep over
the output of a failing build. Instead the output is captured through a command
substitution, whose assignment carries `cargo`'s own exit status, and the
captured text is then searched separately for the summary line proving the named
case actually ran rather than being filtered out to zero tests.

```verify:cli
test -f crates/spec-spine-core/tests/ai_review_policy.rs
cargo test -p spec-spine-core --locked --test ai_review_policy
grep -q '\.github/workflows/ai-pr-review\.yml' crates/spec-spine-core/tests/ai_review_policy.rs
out="$(cargo test -p spec-spine-core --locked --test ai_review_policy inversion 2>&1)" && printf '%s\n' "$out" | grep -qE 'test result: ok\. [1-9][0-9]* passed'
out="$(cargo test -p spec-spine-core --locked --test ai_review_policy publication_failure 2>&1)" && printf '%s\n' "$out" | grep -qE 'test result: ok\. [1-9][0-9]* passed'
out="$(cargo test -p spec-spine-core --locked --test ai_review_policy post_step_condition 2>&1)" && printf '%s\n' "$out" | grep -qE 'test result: ok\. [1-9][0-9]* passed'
test 0 -eq "$(grep -c 'could not post API-failure notice comment' .github/workflows/ai-pr-review.yml)"
test 1 -le "$(grep -c 'could not post fork-skip notice comment' .github/workflows/ai-pr-review.yml)"
test 1 -le "$(grep -c 'could not post Dependabot-skip notice comment' .github/workflows/ai-pr-review.yml)"
test 0 -eq "$(grep -c 'AI_REVIEW_TIMEOUT' .github/workflows/ai-pr-review.yml)"
test 0 -eq "$(grep -cE '(^|[^-[:alnum:]_])timeout [0-9]' .github/workflows/ai-pr-review.yml)"
grep -q 'timeout-minutes: 10' .github/workflows/ai-pr-review.yml
grep -q 'AI_REVIEW_TMP' .github/workflows/ai-pr-review.yml
./target/release/spec-spine lint --fail-on-warn
./target/release/spec-spine check
```
