//! Spec 160: documentation manifests and freshness, in core.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use spec_spine_core::{
    ExportedSpec, Exports, OutputBytes, context_packet, documentation_freshness,
    documentation_freshness_document, documentation_manifest_document, freshness_digest,
    manifest_digest, packet_snapshot, parse_documentation_manifest, read_documentation_outputs,
    shard, validate_documentation_manifest,
};
use spec_spine_types::{
    Config, ContentDirtyState, ContentProjection, ContentSelector, ContentSnapshot,
    ContentSnapshotBinding, ContextPacket, DOCUMENTATION_MANIFEST_SCHEMA, DocumentationFreshness,
    DocumentationManifest, Error, ExampleExit, ExternalPurpose, ExternalResolution,
    FreshnessReason, FreshnessStatus, ManifestExample, ManifestExternalReference,
    ManifestGenerator, ManifestInput, ManifestInterface, ManifestOmission, ManifestOutput,
    ManifestPacket, ManifestRegion, PacketClosure, PacketMemberRequest, PacketOmissionReason,
    PacketRequest, PacketRequirement, PinnedSection, RegionKind, RegionSpan, RepoPath,
};

const BUILD: &str = "sha256:abababababababababababababababababababababababababababababababab";
const OUT: &str = "docs/out.md";
const OUT_TEXT: &str = "Alpha is documented.\nA human wrote this.\nZulu is documented.\n";

fn sha(bytes: &[u8]) -> String {
    let hex: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("sha256:{hex}")
}

fn d(c: char) -> String {
    format!("sha256:{}", c.to_string().repeat(64))
}

fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// One spec, two inputs, one output and a committed ledger.
fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-10-10\"\n\
         summary: \"s\"\n---\n# 001\n\nWhy.\n",
    );
    write(tmp.path(), "notes/a.txt", "alpha\n");
    write(tmp.path(), "notes/z.txt", "zulu\n");
    write(tmp.path(), OUT, OUT_TEXT);
    let cfg = Config::default();
    let compiled = spec_spine_core::compile(&cfg, tmp.path()).unwrap();
    assert!(compiled.validation_passed);
    let dir = spec_spine_core::registry_dir(&cfg, tmp.path()).join(shard::BY_SPEC_DIR);
    shard::sync_dir(
        &dir,
        &spec_spine_core::registry_shard_files(&compiled.shards).unwrap(),
    )
    .unwrap();
    tmp
}

fn snapshot(tree: char) -> ContentSnapshot {
    ContentSnapshot {
        repository: "example/repo".into(),
        revision: "1".repeat(40),
        tree: tree.to_string().repeat(40),
        dirty_state: ContentDirtyState::CleanExport,
        binding: ContentSnapshotBinding::CallerSupplied,
    }
}

fn member(path: &str, requirement: PacketRequirement) -> PacketMemberRequest {
    PacketMemberRequest {
        selector: ContentSelector::File {
            path: RepoPath::parse(path).unwrap(),
            projection: None,
            required: true,
        },
        requirement,
    }
}

fn request(max_items: usize) -> PacketRequest {
    PacketRequest {
        closure: PacketClosure {
            root: Some("001".into()),
            digest: None,
            document: None,
        },
        members: vec![
            member("notes/a.txt", PacketRequirement::Required),
            member("notes/z.txt", PacketRequirement::Optional),
            member("notes/gone.txt", PacketRequirement::Optional),
        ],
        max_bytes: 524_288,
        max_items,
        continuation: None,
        rationale: None,
        work_identity: None,
        consumer_schema_version: "1.0.0".into(),
    }
}

/// Every page of the packet for `request(max_items)` at `snap`.
fn pages(root: &Path, snap: &ContentSnapshot, max_items: usize) -> Vec<ContextPacket> {
    let mut req = request(max_items);
    let mut out = Vec::new();
    loop {
        let page = context_packet(&Config::default(), root, &req, snap, Some(BUILD)).unwrap();
        let next = page.continuation.clone();
        out.push(page);
        match next {
            Some(token) => req.continuation = Some(token),
            None => return out,
        }
    }
}

fn digest_of(pages: &[ContextPacket], identity: &str) -> String {
    pages
        .iter()
        .flat_map(|p| &p.members)
        .find(|m| m.identity == identity)
        .unwrap()
        .item
        .digest
        .clone()
}

fn region(id: &str, kind: RegionKind, line: usize, text: &str, inputs: &[&str]) -> ManifestRegion {
    ManifestRegion {
        region_id: id.into(),
        kind,
        span: RegionSpan {
            start_line: line,
            end_line: line,
        },
        digest: sha(text.as_bytes()),
        input_ids: inputs.iter().map(|s| s.to_string()).collect(),
    }
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// A manifest over `pages` that declares everything this corpus offers, sealed.
fn manifest(pages: &[ContextPacket]) -> DocumentationManifest {
    let first = &pages[0];
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
        inputs: vec![
            ManifestInput {
                input_id: "zulu".into(),
                identity: "file:notes/z.txt".into(),
                projection: ContentProjection::Full,
                schema_version: first.members[0].item.schema_version.clone(),
                digest: digest_of(pages, "file:notes/z.txt"),
                requirement: PacketRequirement::Optional,
                uses: strings(&["out"]),
                label: Some("zulu notes".into()),
            },
            ManifestInput {
                input_id: "alpha".into(),
                identity: "file:notes/a.txt".into(),
                projection: ContentProjection::Full,
                schema_version: first.members[0].item.schema_version.clone(),
                digest: digest_of(pages, "file:notes/a.txt"),
                requirement: PacketRequirement::Required,
                uses: strings(&["run-alpha", "out"]),
                label: None,
            },
        ],
        interfaces: vec![ManifestInterface {
            input_id: "upstream".into(),
            corpus: "upstream".into(),
            spec: "010-up".into(),
            digest: upstream_digest(UPSTREAM),
            sections: Vec::new(),
            requirement: PacketRequirement::Required,
            uses: strings(&["out"]),
        }],
        external_references: vec![ManifestExternalReference {
            input_id: "policy".into(),
            namespace: "statecraft".into(),
            schema_version: "1.0.0".into(),
            identity: "policy/docs".into(),
            digest: d('c'),
            requirement: PacketRequirement::Required,
            uses: strings(&["out"]),
            purpose: ExternalPurpose::Policy,
        }],
        generator: ManifestGenerator {
            name: "docgen".into(),
            version: "1.2.3".into(),
            build_digest: d('b'),
            arguments: strings(&["--b", "--a"]),
            configuration_digest: d('e'),
        },
        outputs: vec![ManifestOutput {
            output_id: "out".into(),
            path: RepoPath::parse(OUT).unwrap(),
            media_type: "text/markdown".into(),
            digest: sha(OUT_TEXT.as_bytes()),
            byte_length: OUT_TEXT.len() as u64,
            regions: vec![
                region(
                    "zulu",
                    RegionKind::Generated,
                    3,
                    "Zulu is documented.\n",
                    &["zulu", "upstream", "policy"],
                ),
                region(
                    "alpha",
                    RegionKind::Generated,
                    1,
                    "Alpha is documented.\n",
                    &["alpha"],
                ),
                region(
                    "human",
                    RegionKind::HumanAuthored,
                    2,
                    "A human wrote this.\n",
                    &[],
                ),
            ],
            examples: vec![ManifestExample {
                example_id: "run-alpha".into(),
                region_id: "alpha".into(),
                arguments: strings(&["docgen", "--check"]),
                working_directory: ".".into(),
                input_ids: strings(&["alpha"]),
                expected_exit: ExampleExit::Ok,
                stdout_digest: Some(d('f')),
                stderr_digest: None,
            }],
        }],
        omissions: vec![ManifestOmission {
            input_id: "gone".into(),
            identity: "file:notes/gone.txt".into(),
            projection: ContentProjection::Full,
            uses: strings(&["out"]),
            reason: PacketOmissionReason::Missing,
        }],
        manifest_digest: String::new(),
    };
    // The omitted optional input is cited by the human region, so the
    // citation stays reciprocal.
    m.outputs[0].regions[2].input_ids = strings(&["gone"]);
    seal(&mut m);
    m
}

fn seal(m: &mut DocumentationManifest) {
    m.manifest_digest = manifest_digest(m).unwrap();
}

const UPSTREAM: &str = "---\nid: \"010-up\"\ntitle: \"U\"\nstatus: approved\ncreated: \"2026-01-01\"\nsummary: \"s\"\n---\n# 010\n\n## 3. Behavior\n\nThe producer contract.\n";

fn upstream_digest(text: &str) -> String {
    let mut exports = Exports::new();
    exports.insert("upstream".into(), upstream(text));
    // The pin is whatever the spec 110 verifier observes for this text.
    let r = spec_spine_core::verify_reference(
        "probe",
        &spec_spine_types::InterfaceReference {
            corpus: "upstream".into(),
            spec: "010-up".into(),
            digest: d('0'),
            sections: Vec::new(),
            obtained: "2026-10-10".into(),
            rationale: None,
        },
        &exports,
    );
    r.observed_digest.unwrap()
}

fn upstream(text: &str) -> BTreeMap<String, ExportedSpec> {
    let mut specs = BTreeMap::new();
    specs.insert(
        "010-up".into(),
        ExportedSpec {
            path: "specs/010-up/spec.md".into(),
            text: text.into(),
        },
    );
    specs
}

fn exports() -> Exports {
    let mut e = Exports::new();
    e.insert("upstream".into(), upstream(UPSTREAM));
    e
}

fn resolutions() -> Vec<ExternalResolution> {
    vec![ExternalResolution {
        namespace: "statecraft".into(),
        schema_version: "1.0.0".into(),
        identity: "policy/docs".into(),
        digest: d('c'),
    }]
}

fn outputs(root: &Path, m: &DocumentationManifest) -> OutputBytes {
    read_documentation_outputs(&Config::default(), root, ".", m).unwrap()
}

fn validate(
    root: &Path,
    m: &DocumentationManifest,
    pages: &[ContextPacket],
) -> Result<DocumentationManifest, Error> {
    validate_documentation_manifest(
        m,
        pages,
        &packet_snapshot(&snapshot('2')),
        &outputs(root, m),
    )
}

fn fresh_at(
    root: &Path,
    m: &DocumentationManifest,
    tree: char,
) -> Result<DocumentationFreshness, Error> {
    let snap = snapshot(tree);
    let current = pages(root, &snap, 96);
    documentation_freshness(
        m,
        &packet_snapshot(&snap),
        &current,
        &exports(),
        &resolutions(),
        &outputs(root, m),
    )
}

fn kinds(result: &DocumentationFreshness) -> Vec<(FreshnessReason, String)> {
    result
        .findings
        .iter()
        .map(|f| (f.kind, f.identity.clone()))
        .collect()
}

fn usage(r: Result<DocumentationManifest, Error>) -> String {
    match r {
        Err(Error::Usage(m)) => m,
        other => panic!("expected a usage error, got {other:?}"),
    }
}

fn refused<T: std::fmt::Debug>(r: Result<T, Error>) -> String {
    match r {
        Err(Error::Refused(m)) => m,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

fn schema_validator() -> jsonschema::Validator {
    let schema: serde_json::Value = serde_json::from_str(DOCUMENTATION_MANIFEST_SCHEMA).unwrap();
    jsonschema::validator_for(&schema).unwrap()
}

// ---- 3.1 - 3.9, 3.13: a complete manifest validates to canonical bytes ------

#[test]
fn a_complete_manifest_validates_to_canonical_bytes_and_conforms() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let m = manifest(&p);
    let canonical = validate(tmp.path(), &m, &p).unwrap();
    let text = documentation_manifest_document(&canonical).unwrap();
    assert!(text.ends_with("}\n") && !text.ends_with("\n\n"));
    assert!(!text.contains('\r'));
    // Canonical order: inputs, regions and uses sorted; arguments kept.
    assert_eq!(canonical.inputs[0].input_id, "alpha");
    assert_eq!(canonical.inputs[0].uses, strings(&["out", "run-alpha"]));
    let ids: Vec<&str> = canonical.outputs[0]
        .regions
        .iter()
        .map(|r| r.region_id.as_str())
        .collect();
    assert_eq!(ids, ["alpha", "human", "zulu"]);
    assert_eq!(canonical.generator.arguments, strings(&["--b", "--a"]));
    assert_eq!(canonical.manifest_digest, m.manifest_digest);
    // Reparsing the document gives the same bytes and digest.
    let again = parse_documentation_manifest(&text).unwrap();
    assert_eq!(documentation_manifest_document(&again).unwrap(), text);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let validator = schema_validator();
    let errors: Vec<String> = validator
        .iter_errors(&value)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn authored_order_never_changes_the_digest() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let m = manifest(&p);
    let mut shuffled = m.clone();
    shuffled.inputs.reverse();
    shuffled.outputs[0].regions.reverse();
    shuffled.inputs[0].uses.reverse();
    assert_eq!(manifest_digest(&shuffled).unwrap(), m.manifest_digest);
    assert_eq!(
        documentation_manifest_document(&shuffled).unwrap(),
        documentation_manifest_document(&m).unwrap()
    );
    let mut args = m.clone();
    args.generator.arguments.reverse();
    assert_ne!(manifest_digest(&args).unwrap(), m.manifest_digest);
}

#[test]
fn the_fixture_manifest_has_stable_canonical_bytes() {
    // A pure structural fixture: its canonical bytes and digest are pinned,
    // so a platform or serializer change that moved them would fail here.
    let authored = include_str!("fixtures/documentation-manifest/authored.json");
    let canonical = include_str!("fixtures/documentation-manifest/canonical.json");
    let m = parse_documentation_manifest(authored).unwrap();
    assert_eq!(documentation_manifest_document(&m).unwrap(), canonical);
    let pinned = parse_documentation_manifest(canonical).unwrap();
    assert_eq!(manifest_digest(&m).unwrap(), pinned.manifest_digest);
}

#[test]
fn a_multi_page_packet_validates_and_a_broken_chain_refuses() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 1);
    assert!(p.len() > 1, "the budget pages the packet");
    let m = manifest(&p);
    validate(tmp.path(), &m, &p).unwrap();
    let mut reordered = p.clone();
    reordered.swap(0, 1);
    assert!(refused(validate(tmp.path(), &m, &reordered)).contains("continuation"));
    let mut repeated = p.clone();
    repeated[1] = repeated[0].clone();
    refused(validate(tmp.path(), &m, &repeated));
    let truncated = &p[..p.len() - 1];
    refused(validate(tmp.path(), &m, truncated));
}

// ---- refusals and usage errors (3.2 - 3.9, 3.12, 3.15) ----------------------

#[test]
fn unknown_members_and_continuations_are_usage_errors() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let text = documentation_manifest_document(&manifest(&p)).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
    value["extra"] = serde_json::json!({});
    usage(parse_documentation_manifest(&value.to_string()));
    value.as_object_mut().unwrap().remove("extra");
    value["continuation"] = serde_json::json!("next");
    assert!(usage(parse_documentation_manifest(&value.to_string())).contains("continuation"));
    let big = " ".repeat(spec_spine_types::MANIFEST_MAX_BYTES + 1);
    assert!(usage(parse_documentation_manifest(&big)).contains("limit"));
}

#[test]
fn structural_defects_are_usage_errors() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let base = manifest(&p);
    let check = |edit: &dyn Fn(&mut DocumentationManifest), needle: &str| {
        let mut m = base.clone();
        edit(&mut m);
        seal(&mut m);
        // Structure is checked before any byte, so no output is read here.
        let msg = usage(validate_documentation_manifest(
            &m,
            &p,
            &packet_snapshot(&snapshot('2')),
            &OutputBytes::new(),
        ));
        assert!(msg.contains(needle), "{needle}: {msg}");
    };
    check(
        &|m| m.outputs[0].regions[0].span.start_line = 2,
        "invalid-region",
    );
    check(
        &|m| m.outputs[0].regions[2].span.start_line = 1,
        "invalid-region",
    );
    check(&|m| m.inputs[1].input_id = "zulu".into(), "duplicate-id");
    check(&|m| m.inputs[0].uses.push("nowhere".into()), "dangling-id");
    check(
        &|m| m.inputs[0].uses = strings(&["run-alpha"]),
        "citation-mismatch",
    );
    check(
        &|m| m.outputs[0].regions[1].input_ids.clear(),
        "cites no input",
    );
    check(
        &|m| m.generator.build_digest = "sha256:XYZ".into(),
        "buildDigest",
    );
    check(&|m| m.schema_version = "2.0.0".into(), "unsupported-schema");
    check(&|m| m.packet.closure_digest = d('a'), "closureDigest");
    check(&|m| m.inputs[0].label = Some("x".repeat(257)), "label");
    check(
        &|m| m.packet.page_completeness[0] = spec_spine_types::ContentCompleteness::Partial,
        "incomplete-packet",
    );
    check(
        &|m| m.outputs[0].path = RepoPath::parse("./docs/out.md").unwrap(),
        "normalized",
    );
    check(
        &|m| {
            let mut second = m.outputs[0].clone();
            second.output_id = "copy".into();
            second.examples.clear();
            second.regions = vec![ManifestRegion {
                region_id: "all".into(),
                kind: RegionKind::HumanAuthored,
                span: RegionSpan {
                    start_line: 1,
                    end_line: 3,
                },
                digest: sha(OUT_TEXT.as_bytes()),
                input_ids: Vec::new(),
            }];
            m.outputs.push(second);
        },
        "duplicate-path",
    );
    // An unsealed edit is a digest mismatch.
    let mut m = base.clone();
    m.generator.name = "other".into();
    assert!(usage(validate(tmp.path(), &m, &p)).contains("manifest-digest-mismatch"));
}

#[test]
fn limits_refuse_rather_than_truncate() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let mut m = manifest(&p);
    for i in 0..spec_spine_types::MANIFEST_MAX_EXTERNAL_REFERENCES {
        let mut e = m.external_references[0].clone();
        e.input_id = format!("x{i}");
        e.identity = format!("policy/{i}");
        m.external_references.push(e);
    }
    seal(&mut m);
    assert!(usage(validate(tmp.path(), &m, &p)).contains("limit-exceeded"));
}

#[test]
fn bytes_that_disagree_with_the_declaration_refuse() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let m = manifest(&p);

    // Undeclared trailing bytes.
    write(tmp.path(), OUT, &format!("{OUT_TEXT}tail\n"));
    let mut longer = m.clone();
    longer.outputs[0].digest = sha(format!("{OUT_TEXT}tail\n").as_bytes());
    longer.outputs[0].byte_length += 5;
    seal(&mut longer);
    assert!(refused(validate(tmp.path(), &longer, &p)).contains("undeclared-output-bytes"));

    // A region whose bytes changed under an unchanged span.
    let text = "Alpha is documented.\nA robot wrote this.\nZulu is documented.\n";
    write(tmp.path(), OUT, text);
    let mut edited = m.clone();
    edited.outputs[0].digest = sha(text.as_bytes());
    seal(&mut edited);
    assert!(refused(validate(tmp.path(), &edited, &p)).contains("region-mismatch"));

    // The whole file no longer matches its digest.
    assert!(refused(validate(tmp.path(), &m, &p)).contains("output-mismatch"));

    // A binary output is unsupported.
    fs::write(tmp.path().join(OUT), [0xff, 0xfe, 0x00]).unwrap();
    assert!(refused(validate(tmp.path(), &m, &p)).contains("unsupported-output"));
}

#[test]
fn packet_and_snapshot_mismatches_refuse() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let m = manifest(&p);
    // Validation needs the exact recorded snapshot.
    let other = packet_snapshot(&snapshot('3'));
    assert!(
        refused(validate_documentation_manifest(
            &m,
            &p,
            &other,
            &outputs(tmp.path(), &m)
        ))
        .contains("snapshot-mismatch")
    );
    // A page from another snapshot mixes snapshots.
    let mut mixed = pages(tmp.path(), &snapshot('2'), 1);
    let foreign = pages(tmp.path(), &snapshot('3'), 1);
    let last = mixed.len() - 1;
    mixed[last] = foreign[last].clone();
    let multi = manifest(&pages(tmp.path(), &snapshot('2'), 1));
    assert!(refused(validate(tmp.path(), &multi, &mixed)).contains("mixed-snapshot"));
    // A tampered page does not recompute.
    let mut tampered = p.clone();
    tampered[0].warnings.clear();
    tampered[0].omissions.clear();
    assert!(refused(validate(tmp.path(), &m, &tampered)).contains("packet-integrity"));
}

#[test]
fn inputs_must_match_the_packet() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let base = manifest(&p);
    let mut wrong = base.clone();
    wrong.inputs[1].digest = d('9');
    seal(&mut wrong);
    assert!(refused(validate(tmp.path(), &wrong, &p)).contains("input-mismatch"));
    // A required input the packet does not carry.
    let mut absent = base.clone();
    absent.inputs[1].identity = "file:notes/none.txt".into();
    seal(&mut absent);
    assert!(refused(validate(tmp.path(), &absent, &p)).contains("input-missing"));
    // An omitted optional input presented as present content.
    let mut present = base.clone();
    let o = present.omissions.remove(0);
    present.inputs.push(ManifestInput {
        input_id: o.input_id,
        identity: o.identity,
        projection: o.projection,
        schema_version: base.inputs[0].schema_version.clone(),
        digest: d('1'),
        requirement: PacketRequirement::Optional,
        uses: o.uses,
        label: None,
    });
    seal(&mut present);
    assert!(refused(validate(tmp.path(), &present, &p)).contains("input-omitted"));
    // An omission whose reason is not the packet's.
    let mut reason = base.clone();
    reason.omissions[0].reason = PacketOmissionReason::Removed;
    seal(&mut reason);
    assert!(refused(validate(tmp.path(), &reason, &p)).contains("omission-mismatch"));
}

#[test]
fn outputs_are_contained_beneath_the_output_root() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let m = manifest(&p);
    let cfg = Config::default();
    read_documentation_outputs(&cfg, tmp.path(), "docs", &m).unwrap();
    assert!(
        refused(read_documentation_outputs(&cfg, tmp.path(), "notes", &m)).contains("path-escape")
    );
    let mut derived = m.clone();
    let generated = format!("{}/out.md", cfg.layout.derived_dir);
    derived.outputs[0].path = RepoPath::parse(&generated).unwrap();
    assert!(
        refused(read_documentation_outputs(&cfg, tmp.path(), ".", &derived))
            .contains("generated-root")
    );
    let mut git = m.clone();
    git.outputs[0].path = RepoPath::parse(".git/out.md").unwrap();
    refused(read_documentation_outputs(&cfg, tmp.path(), ".", &git));
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("x.md"), OUT_TEXT).unwrap();
        fs::remove_file(tmp.path().join(OUT)).unwrap();
        std::os::unix::fs::symlink(outside.path().join("x.md"), tmp.path().join(OUT)).unwrap();
        assert!(
            refused(read_documentation_outputs(&cfg, tmp.path(), ".", &m)).contains("path-escape")
        );
    }
}

// ---- 3.10, 3.11, 3.14: freshness ---------------------------------------------

#[test]
fn unchanged_declarations_are_fresh_and_a_changed_tree_is_said() {
    let tmp = corpus();
    let m = manifest(&pages(tmp.path(), &snapshot('2'), 96));
    let same = fresh_at(tmp.path(), &m, '2').unwrap();
    assert_eq!(same.status, FreshnessStatus::Fresh);
    assert!(same.findings.is_empty());
    assert!(!same.repository_tree_changed);
    // Another clean tree whose declared slice is unchanged is fresh, and
    // says the tree changed: only the declared slice is (3.14, D-17).
    let moved = fresh_at(tmp.path(), &m, '3').unwrap();
    assert!(moved.repository_tree_changed);
    assert!(moved.findings.is_empty(), "{:?}", moved.findings);
    assert_eq!(moved.status, FreshnessStatus::Fresh);
    assert_ne!(moved.current_snapshot, moved.recorded_snapshot);
    let text = documentation_freshness_document(&same).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let errors: Vec<String> = schema_validator()
        .iter_errors(&value)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(same.freshness_digest, freshness_digest(&same).unwrap());
}

#[test]
fn a_changed_input_and_output_are_named_without_content() {
    let tmp = corpus();
    let m = manifest(&pages(tmp.path(), &snapshot('2'), 96));
    write(tmp.path(), "notes/a.txt", "SECRET-ALPHA\n");
    write(
        tmp.path(),
        OUT,
        "Inserted first.\nAlpha is documented.\nA human wrote this.\nZulu changed.\n",
    );
    let r = fresh_at(tmp.path(), &m, '2').unwrap();
    assert_eq!(
        kinds(&r),
        vec![
            (FreshnessReason::Changed, "input:alpha".into()),
            (FreshnessReason::Changed, "packet".into()),
            (FreshnessReason::Changed, "region:out#zulu".into()),
            (FreshnessReason::RegionShifted, "region:out#alpha".into()),
            (FreshnessReason::RegionShifted, "region:out#human".into()),
            (FreshnessReason::OutputChanged, "output:out".into()),
        ]
    );
    assert_eq!(r.status, FreshnessStatus::Stale);
    let alpha = &r.findings[0];
    assert_eq!(alpha.affects, strings(&["out", "run-alpha"]));
    assert!(alpha.current_digest.is_some() && alpha.recorded_digest.is_some());
    let shifted = &r.findings[3];
    assert_eq!(shifted.affects, strings(&["run-alpha"]));
    let text = documentation_freshness_document(&r).unwrap();
    for leaked in ["SECRET", "Inserted", "Zulu changed", "alpha\n"] {
        assert!(!text.contains(leaked), "{leaked} leaked");
    }
}

#[test]
fn a_missing_required_input_is_incomplete_and_an_optional_one_stale() {
    let tmp = corpus();
    let m = manifest(&pages(tmp.path(), &snapshot('2'), 96));
    fs::remove_file(tmp.path().join("notes/z.txt")).unwrap();
    let r = fresh_at(tmp.path(), &m, '2').unwrap();
    assert!(kinds(&r).contains(&(FreshnessReason::Missing, "input:zulu".into())));
    assert_eq!(r.status, FreshnessStatus::Stale);
    fs::remove_file(tmp.path().join("notes/a.txt")).unwrap();
    let r = fresh_at(tmp.path(), &m, '2').unwrap();
    assert!(kinds(&r).contains(&(FreshnessReason::Missing, "input:alpha".into())));
    assert_eq!(r.status, FreshnessStatus::Incomplete);
    // A recorded omission that now resolves is a change.
    write(tmp.path(), "notes/gone.txt", "back\n");
    let r = fresh_at(tmp.path(), &m, '2').unwrap();
    assert!(kinds(&r).contains(&(FreshnessReason::Changed, "input:gone".into())));
}

#[test]
fn external_and_interface_inputs_take_their_owners_results() {
    let tmp = corpus();
    let m = manifest(&pages(tmp.path(), &snapshot('2'), 96));
    let snap = snapshot('2');
    let current = pages(tmp.path(), &snap, 96);
    let outs = outputs(tmp.path(), &m);
    let run = |exports: &Exports, resolutions: &[ExternalResolution]| {
        documentation_freshness(
            &m,
            &packet_snapshot(&snap),
            &current,
            exports,
            resolutions,
            &outs,
        )
        .unwrap()
    };
    // No resolution and no export: unverified, never unchanged.
    let r = run(&Exports::new(), &[]);
    assert_eq!(
        kinds(&r),
        vec![
            (FreshnessReason::Unverified, "input:policy".into()),
            (FreshnessReason::Unverified, "input:upstream".into()),
        ]
    );
    assert_eq!(r.status, FreshnessStatus::Unverified);
    // A moved producer and a different external digest are changes.
    let mut moved = Exports::new();
    moved.insert("upstream".into(), upstream(&format!("{UPSTREAM}More.\n")));
    let mut other = resolutions();
    other[0].digest = d('d');
    let r = run(&moved, &other);
    assert_eq!(
        kinds(&r),
        vec![
            (FreshnessReason::Changed, "input:policy".into()),
            (FreshnessReason::Changed, "input:upstream".into()),
        ]
    );
    assert_eq!(
        r.findings[0].current_digest.as_deref(),
        Some(d('d').as_str())
    );
    // A producer absent from a supplied export is missing.
    let mut empty = Exports::new();
    empty.insert("upstream".into(), BTreeMap::new());
    let r = run(&empty, &resolutions());
    assert_eq!(
        kinds(&r),
        vec![(FreshnessReason::Missing, "input:upstream".into())]
    );
    assert_eq!(r.status, FreshnessStatus::Incomplete);
    // A resolution for another schema version does not match.
    let mut versioned = resolutions();
    versioned[0].schema_version = "2.0.0".into();
    let r = run(&exports(), &versioned);
    assert_eq!(
        kinds(&r),
        vec![(FreshnessReason::Unverified, "input:policy".into())]
    );
}

#[test]
fn a_pinned_section_is_reused_from_spec_110() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let mut m = manifest(&p);
    m.interfaces[0].sections = vec![PinnedSection {
        anchor: "3-behavior".into(),
        digest: d('a'),
    }];
    seal(&mut m);
    let snap = snapshot('2');
    let r = documentation_freshness(
        &m,
        &packet_snapshot(&snap),
        &pages(tmp.path(), &snap, 96),
        &exports(),
        &resolutions(),
        &outputs(tmp.path(), &m),
    )
    .unwrap();
    // The whole-spec digest still matches, so the pin is current.
    assert_eq!(r.status, FreshnessStatus::Fresh, "{:?}", r.findings);
}

#[test]
fn freshness_refuses_mixed_or_foreign_current_pages() {
    let tmp = corpus();
    let m = manifest(&pages(tmp.path(), &snapshot('2'), 96));
    let snap = snapshot('3');
    let current = pages(tmp.path(), &snapshot('2'), 96);
    let outs = outputs(tmp.path(), &m);
    assert!(
        refused(documentation_freshness(
            &m,
            &packet_snapshot(&snap),
            &current,
            &exports(),
            &resolutions(),
            &outs
        ))
        .contains("mixed-snapshot")
    );
    let mut req = request(96);
    req.members.pop();
    let foreign =
        vec![context_packet(&Config::default(), tmp.path(), &req, &snap, Some(BUILD)).unwrap()];
    assert!(
        refused(documentation_freshness(
            &m,
            &packet_snapshot(&snap),
            &foreign,
            &exports(),
            &resolutions(),
            &outs
        ))
        .contains("request-mismatch")
    );
}

#[test]
fn a_missing_or_binary_output_is_named() {
    let tmp = corpus();
    let m = manifest(&pages(tmp.path(), &snapshot('2'), 96));
    fs::write(tmp.path().join(OUT), [0xff, 0x00]).unwrap();
    let r = fresh_at(tmp.path(), &m, '2').unwrap();
    assert!(kinds(&r).contains(&(FreshnessReason::Unsupported, "output:out".into())));
    assert_eq!(r.status, FreshnessStatus::Incomplete);
    fs::remove_file(tmp.path().join(OUT)).unwrap();
    let r = fresh_at(tmp.path(), &m, '2').unwrap();
    assert!(kinds(&r).contains(&(FreshnessReason::Missing, "output:out".into())));
}

#[test]
fn the_json_facades_round_trip() {
    let tmp = corpus();
    let p = pages(tmp.path(), &snapshot('2'), 96);
    let m = manifest(&p);
    let cfg = serde_json::to_string(&Config::default()).unwrap();
    let root = tmp.path().to_str().unwrap();
    let manifest_json = documentation_manifest_document(&m).unwrap();
    let pages_json = serde_json::to_string(&p).unwrap();
    let snap_json = serde_json::to_string(&snapshot('2')).unwrap();
    let validated = spec_spine_core::documentation_manifest::documentation_manifest_json(
        &cfg,
        root,
        &manifest_json,
        &pages_json,
        &snap_json,
        ".",
    )
    .unwrap();
    assert_eq!(validated, manifest_json);
    let exports_json = serde_json::json!({
        "upstream": { "010-up": { "path": "specs/010-up/spec.md", "text": UPSTREAM } }
    })
    .to_string();
    let resolutions_json = serde_json::to_string(&resolutions()).unwrap();
    let fresh = spec_spine_core::documentation_manifest::documentation_freshness_json(
        &cfg,
        root,
        &manifest_json,
        &pages_json,
        &snap_json,
        &exports_json,
        &resolutions_json,
        ".",
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&fresh).unwrap();
    assert_eq!(value["status"], "fresh");
    assert_eq!(value["repositoryTreeChanged"], false);
}
