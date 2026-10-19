//! Spec 166: one deterministic reference page per spec, in core.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    capability_catalog, gather_reference_inputs, reference_check, reference_check_with,
    reference_export, reference_generate, reference_model, reference_model_document,
};
use spec_spine_types::{
    Config, Error, REFERENCE_LIMITATION, REFERENCE_MAX_PAGES, REFERENCE_PAGE_SCHEMA,
    REFERENCE_REGIONS, ReferenceFindingKind, ReferenceRequest,
};

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reference");

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        let kind = e.file_type().unwrap();
        assert!(
            !kind.is_symlink(),
            "fixture entry {} is a symbolic link",
            e.path().display()
        );
        if kind.is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            fs::copy(e.path(), target).unwrap();
        }
    }
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// Commit the registry and index shards as `compile` and `index` write them.
fn commit_ledger(root: &Path) {
    let cfg = Config::default();
    let compiled = spec_spine_core::compile(&cfg, root).unwrap();
    assert!(
        compiled.validation_passed,
        "{:?}",
        compiled.registry.validation
    );
    let dir = spec_spine_core::registry_dir(&cfg, root).join(BY_SPEC_DIR);
    shard::sync_dir(
        &dir,
        &spec_spine_core::registry_shard_files(&compiled.shards).unwrap(),
    )
    .unwrap();
    let outcome = spec_spine_core::index(&cfg, root).unwrap();
    let dir = spec_spine_core::index_dir(&cfg, root);
    let (by_spec, by_package) = spec_spine_core::index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    let (name, inputs) = spec_spine_core::index_inputs_file(&outcome.shards).unwrap();
    fs::write(dir.join(name), inputs).unwrap();
}

/// A catalog naming one operation governed by `001-alpha`, with one example,
/// and one operation whose path is alpha's but whose governor is not.
fn catalog_text() -> String {
    let mut catalog = capability_catalog().unwrap();
    catalog
        .operations
        .retain(|o| o.name == "registry.list" || o.name == "config.show");
    for o in &mut catalog.operations {
        o.governed_by = if o.name == "registry.list" {
            vec!["001-alpha".into()]
        } else {
            vec!["002-beta".into()]
        };
        o.examples
            .retain(|e| e.fixture.as_deref() == Some("corpus"));
    }
    serde_json::to_string_pretty(&catalog).unwrap()
}

/// The fixture corpus, copied, with its ledger committed and its catalog.
fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&Path::new(FIXTURE).join("corpus"), tmp.path());
    write(
        tmp.path(),
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(tmp.path(), "catalog.json", &catalog_text());
    commit_ledger(tmp.path());
    tmp
}

fn request() -> ReferenceRequest {
    serde_json::from_str(&fs::read_to_string(Path::new(FIXTURE).join("request.json")).unwrap())
        .unwrap()
}

fn plain(mut r: ReferenceRequest) -> ReferenceRequest {
    r.catalog = None;
    r.traceability = false;
    r
}

fn pages(root: &Path, r: &ReferenceRequest) -> BTreeMap<String, String> {
    reference_generate(&Config::default(), root, r)
        .unwrap()
        .pages
        .into_iter()
        .collect()
}

fn page<'a>(pages: &'a BTreeMap<String, String>, id: &str) -> &'a str {
    &pages[&format!("docs/reference/{id}.md")]
}

/// The body of one region, delimiters excluded.
fn region<'a>(page: &'a str, id: &str) -> &'a str {
    let kind = if id == "notes" {
        "authored"
    } else {
        "generated"
    };
    let begin = format!("<!-- spec-spine:begin {kind} id=\"{id}\" -->\n");
    let end = format!("<!-- spec-spine:end {kind} id=\"{id}\" -->");
    let a = page.find(&begin).unwrap() + begin.len();
    let b = page[a..].find(&end).unwrap() + a;
    &page[a..b]
}

/// Write `pages` into the tree, as `reference generate` would.
fn commit_pages(root: &Path, pages: &BTreeMap<String, String>) {
    for (path, text) in pages {
        write(root, path, text);
    }
}

fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.clone(), fs::read(&p).unwrap());
            }
        }
    }
    out
}

// ---- acceptance 1: one page per spec, twelve regions, exact absences ----

#[test]
fn every_spec_gets_one_page_with_twelve_regions_in_order() {
    let tmp = corpus();
    let all = pages(tmp.path(), &request());
    let ids = [
        "001-alpha",
        "002-beta",
        "003-gamma",
        "004-delta",
        "005-epsilon",
    ];
    assert_eq!(
        all.keys().cloned().collect::<Vec<_>>(),
        ids.iter()
            .map(|i| format!("docs/reference/{i}.md"))
            .collect::<Vec<_>>()
    );
    for id in ids {
        let p = page(&all, id);
        let order: Vec<usize> = REFERENCE_REGIONS
            .iter()
            .map(|r| {
                let kind = if *r == "notes" {
                    "authored"
                } else {
                    "generated"
                };
                p.find(&format!("<!-- spec-spine:begin {kind} id=\"{r}\" -->"))
                    .unwrap()
            })
            .collect();
        assert!(
            order.windows(2).all(|w| w[0] < w[1]),
            "{id}: regions out of order"
        );
        assert!(p.ends_with("<!-- spec-spine:end authored id=\"notes\" -->\n"));
        assert!(region(p, "limitations").ends_with(&format!("{REFERENCE_LIMITATION}\n")));
    }
    // Each lifecycle state renders from the registry record.
    assert!(region(page(&all, "001-alpha"), "lifecycle").contains("| status | `approved` |"));
    assert!(region(page(&all, "002-beta"), "lifecycle").contains("| status | `draft` |"));
    let gamma = region(page(&all, "003-gamma"), "lifecycle");
    assert!(
        gamma.contains("| status | `superseded` |")
            && gamma.contains("| superseded_by | `004-delta` |")
    );
    assert!(
        region(page(&all, "005-epsilon"), "lifecycle")
            .contains("| retirement_rationale | No longer needed. |")
    );
    // Absences are explicit and exact.
    let beta = page(&all, "002-beta");
    assert_eq!(
        region(beta, "requirements"),
        "## Requirements\n\nNone declared.\n"
    );
    assert_eq!(
        region(beta, "signatures"),
        "## Signatures\n\nNone declared.\n"
    );
    assert!(region(beta, "evidence").ends_with("No declared acceptance block.\n"));
    let alpha = page(&all, "001-alpha");
    assert!(region(alpha, "evidence").contains("```console\nspec-spine registry list\n```\n"));
    assert!(region(alpha, "evidence").contains("| `001-alpha#V-1` | `docs/alpha.md` |"));
    assert!(region(alpha, "limitations").contains("- everything else\n"));
    let own = region(alpha, "ownership");
    assert!(
        own.contains(
            "### References (non-owning)\n\n- `{\"kind\":\"file\",\"path\":\"docs/beta.md\"}`\n"
        ),
        "{own}"
    );
    assert!(region(alpha, "changes").contains("| extended by | `002-beta` |"));
    assert!(region(page(&all, "003-gamma"), "changes").contains("| superseded by | `004-delta` |"));

    let bare = pages(tmp.path(), &plain(request()));
    assert_eq!(
        region(page(&bare, "001-alpha"), "relations"),
        "## Relations\n\nNot supplied.\n"
    );
    assert_eq!(
        region(page(&bare, "001-alpha"), "operations"),
        "## Operations\n\nNot supplied.\n"
    );
    assert_eq!(
        region(page(&all, "001-alpha"), "relations"),
        "## Relations\n\nNone declared.\n"
    );
    assert_eq!(
        region(page(&all, "004-delta"), "operations"),
        "## Operations\n\nNone declared.\n"
    );
}

#[cfg(feature = "symbol-resolution")]
#[test]
fn an_owned_symbol_renders_its_signature_and_its_digest_enters_the_inputs() {
    let tmp = corpus();
    let all = pages(tmp.path(), &request());
    let alpha = page(&all, "001-alpha");
    let sig = region(alpha, "signatures");
    assert!(sig.contains("### `demo::alpha`"), "{sig}");
    assert!(
        sig.contains("```rust\npub fn alpha(x: u32) -> u32\n```\n"),
        "{sig}"
    );
    assert!(region(alpha, "inputs").contains("| `signature:demo::alpha` | `sha256:"));
}

#[cfg(not(feature = "symbol-resolution"))]
#[test]
fn without_a_resolver_a_symbol_renders_its_omission_and_no_other_projection() {
    let tmp = corpus();
    let all = pages(tmp.path(), &request());
    let sig = region(page(&all, "001-alpha"), "signatures");
    assert!(sig.contains("- Omitted: `"), "{sig}");
    assert!(!sig.contains("```"), "no fallback projection: {sig}");
}

// ---- acceptance 2: authoring order, line endings, golden bytes ----

#[test]
fn authoring_order_and_line_endings_never_change_bytes() {
    let base = corpus();
    let before = pages(base.path(), &request());
    let shuffled = corpus();
    let alpha_path = "specs/001-alpha/spec.md";
    let text = fs::read_to_string(shuffled.path().join(alpha_path)).unwrap();
    // Swap the two obligations and the two establishes entries.
    let text = text.replace(
        "  - { kind: file, path: \"docs/alpha.md\" }\n  - { kind: symbol, id: \"demo::alpha\" }\n",
        "  - { kind: symbol, id: \"demo::alpha\" }\n  - { kind: file, path: \"docs/alpha.md\" }\n",
    );
    let text = {
        let v1 = "  - id: \"V-1\"\n    kind: verification\n    text: \"Alpha is checked.\"\n    anchor: \"verification\"\n    inputs:\n      - \"docs/alpha.md\"\n";
        let r1 = "  - id: \"R-1\"\n    kind: requirement\n    text: \"Alpha holds.\"\n    anchor: \"3-1-alpha\"\n";
        text.replace(&format!("{v1}{r1}"), &format!("{r1}{v1}"))
    };
    write(shuffled.path(), alpha_path, &text);
    commit_ledger(shuffled.path());
    let after = pages(shuffled.path(), &plain(request()));
    let before_plain = pages(base.path(), &plain(request()));
    for (path, text) in &before_plain {
        // The registry shard hash moves with the source bytes, so only the
        // inputs region may differ.
        let a = text
            .split("<!-- spec-spine:begin generated id=\"inputs\" -->")
            .next()
            .unwrap();
        let b = after[path]
            .split("<!-- spec-spine:begin generated id=\"inputs\" -->")
            .next()
            .unwrap();
        if path.ends_with("001-alpha.md") {
            assert_eq!(a, b, "{path}");
        } else {
            assert_eq!(text, &after[path], "{path}");
        }
    }
    assert!(!before.is_empty());

    let mut renders = Vec::new();
    // A CR-only spec.md is not a spec (its frontmatter fence does not
    // parse), so CR is exercised on authored notes (D-23).
    for ending in ["\n", "\r\n"] {
        let tmp = corpus();
        for id in ["001-alpha", "002-beta"] {
            let p = format!("specs/{id}/spec.md");
            let text = fs::read_to_string(tmp.path().join(&p))
                .unwrap()
                .replace('\n', ending);
            write(tmp.path(), &p, &text);
        }
        commit_ledger(tmp.path());
        renders.push(pages(tmp.path(), &plain(request())));
    }
    assert_eq!(renders[0], renders[1]);
}

#[test]
fn the_retired_fixture_page_matches_its_committed_bytes() {
    let tmp = corpus();
    let mut r = plain(request());
    r.specs = serde_json::from_value(serde_json::json!({ "ids": ["005-epsilon"] })).unwrap();
    let all = pages(tmp.path(), &r);
    let expected = fs::read_to_string(Path::new(FIXTURE).join("expected/005-epsilon.md")).unwrap();
    assert_eq!(page(&all, "005-epsilon"), expected);
}

// ---- acceptance 3: a hostile summary ----

#[test]
fn a_hostile_summary_renders_escaped_and_parses() {
    let tmp = corpus();
    let p = "specs/002-beta/spec.md";
    let text = fs::read_to_string(tmp.path().join(p)).unwrap().replace(
        "summary: \"Beta extends alpha.\"",
        "summary: \"<!-- spec-spine:end generated id=\\\"header\\\" --> | ``` & \\\\ end\"",
    );
    write(tmp.path(), p, &text);
    commit_ledger(tmp.path());
    let all = pages(tmp.path(), &plain(request()));
    let beta = page(&all, "002-beta");
    assert!(region(beta, "header").contains(
        "- Summary: &lt;!-- spec-spine:end generated id=\"header\" --&gt; \\| ``` &amp; \\\\ end\n"
    ));
    commit_pages(tmp.path(), &all);
    let check = reference_check(&Config::default(), tmp.path(), &plain(request())).unwrap();
    assert!(check.findings.is_empty(), "{:?}", check.findings);
}

#[test]
fn no_generated_line_ends_in_whitespace() {
    let tmp = corpus();
    let p = "specs/002-beta/spec.md";
    let text = fs::read_to_string(tmp.path().join(p))
        .unwrap()
        .replace("summary: \"Beta extends alpha.\"", "summary: \"  \"");
    write(tmp.path(), p, &text);
    commit_ledger(tmp.path());
    let all = pages(tmp.path(), &request());
    assert!(page(&all, "002-beta").contains("- Summary:\n"));
    for (path, text) in &all {
        for (n, line) in text.lines().enumerate() {
            assert_eq!(
                line,
                line.trim_end(),
                "{path} line {} ends in whitespace",
                n + 1
            );
        }
    }
}

// ---- acceptance 4: notes ----

#[test]
fn notes_survive_regeneration_verbatim_and_line_endings_do_not_matter() {
    let tmp = corpus();
    let r = plain(request());
    let first = pages(tmp.path(), &r);
    commit_pages(tmp.path(), &first);
    let path = "docs/reference/001-alpha.md";
    let notes = "Hand written.\n\n```text\nan example in prose\n```\n";
    let edited = first[path].replace(
        "<!-- spec-spine:begin authored id=\"notes\" -->\n",
        &format!("<!-- spec-spine:begin authored id=\"notes\" -->\n{notes}"),
    );
    write(tmp.path(), path, &edited);
    let again = pages(tmp.path(), &r);
    assert_eq!(region(&again[path], "notes"), notes);
    write(tmp.path(), path, &edited.replace('\n', "\r\n"));
    assert_eq!(pages(tmp.path(), &r)[path], again[path]);
    write(tmp.path(), path, &edited.replace('\n', "\r"));
    assert_eq!(pages(tmp.path(), &r)[path], again[path]);
}

#[test]
fn a_notes_region_holding_one_blank_line_round_trips() {
    let tmp = corpus();
    let r = plain(request());
    let first = pages(tmp.path(), &r);
    commit_pages(tmp.path(), &first);
    let path = "docs/reference/001-alpha.md";
    let edited = first[path].replace(
        "<!-- spec-spine:begin authored id=\"notes\" -->\n",
        "<!-- spec-spine:begin authored id=\"notes\" -->\n\n",
    );
    assert_ne!(edited, first[path]);
    write(tmp.path(), path, &edited);
    assert_eq!(pages(tmp.path(), &r)[path], edited);
    let check = reference_check(&Config::default(), tmp.path(), &r).unwrap();
    assert!(check.findings.is_empty(), "{:?}", check.findings);
}

#[cfg(unix)]
#[test]
fn a_symlinked_page_refuses_rather_than_writing_through_it() {
    let tmp = corpus();
    let r = plain(request());
    let first = pages(tmp.path(), &r);
    commit_pages(tmp.path(), &first);
    let link = tmp.path().join("docs/reference/001-alpha.md");
    fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(tmp.path().join("specs/001-alpha/spec.md"), &link).unwrap();
    let before = tree(tmp.path());
    let err = reference_generate(&Config::default(), tmp.path(), &r).unwrap_err();
    assert!(
        matches!(err, Error::Refused(ref m) if m.contains("symbolic link")),
        "{err:?}"
    );
    assert_eq!(tree(tmp.path()), before);
}

#[test]
fn malformed_notes_and_orphans_with_notes_refuse_without_writing() {
    let tmp = corpus();
    let r = plain(request());
    let first = pages(tmp.path(), &r);
    commit_pages(tmp.path(), &first);
    let path = "docs/reference/001-alpha.md";
    let refuse = |text: &str| {
        write(tmp.path(), path, text);
        let before = tree(tmp.path());
        let out = reference_generate(&Config::default(), tmp.path(), &r);
        assert_eq!(tree(tmp.path()), before);
        out
    };
    let lurking = first[path].replace(
        "<!-- spec-spine:begin authored id=\"notes\" -->\n",
        "<!-- spec-spine:begin authored id=\"notes\" -->\n\t<!-- spec-spine: not a delimiter\n",
    );
    assert!(matches!(refuse(&lurking), Err(Error::Refused(m)) if m.contains("page-malformed")));
    let twice = format!(
        "{}\n<!-- spec-spine:begin authored id=\"notes\" -->\n<!-- spec-spine:end authored id=\"notes\" -->\n",
        first[path]
    );
    assert!(matches!(refuse(&twice), Err(Error::Refused(_))));
    write(tmp.path(), path, &first[path]);

    // An orphan with notes refuses; one without is removed.
    let orphan = first["docs/reference/002-beta.md"].replace(
        "<!-- spec-spine:begin authored id=\"notes\" -->\n",
        "<!-- spec-spine:begin authored id=\"notes\" -->\nkeep me\n",
    );
    write(tmp.path(), "docs/reference/009-gone.md", &orphan);
    let out = reference_generate(&Config::default(), tmp.path(), &r);
    assert!(matches!(out, Err(Error::Refused(m)) if m.contains("orphaned-notes")));
    write(
        tmp.path(),
        "docs/reference/009-gone.md",
        &first["docs/reference/002-beta.md"],
    );
    let export = reference_generate(&Config::default(), tmp.path(), &r).unwrap();
    assert_eq!(
        export.remove,
        vec!["docs/reference/009-gone.md".to_string()]
    );
}

// ---- acceptance 5: the regeneration check ----

#[test]
fn the_check_names_every_finding_and_writes_nothing() {
    let tmp = corpus();
    let r = plain(request());
    let cfg = Config::default();
    let first = pages(tmp.path(), &r);
    commit_pages(tmp.path(), &first);
    let before = tree(tmp.path());
    assert!(
        reference_check(&cfg, tmp.path(), &r)
            .unwrap()
            .findings
            .is_empty()
    );

    fs::remove_file(tmp.path().join("docs/reference/002-beta.md")).unwrap();
    let edited = first["docs/reference/001-alpha.md"].replace("Alpha holds.", "Alpha, edited.");
    write(tmp.path(), "docs/reference/001-alpha.md", &edited);
    write(
        tmp.path(),
        "docs/reference/009-gone.md",
        &first["docs/reference/003-gamma.md"],
    );
    write(tmp.path(), "docs/reference/004-delta.md", "not a page\n");
    let check = reference_check(&cfg, tmp.path(), &r).unwrap();
    let got: Vec<(Option<String>, ReferenceFindingKind, Vec<String>)> = check
        .findings
        .iter()
        .map(|f| (f.path.clone(), f.kind, f.regions.clone()))
        .collect();
    assert_eq!(
        got,
        vec![
            (
                Some("docs/reference/001-alpha.md".into()),
                ReferenceFindingKind::PageChanged,
                vec!["requirements".into()]
            ),
            (
                Some("docs/reference/002-beta.md".into()),
                ReferenceFindingKind::PageMissing,
                vec![]
            ),
            (
                Some("docs/reference/004-delta.md".into()),
                ReferenceFindingKind::PageMalformed,
                vec![]
            ),
            (
                Some("docs/reference/009-gone.md".into()),
                ReferenceFindingKind::PageOrphaned,
                vec![]
            ),
        ]
    );
    assert!(
        check.findings.iter().all(|f| !f.detail.contains("Alpha")),
        "no content in findings"
    );

    write(
        tmp.path(),
        "specs/002-beta/spec.md",
        "---\nid: \"002-beta\"\n---\n",
    );
    let stale = reference_check(&cfg, tmp.path(), &r).unwrap();
    assert_eq!(stale.findings.len(), 1);
    assert_eq!(stale.findings[0].kind, ReferenceFindingKind::LedgerStale);
    assert!(matches!(
        reference_generate(&cfg, tmp.path(), &r),
        Err(Error::Stale { .. })
    ));
    let mut after = tree(tmp.path());
    after.remove(&tmp.path().join("specs/002-beta/spec.md"));
    let mut before_cmp = before.clone();
    before_cmp.remove(&tmp.path().join("specs/002-beta/spec.md"));
    for p in [
        "docs/reference/002-beta.md",
        "docs/reference/001-alpha.md",
        "docs/reference/009-gone.md",
        "docs/reference/004-delta.md",
    ] {
        before_cmp.remove(&tmp.path().join(p));
        after.remove(&tmp.path().join(p));
    }
    assert_eq!(after, before_cmp, "the check writes nothing");
}

// ---- acceptance 6: binding only by declaration ----

#[test]
fn ownership_alone_never_produces_a_relation_or_an_operation() {
    let tmp = corpus();
    let all = pages(tmp.path(), &request());
    let alpha = page(&all, "001-alpha");
    let ops = region(alpha, "operations");
    assert!(ops.contains("### `registry.list`"), "{ops}");
    // The catalog's `corpus` example renders as its console line.
    assert!(
        ops.contains("```console\nspec-spine registry list --json\n```"),
        "{ops}"
    );
    assert!(!ops.contains("config.show"), "{ops}");
    // config.show is governed by beta, which owns nothing it touches.
    assert!(region(page(&all, "002-beta"), "operations").contains("### `config.show`"));
    assert_eq!(
        region(alpha, "relations"),
        "## Relations\n\nNone declared.\n"
    );
}

// ---- acceptance 7: declared reads only ----

#[test]
fn undeclared_files_and_git_never_reach_a_page() {
    let tmp = corpus();
    let r = request();
    let before = pages(tmp.path(), &r);
    write(tmp.path(), "secret.txt", "credential\n");
    write(tmp.path(), ".git/HEAD", "ref: refs/heads/main\n");
    write(
        tmp.path(),
        "docs/reference/nested/001-alpha.md",
        "not read\n",
    );
    let inputs = gather_reference_inputs(&Config::default(), tmp.path(), &r).unwrap();
    assert!(inputs.existing_pages.keys().all(|k| !k.contains('/')));
    let after: BTreeMap<String, String> = reference_export(&r, &inputs)
        .unwrap()
        .pages
        .into_iter()
        .collect();
    assert_eq!(before, after);
    assert!(after.values().all(|p| !p.contains("credential")));
}

// ---- acceptance 9: limits ----

#[test]
fn a_limit_excess_refuses_the_whole_export() {
    let tmp = corpus();
    let cfg = Config::default();
    let mut r = plain(request());
    r.limits.max_pages = 4;
    assert!(
        matches!(reference_generate(&cfg, tmp.path(), &r), Err(Error::Refused(m)) if m.contains("limit-exceeded"))
    );
    let mut r = plain(request());
    r.limits.max_page_bytes = 1000;
    assert!(
        matches!(reference_generate(&cfg, tmp.path(), &r), Err(Error::Refused(m)) if m.contains("maxPageBytes"))
    );
    let mut r = plain(request());
    // A limit can only lower its default: one page above the ceiling is a
    // malformed request, not an export that happens to fit.
    r.limits.max_pages = REFERENCE_MAX_PAGES + 1;
    assert!(matches!(
        reference_generate(&cfg, tmp.path(), &r),
        Err(Error::Usage(m)) if m.contains("may only lower their defaults")
    ));
}

// ---- requests, model and schema ----

#[test]
fn requests_are_validated_and_unknown_specs_are_not_found() {
    let tmp = corpus();
    let cfg = Config::default();
    let with = |patch: serde_json::Value| {
        let mut v = serde_json::to_value(request()).unwrap();
        for (k, val) in patch.as_object().unwrap() {
            v[k] = val.clone();
        }
        serde_json::from_value::<ReferenceRequest>(v).unwrap()
    };
    let err = |r: ReferenceRequest| reference_generate(&cfg, tmp.path(), &r).unwrap_err();
    assert!(matches!(
        err(with(serde_json::json!({"outputDir": "../out"}))),
        Error::Usage(_)
    ));
    for dir in ["docs/CON", "docs/a:b", "docs/ref.", "docs/a\u{0}b"] {
        assert!(
            matches!(
                err(with(serde_json::json!({"outputDir": dir}))),
                Error::Usage(_)
            ),
            "{dir}"
        );
    }
    assert!(matches!(
        err(with(
            serde_json::json!({"outputDir": format!("{}/ref", Config::default().layout.derived_dir)})
        )),
        Error::Refused(_)
    ));
    assert!(matches!(
        err(with(serde_json::json!({"specs": {"ids": ["001"]}}))),
        Error::Usage(_)
    ));
    assert!(matches!(
        err(with(serde_json::json!({"specs": {"ids": ["099-none"]}}))),
        Error::NotFound(_)
    ));
    assert!(matches!(
        err(with(serde_json::json!({"schemaVersion": "2.0.0"}))),
        Error::Usage(_)
    ));
}

#[test]
fn the_model_conforms_and_is_what_the_pages_project() {
    let tmp = corpus();
    let model = reference_model(&Config::default(), tmp.path(), &request()).unwrap();
    assert_eq!(model.pages.len(), 5);
    let text = reference_model_document(&model).unwrap();
    let schema: serde_json::Value = serde_json::from_str(REFERENCE_PAGE_SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let instance: serde_json::Value = serde_json::from_str(&text).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    assert!(
        !text.contains("builtAt") && !text.contains("\"revision\""),
        "no snapshot identity"
    );
    let inputs = gather_reference_inputs(&Config::default(), tmp.path(), &request()).unwrap();
    let check = reference_check_with(&request(), &inputs).unwrap();
    assert!(!check.findings.is_empty(), "no page was written yet");
    // The page schema's root is `oneOf [model, check]`; a check is judged by
    // the check definition alone, and a malformed one is refused.
    let check_schema = serde_json::json!({
        "$defs": schema["$defs"].clone(),
        "$ref": "#/$defs/check",
    });
    let check_validator = jsonschema::validator_for(&check_schema).unwrap();
    let mut check_doc: serde_json::Value = serde_json::to_value(&check).unwrap();
    assert!(check_validator.is_valid(&check_doc));
    assert!(
        !check_validator.is_valid(&instance),
        "a model is not a check"
    );
    check_doc["findings"][0]["kind"] = serde_json::json!("page-bogus");
    assert!(!check_validator.is_valid(&check_doc));

    // A spec's index shard hash covers `spec.md` and the files the spec owns,
    // so it equals the registry hash exactly when the spec owns no unit.
    let digest = |spec: &str, source: &str| {
        let page = model.pages.iter().find(|p| p.spec_id == spec).unwrap();
        let id = format!("{source}:{spec}");
        page.inputs
            .iter()
            .find(|i| i.identity == id)
            .unwrap_or_else(|| panic!("{id}"))
            .digest
            .clone()
    };
    assert_ne!(
        digest("001-alpha", "registry"),
        digest("001-alpha", "index")
    );
    assert_eq!(
        digest("005-epsilon", "registry"),
        digest("005-epsilon", "index")
    );
}
