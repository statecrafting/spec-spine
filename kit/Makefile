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

# Spec 114 3.2: whether `gate` runs the whole-tree ownership assertion.
#
#   auto  (default) ask this repository's own effective configuration, through
#         `spec-spine config show`, for `[coupling] require_ownership`. A corpus
#         relying on the default gets the default's answer and not a missing key.
#   1     run the assertion whatever the configuration says, for a CI job that
#         wants to demand it.
#   0     do not run it.
#
# `auto` and `0` both ANNOUNCE the skip and say whole-tree ownership was not
# verified. A silent skip would put back, one layer out, the vacuous pass spec
# 059 took out of the verb: `--fail-on-untraced` on a tree with no discovered
# package used to enumerate nothing and exit 0 from a step named for the
# assertion. A gate that did not run its check does not get to look green.
#
# An unrecognised value is REFUSED (exit 3), never quietly treated as `auto`. A
# caller writing `OWNERSHIP=yes` is asking for the assertion; falling through to
# the configuration would hand them a config-governed run under a word they
# chose to override it with, which is the same class of silent substitution the
# failed-read rule below exists to stop.
OWNERSHIP  ?= auto

# Spec 114 3.3: whether `gate` runs the coupling gate. ON by default, so a local
# `make gate` is the whole governed loop exactly as it was.
#
# `COUPLE=0` is for the one caller that must not couple: a push-event CI leg.
# Spec 064 3.2 requires the shipped workflow to run `couple` on `pull_request`
# only, because a push has already merged (nothing left to refuse) and carries
# no PR body, so a `Spec-Drift-Waiver:` line is unrecoverable. One gate
# definition can serve both legs only if the caller can say which it is, and
# this is that word. The skip is announced, like every other.
#
# Deliberately NOT inferred from `PR_BODY`. Whether to couple is a question
# about the event; whether a waiver is reachable is a question about the body.
# GitHub permits an empty PR description, so a body-shaped inference would turn
# every description-less pull request into a coupling gate that quietly did not
# run.
#
# `1` and `0` are the only values; anything else is refused (exit 3) rather than
# read as "not 0, so couple". A control whose typo means the opposite of what
# was typed is not a control.
COUPLE     ?= 1

# Spec 114 3.3: a file holding the PR body, for the `Spec-Drift-Waiver:` line
# `couple` reads out of it. Unset, `couple` runs without `--pr-body` at all: an
# empty flag pointing at nothing is a different command, and a local session has
# no PR body to give. The path is quoted at the call site, so a body file under
# a directory with a space in its name stays one argument.
PR_BODY    ?=

.PHONY: gate refresh verify test build fmt clippy help

## The governed loop, read-only throughout. A gate that writes repairs what it
## is meant to judge (spec 046), so this uses `compile --check` and never
## `compile`.
##
## Step 3, the ownership assertion, is guarded by OWNERSHIP (spec 114 3.2). The
## probe CAPTURES the governed read, checks its status, and only then looks at
## the text. Never `config show | grep -q`: a pipeline reports grep's status and
## discards the read's, so a binary too old to have the verb, an unparsable
## spec-spine.toml, or a config key this binary rejects would every one of them
## read as "ownership is off" and produce a green gate. A failed read is not a
## skip.
##
## Step 4, the coupling gate, is guarded by COUPLE (spec 114 3.3), and takes the
## optional PR_BODY file. `$(if ...)` adds `--pr-body` only when PR_BODY is set,
## and quotes the path so it reaches `couple` as one argument.
##
## Both guards are an explicit `if`/`then`/`else` and both announce their skip,
## for the reason spec 089 gives about the language targets below.
gate:
	$(SPEC_SPINE) check --fail-on-unresolved --fail-on-warn
	$(SPEC_SPINE) lint --fail-on-warn
	@run=no; why="[coupling] require_ownership is off"; \
	cfg="$${TMPDIR:-/tmp}/spec-spine-gate-config.$$$$"; \
	if test "$(OWNERSHIP)" = "1"; then \
		run=yes; \
	elif test "$(OWNERSHIP)" = "0"; then \
		run=no; why="OWNERSHIP=0"; \
	elif test "$(OWNERSHIP)" != "auto"; then \
		echo "gate: OWNERSHIP=$(OWNERSHIP) is not one of auto, 1, 0" >&2; exit 3; \
	else \
		$(SPEC_SPINE) config show > "$$cfg"; st=$$?; \
		if test $$st -ne 0; then rm -f "$$cfg"; exit $$st; fi; \
		if grep -qF 'require_ownership = true' "$$cfg"; then \
			run=yes; \
		elif ! grep -qF 'require_ownership = false' "$$cfg"; then \
			rm -f "$$cfg"; \
			echo "gate: the effective config named no require_ownership setting, so the ownership decision could not be read" >&2; \
			exit 3; \
		fi; \
		rm -f "$$cfg"; \
	fi; \
	if test "$$run" = yes; then \
		$(SPEC_SPINE) index coverage --fail-on-untraced; \
	else \
		echo "gate: $$why, so whole-tree ownership was NOT verified (the --fail-on-untraced assertion did not run; set OWNERSHIP=1 to demand it)"; \
	fi
	@if test "$(COUPLE)" = "0"; then \
		echo "gate: COUPLE=0, so drift against a base was NOT checked (the coupling gate did not run)"; \
	elif test "$(COUPLE)" != "1"; then \
		echo "gate: COUPLE=$(COUPLE) is not one of 1, 0" >&2; exit 3; \
	else \
		$(SPEC_SPINE) couple --base $(BASE) --head HEAD $(if $(PR_BODY),--pr-body "$(PR_BODY)"); \
	fi

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
	@echo "         OWNERSHIP=auto|1|0  run the whole-tree ownership assertion"
	@echo "         COUPLE=1|0          run the coupling gate"
	@echo "         PR_BODY=<file>      PR body for the waiver line couple reads"
	@echo "refresh  recompute the committed shard trees"
	@echo "verify   SPEC=<id>, one spec's declared acceptance"
	@echo "test build fmt clippy   guarded on a manifest probe"
