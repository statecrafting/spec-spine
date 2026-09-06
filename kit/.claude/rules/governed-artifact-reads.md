# Governed artifact reads

The compiled artifacts under the derived directory are read **only** through
`spec-spine` subcommands (`registry`, `index`), never via ad-hoc `jq`, `grep`,
`python`, `awk`, or `sed` over the JSON. Typed reads make schema drift fail at
the deserializer with a clean error instead of silently encoding stale
assumptions.

Parsing the *output* of a `spec-spine` subcommand (for example
`spec-spine registry plan --json`, or the `--json` verdict envelope any gate
verb emits) is a typed read and is allowed: the tool has already deserialized
the shards and is answering in a contract it versions. The rule is about the
shard files, not about the CLI's answers.
