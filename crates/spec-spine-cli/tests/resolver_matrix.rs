//! Spec 163 §3.5 through the verbs: every reason token the CLI can reach is
//! named by an `index` diagnostic and by a `content select --json` omission,
//! and §3.7's projections answer as the core golden says. `legacy-collision`
//! and `resolver-disabled` have no CLI surface (D-11); the core
//! resolver_matrix test asserts both.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[workspace]\nmembers = [\"demo\", \"declared\"]\n",
    ),
    (
        "package.json",
        "{ \"private\": true, \"workspaces\": [\"web\"] }\n",
    ),
    (
        "demo/Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    ),
    ("demo/src/broken.rs", "pub fn ok() {}\n\npub fn bad( {}\n"),
    (
        "demo/src/docs.rs",
        "/** block doc */\npub fn block_doc() {}\n\n#[doc = \"attr doc\"]\npub fn attr_doc() {}\n\npub fn multi_line_sig(\n    a: u8,\n) -> u8 {\n    a\n}\n",
    ),
    ("demo/tests/common/mod.rs", "pub fn shared() {}\n"),
    (
        "declared/Cargo.toml",
        "[package]\nname = \"declared\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[[test]]\nname = \"x\"\npath = \"tests/x.rs\"\n",
    ),
    ("declared/src/lib.rs", "pub fn d() {}\n"),
    ("declared/tests/x.rs", "#[test]\nfn t() {}\n"),
    (
        "web/package.json",
        "{ \"name\": \"web\", \"version\": \"0.1.0\" }\n",
    ),
    (
        "web/src/util.ts",
        "export class Helper {\n  run() {}\n  field = 1;\n  #priv() {}\n}\nconst { a } = { a: 1 };\nexport namespace NS {\n  export function inNs() {}\n}\nfunction internalFn() {}\nexport { internalFn as renamed };\n",
    ),
    (
        "web/src/comp.tsx",
        "export class Comp {\n  render() {}\n}\n",
    ),
];

const DEMO_LIB: &str = "mod broken;
pub mod docs;

pub fn outer() {
    fn nested() {}
}

pub struct Shape {
    pub field: u8,
}

pub trait Tr {
    fn m(&self);
}

impl Tr for &Shape {
    fn m(&self) {}
}

pub use crate::outer as reexported;

unsafe extern \"C\" {
    fn foreign_fn();
}

pub mod generated {
    make_items!();
}

#[cfg(test)]
mod tests {
    #[test]
    fn works() {}

    #[tokio::test]
    async fn tokio_test() {}
}
";

/// Every symbol unit the fixture spec declares, with the outcome its `index`
/// diagnostic and its `content select` omission must name.
const CASES: &[(&str, &str)] = &[
    ("demo::outer::nested", "unsupported: nested-item"),
    ("demo::<&Shape as Tr>::m", "unsupported: impl-type-form"),
    ("demo::reexported", "unsupported: re-export"),
    ("demo::Shape::field", "unsupported: member"),
    ("demo::foreign_fn", "unsupported: foreign-item"),
    (
        "demo/tests/common/mod::shared",
        "unsupported: unrooted-test-file",
    ),
    ("declared/tests/x::t", "unsupported: declared-test-targets"),
    ("web::src::util::NS::inNs", "unsupported: ts-namespace"),
    ("web::src::util::a", "unsupported: ts-destructuring"),
    (
        "web::src::util::Helper::#priv",
        "unsupported: ts-computed-name",
    ),
    ("web::src::comp::Comp::render", "unsupported: tsx-grammar"),
    ("demo::generated::made", "unknown: item-macro-in-scope"),
    ("demo::broken::missing", "unknown: parse-error-in-scope"),
    ("demo::{DEEP}::leaf", "unknown: nesting-limit"),
];

/// `m1::m2::…::m33`: one level past the walk's depth limit (§3.1).
fn deep_path() -> String {
    (1..=33)
        .map(|i| format!("m{i}"))
        .collect::<Vec<_>>()
        .join("::")
}

fn deep_chain() -> String {
    let mut out = String::new();
    for i in 1..=33 {
        out.push_str(&format!("pub mod m{i} {{\n"));
    }
    out.push_str("pub fn leaf() {}\n");
    for _ in 1..=33 {
        out.push_str("}\n");
    }
    out
}

fn id(case: &str) -> String {
    case.replace("{DEEP}", &deep_path())
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The fixture, compiled, indexed and committed, so `content select` binds a
/// clean head against a fresh committed index. Returns the `index` output.
fn fixture() -> (tempfile::TempDir, String) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    for (path, content) in FILES {
        write(root, path, content);
    }
    write(
        root,
        "demo/src/lib.rs",
        &format!("{DEMO_LIB}{}", deep_chain()),
    );
    let units: String = CASES
        .iter()
        .map(|(case, _)| format!("  - {{ kind: symbol, id: {:?} }}\n", id(case)))
        .collect();
    write(
        root,
        "specs/001-matrix/spec.md",
        &format!(
            "---\nid: \"001-matrix\"\ntitle: \"Matrix\"\nstatus: draft\ncreated: \"2026-10-13\"\nimplementation: pending\nsummary: \"matrix\"\nestablishes:\n{units}---\n# Matrix\n"
        ),
    );
    let compile = run(root, &["compile"]);
    assert_eq!(
        compile.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let index = run(root, &["index"]);
    assert_eq!(
        index.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&index.stderr)
    );
    let mut text = String::from_utf8_lossy(&index.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&index.stderr));
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    git(root, &["config", "user.name", "Test"]);
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "fixture"]);
    (tmp, text)
}

fn select(root: &Path, selectors: serde_json::Value) -> serde_json::Value {
    let dir = tempfile::tempdir().unwrap();
    let request = dir.path().join("request.json");
    fs::write(
        &request,
        serde_json::to_vec(&serde_json::json!({ "selectors": selectors })).unwrap(),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(["content", "select", "--request"])
        .arg(&request)
        .args(["--repository", "example/repo", "--json"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

/// The omission for `identity`, as `(reason, message)`.
fn omission(response: &serde_json::Value, identity: &str) -> (String, String) {
    let found = response["omissions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["identity"] == identity)
        .unwrap_or_else(|| panic!("no omission for {identity}: {response}"));
    (
        found["reason"].as_str().unwrap().to_string(),
        found["message"].as_str().unwrap().to_string(),
    )
}

#[test]
fn every_reachable_reason_is_named_by_index_and_content_select() {
    let (repo, index_output) = fixture();
    let selectors: Vec<serde_json::Value> = CASES
        .iter()
        .map(|(case, _)| serde_json::json!({"kind":"symbol", "id": id(case), "required": false}))
        .collect();
    let response = select(repo.path(), serde_json::Value::Array(selectors));
    for (case, outcome) in CASES {
        let id = id(case);
        let line = format!("symbol unit '{id}' did not resolve ({outcome})");
        assert!(
            index_output.contains(&line),
            "index names {outcome} for {id}:\n{index_output}"
        );
        let (kind, reason) = outcome.split_once(": ").unwrap();
        let (got, message) = omission(&response, &format!("symbol:{id}"));
        let expected = if kind == "unsupported" {
            "unsupported-selector"
        } else {
            "indeterminate-selector"
        };
        assert_eq!(got, expected, "{id}");
        assert_eq!(message, format!("symbol '{id}' is {kind}: {reason}"));
    }
}

#[test]
fn test_selectors_name_their_reasons_and_resolve_a_bare_test() {
    let (repo, _) = fixture();
    let response = select(
        repo.path(),
        serde_json::json!([
            {"kind":"test", "id":"demo::tests::works"},
            {"kind":"test", "id":"demo::tests::tokio_test", "required": false},
            {"kind":"test", "id":"web::src::util::does a thing", "required": false},
            {"kind":"test", "id":"demo::outer", "required": false},
        ]),
    );
    let item = response["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["identity"] == "test:demo::tests::works")
        .unwrap_or_else(|| panic!("the bare test resolves: {response}"));
    assert_eq!(item["content"], "    #[test]\n    fn works() {}");
    assert_eq!(
        omission(&response, "test:demo::tests::tokio_test"),
        (
            "unsupported-selector".into(),
            "test 'demo::tests::tokio_test' is unsupported: framework-test-attribute".into()
        )
    );
    assert_eq!(
        omission(&response, "test:web::src::util::does a thing"),
        (
            "unsupported-selector".into(),
            "test 'web::src::util::does a thing' is unsupported: typescript-test".into()
        )
    );
    assert_eq!(
        omission(&response, "test:demo::outer"),
        (
            "missing-content".into(),
            "test 'demo::outer' is not a test function".into()
        )
    );
}

/// §1.1 rows 18, 19 and 21, through the verb.
#[test]
fn projections_follow_the_grammar() {
    let (repo, _) = fixture();
    let response = select(
        repo.path(),
        serde_json::json!([
            {"kind":"symbol", "id":"demo::docs::block_doc", "projection":"documentation"},
            {"kind":"symbol", "id":"demo::docs::attr_doc", "projection":"documentation", "required": false},
            {"kind":"symbol", "id":"demo::docs::multi_line_sig", "projection":"signature", "required": false},
            {"kind":"symbol", "id":"demo::docs::multi_line_sig", "projection":"body", "required": false},
        ]),
    );
    let doc = response["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["identity"] == "symbol:demo::docs::block_doc")
        .unwrap_or_else(|| panic!("row 18 resolves: {response}"));
    assert_eq!(doc["content"], "/** block doc */");
    assert_eq!(
        omission(&response, "symbol:demo::docs::attr_doc"),
        (
            "unsupported-projection".into(),
            "attribute documentation".into()
        )
    );
    let shared: Vec<&serde_json::Value> = response["omissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["identity"] == "symbol:demo::docs::multi_line_sig")
        .collect();
    assert_eq!(shared.len(), 2, "row 21 has neither projection: {response}");
    assert!(
        shared
            .iter()
            .all(|o| o["reason"] == "unsupported-projection"),
        "{response}"
    );
}
