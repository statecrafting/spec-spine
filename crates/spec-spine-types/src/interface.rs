// Spec: specs/110-an-interface-reference-is-digest-pinned/spec.md
//! Interface reference and verifier-report DTOs (spec 110): a cross-corpus
//! citation pinned to another repository's spec content hash, and optionally
//! to the digests of the sections it relies on, plus the shape one verifier
//! answers in.
//!
//! The authored shape and the compiled shape are the same type: a reference is
//! carried into the registry record verbatim, the way [`crate::Obligation`]
//! and [`crate::Impact`] are. The report types
//! ([`InterfaceReport`], [`ReferenceResult`], [`Outcome`], [`SectionResult`],
//! [`SectionOutcome`], [`Summary`]) are never authored; they are what
//! `interface verify` (spec 110 §3.3) answers, produced by
//! `spec_spine_core::interface`.

use serde::{Deserialize, Serialize};

/// One declared cross-corpus interface reference (spec 110 §3.1).
///
/// `deny_unknown_fields`: a misspelt member is refused as malformed
/// frontmatter (`V-032`-and-up, spec 110 §3.2) rather than silently dropped,
/// the way [`crate::Obligation`] treats an unknown member.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceReference {
    /// The cited corpus's name, `^[a-z0-9][a-z0-9._-]*$` (§3.1). Not a URL and
    /// not a path: this spec defines no way to locate a corpus (§4).
    pub corpus: String,
    /// The cited spec's **full** id in that corpus. A short id cannot be
    /// resolved against another corpus, and is refused (§3.1).
    pub spec: String,
    /// `sha256:` followed by 64 lowercase hex digits: the cited spec's content
    /// hash as the exporting corpus's registry records it (`contentHash` /
    /// the shard's `shardHash`).
    pub digest: String,
    /// Pinned anchors of the cited spec, each with its section digest
    /// (spec 106 §3.5), in the same `sha256:` form. Anchors are unique within
    /// one reference (§3.2). Omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<PinnedSection>,
    /// An authored `YYYY-MM-DD` date recording when the digests were read.
    /// Never taken from a clock and never rewritten (§3.1).
    pub obtained: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

/// One pinned section within an [`InterfaceReference`] (spec 110 §3.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedSection {
    pub anchor: String,
    /// `sha256:` followed by 64 lowercase hex digits.
    pub digest: String,
}

/// The verifier's answer (spec 110 §3.3): one [`ReferenceResult`] per
/// reference checked, and a per-outcome [`Summary`]. A read document
/// (spec 074) when emitted `--json`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceReport {
    pub references: Vec<ReferenceResult>,
    pub summary: Summary,
}

/// One reference's recomputed outcome (spec 110 §3.3), sorted by
/// `(declaredBy, corpus, spec)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceResult {
    /// The spec that declared this reference.
    pub declared_by: String,
    pub corpus: String,
    pub spec: String,
    /// The pinned digest, as declared.
    pub digest: String,
    /// The recomputed content hash, `sha256:`-prefixed, when the cited spec
    /// was found in the supplied export. Absent for `unverified` (no export
    /// supplied) and `missing` (the spec was not in the export).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_digest: Option<String>,
    pub outcome: Outcome,
    /// Per pinned anchor, its own outcome. Omitted when the reference pins no
    /// sections.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<SectionResult>,
}

/// A reference's outcome (spec 110 §3.3 table). Kebab-case on the wire:
/// `current`, `sections-current`, `stale`, `missing`, `unverified`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// The recomputed content hash equals `digest`.
    Current,
    /// The content hash differs, the reference pins sections, and every
    /// pinned section's recomputed digest equals its pin.
    SectionsCurrent,
    /// The content hash differs and no sections are pinned, or a pinned
    /// section's digest differs, or a pinned anchor no longer exists.
    Stale,
    /// An export was supplied for the corpus and the spec is not in it.
    Missing,
    /// No export was supplied for the corpus.
    Unverified,
}

/// One pinned section's recomputed outcome (spec 110 §3.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionResult {
    pub anchor: String,
    /// The pinned digest, as declared.
    pub digest: String,
    /// The recomputed section digest, when the cited spec was found and
    /// still carries the anchor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_digest: Option<String>,
    pub outcome: SectionOutcome,
}

/// A pinned section's outcome. Kebab-case on the wire: `current`, `stale`,
/// `missing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SectionOutcome {
    Current,
    Stale,
    Missing,
}

/// A count per [`Outcome`] (spec 110 §3.3), over every checked reference.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub current: usize,
    pub sections_current: usize,
    pub stale: usize,
    pub missing: usize,
    pub unverified: usize,
}

impl Summary {
    /// Tally one reference's outcome.
    pub fn record(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Current => self.current += 1,
            Outcome::SectionsCurrent => self.sections_current += 1,
            Outcome::Stale => self.stale += 1,
            Outcome::Missing => self.missing += 1,
            Outcome::Unverified => self.unverified += 1,
        }
    }
}

/// The corpus-name grammar (§3.1): `^[a-z0-9][a-z0-9._-]*$`.
pub fn valid_corpus_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

/// The digest grammar (§3.1, §3.3): `^sha256:[0-9a-f]{64}$`. The algorithm is
/// written out so a later construction cannot be mistaken for this one.
pub fn valid_digest(digest: &str) -> bool {
    match digest.strip_prefix("sha256:") {
        Some(hex) => {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        }
        None => false,
    }
}

/// The `obtained` grammar (§3.1): `^\d{4}-\d{2}-\d{2}$`. No calendar check
/// beyond digit-shape, matching `created`'s own grammar elsewhere in this
/// crate.
pub fn valid_obtained_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    bytes.len() == 10
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_name_grammar() {
        for ok in ["a", "statecraft-cli", "a1", "a.b_c-9"] {
            assert!(valid_corpus_name(ok), "{ok}");
        }
        for bad in ["", "-a", "_a", ".a", "A", "aB", "a b", "a/b"] {
            assert!(!valid_corpus_name(bad), "{bad}");
        }
    }

    #[test]
    fn digest_grammar() {
        let hex64 = "3f5c".to_string() + &"0".repeat(60);
        assert!(valid_digest(&format!("sha256:{hex64}")));
        for bad in [
            "",
            "sha256:",
            "sha1:aaaa",
            &format!("SHA256:{hex64}"),
            &format!("sha256:{}", "g".repeat(64)),
            &format!("sha256:{}", "A".repeat(64)),
            &format!("sha256:{}", "0".repeat(63)),
            &format!("sha256:{}", "0".repeat(65)),
        ] {
            assert!(!valid_digest(bad), "{bad}");
        }
    }

    #[test]
    fn obtained_date_grammar() {
        assert!(valid_obtained_date("2026-09-22"));
        for bad in ["2026-9-22", "2026/09/22", "", "2026-09-2", "20260922"] {
            assert!(!valid_obtained_date(bad), "{bad}");
        }
    }

    #[test]
    fn summary_tallies_every_outcome() {
        let mut s = Summary::default();
        for o in [
            Outcome::Current,
            Outcome::SectionsCurrent,
            Outcome::Stale,
            Outcome::Missing,
            Outcome::Unverified,
            Outcome::Current,
        ] {
            s.record(o);
        }
        assert_eq!(
            s,
            Summary {
                current: 2,
                sections_current: 1,
                stale: 1,
                missing: 1,
                unverified: 1,
            }
        );
    }

    #[test]
    fn outcome_serializes_kebab_case() {
        assert_eq!(
            serde_json::to_value(Outcome::SectionsCurrent).unwrap(),
            "sections-current"
        );
        assert_eq!(serde_json::to_value(Outcome::Current).unwrap(), "current");
    }
}
