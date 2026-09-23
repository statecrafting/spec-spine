// Spec: specs/113-a-waiver-has-a-declared-lifecycle/spec.md
//! A waiver's declared lifecycle (spec 113): scope, expiry, ancestry and a use
//! limit, each evaluated over inputs the caller supplies.
//!
//! Pure. Nothing here reads a clock, runs `git`, or writes anything: an expiry
//! is compared against an as-of date somebody passed in, an ancestry against an
//! answer somebody computed, a use limit against a count somebody keeps. A
//! declared check whose input is absent is `not-evaluated`, never satisfied
//! (spec 113 §3.4), and nothing is ever consumed (§3.7).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use spec_spine_types::{Config, Violation};

use crate::couple::claim_matches;
use crate::hash;

/// One waiver as its author declared it (spec 113 §3.1).
///
/// The lifecycle values are kept as written, not parsed into dates and numbers,
/// because a malformed value is a failed check reported with what was wrong
/// (§3.5), not a parse error of the whole run.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WaiverDeclaration {
    pub reason: String,
    /// `None` is unscoped: the waiver clears every violation (§3.2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<String>,
    /// Single-valued keys this declaration repeated. Each fails its check
    /// (§3.1): which of two expiries the author meant is not the gate's guess.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeated: Vec<String>,
}

impl WaiverDeclaration {
    /// A waiver with a reason and nothing else: today's waiver, and the
    /// dependency-only auto-waiver (spec 113 §3.8).
    pub fn unscoped(reason: impl Into<String>) -> Self {
        WaiverDeclaration {
            reason: reason.into(),
            ..WaiverDeclaration::default()
        }
    }

    /// `sha256:` over the declaration (spec 113 §3.6): the key a caller counts
    /// uses against. The corpus's one hash construction, over named pieces, so
    /// the order the lines were written in does not move it and every declared
    /// value does.
    pub fn id(&self) -> String {
        let mut paths = self.paths.clone().unwrap_or_default();
        paths.sort();
        let scoped = if self.paths.is_some() {
            "scoped"
        } else {
            "unscoped"
        };
        let pieces = vec![
            ("reason".to_string(), self.reason.clone()),
            (
                "scope".to_string(),
                format!("{scoped}\n{}", paths.join("\n")),
            ),
            ("until".to_string(), self.until.clone().unwrap_or_default()),
            ("since".to_string(), self.since.clone().unwrap_or_default()),
            (
                "maxUses".to_string(),
                self.max_uses.clone().unwrap_or_default(),
            ),
            ("repeated".to_string(), self.repeated.join("\n")),
        ];
        format!("sha256:{}", hash::content_hash(pieces))
    }
}

/// Every waiver a pull request body declares, and the lifecycle lines that
/// belonged to none (spec 113 §3.1).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WaiverSet {
    pub declarations: Vec<WaiverDeclaration>,
    /// Lifecycle lines with no waiver line above them, trimmed, in body order.
    pub unattached: Vec<String>,
}

impl WaiverSet {
    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty() && self.unattached.is_empty()
    }
}

/// The inputs a caller supplies (spec 113 §3.3). Each is optional, and an
/// absent one leaves its check `not-evaluated`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WaiverInputs {
    /// `YYYY-MM-DD`, compared against each `-Until:`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    /// Keyed by a declared `-Since:` exactly as written: whether that commit is
    /// an ancestor of the head under judgment.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ancestry: BTreeMap<String, bool>,
    /// Keyed by waiver [`id`](WaiverDeclaration::id): how many runs it has
    /// already cleared, excluding this one.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub uses: BTreeMap<String, u64>,
}

impl WaiverInputs {
    /// Refuse an as-of that is not a date: it is the caller's input, so a
    /// malformed one is a usage error of the run (exit 3), unlike a malformed
    /// declaration, which fails only its own waiver.
    pub fn validate(&self) -> Result<(), spec_spine_types::Error> {
        if let Some(d) = &self.as_of
            && !is_date(d)
        {
            return Err(spec_spine_types::Error::Parse(format!(
                "waiver as-of '{d}' is not a YYYY-MM-DD date"
            )));
        }
        Ok(())
    }
}

/// `satisfied`, `failed` or `not-evaluated` (spec 113 §3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckOutcome {
    Satisfied,
    Failed,
    NotEvaluated,
}

/// One declared check and how it came out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaiverCheck {
    /// `expiry`, `ancestry` or `uses`.
    pub check: String,
    pub declared: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    pub outcome: CheckOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// One violation a waiver cleared (spec 113 §3.6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearedViolation {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// One declared waiver, evaluated (spec 113 §3.6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaiverOutcome {
    pub id: String,
    pub reason: String,
    pub scoped: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<WaiverCheck>,
    /// No check failed. Says nothing about whether the waiver was authorized
    /// (§3.7).
    pub effective: bool,
    pub clears: Vec<ClearedViolation>,
}

/// The lifecycle keys, as suffixes of the keyword without its colon.
const PATHS: &str = "-Paths:";
const UNTIL: &str = "-Until:";
const SINCE: &str = "-Since:";
const MAX_USES: &str = "-Max-Uses:";

/// Parse every waiver in a pull request body (spec 113 §3.1).
///
/// A waiver line is exactly what [`crate::parse_waiver`] recognises, so the
/// first declaration's reason is the one that function returns. Lifecycle
/// lines attach to the nearest waiver line above them.
pub fn parse_waivers(cfg: &Config, body: &str) -> WaiverSet {
    let keyword = cfg.coupling.waiver_keyword.as_str();
    let base = keyword.strip_suffix(':').unwrap_or(keyword);
    let mut set = WaiverSet::default();
    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(keyword) {
            let reason = rest.trim();
            if !reason.is_empty() {
                set.declarations.push(WaiverDeclaration::unscoped(reason));
            }
            continue;
        }
        let Some(tail) = trimmed.strip_prefix(base) else {
            continue;
        };
        let Some((key, value)) = [PATHS, UNTIL, SINCE, MAX_USES]
            .iter()
            .find_map(|k| tail.strip_prefix(k).map(|v| (*k, v.trim())))
        else {
            continue;
        };
        let Some(current) = set.declarations.last_mut() else {
            set.unattached.push(trimmed.trim_end().to_string());
            continue;
        };
        let single = |slot: &mut Option<String>, repeated: &mut Vec<String>| {
            if slot.is_some() {
                let name = key
                    .trim_start_matches('-')
                    .trim_end_matches(':')
                    .to_string();
                if !repeated.contains(&name) {
                    repeated.push(name);
                }
            } else {
                *slot = Some(value.to_string());
            }
        };
        match key {
            PATHS => {
                let list = current.paths.get_or_insert_with(Vec::new);
                list.extend(
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|p| !p.is_empty())
                        .map(str::to_string),
                );
            }
            UNTIL => single(&mut current.until, &mut current.repeated),
            SINCE => single(&mut current.since, &mut current.repeated),
            _ => single(&mut current.max_uses, &mut current.repeated),
        }
    }
    for d in &mut set.declarations {
        if let Some(p) = &mut d.paths {
            p.sort();
            p.dedup();
        }
    }
    set
}

/// Evaluate every declaration and pair each violation with the waiver that
/// clears it (spec 113 §3.3 to §3.6). `violations` is the run's full list,
/// already sorted by path; the result is in declaration order.
pub fn evaluate(
    declarations: &[WaiverDeclaration],
    inputs: &WaiverInputs,
    violations: &[Violation],
) -> Vec<WaiverOutcome> {
    let mut outcomes: Vec<WaiverOutcome> = declarations
        .iter()
        .map(|d| {
            let id = d.id();
            let checks = checks(d, &id, inputs);
            let effective = checks.iter().all(|c| c.outcome != CheckOutcome::Failed);
            WaiverOutcome {
                reason: d.reason.clone(),
                scoped: d.paths.is_some(),
                paths: d.paths.clone().unwrap_or_default(),
                checks,
                effective,
                clears: Vec::new(),
                id,
            }
        })
        .collect();
    for v in violations {
        let taker = outcomes.iter_mut().find(|o| {
            o.effective
                && (!o.scoped
                    || v.path
                        .as_deref()
                        .is_some_and(|p| o.paths.iter().any(|e| claim_matches(e, p))))
        });
        if let Some(o) = taker {
            o.clears.push(ClearedViolation {
                code: v.code.clone(),
                path: v.path.clone(),
            });
        }
    }
    outcomes
}

/// The violations no waiver cleared, in the report's order.
pub fn uncleared<'a>(
    violations: &'a [Violation],
    outcomes: &[WaiverOutcome],
) -> Vec<&'a Violation> {
    let cleared: BTreeSet<(&str, Option<&str>)> = outcomes
        .iter()
        .flat_map(|o| o.clears.iter())
        .map(|c| (c.code.as_str(), c.path.as_deref()))
        .collect();
    violations
        .iter()
        .filter(|v| !cleared.contains(&(v.code.as_str(), v.path.as_deref())))
        .collect()
}

fn checks(d: &WaiverDeclaration, id: &str, inputs: &WaiverInputs) -> Vec<WaiverCheck> {
    let mut out = Vec::new();
    let repeated = |name: &str| d.repeated.iter().any(|r| r == name);
    if let Some(until) = &d.until {
        out.push(if repeated("Until") {
            failed("expiry", until, None, "declared more than once")
        } else if !is_date(until) {
            failed("expiry", until, None, "not a YYYY-MM-DD date")
        } else {
            match &inputs.as_of {
                None => not_evaluated("expiry", until),
                // ISO dates order as strings.
                Some(as_of) if as_of.as_str() <= until.as_str() => {
                    satisfied("expiry", until, as_of)
                }
                Some(as_of) => failed(
                    "expiry",
                    until,
                    Some(as_of),
                    &format!("expired: as of {as_of}, after {until}"),
                ),
            }
        });
    }
    if let Some(since) = &d.since {
        out.push(if repeated("Since") {
            failed("ancestry", since, None, "declared more than once")
        } else if !is_commit(since) {
            failed(
                "ancestry",
                since,
                None,
                "not a 4 to 40 character hexadecimal commit",
            )
        } else {
            match inputs.ancestry.get(since) {
                None => not_evaluated("ancestry", since),
                Some(true) => satisfied("ancestry", since, "true"),
                Some(false) => failed(
                    "ancestry",
                    since,
                    Some("false"),
                    &format!("{since} is not an ancestor of the head under judgment"),
                ),
            }
        });
    }
    if let Some(max) = &d.max_uses {
        out.push(if repeated("Max-Uses") {
            failed("uses", max, None, "declared more than once")
        } else {
            match max.parse::<u64>() {
                Ok(m) if m > 0 => match inputs.uses.get(id) {
                    None => not_evaluated("uses", max),
                    Some(n) if *n < m => satisfied("uses", max, &n.to_string()),
                    Some(n) => failed(
                        "uses",
                        max,
                        Some(&n.to_string()),
                        &format!("already used {n} time(s), limit {m}"),
                    ),
                },
                _ => failed("uses", max, None, "not a positive integer"),
            }
        });
    }
    out
}

fn satisfied(check: &str, declared: &str, input: &str) -> WaiverCheck {
    WaiverCheck {
        check: check.to_string(),
        declared: declared.to_string(),
        input: Some(input.to_string()),
        outcome: CheckOutcome::Satisfied,
        detail: None,
    }
}

fn not_evaluated(check: &str, declared: &str) -> WaiverCheck {
    WaiverCheck {
        check: check.to_string(),
        declared: declared.to_string(),
        input: None,
        outcome: CheckOutcome::NotEvaluated,
        detail: Some("no input supplied".to_string()),
    }
}

fn failed(check: &str, declared: &str, input: Option<&str>, detail: &str) -> WaiverCheck {
    WaiverCheck {
        check: check.to_string(),
        declared: declared.to_string(),
        input: input.map(str::to_string),
        outcome: CheckOutcome::Failed,
        detail: Some(detail.to_string()),
    }
}

/// `YYYY-MM-DD` with a month 01 to 12 and a day 01 to 31. Not a calendar: a
/// date like 2026-02-31 is accepted and compares as written, which is all an
/// inclusive "until" needs.
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    if !(digits(0..4) && digits(5..7) && digits(8..10)) {
        return false;
    }
    let month = (b[5] - b'0') * 10 + (b[6] - b'0');
    let day = (b[8] - b'0') * 10 + (b[9] - b'0');
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn is_commit(s: &str) -> bool {
    (4..=40).contains(&s.len()) && s.bytes().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_and_commits_are_checked_by_shape() {
        assert!(is_date("2026-12-31"));
        assert!(!is_date("2026-13-01"));
        assert!(!is_date("2026-1-01"));
        assert!(!is_date("tomorrow"));
        assert!(is_commit("a3d5213d"));
        assert!(!is_commit("abc"));
        assert!(!is_commit("a3d5213z"));
    }
}
