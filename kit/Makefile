# The composite gate every adopter wrote by hand (spec 064).
#
# Copy this to your repository root. Two variables, both overridable:
#
#   SPEC_SPINE  the binary to govern with. A repository that builds its own
#               must point at the one it builds, which is the resolution order
#               spec 051 established: $SPEC_SPINE, then ./target/release, then
#               PATH. Set it once here rather than in every caller.
#   BASE        the ref the coupling gate compares against. Resolved from
#               the repository rather than assumed to be origin/main
#               (spec 072); override it here or on the command line.
#
#   make gate                 read-only: the whole governed loop, in order
#   make refresh              writing: recompute the committed shard trees
#   make verify SPEC=012      run one spec's declared acceptance (spec 049)
#
# Language targets are guarded on a MANIFEST PROBE, not a command probe. A tree
# with cargo installed and no Cargo.toml is the specify-first case, which is
# three of the four governed repositories, and probing for the tool answers the
# wrong question. Every guarded target is a clean no-op on a code-free corpus.
#
# The guard is an explicit `if`, never `test -f M && cmd || echo skipping`
# (spec 089). `&&`/`||` is not if/else: the `||` branch fires when EITHER the
# probe is false OR the command fails, so on a repository that HAS the manifest
# a failing command exits 0 having printed "no manifest, skipping". That makes
# `govern.yml`'s `make build test fmt clippy` job unfailable, which is the worst
# shape a gate can have: the repository looks defended and is not.

SPEC_SPINE ?= spec-spine
# Spec 072 3.3: the coupling base follows the branch this repository
# actually has. The same three steps the push gate resolves with, in the
# same order: $SPEC_SPINE_DEFAULT_BRANCH (make imports the environment, so
# `?=` leaves an exported value alone), then the remote's own HEAD, then
# `main`. An explicit `BASE=` on the command line still wins.
SPEC_SPINE_DEFAULT_BRANCH ?= $(shell git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null | sed 's|^origin/||')
BASE       ?= origin/$(or $(SPEC_SPINE_DEFAULT_BRANCH),main)

.PHONY: gate refresh verify test build fmt clippy help

## The governed loop, read-only throughout. A gate that writes repairs what it
## is meant to judge (spec 046), so this uses `compile --check` and never
## `compile`.
gate:
	$(SPEC_SPINE) check --fail-on-unresolved --fail-on-warn
	$(SPEC_SPINE) lint --fail-on-warn
	$(SPEC_SPINE) index coverage --fail-on-untraced
	$(SPEC_SPINE) couple --base $(BASE) --head HEAD

## The writing half, for a live session that has edited a spec and can commit
## the regenerated shards with the change that made them stale.
refresh:
	$(SPEC_SPINE) compile
	$(SPEC_SPINE) index

## One spec's declared acceptance. Runs code the corpus declares (spec 049),
## which is why it is deliberately not part of `gate`.
verify:
	@test -n "$(SPEC)" || { echo "usage: make verify SPEC=<id>"; exit 3; }
	$(SPEC_SPINE) verify $(SPEC)

test:
	@if test -f Cargo.toml; then cargo test --workspace --locked; else echo "no Cargo.toml, skipping"; fi
	@if test -f package.json; then npm test --if-present; else echo "no package.json, skipping"; fi

build:
	@if test -f Cargo.toml; then cargo build --workspace --locked; else echo "no Cargo.toml, skipping"; fi

fmt:
	@if test -f Cargo.toml; then cargo fmt --all --check; else echo "no Cargo.toml, skipping"; fi

clippy:
	@if test -f Cargo.toml; then cargo clippy --workspace --all-targets --locked -- -D warnings; else echo "no Cargo.toml, skipping"; fi

help:
	@echo "gate     the governed loop, read-only"
	@echo "refresh  recompute the committed shard trees"
	@echo "verify   SPEC=<id>, one spec's declared acceptance"
	@echo "test build fmt clippy   guarded on a manifest probe"
