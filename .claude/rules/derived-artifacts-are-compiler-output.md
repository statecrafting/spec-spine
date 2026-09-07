---
paths:
  - ".derived/**"
---

# Derived artifacts are compiler output

The files under this directory are emitted by `spec-spine compile` and
`spec-spine index`. They are machine truth, not authored truth.

**Do not hand-edit one.** A shard is a pure function of the corpus and the
source tree; editing it makes the ledger disagree with what it describes, and
the next `compile --check` or `index check` reports it as staleness with no clue
that a person put it there. The way to change a shard is to change its input and
regenerate.

**Do not read one with `jq`, `grep`, `sed`, `awk` or `python`.** An ad-hoc
parser encodes today's shape and then goes quietly wrong when the schema moves.
Read through a `spec-spine` subcommand instead: `registry show`, `registry
list`, `registry plan`, `index render`, `index owner`, `index coverage`,
`index diagnostics`. A typed read fails at the deserializer with a clean error
rather than silently returning the wrong answer.

**Parsing the output of a subcommand is fine.** `spec-spine registry plan
--json`, or the `--json` verdict envelope any gate verb emits, is a typed read:
the tool has already deserialized the shards and is answering in a contract it
versions. The rule is about the shard files, not about the CLI's answers.

---

**This rule does not replace `governed-artifact-reads.md`, and cannot.**

That rule is unconditional, and it has to be. The mistake it prevents is
reaching for `jq` **instead of** the subcommand, and an agent about to make that
mistake may never open a file under this directory, so a rule scoped to these
paths would never load. The unconditional rule is what prevents the mistake.
This one reinforces it at the moment somebody actually has a shard open.

If the two look redundant, the redundant-looking one is the one doing the work.
