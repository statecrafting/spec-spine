//! Structural resolution (spec 004 §3.3, spec 016, spec 163): build a
//! deterministic index from `::`-qualified ids to physical `(file, line-span)`
//! locations via tree-sitter, for Rust (`.rs`) and TypeScript (`.ts`/`.tsx`).
//!
//! The construct matrix is spec 163 §3.2: items at any inline-module depth,
//! inline modules as modules, integration-test roots, impl and trait members,
//! `macro_rules!` definitions, TypeScript class methods and top-level
//! declarators, and Rust test functions bound to a bare `#[test]` (§3.4).
//! Everything the walk recognizes but does not support is recorded with a
//! closed reason, so a lookup answers `unsupported` or `unknown` instead of
//! looking like an absence (§3.5). The 0.28.0 construct set (top-level items of
//! `src/` files, top-level inline modules, file modules) is the legacy set: a
//! new construct never adds a location to one of its ids (§3.3). The
//! tree-sitter core and grammar crates are pinned exactly so spans are
//! identical across platforms.
//!
//! Resolution is syntactic (§3.10): no name resolution, macro expansion, `cfg`
//! evaluation, type inference, `#[path]` following, or dependency edges.
//!
//! The tree-sitter machinery (the `build_*` functions and their parse helpers) is
//! gated behind the default `symbol-resolution` feature (spec 025). The
//! [`Resolver`] type and its lookups carry no tree-sitter dependency and are
//! always compiled; built without the feature, every lookup answers
//! `unknown` / `resolver-disabled`.

use std::collections::{BTreeMap, BTreeSet};

use spec_spine_types::ResolvedLocation;

#[cfg(feature = "symbol-resolution")]
use std::fs;
#[cfg(feature = "symbol-resolution")]
use std::path::{Path, PathBuf};

#[cfg(feature = "symbol-resolution")]
use crate::pathutil::{is_excluded, rel_posix};
#[cfg(feature = "symbol-resolution")]
use spec_spine_types::{LayoutConfig, LineSpan, PackageKind, PackageRecord};
#[cfg(feature = "symbol-resolution")]
use tree_sitter::{Language, Node, Parser};

/// The closed reason tokens of spec 163 §3.5.
pub mod reason {
    /// An item inside a `fn` body.
    pub const NESTED_ITEM: &str = "nested-item";
    /// An impl self type that is not a bare or generic type identifier.
    pub const IMPL_TYPE_FORM: &str = "impl-type-form";
    /// A named `use` / `export {…}` binding.
    pub const RE_EXPORT: &str = "re-export";
    /// An enum variant, struct or union field, TS class field or interface member.
    pub const MEMBER: &str = "member";
    /// An item in an `extern` block.
    pub const FOREIGN_ITEM: &str = "foreign-item";
    /// A fn whose test attribute is not the bare `#[test]` (test ids only).
    pub const FRAMEWORK_TEST_ATTRIBUTE: &str = "framework-test-attribute";
    /// A `.rs` file under `tests/` that is not an integration-test root.
    pub const UNROOTED_TEST_FILE: &str = "unrooted-test-file";
    /// A package whose manifest declares its own test targets.
    pub const DECLARED_TEST_TARGETS: &str = "declared-test-targets";
    /// A `test` id naming an npm package; string-named tests have no identity.
    pub const TYPESCRIPT_TEST: &str = "typescript-test";
    /// A TypeScript namespace or a declaration inside one.
    pub const TS_NAMESPACE: &str = "ts-namespace";
    /// A TypeScript destructuring declarator.
    pub const TS_DESTRUCTURING: &str = "ts-destructuring";
    /// A TypeScript computed or private member name.
    pub const TS_COMPUTED_NAME: &str = "ts-computed-name";
    /// A §3.2 new construct in a `.tsx` file.
    pub const TSX_GRAMMAR: &str = "tsx-grammar";
    /// A new construct whose id the legacy set already holds (§3.3).
    pub const LEGACY_COLLISION: &str = "legacy-collision";
    /// The scope holds an item-position macro invocation.
    pub const ITEM_MACRO_IN_SCOPE: &str = "item-macro-in-scope";
    /// The scope's node contains a grammar error.
    pub const PARSE_ERROR_IN_SCOPE: &str = "parse-error-in-scope";
    /// The scope is deeper than the walk descends (§3.1).
    pub const NESTING_LIMIT: &str = "nesting-limit";
    /// Built without the `symbol-resolution` feature (spec 025).
    pub const RESOLVER_DISABLED: &str = "resolver-disabled";
}

/// How deep the walk descends into containers (spec 163 §3.1).
pub const MAX_CONTAINER_DEPTH: usize = 32;

/// Which identity namespace a lookup reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Namespace {
    Symbol,
    Module,
    Test,
}

impl Namespace {
    /// The lowercase name a diagnostic or golden uses.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Symbol => "symbol",
            Self::Module => "module",
            Self::Test => "test",
        }
    }
}

/// The one outcome of a lookup, in spec 163 §3.5's precedence order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lookup {
    Resolved(Vec<ResolvedLocation>),
    Unsupported(&'static str),
    Unknown(&'static str),
    Unresolved,
}

impl Lookup {
    /// The suffix an unresolved unit's index diagnostic carries: ` (<outcome>:
    /// <reason>)` for `unsupported` and `unknown`, and nothing otherwise, so a
    /// plain unresolved message stays byte-identical to 0.28.0 (§3.5).
    pub fn diagnostic_suffix(&self) -> String {
        match self {
            Self::Unsupported(reason) => format!(" (unsupported: {reason})"),
            Self::Unknown(reason) => format!(" (unknown: {reason})"),
            Self::Resolved(_) | Self::Unresolved => String::new(),
        }
    }

    /// The locations, empty for every outcome but `Resolved`.
    pub fn locations(&self) -> Vec<ResolvedLocation> {
        match self {
            Self::Resolved(locations) => locations.clone(),
            _ => Vec::new(),
        }
    }
}

/// The structural index: three id namespaces, the recorded unsupported sites,
/// and the scopes a lookup consults for `unknown`.
#[derive(Debug, Default)]
pub struct Resolver {
    symbols: BTreeMap<String, Vec<ResolvedLocation>>,
    modules: BTreeMap<String, Vec<ResolvedLocation>>,
    tests: BTreeMap<String, Vec<ResolvedLocation>>,
    sites: BTreeMap<(Namespace, String), &'static str>,
    scopes: BTreeSet<String>,
    unknown_scopes: BTreeMap<String, &'static str>,
    npm_packages: BTreeSet<String>,
    disabled: bool,
}

impl Resolver {
    /// The resolver a build without `symbol-resolution` answers with: every
    /// lookup is `unknown` / `resolver-disabled`.
    pub fn disabled() -> Self {
        Self {
            disabled: true,
            ..Self::default()
        }
    }

    /// Resolve `id` in `ns` under spec 163 §3.5's precedence: resolved, then
    /// unsupported, then unknown, then unresolved.
    pub fn lookup(&self, ns: Namespace, id: &str) -> Lookup {
        if self.disabled {
            return Lookup::Unknown(reason::RESOLVER_DISABLED);
        }
        let map = match ns {
            Namespace::Symbol => &self.symbols,
            Namespace::Module => &self.modules,
            Namespace::Test => &self.tests,
        };
        if let Some(locations) = map.get(id).filter(|l| !l.is_empty()) {
            return Lookup::Resolved(locations.clone());
        }
        if let Some(reason) = self.sites.get(&(ns, id.to_string())) {
            return Lookup::Unsupported(reason);
        }
        if ns == Namespace::Test {
            let segments = split_id(id);
            if segments
                .first()
                .is_some_and(|first| self.npm_packages.contains(*first))
            {
                return Lookup::Unsupported(reason::TYPESCRIPT_TEST);
            }
            // A test id is a symbol id (§3.4), so a site the walk recorded for
            // the function (nested, unrooted, declared targets) answers too.
            if let Some(reason) = self.sites.get(&(Namespace::Symbol, id.to_string())) {
                return Lookup::Unsupported(reason);
            }
        }
        let segments = split_id(id);
        for end in (1..segments.len()).rev() {
            let scope = segments[..end].join("::");
            if self.scopes.contains(&scope) {
                return self
                    .unknown_scopes
                    .get(&scope)
                    .map_or(Lookup::Unresolved, |reason| Lookup::Unknown(reason));
            }
        }
        Lookup::Unresolved
    }

    /// Locations for `id` in `ns` (empty unless it resolves).
    pub fn resolve(&self, ns: Namespace, id: &str) -> Vec<ResolvedLocation> {
        self.lookup(ns, id).locations()
    }

    /// Every resolved id in `ns`, in order.
    pub fn ids(&self, ns: Namespace) -> Vec<&str> {
        let map = match ns {
            Namespace::Symbol => &self.symbols,
            Namespace::Module => &self.modules,
            Namespace::Test => &self.tests,
        };
        map.keys().map(String::as_str).collect()
    }

    /// Every recorded unsupported site as `(namespace, would-be id, reason)`,
    /// in order. A `legacy-collision` site's id resolves to its legacy owner.
    pub fn unsupported_sites(&self) -> Vec<(Namespace, &str, &'static str)> {
        self.sites
            .iter()
            .map(|((ns, id), reason)| (*ns, id.as_str(), *reason))
            .collect()
    }

    /// Every scope a lookup reports `unknown` under, with its reason.
    pub fn unknown_scopes(&self) -> Vec<(&str, &'static str)> {
        self.unknown_scopes
            .iter()
            .map(|(scope, reason)| (scope.as_str(), *reason))
            .collect()
    }
}

/// Split an id on `::` outside angle brackets, so a trait-impl segment such
/// as `<T as fmt::Display>` stays one segment (spec 163 D-10).
pub fn split_id(id: &str) -> Vec<&str> {
    let bytes = id.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => depth += 1,
            b'>' => depth = depth.saturating_sub(1),
            b':' if depth == 0 && bytes.get(i + 1) == Some(&b':') => {
                out.push(&id[start..i]);
                i += 2;
                start = i;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&id[start..]);
    out
}

// ===== building =====

/// Where a walked file's ids go: resolved, or recorded unsupported whole.
#[cfg(feature = "symbol-resolution")]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sink {
    /// A `src/` file: its top level is the legacy set.
    Source,
    /// An integration-test root: every id is a new construct.
    TestRoot,
    /// Every id is recorded unsupported with this reason.
    Unsupported(&'static str),
}

#[cfg(feature = "symbol-resolution")]
#[derive(Default)]
struct Builder {
    legacy_symbols: BTreeMap<String, Vec<ResolvedLocation>>,
    legacy_modules: BTreeMap<String, Vec<ResolvedLocation>>,
    new_symbols: BTreeMap<String, Vec<ResolvedLocation>>,
    new_modules: BTreeMap<String, Vec<ResolvedLocation>>,
    tests: BTreeMap<String, Vec<ResolvedLocation>>,
    sites: BTreeMap<(Namespace, String), &'static str>,
    scopes: BTreeSet<String>,
    unknown_scopes: BTreeMap<String, &'static str>,
    npm_packages: BTreeSet<String>,
}

#[cfg(feature = "symbol-resolution")]
fn unknown_rank(reason: &str) -> u8 {
    match reason {
        reason::NESTING_LIMIT => 0,
        reason::PARSE_ERROR_IN_SCOPE => 1,
        _ => 2,
    }
}

#[cfg(feature = "symbol-resolution")]
impl Builder {
    fn site(&mut self, ns: Namespace, id: String, why: &'static str) {
        self.sites.entry((ns, id)).or_insert(why);
    }

    fn unknown(&mut self, scope: &str, why: &'static str) {
        let slot = self.unknown_scopes.entry(scope.to_string()).or_insert(why);
        if unknown_rank(why) < unknown_rank(slot) {
            *slot = why;
        }
    }

    fn put(&mut self, ns: Namespace, legacy: bool, sink: Sink, id: String, loc: ResolvedLocation) {
        if let Sink::Unsupported(why) = sink {
            self.site(ns, id, why);
            return;
        }
        let map = match (ns, legacy) {
            (Namespace::Symbol, true) => &mut self.legacy_symbols,
            (Namespace::Symbol, false) => &mut self.new_symbols,
            (Namespace::Module, true) => &mut self.legacy_modules,
            (Namespace::Module, false) => &mut self.new_modules,
            (Namespace::Test, _) => &mut self.tests,
        };
        map.entry(id).or_default().push(loc);
    }

    fn finish(mut self) -> Resolver {
        let mut symbols = std::mem::take(&mut self.legacy_symbols);
        for (id, locations) in std::mem::take(&mut self.new_symbols) {
            match symbols.entry(id) {
                std::collections::btree_map::Entry::Occupied(taken) => {
                    let id = taken.key().clone();
                    self.site(Namespace::Symbol, id, reason::LEGACY_COLLISION);
                }
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert(locations);
                }
            }
        }
        let mut modules = std::mem::take(&mut self.legacy_modules);
        for (id, locations) in std::mem::take(&mut self.new_modules) {
            match modules.entry(id) {
                std::collections::btree_map::Entry::Occupied(taken) => {
                    let id = taken.key().clone();
                    self.site(Namespace::Module, id, reason::LEGACY_COLLISION);
                }
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert(locations);
                }
            }
        }
        let mut tests = std::mem::take(&mut self.tests);
        for map in [&mut symbols, &mut modules, &mut tests] {
            for locs in map.values_mut() {
                sort_locations(locs);
            }
        }
        Resolver {
            symbols,
            modules,
            tests,
            sites: self.sites,
            scopes: self.scopes,
            unknown_scopes: self.unknown_scopes,
            npm_packages: self.npm_packages,
            disabled: false,
        }
    }
}

#[cfg(feature = "symbol-resolution")]
fn sort_locations(locs: &mut Vec<ResolvedLocation>) {
    locs.sort_by(|a, b| {
        (a.file.as_str(), a.span.map(|s| s.start_line))
            .cmp(&(b.file.as_str(), b.span.map(|s| s.start_line)))
    });
    locs.dedup();
}

/// Build the structural index across all packages. Deterministic: packages,
/// files, containers and members are processed in sorted path and source
/// order, and every id's locations are sorted (spec 163 §3.9).
#[cfg(feature = "symbol-resolution")]
pub fn build_resolver(
    repo_root: &Path,
    packages: &[PackageRecord],
    exclusions: &[String],
    layout: &LayoutConfig,
) -> Resolver {
    let mut b = Builder::default();
    for pkg in packages {
        let pkg_dir = repo_root.join(&pkg.path);
        match pkg.kind {
            PackageKind::RustLib | PackageKind::RustBin | PackageKind::RustLibBin => {
                index_rust_package(&mut b, repo_root, pkg, &pkg_dir, exclusions, layout);
            }
            PackageKind::NpmPackage | PackageKind::NpmWorkspace => {
                b.npm_packages.insert(pkg.name.clone());
                index_ts_package(&mut b, repo_root, pkg, &pkg_dir, exclusions, layout);
            }
        }
    }
    b.finish()
}

// ===== Rust =====

/// The 0.28.0 item kinds: at the top level of a `src/` file they are the
/// legacy set (spec 163 §3.3).
#[cfg(feature = "symbol-resolution")]
const RUST_KINDS: &[&str] = &[
    "function_item",
    "struct_item",
    "enum_item",
    "union_item",
    "trait_item",
    "const_item",
    "static_item",
    "type_item",
    "mod_item",
];

/// Parse `src` (BOM stripped, which moves no row) with `language`.
#[cfg(feature = "symbol-resolution")]
fn parse(src: &str, language: Language) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    parser.parse(src.strip_prefix('\u{feff}').unwrap_or(src), None)
}

#[cfg(feature = "symbol-resolution")]
fn text<'a>(node: Node, src: &'a str) -> &'a str {
    node.utf8_text(src.as_bytes()).unwrap_or("")
}

#[cfg(feature = "symbol-resolution")]
fn strip_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[cfg(feature = "symbol-resolution")]
fn span_of(node: Node) -> LineSpan {
    LineSpan::new(node.start_position().row + 1, node.end_position().row + 1)
}

#[cfg(feature = "symbol-resolution")]
fn name_of<'a>(node: Node, src: &'a str) -> Option<&'a str> {
    node.child_by_field_name("name")
        .map(|n| text(n, src))
        .filter(|n| !n.is_empty())
}

/// Whether a package's manifest takes its test targets off the directory
/// convention (`autotests = false` or any `[[test]]` table, spec 163 §3.4).
#[cfg(feature = "symbol-resolution")]
fn declares_test_targets(pkg_dir: &Path) -> bool {
    let Ok(src) = fs::read_to_string(pkg_dir.join("Cargo.toml")) else {
        return false;
    };
    let Ok(doc) = toml::from_str::<toml::Value>(&src) else {
        return false;
    };
    doc.get("test").is_some_and(toml::Value::is_array)
        || doc
            .get("package")
            .and_then(|p| p.get("autotests"))
            .and_then(toml::Value::as_bool)
            == Some(false)
}

#[cfg(feature = "symbol-resolution")]
fn index_rust_package(
    b: &mut Builder,
    repo_root: &Path,
    pkg: &PackageRecord,
    pkg_dir: &Path,
    exclusions: &[String],
    layout: &LayoutConfig,
) {
    let crate_name = pkg.name.replace('-', "_");
    let src_dir = pkg_dir.join("src");
    for file in walk_files(&src_dir, &["rs"], repo_root, exclusions, layout) {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let module = rust_module_path(&src_dir, &file);
        index_rust_file(
            b,
            repo_root,
            &file,
            &content,
            &crate_name,
            &module,
            Sink::Source,
        );
    }

    // Integration-test roots (§3.1): `tests/<stem>.rs` and `tests/<dir>/main.rs`.
    let tests_dir = pkg_dir.join("tests");
    let declared = declares_test_targets(pkg_dir);
    for file in walk_files(&tests_dir, &["rs"], repo_root, exclusions, layout) {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let rel = file.strip_prefix(&tests_dir).unwrap_or(&file);
        let parts: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        let (target, rooted) = match parts.as_slice() {
            [stem] => (stem.trim_end_matches(".rs").to_string(), true),
            [dir, main] if main == "main.rs" => (dir.clone(), true),
            _ => (parts.join("/").trim_end_matches(".rs").to_string(), false),
        };
        let root = format!("{crate_name}/tests/{target}");
        let sink = if declared {
            Sink::Unsupported(reason::DECLARED_TEST_TARGETS)
        } else if rooted {
            Sink::TestRoot
        } else {
            Sink::Unsupported(reason::UNROOTED_TEST_FILE)
        };
        index_rust_file(b, repo_root, &file, &content, &root, &[], sink);
    }
}

/// Everything one walked Rust file needs to know about where it is.
#[cfg(feature = "symbol-resolution")]
struct RustFile<'a> {
    src: &'a str,
    rel: String,
    sink: Sink,
}

#[cfg(feature = "symbol-resolution")]
impl RustFile<'_> {
    fn loc(&self, span: LineSpan) -> ResolvedLocation {
        ResolvedLocation {
            file: self.rel.clone(),
            span: Some(span),
        }
    }
}

#[cfg(feature = "symbol-resolution")]
fn index_rust_file(
    b: &mut Builder,
    repo_root: &Path,
    file: &Path,
    content: &str,
    root: &str,
    module: &[String],
    sink: Sink,
) {
    let rel = rel_posix(repo_root, file);
    let file_mod_id = qualify_module(root, module);
    // File module: whole file (the crate root resolves to the bare crate name,
    // a test root to `<crate>/tests/<target>`).
    b.put(
        Namespace::Module,
        sink == Sink::Source,
        sink,
        file_mod_id.clone(),
        ResolvedLocation {
            file: rel.clone(),
            span: None,
        },
    );
    if !matches!(sink, Sink::Unsupported(_)) {
        b.scopes.insert(file_mod_id.clone());
    }
    let Some(tree) = parse(content, tree_sitter_rust::LANGUAGE.into()) else {
        return;
    };
    let f = RustFile {
        src: content.strip_prefix('\u{feff}').unwrap_or(content),
        rel,
        sink,
    };
    let root_node = tree.root_node();
    mark_scope(b, &f, root_node, &file_mod_id);
    walk_rust_items(b, &f, root_node, root, module, 0);
}

/// Record why a scope cannot be enumerated, if it cannot (§3.5).
#[cfg(feature = "symbol-resolution")]
fn mark_scope(b: &mut Builder, f: &RustFile, container: Node, scope: &str) {
    if matches!(f.sink, Sink::Unsupported(_)) {
        return;
    }
    if container.has_error() {
        b.unknown(scope, reason::PARSE_ERROR_IN_SCOPE);
    }
    let mut cursor = container.walk();
    for child in container.named_children(&mut cursor) {
        let invocation = child.kind() == "macro_invocation"
            || (child.kind() == "expression_statement"
                && child
                    .named_child(0)
                    .is_some_and(|n| n.kind() == "macro_invocation"));
        if invocation {
            b.unknown(scope, reason::ITEM_MACRO_IN_SCOPE);
        }
    }
}

/// Walk the items of one module-level container: a file's root, or an inline
/// `mod` body at `depth` (spec 163 §3.1, §3.2).
#[cfg(feature = "symbol-resolution")]
fn walk_rust_items(
    b: &mut Builder,
    f: &RustFile,
    container: Node,
    root: &str,
    module: &[String],
    depth: usize,
) {
    let top = depth == 0 && f.sink == Sink::Source;
    let mut cursor = container.walk();
    for child in container.named_children(&mut cursor) {
        let kind = child.kind();
        let Some(name) = name_of(child, f.src) else {
            match kind {
                "impl_item" => walk_impl(b, f, child, root, module, depth),
                "foreign_mod_item" => {
                    if let Some(body) = child.child_by_field_name("body") {
                        let mut c = body.walk();
                        for item in body.named_children(&mut c) {
                            if let Some(n) = name_of(item, f.src) {
                                b.site(
                                    Namespace::Symbol,
                                    qualify(root, module, n),
                                    reason::FOREIGN_ITEM,
                                );
                            }
                        }
                    }
                }
                "use_declaration" => {
                    for bound in use_bindings(child, f.src) {
                        b.site(
                            Namespace::Symbol,
                            qualify(root, module, &bound),
                            reason::RE_EXPORT,
                        );
                    }
                }
                _ => {}
            }
            continue;
        };
        let id = qualify(root, module, name);
        let span = span_of(child);
        match kind {
            "macro_definition" => {
                b.put(
                    Namespace::Symbol,
                    false,
                    f.sink,
                    format!("{id}!"),
                    f.loc(span),
                );
            }
            "mod_item" => {
                b.put(Namespace::Symbol, top, f.sink, id.clone(), f.loc(span));
                let Some(body) = child.child_by_field_name("body") else {
                    continue;
                };
                b.put(Namespace::Module, top, f.sink, id.clone(), f.loc(span));
                if !matches!(f.sink, Sink::Unsupported(_)) {
                    b.scopes.insert(id.clone());
                }
                if depth + 1 > MAX_CONTAINER_DEPTH {
                    b.unknown(&id, reason::NESTING_LIMIT);
                    continue;
                }
                mark_scope(b, f, body, &id);
                let mut path = module.to_vec();
                path.push(name.to_string());
                walk_rust_items(b, f, body, root, &path, depth + 1);
            }
            k if RUST_KINDS.contains(&k) => {
                b.put(Namespace::Symbol, top, f.sink, id.clone(), f.loc(span));
                match k {
                    "function_item" => {
                        test_identity(b, f, child, &id);
                        nested_items(b, child, &id, f.src, 0);
                    }
                    "struct_item" | "union_item" | "enum_item" => {
                        if !matches!(f.sink, Sink::Unsupported(_)) {
                            b.scopes.insert(id.clone());
                            if child.has_error() {
                                b.unknown(&id, reason::PARSE_ERROR_IN_SCOPE);
                            }
                        }
                        members(b, child, &id, f.src);
                    }
                    "trait_item" => walk_trait(b, f, child, &id, depth),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// Fields and variants are `member` sites (§3.5).
#[cfg(feature = "symbol-resolution")]
fn members(b: &mut Builder, item: Node, id: &str, src: &str) {
    let Some(body) = item.child_by_field_name("body") else {
        return;
    };
    let mut cursor = body.walk();
    for member in body.named_children(&mut cursor) {
        if matches!(member.kind(), "field_declaration" | "enum_variant")
            && let Some(name) = name_of(member, src)
        {
            b.site(Namespace::Symbol, format!("{id}::{name}"), reason::MEMBER);
        }
    }
}

/// The names a `use` declaration binds: the last segment of each named path,
/// or its alias. A glob binds no name.
#[cfg(feature = "symbol-resolution")]
fn use_bindings(node: Node, src: &str) -> Vec<String> {
    fn last_segment(node: Node, src: &str) -> Option<String> {
        match node.kind() {
            "identifier" | "self" | "crate" | "super" => Some(text(node, src).to_string()),
            "scoped_identifier" => node
                .child_by_field_name("name")
                .map(|n| text(n, src).to_string()),
            _ => None,
        }
    }
    fn collect(node: Node, src: &str, out: &mut Vec<String>, depth: usize) {
        if depth > 64 {
            return;
        }
        match node.kind() {
            "use_as_clause" => {
                if let Some(alias) = node.child_by_field_name("alias") {
                    out.push(text(alias, src).to_string());
                }
            }
            "use_list" => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    collect(child, src, out, depth + 1);
                }
            }
            "scoped_use_list" => {
                if let Some(list) = node.child_by_field_name("list") {
                    collect(list, src, out, depth + 1);
                }
            }
            "use_wildcard" => {}
            _ => {
                if let Some(name) = last_segment(node, src) {
                    out.push(name);
                }
            }
        }
    }
    let mut out = Vec::new();
    if let Some(argument) = node.child_by_field_name("argument") {
        collect(argument, src, &mut out, 0);
    }
    out
}

/// An impl block's members (§3.2): `<Type>::<name>` inherent,
/// `<Type as TraitPath>::<name>` for a trait impl, or `impl-type-form` sites
/// when the self type is anything but a bare or generic type identifier.
#[cfg(feature = "symbol-resolution")]
fn walk_impl(
    b: &mut Builder,
    f: &RustFile,
    item: Node,
    root: &str,
    module: &[String],
    depth: usize,
) {
    let Some(ty) = item.child_by_field_name("type") else {
        return;
    };
    let (segment, supported) = match ty.kind() {
        "type_identifier" => (text(ty, f.src).to_string(), true),
        "generic_type" => match ty.child_by_field_name("type") {
            Some(inner) if inner.kind() == "type_identifier" => {
                (text(inner, f.src).to_string(), true)
            }
            _ => (strip_ws(text(ty, f.src)), false),
        },
        _ => (strip_ws(text(ty, f.src)), false),
    };
    let segment = match item.child_by_field_name("trait") {
        Some(tr) => format!("<{segment} as {}>", strip_ws(text(tr, f.src))),
        None => segment,
    };
    let scope = qualify(root, module, &segment);
    let Some(body) = item.child_by_field_name("body") else {
        return;
    };
    if supported && !matches!(f.sink, Sink::Unsupported(_)) {
        b.scopes.insert(scope.clone());
        if depth + 1 > MAX_CONTAINER_DEPTH {
            b.unknown(&scope, reason::NESTING_LIMIT);
            return;
        }
        mark_scope(b, f, body, &scope);
    }
    let mut cursor = body.walk();
    for member in body.named_children(&mut cursor) {
        if !matches!(member.kind(), "function_item" | "const_item" | "type_item") {
            continue;
        }
        let Some(name) = name_of(member, f.src) else {
            continue;
        };
        let id = format!("{scope}::{name}");
        if supported {
            b.put(
                Namespace::Symbol,
                false,
                f.sink,
                id.clone(),
                f.loc(span_of(member)),
            );
        } else {
            b.site(Namespace::Symbol, id.clone(), reason::IMPL_TYPE_FORM);
        }
        if member.kind() == "function_item" {
            nested_items(b, member, &id, f.src, 0);
        }
    }
}

/// A trait body's members: provided and required `fn`, `const`, `type` (§3.2).
#[cfg(feature = "symbol-resolution")]
fn walk_trait(b: &mut Builder, f: &RustFile, item: Node, id: &str, depth: usize) {
    if !matches!(f.sink, Sink::Unsupported(_)) {
        b.scopes.insert(id.to_string());
    }
    let Some(body) = item.child_by_field_name("body") else {
        return;
    };
    if !matches!(f.sink, Sink::Unsupported(_)) {
        if depth + 1 > MAX_CONTAINER_DEPTH {
            b.unknown(id, reason::NESTING_LIMIT);
            return;
        }
        mark_scope(b, f, body, id);
    }
    let mut cursor = body.walk();
    for member in body.named_children(&mut cursor) {
        if !matches!(
            member.kind(),
            "function_item" | "function_signature_item" | "const_item" | "associated_type"
        ) {
            continue;
        }
        let Some(name) = name_of(member, f.src) else {
            continue;
        };
        let member_id = format!("{id}::{name}");
        b.put(
            Namespace::Symbol,
            false,
            f.sink,
            member_id.clone(),
            f.loc(span_of(member)),
        );
        if member.kind() == "function_item" {
            nested_items(b, member, &member_id, f.src, 0);
        }
    }
}

/// Items written inside a `fn` body are `nested-item` sites under the
/// function's id (§3.5); a nested `fn` nests further.
#[cfg(feature = "symbol-resolution")]
fn nested_items(b: &mut Builder, node: Node, fn_id: &str, src: &str, depth: usize) {
    if depth > 256 {
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let kind = child.kind();
        let is_item = RUST_KINDS.contains(&kind) || kind == "macro_definition";
        if depth > 0 && is_item {
            if let Some(name) = name_of(child, src) {
                let suffix = if kind == "macro_definition" { "!" } else { "" };
                let id = format!("{fn_id}::{name}{suffix}");
                b.site(Namespace::Symbol, id.clone(), reason::NESTED_ITEM);
                if kind == "function_item" {
                    nested_items(b, child, &id, src, 1);
                }
            }
            continue;
        }
        if depth == 0 && !matches!(kind, "block") {
            // The function's own signature, attributes and parameters.
            continue;
        }
        nested_items(b, child, fn_id, src, depth + 1);
    }
}

/// Bind a test identity to a function preceded by a bare `#[test]` (§3.4).
/// The span runs from the first attribute of the contiguous run to the end of
/// the function. Any other attribute whose path ends in `test` records a
/// `framework-test-attribute` site.
#[cfg(feature = "symbol-resolution")]
fn test_identity(b: &mut Builder, f: &RustFile, item: Node, id: &str) {
    let mut first = item;
    let mut bare = false;
    let mut framework = false;
    let mut cursor = item.prev_named_sibling();
    while let Some(attr) = cursor.filter(|n| n.kind() == "attribute_item") {
        first = attr;
        if let Some(attribute) = attr.named_child(0).filter(|n| n.kind() == "attribute") {
            let path = attribute.named_child(0);
            let has_args = attribute.child_by_field_name("arguments").is_some()
                || attribute.child_by_field_name("value").is_some();
            match path.map(|p| (p.kind(), p)) {
                Some(("identifier", p)) if text(p, f.src) == "test" => {
                    if has_args {
                        framework = true;
                    } else {
                        bare = true;
                    }
                }
                Some(("scoped_identifier", p))
                    if p.child_by_field_name("name")
                        .is_some_and(|n| text(n, f.src) == "test") =>
                {
                    framework = true;
                }
                _ => {}
            }
        }
        cursor = attr.prev_named_sibling();
    }
    if bare {
        let span = LineSpan::new(first.start_position().row + 1, item.end_position().row + 1);
        b.put(Namespace::Test, false, f.sink, id.to_string(), f.loc(span));
    } else if framework {
        b.site(
            Namespace::Test,
            id.to_string(),
            reason::FRAMEWORK_TEST_ATTRIBUTE,
        );
    }
}

/// Rust module path from a file under `src/`: `src/lib.rs`/`main.rs`/`mod.rs`
/// → crate root (`[]`); `src/foo.rs` → `["foo"]`; `src/foo/bar.rs` → `["foo","bar"]`.
#[cfg(feature = "symbol-resolution")]
fn rust_module_path(src_dir: &Path, file: &Path) -> Vec<String> {
    let rel = file.strip_prefix(src_dir).unwrap_or(file);
    let mut parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if let Some(last) = parts.pop() {
        let stem = last.trim_end_matches(".rs");
        if !matches!(stem, "lib" | "main" | "mod") {
            parts.push(stem.to_string());
        }
    }
    parts
}

/// `root::seg::…::seg` for a module path; the root module is the bare root.
#[cfg(feature = "symbol-resolution")]
fn qualify_module(root: &str, module: &[String]) -> String {
    let mut parts = Vec::with_capacity(module.len() + 1);
    parts.push(root.to_string());
    parts.extend(module.iter().cloned());
    parts.join("::")
}

// ===== TypeScript =====

/// The 0.28.0 TypeScript kinds: at a file's top level they are the legacy set.
#[cfg(feature = "symbol-resolution")]
const TS_KINDS: &[&str] = &[
    "function_declaration",
    "class_declaration",
    "interface_declaration",
    "type_alias_declaration",
    "enum_declaration",
];

#[cfg(feature = "symbol-resolution")]
fn index_ts_package(
    b: &mut Builder,
    repo_root: &Path,
    pkg: &PackageRecord,
    pkg_dir: &Path,
    exclusions: &[String],
    layout: &LayoutConfig,
) {
    for file in walk_files(pkg_dir, &["ts", "tsx"], repo_root, exclusions, layout) {
        // .vue and .d.ts are out of v1 scope.
        if file.to_string_lossy().ends_with(".d.ts") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let module = ts_module_path(pkg_dir, &file);
        let rel = rel_posix(repo_root, &file);
        let tsx = rel.ends_with(".tsx");
        // The legacy `.tsx` result keeps the TypeScript grammar (D-5).
        let Some(tree) = parse(&content, tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()) else {
            continue;
        };
        let src = content.strip_prefix('\u{feff}').unwrap_or(&content);
        let file_id = qualify_module(&pkg.name, &module);
        b.scopes.insert(file_id.clone());
        let root = tree.root_node();
        if root.has_error() {
            b.unknown(&file_id, reason::PARSE_ERROR_IN_SCOPE);
        }
        let loc = |span: LineSpan| ResolvedLocation {
            file: rel.clone(),
            span: Some(span),
        };
        let new_sink = if tsx {
            Sink::Unsupported(reason::TSX_GRAMMAR)
        } else {
            Sink::Source
        };
        let mut cursor = root.walk();
        for child in root.named_children(&mut cursor) {
            let decl = if child.kind() == "export_statement" {
                match child.child_by_field_name("declaration") {
                    Some(decl) => decl,
                    None => {
                        for name in export_clause_names(child, src) {
                            b.site(
                                Namespace::Symbol,
                                qualify(&pkg.name, &module, &name),
                                reason::RE_EXPORT,
                            );
                        }
                        continue;
                    }
                }
            } else if child.kind() == "expression_statement" {
                child.named_child(0).unwrap_or(child)
            } else {
                child
            };
            let kind = decl.kind();
            if TS_KINDS.contains(&kind) {
                let Some(name) = name_of(decl, src) else {
                    continue;
                };
                let id = qualify(&pkg.name, &module, name);
                b.put(
                    Namespace::Symbol,
                    true,
                    Sink::Source,
                    id.clone(),
                    loc(span_of(decl)),
                );
                match kind {
                    "class_declaration" => {
                        b.scopes.insert(id.clone());
                        if decl.has_error() {
                            b.unknown(&id, reason::PARSE_ERROR_IN_SCOPE);
                        }
                        ts_class_members(b, decl, &id, src, new_sink, &loc);
                    }
                    "interface_declaration" => {
                        b.scopes.insert(id.clone());
                        if let Some(body) = decl.child_by_field_name("body") {
                            let mut c = body.walk();
                            for member in body.named_children(&mut c) {
                                if let Some(n) = name_of(member, src) {
                                    b.site(Namespace::Symbol, format!("{id}::{n}"), reason::MEMBER);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else if matches!(kind, "lexical_declaration" | "variable_declaration") {
                let mut c = decl.walk();
                for declarator in decl.named_children(&mut c) {
                    if declarator.kind() != "variable_declarator" {
                        continue;
                    }
                    let Some(name) = declarator.child_by_field_name("name") else {
                        continue;
                    };
                    if name.kind() == "identifier" {
                        let id = qualify(&pkg.name, &module, text(name, src));
                        b.put(
                            Namespace::Symbol,
                            false,
                            new_sink,
                            id,
                            loc(span_of(declarator)),
                        );
                    } else {
                        for bound in pattern_identifiers(name, src) {
                            b.site(
                                Namespace::Symbol,
                                qualify(&pkg.name, &module, &bound),
                                reason::TS_DESTRUCTURING,
                            );
                        }
                    }
                }
            } else if kind == "internal_module"
                && let Some(name) = name_of(decl, src)
            {
                let id = qualify(&pkg.name, &module, name);
                b.site(Namespace::Symbol, id.clone(), reason::TS_NAMESPACE);
                ts_namespace_members(b, decl, &id, src, 0);
            }
        }
    }
}

/// A class body's members (§3.2, §3.5): methods with a plain name are symbols;
/// private and computed names, and fields, are recorded unsupported.
#[cfg(feature = "symbol-resolution")]
fn ts_class_members(
    b: &mut Builder,
    class: Node,
    id: &str,
    src: &str,
    sink: Sink,
    loc: &dyn Fn(LineSpan) -> ResolvedLocation,
) {
    let Some(body) = class.child_by_field_name("body") else {
        return;
    };
    let mut cursor = body.walk();
    for member in body.named_children(&mut cursor) {
        let Some(name) = member.child_by_field_name("name") else {
            continue;
        };
        let member_id = format!("{id}::{}", text(name, src));
        match (member.kind(), name.kind()) {
            ("method_definition", "property_identifier") => {
                b.put(
                    Namespace::Symbol,
                    false,
                    sink,
                    member_id,
                    loc(span_of(member)),
                );
            }
            ("method_definition", "private_property_identifier") => {
                b.site(Namespace::Symbol, member_id, reason::TS_COMPUTED_NAME);
            }
            ("method_definition", _) => {}
            (_, "property_identifier" | "private_property_identifier") => {
                b.site(Namespace::Symbol, member_id, reason::MEMBER);
            }
            _ => {}
        }
    }
}

/// Declarations inside a TypeScript namespace are `ts-namespace` sites.
#[cfg(feature = "symbol-resolution")]
fn ts_namespace_members(b: &mut Builder, ns: Node, id: &str, src: &str, depth: usize) {
    if depth > MAX_CONTAINER_DEPTH {
        return;
    }
    let Some(body) = ns.child_by_field_name("body") else {
        return;
    };
    let mut cursor = body.walk();
    for child in body.named_children(&mut cursor) {
        let decl = match child.kind() {
            "export_statement" => child.child_by_field_name("declaration").unwrap_or(child),
            "expression_statement" => child.named_child(0).unwrap_or(child),
            _ => child,
        };
        if let Some(name) = name_of(decl, src) {
            let member = format!("{id}::{name}");
            b.site(Namespace::Symbol, member.clone(), reason::TS_NAMESPACE);
            if decl.kind() == "internal_module" {
                ts_namespace_members(b, decl, &member, src, depth + 1);
            }
        }
    }
}

/// The names an `export { a, b as c }` clause binds (the alias when given).
#[cfg(feature = "symbol-resolution")]
fn export_clause_names(stmt: Node, src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = stmt.walk();
    for child in stmt.named_children(&mut cursor) {
        if child.kind() != "export_clause" {
            continue;
        }
        let mut c = child.walk();
        for spec in child.named_children(&mut c) {
            if spec.kind() != "export_specifier" {
                continue;
            }
            let bound = spec
                .child_by_field_name("alias")
                .or_else(|| spec.child_by_field_name("name"));
            if let Some(bound) = bound {
                out.push(text(bound, src).to_string());
            }
        }
    }
    out
}

/// The identifiers a destructuring pattern binds, in source order.
#[cfg(feature = "symbol-resolution")]
fn pattern_identifiers(pattern: Node, src: &str) -> Vec<String> {
    fn collect(node: Node, src: &str, out: &mut Vec<String>, depth: usize) {
        if depth > 64 {
            return;
        }
        match node.kind() {
            "identifier" | "shorthand_property_identifier_pattern" => {
                out.push(text(node, src).to_string());
            }
            "pair_pattern" => {
                if let Some(value) = node.child_by_field_name("value") {
                    collect(value, src, out, depth + 1);
                }
            }
            "assignment_pattern" | "object_assignment_pattern" => {
                if let Some(left) = node.child_by_field_name("left") {
                    collect(left, src, out, depth + 1);
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    collect(child, src, out, depth + 1);
                }
            }
        }
    }
    let mut out = Vec::new();
    collect(pattern, src, &mut out, 0);
    out
}

/// TS module path from a file under the package dir: components minus extension,
/// dropping a trailing `index`.
#[cfg(feature = "symbol-resolution")]
fn ts_module_path(pkg_dir: &Path, file: &Path) -> Vec<String> {
    let rel = file.strip_prefix(pkg_dir).unwrap_or(file);
    let mut parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if let Some(last) = parts.pop() {
        let stem = last.rsplit_once('.').map(|(s, _)| s).unwrap_or(&last);
        if stem != "index" {
            parts.push(stem.to_string());
        }
    }
    parts
}

// ===== shared =====

/// `prefix::module::...::name` (module segments omitted when empty).
#[cfg(feature = "symbol-resolution")]
fn qualify(prefix: &str, module: &[String], name: &str) -> String {
    let mut parts = Vec::with_capacity(module.len() + 2);
    parts.push(prefix.to_string());
    parts.extend(module.iter().cloned());
    parts.push(name.to_string());
    parts.join("::")
}

/// Node kinds whose `signature` / `body` projections the grammar defines
/// (spec 155 §3.5, spec 163 §3.7). A kind without a `body` field has neither.
#[cfg(feature = "symbol-resolution")]
const STRUCTURAL_KINDS: &[&str] = &[
    "function_item",
    "struct_item",
    "enum_item",
    "union_item",
    "trait_item",
    "const_item",
    "static_item",
    "type_item",
    "mod_item",
    "function_signature_item",
    "associated_type",
    "macro_definition",
    "function_declaration",
    "class_declaration",
    "interface_declaration",
    "type_alias_declaration",
    "enum_declaration",
    "method_definition",
    "variable_declarator",
];

/// Return line-bounded signature and body spans for the item beginning on
/// `start_line`. When the body begins on a line that also holds signature text
/// before it, the boundary shares a line and neither projection is
/// representable as whole lines (spec 163 §3.7), so both are `None`.
#[cfg(feature = "symbol-resolution")]
pub fn structural_spans(
    src: &str,
    extension: &str,
    start_line: usize,
) -> Option<(Option<LineSpan>, Option<LineSpan>)> {
    let language = language_for(extension)?;
    let tree = parse(src, language)?;
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let node = find_node_starting_on(tree.root_node(), start_line.saturating_sub(1))?;
    whole_line_spans(node, src)
}

/// [`structural_spans`] for an item whose span ends on `end_line` and starts at
/// or after `start_line`: a test location, whose span opens on its attribute
/// run rather than on the function (spec 163 §3.4).
#[cfg(feature = "symbol-resolution")]
pub fn structural_spans_ending(
    src: &str,
    extension: &str,
    start_line: usize,
    end_line: usize,
) -> Option<(Option<LineSpan>, Option<LineSpan>)> {
    let language = language_for(extension)?;
    let tree = parse(src, language)?;
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let node = find_node_ending_on(
        tree.root_node(),
        start_line.saturating_sub(1),
        end_line.saturating_sub(1),
        0,
    )?;
    whole_line_spans(node, src)
}

#[cfg(feature = "symbol-resolution")]
fn language_for(extension: &str) -> Option<Language> {
    match extension {
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "ts" | "tsx" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        _ => None,
    }
}

#[cfg(feature = "symbol-resolution")]
fn whole_line_spans(node: Node, src: &str) -> Option<(Option<LineSpan>, Option<LineSpan>)> {
    let body = node.child_by_field_name("body")?;
    let item_row = node.start_position().row;
    let body_row = body.start_position().row;
    let body_col = body.start_position().column;
    let shares_line = body_row == item_row
        || src
            .lines()
            .nth(body_row)
            .and_then(|line| line.get(..body_col))
            .is_some_and(|before| !before.trim().is_empty());
    if shares_line {
        return Some((None, None));
    }
    let signature = LineSpan::new(item_row + 1, body_row);
    let body = LineSpan::new(body_row + 1, body.end_position().row + 1);
    Some((Some(signature), Some(body)))
}

#[cfg(feature = "symbol-resolution")]
fn find_node_starting_on(node: Node<'_>, row: usize) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let candidate = if child.kind() == "export_statement" {
            child.child_by_field_name("declaration").unwrap_or(child)
        } else {
            child
        };
        if candidate.start_position().row == row && STRUCTURAL_KINDS.contains(&candidate.kind()) {
            return Some(candidate);
        }
        if candidate.start_position().row <= row
            && candidate.end_position().row >= row
            && let Some(found) = find_node_starting_on(candidate, row)
        {
            return Some(found);
        }
    }
    None
}

#[cfg(feature = "symbol-resolution")]
fn find_node_ending_on(
    node: Node<'_>,
    min_row: usize,
    end_row: usize,
    depth: usize,
) -> Option<Node<'_>> {
    if depth > 256 {
        return None;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.end_position().row == end_row
            && child.start_position().row >= min_row
            && child.kind() == "function_item"
        {
            return Some(child);
        }
        if child.start_position().row <= end_row
            && child.end_position().row >= end_row
            && let Some(found) = find_node_ending_on(child, min_row, end_row, depth + 1)
        {
            return Some(found);
        }
    }
    None
}

/// Recursively collect files with one of `exts` under `root`, sorted, skipping
/// excluded directories.
#[cfg(feature = "symbol-resolution")]
fn walk_files(
    root: &Path,
    exts: &[&str],
    repo_root: &Path,
    exclusions: &[String],
    layout: &LayoutConfig,
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, exts, repo_root, exclusions, layout, &mut out);
    out.sort();
    out
}

#[cfg(feature = "symbol-resolution")]
fn walk(
    dir: &Path,
    exts: &[&str],
    repo_root: &Path,
    exclusions: &[String],
    layout: &LayoutConfig,
    out: &mut Vec<PathBuf>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths {
        if is_excluded(repo_root, &path, exclusions)
            || layout.is_state_path(&rel_posix(repo_root, &path))
        {
            continue;
        }
        if path.is_dir() {
            walk(&path, exts, repo_root, exclusions, layout, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| exts.contains(&e))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_keeps_a_trait_impl_segment_whole() {
        assert_eq!(
            split_id("demo::<T as fmt::Display>::fmt"),
            vec!["demo", "<T as fmt::Display>", "fmt"]
        );
        assert_eq!(split_id("demo/tests/x::t"), vec!["demo/tests/x", "t"]);
    }

    #[test]
    fn a_disabled_resolver_answers_unknown() {
        assert_eq!(
            Resolver::disabled().lookup(Namespace::Symbol, "a::b"),
            Lookup::Unknown(reason::RESOLVER_DISABLED)
        );
    }

    #[cfg(feature = "symbol-resolution")]
    #[test]
    fn rust_module_qualification() {
        let src = Path::new("/p/src");
        assert!(rust_module_path(src, Path::new("/p/src/lib.rs")).is_empty());
        assert_eq!(
            rust_module_path(src, Path::new("/p/src/compile.rs")),
            vec!["compile"]
        );
        assert_eq!(
            rust_module_path(src, Path::new("/p/src/index/mod.rs")),
            vec!["index"]
        );
        assert_eq!(
            rust_module_path(src, Path::new("/p/src/index/foo.rs")),
            vec!["index", "foo"]
        );
    }
}
