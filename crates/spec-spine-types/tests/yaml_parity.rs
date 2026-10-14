//! Spec 196 §3.3: the refusal and acceptance parity table. Each row is one
//! frontmatter, the extra keys a config declares for it, and the outcome the
//! frontmatter reader gives: `V-002` (malformed), `V-013` (a declared key's
//! value JSON cannot carry), or the canonical JSON of the extra keys it
//! accepted. The `serde_yaml` column is what the pre-196 reader (serde_yaml
//! 0.9.34 over unsafe-libyaml 0.2.11) answered, captured by running this same
//! table against it before it was removed; the `now` column is what this
//! build answers. A row whose columns differ is a decided divergence, named by
//! the spec 196 decision that keeps it.

use spec_spine_types::frontmatter::{FrontmatterIssue, parse_frontmatter_with};

const BASE: &str = "id: \"001-a\"\ntitle: \"A\"\nstatus: draft\ncreated: \"2026-10-14\"\nimplementation: pending\nsummary: \"a\"\n";

/// `(name, extra frontmatter lines, declared keys, serde_yaml, now)`.
const ROWS: &[(&str, &str, &[&str], &str, &str)] = &[
    ("duplicate key", "x-a: 1\nx-a: 2\n", &[], "V-002", "V-002"),
    (
        "duplicate known key",
        "title: \"B\"\n",
        &[],
        "V-002",
        "V-002",
    ),
    (
        "yaml 1.1 words as values",
        "x-b: yes\nx-c: no\nx-d: on\nx-e: off\nx-f: y\nx-g: n\nx-h: Yes\nx-i: NO\n",
        &[],
        r#"{"x-b":"yes","x-c":"no","x-d":"on","x-e":"off","x-f":"y","x-g":"n","x-h":"Yes","x-i":"NO"}"#,
        r#"{"x-b":"yes","x-c":"no","x-d":"on","x-e":"off","x-f":"y","x-g":"n","x-h":"Yes","x-i":"NO"}"#,
    ),
    (
        "yaml 1.1 words as keys",
        "yes: 1\non: 2\noff: 3\n",
        &[],
        r#"{"off":3,"on":2,"yes":1}"#,
        r#"{"off":3,"on":2,"yes":1}"#,
    ),
    (
        "true and false",
        "x-t: true\nx-f: false\nx-u: True\nx-v: FALSE\n",
        &[],
        r#"{"x-f":false,"x-t":true,"x-u":true,"x-v":false}"#,
        r#"{"x-f":false,"x-t":true,"x-u":true,"x-v":false}"#,
    ),
    ("a boolean key", "true: 1\n", &[], "V-002", "V-002"),
    ("an integer key", "1: a\n", &[], "V-002", "V-002"),
    (
        "null forms drop the key",
        "x-a: ~\nx-b: null\nx-c:\nx-d: Null\n",
        &[],
        "{}",
        "{}",
    ),
    (
        "octal 0o17",
        "x-n: 0o17\n",
        &[],
        r#"{"x-n":15}"#,
        r#"{"x-n":15}"#,
    ),
    // D-3: a leading-zero numeral is a number, read as the float 17.0.
    (
        "leading zero 017",
        "x-n: 017\n",
        &[],
        r#"{"x-n":"017"}"#,
        r#"{"x-n":17.0}"#,
    ),
    (
        "exponent 1e3",
        "x-n: 1e3\n",
        &[],
        r#"{"x-n":1000.0}"#,
        r#"{"x-n":1000.0}"#,
    ),
    // D-4: a non-finite float is refused by the parser, everywhere.
    ("infinity", "x-n: .inf\n", &[], "V-002", "V-002"),
    ("not a number", "x-n: .nan\n", &[], "V-002", "V-002"),
    (
        "declared infinity",
        "x-n: .inf\n",
        &["x-n"],
        "V-013",
        "V-002",
    ),
    (
        "plus sign",
        "x-n: +1\n",
        &[],
        r#"{"x-n":1}"#,
        r#"{"x-n":1}"#,
    ),
    // D-3: an underscore-grouped numeral is an integer.
    (
        "underscores 1_000",
        "x-n: 1_000\n",
        &[],
        r#"{"x-n":"1_000"}"#,
        r#"{"x-n":1000}"#,
    ),
    (
        "hex 0x1F",
        "x-n: 0x1F\n",
        &[],
        r#"{"x-n":31}"#,
        r#"{"x-n":31}"#,
    ),
    (
        "past i64",
        "x-n: 18446744073709551615\n",
        &[],
        r#"{"x-n":1.8446744073709552e+19}"#,
        r#"{"x-n":1.8446744073709552e+19}"#,
    ),
    (
        "anchor and alias",
        "x-a: &v hello\nx-b: *v\n",
        &[],
        r#"{"x-a":"hello","x-b":"hello"}"#,
        r#"{"x-a":"hello","x-b":"hello"}"#,
    ),
    (
        "a merge key stays a key",
        "x-base: &b {k: 1}\nx-m:\n  <<: *b\n  y: 2\n",
        &["x-base", "x-m"],
        r#"{"x-base":{"k":1},"x-m":{"<<":{"k":1},"y":2}}"#,
        r#"{"x-base":{"k":1},"x-m":{"<<":{"k":1},"y":2}}"#,
    ),
    (
        "an undeclared nested map",
        "x-m: {k: 1}\n",
        &[],
        "V-002",
        "V-002",
    ),
    (
        "a declared non-string nested key",
        "x-m: {1: a}\n",
        &["x-m"],
        "V-013",
        "V-013",
    ),
    (
        "core tag !!str",
        "x-s: !!str 5\n",
        &[],
        r#"{"x-s":"5"}"#,
        r#"{"x-s":"5"}"#,
    ),
    ("custom tag", "x-c: !custom x\n", &[], "V-002", "V-002"),
    // D-5: the parser refuses an unknown tag before the declared path reads it.
    (
        "declared custom tag",
        "x-c: !custom x\n",
        &["x-c"],
        "V-013",
        "V-002",
    ),
    // D-5: serde_yaml dropped an unknown tag on a known key and kept the value.
    (
        "custom tag on a known key",
        "kind: !custom governance\n",
        &[],
        "{}",
        "V-002",
    ),
    // A `---` line closes the fence, so a second document can only follow an
    // explicit `...` end marker.
    (
        "a second document",
        "x-a: 1\n...\nx-b: 2\n",
        &[],
        "V-002",
        "V-002",
    ),
    (
        "non-ASCII keys and values",
        "x-ключ: значение\nx-k: é\n",
        &[],
        r#"{"x-k":"é","x-ключ":"значение"}"#,
        r#"{"x-k":"é","x-ключ":"значение"}"#,
    ),
    (
        "block scalars",
        "x-l: |\n  a\n  b\nx-f: >\n  a\n  b\nx-s: |-\n  z\nx-k: >+\n  q\n\n",
        &[],
        "{\"x-f\":\"a b\\n\",\"x-k\":\"q\\n\\n\",\"x-l\":\"a\\nb\\n\",\"x-s\":\"z\"}",
        "{\"x-f\":\"a b\\n\",\"x-k\":\"q\\n\\n\",\"x-l\":\"a\\nb\\n\",\"x-s\":\"z\"}",
    ),
    (
        "a string list",
        "x-l: [a, b]\n",
        &[],
        r#"{"x-l":["a","b"]}"#,
        r#"{"x-l":["a","b"]}"#,
    ),
    ("a mixed list", "x-l: [a, 1]\n", &[], "V-002", "V-002"),
    (
        "a wrong type on a known key",
        "title: 5\n",
        &[],
        "V-002",
        "V-002",
    ),
    (
        "a list where a string is",
        "title: [a]\n",
        &[],
        "V-002",
        "V-002",
    ),
    ("not a mapping", "- a\n", &[], "V-002", "V-002"),
    // Specs 057 and 061 hold such a key: null reads as the empty list.
    (
        "a list key holding only comments",
        "extends:\n  # nothing yet\nreferences:\n",
        &[],
        "{}",
        "{}",
    ),
    (
        "an unquoted date",
        "x-d: 2026-10-14\n",
        &[],
        r#"{"x-d":"2026-10-14"}"#,
        r#"{"x-d":"2026-10-14"}"#,
    ),
];

fn outcome(extra: &str, declared: &[&str], bom: bool) -> String {
    // A row that redefines a known key replaces it, so the base stays valid.
    let base: String = BASE
        .lines()
        .filter(|line| {
            let key = line.split(':').next().unwrap_or("");
            !extra.lines().any(|l| l.starts_with(&format!("{key}:")))
        })
        .map(|line| format!("{line}\n"))
        .collect();
    let base = if extra.starts_with("title: \"B\"") {
        BASE.to_string()
    } else {
        base
    };
    let doc = format!(
        "{}---\n{base}{extra}---\n# A\n",
        if bom { "\u{feff}" } else { "" }
    );
    let declared: Vec<String> = declared.iter().map(|d| d.to_string()).collect();
    match parse_frontmatter_with(&doc, &declared) {
        Ok(fm) => serde_json::to_string(&fm.extra_frontmatter).unwrap(),
        Err(FrontmatterIssue::Malformed(_)) => "V-002".into(),
        Err(FrontmatterIssue::UnrepresentableDeclared { .. }) => "V-013".into(),
    }
}

#[test]
fn every_row_answers_its_recorded_outcome() {
    let mut failures = Vec::new();
    for (name, extra, declared, _old, now) in ROWS {
        let got = outcome(extra, declared, false);
        if std::env::var_os("SPEC_SPINE_PRINT_PARITY").is_some() {
            println!("{name}\t{got}");
        }
        if got != *now {
            failures.push(format!("{name}: expected {now}, got {got}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A byte-order mark before the opening fence changes nothing (spec 143).
#[test]
fn a_byte_order_mark_changes_nothing() {
    for (name, extra, declared, _, _) in ROWS {
        assert_eq!(
            outcome(extra, declared, true),
            outcome(extra, declared, false),
            "{name}"
        );
    }
}

/// Every divergence from the pre-196 reader is one a decision names: the rows
/// whose columns differ are exactly these.
#[test]
fn the_divergences_are_the_decided_ones() {
    let differing: Vec<&str> = ROWS
        .iter()
        .filter(|(_, _, _, old, now)| old != now)
        .map(|(name, ..)| *name)
        .collect();
    assert_eq!(
        differing,
        [
            "leading zero 017",
            "declared infinity",
            "underscores 1_000",
            "declared custom tag",
            "custom tag on a known key",
        ]
    );
}
