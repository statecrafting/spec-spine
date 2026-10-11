//! Spec 159: one repository context packet, assembled in core.

use std::fs;
use std::path::Path;

use spec_spine_core::{
    context_packet, context_packet_document, context_packet_json, packet_digest, shard,
};
use spec_spine_types::{
    CONTEXT_PACKET_SCHEMA, Config, ContentCompleteness, ContentDirtyState, ContentProjection,
    ContentSelector, ContentSnapshot, ContentSnapshotBinding, ContextPacket, Error, PacketClosure,
    PacketMemberRequest, PacketOmissionReason, PacketOrigin, PacketRequest, PacketRequirement,
    PacketWarningCode, RepoPath,
};

const BODY: &str = "# 001\n\n## 3. Behavior\n\n### 3.1 The rule\n\nThe rule text.\n\n### 3.2 Retired\n\nOld text.\n";
const OBLIGATIONS: &str = "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n  - id: \"R-2\"\n    kind: requirement\n    text: \"Retired.\"\n    anchor: \"3-2-retired\"\n    withdrawn: true\n";
const BUILD: &str = "sha256:abababababababababababababababababababababababababababababababab";

fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn spec(root: &Path, id: &str, extra: &str, body: &str) {
    write(
        root,
        &format!("specs/{id}/spec.md"),
        &format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-10-10\"\n\
             summary: \"s\"\n{extra}---\n{body}"
        ),
    );
}

/// Commit the registry shards, as `spec-spine compile` writes them.
fn commit_ledger(root: &Path) {
    let cfg = Config::default();
    let compiled = spec_spine_core::compile(&cfg, root).unwrap();
    assert!(
        compiled.validation_passed,
        "{:?}",
        compiled.registry.validation
    );
    let dir = spec_spine_core::registry_dir(&cfg, root).join(shard::BY_SPEC_DIR);
    shard::sync_dir(
        &dir,
        &spec_spine_core::registry_shard_files(&compiled.shards).unwrap(),
    )
    .unwrap();
}

/// Two specs, two plain files, and a committed ledger.
fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    spec(tmp.path(), "001-a", OBLIGATIONS, BODY);
    spec(tmp.path(), "002-b", "", "# 002\n\nWhy.\n");
    write(tmp.path(), "notes/a.txt", "alpha\n");
    write(tmp.path(), "notes/z.txt", "zulu\n");
    commit_ledger(tmp.path());
    tmp
}

fn snapshot() -> ContentSnapshot {
    ContentSnapshot {
        repository: "example/repo".into(),
        revision: "1".repeat(40),
        tree: "2".repeat(40),
        dirty_state: ContentDirtyState::CleanExport,
        binding: ContentSnapshotBinding::CallerSupplied,
    }
}

fn file(path: &str, requirement: PacketRequirement) -> PacketMemberRequest {
    PacketMemberRequest {
        selector: ContentSelector::File {
            path: RepoPath::parse(path).unwrap(),
            projection: None,
            required: true,
        },
        requirement,
    }
}

fn request(root: &str, members: Vec<PacketMemberRequest>) -> PacketRequest {
    PacketRequest {
        closure: PacketClosure {
            root: Some(root.into()),
            digest: None,
            document: None,
        },
        members,
        max_bytes: 524_288,
        max_items: 96,
        continuation: None,
        rationale: None,
        work_identity: None,
        consumer_schema_version: "1.0.0".into(),
    }
}

fn assemble(root: &Path, req: &PacketRequest) -> Result<ContextPacket, Error> {
    context_packet(&Config::default(), root, req, &snapshot(), Some(BUILD))
}

fn keys(packet: &ContextPacket) -> Vec<String> {
    packet.members.iter().map(|m| m.identity.clone()).collect()
}

// ---- 3.1 - 3.5: identity, members and deduplication --------------------------

#[test]
fn a_root_closure_and_additional_selectors_give_the_exact_ordered_members() {
    let tmp = corpus();
    let packet = assemble(
        tmp.path(),
        &request(
            "002",
            vec![
                file("notes/z.txt", PacketRequirement::Optional),
                file("notes/a.txt", PacketRequirement::Required),
            ],
        ),
    )
    .unwrap();
    assert_eq!(
        keys(&packet),
        vec!["file:notes/a.txt", "file:notes/z.txt", "spec:002-b"]
    );
    assert_eq!(packet.completeness, ContentCompleteness::Complete);
    assert!(packet.omissions.is_empty());
    assert!(packet.warnings.is_empty(), "{:?}", packet.warnings);
    let spec = &packet.members[2];
    assert_eq!(spec.origins, vec![PacketOrigin::Closure]);
    assert_eq!(spec.requirement, PacketRequirement::Required);
    assert_eq!(packet.members[1].origins, vec![PacketOrigin::Additional]);
    assert_eq!(packet.members[1].requirement, PacketRequirement::Optional);
    // Every member repeats the snapshot identity, as the packet does.
    for m in &packet.members {
        assert_eq!(m.item.repository, "example/repo");
        assert_eq!(m.item.revision, packet.snapshot.revision);
        assert_eq!(m.item.tree, packet.snapshot.tree);
        assert_eq!(m.item.identity, m.identity);
    }
    assert_eq!(packet.closure_digest, packet.closure["digest"]);
    assert_eq!(packet.producer.build, BUILD);
    assert_eq!(packet.producer.package, "spec-spine");
    assert!(packet.request_digest.starts_with("sha256:"));
    assert_eq!(packet.packet_digest, packet_digest(&packet).unwrap());
}

#[test]
fn duplicates_collapse_with_required_winning_and_origins_kept() {
    let tmp = corpus();
    let spec_again = |requirement| PacketMemberRequest {
        selector: ContentSelector::Spec {
            spec: "002-b".into(),
            projection: None,
            required: true,
        },
        requirement,
    };
    let packet = assemble(
        tmp.path(),
        &request(
            "002",
            vec![
                spec_again(PacketRequirement::Optional),
                file("notes/a.txt", PacketRequirement::Optional),
                file("./notes//a.txt", PacketRequirement::Required),
            ],
        ),
    )
    .unwrap();
    assert_eq!(keys(&packet), vec!["file:notes/a.txt", "spec:002-b"]);
    assert_eq!(packet.members[0].requirement, PacketRequirement::Required);
    assert_eq!(packet.members[0].origins, vec![PacketOrigin::Additional]);
    assert_eq!(packet.members[1].requirement, PacketRequirement::Required);
    assert_eq!(
        packet.members[1].origins,
        vec![PacketOrigin::Closure, PacketOrigin::Additional]
    );
}

#[test]
fn equal_bytes_under_distinct_identities_stay_distinct() {
    let tmp = corpus();
    write(tmp.path(), "notes/copy.txt", "alpha\n");
    let packet = assemble(
        tmp.path(),
        &request(
            "002",
            vec![
                file("notes/a.txt", PacketRequirement::Required),
                file("notes/copy.txt", PacketRequirement::Required),
            ],
        ),
    )
    .unwrap();
    assert_eq!(packet.members.len(), 3);
    assert_eq!(
        packet.members[0].item.content,
        packet.members[1].item.content
    );
}

// ---- 3.6 - 3.9: omissions, warnings and completeness -------------------------

#[test]
fn a_missing_required_member_is_incomplete_and_never_disappears() {
    let tmp = corpus();
    let packet = assemble(
        tmp.path(),
        &request(
            "002",
            vec![file("notes/gone.txt", PacketRequirement::Required)],
        ),
    )
    .unwrap();
    assert_eq!(packet.completeness, ContentCompleteness::Incomplete);
    assert_eq!(packet.omissions.len(), 1);
    let o = &packet.omissions[0];
    assert_eq!(o.identity, "file:notes/gone.txt");
    assert_eq!(o.reason, PacketOmissionReason::Missing);
    assert_eq!(o.requirement, PacketRequirement::Required);
    assert_eq!(keys(&packet), vec!["spec:002-b"]);
}

#[test]
fn an_optional_omission_is_recorded_and_the_packet_stays_complete() {
    let tmp = corpus();
    fs::write(tmp.path().join("notes/blob.bin"), [0xff, 0xfe, 0x00, 0x01]).unwrap();
    let packet = assemble(
        tmp.path(),
        &request(
            "002",
            vec![
                file("notes/gone.txt", PacketRequirement::Optional),
                file("notes/blob.bin", PacketRequirement::Optional),
                PacketMemberRequest {
                    selector: ContentSelector::File {
                        path: RepoPath::parse("notes/a.txt").unwrap(),
                        projection: Some(ContentProjection::Signature),
                        required: true,
                    },
                    requirement: PacketRequirement::Optional,
                },
            ],
        ),
    )
    .unwrap();
    assert_eq!(packet.completeness, ContentCompleteness::Complete);
    let reasons: Vec<_> = packet.omissions.iter().map(|o| o.reason).collect();
    assert_eq!(
        reasons,
        vec![
            PacketOmissionReason::UnsupportedProjection,
            PacketOmissionReason::NonText,
            PacketOmissionReason::Missing,
        ]
    );
    let codes: Vec<_> = packet.warnings.iter().map(|w| w.code).collect();
    assert_eq!(codes, vec![PacketWarningCode::OptionalMemberOmitted; 3]);
}

#[test]
fn an_oversized_member_is_omitted_rather_than_continued_forever() {
    let tmp = corpus();
    write(tmp.path(), "notes/big.txt", &"x".repeat(64));
    let mut req = request(
        "002",
        vec![file("notes/big.txt", PacketRequirement::Required)],
    );
    req.max_bytes = 32;
    let packet = assemble(tmp.path(), &req).unwrap();
    assert_eq!(packet.completeness, ContentCompleteness::Incomplete);
    assert_eq!(packet.continuation, None, "no page could ever hold it");
    let big = packet
        .omissions
        .iter()
        .find(|o| o.identity == "file:notes/big.txt")
        .unwrap();
    assert_eq!(big.reason, PacketOmissionReason::OversizedMember);
    assert!(!big.detail.contains("xxxx"), "detail never echoes content");
}

#[test]
fn a_withdrawn_closure_obligation_is_an_explicit_omission_with_a_warning() {
    let tmp = corpus();
    let cfg = Config::default();
    let closure = spec_spine_core::closure(
        &cfg,
        tmp.path(),
        &spec_spine_core::ClosureRequest {
            obligations: vec!["001#R-1".into(), "001#R-2".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let document = serde_json::to_value(&closure).unwrap();
    let mut req = request("002", vec![]);
    req.closure = PacketClosure {
        root: None,
        digest: Some(closure.digest.clone()),
        document: Some(document),
    };
    let packet = assemble(tmp.path(), &req).unwrap();
    assert_eq!(keys(&packet), vec!["obligation:001-a#R-1"]);
    assert_eq!(packet.omissions.len(), 1);
    assert_eq!(packet.omissions[0].identity, "obligation:001-a#R-2");
    assert_eq!(packet.omissions[0].reason, PacketOmissionReason::Withdrawn);
    assert_eq!(packet.completeness, ContentCompleteness::Incomplete);
    assert_eq!(
        packet.warnings[0].code,
        PacketWarningCode::ClosureMemberUnsupported
    );
    assert_eq!(packet.closure_digest, closure.digest);
}

#[test]
fn an_unverified_build_is_recorded_and_warned_never_omitted() {
    let tmp = corpus();
    let packet = context_packet(
        &Config::default(),
        tmp.path(),
        &request("002", vec![]),
        &snapshot(),
        None,
    )
    .unwrap();
    assert_eq!(packet.producer.build, "unverified");
    assert_eq!(
        packet.warnings[0].code,
        PacketWarningCode::UnverifiedProducerBuild
    );
    assert_eq!(packet.completeness, ContentCompleteness::Complete);
}

#[test]
fn a_missing_root_is_a_finding_and_a_stale_ledger_is_refused_not_an_empty_packet() {
    let tmp = corpus();
    let missing = assemble(tmp.path(), &request("009", vec![]));
    assert!(matches!(missing, Err(Error::NotFound(_))), "{missing:?}");

    spec(tmp.path(), "002-b", "", "# 002\n\nChanged.\n");
    let stale = assemble(tmp.path(), &request("002", vec![]));
    assert!(matches!(stale, Err(Error::Stale { .. })), "{stale:?}");
}

// ---- 3.10, 3.11: ordering, budgets, continuation and digests -----------------

#[test]
fn pages_continue_without_repeating_or_skipping_a_member() {
    let tmp = corpus();
    let mut req = request(
        "002",
        vec![
            file("notes/a.txt", PacketRequirement::Required),
            file("notes/z.txt", PacketRequirement::Required),
        ],
    );
    req.max_items = 1;
    let mut seen = Vec::new();
    let mut digests = Vec::new();
    loop {
        let page = assemble(tmp.path(), &req).unwrap();
        assert_eq!(page.members.len(), 1);
        seen.extend(keys(&page));
        digests.push(page.packet_digest.clone());
        match page.continuation {
            Some(token) => {
                assert_eq!(page.completeness, ContentCompleteness::Partial);
                req.continuation = Some(token);
            }
            None => {
                assert_eq!(page.completeness, ContentCompleteness::Complete);
                break;
            }
        }
    }
    assert_eq!(
        seen,
        vec!["file:notes/a.txt", "file:notes/z.txt", "spec:002-b"]
    );
    digests.sort();
    digests.dedup();
    assert_eq!(digests.len(), 3, "every page has its own digest");
}

#[test]
fn a_byte_budget_ends_a_page_between_members() {
    let tmp = corpus();
    write(tmp.path(), "notes/m.txt", "mike\n");
    let mut req = request(
        "002",
        vec![
            file("notes/a.txt", PacketRequirement::Required),
            file("notes/m.txt", PacketRequirement::Required),
            file("notes/z.txt", PacketRequirement::Required),
        ],
    );
    // "alpha\n" is 6 bytes and "mike\n" 5: both fit, "zulu\n" would not. The
    // spec itself is larger than the whole budget, so it is oversized.
    req.max_bytes = 11;
    let page = assemble(tmp.path(), &req).unwrap();
    assert_eq!(keys(&page), vec!["file:notes/a.txt", "file:notes/m.txt"]);
    assert!(page.continuation.is_some());
    let mut next = req.clone();
    next.continuation = page.continuation.clone();
    let rest = assemble(tmp.path(), &next).unwrap();
    assert_eq!(keys(&rest), vec!["file:notes/z.txt"]);
    assert_eq!(rest.continuation, None);
    assert_eq!(
        rest.omissions, page.omissions,
        "omissions are page-independent"
    );
}

#[test]
fn reads_are_byte_identical_and_invariant_to_member_order() {
    let tmp = corpus();
    let a = request(
        "002",
        vec![
            file("notes/a.txt", PacketRequirement::Required),
            file("notes/z.txt", PacketRequirement::Optional),
        ],
    );
    let mut b = a.clone();
    b.members.reverse();
    let first = context_packet_document(&assemble(tmp.path(), &a).unwrap()).unwrap();
    let again = context_packet_document(&assemble(tmp.path(), &a).unwrap()).unwrap();
    let reversed = context_packet_document(&assemble(tmp.path(), &b).unwrap()).unwrap();
    assert_eq!(first, again);
    assert_eq!(first, reversed);
    assert!(first.ends_with("}\n") && !first.contains('\r'));
}

#[test]
fn line_endings_give_equal_members_and_packet_digests() {
    let mut digests = Vec::new();
    for text in ["one\ntwo\n", "one\r\ntwo\r\n", "one\rtwo\r"] {
        let tmp = corpus();
        write(tmp.path(), "notes/eol.txt", text);
        let packet = assemble(
            tmp.path(),
            &request(
                "002",
                vec![file("notes/eol.txt", PacketRequirement::Required)],
            ),
        )
        .unwrap();
        assert_eq!(packet.members[0].item.content, "one\ntwo\n");
        digests.push((packet.members[0].item.digest.clone(), packet.packet_digest));
    }
    assert_eq!(digests[0], digests[1]);
    assert_eq!(digests[0], digests[2]);
}

// ---- 3.12: refusals ----------------------------------------------------------

#[test]
fn a_continuation_bound_to_other_inputs_is_stale_and_a_forged_one_is_malformed() {
    let tmp = corpus();
    let mut req = request(
        "002",
        vec![file("notes/a.txt", PacketRequirement::Required)],
    );
    req.max_items = 1;
    let token = assemble(tmp.path(), &req).unwrap().continuation.unwrap();

    let mut other_budget = req.clone();
    other_budget.max_items = 2;
    other_budget.continuation = Some(token.clone());
    let stale = assemble(tmp.path(), &other_budget);
    assert!(
        matches!(&stale, Err(Error::Refused(m)) if m.starts_with("stale-continuation")),
        "{stale:?}"
    );

    let mut other_snapshot = req.clone();
    other_snapshot.continuation = Some(token.clone());
    let mut moved = snapshot();
    moved.revision = "3".repeat(40);
    let stale = context_packet(
        &Config::default(),
        tmp.path(),
        &other_snapshot,
        &moved,
        Some(BUILD),
    );
    assert!(matches!(stale, Err(Error::Refused(_))), "{stale:?}");

    let mut forged = req.clone();
    let mut bytes = token.into_bytes();
    let last = bytes.len() - 3;
    bytes[last] = if bytes[last] == b'A' { b'B' } else { b'A' };
    forged.continuation = Some(String::from_utf8(bytes).unwrap());
    let refused = assemble(tmp.path(), &forged);
    assert!(matches!(refused, Err(Error::Usage(_))), "{refused:?}");
}

#[test]
fn malformed_requests_are_usage_and_unsupported_schemas_are_refused() {
    let tmp = corpus();
    let usage = |req: PacketRequest| {
        let r = assemble(tmp.path(), &req);
        assert!(matches!(r, Err(Error::Usage(_))), "{r:?}");
    };
    let mut both = request("002", vec![]);
    both.closure.digest = Some("0".repeat(64));
    usage(both);
    let mut digest_alone = request("002", vec![]);
    digest_alone.closure = PacketClosure {
        root: None,
        digest: Some("0".repeat(64)),
        document: None,
    };
    usage(digest_alone);
    for (bytes, items) in [(0, 1), (4_194_305, 1), (1, 0), (1, 513)] {
        let mut r = request("002", vec![]);
        r.max_bytes = bytes;
        r.max_items = items;
        usage(r);
    }
    let mut long = request("002", vec![]);
    long.rationale = Some("r".repeat(1025));
    usage(long);
    let mut not_required = request("002", vec![]);
    not_required.members.push(PacketMemberRequest {
        selector: ContentSelector::File {
            path: RepoPath::parse("notes/a.txt").unwrap(),
            projection: None,
            required: false,
        },
        requirement: PacketRequirement::Optional,
    });
    usage(not_required);
    let bad_build = context_packet(
        &Config::default(),
        tmp.path(),
        &request("002", vec![]),
        &snapshot(),
        Some("sha256:XYZ"),
    );
    assert!(matches!(bad_build, Err(Error::Usage(_))), "{bad_build:?}");

    for version in ["2.0.0", "1.1.0"] {
        let mut r = request("002", vec![]);
        r.consumer_schema_version = version.into();
        let refused = assemble(tmp.path(), &r);
        assert!(
            matches!(refused, Err(Error::Refused(_))),
            "{version}: {refused:?}"
        );
    }
    let mut unparsed = request("002", vec![]);
    unparsed.consumer_schema_version = "one".into();
    usage(unparsed);
}

#[test]
fn a_closure_document_that_does_not_recompute_is_refused() {
    let tmp = corpus();
    let cfg = Config::default();
    let closure = spec_spine_core::closure(
        &cfg,
        tmp.path(),
        &spec_spine_core::ClosureRequest {
            specs: vec!["002".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let mut req = request("002", vec![]);
    req.closure = PacketClosure {
        root: None,
        digest: Some("f".repeat(64)),
        document: Some(serde_json::to_value(&closure).unwrap()),
    };
    let refused = assemble(tmp.path(), &req);
    assert!(
        matches!(&refused, Err(Error::Refused(m)) if m.starts_with("closure-mismatch")),
        "{refused:?}"
    );

    req.closure.digest = Some(closure.digest.clone());
    assert!(
        assemble(tmp.path(), &req).is_ok(),
        "the true digest is accepted"
    );

    let mut edited = serde_json::to_value(&closure).unwrap();
    edited["members"][0]["contentHash"] = serde_json::json!("0".repeat(64));
    req.closure.document = Some(edited);
    assert!(matches!(assemble(tmp.path(), &req), Err(Error::Refused(_))));
}

#[test]
fn a_path_escape_is_refused_not_omitted() {
    let req: Result<PacketRequest, _> = serde_json::from_str(
        r#"{"closure":{"root":"002"},"members":[{"selector":{"kind":"file","path":"../x"},"requirement":"optional"}],"consumerSchemaVersion":"1.0.0"}"#,
    );
    // RepoPath refuses the escape at parse, before any read.
    assert!(req.is_err());
}

// ---- 3.11: schema and facade -------------------------------------------------

#[test]
fn emitted_packets_conform_and_the_facade_writes_the_same_bytes() {
    let tmp = corpus();
    let schema: serde_json::Value = serde_json::from_str(CONTEXT_PACKET_SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let mut partial = request(
        "002",
        vec![
            file("notes/a.txt", PacketRequirement::Required),
            file("notes/gone.txt", PacketRequirement::Optional),
        ],
    );
    partial.max_items = 1;
    let incomplete = request(
        "002",
        vec![file("notes/gone.txt", PacketRequirement::Required)],
    );
    for req in [request("002", vec![]), partial, incomplete] {
        let typed = context_packet(&Config::default(), tmp.path(), &req, &snapshot(), None)
            .map(|p| context_packet_document(&p).unwrap())
            .unwrap();
        let facade = context_packet_json(
            &serde_json::to_string(&Config::default()).unwrap(),
            tmp.path().to_str().unwrap(),
            &serde_json::to_string(&req).unwrap(),
            &serde_json::to_string(&snapshot()).unwrap(),
        )
        .unwrap();
        assert_eq!(typed, facade);
        let instance: serde_json::Value = serde_json::from_str(&typed).unwrap();
        let errors: Vec<String> = validator
            .iter_errors(&instance)
            .map(|e| e.to_string())
            .collect();
        assert!(errors.is_empty(), "{}", errors.join("\n"));
        assert_eq!(instance["schemaVersion"], "1.0.0");
    }
    let drifted = serde_json::json!({ "schemaVersion": "1.0.0", "members": [] });
    assert!(!validator.is_valid(&drifted));
}

#[test]
fn unknown_request_members_are_refused_at_the_facade() {
    let tmp = corpus();
    let out = context_packet_json(
        &serde_json::to_string(&Config::default()).unwrap(),
        tmp.path().to_str().unwrap(),
        r#"{"closure":{"root":"002"},"consumerSchemaVersion":"1.0.0","extra":1}"#,
        &serde_json::to_string(&snapshot()).unwrap(),
    );
    assert!(matches!(out, Err(Error::Usage(_))), "{out:?}");
}

#[test]
fn the_documented_request_shape_parses_and_assembles() {
    let tmp = corpus();
    let text = include_str!("fixtures/context-packet/request.json");
    let req: PacketRequest = serde_json::from_str(text).unwrap();
    let packet = assemble(tmp.path(), &req).unwrap();
    assert_eq!(
        keys(&packet),
        vec![
            "file:notes/z.txt",
            "section:001-a#3-1-the-rule",
            "spec:002-b"
        ]
    );
    assert_eq!(packet.completeness, ContentCompleteness::Complete);
    assert_eq!(
        packet.request.rationale.as_deref(),
        Some("fixture: the documented request shape")
    );
}
