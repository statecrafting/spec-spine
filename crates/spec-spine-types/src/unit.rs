//! The authority-unit grammar.
//!
//! A spec declares the units it owns via a `unit:` on a typed edge. The grammar
//! resolves six granularities: [`Unit::File`], [`Unit::Section`],
//! [`Unit::Symbol`], [`Unit::Directory`], [`Unit::Crate`], and [`Unit::Module`]
//! (ported from OAP `spec-types::LogicalUnit`). All six are implemented:
//! `file`/`section`/`symbol` shipped first, and `directory`/`crate`/`module`
//! landed in spec 017 (originally reserved in `docs/design/00-architecture.md`
//! §2.2 Q5). They were a MINOR bump because the schema is permissive on the unit
//! payload (no schema-file edit), and a bare string remains shorthand for a file
//! unit (a trailing-slash path denotes a directory subtree).

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

/// An authority unit: the granularity at which a spec claims ownership.
///
/// Serializes internally-tagged on `kind` (e.g. `{ "kind": "file", "path": ... }`).
/// Deserializes from that tagged map, a bare string (= a file unit), **or** a
/// `{ unit: <unit> }` wrapper (spec 015 sugar), so authors can write
/// `establishes: ["src/lib.rs"]` or `establishes: [{ unit: "src/lib.rs" }]`
/// interchangeably; all three normalize to the same unit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Unit {
    /// A file path (bare string shorthand resolves here). A trailing `/` denotes
    /// the directory subtree rooted at `path`.
    File {
        path: String,
        /// Spec 076: the spec claims this territory and has not written it
        /// yet. A declared state, not a diagnostic: an unresolved unit marked
        /// `planned` produces no `W-001`, while one that is merely wrong still
        /// does, which is the distinction that makes the flag safe to add to a
        /// corpus gating on `--fail-on-unresolved`.
        ///
        /// Serialized only when true, so a corpus that uses no planned units
        /// emits byte-identical shards across this change. A written
        /// `planned: false` therefore normalizes to absent: it means exactly
        /// what omitting it means, and one canonical form per meaning is what
        /// keeps the four-triple determinism gate from seeing a round-trip as
        /// drift.
        #[serde(default, skip_serializing_if = "is_false")]
        planned: bool,
    },
    /// A named section within a file: a Makefile target, a Markdown heading slug,
    /// a `region:` marker, or a CI `jobs.<name>`.
    Section {
        file: String,
        anchor: String,
        /// Spec 076: the spec claims this territory and has not written it
        /// yet. A declared state, not a diagnostic: an unresolved unit marked
        /// `planned` produces no `W-001`, while one that is merely wrong still
        /// does, which is the distinction that makes the flag safe to add to a
        /// corpus gating on `--fail-on-unresolved`.
        ///
        /// Serialized only when true, so a corpus that uses no planned units
        /// emits byte-identical shards across this change. A written
        /// `planned: false` therefore normalizes to absent: it means exactly
        /// what omitting it means, and one canonical form per meaning is what
        /// keeps the four-triple determinism gate from seeing a round-trip as
        /// drift.
        #[serde(default, skip_serializing_if = "is_false")]
        planned: bool,
    },
    /// A symbol (function / type / export), resolved by the indexer via
    /// tree-sitter (Rust `.rs` and TypeScript `.ts`/`.tsx`).
    Symbol {
        id: String,
        /// Spec 076: the spec claims this territory and has not written it
        /// yet. A declared state, not a diagnostic: an unresolved unit marked
        /// `planned` produces no `W-001`, while one that is merely wrong still
        /// does, which is the distinction that makes the flag safe to add to a
        /// corpus gating on `--fail-on-unresolved`.
        ///
        /// Serialized only when true, so a corpus that uses no planned units
        /// emits byte-identical shards across this change. A written
        /// `planned: false` therefore normalizes to absent: it means exactly
        /// what omitting it means, and one canonical form per meaning is what
        /// keeps the four-triple determinism gate from seeing a round-trip as
        /// drift.
        #[serde(default, skip_serializing_if = "is_false")]
        planned: bool,
    },
    /// A directory subtree, named explicitly (`{ kind: directory, path }`). The
    /// subtree-prefix resolution is identical to a trailing-slash file unit; the
    /// distinct kind preserves the author's intent across the round-trip (spec
    /// 017). Resolves to the directory path; the gate prefix-matches it.
    Directory {
        path: String,
        /// Spec 076: the spec claims this territory and has not written it
        /// yet. A declared state, not a diagnostic: an unresolved unit marked
        /// `planned` produces no `W-001`, while one that is merely wrong still
        /// does, which is the distinction that makes the flag safe to add to a
        /// corpus gating on `--fail-on-unresolved`.
        ///
        /// Serialized only when true, so a corpus that uses no planned units
        /// emits byte-identical shards across this change. A written
        /// `planned: false` therefore normalizes to absent: it means exactly
        /// what omitting it means, and one canonical form per meaning is what
        /// keeps the four-triple determinism gate from seeing a round-trip as
        /// drift.
        #[serde(default, skip_serializing_if = "is_false")]
        planned: bool,
    },
    /// A compilation unit by its manifest name (Cargo `[package].name` or npm
    /// `package.json:name`), resolved against the discovered package inventory to
    /// the package directory subtree (spec 017).
    Crate {
        id: String,
        /// Spec 076: the spec claims this territory and has not written it
        /// yet. A declared state, not a diagnostic: an unresolved unit marked
        /// `planned` produces no `W-001`, while one that is merely wrong still
        /// does, which is the distinction that makes the flag safe to add to a
        /// corpus gating on `--fail-on-unresolved`.
        ///
        /// Serialized only when true, so a corpus that uses no planned units
        /// emits byte-identical shards across this change. A written
        /// `planned: false` therefore normalizes to absent: it means exactly
        /// what omitting it means, and one canonical form per meaning is what
        /// keeps the four-triple determinism gate from seeing a round-trip as
        /// drift.
        #[serde(default, skip_serializing_if = "is_false")]
        planned: bool,
    },
    /// A module by its `::`-qualified path (e.g. `my_crate::serialization`),
    /// resolved by the indexer's Rust module index: file-modules (whole file)
    /// and top-level inline `mod` blocks (line-span) (spec 017).
    Module {
        id: String,
        /// Spec 076: the spec claims this territory and has not written it
        /// yet. A declared state, not a diagnostic: an unresolved unit marked
        /// `planned` produces no `W-001`, while one that is merely wrong still
        /// does, which is the distinction that makes the flag safe to add to a
        /// corpus gating on `--fail-on-unresolved`.
        ///
        /// Serialized only when true, so a corpus that uses no planned units
        /// emits byte-identical shards across this change. A written
        /// `planned: false` therefore normalizes to absent: it means exactly
        /// what omitting it means, and one canonical form per meaning is what
        /// keeps the four-triple determinism gate from seeing a round-trip as
        /// drift.
        #[serde(default, skip_serializing_if = "is_false")]
        planned: bool,
    },
}

impl Unit {
    /// A file unit from a path (the bare-string shorthand target).
    pub fn file(path: impl Into<String>) -> Self {
        Unit::File {
            path: path.into(),
            planned: false,
        }
    }

    /// Spec 076 §3.1: whether the claiming spec has declared this territory
    /// planned but not yet written.
    pub fn is_planned(&self) -> bool {
        match self {
            Unit::File { planned, .. }
            | Unit::Section { planned, .. }
            | Unit::Symbol { planned, .. }
            | Unit::Directory { planned, .. }
            | Unit::Crate { planned, .. }
            | Unit::Module { planned, .. } => *planned,
        }
    }

    /// The same unit with the flag cleared: its **subject**, which is what
    /// identity is about.
    ///
    /// `planned` annotates a claim's resolution state, not which territory the
    /// claim names, so two units differing only in the flag name one unit. The
    /// index stores subjects, so a planned claim that has resolved answers an
    /// `authorities` lookup exactly as an unplanned one does (spec 076 §3.4),
    /// and the collision checks in §3.5 compare subjects rather than payloads.
    pub fn subject(&self) -> Self {
        let mut out = self.clone();
        match &mut out {
            Unit::File { planned, .. }
            | Unit::Section { planned, .. }
            | Unit::Symbol { planned, .. }
            | Unit::Directory { planned, .. }
            | Unit::Crate { planned, .. }
            | Unit::Module { planned, .. } => *planned = false,
        }
        out
    }

    /// True if this unit resolves to a directory subtree: a file unit whose path
    /// ends `/`, or an explicit [`Unit::Directory`] (spec 017).
    pub fn is_directory_subtree(&self) -> bool {
        match self {
            Unit::File { path, .. } => path.ends_with('/'),
            Unit::Directory { .. } => true,
            _ => false,
        }
    }
}

impl<'de> Deserialize<'de> for Unit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Accept a bare string (-> file unit), the tagged map form, or the
        // `{ unit: <unit> }` wrapper (spec 015).
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Bare(String),
            Tagged(Tagged),
            // A predecessor dialect authors every `establishes` item as a
            // single-key `unit:` map. The wrapper carries no information beyond
            // the unit it wraps, so it normalizes away to that inner unit -- a
            // third 1:1 representation alongside the bare-string and tagged
            // forms, resolved by recursing through this same impl (so the inner
            // unit may itself be bare or tagged, and inherits its validation).
            Wrapped { unit: Box<Unit> },
        }
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
        enum Tagged {
            File {
                path: String,
                #[serde(default)]
                planned: bool,
            },
            Section {
                file: String,
                anchor: String,
                #[serde(default)]
                planned: bool,
            },
            Symbol {
                id: String,
                #[serde(default)]
                planned: bool,
            },
            Directory {
                path: String,
                #[serde(default)]
                planned: bool,
            },
            Crate {
                id: String,
                #[serde(default)]
                planned: bool,
            },
            Module {
                id: String,
                #[serde(default)]
                planned: bool,
            },
        }

        match Repr::deserialize(deserializer)? {
            Repr::Bare(path) => {
                if path.trim().is_empty() {
                    return Err(de::Error::custom("unit path must not be empty"));
                }
                // Spec 076 §3.1: the bare-string shorthand cannot carry the
                // flag, and the grammar is deliberately not extended to let it.
                // A second string grammar would be parsed in a second place,
                // where a misspelling degrades silently into a path.
                Ok(Unit::File {
                    path,
                    planned: false,
                })
            }
            Repr::Tagged(Tagged::File { path, planned }) => Ok(Unit::File { path, planned }),
            Repr::Tagged(Tagged::Section {
                file,
                anchor,
                planned,
            }) => Ok(Unit::Section {
                file,
                anchor,
                planned,
            }),
            Repr::Tagged(Tagged::Symbol { id, planned }) => Ok(Unit::Symbol { id, planned }),
            Repr::Tagged(Tagged::Directory { path, planned }) => {
                Ok(Unit::Directory { path, planned })
            }
            Repr::Tagged(Tagged::Crate { id, planned }) => Ok(Unit::Crate { id, planned }),
            Repr::Tagged(Tagged::Module { id, planned }) => Ok(Unit::Module { id, planned }),
            Repr::Wrapped { unit } => Ok(*unit),
        }
    }
}

/// `skip_serializing_if` for a `bool` that is only meaningful when true.
fn is_false(b: &bool) -> bool {
    !*b
}
