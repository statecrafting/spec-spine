---
id: quickstart
title: Quickstart
sidebar_position: 2
---

# Quickstart

This guide walks you through setting up a spec-spine corpus by hand, compiling the registry, indexing your codebase, and running the coupling gate. It assumes you have already [installed spec-spine](installation.md).

You can run this end-to-end in an empty repository to see the mechanics in action.

## 1. Create the corpus

spec-spine has no initialization command: it governs a repository, it does not set one up. The [Statecraft CLI](../statecraft.md) is what initializes a managed project, and it calls spec-spine's library to produce the governance starter files. For this walkthrough you only need a configuration file and a specs directory, both of which you can write yourself.

Create `spec-spine.toml` at the root of your repository:

```toml
[layout]
specs_dir = "specs"
standards_dir = "standards/spec"
derived_dir = ".derived"
```

Those are the defaults, so an empty file works too. See [Configuration](../configuration.md) for the full set of knobs.

```bash
mkdir -p specs standards/spec
```

A production corpus also carries `standards/spec/constitution.md` (the tier-2 durable principles) and a tier-1 bootstrap spec at `specs/000-bootstrap/spec.md`. Both are part of what the library producer emits, and neither is needed to follow the steps below.

## 2. Author a spec

Create a new specification file for a feature. In this example, we will create `specs/001-hello-world/spec.md`.

```bash
mkdir -p specs/001-hello-world
```

Create the file `specs/001-hello-world/spec.md` with the following content:

```markdown
---
status: approved
establishes:
  - src/main.rs
---

# Hello World

This spec establishes the main entry point for the application.
```

This frontmatter declares that the spec `001-hello-world` owns the file `src/main.rs` via an `establishes` edge.

## 3. Create the code

Now, create the file that the spec claims to own.

```bash
mkdir -p src
```

Create `src/main.rs` and add a comment header linking it back to the spec:

```rust
// Spec: specs/001-hello-world/spec.md

fn main() {
    println!("Hello, world!");
}
```

## 4. Compile the registry

The compiler reads the markdown corpus and emits a frozen JSON registry. This is the spec-as-source view.

```bash
spec-spine compile
```

You will see output indicating that the registry shards have been written to `.derived/spec-registry/by-spec/`.

## 5. Index the codebase

The indexer scans the repository for manifests and code files, mapping them back to their owning specs. This is the code-as-source view.

```bash
spec-spine index
```

The index shards are written to `.derived/codebase-index/by-spec/` and `.../by-package/`.

## 6. Run the coupling gate

The coupling gate joins the registry and the index against a Git diff. It refuses the merge if a path is modified without its owning spec also being modified (or vice versa).

First, commit your changes so the gate has a baseline to compare against:

```bash
git add .
git commit -m "Initial commit with spec and code"
```

Now, make a modification to `src/main.rs` without updating the spec:

```rust
// Spec: specs/001-hello-world/spec.md

fn main() {
    println!("Hello, modified world!");
}
```

Run the coupling gate against the `HEAD` commit:

```bash
spec-spine couple --base HEAD --head HEAD
```

*(Note: In a real CI environment, `--base` would be `origin/main` and `--head` would be the PR branch. Here we use `HEAD` to simulate a local diff.)*

Because we modified `src/main.rs` without modifying `specs/001-hello-world/spec.md`, the gate will exit with status code `1` and report a drift violation.

To resolve the drift, you must either modify the spec to reflect the code change, or provide a waiver in the PR body.

## Next steps

- Read the [Concepts](../concepts/overview.md) to understand the authority graph.
- Explore the [CLI Reference](../cli/overview.md) for detailed command usage.
- See the [Adoption Guide](../adoption-guide.md) for integrating spec-spine into an existing project.
