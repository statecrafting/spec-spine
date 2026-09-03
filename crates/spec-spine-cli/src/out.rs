//! Stdout writes that survive a closed reader (spec 035).
//!
//! `println!` unwraps its write, so a reader that stops early made the process
//! panic: `spec-spine registry list --json | head` exited **101**, outside the
//! documented `0`/`1`/`2`/`3` contract, with a backtrace on stderr. Piping into
//! `head`, `less` or `grep -q` is ordinary use, not an error.
//!
//! Every stdout write in the CLI goes through [`line`]. Stderr keeps using
//! `eprintln!`: diagnostics are small, and a broken stderr has nowhere left to
//! report itself anyway.

use std::fmt::Arguments;
use std::io::{self, Write};

/// What a stdout write means for the process.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Written.
    Wrote,
    /// The reader went away: `head` had enough, a pager was quit. The consumer
    /// got what it asked for, so this is a normal end, not a failure.
    ReaderGone,
    /// A genuine I/O failure (a full disk, a closed descriptor).
    Failed,
}

/// Classify a write result. Split out from [`line`] so the policy is testable
/// without arranging a real broken pipe.
pub(crate) fn classify(res: &io::Result<()>) -> Outcome {
    match res {
        Ok(()) => Outcome::Wrote,
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Outcome::ReaderGone,
        Err(_) => Outcome::Failed,
    }
}

/// Write one line to stdout, ending the process cleanly if the reader is gone.
///
/// Exits `0` on a closed pipe: the reader chose to stop, and the shell
/// convention is that `producer | head` succeeds. Exits `3` on a real I/O
/// failure, matching the `Error::exit_code()` mapping for I/O in `main.rs`.
pub(crate) fn line(args: Arguments<'_>) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let res = writeln!(handle, "{args}");
    match classify(&res) {
        Outcome::Wrote => {}
        Outcome::ReaderGone => std::process::exit(0),
        Outcome::Failed => {
            if let Err(e) = res {
                eprintln!("spec-spine: cannot write to stdout: {e}");
            }
            std::process::exit(3);
        }
    }
}

/// Write a pre-formatted block to stdout verbatim, adding no newline.
///
/// The rendered projections (`index render`, `index coverage`) build their
/// whole output as a single string that already ends in a newline, so they need
/// a write that does not append a second one. Same classification as [`line`].
pub(crate) fn block(args: Arguments<'_>) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let res = write!(handle, "{args}").and_then(|()| handle.flush());
    match classify(&res) {
        Outcome::Wrote => {}
        Outcome::ReaderGone => std::process::exit(0),
        Outcome::Failed => {
            if let Err(e) = res {
                eprintln!("spec-spine: cannot write to stdout: {e}");
            }
            std::process::exit(3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Outcome, classify};
    use std::io::{self, ErrorKind};

    #[test]
    fn ok_is_wrote() {
        assert_eq!(classify(&Ok(())), Outcome::Wrote);
    }

    #[test]
    fn broken_pipe_is_reader_gone() {
        let e = io::Error::new(ErrorKind::BrokenPipe, "closed");
        assert_eq!(classify(&Err(e)), Outcome::ReaderGone);
    }

    #[test]
    fn other_io_errors_are_failures() {
        for kind in [ErrorKind::PermissionDenied, ErrorKind::WriteZero] {
            let e = io::Error::new(kind, "x");
            assert_eq!(classify(&Err(e)), Outcome::Failed, "{kind:?}");
        }
    }
}
