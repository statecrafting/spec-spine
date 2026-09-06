# 02: spec-spine as the agentic-builder substrate

A design note, not a spec. It records what spec-spine must grow in order to
serve as the adjudicator for a provider-agnostic autonomous builder, and, with
equal weight, what it must refuse to grow. The specs it proposes are filed
separately; this note is the reasoning they cite.

## 1. Where the requirement came from

Design work on a provider-agnostic autonomous builder (an agent-plural successor
to a Claude-only orchestrator) settled on spec-spine as its substrate. The
builder drives one spec per session through build, ship, and verify, using any
agent (Claude Code, Codex, Gemini, Cursor, or an ACP-speaking long tail), and the
**gate, not the session's own claim, decides whether the work is done**. Its
one-line pitch is *done is not self-authored*, and it is provider-neutral by
construction because the adjudicator never trusts the agent.

That product needs an adjudicator with four properties, all of which spec-spine
either has or is the natural home for: a typed corpus with declared territory, a
deterministic external verdict, a stable exit-code taxonomy, and a tamper-evident
record that verifies offline. This note takes the builder's requirement set,
subtracts what spec-spine already provides, and records the remainder.

## 2. The boundary, stated before the backlog

The test for whether a requirement belongs in spec-spine at all:

> Would an adopter who will never run the builder still want it?

| Belongs here | Belongs to the builder |
|---|---|
| Driving an arbitrary target repo (`--repo`) | Driver protocol and capability tiers |
| A declared, unowned tool-state root | Sensors, agent-home watchers |
| Per-spec, reproducible attestation | Session transcripts, cost, token accounting |
| Machine-readable verdicts | Stage routing tables, model preference lists |
| A dependency-ordered ready set | Quota leases, provider budgets |
| An agent-neutral scaffold | Service installation, reboot resume |

The right-hand column must not reach `Config`. Where a builder genuinely needs
per-spec configuration, the mechanism already exists and requires no change:
`frontmatter.extra_known_keys` (spec 013) lets a spec carry `agent: codex` today,
passed through into the registry as declared extra frontmatter that spec-spine
stores and never interprets.

This is not fastidiousness. spec-spine's value to the builder is precisely that
it is not the builder. An adjudicator that knows what an agent is can be argued
with about agents; one that only knows specs, units, and hashes cannot. The
neutrality is the moat, and the conformance-matrix idea (publishing which agents
reach which capability tier, measured against a common yardstick) only works if
the yardstick has no stake in the outcome.

## 3. What spec-spine already provides

Most of the builder's substrate asks are already shipped. Subtracting them first
is what makes the remaining backlog small.

| Requirement | Provided by |
|---|---|
| Adjudicate an arbitrary target repo from a daemon's own home | the global `--repo <DIR>` flag |
| Target corpora not rooted at `specs/` | `layout.specs_dir` (spec 036) |
| Per-spec configuration the substrate ignores | `frontmatter.extra_known_keys` (spec 013) |
| "A gate is a command, an exit-code taxonomy, an artifact path" | the `0/1/2/3` contract, mapped in exactly one place |
| Adjudicate outside a pull request, without git | `couple --paths-from`, `--pr-body` / `$SPEC_SPINE_PR_BODY` |
| A tamper-evident, offline-verifiable record | `attest` / `verify-attestation`: `CorpusAttestation` plus a detached Ed25519 `LedgerSeal` (spec 023) |
| A dependency graph for the scheduler | `depends_on` plus cycle refusal (spec 033) |
| A non-Rust consumer | the JSON facade, complete across every verb |
| Draft specs whose units do not exist yet | severity tiers on unresolved units (spec 025): a `draft` or `implementation: pending` spec yields counted `W-001`/`W-002` warnings, not blocking errors |

Spec 023 also already solved the hardest design problem the builder poses, in a
form that generalizes: it keeps the reproducible payload pure and puts the
wall-clock instant and the signer identity in the **detached seal**. Section 5
holds every new artifact to that shape.

## 4. The gaps

### G1. `implementation:` is unverified self-assertion

`implementation: pending | in-progress | complete | n-a | deferred` exists in the
frontmatter grammar, and `lint.rs` contains no rule that reads it. A spec's own
frontmatter declares its work done, adjudicated by nothing.

For a corpus maintained by hand this is harmless descriptive metadata. For a
builder that routes on it, it is the agent writing its own diploma, and it
contradicts the product's central claim directly. The fix is a **completion
gate**: `implementation: complete` is accepted only against evidence, meaning
every unit the spec claims resolves and a matching per-spec attestation is
present. Until that exists, the field is a liability rather than a feature, and
the builder must not read it.

This gap is the single most important item here and it did not appear in the
builder's own backlog, because from inside the builder the field looks like it
already works.

### G2. Attestation is corpus-scoped; the builder's unit of work is one spec

`CorpusAttestation` freezes one verdict set (`compile`, `lint`, optionally
`couple`) over the entire corpus. The builder produces evidence per spec, and
that per-spec bundle is the **only** artifact a hosted control plane consumes.

Its schema is therefore the highest-leverage design artifact in the whole plan,
and it must be settled before any consumer exists, not discovered by one. A
`SpecAttestation` is scoped to a single spec id and covers that spec's own source
bytes, the content hashes of the units it claims, and the verdicts restricted to
it.

### G3. No `--json` on the adjudicating verbs

`registry` (four subcommands) and `index coverage` emit `--json`. The verbs that
actually render a verdict do not: `couple`, `lint`, `compile --check`,
`index check`, and `attest` all print prose to stdout.

An orchestrator that string-matches `"index is fresh"` is doing exactly the
ad-hoc parsing the constitution forbids (§II) and the governed-artifact-reads
rule was written against. The facade already returns structured reports for all
of these; only the CLI surface is missing. Small, additive, and blocking for
every other item, which makes it the first spec.

### G4. `layout.state_dir` needs two halves

Declaring the key is trivial. The half that matters: `couple`'s bypass floor and
`index coverage`'s classifier must both understand it, or a daemon writing its
own state into the target repo either trips the gate or reports as unclaimed
source. Note also the claimed-path-overrides-bypass interaction, and that the
boundary *inside* the directory is real: committed evidence on one side,
gitignored transcripts and databases on the other, stated rather than implied.

### G5. The scaffold is Claude-only

`scaffold.rs` writes three `.claude/rules/*` files and nothing else; `kit/` is
`.claude` plus an `AGENTS.md`. A provider-agnostic product must scaffold
`AGENTS.md` as the primary artifact with per-agent adapters as thin dispatchers,
which is the pattern this repo already runs on itself. This is the adoption
surface: every non-Claude agent that ever adopts spec-spine arrives through that
file.

### G6. No ready-set query

`depends_on`, `status`, and `implementation` all exist; nothing answers "which
specs are ready to build, in dependency order". The builder needs it once per
session. Cheap, because spec 033 already walks the graph and refuses cycles.

### G7. The coherence guard is a prompt, not a gate

`.claude/rules/adversarial-prompt-refusal.md` asks the agent not to resolve a
failing gate by editing the spec to match the code it just wrote. It is prose
addressed to a cooperative reader. Codex has no `PreToolUse` hook, so the
enforcement path that exists for one vendor does not generalize, and even for
that vendor a prompt is unenforceable against an adversarial or merely careless
session. This belongs in the gate. It is also the hardest item here, and no
mechanism should be asserted before it is designed: the naive rule (refuse a diff
that both narrows a claim and edits the claimed code) collides with legitimate
refactors, which are the same shape.

### G8. Redaction is undefined

If spec-spine owns the bundle format, it owns the guarantee about what a bundle
cannot contain, or it disclaims that guarantee explicitly. Unowned redaction in a
product sold on compliance is a liability, and the answer is cheap to state while
the format is still being designed.

## 5. Four constraints on how these land

**Evidence cannot be a `compile` output.** Every artifact-producing function is a
pure function of `(Config, file contents)`: no clock, no env, no git. A build
record (which agent, which session, at what time, at what cost) is irreducibly
impure. Follow spec 023 exactly: the attestation payload stays pure and
reproducible, and the impure record is a signed sibling excluded from the
determinism gates. Do not let the builder's needs leak a clock into the ledger;
determinism is the central claim, and the four-triple CI gate will catch it
anyway, loudly and late.

**Every committed artifact class needs its freshness gate designed with it.**
This repo learned it expensively: committed registry shards had no freshness
check, stale `shardHash`es reached `main` undetected, and spec 031 exists to
close that. Evidence committed by a *daemon* rather than by `compile` is a third
class with no gate at all. Specify the check verb in the same document that
introduces the artifact, not in the follow-up after it rots.

**Sharding already gives evidence its conflict story.** Per-spec bundles are
naturally disjoint, so keeping them under `by-spec/`-shaped paths (spec 024)
means two concurrent builds never contend for one file. Take the property for
free rather than reinventing it.

**Binding `lint` to artifacts another tool produced is a real, bounded
exception.** It remains legitimate (lint still reads only committed file
contents, so it stays a pure function of the tree) but it weakens the "lint is a
function of the corpus" mental model. Say so in the spec rather than letting a
later reader discover it.

## 6. Waves

| Wave | Items | Specs | Character |
|---|---|---|---|
| 1 | G3, G6, G4 | 037, 038, 039 | Additive, unblocks the builder, no schema MAJOR |
| 2 | G1, G2 | 041, 042 | Filed, and narrower than sketched below |
| 3 | G5, G8 | to be filed | Adoption and the compliance guarantee |
| 4 | G7 | to be filed | Design first; assert no mechanism yet |

An unnumbered item landed alongside wave 1: spec 040 writes down how an
amendment is authored (declared once, in the amending spec; the predecessor is
never edited). It adds no mechanism, and exists because a review asked the same
question six times without the corpus being able to answer it.

Wave 1 is filed alongside this note.

**Wave 2 as filed differs from the sketch above, and the difference is the
point of having checked.** Two findings from the code, not from this analysis:

- **G1 was over-stated here.** `index.rs::in_flight` is
  `status == "draft" || implementation == Pending`, and spec 025 already makes an
  unresolved owning unit a blocking error for any spec that is not in flight. So
  an *approved* spec marked `complete` is already held to its claims. What was
  missing is one arm of one predicate: `Implementation::Complete` never enters
  the expression, so `status: draft` alone buys leniency, and this corpus files
  every spec as `draft` + `complete` when its code lands. Spec 041 is that fix,
  and needs no attestation.
- **G2's committed bundle does not survive contact with the repo.**
  `.derived/attestation/` is gitignored: 023's attestation is on-demand by
  design. A committed per-spec bundle would restale on every edit to any claimed
  unit and would need a fourth committed tree with a fifth gate verb, which is
  this note's own constraint turned against the proposal. Spec 042 keeps the
  artifact on-demand and signed.

The two are therefore independent rather than one consuming the other, and
§3.4 of 042 records why the pairing could not have worked in `lint` regardless:
the payload carries the lint verdict, so a lint rule reading it would grade
itself.

## 7. Open questions

- **Does the bundle format become a fourth schema axis?** The existing three
  (`registry`, `index`, `build-meta`, plus `config`) are compile-time constants
  with a conformance test. A per-spec attestation consumed by a third party wants
  the same discipline, and probably wants to be independently versioned so an
  external emitter can target it without tracking the registry's version.
- **Who owns redaction?** Stating the guarantee in the format is cheapest; the
  alternative is an explicit disclaimer, and the one unacceptable answer is
  silence.
- **Two frontmatter keys already exist that nothing reads:** `feature_branch` and
  `code_aliases`. Whether they were anticipating a driver is not recorded. If
  wave 2 gives them meaning, that should be a deliberate decision with the
  reasoning written down, not an accretion.
