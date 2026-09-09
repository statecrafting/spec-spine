---
name: research
description: "Deep research with parallel sub-agents, query classification, and filesystem artifact passing; corpus questions read specs through spec-spine, external questions use the web."
allowed-tools: Agent, Read, Write, Bash(git log:*), Bash(git diff:*), Bash(spec-spine:*), WebSearch, WebFetch, Glob, Grep
argument-hint: "<question or topic to investigate>"
---

# Research

Conduct deep, parallel research on a topic using specialized sub-agents.
Agents cost tokens: classify first, spawn the fewest that answer the
question, and pass reports through files rather than inline.

## Research query

`$ARGUMENTS`

## Phase 1: classify

| Type | Characteristics | Sub-agents | Depth each |
|---|---|---|---|
| Breadth-first | several independent aspects, surveys, comparisons | 3 to 6 | 5 to 10 searches |
| Depth-first | one topic needing thorough understanding | 2 to 3 | 10 to 15 searches |
| Simple factual | one fact, one lookup | 1 | 3 to 5 searches |

Decide: query type, agent count, domains (corpus, codebase, external
docs, papers, general web), and scope (corpus-only, web-only, hybrid).

- **Corpus questions** ("what does spec 020 say about erasure", "who owns
  this file", "what depends on 064", "how does the compiler validate
  frontmatter"): use the `explorer` agent with
  `spec-spine registry show|relationships <id>`, `spec-spine index render`,
  `spec-spine index coverage`, `Grep` over `specs/` and the source, and
  `git log`. Never parse `.derived/` directly.
- **External questions** (a protocol, a library's semantics, an RFC):
  `WebSearch` and `WebFetch`, preferring primary sources (specifications,
  RFCs, the library's own docs).
- Many questions are hybrid; split them across agents by domain.

## Phase 2: parallel execution

Spawn all sub-agents in one message. Each prompt begins with a depth
trigger: "Quick check:", "Investigate:", or "Deep dive:".

Each sub-agent MUST write its full report to the session scratchpad
directory when one is listed in the system prompt (otherwise a
`research/` directory under the tool-state root `spec-spine.toml
[layout] state_dir` names, never committed) as
`research_<timestamp>_<slug>.md` and return only: the file path, a two to
three sentence summary, key topics, and the source count.

Example, hybrid ("how should the new spec bind an identity key to the
transport certificate?"):

```
Task 1: "Deep dive: raw public key TLS (RFC 7250) support in the libraries this stack uses"
Task 2: "Investigate: what the existing specs already fix about identity binding; use spec-spine registry show and relationships"
```

Example, corpus-only ("trace the spec compiler's validation pipeline"):

```
Task 1: "Investigate: which spec owns the validation code (spec-spine index coverage, registry show), then read it and list the diagnostics it can emit in order"
```

## Phase 3: synthesis

Collect the report paths, read them, merge (themes, deduplication,
contradictions flagged), consolidate sources, and write the final report to
the same directory as `research_final_<timestamp>.md`.

## Phase 4: deliver

```markdown
# Research Report: <topic>
## Executive Summary
## Key Findings
## Detailed Analysis
## Implications for the corpus
(which spec would change, and whether that is an amendment to a shipped
spec, which invalidates dependents, or an edit to a pending one)
## Sources
## Metadata (classification, agents, source count, artifact paths)
```

Show the summary and key findings inline, give the report path, list the
sub-agent report paths, and name contradictions and gaps explicitly.

## Quality

Prefer primary sources; cross-reference important claims; state what could
not be determined; separate fact from inference; prefer recent sources and
flag stale ones. Never use the em dash character in any written artifact.

## Project layer

Nothing here is project-specific. External topics come from the question;
the report location comes from the harness or `spec-spine.toml`.
