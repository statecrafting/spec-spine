//! Spec 163: the measured resolver matrix. One fixture holds a construct for
//! every row of §1.1; the golden below is the full expected map of ids, spans,
//! test ids, outcomes and reasons, so passing on a release triple proves that
//! triple emits it (§3.9). The legacy golden was captured with the 0.29.0
//! release, whose resolver is byte-identical to 0.28.0's (§3.3).

use std::fs;
use std::path::Path;

use spec_spine_core::index;
use spec_spine_types::Config;

/// The fixture's files. `demo/src/lib.rs` is [`DEMO_LIB`] plus a generated
/// 33-deep inline-module chain, and `crlf_twin/src/lib.rs` is the LF twin's
/// text with a BOM and CRLF line endings.
const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        r#"[workspace]
members = ["demo", "declared", "lf_twin", "crlf_twin"]
"#,
    ),
    (
        "package.json",
        r#"{ "private": true, "workspaces": ["web"] }
"#,
    ),
    (
        "demo/Cargo.toml",
        r#"[package]
name = "demo"
version = "0.1.0"
edition = "2024"
"#,
    ),
    (
        "demo/src/sub.rs",
        r#"pub struct Helper;

pub fn helper() {}
"#,
    ),
    (
        "demo/src/thing.rs",
        r#"pub fn run() {}
"#,
    ),
    (
        "demo/src/broken.rs",
        r#"pub fn ok() {}

pub fn bad( {}
"#,
    ),
    (
        "demo/src/docs.rs",
        r#"/// Line doc.
#[inline]
pub fn doc_then_attr() {}

/** block doc */
pub fn block_doc() {}

#[doc = "attr doc"]
pub fn attr_doc() {}

/// Separated.

pub fn blank_separated() {}

pub fn multi_line_sig(
    a: u8,
) -> u8 {
    a
}

pub fn allman()
{
    let _x = 1;
}

/**
 * multi
 */
pub fn multi_block() {}
"#,
    ),
    (
        "demo/tests/integration.rs",
        r#"#[test]
fn integration_test_fn() {}

fn helper() {}

mod inner {
    #[test]
    fn inner_test() {}
}
"#,
    ),
    (
        "demo/tests/dirtest/main.rs",
        r#"mod support;

#[test]
fn in_dir() {}
"#,
    ),
    (
        "demo/tests/dirtest/support.rs",
        r#"pub fn support() {}
"#,
    ),
    (
        "demo/tests/common/mod.rs",
        r#"pub fn shared() {}
"#,
    ),
    (
        "declared/Cargo.toml",
        r#"[package]
name = "declared"
version = "0.1.0"
edition = "2024"

[[test]]
name = "x"
path = "tests/x.rs"
"#,
    ),
    (
        "declared/src/lib.rs",
        r#"pub fn d() {}
"#,
    ),
    (
        "declared/tests/x.rs",
        r#"#[test]
fn t() {}
"#,
    ),
    (
        "lf_twin/Cargo.toml",
        r#"[package]
name = "lf_twin"
version = "0.1.0"
edition = "2024"
"#,
    ),
    (
        "lf_twin/src/lib.rs",
        r#"pub fn first() {}

impl Twin {
    pub fn m() {}
}

pub struct Twin;
"#,
    ),
    (
        "crlf_twin/Cargo.toml",
        r#"[package]
name = "crlf_twin"
version = "0.1.0"
edition = "2024"
"#,
    ),
    (
        "web/package.json",
        r#"{ "name": "web", "version": "0.1.0" }
"#,
    ),
    (
        "web/src/util.ts",
        r#"export function formatDate() {}
/** single */
export function single() {}
function internalFn() {}
export class Helper {
  run() {}
  static make() {}
  field = 1;
  #priv() {}
  ['computed']() {}
}
export interface Shape {
  size: number;
}
export const arrowFn = () => {
  return 1;
};
let counter = 0;
var legacyVar = 1;
const { a, b } = { a: 1, b: 2 };
export namespace NS {
  export function inNs() {}
}
export { internalFn as renamed };
it("does a thing", () => {});
"#,
    ),
    (
        "web/src/comp.tsx",
        r#"export function After() {}
export class Comp {
  render() {}
}
export const tsxConst = 1;
"#,
    ),
];

const DEMO_LIB: &str = r#"mod sub;
pub mod docs;
pub mod thing;
pub mod broken;

pub fn top_fn() {}

pub struct TopStruct {
    pub field: u8,
}

pub enum TopEnum {
    A,
    B,
}

pub union TopUnion {
    a: u8,
}

pub trait TopTrait {
    fn trait_method(&self);
    fn provided(&self) -> u8 {
        1
    }
    const TC: u8;
    type Out;
}

impl TopStruct {
    pub const ASSOC: u8 = 1;
    pub type Alias = u8;
    pub fn inherent_method(&self) {}
}

impl TopTrait for TopStruct {
    fn trait_method(&self) {}
    const TC: u8 = 2;
    type Out = u8;
}

impl std::fmt::Display for TopStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x")
    }
}

impl From<u8> for TopStruct {
    fn from(v: u8) -> Self {
        TopStruct { field: v }
    }
}

pub struct Generic<T>(T);

impl<T> Generic<T> {
    pub fn gen_method(&self) {}
}

impl TopTrait for &Generic<u8> {
    fn trait_method(&self) {}
    const TC: u8 = 3;
    type Out = u8;
}

impl sub::Helper {
    pub fn scoped(&self) {}
}

#[allow(non_camel_case_types)]
pub struct thing;

impl thing {
    pub fn run() {}
}

pub mod inline_mod {
    pub fn in_inline() {}
    pub mod deeper {
        pub fn deepest() {}
    }
}

pub fn outer() {
    fn nested() {}
}

macro_rules! make_fn {
    ($name:ident) => {
        pub fn $name() {}
    };
}

#[macro_export]
macro_rules! exported_macro {
    () => {};
}

pub mod generated {
    make_fn!(macro_generated);
    pub fn written() {}
}

#[cfg(unix)]
pub fn cfg_twin() {}
#[cfg(not(unix))]
pub fn cfg_twin() {}

pub use sub::helper as reexported;

unsafe extern "C" {
    fn foreign_fn();
}

#[test]
fn top_level_test() {}

#[cfg(test)]
mod tests {
    /// Documented test.
    #[test]
    #[should_panic]
    fn unit_test_in_src() {
        panic!("x");
    }

    #[tokio::test]
    async fn tokio_test() {}

    #[test(arg)]
    fn test_with_args() {}

    fn helper_not_test() {}
}
"#;

/// `n1::n2::…::n31`, the inline-module path to the deepest descended container.
fn deep_path() -> String {
    (1..=31)
        .map(|i| format!("n{i}"))
        .collect::<Vec<_>>()
        .join("::")
}

fn deep_chain() -> String {
    let mut out = String::from("pub mod deep {\n");
    for i in 0..33 {
        out.push_str(&"    ".repeat(i + 1));
        out.push_str(&format!("pub mod n{} {{\n", i + 1));
    }
    out.push_str(&"    ".repeat(34));
    out.push_str("pub fn leaf() {}\n");
    for i in (0..33).rev() {
        out.push_str(&"    ".repeat(i + 1));
        out.push_str("}\n");
    }
    out.push_str("}\n");
    out
}

/// Replace `{DEEP}` with [`deep_path`] in an id.
fn expand(id: &str) -> String {
    id.replace("{DEEP}", &deep_path())
}

const SYMBOLS: &[&str] = &[
    "demo::top_fn",
    "demo::TopStruct",
    "demo::TopEnum",
    "demo::TopUnion",
    "demo::TopTrait",
    "demo::Generic",
    "demo::thing",
    "demo::sub",
    "demo::docs",
    "demo::inline_mod",
    "demo::outer",
    "demo::generated",
    "demo::cfg_twin",
    "demo::top_level_test",
    "demo::tests",
    "demo::deep",
    "demo::thing::run",
    "demo::sub::Helper",
    "demo::sub::helper",
    "demo::broken::ok",
    "demo::docs::doc_then_attr",
    "demo::docs::block_doc",
    "demo::docs::attr_doc",
    "demo::docs::blank_separated",
    "demo::docs::multi_line_sig",
    "demo::docs::allman",
    "demo::docs::multi_block",
    "declared::d",
    "lf_twin::first",
    "lf_twin::Twin",
    "crlf_twin::first",
    "crlf_twin::Twin",
    "web::src::util::formatDate",
    "web::src::util::single",
    "web::src::util::internalFn",
    "web::src::util::Helper",
    "web::src::util::Shape",
    "web::src::comp::After",
    "web::src::comp::Comp",
    "demo::inline_mod::in_inline",
    "demo::inline_mod::deeper",
    "demo::inline_mod::deeper::deepest",
    "demo::tests::unit_test_in_src",
    "demo::tests::helper_not_test",
    "demo::tests::tokio_test",
    "demo::TopStruct::ASSOC",
    "demo::TopStruct::Alias",
    "demo::TopStruct::inherent_method",
    "demo::Generic::gen_method",
    "demo::<TopStruct as TopTrait>::trait_method",
    "demo::<TopStruct as TopTrait>::TC",
    "demo::<TopStruct as TopTrait>::Out",
    "demo::<TopStruct as std::fmt::Display>::fmt",
    "demo::<TopStruct as From<u8>>::from",
    "demo::TopTrait::trait_method",
    "demo::TopTrait::provided",
    "demo::TopTrait::TC",
    "demo::TopTrait::Out",
    "demo::make_fn!",
    "demo::exported_macro!",
    "demo::generated::written",
    "demo/tests/integration::integration_test_fn",
    "demo/tests/integration::helper",
    "demo/tests/integration::inner",
    "demo/tests/integration::inner::inner_test",
    "demo/tests/dirtest::in_dir",
    "demo/tests/dirtest::support",
    "lf_twin::Twin::m",
    "crlf_twin::Twin::m",
    "web::src::util::Helper::run",
    "web::src::util::Helper::make",
    "web::src::util::arrowFn",
    "web::src::util::counter",
    "web::src::util::legacyVar",
    "demo::deep::{DEEP}::n32",
    "demo::outer::nested",
    "demo::<&Generic<u8> as TopTrait>::trait_method",
    "demo::sub::Helper::scoped",
    "demo::reexported",
    "web::src::util::renamed",
    "demo::TopStruct::field",
    "demo::TopEnum::A",
    "demo::TopUnion::a",
    "web::src::util::Helper::field",
    "web::src::util::Shape::size",
    "demo::foreign_fn",
    "demo/tests/common/mod::shared",
    "demo/tests/dirtest/support::support",
    "declared/tests/x::t",
    "web::src::util::NS",
    "web::src::util::NS::inNs",
    "web::src::util::a",
    "web::src::util::Helper::#priv",
    "web::src::comp::Comp::render",
    "web::src::comp::tsxConst",
    "demo::generated::macro_generated",
    "demo::broken::missing",
    "demo::deep::{DEEP}::n32::n33",
    "demo::deep::{DEEP}::n32::n33::leaf",
    "demo::TopStruct::absent",
    "demo::absent",
    "nothing::here",
];

const MODULES: &[&str] = &[
    "demo",
    "demo::sub",
    "demo::docs",
    "demo::thing",
    "demo::broken",
    "demo::inline_mod",
    "demo::generated",
    "demo::tests",
    "demo::deep",
    "demo::inline_mod::deeper",
    "demo::deep::{DEEP}::n32",
    "demo/tests/integration",
    "demo/tests/integration::inner",
    "demo/tests/dirtest",
    "demo/tests/common/mod",
    "demo/tests/dirtest/support",
    "declared/tests/x",
    "demo::deep::{DEEP}::n32::n33",
    "demo::absent",
];

#[cfg_attr(not(feature = "symbol-resolution"), allow(dead_code))]
const TESTS: &[&str] = &[
    "demo::top_level_test",
    "demo::tests::unit_test_in_src",
    "demo::tests::tokio_test",
    "demo::tests::test_with_args",
    "demo::tests::helper_not_test",
    "demo/tests/integration::integration_test_fn",
    "demo/tests/integration::inner::inner_test",
    "demo/tests/dirtest::in_dir",
    "demo/tests/common/mod::shared",
    "declared/tests/x::t",
    "web::src::util::does a thing",
    "demo::outer::nested",
    "demo::generated::macro_generated",
    "demo::absent",
];

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// The LF twin's text, with a BOM and CRLF line endings (spec 143).
fn crlf_twin() -> String {
    let lf = FILES
        .iter()
        .find(|(path, _)| *path == "lf_twin/src/lib.rs")
        .unwrap()
        .1;
    format!("\u{feff}{}", lf.replace('\n', "\r\n"))
}

/// The fixture repository, with one draft spec declaring every symbol and
/// module id so the index surfaces each outcome as a `W-001` message.
fn fixture() -> tempfile::TempDir {
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
    write(root, "crlf_twin/src/lib.rs", &crlf_twin());
    let mut units = String::new();
    for id in SYMBOLS {
        units.push_str(&format!("  - {{ kind: symbol, id: {:?} }}\n", expand(id)));
    }
    for id in MODULES {
        units.push_str(&format!("  - {{ kind: module, id: {:?} }}\n", expand(id)));
    }
    write(
        root,
        "specs/001-matrix/spec.md",
        &format!(
            "---\nid: \"001-matrix\"\ntitle: \"Matrix\"\nstatus: draft\ncreated: \"2026-10-10\"\nimplementation: pending\nsummary: \"matrix\"\nestablishes:\n{units}---\n# Matrix\n"
        ),
    );
    tmp
}

/// The index's message for one declared unit: `resolved <locations>` when it
/// resolves, else the text after `did not resolve`.
fn index_outcomes(root: &Path) -> Vec<String> {
    let outcome = index(&Config::default(), root).unwrap();
    let mapping = outcome
        .index
        .traceability
        .mappings
        .iter()
        .find(|m| m.spec_id == "001-matrix")
        .unwrap();
    let mut lines = Vec::new();
    for (kind, ids) in [("symbol", SYMBOLS), ("module", MODULES)] {
        for id in ids {
            let id = expand(id);
            let resolved = mapping.resolved_units.iter().find(|r| {
                let subject = serde_json::to_value(r.unit.subject()).unwrap();
                subject["kind"] == kind && subject["id"] == id.as_str()
            });
            let locations = resolved.map(|r| r.locations.clone()).unwrap_or_default();
            if !locations.is_empty() {
                lines.push(format!("{kind} {id} = resolved {}", render(&locations)));
                continue;
            }
            let needle = format!("{kind} unit '{id}' did not resolve");
            let message = outcome
                .index
                .diagnostics
                .warnings
                .iter()
                .map(|d| d.message.as_str())
                .find(|m| m.ends_with(&needle) || m.contains(&format!("{needle} (")))
                .unwrap_or_else(|| panic!("no diagnostic for {kind} {id}"));
            let tail = &message[message.find(&needle).unwrap() + needle.len()..];
            lines.push(format!(
                "{kind} {id} ={}",
                if tail.is_empty() { " unresolved" } else { tail }
            ));
        }
    }
    lines
}

fn render(locations: &[spec_spine_types::ResolvedLocation]) -> String {
    locations
        .iter()
        .map(|l| match l.span {
            Some(span) => format!("{}:{}-{}", l.file, span.start_line, span.end_line),
            None => l.file.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(feature = "symbol-resolution")]
mod resolved {
    use super::*;
    use spec_spine_core::symbols::{Lookup, Namespace, Resolver, build_resolver};
    use spec_spine_types::{
        ContentDirtyState, ContentOmissionReason, ContentProjection, ContentRequest,
        ContentSnapshot, ContentSnapshotBinding,
    };

    fn resolver(root: &Path) -> Resolver {
        let cfg = Config::default();
        let discovered = spec_spine_core::manifest::discover(&cfg, root);
        build_resolver(
            root,
            &discovered.packages,
            &cfg.index.resolver_exclusions,
            &cfg.layout,
        )
    }

    fn outcome(lookup: &Lookup) -> String {
        match lookup {
            Lookup::Resolved(locations) => format!("resolved {}", render(locations)),
            Lookup::Unsupported(reason) => format!("unsupported: {reason}"),
            Lookup::Unknown(reason) => format!("unknown: {reason}"),
            Lookup::Unresolved => "unresolved".into(),
        }
    }

    /// The whole map: every declared id's outcome in each namespace, every
    /// recorded site, and every unknown scope.
    fn matrix(root: &Path) -> String {
        let r = resolver(root);
        let mut out = String::new();
        for (ns, ids) in [
            (Namespace::Symbol, SYMBOLS),
            (Namespace::Module, MODULES),
            (Namespace::Test, TESTS),
        ] {
            for id in ids {
                let id = expand(id);
                out.push_str(&format!(
                    "{} {id} = {}\n",
                    ns.as_str(),
                    outcome(&r.lookup(ns, &id))
                ));
            }
        }
        for (ns, id, reason) in r.unsupported_sites() {
            out.push_str(&format!("site {} {id} = {reason}\n", ns.as_str()));
        }
        for (scope, reason) in r.unknown_scopes() {
            out.push_str(&format!("scope {scope} = {reason}\n"));
        }
        out.replace(&deep_path(), "{DEEP}")
    }

    #[test]
    fn the_matrix_is_the_golden() {
        let repo = fixture();
        let actual = matrix(repo.path());
        if std::env::var_os("SPEC_SPINE_PRINT_MATRIX").is_some() {
            println!("{actual}");
        }
        assert_eq!(actual, GOLDEN, "the resolver matrix moved");
    }

    #[test]
    fn every_legacy_id_keeps_its_0_28_0_locations() {
        let repo = fixture();
        let r = resolver(repo.path());
        for line in LEGACY_GOLDEN.lines() {
            let (head, locations) = line.split_once(" = ").unwrap();
            let (kind, id) = head.split_once(' ').unwrap();
            let ns = if kind == "module" {
                Namespace::Module
            } else {
                Namespace::Symbol
            };
            assert_eq!(
                outcome(&r.lookup(ns, &expand(id))),
                format!("resolved {locations}"),
                "legacy {kind} {id} moved"
            );
        }
    }

    #[test]
    fn index_messages_name_the_outcome_and_leave_plain_unresolved_alone() {
        let repo = fixture();
        let actual = index_outcomes(repo.path()).join("\n");
        assert!(actual.contains("symbol demo::outer::nested = (unsupported: nested-item)"));
        assert!(
            actual.contains(
                "symbol demo::generated::macro_generated = (unknown: item-macro-in-scope)"
            )
        );
        assert!(actual.contains("symbol demo::absent = unresolved"));
        // Byte-identical to 0.28.0: no suffix, nothing after the verb.
        let outcome = index(&Config::default(), repo.path()).unwrap();
        assert!(
            outcome
                .index
                .diagnostics
                .warnings
                .iter()
                .any(|d| d.code == "W-001"
                    && d.message
                        == "spec '001-matrix' symbol unit 'nothing::here' did not resolve")
        );
        // The index agrees with the lookups for every declared unit.
        let lookups = matrix(repo.path());
        for line in index_outcomes(repo.path()) {
            let line = line.replace(&deep_path(), "{DEEP}");
            let (head, tail) = line.split_once(" = ").unwrap();
            let expected = tail
                .trim_start_matches('(')
                .trim_end_matches(')')
                .to_string();
            assert!(
                lookups.contains(&format!("{head} = {expected}\n")),
                "{line}"
            );
        }
    }

    #[test]
    fn crlf_and_bom_yield_the_lf_spans() {
        let repo = fixture();
        let r = resolver(repo.path());
        for name in ["first", "Twin", "Twin::m"] {
            let lf = r.resolve(Namespace::Symbol, &format!("lf_twin::{name}"));
            let crlf = r.resolve(Namespace::Symbol, &format!("crlf_twin::{name}"));
            assert_eq!(lf.len(), 1, "{name}");
            assert_eq!(
                lf.iter().map(|l| l.span).collect::<Vec<_>>(),
                crlf.iter().map(|l| l.span).collect::<Vec<_>>(),
                "{name}"
            );
        }
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

    /// Commit the fixture's index as `spec-spine index` writes it, so the
    /// content read finds a fresh ledger.
    fn commit_index(root: &Path) {
        let cfg = Config::default();
        let outcome = index(&cfg, root).unwrap();
        let dir = spec_spine_core::index_dir(&cfg, root);
        let (by_spec, by_package) = spec_spine_core::index_shard_files(&outcome.shards).unwrap();
        spec_spine_core::shard::sync_dir(&dir.join(spec_spine_core::shard::BY_SPEC_DIR), &by_spec)
            .unwrap();
        spec_spine_core::shard::sync_dir(
            &dir.join(spec_spine_core::shard::BY_PACKAGE_DIR),
            &by_package,
        )
        .unwrap();
        let (name, inputs) = spec_spine_core::index_inputs_file(&outcome.shards).unwrap();
        fs::write(dir.join(name), inputs).unwrap();
    }

    /// One selector's result: the item's span and content, or the omission.
    fn select(root: &Path, kind: &str, id: &str, projection: &str) -> String {
        let request: ContentRequest = serde_json::from_value(serde_json::json!({
            "selectors": [{"kind": kind, "id": id, "projection": projection}]
        }))
        .unwrap();
        let response =
            spec_spine_core::selected_content(&Config::default(), root, &request, &snapshot())
                .unwrap();
        if let Some(item) = response.items.first() {
            return format!(
                "{}-{} {:?}",
                item.span.start_line, item.span.end_line, item.content
            );
        }
        let omission = &response.omissions[0];
        format!(
            "{} {}",
            serde_json::to_value(omission.reason)
                .unwrap()
                .as_str()
                .unwrap(),
            omission.message
        )
    }

    #[test]
    fn projections_follow_the_grammar_and_tests_select() {
        let repo = fixture();
        commit_index(repo.path());
        let root = repo.path();
        let cases = [
            // §3.7: every 0.28.0 documentation span is unchanged.
            (
                "symbol",
                "demo::docs::doc_then_attr",
                "documentation",
                "1-1 \"/// Line doc.\"",
            ),
            (
                "symbol",
                "demo::docs::multi_block",
                "documentation",
                "26-28 \"/**\\n * multi\\n */\"",
            ),
            // §3.7: a one-line block and an attribute are no longer called absent.
            (
                "symbol",
                "demo::docs::block_doc",
                "documentation",
                "5-5 \"/** block doc */\"",
            ),
            (
                "symbol",
                "demo::docs::attr_doc",
                "documentation",
                "unsupported-projection attribute documentation",
            ),
            (
                "symbol",
                "demo::docs::blank_separated",
                "documentation",
                "missing-content no attached documentation comments",
            ),
            // §3.7: a body sharing a line with signature text has neither projection.
            (
                "symbol",
                "demo::docs::multi_line_sig",
                "signature",
                "unsupported-projection the grammar does not expose a line-bounded signature",
            ),
            (
                "symbol",
                "demo::docs::multi_line_sig",
                "body",
                "unsupported-projection the grammar does not expose a line-bounded body",
            ),
            (
                "symbol",
                "demo::top_fn",
                "body",
                "unsupported-projection the grammar does not expose a line-bounded body",
            ),
            (
                "symbol",
                "demo::docs::allman",
                "signature",
                "21-21 \"pub fn allman()\"",
            ),
            (
                "symbol",
                "demo::docs::allman",
                "body",
                "22-24 \"{\\n    let _x = 1;\\n}\"",
            ),
            // §3.2: new constructs select.
            (
                "symbol",
                "demo::TopStruct::inherent_method",
                "full",
                "33-33 \"    pub fn inherent_method(&self) {}\"",
            ),
            (
                "symbol",
                "web::src::util::arrowFn",
                "body",
                "unsupported-projection the grammar does not expose a line-bounded body",
            ),
            // §3.4: a test selects its attribute run; documentation sits above it.
            (
                "test",
                "demo::tests::unit_test_in_src",
                "full",
                "121-125 \"    #[test]\\n    #[should_panic]\\n    fn unit_test_in_src() {\\n        panic!(\\\"x\\\");\\n    }\"",
            ),
            (
                "test",
                "demo::tests::unit_test_in_src",
                "documentation",
                "120-120 \"    /// Documented test.\"",
            ),
            (
                "test",
                "demo::tests::unit_test_in_src",
                "body",
                "unsupported-projection the grammar does not expose a line-bounded body",
            ),
            (
                "test",
                "demo/tests/integration::integration_test_fn",
                "full",
                "1-2 \"#[test]\\nfn integration_test_fn() {}\"",
            ),
            (
                "test",
                "demo::tests::helper_not_test",
                "full",
                "missing-content test 'demo::tests::helper_not_test' is not a test function",
            ),
            (
                "test",
                "demo::absent",
                "full",
                "missing-content test 'demo::absent' does not resolve",
            ),
            // §3.5: the outcomes map to omission reasons, each naming its token.
            (
                "test",
                "demo::tests::tokio_test",
                "full",
                "unsupported-selector test 'demo::tests::tokio_test' is unsupported: framework-test-attribute",
            ),
            (
                "test",
                "web::src::util::does a thing",
                "full",
                "unsupported-selector test 'web::src::util::does a thing' is unsupported: typescript-test",
            ),
            (
                "symbol",
                "demo::generated::macro_generated",
                "full",
                "indeterminate-selector symbol 'demo::generated::macro_generated' is unknown: item-macro-in-scope",
            ),
            (
                "module",
                "demo/tests/common/mod",
                "full",
                "unsupported-selector module 'demo/tests/common/mod' is unsupported: unrooted-test-file",
            ),
            (
                "symbol",
                "demo::absent",
                "full",
                "missing-content symbol 'demo::absent' does not resolve",
            ),
        ];
        let mut failures = Vec::new();
        for (kind, id, projection, expected) in cases {
            let actual = select(root, kind, id, projection);
            if actual != expected {
                failures.push(format!(
                    "{kind} {id} {projection}:\n  expected {expected}\n  actual   {actual}"
                ));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
        let request: ContentRequest = serde_json::from_value(serde_json::json!({
            "selectors": [{"kind": "symbol", "id": "demo::cfg_twin"}]
        }))
        .unwrap();
        assert!(
            spec_spine_core::selected_content(&Config::default(), root, &request, &snapshot())
                .is_err(),
            "row 13: a two-location id still refuses"
        );
        let _ = (
            ContentOmissionReason::IndeterminateSelector,
            ContentProjection::Full,
        );
    }
}

/// Feature off (spec 025): every symbol and module unit is `unknown` /
/// `resolver-disabled`, and the crate builds without tree-sitter.
#[cfg(not(feature = "symbol-resolution"))]
#[test]
fn without_the_resolver_every_lookup_is_unknown() {
    use spec_spine_core::symbols::{Lookup, Namespace, Resolver};
    let repo = fixture();
    for line in index_outcomes(repo.path()) {
        assert!(line.ends_with(" = (unknown: resolver-disabled)"), "{line}");
    }
    for ns in [Namespace::Symbol, Namespace::Module, Namespace::Test] {
        assert_eq!(
            Resolver::disabled().lookup(ns, "demo::top_fn"),
            Lookup::Unknown("resolver-disabled")
        );
    }
}
/// The full expected map (§3.9): every declared id's outcome per namespace,
/// every recorded unsupported site, and every unknown scope. `{DEEP}` stands
/// for [`deep_path`].
#[cfg_attr(not(feature = "symbol-resolution"), allow(dead_code))]
const GOLDEN: &str = "\
symbol demo::top_fn = resolved demo/src/lib.rs:6-6\n\
symbol demo::TopStruct = resolved demo/src/lib.rs:8-10\n\
symbol demo::TopEnum = resolved demo/src/lib.rs:12-15\n\
symbol demo::TopUnion = resolved demo/src/lib.rs:17-19\n\
symbol demo::TopTrait = resolved demo/src/lib.rs:21-28\n\
symbol demo::Generic = resolved demo/src/lib.rs:54-54\n\
symbol demo::thing = resolved demo/src/lib.rs:3-3 demo/src/lib.rs:71-71\n\
symbol demo::sub = resolved demo/src/lib.rs:1-1\n\
symbol demo::docs = resolved demo/src/lib.rs:2-2\n\
symbol demo::inline_mod = resolved demo/src/lib.rs:77-82\n\
symbol demo::outer = resolved demo/src/lib.rs:84-86\n\
symbol demo::generated = resolved demo/src/lib.rs:99-102\n\
symbol demo::cfg_twin = resolved demo/src/lib.rs:105-105 demo/src/lib.rs:107-107\n\
symbol demo::top_level_test = resolved demo/src/lib.rs:116-116\n\
symbol demo::tests = resolved demo/src/lib.rs:119-134\n\
symbol demo::deep = resolved demo/src/lib.rs:135-203\n\
symbol demo::thing::run = resolved demo/src/thing.rs:1-1\n\
symbol demo::sub::Helper = resolved demo/src/sub.rs:1-1\n\
symbol demo::sub::helper = resolved demo/src/sub.rs:3-3\n\
symbol demo::broken::ok = resolved demo/src/broken.rs:1-1\n\
symbol demo::docs::doc_then_attr = resolved demo/src/docs.rs:3-3\n\
symbol demo::docs::block_doc = resolved demo/src/docs.rs:6-6\n\
symbol demo::docs::attr_doc = resolved demo/src/docs.rs:9-9\n\
symbol demo::docs::blank_separated = resolved demo/src/docs.rs:13-13\n\
symbol demo::docs::multi_line_sig = resolved demo/src/docs.rs:15-19\n\
symbol demo::docs::allman = resolved demo/src/docs.rs:21-24\n\
symbol demo::docs::multi_block = resolved demo/src/docs.rs:29-29\n\
symbol declared::d = resolved declared/src/lib.rs:1-1\n\
symbol lf_twin::first = resolved lf_twin/src/lib.rs:1-1\n\
symbol lf_twin::Twin = resolved lf_twin/src/lib.rs:7-7\n\
symbol crlf_twin::first = resolved crlf_twin/src/lib.rs:1-1\n\
symbol crlf_twin::Twin = resolved crlf_twin/src/lib.rs:7-7\n\
symbol web::src::util::formatDate = resolved web/src/util.ts:1-1\n\
symbol web::src::util::single = resolved web/src/util.ts:3-3\n\
symbol web::src::util::internalFn = resolved web/src/util.ts:4-4\n\
symbol web::src::util::Helper = resolved web/src/util.ts:5-11\n\
symbol web::src::util::Shape = resolved web/src/util.ts:12-14\n\
symbol web::src::comp::After = resolved web/src/comp.tsx:1-1\n\
symbol web::src::comp::Comp = resolved web/src/comp.tsx:2-4\n\
symbol demo::inline_mod::in_inline = resolved demo/src/lib.rs:78-78\n\
symbol demo::inline_mod::deeper = resolved demo/src/lib.rs:79-81\n\
symbol demo::inline_mod::deeper::deepest = resolved demo/src/lib.rs:80-80\n\
symbol demo::tests::unit_test_in_src = resolved demo/src/lib.rs:123-125\n\
symbol demo::tests::helper_not_test = resolved demo/src/lib.rs:133-133\n\
symbol demo::tests::tokio_test = resolved demo/src/lib.rs:128-128\n\
symbol demo::TopStruct::ASSOC = resolved demo/src/lib.rs:31-31\n\
symbol demo::TopStruct::Alias = resolved demo/src/lib.rs:32-32\n\
symbol demo::TopStruct::inherent_method = resolved demo/src/lib.rs:33-33\n\
symbol demo::Generic::gen_method = resolved demo/src/lib.rs:57-57\n\
symbol demo::<TopStruct as TopTrait>::trait_method = resolved demo/src/lib.rs:37-37\n\
symbol demo::<TopStruct as TopTrait>::TC = resolved demo/src/lib.rs:38-38\n\
symbol demo::<TopStruct as TopTrait>::Out = resolved demo/src/lib.rs:39-39\n\
symbol demo::<TopStruct as std::fmt::Display>::fmt = resolved demo/src/lib.rs:43-45\n\
symbol demo::<TopStruct as From<u8>>::from = resolved demo/src/lib.rs:49-51\n\
symbol demo::TopTrait::trait_method = resolved demo/src/lib.rs:22-22\n\
symbol demo::TopTrait::provided = resolved demo/src/lib.rs:23-25\n\
symbol demo::TopTrait::TC = resolved demo/src/lib.rs:26-26\n\
symbol demo::TopTrait::Out = resolved demo/src/lib.rs:27-27\n\
symbol demo::make_fn! = resolved demo/src/lib.rs:88-92\n\
symbol demo::exported_macro! = resolved demo/src/lib.rs:95-97\n\
symbol demo::generated::written = resolved demo/src/lib.rs:101-101\n\
symbol demo/tests/integration::integration_test_fn = resolved demo/tests/integration.rs:2-2\n\
symbol demo/tests/integration::helper = resolved demo/tests/integration.rs:4-4\n\
symbol demo/tests/integration::inner = resolved demo/tests/integration.rs:6-9\n\
symbol demo/tests/integration::inner::inner_test = resolved demo/tests/integration.rs:8-8\n\
symbol demo/tests/dirtest::in_dir = resolved demo/tests/dirtest/main.rs:4-4\n\
symbol demo/tests/dirtest::support = resolved demo/tests/dirtest/main.rs:1-1\n\
symbol lf_twin::Twin::m = resolved lf_twin/src/lib.rs:4-4\n\
symbol crlf_twin::Twin::m = resolved crlf_twin/src/lib.rs:4-4\n\
symbol web::src::util::Helper::run = resolved web/src/util.ts:6-6\n\
symbol web::src::util::Helper::make = resolved web/src/util.ts:7-7\n\
symbol web::src::util::arrowFn = resolved web/src/util.ts:15-17\n\
symbol web::src::util::counter = resolved web/src/util.ts:18-18\n\
symbol web::src::util::legacyVar = resolved web/src/util.ts:19-19\n\
symbol demo::deep::{DEEP}::n32 = resolved demo/src/lib.rs:167-171\n\
symbol demo::outer::nested = unsupported: nested-item\n\
symbol demo::<&Generic<u8> as TopTrait>::trait_method = unsupported: impl-type-form\n\
symbol demo::sub::Helper::scoped = unsupported: impl-type-form\n\
symbol demo::reexported = unsupported: re-export\n\
symbol web::src::util::renamed = unsupported: re-export\n\
symbol demo::TopStruct::field = unsupported: member\n\
symbol demo::TopEnum::A = unsupported: member\n\
symbol demo::TopUnion::a = unsupported: member\n\
symbol web::src::util::Helper::field = unsupported: member\n\
symbol web::src::util::Shape::size = unsupported: member\n\
symbol demo::foreign_fn = unsupported: foreign-item\n\
symbol demo/tests/common/mod::shared = unsupported: unrooted-test-file\n\
symbol demo/tests/dirtest/support::support = unsupported: unrooted-test-file\n\
symbol declared/tests/x::t = unsupported: declared-test-targets\n\
symbol web::src::util::NS = unsupported: ts-namespace\n\
symbol web::src::util::NS::inNs = unsupported: ts-namespace\n\
symbol web::src::util::a = unsupported: ts-destructuring\n\
symbol web::src::util::Helper::#priv = unsupported: ts-computed-name\n\
symbol web::src::comp::Comp::render = unsupported: tsx-grammar\n\
symbol web::src::comp::tsxConst = unsupported: tsx-grammar\n\
symbol demo::generated::macro_generated = unknown: item-macro-in-scope\n\
symbol demo::broken::missing = unknown: parse-error-in-scope\n\
symbol demo::deep::{DEEP}::n32::n33 = unknown: nesting-limit\n\
symbol demo::deep::{DEEP}::n32::n33::leaf = unknown: nesting-limit\n\
symbol demo::TopStruct::absent = unresolved\n\
symbol demo::absent = unresolved\n\
symbol nothing::here = unresolved\n\
module demo = resolved demo/src/lib.rs\n\
module demo::sub = resolved demo/src/sub.rs\n\
module demo::docs = resolved demo/src/docs.rs\n\
module demo::thing = resolved demo/src/thing.rs\n\
module demo::broken = resolved demo/src/broken.rs\n\
module demo::inline_mod = resolved demo/src/lib.rs:77-82\n\
module demo::generated = resolved demo/src/lib.rs:99-102\n\
module demo::tests = resolved demo/src/lib.rs:119-134\n\
module demo::deep = resolved demo/src/lib.rs:135-203\n\
module demo::inline_mod::deeper = resolved demo/src/lib.rs:79-81\n\
module demo::deep::{DEEP}::n32 = resolved demo/src/lib.rs:167-171\n\
module demo/tests/integration = resolved demo/tests/integration.rs\n\
module demo/tests/integration::inner = resolved demo/tests/integration.rs:6-9\n\
module demo/tests/dirtest = resolved demo/tests/dirtest/main.rs\n\
module demo/tests/common/mod = unsupported: unrooted-test-file\n\
module demo/tests/dirtest/support = unsupported: unrooted-test-file\n\
module declared/tests/x = unsupported: declared-test-targets\n\
module demo::deep::{DEEP}::n32::n33 = unknown: nesting-limit\n\
module demo::absent = unresolved\n\
test demo::top_level_test = resolved demo/src/lib.rs:115-116\n\
test demo::tests::unit_test_in_src = resolved demo/src/lib.rs:121-125\n\
test demo::tests::tokio_test = unsupported: framework-test-attribute\n\
test demo::tests::test_with_args = unsupported: framework-test-attribute\n\
test demo::tests::helper_not_test = unresolved\n\
test demo/tests/integration::integration_test_fn = resolved demo/tests/integration.rs:1-2\n\
test demo/tests/integration::inner::inner_test = resolved demo/tests/integration.rs:7-8\n\
test demo/tests/dirtest::in_dir = resolved demo/tests/dirtest/main.rs:3-4\n\
test demo/tests/common/mod::shared = unsupported: unrooted-test-file\n\
test declared/tests/x::t = unsupported: declared-test-targets\n\
test web::src::util::does a thing = unsupported: typescript-test\n\
test demo::outer::nested = unsupported: nested-item\n\
test demo::generated::macro_generated = unknown: item-macro-in-scope\n\
test demo::absent = unresolved\n\
site symbol declared/tests/x::t = declared-test-targets\n\
site symbol demo/tests/common/mod::shared = unrooted-test-file\n\
site symbol demo/tests/dirtest/support::support = unrooted-test-file\n\
site symbol demo::<&Generic<u8> as TopTrait>::Out = impl-type-form\n\
site symbol demo::<&Generic<u8> as TopTrait>::TC = impl-type-form\n\
site symbol demo::<&Generic<u8> as TopTrait>::trait_method = impl-type-form\n\
site symbol demo::TopEnum::A = member\n\
site symbol demo::TopEnum::B = member\n\
site symbol demo::TopStruct::field = member\n\
site symbol demo::TopUnion::a = member\n\
site symbol demo::foreign_fn = foreign-item\n\
site symbol demo::outer::nested = nested-item\n\
site symbol demo::reexported = re-export\n\
site symbol demo::sub::Helper::scoped = impl-type-form\n\
site symbol demo::thing::run = legacy-collision\n\
site symbol web::src::comp::Comp::render = tsx-grammar\n\
site symbol web::src::comp::tsxConst = tsx-grammar\n\
site symbol web::src::util::Helper::#priv = ts-computed-name\n\
site symbol web::src::util::Helper::field = member\n\
site symbol web::src::util::NS = ts-namespace\n\
site symbol web::src::util::NS::inNs = ts-namespace\n\
site symbol web::src::util::Shape::size = member\n\
site symbol web::src::util::a = ts-destructuring\n\
site symbol web::src::util::b = ts-destructuring\n\
site symbol web::src::util::renamed = re-export\n\
site module declared/tests/x = declared-test-targets\n\
site module demo/tests/common/mod = unrooted-test-file\n\
site module demo/tests/dirtest/support = unrooted-test-file\n\
site test declared/tests/x::t = declared-test-targets\n\
site test demo::tests::test_with_args = framework-test-attribute\n\
site test demo::tests::tokio_test = framework-test-attribute\n\
scope demo::broken = parse-error-in-scope\n\
scope demo::deep::{DEEP}::n32 = nesting-limit\n\
scope demo::generated = item-macro-in-scope\n";

/// The legacy set for this fixture, captured with the 0.29.0 release
/// (`spec-spine index` over this fixture): `<kind> <id> = <locations>`.
#[cfg_attr(not(feature = "symbol-resolution"), allow(dead_code))]
const LEGACY_GOLDEN: &str = "\
module demo = demo/src/lib.rs\n\
module demo::broken = demo/src/broken.rs\n\
module demo::deep = demo/src/lib.rs:135-203\n\
module demo::docs = demo/src/docs.rs\n\
module demo::generated = demo/src/lib.rs:99-102\n\
module demo::inline_mod = demo/src/lib.rs:77-82\n\
module demo::sub = demo/src/sub.rs\n\
module demo::tests = demo/src/lib.rs:119-134\n\
module demo::thing = demo/src/thing.rs\n\
symbol crlf_twin::Twin = crlf_twin/src/lib.rs:7-7\n\
symbol crlf_twin::first = crlf_twin/src/lib.rs:1-1\n\
symbol declared::d = declared/src/lib.rs:1-1\n\
symbol demo::Generic = demo/src/lib.rs:54-54\n\
symbol demo::TopEnum = demo/src/lib.rs:12-15\n\
symbol demo::TopStruct = demo/src/lib.rs:8-10\n\
symbol demo::TopTrait = demo/src/lib.rs:21-28\n\
symbol demo::TopUnion = demo/src/lib.rs:17-19\n\
symbol demo::broken::ok = demo/src/broken.rs:1-1\n\
symbol demo::cfg_twin = demo/src/lib.rs:105-105 demo/src/lib.rs:107-107\n\
symbol demo::deep = demo/src/lib.rs:135-203\n\
symbol demo::docs = demo/src/lib.rs:2-2\n\
symbol demo::docs::allman = demo/src/docs.rs:21-24\n\
symbol demo::docs::attr_doc = demo/src/docs.rs:9-9\n\
symbol demo::docs::blank_separated = demo/src/docs.rs:13-13\n\
symbol demo::docs::block_doc = demo/src/docs.rs:6-6\n\
symbol demo::docs::doc_then_attr = demo/src/docs.rs:3-3\n\
symbol demo::docs::multi_block = demo/src/docs.rs:29-29\n\
symbol demo::docs::multi_line_sig = demo/src/docs.rs:15-19\n\
symbol demo::generated = demo/src/lib.rs:99-102\n\
symbol demo::inline_mod = demo/src/lib.rs:77-82\n\
symbol demo::outer = demo/src/lib.rs:84-86\n\
symbol demo::sub = demo/src/lib.rs:1-1\n\
symbol demo::sub::Helper = demo/src/sub.rs:1-1\n\
symbol demo::sub::helper = demo/src/sub.rs:3-3\n\
symbol demo::tests = demo/src/lib.rs:119-134\n\
symbol demo::thing = demo/src/lib.rs:3-3 demo/src/lib.rs:71-71\n\
symbol demo::thing::run = demo/src/thing.rs:1-1\n\
symbol demo::top_fn = demo/src/lib.rs:6-6\n\
symbol demo::top_level_test = demo/src/lib.rs:116-116\n\
symbol lf_twin::Twin = lf_twin/src/lib.rs:7-7\n\
symbol lf_twin::first = lf_twin/src/lib.rs:1-1\n\
symbol web::src::comp::After = web/src/comp.tsx:1-1\n\
symbol web::src::comp::Comp = web/src/comp.tsx:2-4\n\
symbol web::src::util::Helper = web/src/util.ts:5-11\n\
symbol web::src::util::Shape = web/src/util.ts:12-14\n\
symbol web::src::util::formatDate = web/src/util.ts:1-1\n\
symbol web::src::util::internalFn = web/src/util.ts:4-4\n\
symbol web::src::util::single = web/src/util.ts:3-3
";
