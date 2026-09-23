// Spec: specs/108-a-work-scope-is-declared/spec.md
//! `spec-spine scope …` (spec 108 §3.6): evaluate a declared work scope
//! against the committed ownership index, or compare two declared scopes for
//! conflicting intentions.
//!
//! A scope lives in the consumer's record; nothing here writes, locks,
//! reserves or permits anything (§3.7). Evaluation refuses a stale committed
//! index (exit 2); comparison reads no ledger at all.

use std::io::Read as _;
use std::path::Path;

use clap::Subcommand;
use spec_spine_core::{
    Conflict, EvaluatedEntry, Finding, Role, ScopeComparison, ScopeEvaluation, ScopeRequest,
    Versioning, compare_scopes, evaluate, read_document,
};
use spec_spine_types::Error;

use crate::load_repo_config;

#[derive(Subcommand)]
pub enum ScopeAction {
    /// Resolve a declared scope's paths against the committed index and
    /// report where the declaration and the ownership disagree.
    Evaluate {
        /// A scope document, or `-` for stdin.
        #[arg(long, value_name = "FILE")]
        scope: String,
        #[arg(long)]
        json: bool,
    },
    /// Compare two declared scopes for conflicting intentions. Neither
    /// resolved against the committed ledger: a pure function of the two
    /// documents.
    Compare {
        /// A scope document, or `-` for stdin.
        a: String,
        /// A scope document, or `-` for stdin (only one of `a` / `b` may be).
        b: String,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(repo: &Path, action: &ScopeAction) -> Result<u8, Error> {
    match action {
        ScopeAction::Evaluate { scope, json } => run_evaluate(repo, scope, *json),
        ScopeAction::Compare { a, b, json } => run_compare(a, b, *json),
    }
}

/// Read a scope argument, allowing exactly one of a pair to be `-` (stdin).
fn read_arg(arg: &str, stdin_claimed: &mut bool) -> Result<String, Error> {
    if arg == "-" {
        if *stdin_claimed {
            return Err(Error::Parse(
                "only one of the two scope arguments may read stdin ('-')".into(),
            ));
        }
        *stdin_claimed = true;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| Error::Io(format!("read scope from stdin: {e}")))?;
        Ok(buf)
    } else {
        std::fs::read_to_string(arg).map_err(|e| Error::Io(format!("read scope {arg}: {e}")))
    }
}

fn parse_request(text: &str) -> Result<ScopeRequest, Error> {
    serde_json::from_str(text).map_err(|e| Error::Parse(format!("invalid scope request: {e}")))
}

fn run_evaluate(repo: &Path, scope: &str, json: bool) -> Result<u8, Error> {
    let mut stdin_claimed = false;
    let text = read_arg(scope, &mut stdin_claimed)?;
    let request = parse_request(&text)?;
    let cfg = load_repo_config(repo)?;
    let evaluated = evaluate(&cfg, repo, &request)?;
    if json {
        out!("{}", read_document(&evaluated, Versioning::Stamp)?);
    } else {
        print_evaluation(&evaluated);
    }
    // §3.3, §3.7: a report, not a gate. Exit 0 whether or not it found
    // anything.
    Ok(0)
}

fn run_compare(a: &str, b: &str, json: bool) -> Result<u8, Error> {
    let mut stdin_claimed = false;
    let a_text = read_arg(a, &mut stdin_claimed)?;
    let b_text = read_arg(b, &mut stdin_claimed)?;
    let a_req = parse_request(&a_text)?;
    let b_req = parse_request(&b_text)?;
    let comparison = compare_scopes(&a_req, &b_req)?;
    if json {
        out!("{}", read_document(&comparison, Versioning::Stamp)?);
    } else {
        print_comparison(&comparison);
    }
    // §3.5: exits 0 whether or not the two scopes conflict.
    Ok(0)
}

fn role_label(role: Role) -> &'static str {
    match role {
        Role::Mutable => "mutable",
        Role::Shared => "shared",
        Role::ReadOnly => "readOnly",
    }
}

fn print_evaluation(e: &ScopeEvaluation) {
    for entry in &e.entries {
        let EvaluatedEntry {
            path,
            role,
            owners,
            with,
        } = entry;
        outln!(
            "{}  {path}  owners=[{}]{}",
            role_label(*role),
            owners.join(", "),
            with.as_ref()
                .map(|w| format!("  with=[{}]", w.join(", ")))
                .unwrap_or_default()
        );
    }
    for f in &e.findings {
        let Finding {
            code,
            path,
            own_spec,
            owners,
            with,
        } = f;
        outln!(
            "{code}  {path}{}{}{}",
            own_spec
                .as_deref()
                .map(|s| format!("  ownSpec={s}"))
                .unwrap_or_default(),
            if owners.is_empty() {
                String::new()
            } else {
                format!("  owners=[{}]", owners.join(", "))
            },
            if with.is_empty() {
                String::new()
            } else {
                format!("  with=[{}]", with.join(", "))
            }
        );
    }
}

fn print_comparison(c: &ScopeComparison) {
    for Conflict { kind, a, b } in &c.conflicts {
        outln!(
            "{kind}  {} ({})  {} ({})",
            a.path,
            role_label(a.role),
            b.path,
            role_label(b.role)
        );
    }
    outln!("conflicts: {}", c.conflicts.len());
}
