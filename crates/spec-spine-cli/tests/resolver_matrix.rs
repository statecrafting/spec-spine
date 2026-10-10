//! Spec 163: every reason token of §3.5 surfaced through the CLI, on the
//! `index` diagnostic and on a `content select --json` omission. Two tokens
//! have no CLI surface and are asserted in core only: `legacy-collision`
//! (its id resolves to the legacy owner, by definition) and
//! `resolver-disabled` (this binary is built with `symbol-resolution`).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
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

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

const LIB: &str = r#"pub mod generated {
    make!(hidden);
}

pub mod broken_scope;

pub struct Pair {
    pub width: u8,
}

pub fn outer() {
    fn nested() {}
}

pub trait Shape {
    fn area(&self) -> u8;
}

impl Shape for &Pair {
    fn area(&self) -> u8 {
        1
    }
}

impl Pair {
    pub fn left(&self) -> u8 {
        self.width
    }
}

pub use generated::hidden as reexported;

unsafe extern "C" {
    fn foreign();
}

#[cfg(test)]
mod tests {
    /// Proves the pair.
    #[test]
    fn proves() {}

    #[tokio::test]
    async fn framework() {}
}
"#;

/// A repository whose draft spec declares one unit per reason the index can
/// surface, plus the resolved constructs, committed so `content select` binds.
fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"demo\", \"declared\"]\n",
    );
    write(
        root,
        "package.json",
        r#"{ "private": true, "workspaces": ["web"] }"#,
    );
    write(
        root,
        "demo/Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    let mut deep = String::from("pub mod d0 {\n");
    for i in 1..=33 {
        deep.push_str(&format!("pub mod d{i} {{\n"));
    }
    deep.push_str("pub fn leaf() {}\n");
    deep.push_str(&"}\n".repeat(34));
    write(root, "demo/src/lib.rs", &format!("{LIB}{deep}"));
    write(
        root,
        "demo/src/broken_scope.rs",
        "pub fn ok() {}\npub fn bad( {}\n",
    );
    write(
        root,
        "demo/tests/flow.rs",
        "#[test]\nfn flows() {}\n\nfn helper() {}\n",
    );
    write(root, "demo/tests/common/mod.rs", "pub fn shared() {}\n");
    write(
        root,
        "declared/Cargo.toml",
        "[package]\nname = \"declared\"\nversion = \"0.1.0\"\nedition = \"2024\"\nautotests = false\n",
    );
    write(root, "declared/src/lib.rs", "pub fn d() {}\n");
    write(root, "declared/tests/x.rs", "#[test]\nfn t() {}\n");
    write(root, "web/package.json", r#"{ "name": "web" }"#);
    write(
        root,
        "web/src/util.ts",
        "export class Box {\n  open() {}\n  #seal() {}\n}\nexport namespace NS {\n  export function inner() {}\n}\nconst { a } = { a: 1 };\nfunction local() {}\nexport { local as renamed };\nit(\"works\", () => {});\n",
    );
    write(
        root,
        "web/src/view.tsx",
        "export class View {\n  show() {}\n}\n",
    );
    let deep_path = (1..=32)
        .map(|i| format!("d{i}"))
        .collect::<Vec<_>>()
        .join("::");
    let symbols = [
        "demo::Pair::left".to_string(),
        "demo::outer::nested".into(),
        "demo::<&Pair as Shape>::area".into(),
        "demo::reexported".into(),
        "demo::Pair::width".into(),
        "demo::foreign".into(),
        "demo/tests/common/mod::shared".into(),
        "declared/tests/x::t".into(),
        "web::src::util::NS".into(),
        "web::src::util::a".into(),
        "web::src::util::Box::#seal".into(),
        "web::src::view::View::show".into(),
        "web::src::util::renamed".into(),
        "demo::generated::hidden".into(),
        "demo::broken_scope::missing".into(),
        format!("demo::d0::{deep_path}::d33::leaf"),
        "demo::absent".into(),
    ];
    let mut units = String::new();
    for id in &symbols {
        units.push_str(&format!("  - {{ kind: symbol, id: {id:?} }}\n"));
    }
    write(
        root,
        "specs/001-matrix/spec.md",
        &format!(
            "---\nid: \"001-matrix\"\ntitle: \"Matrix\"\nstatus: draft\ncreated: \"2026-10-10\"\nimplementation: pending\nsummary: \"matrix\"\nestablishes:\n{units}  - {{ kind: module, id: \"demo/tests/common/mod\" }}\n---\n# Matrix\n"
        ),
    );
    tmp
}

#[test]
fn index_messages_name_each_outcome() {
    let repo = fixture();
    let output = run(repo.path(), &["index"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let deep_path = (1..=32)
        .map(|i| format!("d{i}"))
        .collect::<Vec<_>>()
        .join("::");
    let deep = format!("demo::d0::{deep_path}::d33::leaf");
    let expected = [
        ("demo::outer::nested", "(unsupported: nested-item)"),
        (
            "demo::<&Pair as Shape>::area",
            "(unsupported: impl-type-form)",
        ),
        ("demo::reexported", "(unsupported: re-export)"),
        ("demo::foreign", "(unsupported: foreign-item)"),
        ("demo::Pair::width", "(unsupported: member)"),
        (
            "demo/tests/common/mod::shared",
            "(unsupported: unrooted-test-file)",
        ),
        (
            "declared/tests/x::t",
            "(unsupported: declared-test-targets)",
        ),
        ("web::src::util::NS", "(unsupported: ts-namespace)"),
        ("web::src::util::a", "(unsupported: ts-destructuring)"),
        (
            "web::src::util::Box::#seal",
            "(unsupported: ts-computed-name)",
        ),
        ("web::src::view::View::show", "(unsupported: tsx-grammar)"),
        ("web::src::util::renamed", "(unsupported: re-export)"),
        ("demo::generated::hidden", "(unknown: item-macro-in-scope)"),
        (
            "demo::broken_scope::missing",
            "(unknown: parse-error-in-scope)",
        ),
        (deep.as_str(), "(unknown: nesting-limit)"),
    ];
    for (id, suffix) in expected {
        let line = format!("symbol unit '{id}' did not resolve {suffix}");
        assert!(text.contains(&line), "missing `{line}` in:\n{text}");
    }
    assert!(
        text.contains(
            "module unit 'demo/tests/common/mod' did not resolve (unsupported: unrooted-test-file)"
        ),
        "{text}"
    );
    // A plain unresolved unit keeps the 0.28.0 message, with nothing after it.
    assert!(
        text.lines()
            .any(|l| l.ends_with("symbol unit 'demo::absent' did not resolve")),
        "{text}"
    );
    // A method resolves.
    assert!(
        !text.contains("'demo::Pair::left' did not resolve"),
        "{text}"
    );
}

#[test]
fn content_select_maps_each_outcome_to_an_omission() {
    let repo = fixture();
    let root = repo.path();
    for verb in ["compile", "index"] {
        let output = run(root, &[verb]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{verb}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    git(root, &["config", "user.name", "Test"]);
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "fixture"]);
    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    let selectors = serde_json::json!([
        {"kind": "test", "id": "demo::tests::proves", "required": false},
        {"kind": "test", "id": "demo/tests/flow::flows", "required": false},
        {"kind": "test", "id": "demo::tests::framework", "required": false},
        {"kind": "test", "id": "web::src::util::works", "required": false},
        {"kind": "test", "id": "demo::Pair::left", "required": false},
        {"kind": "symbol", "id": "demo::Pair::left", "projection": "full", "required": false},
        {"kind": "symbol", "id": "demo::generated::hidden", "required": false},
        {"kind": "symbol", "id": "demo::outer::nested", "required": false},
        {"kind": "module", "id": "demo/tests/common/mod", "required": false},
    ]);
    fs::write(
        &request,
        serde_json::to_vec(&serde_json::json!({ "selectors": selectors })).unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(["content", "select", "--request"])
        .arg(&request)
        .args(["--repository", "example/repo", "--json"])
        .output()
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "{e}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert_eq!(value["schemaVersion"], "0.11.0", "{value}");
    let item = |identity: &str| {
        value["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["identity"] == identity)
            .unwrap_or_else(|| panic!("no item {identity} in {value}"))
            .clone()
    };
    let omission = |identity: &str| {
        let o = value["omissions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["identity"] == identity)
            .unwrap_or_else(|| panic!("no omission {identity} in {value}"))
            .clone();
        format!(
            "{} {}",
            o["reason"].as_str().unwrap(),
            o["message"].as_str().unwrap()
        )
    };
    let proves = item("test:demo::tests::proves");
    assert_eq!(
        proves["content"], "    #[test]\n    fn proves() {}",
        "{value}"
    );
    let flows = item("test:demo/tests/flow::flows");
    assert_eq!(flows["content"], "#[test]\nfn flows() {}", "{value}");
    assert_eq!(
        item("symbol:demo::Pair::left")["content"],
        "    pub fn left(&self) -> u8 {\n        self.width\n    }"
    );
    assert_eq!(
        omission("test:demo::tests::framework"),
        "unsupported-selector test 'demo::tests::framework' is unsupported: framework-test-attribute"
    );
    assert_eq!(
        omission("test:web::src::util::works"),
        "unsupported-selector test 'web::src::util::works' is unsupported: typescript-test"
    );
    assert_eq!(
        omission("test:demo::Pair::left"),
        "missing-content test 'demo::Pair::left' is not a test function"
    );
    assert_eq!(
        omission("symbol:demo::generated::hidden"),
        "indeterminate-selector symbol 'demo::generated::hidden' is unknown: item-macro-in-scope"
    );
    assert_eq!(
        omission("symbol:demo::outer::nested"),
        "unsupported-selector symbol 'demo::outer::nested' is unsupported: nested-item"
    );
    assert_eq!(
        omission("module:demo/tests/common/mod"),
        "unsupported-selector module 'demo/tests/common/mod' is unsupported: unrooted-test-file"
    );
}
