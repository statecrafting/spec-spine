//! The machine-readable verdict envelope (spec 037).
//!
//! The read verbs have spoken JSON since spec 010; the verbs that render a
//! *verdict* (`compile --check`, `index check`, `lint`, `couple`, `attest`,
//! `verify-attestation`) spoke only prose, so a programmatic consumer of the
//! gate chain had to string-match sentences like `index is fresh` and infer a
//! refusal's reasons from formatted text. That is the ad-hoc parsing
//! constitution II forbids, and until this module there was no supported
//! alternative for exactly the commands whose output is a decision.
//!
//! One envelope covers all six. `report` carries the verb's existing facade
//! payload, so a consumer parses one header shape across the chain and one
//! payload shape per verb, never a library spelling and a CLI spelling that
//! drift. The flag changes what is *written*, never what is *decided*: `ok` and
//! `exit_code` always agree with the code the process returns without it.

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::registry::Violation;
use crate::version::VERDICT_SCHEMA_VERSION;

/// The stable dotted command paths carried in [`Verdict::verb`].
///
/// Named constants rather than string literals at each call site: the verb is
/// part of the external contract, and a typo in one arm of a six-way surface is
/// otherwise invisible until a consumer's match falls through.
pub mod verb {
    /// `spec-spine compile --check`.
    pub const COMPILE_CHECK: &str = "compile.check";
    /// `spec-spine index check` (including `--slice`).
    pub const INDEX_CHECK: &str = "index.check";
    /// `spec-spine lint`.
    pub const LINT: &str = "lint";
    /// `spec-spine couple`.
    pub const COUPLE: &str = "couple";
    /// `spec-spine attest`.
    pub const ATTEST: &str = "attest";
    /// `spec-spine verify-attestation`.
    pub const VERIFY_ATTESTATION: &str = "verify-attestation";
}

/// One adjudicating verb's verdict, as written to stdout under `--json`.
///
/// `report` and `error` are mutually exclusive and exactly one is present; the
/// two constructors are the only way to build one, so that invariant cannot be
/// violated by construction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Verdict {
    /// [`VERDICT_SCHEMA_VERSION`] at the time of emission.
    pub schema_version: String,
    /// The stable dotted command path; see [`verb`].
    pub verb: String,
    /// The verdict. Always equals `exit_code == 0`, redundantly on purpose: a
    /// consumer holding the exit code and one holding only the document read
    /// the same fact.
    pub ok: bool,
    /// The code the process will actually return.
    pub exit_code: u8,
    /// The verb's facade payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<serde_json::Value>,
    /// The failure, when the verb could not render a report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<VerdictError>,
}

/// A failure, classified for branching without matching on the message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerdictError {
    /// A stable lowercase token from the closed set in [`error_kind`].
    pub kind: String,
    /// Human text. Carries no stability promise; branch on `kind`.
    pub message: String,
    /// The violations, present only when `kind` is `validation`.
    ///
    /// `Error::Validation` is the one variant carrying a structured payload,
    /// and `Display` reduces it to a count. Without this member a consumer
    /// handling `lint --json` exit 1 by reading its violation array would get
    /// nothing structured from `compile --check --json` on the same exit code,
    /// and would have to fall back to parsing stderr to learn which specs
    /// failed. Omitted, not null, when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<Violation>,
}

impl Verdict {
    /// A verdict carrying the verb's payload. `ok` is derived from `exit_code`,
    /// never passed separately, so the two cannot disagree.
    pub fn report(verb: &str, exit_code: u8, report: serde_json::Value) -> Self {
        Self {
            schema_version: VERDICT_SCHEMA_VERSION.to_string(),
            verb: verb.to_string(),
            ok: exit_code == 0,
            exit_code,
            report: Some(report),
            error: None,
        }
    }

    /// A verdict carrying a failure. The exit code is the error's own mapping,
    /// so the envelope can never advertise a code the process will not return.
    pub fn failure(verb: &str, error: &Error) -> Self {
        let exit_code = error.exit_code();
        Self {
            schema_version: VERDICT_SCHEMA_VERSION.to_string(),
            verb: verb.to_string(),
            ok: false,
            exit_code,
            report: None,
            error: Some(VerdictError {
                kind: error_kind(error).to_string(),
                message: error.to_string(),
                violations: match error {
                    Error::Validation(v) => v.clone(),
                    _ => Vec::new(),
                },
            }),
        }
    }

    /// Serialize to canonical JSON: sorted keys, 2-space pretty-print, LF, and
    /// a trailing newline, matching every other artifact this project emits, so
    /// `--json` output diffs cleanly and can be committed by a consumer that
    /// chooses to. Key sorting falls out of `serde_json::Map` being a
    /// `BTreeMap` (`preserve_order` is deliberately not enabled).
    pub fn to_canonical_json(&self) -> Result<String, Error> {
        let value = serde_json::to_value(self).map_err(|e| Error::Schema(e.to_string()))?;
        let mut out =
            serde_json::to_string_pretty(&value).map_err(|e| Error::Schema(e.to_string()))?;
        out.push('\n');
        Ok(out)
    }
}

/// The stable token for an [`Error`]'s class (spec 037 3.3).
///
/// Spelled out rather than derived from the variant name. Deriving it would make
/// an internal rename a silent breaking change to an external contract, with no
/// gate to catch it. The match is exhaustive inside this crate even though
/// `Error` is `#[non_exhaustive]`, so adding a variant fails the build here:
/// that is what makes [`VERDICT_SCHEMA_VERSION`]'s promise (a new `kind` is a
/// MINOR, a renamed or removed one a MAJOR) checkable rather than decorative.
pub fn error_kind(error: &Error) -> &'static str {
    match error {
        Error::Config(_) => "config",
        Error::Validation(_) => "validation",
        Error::NotFound(_) => "not-found",
        Error::Stale { .. } => "stale",
        Error::Io(_) => "io",
        Error::Parse(_) => "parse",
        Error::Schema(_) => "schema",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_and_error_are_mutually_exclusive() {
        let ok = Verdict::report(verb::LINT, 0, serde_json::json!([]));
        assert!(ok.report.is_some() && ok.error.is_none());
        let bad = Verdict::failure(verb::LINT, &Error::Parse("x".into()));
        assert!(bad.report.is_none() && bad.error.is_some());
    }

    #[test]
    fn ok_always_agrees_with_the_exit_code() {
        for code in 0u8..4 {
            let v = Verdict::report(verb::COUPLE, code, serde_json::json!({}));
            assert_eq!(v.ok, code == 0, "exit {code}");
        }
    }

    #[test]
    fn failure_carries_the_errors_own_exit_code() {
        let cases: [(Error, u8, &str); 7] = [
            (Error::Config("c".into()), 3, "config"),
            (Error::Validation(Vec::new()), 1, "validation"),
            (Error::NotFound("n".into()), 1, "not-found"),
            (
                Error::Stale {
                    expected: "a".into(),
                    actual: "b".into(),
                },
                2,
                "stale",
            ),
            (Error::Io("i".into()), 3, "io"),
            (Error::Parse("p".into()), 3, "parse"),
            (Error::Schema("s".into()), 3, "schema"),
        ];
        for (error, code, kind) in cases {
            let v = Verdict::failure(verb::INDEX_CHECK, &error);
            assert_eq!(v.exit_code, code, "{kind}");
            assert_eq!(v.error.as_ref().unwrap().kind, kind);
            assert!(!v.ok);
        }
    }

    #[test]
    fn canonical_json_is_sorted_and_newline_terminated() {
        let out = Verdict::report(verb::ATTEST, 0, serde_json::json!({"z": 1, "a": 2}))
            .to_canonical_json()
            .unwrap();
        assert!(out.ends_with("}\n"), "{out}");
        // Envelope keys sort: exitCode < ok < report < schemaVersion < verb.
        assert!(out.find("\"exitCode\"").unwrap() < out.find("\"ok\"").unwrap());
        assert!(out.find("\"report\"").unwrap() < out.find("\"schemaVersion\"").unwrap());
        // ...and so do the report's own keys.
        assert!(out.find("\"a\"").unwrap() < out.find("\"z\"").unwrap());
    }

    #[test]
    fn a_validation_failure_carries_its_violations() {
        use crate::registry::Severity;
        let v = Verdict::failure(
            verb::COMPILE_CHECK,
            &Error::Validation(vec![Violation {
                code: "V-001".to_string(),
                severity: Severity::Error,
                message: "boom".to_string(),
                path: Some("specs/001-a/spec.md".to_string()),
            }]),
        );
        let e = v.error.as_ref().unwrap();
        assert_eq!(e.kind, "validation");
        assert_eq!(e.violations.len(), 1, "the payload is not discarded");
        assert_eq!(e.violations[0].code, "V-001");
        // Every other kind leaves it empty, so it is omitted from the JSON.
        let other = Verdict::failure(verb::LINT, &Error::Io("x".into()));
        assert!(other.error.as_ref().unwrap().violations.is_empty());
        assert!(
            !other.to_canonical_json().unwrap().contains("violations"),
            "an empty violation list is omitted, not emitted as []"
        );
    }

    #[test]
    fn absent_members_are_omitted_not_null() {
        let out = Verdict::report(verb::LINT, 0, serde_json::json!([]))
            .to_canonical_json()
            .unwrap();
        assert!(!out.contains("\"error\""), "{out}");
    }
}
