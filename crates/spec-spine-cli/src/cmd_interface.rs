// Spec: specs/110-an-interface-reference-is-digest-pinned/spec.md
//! `spec-spine interface verify` (spec 110 §3.3): recompute every declared
//! cross-corpus interface reference against local directories the caller
//! names, and answer per reference `current`, `sections-current`, `stale`,
//! `missing` or `unverified`.
//!
//! The verb parses `--export <corpus>=<dir>` and hands the library a typed
//! map; the library refuses a stale ledger before reading any export, reads
//! only the referenced `spec.md` files under each directory, and never writes.
//! Nothing here opens a network connection or fills a pin from what it saw
//! (§3.6): the observed digest is printed for a human to copy.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::Subcommand;
use spec_spine_core::{Versioning, interface_verify, read_document};
use spec_spine_types::{Error, InterfaceReport, Outcome, SectionOutcome};

use crate::load_repo_config;

#[derive(Subcommand)]
pub enum InterfaceAction {
    /// Recompute every declared interface reference (spec 110) against local
    /// checkouts of the cited corpora. Exit 0 when every checked reference is
    /// `current` or `sections-current`, 1 when any is `stale`, `missing` or
    /// `unverified`, 2 when the committed registry is stale.
    ///
    /// Reads only the directories named by `--export`; fetches nothing and
    /// writes nothing. A corpus with no `--export` is `unverified`, which
    /// refuses: a citation nobody could check did not hold.
    Verify {
        /// `<corpus>=<dir>`: a local directory the caller asserts is that
        /// corpus's repository root. Repeat once per corpus.
        #[arg(long = "export", value_name = "CORPUS=DIR")]
        exports: Vec<String>,
        /// Check only the references this spec declares. Accepts the short id
        /// (`110`).
        #[arg(long, value_name = "ID")]
        spec: Option<String>,
        /// Emit the report as a read document on stdout (spec 074).
        #[arg(long)]
        json: bool,
    },
}

pub fn run(repo: &Path, action: &InterfaceAction) -> Result<u8, Error> {
    match action {
        InterfaceAction::Verify {
            exports,
            spec,
            json,
        } => verify(repo, exports, spec.as_deref(), *json),
    }
}

/// Parse `--export` arguments (§3.3): no `=`, an empty name or a name given
/// twice is a usage error (exit 3). The directory is taken as written, relative
/// to the working directory as any shell path is; the library reports an
/// unreadable one.
fn parse_exports(args: &[String]) -> Result<BTreeMap<String, PathBuf>, Error> {
    let mut out = BTreeMap::new();
    for arg in args {
        let Some((name, dir)) = arg.split_once('=') else {
            return Err(Error::Parse(format!(
                "--export '{arg}' has no '=': the form is <corpus>=<dir>"
            )));
        };
        if name.is_empty() {
            return Err(Error::Parse(format!(
                "--export '{arg}' names no corpus: the form is <corpus>=<dir>"
            )));
        }
        if dir.is_empty() {
            return Err(Error::Parse(format!(
                "--export '{arg}' names no directory: the form is <corpus>=<dir>"
            )));
        }
        if out.insert(name.to_string(), PathBuf::from(dir)).is_some() {
            return Err(Error::Parse(format!(
                "--export names corpus '{name}' twice: one directory per corpus"
            )));
        }
    }
    Ok(out)
}

fn verify(repo: &Path, exports: &[String], spec: Option<&str>, json: bool) -> Result<u8, Error> {
    let dirs = parse_exports(exports)?;
    let cfg = load_repo_config(repo)?;
    let report = interface_verify(&cfg, repo, &dirs, spec)?;
    if json {
        out!("{}", read_document(&report, Versioning::Stamp)?);
    } else {
        print_text(&report);
    }
    Ok(if held(&report) { 0 } else { 1 })
}

/// Exit 0 exactly when every checked reference is `current` or
/// `sections-current` (§3.3 table). An empty report holds.
fn held(report: &InterfaceReport) -> bool {
    report
        .references
        .iter()
        .all(|r| matches!(r.outcome, Outcome::Current | Outcome::SectionsCurrent))
}

fn label(o: Outcome) -> &'static str {
    match o {
        Outcome::Current => "current",
        Outcome::SectionsCurrent => "sections-current",
        Outcome::Stale => "stale",
        Outcome::Missing => "missing",
        Outcome::Unverified => "unverified",
    }
}

/// The text form names every reference that is not `current`, and what moved
/// (§3.3). Observed digests are printed for a human to copy (§3.6).
fn print_text(report: &InterfaceReport) {
    for r in &report.references {
        if r.outcome == Outcome::Current {
            continue;
        }
        outln!(
            "{}  {} -> {}:{}",
            label(r.outcome),
            r.declared_by,
            r.corpus,
            r.spec
        );
        match r.outcome {
            Outcome::Unverified => {
                outln!("  no --export was supplied for corpus '{}'", r.corpus)
            }
            Outcome::Missing => outln!("  the export for '{}' has no spec '{}'", r.corpus, r.spec),
            _ => {
                outln!("  pinned   {}", r.digest);
                if let Some(o) = &r.observed_digest {
                    outln!("  observed {o}");
                }
            }
        }
        for s in &r.sections {
            match s.outcome {
                SectionOutcome::Current => outln!("  section {}: current", s.anchor),
                SectionOutcome::Stale => outln!(
                    "  section {}: stale, pinned {} observed {}",
                    s.anchor,
                    s.digest,
                    s.observed_digest.as_deref().unwrap_or("-")
                ),
                SectionOutcome::Missing => {
                    outln!("  section {}: missing from the cited spec", s.anchor)
                }
            }
        }
    }
    let s = &report.summary;
    outln!(
        "references: {} (current {}, sections-current {}, stale {}, missing {}, unverified {})",
        report.references.len(),
        s.current,
        s.sections_current,
        s.stale,
        s.missing,
        s.unverified
    );
}
