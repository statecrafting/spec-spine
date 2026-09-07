# Specify-first: governing a corpus before the code exists

Three of the four repositories governed by spec-spine work this way: the whole
corpus is ratified before a line of code is written, and it stays that way for
months. Every read verb, every count, and every warning tier means something
slightly different in that mode, and this page says what.

## What specify-first means

A spec is `approved` when the corpus has agreed to it, and `implementation:
pending` until somebody builds it. **Those two together are the steady state,
not a transitional one.** A repository can hold sixty approved-and-pending specs
for a quarter and be in perfect health.

The table of which combinations are schedulable, and how strictly each holds
its claims, lives under **Lifecycle as scheduling** in
[`standards/spec/contract.md`](../standards/spec/contract.md). It is not
repeated here: two tables that must agree by hand is a defect waiting to
happen, and the contract is the one that ships to adopters.

## `implementation: n-a` for a spec that owns no code

A spec that defines a convention, records a decision, or bootstraps the system
owns no code and never will. Left `pending` it is offered as ready forever, and
a scheduler that keeps proposing it is a scheduler nobody trusts.

`implementation: n-a` is the answer: ratified, owns nothing, not schedulable.
The scaffolded bootstrap spec already carries it for exactly this reason.

## The warnings are the defined state, not debt

On an unbuilt corpus, every claimed unit resolves to nothing, because nothing
has been written. That is `W-001` per unit, and on a large corpus it is a large
number: aicortex carried 248 and was in exactly the state its specs described.

Three things follow, and all three are correct:

- **`index check` calls the ledger fresh.** Freshness is "do the committed
  shards match what the corpus compiles to", and they do. Unresolved units are a
  separate axis, reported alongside rather than folded into the verdict.
- **`W-001` is a warning, not an error.** A draft or a pending spec is expected
  to claim territory it has not written yet. The error tier is reserved for a
  spec claiming `complete` while its units resolve to nothing, which is a claim
  contradicted by the ledger.
- **`--fail-on-unresolved` is the flag for a corpus that has moved past this
  stage.** Turn it on when your repository builds what it claims within one pull
  request. Turning it on earlier refuses your own normal state.

`spec-spine index diagnostics` lists them, and `spec-spine index orphans`
separates the specs that are genuinely orphaned from the ones merely in flight.

## The composite gate on a code-free tree

`kit/Makefile` and `kit/govern.yml` are the gate and the CI workflow, and both
are built for this mode: the language targets are guarded on a **manifest
probe**, so `make test` on a tree with no `Cargo.toml` is a clean no-op rather
than a failure. Install them with `spec-spine init --with-kit`.

One finding from that workflow is worth carrying even if you write your own:
**GitHub rejects `hashFiles` in a job-level `if`.** The expression is evaluated
before the workspace exists, so the function has no files to hash. Conditioning
a job on "does this repository contain a `Cargo.toml`" needs a probe job that
checks after checkout and publishes the answer as a job output. Three adopters
rediscovered that independently.

## What to run first, on a repository with no code

```sh
spec-spine compile          # the corpus is the thing that exists; validate it
spec-spine index            # emits the index; every unit is unresolved, and that is fine
spec-spine lint             # corpus conformance
```

**Not `couple` yet.** The coupling gate compares a base to a head and refuses
code that drifts from its owning spec. With no code and no pull request there is
nothing for it to compare, and wiring it in before there is will only teach you
to ignore it.

`spec-spine index coverage` is worth running early anyway: on a code-free tree
it reports an empty universe, and with `--fail-on-untraced` it **refuses** rather
than passing vacuously, which is the honest answer to "is every source file
owned" when there are no source files.
