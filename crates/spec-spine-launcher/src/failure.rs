//! One failure type carrying the exit code and the envelope `error.kind`.
//!
//! The launcher speaks the family exit contract (spec 132): `1` a finding, `2`
//! refused (nothing was done), `3` usage, `4` failed. It is carried with the
//! message so the text mode and the `--json` mode of `launcher resolve` cannot
//! disagree about either.

/// A launcher failure: the exit code, the closed `error.kind` token, and text.
#[derive(Debug)]
pub struct Failure {
    pub code: u8,
    pub kind: &'static str,
    pub message: String,
}

pub type Res<T> = Result<T, Failure>;

impl Failure {
    fn new(code: u8, kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            kind,
            message: message.into(),
        }
    }

    /// A precondition the operator supplies was not met; nothing was done.
    pub fn refused(message: impl Into<String>) -> Self {
        Self::new(2, "refused", message)
    }

    /// The repository's pin or lock (or the user's configuration) cannot be used.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(2, "config", message)
    }

    /// The pinned engine is not installed (`launcher resolve` only).
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(1, "not-found", message)
    }

    /// The invocation itself was malformed.
    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(3, "usage", message)
    }

    /// An operation the launcher attempted broke.
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(4, "io", message)
    }

    /// The launcher refuses to recurse, or to execute a launcher.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(4, "internal", message)
    }
}
