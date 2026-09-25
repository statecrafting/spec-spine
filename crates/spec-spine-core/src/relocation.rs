//! A relocation is proven, not asserted (spec 142 §3.3).
//!
//! A spec that takes over a section of another spec declares it
//! (`relocates: [{ spec, from, to? }]`). `delta` then compares the section as it
//! was in the source at the merge base with the section as it is in the
//! receiver at head, through [`relocation_digest`]; when they match, and the
//! source's body at head is its base body with exactly those sections removed
//! ([`relocation_only`]), the source's change is a `relocation`, not a
//! `requirement`.
//!
//! Both comparisons ignore what a move legitimately changes and nothing else:
//! the section's own heading line (its number and level belong to where it
//! sits), the leading section numbers of every heading inside it, heading levels
//! inside the moved text, trailing whitespace, and runs of blank lines. Every
//! other byte must match. Pure functions of text.

use std::collections::BTreeSet;

use crate::hash;
use crate::sections::{markdown_section_spans, markdown_section_texts};

/// A section's relocation digest: [`hash::content_hash`] over the section text
/// without its own heading line, normalized as the module describes, under an
/// empty name. The empty name is the point: a section digest in the registry
/// is keyed `<specPath>#<anchor>` (spec 106), so the same text in two specs is
/// two identities, and a relocation is exactly the case where the text is the
/// same and the place is not.
pub fn relocation_digest(section_text: &str) -> String {
    let mut lines = section_text.lines();
    lines.next(); // the section's own heading
    let normalized = normalize(&lines.collect::<Vec<_>>().join("\n"), false);
    hash::content_hash(vec![(String::new(), normalized)])
}

/// The relocation digest of the first section anchored `anchor` in `body`, if
/// there is one.
pub fn section_relocation_digest(body: &str, anchor: &str) -> Option<String> {
    markdown_section_texts(body)
        .into_iter()
        .find(|(a, _)| a == anchor)
        .map(|(_, text)| relocation_digest(&text))
}

/// Whether `head` is `base` with the sections anchored in `removed` taken out
/// (each with its subsections), and nothing else changed except the leading
/// numbers of the headings that remain and blank-line runs.
///
/// `removed` must be non-empty: a body that changed without losing a relocated
/// section is a requirement change, whatever else it is.
pub fn relocation_only(base: &str, head: &str, removed: &BTreeSet<String>) -> bool {
    if removed.is_empty() {
        return false;
    }
    let lines: Vec<&str> = base.lines().collect();
    let mut drop = vec![false; lines.len()];
    let mut found: BTreeSet<&str> = BTreeSet::new();
    for (anchor, span) in markdown_section_spans(base) {
        if !removed.contains(&anchor) || !found.insert(removed.get(&anchor).map_or("", String::as_str)) {
            continue;
        }
        let from = span.start_line.saturating_sub(1);
        let to = span.end_line.min(lines.len());
        for d in drop.iter_mut().take(to).skip(from) {
            *d = true;
        }
    }
    if found.len() != removed.len() {
        return false; // a declared section the base does not have
    }
    let rest: Vec<&str> = lines
        .iter()
        .zip(&drop)
        .filter(|(_, d)| !**d)
        .map(|(l, _)| *l)
        .collect();
    normalize(&rest.join("\n"), true) == normalize(head, true)
}

/// Normalize markdown text for comparison. Headings outside fences lose their
/// leading section number; with `keep_levels` false they also lose their level.
/// Trailing whitespace is dropped, runs of blank lines collapse to one, and
/// leading and trailing blank lines go.
fn normalize(text: &str, keep_levels: bool) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut in_fence = false;
    for raw in text.lines() {
        let line = raw.trim_end();
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            out.push(line.to_string());
            continue;
        }
        let level = t.bytes().take_while(|&b| b == b'#').count();
        let heading = !in_fence
            && (1..=6).contains(&level)
            && t[level..].starts_with([' ', '\t']);
        if heading {
            let text = strip_number(t[level..].trim());
            let hashes = if keep_levels { "#".repeat(level) } else { "#".to_string() };
            out.push(format!("{hashes} {text}"));
        } else if line.is_empty() {
            if out.last().is_some_and(String::is_empty) || out.is_empty() {
                continue;
            }
            out.push(String::new());
        } else {
            out.push(line.to_string());
        }
    }
    while out.last().is_some_and(String::is_empty) {
        out.pop();
    }
    out.join("\n")
}

/// A heading's text without a leading section number (`3`, `3.`, `3.1`,
/// `3.1.2.`, optionally after `§`).
fn strip_number(text: &str) -> &str {
    let t = text.strip_prefix('§').unwrap_or(text);
    let mut end = 0;
    let bytes = t.as_bytes();
    let mut saw_digit = false;
    while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
        saw_digit |= bytes[end].is_ascii_digit();
        end += 1;
    }
    if saw_digit && end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
        t[end..].trim_start()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_levels_and_blank_runs_do_not_matter_and_words_do() {
        let a = "## 2. Y\n\nBody.\n\n### 2.1 Sub\n\nMore.\n";
        let b = "### 5. Y\n\n\nBody.\n\n#### 5.1 Sub\n\n\n\nMore.   \n";
        assert_eq!(relocation_digest(a), relocation_digest(b));
        let c = "### 5. Y\n\nBody!\n\n#### 5.1 Sub\n\nMore.\n";
        assert_ne!(relocation_digest(a), relocation_digest(c));
    }

    #[test]
    fn a_relocation_only_body_is_the_base_less_the_sections() {
        let base = "# 002\n\n## 1. X\n\nx\n\n## 2. Y\n\ny\n\n### 2.1 Z\n\nz\n\n## 3. W\n\nw\n";
        let head = "# 002\n\n## 1. X\n\nx\n\n## 2. W\n\nw\n";
        let removed: BTreeSet<String> = ["2-y".to_string()].into();
        assert!(relocation_only(base, head, &removed));
        let edited = "# 002\n\n## 1. X\n\nx, edited\n\n## 2. W\n\nw\n";
        assert!(!relocation_only(base, edited, &removed));
        let none: BTreeSet<String> = BTreeSet::new();
        assert!(!relocation_only(base, head, &none));
        let absent: BTreeSet<String> = ["9-q".to_string()].into();
        assert!(!relocation_only(base, head, &absent));
    }

    #[test]
    fn a_fenced_hash_line_is_not_a_heading() {
        assert_eq!(strip_number("3.1 The rule"), "The rule");
        assert_eq!(strip_number("§3 The rule"), "The rule");
        assert_eq!(strip_number("Out of scope"), "Out of scope");
        let a = "## 1. A\n\n```sh\n# 1. comment\n```\n";
        let b = "## 1. A\n\n```sh\n# 2. comment\n```\n";
        assert_ne!(relocation_digest(a), relocation_digest(b));
    }
}
