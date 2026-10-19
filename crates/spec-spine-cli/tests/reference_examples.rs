//! Spec 166 3.7: the examples a reference page renders are declared, not
//! run, by generation. This is the separate acceptance step that runs every
//! example of the fixture catalog against the built binary, inside a copied
//! fixture corpus, and asserts its exit code and stdout substrings.

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../spec-spine-core/tests/fixtures/reference"
);

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            fs::copy(e.path(), target).unwrap();
        }
    }
}

fn bin(root: &Path, args: &[String], stdin: Option<&str>) -> std::process::Output {
    use std::io::Write as _;
    let mut child = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .current_dir(root)
        .arg("--repo")
        .arg(root)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(text) = stdin {
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
    }
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&Path::new(FIXTURE).join("corpus"), tmp.path());
    for verb in ["compile", "index"] {
        let out = bin(tmp.path(), &[verb.to_string()], None);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    tmp
}

/// The fixture catalog: this build's catalog, its `registry.list` operation
/// governed by `001-alpha` and carrying the fixture's declared examples.
fn fixture_catalog() -> serde_json::Value {
    let mut catalog = spec_spine_core::capability_catalog().unwrap();
    catalog.operations.retain(|o| o.name == "registry.list");
    assert_eq!(
        catalog.operations.len(),
        1,
        "the catalog must hold exactly one registry.list operation"
    );
    let examples: Vec<spec_spine_types::Example> = serde_json::from_str(
        &fs::read_to_string(Path::new(FIXTURE).join("examples.json")).unwrap(),
    )
    .unwrap();
    catalog.operations[0].governed_by = vec!["001-alpha".into()];
    catalog.operations[0].examples = examples;
    serde_json::to_value(catalog).unwrap()
}

#[test]
fn every_rendered_example_holds_against_the_built_binary() {
    let repo = corpus();
    let root = repo.path();
    fs::write(
        root.join("catalog.json"),
        serde_json::to_string_pretty(&fixture_catalog()).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("reference-request.json"),
        r#"{"schemaVersion":"1.0.0","outputDir":"docs/reference","specs":{"ids":["001-alpha"]},"catalog":"catalog.json"}"#,
    )
    .unwrap();
    let model = bin(
        root,
        &[
            "reference",
            "model",
            "--request",
            "reference-request.json",
            "--json",
        ]
        .map(String::from),
        None,
    );
    assert_eq!(
        model.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&model.stderr)
    );
    let model: serde_json::Value = serde_json::from_slice(&model.stdout).unwrap();
    let examples = model["pages"][0]["operations"][0]["examples"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(examples.len(), 3, "every declared example is rendered");

    // Generation spawned nothing: the pages carry the examples as data only.
    let page = bin(
        root,
        &[
            "reference",
            "generate",
            "--request",
            "reference-request.json",
        ]
        .map(String::from),
        None,
    );
    assert_eq!(page.status.code(), Some(0));
    let text = fs::read_to_string(root.join("docs/reference/001-alpha.md")).unwrap();
    assert!(text.contains("#### Example `registry-list-names-alpha`"));

    for e in examples {
        let example: spec_spine_types::Example = serde_json::from_value(e).unwrap();
        // `corpus` is the one fixture this test builds; another name would
        // run the example in the wrong environment.
        assert_eq!(
            example.fixture.as_deref(),
            Some("corpus"),
            "unknown example fixture"
        );
        let fixture = corpus();
        let out = bin(fixture.path(), &example.argv, example.stdin.as_deref());
        assert_eq!(
            out.status.code(),
            Some(i32::from(example.exit_code)),
            "{}: {}",
            example.id,
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        for needle in &example.stdout_includes {
            assert!(
                stdout.contains(needle.as_str()),
                "{}: stdout lacks {needle}",
                example.id
            );
        }
    }
}
