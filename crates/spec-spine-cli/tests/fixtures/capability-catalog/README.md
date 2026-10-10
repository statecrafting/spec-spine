# Capability-catalog example fixtures (spec 162 §3.9)

Each directory is a repository the CLI suite copies, compiles, indexes and
commits before running the catalog's examples against the built binary.

- `corpus/`: two specs (one approved and complete, one draft), a claimed and
  an unclaimed source file, and the request documents the examples name.
- `pinned/`: a configuration whose `required_version` no build meets, so
  every pinned operation refuses (exit 2). It is not compiled.
- `history/`: `base/` is committed first, then `head/` is laid over it and
  committed, so `couple` and `delta` have a range to judge.

A file named `*.fixture` is copied without that suffix: `Cargo.toml.fixture`
becomes the fixture's package manifest, which cannot be committed under its
own name here without cargo reading it as a package of this workspace.
