// Spec: specs/000-spec-spine-bootstrap/spec.md
//! The single, stable error type for spec-spine.
//!
//! Every variant maps to a stable CLI exit code (see [`Error::exit_code`]); the
//! CLI is the only place that translates an `Error` into a process exit. Inside
//! the library we never `panic!` on user input, never `process::exit`, and never
//! `println!` data; those belong only in the CLI crate.
//!
//! Exit-code contract (spec 132, the family contract shared with Statecraft):
//!
//! | code | outcome   | meaning                                                    |
//! |------|-----------|------------------------------------------------------------|
//! | `0`  | `ok`      | the verb did what was asked and found nothing              |
//! | `1`  | `finding` | a validation failure, drift, staleness, an unresolved claim, a not-found, or authored content that does not parse |
//! | `2`  | `refused` | a precondition or policy was not met and nothing was done: invalid configuration, a version pin the binary does not satisfy, a containment refusal |
//! | `3`  | `usage`   | the invocation itself is wrong                             |
//! | `4`  | `failed`  | the tool could not do its work: I/O, an internal error, or an artifact the tool produced that fails its schema |
//!
//! Coupling drift exits `1` but is carried as a `CoupleReport` (not an
//! `Error`) so the JSON facade returns the structured report even on drift.

use crate::registry::Violation;

/// The stable error enum returned across the `spec-spine-core` public boundary.
///
/// `#[non_exhaustive]` so new variants are an additive (non-breaking) change.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A malformed or invalid `spec-spine.toml`, or a configuration passed as
    /// JSON that the loader would refuse. → exit 2, kind `config`.
    Config(String),
    /// A precondition or policy was not met and nothing was done (spec 132):
    /// a version pin the running binary does not satisfy, a write the
    /// containment rules of specs 126 to 128 decline. → exit 2, kind `refused`.
    Refused(String),
    /// Compile validation failed (the registry's `validation.passed` is false). → exit 1.
    Validation(Vec<Violation>),
    /// A requested spec id / view / path was not found. → exit 1.
    NotFound(String),
    /// The committed ledger is out of date relative to current inputs. → exit 1
    /// since spec 132 (it was 2), kind `stale`.
    Stale { expected: String, actual: String },
    /// A filesystem / git / process failure. → exit 4, kind `io`.
    Io(String),
    /// Authored content (a spec's frontmatter, a document the corpus holds)
    /// that does not parse. A finding about the corpus: → exit 1, kind
    /// `validation`.
    Parse(String),
    /// A document the tool produced (a committed shard, an attestation) that
    /// fails to parse or fails its schema or version check. → exit 4, kind
    /// `schema`.
    Schema(String),
    /// The invocation is wrong: an argument combination the verb rejects, or a
    /// request document the caller supplied that is not one (spec 132). → exit
    /// 3, kind `usage`.
    Usage(String),
    /// A defect in the tool itself, such as a value it built that will not
    /// serialize. → exit 4, kind `internal`.
    Internal(String),
}

/// The five outcomes of spec 132's contract, one per exit code.
pub mod outcome {
    /// Exit 0.
    pub const OK: &str = "ok";
    /// Exit 1.
    pub const FINDING: &str = "finding";
    /// Exit 2.
    pub const REFUSED: &str = "refused";
    /// Exit 3.
    pub const USAGE: &str = "usage";
    /// Exit 4.
    pub const FAILED: &str = "failed";

    /// The outcome an exit code names. A code outside the contract is a
    /// failure: nothing this tool returns means success except 0.
    pub fn of(exit_code: u8) -> &'static str {
        match exit_code {
            0 => OK,
            1 => FINDING,
            2 => REFUSED,
            3 => USAGE,
            _ => FAILED,
        }
    }
}

impl Error {
    /// The stable process exit code for this error (spec 132).
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::Validation(_) | Error::NotFound(_) | Error::Stale { .. } | Error::Parse(_) => 1,
            Error::Config(_) | Error::Refused(_) => 2,
            Error::Usage(_) => 3,
            Error::Io(_) | Error::Schema(_) | Error::Internal(_) => 4,
        }
    }

    /// The outcome token for [`Error::exit_code`].
    pub fn outcome(&self) -> &'static str {
        outcome::of(self.exit_code())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Config(m) => write!(f, "config error: {m}"),
            Error::Validation(v) => write!(f, "validation failed: {} violation(s)", v.len()),
            Error::NotFound(m) => write!(f, "not found: {m}"),
            Error::Stale { expected, actual } => {
                write!(
                    f,
                    "index is stale: expected content-hash {expected}, got {actual}"
                )
            }
            Error::Io(m) => write!(f, "io error: {m}"),
            Error::Parse(m) => write!(f, "parse error: {m}"),
            Error::Schema(m) => write!(f, "schema error: {m}"),
            Error::Refused(m) => write!(f, "refused: {m}"),
            Error::Usage(m) => write!(f, "usage error: {m}"),
            Error::Internal(m) => write!(f, "internal error: {m}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;
