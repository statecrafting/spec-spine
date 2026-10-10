//! Spec 163: the structural resolver's outcomes, one focused assertion per
//! construct. The full measured matrix (every §1.1 row as a golden, run on
//! each release triple) is the next change.

use std::fs;
use std::path::Path;

use spec_spine_core::index;
use spec_spine_types::Config;

const LIB: &str = r#"pub mod generated {
    make!(hidden);
    pub fn written() {}
}

pub mod broken;

pub struct Pair {
    pub width: u8,
}

pub fn outer() {
    fn nested() {}
}

pub trait Shape {
    fn area(&self) -> u8;
}

impl Shape for Pair {
    fn area(&self) -> u8 {
        self.width
    }
}

impl Pair {
    pub fn new() -> Self {
        Pair { width: 0 }
    }
}

macro_rules! make {
    ($n:ident) => {
        pub fn $n() {}
    };
}

pub mod outer_mod {
    pub mod inner {
        pub fn deep() {}
    }
}
"#;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// One crate, and a draft spec declaring `units` as symbol units.
fn fixture(units: &[&str]) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"demo\"]\n");
    write(
        root,
        "demo/Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "demo/src/lib.rs", LIB);
    write(
        root,
        "demo/src/broken.rs",
        "pub fn ok() {}\n\npub fn bad( {}\n",
    );
    let mut declared = String::new();
    for id in units {
        declared.push_str(&format!("  - {{ kind: symbol, id: {id:?} }}\n"));
    }
    write(
        root,
        "specs/001-units/spec.md",
        &format!(
            "---\nid: \"001-units\"\ntitle: \"Units\"\nstatus: draft\ncreated: \"2026-10-10\"\nimplementation: pending\nsummary: \"units\"\nestablishes:\n{declared}---\n# Units\n"
        ),
    );
    tmp
}

#[cfg(feature = "symbol-resolution")]
mod resolved {
    use super::*;
    use spec_spine_core::symbols::{Lookup, Namespace, Resolver, build_resolver, reason};
    use spec_spine_types::{LineSpan, ResolvedLocation};

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

    fn at(start: usize, end: usize) -> Lookup {
        Lookup::Resolved(vec![ResolvedLocation {
            file: "demo/src/lib.rs".into(),
            span: Some(LineSpan::new(start, end)),
        }])
    }

    #[test]
    fn each_construct_answers_its_outcome() {
        let repo = fixture(&[]);
        let r = resolver(repo.path());
        let symbol = |id: &str| r.lookup(Namespace::Symbol, id);
        // §3.2: nested items, impl and trait members, macros.
        assert_eq!(symbol("demo::outer_mod::inner::deep"), at(40, 40));
        assert_eq!(symbol("demo::Pair::new"), at(27, 29));
        assert_eq!(symbol("demo::<Pair as Shape>::area"), at(21, 23));
        assert_eq!(symbol("demo::Shape::area"), at(17, 17));
        assert_eq!(symbol("demo::make!"), at(32, 36));
        assert_eq!(symbol("demo::generated::written"), at(3, 3));
        assert_eq!(
            r.lookup(Namespace::Module, "demo::outer_mod::inner"),
            at(39, 41)
        );
        // Legacy ids keep their 0.28.0 spans (I-1).
        assert_eq!(symbol("demo::Pair"), at(8, 10));
        assert_eq!(symbol("demo::outer"), at(12, 14));
        // §3.5: unsupported sites and unknown scopes.
        assert_eq!(
            symbol("demo::outer::nested"),
            Lookup::Unsupported(reason::NESTED_ITEM)
        );
        assert_eq!(
            symbol("demo::Pair::width"),
            Lookup::Unsupported(reason::MEMBER)
        );
        assert_eq!(
            symbol("demo::generated::hidden"),
            Lookup::Unknown(reason::ITEM_MACRO_IN_SCOPE)
        );
        assert_eq!(
            symbol("demo::broken::missing"),
            Lookup::Unknown(reason::PARSE_ERROR_IN_SCOPE)
        );
        assert_eq!(symbol("demo::absent"), Lookup::Unresolved);
    }

    #[test]
    fn index_messages_name_the_outcome_and_leave_plain_unresolved_alone() {
        let repo = fixture(&[
            "demo::outer::nested",
            "demo::generated::hidden",
            "demo::absent",
        ]);
        let outcome = index(&Config::default(), repo.path()).unwrap();
        let messages: Vec<&str> = outcome
            .index
            .diagnostics
            .warnings
            .iter()
            .filter(|d| d.code == "W-001")
            .map(|d| d.message.as_str())
            .collect();
        for expected in [
            "spec '001-units' symbol unit 'demo::outer::nested' did not resolve (unsupported: nested-item)",
            "spec '001-units' symbol unit 'demo::generated::hidden' did not resolve (unknown: item-macro-in-scope)",
            // Byte-identical to 0.28.0: nothing after the verb.
            "spec '001-units' symbol unit 'demo::absent' did not resolve",
        ] {
            assert!(messages.contains(&expected), "{expected}\n{messages:?}");
        }
    }
}

/// Feature off (spec 025): every lookup is `unknown` / `resolver-disabled`,
/// and the index message says so.
#[cfg(not(feature = "symbol-resolution"))]
#[test]
fn without_the_resolver_every_lookup_is_unknown() {
    use spec_spine_core::symbols::{Lookup, Namespace, Resolver};
    let repo = fixture(&["demo::Pair"]);
    let outcome = index(&Config::default(), repo.path()).unwrap();
    assert!(outcome.index.diagnostics.warnings.iter().any(|d| d.message
        == "spec '001-units' symbol unit 'demo::Pair' did not resolve (unknown: resolver-disabled)"));
    assert_eq!(
        Resolver::disabled().lookup(Namespace::Symbol, "demo::Pair"),
        Lookup::Unknown("resolver-disabled")
    );
}
