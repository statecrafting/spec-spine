// Spec: specs/002-registry-query/spec.md
//! Query: load_registry version gating + list / show / status_report /
//! relationships over a compiled registry.

use std::fs;
use std::path::Path;

use spec_spine_core::{
    ListFilter, compile, list, load_registry, relationships, show, status_report,
};
use spec_spine_types::{Config, Error, Status};

fn write_spec(root: &Path, id: &str, extra: &str) {
    let spec_dir = root.join("specs").join(id);
    fs::create_dir_all(&spec_dir).unwrap();
    let body = format!(
        "---\nid: \"{id}\"\ntitle: \"Title {id}\"\nstatus: approved\ncreated: \"2026-06-08\"\nsummary: \"s\"\n{extra}---\n# {id}\n"
    );
    fs::write(spec_dir.join("spec.md"), body).unwrap();
}

fn corpus() -> spec_spine_types::Registry {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-alpha", "");
    write_spec(tmp.path(), "002-beta", "depends_on: [\"001-alpha\"]\n");
    compile(&Config::default(), tmp.path()).unwrap().registry
}

#[test]
fn load_registry_rejects_unknown_major() {
    let bad = r#"{"specVersion":"9.0.0","build":{"compilerId":"x","compilerVersion":"0.1.0","inputRoot":".","contentHash":"0000000000000000000000000000000000000000000000000000000000000000"},"specs":[],"validation":{"passed":true,"violations":[]}}"#;
    let err = load_registry(bad.as_bytes()).unwrap_err();
    assert!(matches!(err, Error::Schema(_)));
    assert_eq!(err.exit_code(), 3);
}

#[test]
fn load_registry_accepts_our_major() {
    let reg = corpus();
    let bytes = serde_json::to_vec(&reg).unwrap();
    let loaded = load_registry(&bytes).unwrap();
    assert_eq!(loaded.specs.len(), 2);
}

#[test]
fn list_filters_by_status() {
    let reg = corpus();
    assert_eq!(list(&reg, &ListFilter::default()).len(), 2);
    let approved = list(
        &reg,
        &ListFilter {
            status: Some(Status::Approved),
        },
    );
    assert_eq!(approved.len(), 2);
    let drafts = list(
        &reg,
        &ListFilter {
            status: Some(Status::Draft),
        },
    );
    assert!(drafts.is_empty());
}

#[test]
fn show_finds_or_not_found() {
    let reg = corpus();
    assert_eq!(show(&reg, "001-alpha").unwrap().id, "001-alpha");
    assert!(matches!(show(&reg, "404-x"), Err(Error::NotFound(_))));
}

#[test]
fn status_report_counts() {
    let report = status_report(&corpus());
    assert_eq!(report.total, 2);
    assert_eq!(report.approved, 2);
    assert_eq!(report.draft, 0);
}

#[test]
fn relationships_show_incoming_and_outgoing() {
    let reg = corpus();
    let alpha = relationships(&reg, "001-alpha").unwrap();
    assert_eq!(alpha.depended_on_by, vec!["002-beta".to_string()]);
    assert!(alpha.depends_on.is_empty());

    let beta = relationships(&reg, "002-beta").unwrap();
    assert_eq!(beta.depends_on, vec!["001-alpha".to_string()]);
    assert!(beta.depended_on_by.is_empty());
}
