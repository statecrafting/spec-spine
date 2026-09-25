// Spec: specs/144-a-repository-path-is-one-type/spec.md
//! One rule for a repository-relative path (spec 144 §3.1).
//!
//! Every path the configuration or a compaction plan names is a string the
//! engine later joins onto the repository root. Before spec 144 each key had
//! its own check, or none: `derived_dir` refused `/`, `..`, `\` and `:` (spec
//! 128); `specs_dir` and `standards_dir` were not checked at all; a compact plan
//! refused a leading `/` or `\` (spec 134, WF-8) and let `C:foo`, a
//! drive-relative path on Windows, through. The rule here is decided on the
//! string, the same on every platform, and never resolves anything against the
//! filesystem, so a repository reached through a link is unaffected.

use serde::{Deserialize, Serialize};

/// Why `raw` is not a path inside the repository, or `None` when it is one.
///
/// Refused on every platform:
/// - a leading `/` or `\` (absolute or root-relative);
/// - any `:` (a Windows drive, a drive-relative path such as `C:foo`, or an
///   alternate data stream);
/// - any `\` (a separator on Windows, so `a\..\..\x` would climb there);
/// - a NUL byte;
/// - a `..` segment;
/// - a segment that is a reserved Windows device name (`CON`, `nul.txt`,
///   `COM1`), which Windows opens as the device wherever it appears;
/// - a segment ending in `.` or a space, other than `.` itself, which Windows
///   strips, so two spellings would name one directory there and two here.
///
/// An empty string and `.` name the repository root and are accepted; a caller
/// that must not name the root refuses them itself.
pub fn repo_path_problem(raw: &str) -> Option<&'static str> {
    if raw.starts_with('/') || raw.starts_with('\\') {
        return Some("is an absolute path");
    }
    if raw.contains(':') {
        return Some("contains a ':', a drive, drive-relative or stream form on Windows");
    }
    if raw.contains('\\') {
        return Some("contains a '\\', which is a separator on Windows");
    }
    if raw.contains('\0') {
        return Some("contains a NUL byte");
    }
    for segment in raw.split('/') {
        if segment == ".." {
            return Some("has a '..' segment");
        }
        if segment.is_empty() || segment == "." {
            continue;
        }
        if is_device_name(segment) {
            return Some("has a segment that is a reserved Windows device name");
        }
        if segment.ends_with(['.', ' ']) {
            return Some("has a segment ending in '.' or a space, which Windows strips");
        }
    }
    None
}

/// Whether `name` is a reserved Windows device name (spec 127 §3.5): its stem,
/// the text before the first `.` with trailing spaces removed, is `CON`, `PRN`,
/// `AUX`, `NUL`, `COM1`..`COM9`, `LPT1`..`LPT9` or a superscript `COM¹²³` /
/// `LPT¹²³`, ignoring ASCII case.
pub fn is_device_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).trim_end_matches(' ');
    if ["CON", "PRN", "AUX", "NUL"]
        .iter()
        .any(|d| stem.eq_ignore_ascii_case(d))
    {
        return true;
    }
    let mut chars = stem.chars();
    let prefix: String = chars.by_ref().take(3).collect();
    let (digit, rest) = (chars.next(), chars.next());
    (prefix.eq_ignore_ascii_case("COM") || prefix.eq_ignore_ascii_case("LPT"))
        && rest.is_none()
        && digit.is_some_and(|d| matches!(d, '1'..='9' | '\u{b9}' | '\u{b2}' | '\u{b3}'))
}

/// A validated repository-relative path: a string [`repo_path_problem`]
/// accepts. Serializes as the string it holds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepoPath(String);

impl RepoPath {
    /// Validate `raw`, returning why it was refused.
    pub fn parse(raw: &str) -> Result<Self, String> {
        match repo_path_problem(raw) {
            None => Ok(RepoPath(raw.to_string())),
            Some(why) => Err(format!("'{raw}' {why}")),
        }
    }

    /// The path as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RepoPath {
    type Error = String;
    fn try_from(raw: String) -> Result<Self, Self::Error> {
        RepoPath::parse(&raw)
    }
}

impl From<RepoPath> for String {
    fn from(p: RepoPath) -> String {
        p.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_relative_paths_pass() {
        for ok in [
            "",
            ".",
            "specs",
            "./specs",
            "a/b/c",
            ".statecraft/derived",
            "specs/",
            "a.b/c",
        ] {
            assert_eq!(repo_path_problem(ok), None, "{ok}");
        }
    }

    #[test]
    fn every_escape_and_windows_hazard_is_refused() {
        for bad in [
            "/etc",
            "\\x",
            "C:foo",
            "C:\\x",
            "a:b",
            "a\\b",
            "..",
            "a/../b",
            "a/..",
            "con",
            "a/NUL.txt",
            "com1/x",
            "lpt9",
            "a./b",
            "a /b",
            "x\0y",
        ] {
            assert!(repo_path_problem(bad).is_some(), "{bad:?}");
        }
    }

    #[test]
    fn the_type_refuses_what_the_rule_refuses() {
        assert!(RepoPath::parse("specs").is_ok());
        assert!(RepoPath::parse("C:foo").is_err());
        let back: Result<RepoPath, _> = serde_json::from_str("\"../x\"");
        assert!(back.is_err());
    }
}
