# The corpus map

Spec 095 collapsed what spec 092's removal of the kit left behind. This file is
the map from every spec id the corpus used to hold to the one that holds its
requirements now, and from every old ordinal to its new one.

It exists because a citation outlives the document it cites. Git history still
carries every removed spec, every merged pull request names them, and four
adopter repositories pin releases whose notes do too. None of that is
rewritten. Without this file a reader who finds "spec 048" has no way to learn
what became of it, and after the renumber a bare old ordinal is worse than a
dangling one: it resolves, to a different document.

## 1. Removed specs, and where their requirements went

| Removed | Title it carried | Requirements now in |
|---|---|---|
| `006-init-scaffold` | init: scaffold a new adopter | `095-the-corpus-describes-what-exists` |
| `020-derived-artifact-merge-driver` | Derived-artifact merge driver and merge-queue serialization | `094-one-gate-and-the-boundaries-it-holds` |
| `029-claude-code-skill-kit` | Claude Code skill kit: a vendored, governed adoption bundle | `095-the-corpus-describes-what-exists` |
| `046-kit-hooks-read-never-write` | The kit's hooks observe the tree; they do not repair it | `093-the-harness-this-repository-runs` |
| `047-harness-rules-name-the-legitimate-edits` | The harness rules name the reads and the edits they permit | `093-the-harness-this-repository-runs` |
| `048-kit-ships-the-governed-loop-skills` | The kit ships one skill set for the governed loop | `093-the-harness-this-repository-runs` |
| `051-harness-runs-the-verbs-it-ships` | The harness runs the verbs it ships | `093-the-harness-this-repository-runs` |
| `063-a-stale-binary-is-not-a-stale-ledger` | A stale binary is not a stale ledger | `093-the-harness-this-repository-runs` |
| `064-the-kit-ships-the-composite-gate` | The kit ships the composite gate and the merge driver | `094-one-gate-and-the-boundaries-it-holds` |
| `065-init-and-the-kit-are-one-adoption` | Init and the kit are one adoption | `095-the-corpus-describes-what-exists` |
| `068-a-path-scoped-rule-example` | A path-scoped rule the kit actually ships | `093-the-harness-this-repository-runs` |
| `071-a-tag-push-is-not-a-push-to-main` | A tag push is not a push to main | `093-the-harness-this-repository-runs` |
| `072-the-default-branch-is-configured-not-assumed` | The default branch is configured, not assumed | `093-the-harness-this-repository-runs` |
| `078-the-protocol-has-an-owner` | The protocol has an owner | `093-the-harness-this-repository-runs` |
| `080-a-gate-that-cannot-ask-says-so` | A gate that cannot ask says so | `093-the-harness-this-repository-runs` |
| `081-the-kit-ships-what-the-loop-calls` | The kit ships what the loop calls | `093-the-harness-this-repository-runs` |
| `082-a-refusal-is-not-a-remediation-round` | A refusal is not a remediation round | `093-the-harness-this-repository-runs` |
| `089-a-skip-and-a-failure-are-different-answers` | A skip and a failure are different answers | `094-one-gate-and-the-boundaries-it-holds` |
| `090-a-hook-bound-to-a-tool-route-misses-the-work` | A hook bound to a tool route misses the work | `094-one-gate-and-the-boundaries-it-holds` |
| `099-the-session-hooks-report-the-verdict` | The session hooks report the verdict, not a guess | `093-the-harness-this-repository-runs` |
| `100-one-source-generates-the-agent-trees` | One source generates the agent instruction trees | `095-the-corpus-describes-what-exists` |
| `104-every-hook-reads-the-code-the-same-way` | Every hook reads the exit code the same way | `093-the-harness-this-repository-runs` |
| `110-a-refusal-names-the-branch-it-resolved` | A refusal names the branch it resolved | `093-the-harness-this-repository-runs` |
| `113-the-scaffolded-protocol-is-the-gate-the-kit-ships` | The scaffolded protocol is the gate the kit ships | `095-the-corpus-describes-what-exists` |
| `114-one-gate-definition-that-holds-on-a-code-free-corpus` | One gate definition, and it holds on a code-free corpus | `094-one-gate-and-the-boundaries-it-holds` |
| `115-the-kit-ships-no-claim-an-adopter-cannot-resolve` | The kit ships no claim an adopter cannot resolve | `095-the-corpus-describes-what-exists` |
| `116-shepherd-reads-every-reviewer` | Shepherd reads every reviewer | `093-the-harness-this-repository-runs` |

The six mapped to `095-the-corpus-describes-what-exists` have **no** surviving requirement: each described a
surface the realignment removed entirely, and that spec's 3.1 says so path by
path.

## 2. Every other spec: old ordinal to new

Renumbered in place, chronological order preserved, so a spec filed earlier
keeps a lower ordinal than one filed later. A row whose two cells are equal is
a spec that did not move.

| Was | Is |
|---|---|
| `000-spec-spine-bootstrap` | `000-spec-spine-bootstrap` |
| `001-compile-registry` | `001-compile-registry` |
| `002-registry-query` | `002-registry-query` |
| `003-conformance-lint` | `003-conformance-lint` |
| `004-codebase-index` | `004-codebase-index` |
| `005-coupling-gate` | `005-coupling-gate` |
| `007-distribution` | `006-distribution` |
| `008-python-distribution` | `007-python-distribution` |
| `009-coupling-floor-claim-precedence` | `008-coupling-floor-claim-precedence` |
| `010-registry-query-projection-flags` | `009-registry-query-projection-flags` |
| `011-index-render-orphans` | `010-index-render-orphans` |
| `012-index-hash-slices` | `011-index-hash-slices` |
| `013-declared-extra-frontmatter-passthrough` | `012-declared-extra-frontmatter-passthrough` |
| `014-edge-paths-grammar-sugar` | `013-edge-paths-grammar-sugar` |
| `015-establishes-wrapper-na-alias` | `014-establishes-wrapper-na-alias` |
| `016-short-id-resolution` | `015-short-id-resolution` |
| `017-directory-crate-module-units` | `016-directory-crate-module-units` |
| `018-constrains-discriminator-optional-unit` | `017-constrains-discriminator-optional-unit` |
| `019-structured-partial-supersedes` | `018-structured-partial-supersedes` |
| `021-release-supply-chain-artifacts` | `019-release-supply-chain-artifacts` |
| `022-keypath-section-anchors` | `020-keypath-section-anchors` |
| `023-ledger-seal` | `021-ledger-seal` |
| `024-index-sharding` | `022-index-sharding` |
| `025-unresolved-unit-severity` | `023-unresolved-unit-severity` |
| `026-resolution-discovery-fixes` | `024-resolution-discovery-fixes` |
| `027-symbol-resolution-feature-gate` | `025-symbol-resolution-feature-gate` |
| `028-references-provenance-derived-at` | `026-references-provenance-derived-at` |
| `030-cargo-workflow-dependency-waiver` | `027-cargo-workflow-dependency-waiver` |
| `031-registry-freshness-check` | `028-registry-freshness-check` |
| `032-ownership-coverage` | `029-ownership-coverage` |
| `033-dependency-cycle-refusal` | `030-dependency-cycle-refusal` |
| `034-references-non-owning-paths` | `031-references-non-owning-paths` |
| `035-stdout-closed-reader` | `032-stdout-closed-reader` |
| `036-configured-corpus-root` | `033-configured-corpus-root` |
| `037-machine-readable-verdicts` | `034-machine-readable-verdicts` |
| `038-registry-plan-ready-set` | `035-registry-plan-ready-set` |
| `039-declared-state-dir` | `036-declared-state-dir` |
| `040-amendment-authoring` | `037-amendment-authoring` |
| `041-completion-held-to-claims` | `038-completion-held-to-claims` |
| `042-per-spec-attestation` | `039-per-spec-attestation` |
| `043-governance-document-gaps` | `040-governance-document-gaps` |
| `044-in-progress-is-in-flight` | `041-in-progress-is-in-flight` |
| `045-absent-implementation-defers-to-status` | `042-absent-implementation-defers-to-status` |
| `049-verify-declared-acceptance` | `043-verify-declared-acceptance` |
| `050-index-diagnostics-reach-a-gate` | `044-index-diagnostics-reach-a-gate` |
| `052-couple-names-the-crossing` | `045-couple-names-the-crossing` |
| `053-depends-on-ordinal-monotonicity` | `046-depends-on-ordinal-monotonicity` |
| `054-effective-config-is-a-governed-read` | `047-effective-config-is-a-governed-read` |
| `055-the-ledger-answers-what-consumers-rebuild` | `048-the-ledger-answers-what-consumers-rebuild` |
| `056-compile-one-spec` | `049-compile-one-spec` |
| `057-claimed-but-unwitnessed` | `050-claimed-but-unwitnessed` |
| `058-retroactive-adoption-shape` | `051-retroactive-adoption-shape` |
| `059-read-verbs-on-a-code-free-corpus` | `052-read-verbs-on-a-code-free-corpus` |
| `060-plan-answers-the-whole-question` | `053-plan-answers-the-whole-question` |
| `061-the-scaffold-ships-what-adopters-wrote` | `054-the-scaffold-ships-what-adopters-wrote` |
| `062-a-version-pin-the-cli-can-check` | `055-a-version-pin-the-cli-can-check` |
| `066-the-contract-records-the-lifecycle-table` | `056-the-contract-records-the-lifecycle-table` |
| `067-the-docs-name-what-adopters-derived` | `057-the-docs-name-what-adopters-derived` |
| `069-the-shipped-default-hashes-what-it-names` | `058-the-shipped-default-hashes-what-it-names` |
| `070-a-malformed-id-is-refused-not-a-panic` | `059-a-malformed-id-is-refused-not-a-panic` |
| `073-a-workflow-bump-is-not-a-governed-change` | `060-a-workflow-bump-is-not-a-governed-change` |
| `074-shipped-is-not-the-same-as-working` | `061-shipped-is-not-the-same-as-working` |
| `075-one-name-one-freshness-verb` | `062-one-name-one-freshness-verb` |
| `076-planned-territory-is-declared-not-inferred` | `063-planned-territory-is-declared-not-inferred` |
| `077-compile-warnings-reach-the-gate` | `064-compile-warnings-reach-the-gate` |
| `079-a-dead-glob-is-dead-in-both-tables` | `065-a-dead-glob-is-dead-in-both-tables` |
| `083-an-attestation-covers-the-territory-it-claims` | `066-an-attestation-covers-the-territory-it-claims` |
| `084-a-short-id-names-the-same-spec-at-every-verb` | `067-a-short-id-names-the-same-spec-at-every-verb` |
| `085-a-verifier-checks-the-bytes-it-was-given` | `068-a-verifier-checks-the-bytes-it-was-given` |
| `086-the-committed-index-is-compared-not-trusted` | `069-the-committed-index-is-compared-not-trusted` |
| `087-an-authority-snapshot-says-what-it-read` | `070-an-authority-snapshot-says-what-it-read` |
| `088-a-change-is-classified-under-the-bases-rules` | `071-a-change-is-classified-under-the-bases-rules` |
| `091-two-ready-specs-can-collide` | `072-two-ready-specs-can-collide` |
| `092-a-mode-only-or-binary-change-is-a-change` | `073-a-mode-only-or-binary-change-is-a-change` |
| `093-a-governed-read-names-its-version` | `074-a-governed-read-names-its-version` |
| `094-a-claim-below-the-header-window-is-not-silent` | `075-a-claim-below-the-header-window-is-not-silent` |
| `095-a-stray-shard-is-orphaned-at-the-verbs` | `076-a-stray-shard-is-orphaned-at-the-verbs` |
| `096-one-hash-one-construction-one-name` | `077-one-hash-one-construction-one-name` |
| `097-governed-scope-is-declared-not-inferred` | `078-governed-scope-is-declared-not-inferred` |
| `098-a-blocking-claim-is-not-a-stale-shard` | `079-a-blocking-claim-is-not-a-stale-shard` |
| `101-an-unresolved-claim-is-not-stale` | `080-an-unresolved-claim-is-not-stale` |
| `102-coupling-sees-the-change-being-committed` | `081-coupling-sees-the-change-being-committed` |
| `103-an-amended-acceptance-is-the-one-that-runs` | `082-an-amended-acceptance-is-the-one-that-runs` |
| `105-an-amendment-carries-the-acceptance-it-replaces` | `083-an-amendment-carries-the-acceptance-it-replaces` |
| `106-an-acceptance-outlives-the-output-it-was-written-against` | `084-an-acceptance-outlives-the-output-it-was-written-against` |
| `107-a-version-pin-is-not-a-contract` | `085-a-version-pin-is-not-a-contract` |
| `108-an-exact-key-set-refuses-what-the-rule-allows` | `086-an-exact-key-set-refuses-what-the-rule-allows` |
| `109-the-answer-is-a-member-not-the-document` | `087-the-answer-is-a-member-not-the-document` |
| `111-the-template-teaches-the-whole-grammar` | `088-the-template-teaches-the-whole-grammar` |
| `112-nothing-reruns-a-merged-acceptance` | `089-nothing-reruns-a-merged-acceptance` |
| `118-the-verdict-is-the-only-thing-on-stdout` | `090-the-verdict-is-the-only-thing-on-stdout` |
| `119-an-unclassified-review-failure-blocks-the-merge` | `091-an-unclassified-review-failure-blocks-the-merge` |
| `120-the-engine-ships-governance-not-an-environment` | `092-the-engine-ships-governance-not-an-environment` |

## 3. The three specs filed by the collapse

- `093-the-harness-this-repository-runs`
- `094-one-gate-and-the-boundaries-it-holds`
- `095-the-corpus-describes-what-exists`

## 4. Reading a citation

A citation of a removed spec names a document that is in git and not in the
corpus; 1 says which spec to read instead. A citation of a surviving spec
written before the renumber names an ordinal that has moved; 2 says where.
