//! Spec 155: bounded selected-content core behavior.

use std::fs;

use spec_spine_core::{selected_content, selected_content_json};
use spec_spine_types::{
    Config, ContentCompleteness, ContentDirtyState, ContentOmissionReason, ContentProjection,
    ContentRequest, ContentSelector, ContentSnapshot, ContentSnapshotBinding, RepoPath,
};

fn snapshot() -> ContentSnapshot {
    ContentSnapshot {
        repository: "example/repo".into(),
        revision: "1".repeat(40),
        tree: "2".repeat(40),
        dirty_state: ContentDirtyState::CleanExport,
        binding: ContentSnapshotBinding::CallerSupplied,
    }
}

fn file(path: &str) -> ContentSelector {
    ContentSelector::File {
        path: RepoPath::parse(path).unwrap(),
        projection: None,
        required: true,
    }
}

#[test]
fn normalizes_orders_deduplicates_and_pages_between_items() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("a.txt"), "a\r\nb\r").unwrap();
    fs::write(tmp.path().join("z.txt"), "z\n").unwrap();
    let request = ContentRequest {
        selectors: vec![file("z.txt"), file("a.txt"), file("a.txt")],
        default_projection: ContentProjection::Full,
        max_bytes: 10,
        max_items: 1,
        continuation: None,
    };
    let first = selected_content(&Config::default(), tmp.path(), &request, &snapshot()).unwrap();
    assert_eq!(first.completeness, ContentCompleteness::Partial);
    assert_eq!(first.items[0].identity, "file:a.txt");
    assert_eq!(first.items[0].content, "a\nb\n");
    assert_eq!(first.coalesced_selectors[0].positions, vec![1, 2]);
    assert!(first.items[0].digest.starts_with("sha256:"));

    let mut next = request;
    next.continuation = first.continuation;
    let second = selected_content(&Config::default(), tmp.path(), &next, &snapshot()).unwrap();
    assert_eq!(second.items[0].identity, "file:z.txt");
    assert_eq!(second.completeness, ContentCompleteness::Complete);
}

#[test]
fn canonical_paths_coalesce_and_directory_members_use_the_same_spelling() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir(tmp.path().join("dir")).unwrap();
    fs::write(tmp.path().join("dir/item.txt"), "item\n").unwrap();
    let request = ContentRequest {
        selectors: vec![
            file("./dir//item.txt"),
            file("dir/item.txt"),
            ContentSelector::DirectoryMember {
                directory: RepoPath::parse("./dir/").unwrap(),
                member: RepoPath::parse("./item.txt").unwrap(),
                projection: None,
                required: true,
            },
        ],
        default_projection: ContentProjection::Full,
        max_bytes: 100,
        max_items: 10,
        continuation: None,
    };
    let response = selected_content(&Config::default(), tmp.path(), &request, &snapshot()).unwrap();
    assert_eq!(response.items.len(), 2);
    assert_eq!(response.items[0].identity, "directory-member:dir#item.txt");
    assert_eq!(response.items[1].identity, "file:dir/item.txt");
    assert_eq!(response.coalesced_selectors[0].positions, vec![0, 1]);
}

#[test]
fn unsupported_binary_and_oversized_selections_are_explicit() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("binary"), [0xff, 0xfe]).unwrap();
    fs::write(tmp.path().join("large"), "abcdef").unwrap();
    let request = serde_json::json!({
        "selectors": [
            {"kind":"file", "path":"missing"},
            {"kind":"file", "path":"binary"},
            {"kind":"file", "path":"large"},
            {"kind":"file", "path":"large", "projection":"body"}
        ],
        "maxBytes": 3
    });
    let json = selected_content_json(
        "{}",
        tmp.path().to_str().unwrap(),
        &request.to_string(),
        &serde_json::to_string(&snapshot()).unwrap(),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["schemaVersion"], "0.9.0");
    let reasons: Vec<&str> = value["omissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["reason"].as_str())
        .collect();
    assert!(reasons.contains(&"missing-content"));
    assert!(reasons.contains(&"binary-content"));
    assert!(reasons.contains(&"item-exceeds-byte-budget"));
    assert!(reasons.contains(&"unsupported-projection"));
    assert_eq!(value["completeness"], "incomplete");
}

#[test]
fn a_whole_file_that_cannot_normalize_under_the_budget_is_omitted_before_reading() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("large"), "0123456789").unwrap();
    let request = ContentRequest {
        selectors: vec![file("large")],
        default_projection: ContentProjection::Full,
        max_bytes: 3,
        max_items: 1,
        continuation: None,
    };
    let response = selected_content(&Config::default(), tmp.path(), &request, &snapshot()).unwrap();
    assert!(response.items.is_empty());
    assert_eq!(
        response.omissions[0].reason,
        ContentOmissionReason::ItemExceedsByteBudget
    );
    assert!(response.omissions[0].message.contains("cannot fit"));
}

#[test]
fn private_components_are_excluded_at_every_depth() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("nested/.git")).unwrap();
    fs::write(tmp.path().join("nested/.git/config"), "secret\n").unwrap();
    let request = ContentRequest {
        selectors: vec![file("nested/.git/config")],
        default_projection: ContentProjection::Full,
        max_bytes: 100,
        max_items: 1,
        continuation: None,
    };
    let response = selected_content(&Config::default(), tmp.path(), &request, &snapshot()).unwrap();
    assert!(response.items.is_empty());
    assert!(response.omissions[0].message.contains("excluded"));
}

#[test]
fn an_omission_remains_incomplete_when_a_budget_also_creates_a_continuation() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("a"), "a").unwrap();
    fs::write(tmp.path().join("b"), "b").unwrap();
    let request = ContentRequest {
        selectors: vec![file("missing"), file("a"), file("b")],
        default_projection: ContentProjection::Full,
        max_bytes: 2,
        max_items: 1,
        continuation: None,
    };
    let response = selected_content(&Config::default(), tmp.path(), &request, &snapshot()).unwrap();
    assert_eq!(response.completeness, ContentCompleteness::Incomplete);
    assert!(response.continuation.is_some());
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.omissions.len(), 1);
}

#[test]
fn invalid_bounds_and_stale_continuation_are_refused() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("a"), "a").unwrap();
    let mut request = ContentRequest {
        selectors: vec![file("a")],
        default_projection: ContentProjection::Full,
        max_bytes: 0,
        max_items: 1,
        continuation: None,
    };
    assert_eq!(
        selected_content(&Config::default(), tmp.path(), &request, &snapshot())
            .unwrap_err()
            .exit_code(),
        3
    );
    request.max_bytes = 1;
    request.continuation = Some(spec_spine_types::ContentContinuation {
        request_digest: "sha256:bad".into(),
        snapshot_digest: "sha256:bad".into(),
        next_identity: "file:a".into(),
        next_ordinal: 0,
    });
    assert_eq!(
        selected_content(&Config::default(), tmp.path(), &request, &snapshot())
            .unwrap_err()
            .exit_code(),
        2
    );
}

#[test]
fn line_endings_produce_identical_content_spans_and_digests() {
    let tmp = tempfile::tempdir().unwrap();
    let request = ContentRequest {
        selectors: vec![file("a.txt")],
        default_projection: ContentProjection::Full,
        max_bytes: 100,
        max_items: 1,
        continuation: None,
    };
    let mut answers = Vec::new();
    for content in ["one\ntwo\n", "one\r\ntwo\r\n", "one\rtwo\r"] {
        fs::write(tmp.path().join("a.txt"), content).unwrap();
        let response =
            selected_content(&Config::default(), tmp.path(), &request, &snapshot()).unwrap();
        answers.push((
            response.items[0].content.clone(),
            response.items[0].span,
            response.items[0].digest.clone(),
        ));
    }
    assert_eq!(answers[0], answers[1]);
    assert_eq!(answers[1], answers[2]);
}

#[cfg(unix)]
#[test]
fn directory_member_symlink_leaving_the_named_directory_is_refused() {
    use std::os::unix::fs::symlink;

    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir(tmp.path().join("docs")).unwrap();
    fs::write(tmp.path().join("outside.txt"), "outside\n").unwrap();
    symlink("../outside.txt", tmp.path().join("docs/out.txt")).unwrap();
    let request = ContentRequest {
        selectors: vec![ContentSelector::DirectoryMember {
            directory: RepoPath::parse("docs").unwrap(),
            member: RepoPath::parse("out.txt").unwrap(),
            projection: None,
            required: true,
        }],
        default_projection: ContentProjection::Full,
        max_bytes: 100,
        max_items: 1,
        continuation: None,
    };
    assert_eq!(
        selected_content(&Config::default(), tmp.path(), &request, &snapshot())
            .unwrap_err()
            .exit_code(),
        2
    );
}

#[test]
fn unsupported_test_selector_is_an_explicit_omission() {
    let tmp = tempfile::tempdir().unwrap();
    let request: ContentRequest = serde_json::from_value(serde_json::json!({
        "selectors": [{"kind":"test", "id":"crate::tests::works"}]
    }))
    .unwrap();
    let response = selected_content(&Config::default(), tmp.path(), &request, &snapshot()).unwrap();
    assert_eq!(
        response.omissions[0].reason,
        ContentOmissionReason::UnsupportedSelector
    );
}
