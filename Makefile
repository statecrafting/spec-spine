# The composite gate, and the ONE definition of the governed loop (spec 094,
# moved here by spec 092 3.6).
#
# It used to live in `kit/Makefile`, the copy this repository distributed to
# adopters and then ran on itself. The kit is gone and the distribution is
# Statecraft's; the gate is not, so the file moved to the repository root and
# kept every semantic it had. `.github/workflows/ci.yml` calls this target
# rather than restating the chain, which is the whole point of there being one
# definition (spec 094 D-1 records what happened the one time a workflow
# restated it).
#
# Variables, all overridable:
#
#   SPEC_SPINE  the binary to govern with. A repository that builds its own
#               must point at the one it builds, which is the resolution order
#               spec 093 established for the hooks and spec 117 3.1 implements
#               here: $SPEC_SPINE, then ./target/release, then PATH. Set it
#               once here rather than in every caller. An explicit value that
#               names nothing executable is REFUSED, never replaced (spec 117
#               3.2): the gate's whole question is which binary answered.
#   BASE        the ref the coupling gate compares against. Resolved from
#               the repository rather than assumed to be origin/main
#               (spec 093); override it here or on the command line.
#   HEAD        the ref the coupling gate compares TO. `HEAD` locally, which is
#               what a session wants. A pull-request CI leg passes the event's
#               frozen head SHA instead: the checked-out `refs/pull/N/merge`
#               HEAD re-resolves against the base every time the job runs, so a
#               gate diffing to it folds in changes merged after the PR opened
#               and reports them as this PR's drift. One definition can serve
#               both callers only if the caller can say which ref it means.
#
#   make gate                 read-only: the whole governed loop, in order
#   make refresh              writing: recompute the committed shard trees
#   make verify SPEC=012      run one spec's declared acceptance (spec 043)
#
# Language targets are guarded on a MANIFEST PROBE, not a command probe. A tree
# with cargo installed and no Cargo.toml is the specify-first case, which is
# three of the four governed repositories, and probing for the tool answers the
# wrong question. Every guarded target is a clean no-op on a code-free corpus.
#
# The guard is an explicit `if`, never `test -f M && cmd || echo skipping`
# (spec 094). `&&`/`||` is not if/else: the `||` branch fires when EITHER the
# probe is false OR the command fails, so on a repository that HAS the manifest
# a failing command exits 0 having printed "no manifest, skipping". That makes
# `govern.yml`'s `make build test fmt clippy` job unfailable, which is the worst
# shape a gate can have: the repository looks defended and is not.

# Spec 117 3.1: the documented order, actually implemented. `SPEC_SPINE ?=
# spec-spine` was step three written as if it were the whole answer: `?=`
# honours an override, so step one was real, step two did not exist, and on any
# machine with an installed CLI the gate governed with that one. Measured here
# at 0.20.0 against a 0.22.0 checkout, and found only because a built lint
# exemption disagreed loudly across the two.
#
# `$(origin ...)`, not `?=`: the fallback has to be able to tell "the caller
# said nothing" from "the caller said something unusable", because those get
# different answers (3.2). Resolved once, with `:=`, so every recipe in this
# file uses the same binary; a gate whose steps could disagree about which
# binary they ran is worse than the one this replaces.
#
# `test -f` as well as `test -x`: a DIRECTORY named target/release/spec-spine
# answers `test -x` and is not a binary.
ifeq ($(origin SPEC_SPINE),undefined)
  ifeq ($(shell test -f ./target/release/spec-spine && test -x ./target/release/spec-spine && echo yes),yes)
    SPEC_SPINE := ./target/release/spec-spine
    SPEC_SPINE_ORIGIN := the binary this checkout builds
  else
    SPEC_SPINE := spec-spine
    SPEC_SPINE_ORIGIN := the PATH fallback -- this repository builds its own binary, so an installed CLI may predate this corpus
  endif
else
  SPEC_SPINE_ORIGIN := an explicit SPEC_SPINE override
endif
# Spec 093 3.3: the coupling base follows the branch this repository
# actually has. The same three steps the push gate resolves with, in the
# same order: $SPEC_SPINE_DEFAULT_BRANCH (make imports the environment, so
# `?=` leaves an exported value alone), then the remote's own HEAD, then
# `main`. An explicit `BASE=` on the command line still wins.
SPEC_SPINE_DEFAULT_BRANCH ?= $(shell git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null | sed 's|^origin/||')
BASE       ?= origin/$(or $(SPEC_SPINE_DEFAULT_BRANCH),main)
# Spec 092 3.6: the head side of the same question. `HEAD` is right for every
# local caller and wrong for a pull-request CI leg; see the header.
HEAD       ?= HEAD

# Spec 094 3.2: whether `gate` runs the whole-tree ownership assertion.
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
# 052 took out of the verb: `--fail-on-untraced` on a tree with no discovered
# package used to enumerate nothing and exit 0 from a step named for the
# assertion. A gate that did not run its check does not get to look green.
#
# An unrecognised value is REFUSED (exit 3), never quietly treated as `auto`. A
# caller writing `OWNERSHIP=yes` is asking for the assertion; falling through to
# the configuration would hand them a config-governed run under a word they
# chose to override it with, which is the same class of silent substitution the
# failed-read rule below exists to stop.
OWNERSHIP  ?= auto

# Spec 094 3.3: whether `gate` runs the coupling gate. ON by default, so a local
# `make gate` is the whole governed loop exactly as it was.
#
# `COUPLE=0` is for the one caller that must not couple: a push-event CI leg.
# Spec 094 3.2 requires the shipped workflow to run `couple` on `pull_request`
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

# Spec 094 3.3: a file holding the PR body, for the `Spec-Drift-Waiver:` line
# `couple` reads out of it. Unset, `couple` runs without `--pr-body` at all: an
# empty flag pointing at nothing is a different command, and a local session has
# no PR body to give. The path is quoted at the call site, so a body file under
# a directory with a space in its name stays one argument.
PR_BODY    ?=

.PHONY: gate refresh verify test build fmt clippy help spec-spine-binary

## The governed loop, read-only throughout. A gate that writes repairs what it
## is meant to judge (spec 093), so this uses `compile --check` and never
## `compile`.
##
## Step 3, the ownership assertion, is guarded by OWNERSHIP (spec 094 3.2). The
## probe CAPTURES the governed read, checks its status, and only then looks at
## the text. Never `config show | grep -q`: a pipeline reports grep's status and
## discards the read's, so a binary too old to have the verb, an unparsable
## spec-spine.toml, or a config key this binary rejects would every one of them
## read as "ownership is off" and produce a green gate. A failed read is not a
## skip.
##
## Step 4, the coupling gate, is guarded by COUPLE (spec 094 3.3), and takes the
## optional PR_BODY file. `$(if ...)` adds `--pr-body` only when PR_BODY is set,
## and quotes the path so it reaches `couple` as one argument.
##
## Both guards are an explicit `if`/`then`/`else` and both announce their skip,
## for the reason spec 094 gives about the language targets below.
## Spec 117 3.2 and 3.3: resolve, refuse, announce. A prerequisite of every
## target that runs the binary, so no verb runs before the question "which
## binary is this" has an answer on the record.
##
## The refusal is deliberately NOT the fall-through `.githooks/pre-commit`
## performs (spec 093 3.12). A hook that must exit 0 whatever happens is right
## to advise with some other binary; a gate asked whether a tree is governed and
## answering with a binary the caller did not name has substituted the subject
## of the question. Spec 117 D-2.
##
## Announced on STDERR: `make verify SPEC=<id>` hands its stdout to the verb,
## and spec 090 made that channel the verdict's. Spec 117 D-3.
##
## The announcement is not a version check and must not become one. The floor is
## `[meta] required_version`, the binary enforces it on every verb, and a second
## comparison here would be a copy that can drift from it. Spec 117 D-5.
##
## The resolved path is copied into a SHELL variable rather than expanded inline
## into each message. `tests/gate.rs` reads `$(SPEC_SPINE) <word>` as an
## invocation and checks `<word>` against the verb list, which is the detector
## that keeps this file from growing a second chain; an announcement reading
## "governing with $(SPEC_SPINE) [..." would be a false invocation fed to it.
## Spec 117 D-6.
## The value reaches the shell through the ENVIRONMENT rather than through a
## quoted expansion. `b='$(SPEC_SPINE)'` breaks on a value containing a single
## quote, producing a shell syntax error instead of this target's own refusal,
## and double-quoting trades that for `$` expansion. A target-specific export
## hands the bytes over verbatim and neither character is special. Spec 117 D-8.
##
## Usability is tested with `test -x` for a value naming a PATH, and with
## `command -v` only for a bare name. `command -v` does not agree across shells
## on a value containing a slash: bash checks the executable bit, dash returns
## the string unchanged, so a non-executable absolute path passed this guard on
## Linux and failed one step later with the wrong message. The refusal a caller
## sees must not depend on which `/bin/sh` the runner has. Spec 117 D-9.
## `SPEC_SPINE_ORIGIN` crosses the same way, for the same reason. It is an
## internal variable rather than a documented input, but a command-line
## assignment overrides a `:=` one in GNU make, so it is reachable, and leaving
## half a class eliminated is worse than not knowing about it. Spec 117 D-8.
spec-spine-binary: export SPEC_SPINE_VALUE = $(SPEC_SPINE)
spec-spine-binary: export SPEC_SPINE_ORIGIN_VALUE = $(SPEC_SPINE_ORIGIN)
spec-spine-binary:
	@b="$$SPEC_SPINE_VALUE"; \
	if test -z "$$b"; then \
		echo "gate: SPEC_SPINE is set and empty, so no binary was named. An empty value would drop the binary from the command line and run the verb name as a command; refusing instead (spec 117 3.2)." >&2; \
		exit 3; \
	fi; \
	usable=no; \
	case "$$b" in \
		*/*) test -f "$$b" && test -x "$$b" && usable=yes ;; \
		*) command -v "$$b" >/dev/null 2>&1 && usable=yes ;; \
	esac; \
	if test "$$usable" = no; then \
		if test "$$SPEC_SPINE_ORIGIN_VALUE" = "an explicit SPEC_SPINE override"; then \
			echo "gate: SPEC_SPINE=$$b names nothing executable. Refusing rather than falling back to ./target/release/spec-spine or PATH: an explicit override is the caller naming the binary that must answer (spec 117 3.2)." >&2; \
		else \
			echo "gate: could not resolve a binary to govern with. Looked for ./target/release/spec-spine (build it with \`cargo build --release -p spec-spine-cli\`), then \`spec-spine\` on PATH (spec 117 3.1)." >&2; \
		fi; \
		exit 3; \
	fi; \
	v=$$("$$b" --version 2>&1) || { echo "gate: $$b --version failed; the resolved binary does not run here." >&2; exit 3; }; \
	echo "gate: governing with $$b [$$SPEC_SPINE_ORIGIN_VALUE] -- $$v" >&2

gate: spec-spine-binary
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
		$(SPEC_SPINE) couple --base $(BASE) --head $(HEAD) $(if $(PR_BODY),--pr-body "$(PR_BODY)"); \
	fi

## The writing half, for a live session that has edited a spec and can commit
## the regenerated shards with the change that made them stale.
refresh: spec-spine-binary
	$(SPEC_SPINE) compile
	$(SPEC_SPINE) index

## One spec's declared acceptance. Runs code the corpus declares (spec 043),
## which is why it is deliberately not part of `gate`.
verify: spec-spine-binary
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
	@echo "         SPEC_SPINE=<path>   the binary to govern with; default"
	@echo "                             ./target/release/spec-spine, else PATH"
	@echo "         OWNERSHIP=auto|1|0  run the whole-tree ownership assertion"
	@echo "         COUPLE=1|0          run the coupling gate"
	@echo "         PR_BODY=<file>      PR body for the waiver line couple reads"
	@echo "         BASE=<ref> HEAD=<ref>  what the coupling gate diffs"
	@echo "refresh  recompute the committed shard trees"
	@echo "verify   SPEC=<id>, one spec's declared acceptance"
	@echo "test build fmt clippy   guarded on a manifest probe"
