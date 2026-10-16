//! Spec 164: a packet delta reproduces the packet it names, in core.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use spec_spine_core::{
    context_packet_apply, context_packet_delta, context_packet_delta_document,
    context_packet_document, context_packet_pages, manifest_digest, packet_digest, shard,
};
use spec_spine_types::{
    CONTEXT_PACKET_DELTA_SCHEMA, ChangeBasis, ChangedField, Config, ContentDirtyState,
    ContentProjection, ContentSelector, ContentSnapshot, ContentSnapshotBinding,
    ContextDeltaRequest, ContextDeltaResult, ContextPacket, ContextPacketDelta,
    DeltaFallbackReason, DeltaMemberState, DocumentationManifest, Error, ManifestGenerator,
    ManifestInput, ManifestOutput, ManifestPacket, ManifestRegion, MemberChange, PacketClosure,
    PacketMemberRequest, PacketOmissionReason, PacketRequest, PacketRequirement, RegionKind,
    RegionSpan, RepoPath,
};

const BODY: &str = "# 001\n\n## 3. Behavior\n\n### 3.1 The rule\n\nThe rule text.\n\n### 3.2 Other\n\nOther text.\n";
const OBLIGATIONS: &str = "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n";
// Four visible 16-digit chunks: 64 hex digits.
const BUILD: &str = concat!(
    "sha256:",
    "abababababababab",
    "abababababababab",
    "abababababababab",
    "abababababababab"
);
const OTHER_BUILD: &str = concat!(
    "sha256:",
    "cdcdcdcdcdcdcdcd",
    "cdcdcdcdcdcdcdcd",
    "cdcdcdcdcdcdcdcd",
    "cdcdcdcdcdcdcdcd"
);

/// `BUILD` and `OTHER_BUILD` are well-formed: `sha256:` and 64 lowercase
/// hex digits, as `context_packet` requires of a producer build.
#[test]
fn the_fixture_build_digests_are_well_formed() {
    for build in [BUILD, OTHER_BUILD] {
        let hex = build.strip_prefix("sha256:").unwrap();
        assert_eq!(hex.len(), 64, "{build}");
        assert!(
            hex.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        );
    }
}

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
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-10-15\"\n\
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

/// Two specs and a few plain files, with a committed ledger.
fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    spec(tmp.path(), "001-a", OBLIGATIONS, BODY);
    spec(tmp.path(), "002-b", "", "# 002\n\nWhy.\n");
    for (name, text) in [
        ("a", "alpha\n"),
        ("b", "bravo\n"),
        ("c", "charlie\n"),
        ("z", "zulu\n"),
    ] {
        write(tmp.path(), &format!("notes/{name}.txt"), text);
    }
    commit_ledger(tmp.path());
    tmp
}

fn snapshot(tree: char) -> ContentSnapshot {
    ContentSnapshot {
        repository: "example/repo".into(),
        revision: tree.to_string().repeat(40),
        tree: tree.to_string().repeat(40),
        dirty_state: ContentDirtyState::CleanExport,
        binding: ContentSnapshotBinding::CallerSupplied,
    }
}

fn selector(selector: ContentSelector, requirement: PacketRequirement) -> PacketMemberRequest {
    PacketMemberRequest {
        selector,
        requirement,
    }
}

fn file(path: &str, requirement: PacketRequirement) -> PacketMemberRequest {
    selector(
        ContentSelector::File {
            path: RepoPath::parse(path).unwrap(),
            projection: None,
            required: true,
        },
        requirement,
    )
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
        consumer_schema_version: "1.1.0".into(),
    }
}

fn notes(names: &[&str]) -> Vec<PacketMemberRequest> {
    names
        .iter()
        .map(|n| file(&format!("notes/{n}.txt"), PacketRequirement::Optional))
        .collect()
}

/// The complete page set spec 159 assembles, as canonical documents.
fn pages_at(root: &Path, req: &PacketRequest, tree: char, build: &str) -> Vec<String> {
    context_packet_pages(&Config::default(), root, req, &snapshot(tree), Some(build))
        .unwrap()
        .iter()
        .map(|p| context_packet_document(p).unwrap())
        .collect()
}

fn delta_request(req: &PacketRequest, predecessor: Vec<String>) -> ContextDeltaRequest {
    ContextDeltaRequest {
        request: req.clone(),
        predecessor_pages: predecessor,
        manifests: Vec::new(),
        consumer_schema_version: "1.0.0".into(),
    }
}

fn compute(
    root: &Path,
    dreq: &ContextDeltaRequest,
    tree: char,
    build: &str,
) -> Result<ContextDeltaResult, Error> {
    context_packet_delta(&Config::default(), root, dreq, &snapshot(tree), Some(build))
}

fn delta(result: ContextDeltaResult) -> ContextPacketDelta {
    match result {
        ContextDeltaResult::Delta(d) => *d,
        ContextDeltaResult::Fallback(f) => panic!("expected a delta, got fallback {:?}", f.reason),
    }
}

fn fallback_reason(result: ContextDeltaResult) -> DeltaFallbackReason {
    match result {
        ContextDeltaResult::Fallback(f) => {
            assert!(f.invalidation_candidates.items.is_empty() || !f.request_digest.is_empty());
            f.reason
        }
        ContextDeltaResult::Delta(_) => panic!("expected a fallback"),
    }
}

fn changes(d: &ContextPacketDelta) -> Vec<(String, MemberChange)> {
    d.changes
        .iter()
        .map(|e| (e.key.identity.clone(), e.change))
        .collect()
}

/// Compute a delta from `before` to the tree as it stands, check that it
/// reproduces the target, and return it.
fn reproduce(
    root: &Path,
    req: &PacketRequest,
    before: &[String],
    tree: char,
) -> ContextPacketDelta {
    let d = delta(compute(root, &delta_request(req, before.to_vec()), tree, BUILD).unwrap());
    let applied = context_packet_apply(before, &d).unwrap();
    assert_eq!(
        applied,
        pages_at(root, req, tree, BUILD),
        "the reproduction law"
    );
    d
}

// ---- 3.4: the transition table ----------------------------------------------

#[test]
fn an_unchanged_set_yields_no_entry_and_reproduces() {
    let tmp = corpus();
    let req = request("002", notes(&["a", "b"]));
    let before = pages_at(tmp.path(), &req, '1', BUILD);
    let d = reproduce(tmp.path(), &req, &before, '2');
    assert!(d.changes.is_empty(), "{:?}", changes(&d));
    assert!(!d.producer_changed);
    assert_eq!(d.target.closure, None, "an equal closure is reused");
    assert_eq!(d.predecessor.snapshot.tree, "1".repeat(40));
    assert_eq!(d.target.snapshot.tree, "2".repeat(40));
}

#[test]
fn every_row_of_the_table_yields_exactly_its_entry() {
    let tmp = corpus();
    let root = tmp.path();
    // Before: a, b, c present; gone.txt omitted (missing); z present.
    let before_req = request(
        "002",
        [notes(&["a", "b", "c", "z"]), notes(&["gone"])].concat(),
    );
    let before = pages_at(root, &before_req, '1', BUILD);

    // After: a changed; b removed from the tree (now missing: newly
    // unresolved); c unchanged; gone.txt now exists (newly available); z
    // dropped from the request is not possible under one request, so the
    // request is held fixed and z becomes non-text (newly omitted).
    write(root, "notes/a.txt", "alpha, changed\n");
    fs::remove_file(root.join("notes/b.txt")).unwrap();
    write(root, "notes/gone.txt", "back\n");
    fs::write(root.join("notes/z.txt"), [0xff, 0xfe, 0x00]).unwrap();
    let d = reproduce(root, &before_req, &before, '2');
    assert_eq!(
        changes(&d),
        vec![
            ("file:notes/a.txt".into(), MemberChange::Changed),
            ("file:notes/b.txt".into(), MemberChange::NewlyUnresolved),
            ("file:notes/gone.txt".into(), MemberChange::NewlyAvailable),
            ("file:notes/z.txt".into(), MemberChange::NewlyOmitted),
        ]
    );
    let a = &d.changes[0];
    assert_eq!(a.fields, BTreeSet::from([ChangedField::Content]));
    assert_eq!(a.prior_state, DeltaMemberState::Present);
    assert!(a.prior_item_digest.is_some());
    assert_eq!(a.record.as_ref().unwrap().item.content, "alpha, changed\n");
    let b = &d.changes[1];
    assert_eq!(b.reason, Some(PacketOmissionReason::Missing));
    assert!(
        b.record.is_none(),
        "an omission entry carries identities only"
    );
    let gone = &d.changes[2];
    assert_eq!(gone.prior_state, DeltaMemberState::Omitted);
    assert_eq!(gone.prior_reason, Some(PacketOmissionReason::Missing));
    assert_eq!(d.changes[3].reason, Some(PacketOmissionReason::NonText));
    assert_eq!(d.counts.changed, 1);
    assert_eq!(d.counts.newly_unresolved, 1);
    assert_eq!(d.counts.newly_available, 1);
    assert_eq!(d.counts.newly_omitted, 1);
    assert_eq!(d.counts.removed, 0);
    assert_eq!(d.counts.interface_changed, 0);
}

#[test]
fn a_key_that_leaves_the_resolved_set_is_removed() {
    // Under one request a key leaves the resolved set only when the
    // predecessor named it and the target does not: a predecessor carrying a
    // member the request never resolves to.
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", notes(&["a"]));
    let honest = pages_at(root, &req, '1', BUILD);
    let mut forged: ContextPacket = serde_json::from_str(&honest[0]).unwrap();
    let mut extra = forged.members[0].clone();
    extra.identity = "file:notes/zz.txt".into();
    extra.item.identity = extra.identity.clone();
    // Keys stay in canonical order: after `file:notes/a.txt`, before the spec.
    forged.members.insert(1, extra);
    forged.packet_digest = packet_digest(&forged).unwrap();
    let pages = vec![context_packet_document(&forged).unwrap()];
    let d = reproduce(root, &req, &pages, '1');
    assert_eq!(
        changes(&d),
        vec![("file:notes/zz.txt".into(), MemberChange::Removed)]
    );
    let e = &d.changes[0];
    assert_eq!(e.prior_state, DeltaMemberState::Present);
    assert!(e.prior_item_digest.is_some());
    assert!(e.record.is_none());
    assert_eq!(d.counts.removed, 1);
}

#[test]
fn an_interface_change_wins_over_a_record_change() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", vec![]);
    let before = pages_at(root, &req, '1', BUILD);
    spec(root, "002-b", "", "# 002\n\nWhy, now.\n");
    commit_ledger(root);
    let d = reproduce(root, &req, &before, '2');
    assert_eq!(
        changes(&d),
        vec![("spec:002-b".into(), MemberChange::InterfaceChanged)]
    );
    let e = &d.changes[0];
    assert_ne!(e.prior_interface, e.interface);
    assert!(e.prior_interface.as_ref().unwrap().digest.is_some());
    assert_eq!(e.basis, ChangeBasis::Digest);
}

// ---- acceptance 4: interface digests and spans --------------------------------

#[test]
fn an_anchor_section_change_under_an_obligation_is_an_interface_change() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request(
        "002",
        vec![selector(
            ContentSelector::Obligation {
                obligation: "001-a#R-1".into(),
                projection: Some(ContentProjection::Declaration),
                required: true,
            },
            PacketRequirement::Required,
        )],
    );
    let before = pages_at(root, &req, '1', BUILD);
    spec(
        root,
        "001-a",
        OBLIGATIONS,
        &BODY.replace("The rule text.", "The rule, amended."),
    );
    commit_ledger(root);
    let d = reproduce(root, &req, &before, '2');
    let e = d
        .changes
        .iter()
        .find(|e| e.key.identity == "obligation:001-a#R-1")
        .expect("the obligation has an entry");
    assert_eq!(e.change, MemberChange::InterfaceChanged);
    assert_eq!(
        e.prior_item_digest.as_deref(),
        Some(e.record.as_ref().unwrap().item.digest.as_str()),
        "the item is equal; only the interface moved"
    );
}

#[test]
fn a_span_only_shift_is_a_change_with_span() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request(
        "002",
        vec![selector(
            ContentSelector::SpecSection {
                spec: "001".into(),
                anchor: "3-2-other".into(),
                projection: None,
                required: true,
            },
            PacketRequirement::Required,
        )],
    );
    let before = pages_at(root, &req, '1', BUILD);
    // A line inserted above the selected section moves its span; its own
    // bytes, and so its section digest, are unchanged.
    spec(
        root,
        "001-a",
        OBLIGATIONS,
        &BODY.replace("The rule text.\n", "The rule text.\nMore.\n"),
    );
    commit_ledger(root);
    let d = reproduce(root, &req, &before, '2');
    let e = d
        .changes
        .iter()
        .find(|e| e.key.identity == "section:001-a#3-2-other")
        .expect("the section has an entry");
    assert_eq!(e.change, MemberChange::Changed);
    assert_eq!(e.fields, BTreeSet::from([ChangedField::Span]));
}

#[cfg(feature = "symbol-resolution")]
fn index_crate(root: &Path, lib: &str) {
    use spec_spine_core::shard::{BY_PACKAGE_DIR, BY_SPEC_DIR};
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "src/lib.rs", lib);
    let cfg = Config::default();
    let outcome = spec_spine_core::index(&cfg, root).unwrap();
    let dir = spec_spine_core::index_dir(&cfg, root);
    let (by_spec, by_package) = spec_spine_core::index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    let (inputs_name, inputs) = spec_spine_core::index_inputs_file(&outcome.shards).unwrap();
    fs::write(dir.join(inputs_name), inputs).unwrap();
}

#[cfg(feature = "symbol-resolution")]
#[test]
fn a_signature_change_under_a_body_projection_is_an_interface_change() {
    let tmp = corpus();
    let root = tmp.path();
    index_crate(root, "pub fn alpha(x: u32) -> u32\n{\n    x + 1\n}\n");
    let req = request(
        "002",
        vec![selector(
            ContentSelector::Symbol {
                id: "demo::alpha".into(),
                projection: Some(ContentProjection::Body),
                required: true,
            },
            PacketRequirement::Required,
        )],
    );
    let before = pages_at(root, &req, '1', BUILD);
    index_crate(root, "pub fn alpha(x: u64) -> u32\n{\n    x + 1\n}\n");
    let d = reproduce(root, &req, &before, '2');
    let e = d
        .changes
        .iter()
        .find(|e| e.key.identity == "symbol:demo::alpha")
        .expect("the symbol has an entry");
    assert_eq!(e.change, MemberChange::InterfaceChanged);
    assert_eq!(e.basis, ChangeBasis::Digest);
}

// ---- acceptance 2: the reproduction law ---------------------------------------

#[test]
fn multi_page_sets_reproduce_and_deltas_chain_three_times() {
    let tmp = corpus();
    let root = tmp.path();
    let mut req = request("002", notes(&["a", "b", "c", "z"]));
    req.max_items = 2;
    let mut held = pages_at(root, &req, '1', BUILD);
    assert!(held.len() > 1, "the set must span pages");
    for (step, tree) in ['2', '3', '4'].into_iter().enumerate() {
        write(root, "notes/b.txt", &format!("bravo {step}\n"));
        if step == 1 {
            fs::remove_file(root.join("notes/c.txt")).unwrap();
        }
        let d = reproduce(root, &req, &held, tree);
        // The reproduced pages are a valid predecessor for the next delta.
        held = context_packet_apply(&held, &d).unwrap();
    }
    assert_eq!(held, pages_at(root, &req, '4', BUILD));
}

#[test]
fn a_changed_producer_build_is_a_change_not_a_fallback() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", notes(&["a"]));
    let before = pages_at(root, &req, '1', BUILD);
    let d = delta(compute(root, &delta_request(&req, before.clone()), '1', OTHER_BUILD).unwrap());
    assert!(d.producer_changed);
    assert_eq!(
        context_packet_apply(&before, &d).unwrap(),
        pages_at(root, &req, '1', OTHER_BUILD)
    );
}

#[test]
fn apply_refuses_other_bytes_and_a_tampered_delta_writes_nothing() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", notes(&["a", "b"]));
    let before = pages_at(root, &req, '1', BUILD);
    write(root, "notes/a.txt", "alpha 2\n");
    let d = reproduce(root, &req, &before, '2');

    let other = pages_at(root, &req, '2', BUILD);
    assert!(matches!(
        context_packet_apply(&other, &d),
        Err(Error::Refused(_))
    ));

    let mut tampered = d.clone();
    tampered.changes[0].record.as_mut().unwrap().item.content = "forged\n".into();
    assert!(matches!(
        context_packet_apply(&before, &tampered),
        Err(Error::Refused(_))
    ));

    let mut doubled = d.clone();
    doubled.changes.push(d.changes[0].clone());
    assert!(matches!(
        context_packet_apply(&before, &doubled),
        Err(Error::Usage(_))
    ));

    let mut missing = d.clone();
    missing.target.pages[0]
        .members
        .push(spec_spine_types::MemberKey {
            identity: "file:notes/nowhere.txt".into(),
            projection: ContentProjection::Full,
        });
    assert!(matches!(
        context_packet_apply(&before, &missing),
        Err(Error::Usage(_))
    ));
}

// ---- acceptance 3: every fallback row -----------------------------------------

#[test]
fn every_fallback_row_is_reached_with_its_reason_and_no_content() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", notes(&["a"]));
    let before = pages_at(root, &req, '1', BUILD);
    let run = |req: &PacketRequest, pages: Vec<String>| {
        fallback_reason(compute(root, &delta_request(req, pages), '2', BUILD).unwrap())
    };

    // Row 2: a page whose recorded digest is not its recomputed one.
    let mut value: serde_json::Value = serde_json::from_str(&before[0]).unwrap();
    value["members"][0]["item"]["content"] = "edited\n".into();
    assert_eq!(
        run(&req, vec![value.to_string()]),
        DeltaFallbackReason::PredecessorUnverifiable
    );

    // Row 3: one page of a multi-page set.
    let mut paged = request("002", notes(&["a", "b", "c"]));
    paged.max_items = 1;
    let pages = pages_at(root, &paged, '1', BUILD);
    assert_eq!(
        run(&paged, vec![pages[0].clone()]),
        DeltaFallbackReason::PredecessorIncomplete
    );

    // Row 4: another repository.
    let foreign = context_packet_pages(
        &Config::default(),
        root,
        &req,
        &ContentSnapshot {
            repository: "other/repo".into(),
            ..snapshot('1')
        },
        Some(BUILD),
    )
    .unwrap();
    assert_eq!(
        run(&req, vec![context_packet_document(&foreign[0]).unwrap()]),
        DeltaFallbackReason::RepositoryChanged
    );

    // Row 5: another context-packet schema version, resealed.
    let mut older: ContextPacket = serde_json::from_str(&before[0]).unwrap();
    older.schema_version = "1.0.0".into();
    for m in &mut older.members {
        m.interface = None;
    }
    older.packet_digest = packet_digest(&older).unwrap();
    assert_eq!(
        run(&req, vec![context_packet_document(&older).unwrap()]),
        DeltaFallbackReason::SchemaVersionChanged
    );

    // Row 6: another budget.
    let mut tighter = req.clone();
    tighter.max_items = 95;
    assert_eq!(
        run(&tighter, before.clone()),
        DeltaFallbackReason::BudgetChanged
    );

    // Row 7: another request.
    let other = request("002", notes(&["a", "b"]));
    assert_eq!(
        run(&other, before.clone()),
        DeltaFallbackReason::RequestChanged
    );
}

#[test]
fn a_delta_above_the_limit_falls_back_rather_than_truncating() {
    let tmp = corpus();
    let root = tmp.path();
    let mut req = request("002", notes(&["big1", "big2"]));
    req.max_bytes = 3_000_000;
    let before = pages_at(root, &req, '1', BUILD);
    let line = "x".repeat(99) + "\n";
    write(root, "notes/big1.txt", &line.repeat(25_000));
    write(root, "notes/big2.txt", &line.repeat(25_000));
    let result = compute(root, &delta_request(&req, before), '2', BUILD).unwrap();
    let text = context_packet_delta_document(&result).unwrap();
    assert!(!text.contains("xxxx"), "a fallback carries no content");
    assert_eq!(fallback_reason(result), DeltaFallbackReason::DeltaTooLarge);
}

#[test]
fn usage_errors_and_refusals_return_no_document() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", notes(&["a"]));
    let before = pages_at(root, &req, '1', BUILD);
    let err = |dreq: ContextDeltaRequest| compute(root, &dreq, '2', BUILD).unwrap_err();

    assert!(matches!(err(delta_request(&req, vec![])), Error::Usage(_)));
    assert!(matches!(
        err(delta_request(&req, vec!["{".into()])),
        Error::Usage(_)
    ));
    let mut continued = req.clone();
    continued.continuation = Some("abc".into());
    assert!(matches!(
        err(delta_request(&continued, before.clone())),
        Error::Usage(_)
    ));
    let mut major = delta_request(&req, before.clone());
    major.consumer_schema_version = "2.0.0".into();
    assert!(matches!(err(major), Error::Refused(_)));
    let mut bad_manifest = delta_request(&req, before);
    bad_manifest.manifests = vec!["{}".into()];
    let e = err(bad_manifest);
    assert!(
        matches!(&e, Error::Usage(m) if m.contains("manifest 1")),
        "{e:?}"
    );
}

// ---- acceptance 5: forged predecessors and non-disclosure ---------------------

#[test]
fn a_forged_self_consistent_predecessor_still_yields_the_true_target() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", notes(&["a", "b"]));
    let honest = pages_at(root, &req, '1', BUILD);
    let mut forged: ContextPacket = serde_json::from_str(&honest[0]).unwrap();
    forged.members[0].item.content = "forged alpha\n".into();
    // A forged closure document under the same digest is never reused.
    forged.closure["rationale"] = "forged".into();
    forged.packet_digest = packet_digest(&forged).unwrap();
    let forged_pages = vec![context_packet_document(&forged).unwrap()];

    let d = delta(compute(root, &delta_request(&req, forged_pages.clone()), '1', BUILD).unwrap());
    assert_eq!(
        context_packet_apply(&forged_pages, &d).unwrap(),
        pages_at(root, &req, '1', BUILD),
        "the true target, from forged bytes"
    );
    assert!(d.target.closure.is_some());
    let document =
        context_packet_delta_document(&ContextDeltaResult::Delta(Box::new(d.clone()))).unwrap();
    assert!(
        !document.contains("forged"),
        "no predecessor byte appears in a delta"
    );
    assert!(matches!(
        context_packet_apply(&honest, &d),
        Err(Error::Refused(_))
    ));
}

#[test]
fn a_forged_predecessor_cannot_probe_an_unselected_key() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", notes(&["a"]));
    let honest = pages_at(root, &req, '1', BUILD);
    // Claim an omission for a key the target never selects; the delta only
    // says it left the set, and carries none of its content.
    let mut forged: ContextPacket = serde_json::from_str(&honest[0]).unwrap();
    let mut probe =
        forged
            .omissions
            .first()
            .cloned()
            .unwrap_or_else(|| spec_spine_types::PacketOmission {
                identity: String::new(),
                projection: ContentProjection::Full,
                requirement: PacketRequirement::Optional,
                origins: vec![spec_spine_types::PacketOrigin::Additional],
                reason: PacketOmissionReason::Missing,
                detail: String::new(),
            });
    probe.identity = "file:notes/z.txt".into();
    probe.detail = "probe".into();
    forged.omissions.push(probe);
    forged.packet_digest = packet_digest(&forged).unwrap();
    let forged_pages = vec![context_packet_document(&forged).unwrap()];
    let d = delta(compute(root, &delta_request(&req, forged_pages), '1', BUILD).unwrap());
    let e = d
        .changes
        .iter()
        .find(|e| e.key.identity == "file:notes/z.txt")
        .unwrap();
    assert_eq!(e.change, MemberChange::Removed);
    assert!(e.record.is_none());
    let document = context_packet_delta_document(&ContextDeltaResult::Delta(Box::new(d))).unwrap();
    assert!(!document.contains("zulu"));
}

// ---- acceptance 6: invalidation candidates ------------------------------------

fn sha(c: char) -> String {
    format!("sha256:{}", c.to_string().repeat(64))
}

/// A sealed manifest over `pages` whose inputs are the named file members.
fn manifest(pages: &[ContextPacket], inputs: &[(&str, &str)]) -> DocumentationManifest {
    let first = &pages[0];
    let digest = |identity: &str| {
        pages
            .iter()
            .flat_map(|p| &p.members)
            .find(|m| m.identity == identity)
            .map(|m| m.item.digest.clone())
            .unwrap()
    };
    let mut m = DocumentationManifest {
        schema_version: "1.0.0".into(),
        packet: ManifestPacket {
            schema_version: first.schema_version.clone(),
            snapshot: first.snapshot.clone(),
            request_digest: first.request_digest.clone(),
            closure_digest: first.closure_digest.clone(),
            page_digests: pages.iter().map(|p| p.packet_digest.clone()).collect(),
            page_completeness: pages.iter().map(|p| p.completeness).collect(),
        },
        inputs: inputs
            .iter()
            .map(|(id, identity)| ManifestInput {
                input_id: (*id).into(),
                identity: (*identity).into(),
                projection: ContentProjection::Full,
                schema_version: first.members[0].item.schema_version.clone(),
                digest: digest(identity),
                requirement: PacketRequirement::Required,
                uses: vec!["out".into()],
                label: None,
            })
            .collect(),
        interfaces: Vec::new(),
        external_references: Vec::new(),
        generator: ManifestGenerator {
            name: "docgen".into(),
            version: "1.0.0".into(),
            build_digest: sha('b'),
            arguments: Vec::new(),
            configuration_digest: sha('e'),
        },
        outputs: vec![ManifestOutput {
            output_id: "out".into(),
            path: RepoPath::parse("docs/out.md").unwrap(),
            media_type: "text/markdown".into(),
            digest: sha('d'),
            byte_length: 1,
            regions: vec![ManifestRegion {
                region_id: "all".into(),
                kind: RegionKind::Generated,
                span: RegionSpan {
                    start_line: 1,
                    end_line: 1,
                },
                digest: sha('f'),
                input_ids: inputs.iter().map(|(id, _)| (*id).to_string()).collect(),
            }],
            examples: Vec::new(),
        }],
        omissions: Vec::new(),
        manifest_digest: String::new(),
    };
    m.manifest_digest = manifest_digest(&m).unwrap();
    m
}

#[test]
fn bound_and_unbound_manifests_yield_candidates_in_an_order_free_of_input_order() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", notes(&["a", "b", "c"]));
    let typed =
        context_packet_pages(&Config::default(), root, &req, &snapshot('1'), Some(BUILD)).unwrap();
    let before: Vec<String> = typed
        .iter()
        .map(|p| context_packet_document(p).unwrap())
        .collect();
    let bound = manifest(
        &typed,
        &[("bravo", "file:notes/b.txt"), ("alpha", "file:notes/a.txt")],
    );
    let other_req = request("002", notes(&["c"]));
    let other_pages = context_packet_pages(
        &Config::default(),
        root,
        &other_req,
        &snapshot('1'),
        Some(BUILD),
    )
    .unwrap();
    let unbound = manifest(&other_pages, &[("charlie", "file:notes/c.txt")]);

    write(root, "notes/a.txt", "alpha 2\n");
    write(root, "notes/b.txt", "bravo 2\n");
    let as_text = |m: &DocumentationManifest| serde_json::to_string(m).unwrap();
    let run = |manifests: Vec<String>| {
        let mut dreq = delta_request(&req, before.clone());
        dreq.manifests = manifests;
        delta(compute(root, &dreq, '2', BUILD).unwrap())
    };
    let one = run(vec![as_text(&bound), as_text(&unbound)]);
    let two = run(vec![as_text(&unbound), as_text(&bound)]);
    assert_eq!(one, two, "manifest order never affects bytes");

    let items = &one.invalidation_candidates.items;
    let bound_items: Vec<_> = items
        .iter()
        .filter(|c| c.manifest_digest == bound.manifest_digest)
        .map(|c| (c.input_id.clone(), c.change, c.basis))
        .collect();
    assert_eq!(
        bound_items,
        vec![
            (
                Some("alpha".into()),
                Some(MemberChange::Changed),
                ChangeBasis::Digest
            ),
            (
                Some("bravo".into()),
                Some(MemberChange::Changed),
                ChangeBasis::Digest
            ),
        ]
    );
    let unbound_items: Vec<_> = items
        .iter()
        .filter(|c| c.manifest_digest == unbound.manifest_digest)
        .collect();
    assert_eq!(unbound_items.len(), 1);
    assert_eq!(unbound_items[0].basis, ChangeBasis::UnboundManifest);
    assert_eq!(unbound_items[0].input_id, None);
    assert_eq!(unbound_items[0].key, None);

    // On a fallback every manifest is a whole-manifest candidate.
    let mut tighter = req.clone();
    tighter.max_bytes = 1000;
    let mut dreq = delta_request(&tighter, before.clone());
    dreq.manifests = vec![as_text(&bound), as_text(&unbound)];
    match compute(root, &dreq, '2', BUILD).unwrap() {
        ContextDeltaResult::Fallback(f) => {
            assert_eq!(f.invalidation_candidates.items.len(), 2);
            assert!(
                f.invalidation_candidates
                    .items
                    .iter()
                    .all(|c| c.basis == ChangeBasis::Fallback && c.input_id.is_none())
            );
        }
        ContextDeltaResult::Delta(_) => panic!("expected a fallback"),
    }
}

#[test]
fn a_changed_closure_selects_inputs_omitted_on_both_sides_by_closure() {
    let tmp = corpus();
    let root = tmp.path();
    // The closure root is larger than the budget, so it is omitted as
    // oversized on both sides with an equal record; only its closure digest
    // moves when one byte of its text changes.
    spec(root, "003-c", "", &format!("{BODY}Tail a.\n"));
    commit_ledger(root);
    let mut req = request("003", notes(&["a"]));
    req.max_bytes = 50;
    let typed =
        context_packet_pages(&Config::default(), root, &req, &snapshot('1'), Some(BUILD)).unwrap();
    assert!(
        typed[0].omissions.iter().any(
            |o| o.identity == "spec:003-c" && o.reason == PacketOmissionReason::OversizedMember
        ),
        "{:?}",
        typed[0].omissions
    );
    let before: Vec<String> = typed
        .iter()
        .map(|p| context_packet_document(p).unwrap())
        .collect();
    let mut m = manifest(&typed, &[("alpha", "file:notes/a.txt")]);
    m.omissions.push(spec_spine_types::ManifestOmission {
        input_id: "root".into(),
        identity: "spec:003-c".into(),
        projection: ContentProjection::Full,
        uses: vec!["out".into()],
        reason: PacketOmissionReason::OversizedMember,
    });
    m.outputs[0].regions[0].input_ids.push("root".into());
    // A closure member is required, so its omission makes the packet
    // incomplete, which spec 160 does not record. The delta binds a manifest
    // by request and page digests alone, so one that names these pages as
    // complete is still bound, and the rule must still select it.
    m.packet.page_completeness = vec![spec_spine_types::ContentCompleteness::Complete];
    m.manifest_digest = manifest_digest(&m).unwrap();

    spec(root, "003-c", "", &format!("{BODY}Tail b.\n"));
    commit_ledger(root);
    let mut dreq = delta_request(&req, before);
    dreq.manifests = vec![serde_json::to_string(&m).unwrap()];
    let d = delta(compute(root, &dreq, '2', BUILD).unwrap());
    assert!(d.changes.is_empty(), "{:?}", changes(&d));
    assert!(d.target.closure.is_some());
    let items: Vec<_> = d
        .invalidation_candidates
        .items
        .iter()
        .map(|c| (c.input_id.as_deref(), c.change, c.basis))
        .collect();
    assert_eq!(
        items,
        vec![(Some("root"), None, ChangeBasis::ClosureChanged)]
    );
}

// ---- acceptance 7: line endings, facades and schema ---------------------------

#[test]
fn line_endings_do_not_change_a_delta() {
    let mut deltas = Vec::new();
    for ending in ["\n", "\r\n", "\r"] {
        let tmp = corpus();
        let root = tmp.path();
        let req = request("002", notes(&["a"]));
        write(root, "notes/a.txt", &format!("one{ending}two{ending}"));
        let before = pages_at(root, &req, '1', BUILD);
        write(root, "notes/a.txt", &format!("one{ending}three{ending}"));
        deltas.push(
            context_packet_delta_document(
                &compute(root, &delta_request(&req, before), '2', BUILD).unwrap(),
            )
            .unwrap(),
        );
    }
    assert_eq!(deltas[0], deltas[1]);
    assert_eq!(deltas[0], deltas[2]);
}

#[test]
fn deltas_and_fallbacks_conform_to_the_embedded_schema() {
    let tmp = corpus();
    let root = tmp.path();
    let req = request("002", notes(&["a", "b"]));
    let before = pages_at(root, &req, '1', BUILD);
    write(root, "notes/a.txt", "alpha 2\n");
    let schema: serde_json::Value = serde_json::from_str(CONTEXT_PACKET_DELTA_SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let document = context_packet_delta_document(
        &compute(root, &delta_request(&req, before.clone()), '2', BUILD).unwrap(),
    )
    .unwrap();
    let instance: serde_json::Value = serde_json::from_str(&document).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    assert_eq!(instance["outcome"], "delta");

    let mut tighter = req.clone();
    tighter.max_items = 3;
    let fallback = context_packet_delta_document(
        &compute(root, &delta_request(&tighter, before), '2', BUILD).unwrap(),
    )
    .unwrap();
    let instance: serde_json::Value = serde_json::from_str(&fallback).unwrap();
    assert!(validator.is_valid(&instance), "a fallback conforms");
    assert_eq!(instance["outcome"], "fallback");
    let mut drifted = instance.clone();
    drifted["outcome"] = "delta".into();
    assert!(!validator.is_valid(&drifted), "a fallback is not a delta");
}
