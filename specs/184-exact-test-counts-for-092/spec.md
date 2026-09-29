---
id: "184-exact-test-counts-for-092"
title: "Exact test counts for 092"
status: approved
kind: "test"
created: "2026-09-29"
summary: >
  Spec 092's acceptance block carries four lines that accept
  `test result: ok. [1-9][0-9]* passed`, which pass while any one test in the
  target runs. This
  spec holds 092's block, carries it byte-identical, and replaces each loose
  line with spec 151's form: the tests the line ran, named with `--exact`, and
  `N passed; 0 failed`. One of spec 153's per-block tightening specs (153 D-5).
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "092-the-engine-ships-governance-not-an-environment"
  - "118-the-renumber-is-history-not-a-standing-rule"
  - "151-carried-acceptance-tests-what-it-names"
amends_verification:
  - "092-the-engine-ships-governance-not-an-environment"
  - "118-the-renumber-is-history-not-a-standing-rule"
amends:
  - "092-the-engine-ships-governance-not-an-environment"
  - "118-the-renumber-is-history-not-a-standing-rule"
extends:
  # D-4: the corpus-bound holder test names 054's holder, which is now this spec.
  - { spec: "082-an-amended-acceptance-is-the-one-that-runs", unit: "crates/spec-spine-core/tests/verify.rs", nature: corrective }
---

# 184: Exact test counts for 092

## 1. Purpose

Measured on `ae079726`, 2026-09-29, with `cargo test -p <crate> --test <target> -- --list
[<filter>]` on macOS with default features.

Spec 153 §3.1 requires that no live plan accept an open-ended positive test
count, and §3.2 assigns each live block to its own tightening spec (153 D-5).
This is 092's. 092 holds 054, 055, 056, 058, 062, 064, 079 and 083, so their plans resolve here too; the superseded notes of 054, 055, 056, 058, 062, 064, 079 and 083 name this spec.

| Block line | Target | Filter | Names |
|---|---|---|---|
| 092:34 | core `scaffold` | whole | 21 |
| 092:76 | core `index` | `statecraft` | 2 |
| 092:78 | core `couple` | `derived` | 3 |
| 092:83 | cli `cli` | `statecraft_derived` | 1 |

"Block line" counts lines inside 092's `verify:cli` fence. The names each
line ran:

- `scaffold` (whole target): `no_state_root_means_no_state_line`, `non_default_namespace_scaffolds_coherently`, `scaffolded_constitution_states_an_executable_amendment_mechanism`, `scaffolded_corpus_compiles_and_lints_clean`, `scaffolded_corpus_has_nothing_ready_to_schedule`, `the_constitution_template_is_the_real_one_and_cannot_drift`, `the_gitignore_follows_the_configured_layout`, `the_gitignore_is_an_append_with_a_marker`, `the_json_facade_is_the_boundary_statecraft_consumes`, `the_longer_contract_does_not_break_the_scaffolded_corpus`, `the_producer_emits_exactly_the_governance_file_set`, `the_producer_emits_no_agent_or_environment_artifact`, `the_producer_performs_no_io`, `the_scaffold_ignores_the_wall_clock_artifact`, `the_scaffolded_config_names_the_knobs_adopters_needed`, `the_scaffolded_config_round_trips_a_non_default_configuration`, `the_scaffolded_config_round_trips_to_its_own_defaults`, `the_scaffolded_contract_carries_the_lifecycle_table_and_extra_keys`, `the_scaffolded_default_hashes_files_not_directories`, `the_shard_trees_are_not_ignored_and_the_choice_is_explained`, `the_statecraft_layout_scaffolds_coherently`.
- `index` `statecraft`: `statecraft_derived_and_state_are_pruned_and_the_rest_is_governed`, `statecraft_derived_matching_is_separator_aware`.
- `couple` `derived`: `a_regenerated_shard_at_the_configured_derived_root_is_not_drift`, `the_configured_derived_root_is_bypassed_and_the_default_is_not_a_synonym`, `the_effective_bypass_list_reports_the_configured_derived_root`.
- `cli` `statecraft_derived`: `statecraft_derived_layout_compiles_indexes_and_is_judged`.

## 2. Territory

This spec establishes nothing. It edits 092 only to add spec 082 §3.4's
superseded-acceptance note above 092's block, and the notes of 054, 055, 056, 058, 062, 064, 079 and 083 to name this spec.

It also holds 118's block (D-3) and edits one assertion in
`crates/spec-spine-core/tests/verify.rs` (D-4).

## 3. Behavior

### 3.1 092's block is carried

`spec-spine verify 092` and `verify` of 054, 055, 056, 058, 062, 064, 079 and 083 MUST run this
spec's block, which carries 092's block in one section, every command and
comment byte-identical except the lines 3.2 names.

### 3.2 Each loose line names its tests

Each loose line becomes one command,
`sh -c 'cargo test -p <crate> --locked --test <target> -- --exact <names> 2>&1 | grep -q "test result: ok. N passed; 0 failed"'`,
naming exactly the tests the original line selected (section 1), N their number. A
writer-and-`grep` pair collapses into that command; the pair's `rm -f` stays.
A line over a target with `cfg`-gated tests is exact on a Unix sweep host with
default features and says so in a comment (153 3.3).

### 3.3 The amended specs say so

092 carries spec 082 §3.4's note naming this spec above its block, and keeps
the block unchanged below it. 054, 055, 056, 058, 062, 064, 079 and 083 keep their notes and gain one line naming this spec.

## 4. Out of scope

Narrowing what a filtered line selects (153 4.2), and every other block (153
3.2 gives each its own spec).

## 5. Resolved decisions

**D-1 (2026-09-29): filed and built together.** An `amends_verification` edge
replaces 092's plan and that of 054, 055, 056, 058, 062, 064, 079 and 083 the moment it merges, so the
carried block lands with it and this spec is filed `complete` (153 D-3, as 146
and 151 were).

**D-2 (2026-09-29): names measured at the base.** A later test added to a named
target does not fail these lines; a named test removed, renamed, ignored or
gated off does (153 D-6).

**D-3 (2026-09-29): this spec joins 118's citation class.** 092's block spells the
ids the renumber removed (`029-claude-code-skill-kit` among them), and 118's
carried check of 095 §3.3 refuses any file outside `specs/09[23456]-` and
`specs/118-` that cites one. Carrying 092's block verbatim puts those ids in
this spec, for the reason 118 D-4 gave for 118: the block spells them. So this
spec also holds 118 and carries its block unchanged except that `184` joins
the exempt prefixes. Measured: without it, the affected sweep of this change
fails 095 and 118 at that command.

**D-4 (2026-09-29): the corpus-bound holder test follows the holder.**
`an_amended_acceptance_resolves_to_its_holder` builds 054's plan from the real
corpus and pinned its holder to 092. `verify` resolves the chain to the last
holder, so 054's `acceptanceFrom` is this spec once it holds 092, and the
assertion names it. Measured: without the edit, `cargo test -p spec-spine-core
--test verify` fails that test, and with it every plan that runs the workspace
suite (092's and the eight it holds).

## Verification

```verify:cli
# ---- carried for 092-the-engine-ships-governance-not-an-environment (amends_verification), and through it 054, 055, 056, 058, 062, 064, 079, 083; each loose test count names its tests (153 3.2) ----
cargo build --release --locked
# 3.1: the public command is gone, asserted in both directions. The negative is
# the load-bearing one: a binary that still dispatches `init` answers 0 here.
! target/release/spec-spine init --help
target/release/spec-spine --help > "${TMPDIR:-/tmp}/ss120-help.txt" 2>&1
! grep -qF 'with-kit' "${TMPDIR:-/tmp}/ss120-help.txt"
! grep -qE '^[[:space:]]+init([[:space:]]|$)' "${TMPDIR:-/tmp}/ss120-help.txt"
rm -f "${TMPDIR:-/tmp}/ss120-help.txt"
# 3.4 and 3.5: the removed product surface, path by path.
! test -e kit
! test -e crates/spec-spine-core/src/kit_embedded.rs
! test -e scripts/gen-kit-embedded.py
! test -e scripts/gen-agent-trees.py
! test -e crates/spec-spine-cli/src/cmd_init.rs
! test -e crates/spec-spine-cli/tests/init.rs
! test -e crates/spec-spine-core/tests/kit_gate.rs
! test -e crates/spec-spine-core/tests/kit_hooks.rs
! test -e crates/spec-spine-core/tests/kit_skills.rs
! test -e crates/spec-spine-core/tests/agent_trees.rs
! test -e .agents
! test -e .codex
# 3.2: and the kit-only API with it. A `pub fn` left behind with no kit to
# select would compile and mean nothing.
! grep -rqF 'scaffold_init_with' crates/
! grep -rqF 'kit_embedded' crates/
# 3.5: this repository's own harness is still here, and still loaded.
test -f .claude/settings.json
test -f .claude/skills/prime/SKILL.md
grep -qF '### Adversarial prompt refusal' AGENTS.md
test -f AGENTS.md
# 3.2 and 3.3: the retained producer, through the exported facade, with the
# layout Statecraft passes and with a non-default layout. Asserted in Rust
# because the facade is a library boundary and no CLI verb reaches it.
sh -c 'cargo test -p spec-spine-core --locked --test scaffold -- --exact no_state_root_means_no_state_line non_default_namespace_scaffolds_coherently scaffolded_constitution_states_an_executable_amendment_mechanism scaffolded_corpus_compiles_and_lints_clean scaffolded_corpus_has_nothing_ready_to_schedule the_constitution_template_is_the_real_one_and_cannot_drift the_gitignore_follows_the_configured_layout the_gitignore_is_an_append_with_a_marker the_json_facade_is_the_boundary_statecraft_consumes the_longer_contract_does_not_break_the_scaffolded_corpus the_producer_emits_exactly_the_governance_file_set the_producer_emits_no_agent_or_environment_artifact the_producer_performs_no_io the_scaffold_ignores_the_wall_clock_artifact the_scaffolded_config_names_the_knobs_adopters_needed the_scaffolded_config_round_trips_a_non_default_configuration the_scaffolded_config_round_trips_to_its_own_defaults the_scaffolded_contract_carries_the_lifecycle_table_and_extra_keys the_scaffolded_default_hashes_files_not_directories the_shard_trees_are_not_ignored_and_the_choice_is_explained the_statecraft_layout_scaffolds_coherently 2>&1 | grep -q "test result: ok. 21 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss120-scaffold.txt"
# 3.6: the gate is repository-owned, names no kit, and runs. COUPLE=0 because
# this line has no base to diff against; the coupling half runs in CI and in
# the gate the session runs before every commit.
test -f Makefile
! grep -rqF 'kit/Makefile' .github/workflows/
! grep -qF 'kit/Makefile' AGENTS.md
! grep -qF 'kit/Makefile' CLAUDE.md
# 3.7: and nothing still points at the old path. Asserted through git rather
# than by grepping prose: a tracked file under `.derived/` is the failure, and
# a sentence mentioning the old path in a design note is not.
test "$(git ls-files .derived | wc -l | tr -d ' ')" = 0
! grep -qF ' .derived/' .gitattributes
! grep -qE '^\.derived/' .gitignore
make gate SPEC_SPINE=target/release/spec-spine COUPLE=0
# 3.6: an unrecognised control word is still refused rather than read as a
# default, which is spec 094's rule and the reason the file moved intact.
! make gate SPEC_SPINE=target/release/spec-spine OWNERSHIP=yes COUPLE=0
! make gate SPEC_SPINE=target/release/spec-spine COUPLE=maybe
# 3.7: the committed trees are at the configured path and the old one is gone.
! test -e .derived
test -d .statecraft/derived/spec-registry/by-spec
test -d .statecraft/derived/codebase-index/by-spec
target/release/spec-spine config show > "${TMPDIR:-/tmp}/ss120-cfg.txt"
grep -qF 'derived_dir = ".statecraft/derived"' "${TMPDIR:-/tmp}/ss120-cfg.txt"
grep -qF 'state_dir = ".statecraft/state"' "${TMPDIR:-/tmp}/ss120-cfg.txt"
rm -f "${TMPDIR:-/tmp}/ss120-cfg.txt"
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine index coverage --fail-on-untraced
target/release/spec-spine lint --fail-on-warn
# 3.9: .statecraft is not wholesale ignored. The derived ledger is tracked and
# the state root is not, asserted through git rather than by reading the file.
! git check-ignore -q .statecraft/derived/spec-registry/by-spec/092-the-engine-ships-governance-not-an-environment.json
git check-ignore -q .statecraft/state/anything
git check-ignore -q .statecraft/derived/spec-registry/build-meta.json
git ls-files --error-unmatch .statecraft/derived/spec-registry/by-spec/000-spec-spine-bootstrap.json > /dev/null
# 3.8 and 3.9: the mechanism, on a fixture whose derived tree is at the new
# path. A source file under `.statecraft/` is governed; one under the state
# root and one under the derived tree are not. Built from nothing each run, so
# an empty or default corpus cannot produce the green.
# exact on a Unix sweep host with default features: index has tests behind the default `symbol-resolution` feature (spec 153 3.3).
sh -c 'cargo test -p spec-spine-core --locked --test index -- --exact statecraft_derived_and_state_are_pruned_and_the_rest_is_governed statecraft_derived_matching_is_separator_aware 2>&1 | grep -q "test result: ok. 2 passed; 0 failed"'
sh -c 'cargo test -p spec-spine-core --locked --test couple -- --exact a_regenerated_shard_at_the_configured_derived_root_is_not_drift the_configured_derived_root_is_bypassed_and_the_default_is_not_a_synonym the_effective_bypass_list_reports_the_configured_derived_root 2>&1 | grep -q "test result: ok. 3 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss120-idx.txt" "${TMPDIR:-/tmp}/ss120-cpl.txt"
# 3.7: a relocated tree is still refused when it is stale, missing or has a
# stray shard. Run end to end against a scratch corpus at the new path.
sh -c 'cargo test -p spec-spine-cli --locked --test cli -- --exact statecraft_derived_layout_compiles_indexes_and_is_judged 2>&1 | grep -q "test result: ok. 1 passed; 0 failed"'
rm -f "${TMPDIR:-/tmp}/ss120-cli.txt"
# 3.6: the suites that replaced the removed ones still assert what they
# asserted about the files this repository still has.
cargo test -p spec-spine-core --test gate --locked
cargo test -p spec-spine-core --test harness_hooks --locked
cargo test -p spec-spine-core --test harness_skills --locked
# 3.11: nothing claims a removed path. Asserted through the governed read
# rather than by grepping frontmatter: a claim on a path that does not exist is
# an unresolved claim, and `--fail-on-unresolved` above is exactly the refusal
# for it. A grep would also match its own line in this block, which is how spec
# 071's first attempt at a pattern assertion passed against itself.
target/release/spec-spine index diagnostics > "${TMPDIR:-/tmp}/ss120-diag.txt" 2>&1
test ! -s "${TMPDIR:-/tmp}/ss120-diag.txt"
rm -f "${TMPDIR:-/tmp}/ss120-diag.txt"
# 3.11 rule 3: the spec left owning nothing after the withdrawal was 029, whose
# whole territory was `kit/`. Spec 095's collapse removed it outright rather
# than leaving it `superseded` and empty, and `docs/corpus-map.md` is where it
# is accounted for now.
grep -qF '029-claude-code-skill-kit' docs/corpus-map.md
! test -e specs/029-claude-code-skill-kit
# 3.13: the withdrawal is carried by the successor's text, in a document every
# clone has. The three lines this replaces asserted the existence of a local
# branch and of a blob at a commit on it, neither of which was ever pushed: they
# passed in one checkout and failed in every other, including CI's. What is
# asserted instead is that spec 093 states the requirement and that the gate
# body actually asks the question, which is the fact the withdrawal was
# protecting. D-7.
grep -qF 'The derived-tree question is asked in every state git distinguishes' specs/093-the-harness-this-repository-runs/spec.md
! grep -qF 'diff --quiet -- .statecraft/derived/' .claude/settings.json
# Declared and read through the CLI, redirected rather than piped (spec 085 D-4).
target/release/spec-spine registry show 092 --json > "${TMPDIR:-/tmp}/ss120-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss120-show.json')); assert d['id'] == '092-the-engine-ships-governance-not-an-environment', d"
rm -f "${TMPDIR:-/tmp}/ss120-show.json"
# The stack's own gate, last, because a green governance loop over code that
# does not compile asserts nothing.
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
# ---- carried for 118-the-renumber-is-history-not-a-standing-rule (amends_verification), and through it 095; `184` joins the exempt prefixes of the citation check (D-3) ----
cargo build --release --locked
# --- 095's acceptance, carried ---
# 095 3.1, 3.2: none of the twenty-seven remains, by directory and by registry.
! test -e specs/006-init-scaffold
! test -e specs/029-claude-code-skill-kit
! test -e specs/046-kit-hooks-read-never-write
! test -e specs/064-the-kit-ships-the-composite-gate
! test -e specs/100-one-source-generates-the-agent-trees
! test -e specs/116-shepherd-reads-every-reviewer
# 095 3.3: every reference resolves. A dangling id is a compile error (V-004
# family), and an unresolved unit is what `check` refuses.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
# 095 3.3: no removed id is cited outside the specs that account for it. 118
# joins the class because this block spells the ids (D-4).
sh -c 'for id in 006-init-scaffold 029-claude-code-skill-kit 046-kit-hooks-read-never-write 064-the-kit-ships-the-composite-gate 100-one-source-generates-the-agent-trees 116-shepherd-reads-every-reviewer; do if grep -rlF "$id" specs crates .claude/skills .claude/agents AGENTS.md CLAUDE.md README.md 2>/dev/null | grep -qvE "^specs/(09[23456]|118|184)-"; then echo "still cited: $id"; exit 1; fi; done; exit 0'
# 095 3.2 and 3.4: every removed spec is named by a successor and is in the map.
test "$(grep -cE '^\| `[0-9]{3}-[a-z0-9-]+` \|' docs/corpus-map.md)" -ge 27
sh -c 'n=0; for f in specs/*/spec.md; do n=$((n + $(grep -cE "^- \`[0-9]{3}-[a-z0-9-]+\`$" "$f"))); done; test "$n" -eq 27 || { echo "predecessors named: $n, want 27"; exit 1; }'
# 095 3.5: the sweep's ledger is live, closed, and every entry names a real spec.
grep -qE '^readonly LEDGER_CLOSED_AT=[0-9]+$' scripts/verify-sweep.sh
bash -n scripts/verify-sweep.sh
# 095 3.5: nothing in the repository still carries the removed product (D-3).
! test -e kit
test -z "$(git ls-files -- kit .agents .codex)"
# And the whole loop, over the corpus.
target/release/spec-spine index coverage --fail-on-untraced
make gate SPEC_SPINE=target/release/spec-spine COUPLE=0
cargo test --workspace --locked
# --- 118 3.1: the superseded form is still in 095's own file ---
grep -qF 'assert o==list(range(len(o)))' specs/095-the-corpus-describes-what-exists/spec.md
# --- 118 3.2: the historical renumber, read from the map ---
python3 -c 'import re,sys; t=open(sys.argv[1]).read(); s2=t.split("\n## 2.")[1].split("\n## 3.")[0]; s3=t.split("\n## 3.")[1].split("\n## 4.")[0]; new=re.findall(r"^\| `[0-9]{3}-[a-z0-9-]+` \| `([0-9]{3}-[a-z0-9-]+)` \|$", s2, re.M)+re.findall(r"^- `([0-9]{3}-[a-z0-9-]+)`$", s3, re.M); o=sorted(int(i[:3]) for i in new); assert len(set(o))==len(o), "a renumbered ordinal repeats"; assert o==list(range(len(o))), "the renumber left a gap"; assert o[-1]==95, o[-1]; print(len(o), "ids after the renumber, contiguous 000 to 095")' docs/corpus-map.md
python3 -c 'import re,subprocess; t=open("docs/corpus-map.md").read(); s2=t.split("\n## 2.")[1].split("\n## 3.")[0]; s3=t.split("\n## 3.")[1].split("\n## 4.")[0]; new=set(re.findall(r"^\| `[0-9]{3}-[a-z0-9-]+` \| `([0-9]{3}-[a-z0-9-]+)` \|$", s2, re.M)+re.findall(r"^- `([0-9]{3}-[a-z0-9-]+)`$", s3, re.M)); ids=set(subprocess.check_output(["target/release/spec-spine","registry","list","--ids-only"], text=True).split()); gone=sorted(new-ids); assert new, "no renumbered ids parsed from docs/corpus-map.md"; assert not gone, ("renumbered ids missing from the corpus", gone)'
# The checker can fail: one survivor row removed leaves a gap...
sh -c 'mkdir -p "${TMPDIR:-/tmp}/ss118" && grep -vF "| \`050-" docs/corpus-map.md > "${TMPDIR:-/tmp}/ss118/map-gap.md" && ! cmp -s docs/corpus-map.md "${TMPDIR:-/tmp}/ss118/map-gap.md"'
! python3 -c 'import re,sys; t=open(sys.argv[1]).read(); s2=t.split("\n## 2.")[1].split("\n## 3.")[0]; s3=t.split("\n## 3.")[1].split("\n## 4.")[0]; new=re.findall(r"^\| `[0-9]{3}-[a-z0-9-]+` \| `([0-9]{3}-[a-z0-9-]+)` \|$", s2, re.M)+re.findall(r"^- `([0-9]{3}-[a-z0-9-]+)`$", s3, re.M); o=sorted(int(i[:3]) for i in new); assert len(set(o))==len(o), "a renumbered ordinal repeats"; assert o==list(range(len(o))), "the renumber left a gap"; assert o[-1]==95, o[-1]' "${TMPDIR:-/tmp}/ss118/map-gap.md"
# ...and one row duplicated repeats an ordinal.
sh -c 'mkdir -p "${TMPDIR:-/tmp}/ss118" && awk "{print} /\\| \`050-/{print}" docs/corpus-map.md > "${TMPDIR:-/tmp}/ss118/map-dup.md" && test "$(grep -c "| \`050-" "${TMPDIR:-/tmp}/ss118/map-dup.md")" -gt "$(grep -c "| \`050-" docs/corpus-map.md)"'
! python3 -c 'import re,sys; t=open(sys.argv[1]).read(); s2=t.split("\n## 2.")[1].split("\n## 3.")[0]; s3=t.split("\n## 3.")[1].split("\n## 4.")[0]; new=re.findall(r"^\| `[0-9]{3}-[a-z0-9-]+` \| `([0-9]{3}-[a-z0-9-]+)` \|$", s2, re.M)+re.findall(r"^- `([0-9]{3}-[a-z0-9-]+)`$", s3, re.M); o=sorted(int(i[:3]) for i in new); assert len(set(o))==len(o), "a renumbered ordinal repeats"; assert o==list(range(len(o))), "the renumber left a gap"; assert o[-1]==95, o[-1]' "${TMPDIR:-/tmp}/ss118/map-dup.md"
# --- 118 3.3: the live corpus, by rules that survive growth ---
python3 -c 'import re,subprocess; ids=subprocess.check_output(["target/release/spec-spine","registry","list","--ids-only"], text=True).split(); bad=[i for i in ids if not re.fullmatch(r"[0-9]{3}-[a-z0-9]+(-[a-z0-9]+)*", i)]; assert ids and not bad, bad; o=[int(i[:3]) for i in ids]; assert all(a<b for a,b in zip(o,o[1:])), "ordinals repeat or are out of order"; print(len(ids), "ids, unique and ordered; gaps permitted")'
test "$(ls -d specs/*/ | sed 's#^specs/##; s#/$##' | sort)" = "$(target/release/spec-spine registry list --ids-only | sort)"
# --- 118 3.4: fixture corpora, each isolated under this block's TMPDIR ---
# 1. A reserved later ordinal is legitimate.
sh -c 'r="${TMPDIR:-/tmp}/ss118/gap"; rm -rf "$r"; for s in 000-first 001-second 117-after-a-reserved-gap; do mkdir -p "$r/specs/$s" && printf -- "---\nid: \"%s\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n---\n\n# t\n" "$s" > "$r/specs/$s/spec.md" || exit 1; done'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/gap" compile
test "$(target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/gap" registry list --ids-only | tr '\n' ' ')" = "000-first 001-second 117-after-a-reserved-gap "
# 2. Two specs sharing an ordinal are refused.
sh -c 'r="${TMPDIR:-/tmp}/ss118/dup"; rm -rf "$r"; for s in 117-a 117-b; do mkdir -p "$r/specs/$s" && printf -- "---\nid: \"%s\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n---\n\n# t\n" "$s" > "$r/specs/$s/spec.md" || exit 1; done'
sh -c 'target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/dup" compile >/dev/null 2>&1; test $? -eq 1'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/dup" compile 2>&1 | grep -q "V-004"
# 3. An id outside the grammar is refused.
sh -c 'r="${TMPDIR:-/tmp}/ss118/bad"; rm -rf "$r"; mkdir -p "$r/specs/17-short" && printf -- "---\nid: \"17-short\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n---\n\n# t\n" > "$r/specs/17-short/spec.md"'
sh -c 'target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/bad" compile >/dev/null 2>&1; test $? -eq 1'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/bad" compile 2>&1 | grep -q "V-012"
# 4. A directory that does not equal its id is refused.
sh -c 'r="${TMPDIR:-/tmp}/ss118/dir"; rm -rf "$r"; mkdir -p "$r/specs/118-x" && printf -- "---\nid: \"118-y\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n---\n\n# t\n" > "$r/specs/118-x/spec.md"'
sh -c 'target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/dir" compile >/dev/null 2>&1; test $? -eq 1'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/dir" compile 2>&1 | grep -q "V-001"
rm -rf "${TMPDIR:-/tmp}/ss118"
```
