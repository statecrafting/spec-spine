//! The capability catalog (spec 162): the one definition of every operation
//! this binary performs.
//!
//! Static data, built into the binary: [`capability_catalog`] reads no file,
//! clock, environment variable or repository (162 §3.1, §3.14). The CLI's
//! clap tree, the facade in `lib.rs`, the verdict verbs and the managed gate
//! are held to it by census tests (162 §3.10), and every example runs against
//! the built binary (162 §3.9). It describes; it authorizes nothing.

use std::collections::BTreeMap;

use serde::Serialize;
use sha2::{Digest, Sha256};
use spec_spine_types::{
    ATTESTATION_SCHEMA_VERSION, AddedEffects, BUILD_META_SCHEMA_VERSION, Bound, Budget,
    CAPABILITIES_SCHEMA_VERSION, CATALOG_SCHEMA_VERSION, CONFIG_VERSION, CapabilitiesDiscovery,
    CapabilityCatalog, CapabilityVerifyReport, CapabilityVerifyRequest, CatalogDiscovery,
    CliBinding, ConditionalEffects, DELTA_SCHEMA_VERSION, Effects, Error, Example, ExitOutcome,
    FacadeBinding, Flag, INDEX_SCHEMA_VERSION, InterfaceDiscovery, Operation, PinResult,
    Precondition, READ_SCHEMA_VERSION, REGISTRY_SCHEMA_VERSION, RequestRef, ResponseRef,
    SNAPSHOT_SCHEMA_VERSION, SPEC_ATTESTATION_SCHEMA_VERSION, VERDICT_SCHEMA_VERSION,
};

use crate::canonical_json;

/// The embedded schemas' `$id`s the catalog cites (162 §3.6).
const SCHEMA_BASE: &str = "https://spec-spine.dev/schemas/";

fn schema_id(file: &str) -> Option<String> {
    Some(format!("{SCHEMA_BASE}{file}"))
}

/// Every schema axis the types crate declares, at its current version
/// (162 §3.1). Read from the constants, never restated.
pub fn compatibility() -> BTreeMap<String, String> {
    [
        ("attestation", ATTESTATION_SCHEMA_VERSION),
        ("build-meta", BUILD_META_SCHEMA_VERSION),
        ("capabilities", CAPABILITIES_SCHEMA_VERSION),
        ("catalog", CATALOG_SCHEMA_VERSION),
        ("config", CONFIG_VERSION),
        ("delta", DELTA_SCHEMA_VERSION),
        ("index", INDEX_SCHEMA_VERSION),
        ("read", READ_SCHEMA_VERSION),
        ("registry", REGISTRY_SCHEMA_VERSION),
        ("snapshot", SNAPSHOT_SCHEMA_VERSION),
        ("spec-attestation", SPEC_ATTESTATION_SCHEMA_VERSION),
        ("verdict", VERDICT_SCHEMA_VERSION),
    ]
    .into_iter()
    .map(|(axis, version)| (axis.to_string(), version.to_string()))
    .collect()
}

/// The version an axis carries in this build: its [`compatibility`] entry, or
/// empty for `unversioned` and for an axis no entry names. An undeclared axis
/// is refused by [`capability_catalog`] as an internal error rather than
/// panicking here, so a typo in the static data cannot abort the process.
fn axis_version(axis: &str) -> String {
    compatibility().remove(axis).unwrap_or_default()
}

fn strings(items: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = items.iter().map(|s| s.to_string()).collect();
    out.sort();
    out.dedup();
    out
}

/// The exit pairs of spec 132 and the error kinds `Error::exit_code` maps to
/// each.
fn outcome(code: u8, kinds: &[&str]) -> ExitOutcome {
    ExitOutcome {
        exit_code: code,
        outcome: spec_spine_types::outcome::of(code).to_string(),
        kinds: strings(kinds),
    }
}

/// One operation under construction. Defaults describe a pinned CLI read of
/// the committed ledger: `reads: [config, corpus, derived-ledger]`, the pin
/// and configuration preconditions, and the outcomes every such verb has
/// (0, 2, 3, 4).
struct Op(Operation);

fn op(name: &str, summary: &str, governed_by: &[&str]) -> Op {
    Op(Operation {
        name: name.to_string(),
        summary: summary.to_string(),
        governed_by: strings(governed_by),
        cli: None,
        facade: Vec::new(),
        request: RequestRef {
            kind: "argv".to_string(),
            schema: None,
        },
        response: Vec::new(),
        effects: Effects {
            reads: strings(&["config", "corpus", "derived-ledger"]),
            ..Effects::default()
        },
        effects_when: Vec::new(),
        preconditions: vec![
            Precondition {
                name: "config-loadable".to_string(),
                exit_codes: vec![2],
            },
            Precondition {
                name: "version-pin-met".to_string(),
                exit_codes: vec![2],
            },
        ],
        outcomes: vec![
            outcome(0, &[]),
            outcome(2, &["config", "refused"]),
            outcome(3, &["usage"]),
            outcome(4, &["internal", "io", "schema"]),
        ],
        budget: None,
        pagination: "none".to_string(),
        stability: "stable".to_string(),
        since: None,
        deprecation: None,
        examples: Vec::new(),
        operation_digest: String::new(),
    })
}

/// A facade-only operation: no CLI binding, no pin, no CLI effects beyond
/// the core invariant (162 §3.4), so `reads` is empty.
fn library(name: &str, summary: &str, governed_by: &[&str]) -> Op {
    let mut o = op(name, summary, governed_by);
    o.0.effects.reads.clear();
    o.0.preconditions.clear();
    o.0.outcomes = vec![
        outcome(0, &[]),
        outcome(2, &["config"]),
        outcome(4, &["internal"]),
    ];
    o
}

impl Op {
    /// Bind the CLI form. `flags` use a compact spelling: `--long`,
    /// `--long=VALUE`, `<id>=VALUE` for a positional, `!` for required and
    /// `*` for repeatable as suffixes. The global `--repo=DIR` is added.
    fn cli(mut self, argv: &str, output: &str, flags: &[&str]) -> Self {
        let mut parsed: Vec<Flag> = flags
            .iter()
            .chain(["--repo=DIR"].iter())
            .map(|spec| parse_flag(spec))
            .collect();
        parsed.sort_by(|a, b| a.name.cmp(&b.name));
        self.0.cli = Some(CliBinding {
            argv: argv.split_whitespace().map(str::to_string).collect(),
            flags: parsed,
            output: output.to_string(),
        });
        self
    }

    fn facade(mut self, function: &str, selector: Option<&str>, reads_repository: bool) -> Self {
        self.0.facade.push(FacadeBinding {
            function: function.to_string(),
            selector: selector.map(str::to_string),
            reads_repository,
        });
        self
    }

    fn request_document(mut self) -> Self {
        self.0.request.kind = "document".to_string();
        self
    }

    fn response(
        mut self,
        when: Option<&str>,
        axis: &str,
        document: &str,
        schema: Option<String>,
    ) -> Self {
        self.0.response.push(ResponseRef {
            when: when.map(str::to_string),
            axis: axis.to_string(),
            version: axis_version(axis),
            document: document.to_string(),
            schema,
        });
        self
    }

    fn verdict(self) -> Self {
        self.response(None, "verdict", "verdict-envelope", None)
    }

    fn read(self, document: &str) -> Self {
        self.response(None, "read", document, None)
    }

    fn reads(mut self, items: &[&str]) -> Self {
        self.0.effects.reads = strings(items);
        self
    }

    fn writes(mut self, items: &[&str]) -> Self {
        self.0.effects.writes = strings(items);
        self
    }

    fn executes(mut self, items: &[&str]) -> Self {
        self.0.effects.executes = strings(items);
        self
    }

    fn environment(mut self, items: &[&str]) -> Self {
        self.0.effects.environment = strings(items);
        self
    }

    fn authority(mut self, items: &[&str]) -> Self {
        self.0.effects.authority = strings(items);
        self
    }

    /// `declared-commands` and its mandatory companions (162 §3.4).
    fn runs_declared_commands(mut self) -> Self {
        self.0.effects.executes = strings(&["declared-commands"]);
        self.0.effects.writes = strings(&["delegated"]);
        self.0.effects.network = "delegated".to_string();
        self.0.effects.authority = strings(&["code-execution"]);
        self
    }

    fn when(mut self, flag: &str, reads: &[&str], authority: &[&str]) -> Self {
        self.0.effects_when.push(ConditionalEffects {
            flag: flag.to_string(),
            effects: AddedEffects {
                reads: strings(reads),
                authority: strings(authority),
                ..AddedEffects::default()
            },
        });
        self
    }

    fn pre(mut self, name: &str, codes: &[u8]) -> Self {
        self.0.preconditions.push(Precondition {
            name: name.to_string(),
            exit_codes: codes.to_vec(),
        });
        self
    }

    /// Declare exit 1 with the error kinds that produce it. Empty `kinds`
    /// declares a finding the report itself carries (a drift, a stale pin).
    fn finding(mut self, kinds: &[&str]) -> Self {
        self.0.outcomes.push(outcome(1, kinds));
        self
    }

    /// Answered before the version pin (spec 170 D-8, 162 D-5): no pin or
    /// configuration precondition, and so no exit 2.
    fn before_the_pin(mut self) -> Self {
        self.0.preconditions.clear();
        self.0.outcomes.retain(|o| o.exit_code != 2);
        self
    }

    fn experimental(mut self) -> Self {
        self.0.stability = "experimental".to_string();
        self
    }

    fn since(mut self, version: &str) -> Self {
        self.0.since = Some(version.to_string());
        self
    }

    fn bounded(mut self, bytes: Bound, items: Bound) -> Self {
        self.0.budget = Some(Budget {
            max_bytes: bytes,
            max_items: items,
        });
        self.0.pagination = "continuation".to_string();
        self
    }

    fn example(
        mut self,
        id: &str,
        fixture: Option<&str>,
        argv: &[&str],
        exit_code: u8,
        stdout_includes: &[&str],
    ) -> Self {
        self.0.examples.push(Example {
            id: id.to_string(),
            fixture: fixture.map(str::to_string),
            argv: argv.iter().map(|s| s.to_string()).collect(),
            stdin: None,
            exit_code,
            stdout_includes: strings(stdout_includes),
        });
        self
    }

    /// An example fed `stdin`: a CLI example reading `-`, or a facade-only
    /// example whose request it is.
    fn example_stdin(
        mut self,
        id: &str,
        fixture: Option<&str>,
        argv: &[&str],
        stdin: &str,
        exit_code: u8,
        stdout_includes: &[&str],
    ) -> Self {
        self = self.example(id, fixture, argv, exit_code, stdout_includes);
        if let Some(last) = self.0.examples.last_mut() {
            last.stdin = Some(stdin.to_string());
        }
        self
    }

    /// The version-pin refusal every pinned verb shares (162 §3.9): the
    /// `pinned` fixture requires a version no build satisfies.
    fn pinned_example(self, argv: &[&str]) -> Self {
        let id = format!("{}-pin-not-met", self.0.name);
        self.example(&id, Some("pinned"), argv, 2, &[])
    }

    fn done(self) -> Operation {
        let mut o = self.0;
        o.facade.sort_by(|a, b| {
            (a.function.as_str(), a.selector.as_deref())
                .cmp(&(b.function.as_str(), b.selector.as_deref()))
        });
        o.response.sort_by(|a, b| {
            (a.when.is_some(), a.when.as_deref(), a.axis.as_str()).cmp(&(
                b.when.is_some(),
                b.when.as_deref(),
                b.axis.as_str(),
            ))
        });
        o.effects_when.sort_by(|a, b| a.flag.cmp(&b.flag));
        o.preconditions.sort_by(|a, b| a.name.cmp(&b.name));
        o.outcomes.sort_by_key(|x| x.exit_code);
        o.examples.sort_by(|a, b| a.id.cmp(&b.id));
        o
    }
}

fn parse_flag(spec: &str) -> Flag {
    let mut s = spec;
    let mut required = false;
    let mut repeatable = false;
    loop {
        if let Some(rest) = s.strip_suffix('!') {
            required = true;
            s = rest;
        } else if let Some(rest) = s.strip_suffix('*') {
            repeatable = true;
            s = rest;
        } else {
            break;
        }
    }
    let (name, value_name) = match s.split_once('=') {
        Some((n, v)) => (n, Some(v.to_string())),
        None => (s, None),
    };
    let positional = name.starts_with('<');
    let name = name.trim_start_matches('<').trim_end_matches('>');
    Flag {
        name: name.to_string(),
        positional,
        value_name,
        required,
        repeatable,
        conflicts_with: Vec::new(),
    }
}

/// `sha256:` plus the lowercase hex digest of `value`'s canonical JSON with
/// `member` removed from the top-level object (162 §3.11).
fn digest_without<T: Serialize>(value: &T, member: &str) -> Result<String, Error> {
    let mut v = serde_json::to_value(value).map_err(|e| Error::Internal(e.to_string()))?;
    if let Some(map) = v.as_object_mut() {
        map.remove(member);
    }
    let bytes = canonical_json::to_string(&v)?;
    use std::fmt::Write as _;
    let mut hex = String::from("sha256:");
    for byte in Sha256::digest(bytes.as_bytes()) {
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(hex)
}

/// The digest of one operation record (162 §3.11).
pub fn operation_digest(operation: &Operation) -> Result<String, Error> {
    digest_without(operation, "operationDigest")
}

/// The digest of the whole catalog (162 §3.11).
pub fn catalog_digest(catalog: &CapabilityCatalog) -> Result<String, Error> {
    digest_without(catalog, "catalogDigest")
}

const TREE_READ: &[&str] = &["config", "corpus", "derived-ledger", "source-tree"];

fn verify_flags() -> &'static [&'static str] {
    &[
        "<id>=ID",
        "--plan",
        "--affected-by=BASE",
        "--head=REV",
        "--json",
    ]
}

fn compact_flags() -> &'static [&'static str] {
    &["--plan-file=FILE!", "--plan", "--force", "--map-out=FILE"]
}

fn compile_flags() -> &'static [&'static str] {
    &["--check", "--json", "--spec=ID", "--fail-on-warn"]
}

/// Every operation, unsorted. [`capability_catalog`] sorts and digests.
fn operations() -> Vec<Operation> {
    let content_bytes = Bound {
        default: 262_144,
        min: 1,
        max: 1_048_576,
    };
    let content_items = Bound {
        default: 64,
        min: 1,
        max: 256,
    };
    vec![
        op(
            "attest",
            "Emit a reproducible corpus attestation; optionally seal it (spec 021)",
            &["021-ledger-seal", "039-per-spec-attestation", "070-an-authority-snapshot-says-what-it-read"],
        )
        .cli(
            "attest",
            "verdict-envelope",
            &["--spec=ID", "--with-coupling", "--snapshot", "--sign", "--key=PATH", "--key-id=ID", "--json"],
        )
        .reads(&["clock", "config", "corpus", "derived-ledger", "source-tree"])
        .writes(&["attestation"])
        .when("--sign", &["key-material"], &["signing"])
        .verdict()
        .response(Some("--snapshot"), "snapshot", "authority-snapshot", None)
        .response(Some("--spec"), "spec-attestation", "spec-attestation", None)
        .finding(&["not-found", "validation"])
        .facade("attest_json", None, true)
        .facade("attest_spec_json", None, true)
        .facade("attest_snapshot_json", None, true)
        .example("attest-json", Some("corpus"), &["attest", "--json"], 0, &["\"verb\": \"attest\""])
        .pinned_example(&["attest", "--json"])
        .done(),
        op(
            "capabilities",
            "State what this binary supports: every verb, its flags, and the schema its `--json` output carries (spec 170). Reads no repository and writes nothing",
            &["162-capability-catalog", "170-a-consumer-is-served-answers-not-access"],
        )
        .cli("capabilities", "read-document", &["--json", "--operation=NAME"])
        .reads(&[])
        .before_the_pin()
        .finding(&["not-found"])
        .response(None, "capabilities", "capabilities", None)
        .response(Some("--operation"), "catalog", "catalog-operation", None)
        .facade("capability_catalog_json", None, false)
        .experimental()
        .example("capabilities-json", None, &["capabilities", "--json"], 0, &["\"catalogDigest\"", "\"verbs\""])
        .example(
            "capabilities-operation",
            None,
            &["capabilities", "--operation", "capabilities.verify", "--json"],
            0,
            &["\"name\": \"capabilities.verify\""],
        )
        .example(
            "capabilities-operation-unknown",
            None,
            &["capabilities", "--operation", "no.such-operation", "--json"],
            1,
            &["\"kind\": \"not-found\""],
        )
        .done(),
        op(
            "capabilities.verify",
            "Check pinned operation digests against this binary's catalog (spec 162). Reads no repository and writes nothing",
            &["162-capability-catalog"],
        )
        .cli("capabilities verify", "verdict-envelope", &["--expect=NAME=DIGEST!*", "--json"])
        .reads(&[])
        .before_the_pin()
        .finding(&[])
        .verdict()
        .facade("capability_verify_json", None, false)
        .experimental()
        .since("0.30.0")
        .example(
            "capabilities-verify-changed",
            None,
            &[
                "capabilities",
                "verify",
                "--expect",
                "capabilities=sha256:0000000000000000000000000000000000000000000000000000000000000000",
                "--json",
            ],
            1,
            &["\"changed\"", "\"verb\": \"capabilities.verify\""],
        )
        .example(
            "capabilities-verify-missing",
            None,
            &[
                "capabilities",
                "verify",
                "--expect",
                "no.such-operation=sha256:0000000000000000000000000000000000000000000000000000000000000000",
            ],
            1,
            &["missing"],
        )
        .example(
            "capabilities-verify-malformed",
            None,
            &["capabilities", "verify", "--expect", "capabilities", "--json"],
            3,
            &["\"kind\": \"usage\""],
        )
        .done(),
        op(
            "check",
            "Both freshness reads in one verb: are the committed registry shards and the committed index shards current (spec 062)?",
            &["062-one-name-one-freshness-verb"],
        )
        .cli("check", "verdict-envelope", &["--fail-on-unresolved", "--fail-on-warn", "--json"])
        .reads(TREE_READ)
        .authority(&["gate-verdict"])
        .finding(&["stale", "validation"])
        .verdict()
        .facade("check_json", None, true)
        .example("check-fresh", Some("corpus"), &["check", "--json"], 0, &["\"verb\": \"check\""])
        .pinned_example(&["check"])
        .done(),
        op(
            "compact",
            "Rewrite the corpus under an authored compaction plan (spec 096)",
            &["096-compaction-is-a-verb-not-a-session"],
        )
        .cli("compact", "files", compact_flags())
        .reads(&["caller-file", "config", "corpus", "derived-ledger", "source-tree"])
        .writes(&["caller-path", "corpus"])
        .executes(&["git"])
        .authority(&["corpus-rewrite"])
        .pre("clean-tree", &[2])
        .pre("git-repository", &[4])
        .finding(&["not-found", "validation"])
        .response(None, "unversioned", "compaction-map", None)
        .pinned_example(&["compact", "--plan-file", "plan.yaml"])
        .done(),
        op(
            "compact.plan",
            "Rewrite the corpus under an authored compaction plan (spec 096)",
            &["096-compaction-is-a-verb-not-a-session"],
        )
        .cli("compact", "text", compact_flags())
        .reads(&["caller-file", "config", "corpus", "derived-ledger", "source-tree"])
        .finding(&["not-found", "validation"])
        .request_document()
        .response(None, "unversioned", "compaction-report", None)
        .facade("compact_json", None, true)
        .example(
            "compact-plan-missing-file",
            Some("corpus"),
            &["compact", "--plan-file", "no-such-plan.yaml", "--plan"],
            4,
            &[],
        )
        .pinned_example(&["compact", "--plan-file", "plan.yaml", "--plan"])
        .done(),
        op(
            "compile",
            "Compile specs/*/spec.md into a deterministic registry",
            &["001-compile-registry"],
        )
        .cli("compile", "files", compile_flags())
        .reads(&["clock", "config", "corpus"])
        .writes(&["derived-ledger"])
        .authority(&["ledger-rewrite"])
        .finding(&["validation"])
        .response(
            None,
            "registry",
            "registry-spec-shard",
            schema_id("registry-spec-shard.schema.json"),
        )
        .facade("compile_json", None, true)
        .example("compile-writes", Some("corpus"), &["compile"], 0, &["compiled 2 spec(s)"])
        .pinned_example(&["compile"])
        .done(),
        op(
            "compile.check",
            "Compile specs/*/spec.md into a deterministic registry",
            &["001-compile-registry", "062-one-name-one-freshness-verb"],
        )
        .cli("compile", "verdict-envelope", compile_flags())
        .reads(&["config", "corpus", "derived-ledger"])
        .finding(&["stale", "validation"])
        .verdict()
        .facade("check_registry_freshness_json", None, true)
        .example(
            "compile-check-fresh",
            Some("corpus"),
            &["compile", "--check", "--json"],
            0,
            &["\"verb\": \"compile.check\""],
        )
        .pinned_example(&["compile", "--check"])
        .done(),
        op(
            "compile.spec",
            "Compile specs/*/spec.md into a deterministic registry",
            &["001-compile-registry", "049-compile-one-spec"],
        )
        .cli("compile", "verdict-envelope", compile_flags())
        .reads(&["config", "corpus"])
        .finding(&["not-found", "validation"])
        .verdict()
        .example(
            "compile-spec-one",
            Some("corpus"),
            &["compile", "--spec", "001", "--json"],
            0,
            &["\"verb\": \"compile.spec\""],
        )
        .example(
            "compile-spec-unknown",
            Some("corpus"),
            &["compile", "--spec", "999", "--json"],
            1,
            &["\"kind\": \"not-found\""],
        )
        .done(),
        op(
            "config.show",
            "Print the effective configuration: every default resolved, and the bypass floor merged with the adopter's list and attributed",
            &["047-effective-config-is-a-governed-read"],
        )
        .cli("config show", "read-document", &["--json"])
        .reads(&["config"])
        .response(None, "config", "effective-config", None)
        .example("config-show", Some("corpus"), &["config", "show", "--json"], 0, &["\"bypass_prefixes\""])
        .pinned_example(&["config", "show"])
        .done(),
        op(
            "content.select",
            "Select bounded content from HEAD or one exact exported revision",
            &["155-selected-content-accessor"],
        )
        .cli(
            "content select",
            "read-document",
            &["--request=FILE!", "--repository=REPOSITORY!", "--revision=REVISION", "--json!"],
        )
        .reads(&["caller-file", "clock", "config", "corpus", "derived-ledger", "source-tree"])
        .writes(&["temporary"])
        .executes(&["git"])
        .pre("clean-tree", &[2])
        .pre("git-repository", &[4])
        .finding(&["not-found", "stale", "validation"])
        .request_document()
        .read("content-selection")
        .bounded(content_bytes, content_items)
        .facade("selected_content_json", None, true)
        .example(
            "content-select-file",
            Some("corpus"),
            &[
                "content",
                "select",
                "--request",
                "content-request.json",
                "--repository",
                "example/fixture",
                "--json",
            ],
            0,
            &["\"repository\": \"example/fixture\""],
        )
        .pinned_example(&[
            "content",
            "select",
            "--request",
            "content-request.json",
            "--repository",
            "example/fixture",
            "--json",
        ])
        .done(),
        op(
            "couple",
            "The PR-time coupling gate: refuse code that drifts from its owning spec",
            &["005-coupling-gate"],
        )
        .cli(
            "couple",
            "verdict-envelope",
            &[
                "--base=BASE",
                "--head=HEAD",
                "--pr-body=PR_BODY",
                "--paths-from=PATHS_FROM",
                "--include-uncommitted",
                "--waiver-as-of=YYYY-MM-DD",
                "--waiver-uses=ID=N*",
                "--json",
            ],
        )
        .reads(&["caller-file", "clock", "config", "corpus", "derived-ledger", "source-tree"])
        .writes(&["temporary"])
        .executes(&["git"])
        .environment(&["SPEC_SPINE_PR_BODY"])
        .authority(&["gate-verdict", "waiver-evaluation"])
        .pre("git-repository", &[4])
        .pre("refs-resolvable", &[1, 4])
        .finding(&["drift", "not-found", "stale", "validation"])
        .verdict()
        .facade("couple_json", None, true)
        .example(
            "couple-drift",
            Some("history"),
            &["couple", "--base", "HEAD~1", "--head", "HEAD", "--json"],
            1,
            &["\"verb\": \"couple\""],
        )
        .pinned_example(&["couple"])
        .done(),
        op(
            "delta",
            "Classify every path a change touches under the merge base's rules (spec 071). A report, not a gate: exit 0 whenever a report was produced",
            &["071-a-change-is-classified-under-the-bases-rules"],
        )
        .cli("delta", "verdict-envelope", &["--base=BASE", "--head=HEAD", "--json"])
        .reads(&["clock", "config", "corpus", "derived-ledger", "source-tree"])
        .writes(&["temporary"])
        .executes(&["git"])
        .pre("git-repository", &[4])
        .pre("refs-resolvable", &[4])
        .verdict()
        .response(None, "delta", "delta-report", None)
        .facade("delta_json", None, true)
        .example(
            "delta-report",
            Some("history"),
            &["delta", "--base", "HEAD~1", "--head", "HEAD", "--json"],
            0,
            &["\"verb\": \"delta\""],
        )
        .pinned_example(&["delta"])
        .done(),
        op(
            "index.build",
            "Build the codebase index, or check it for staleness",
            &["004-codebase-index"],
        )
        .cli("index", "files", &[])
        .reads(&["clock", "config", "corpus", "source-tree"])
        .writes(&["derived-ledger"])
        .executes(&["git"])
        .authority(&["ledger-rewrite"])
        .finding(&["validation"])
        .response(
            None,
            "index",
            "codebase-index-spec-shard",
            schema_id("codebase-index-spec-shard.schema.json"),
        )
        .facade("index_json", None, true)
        .example("index-writes", Some("corpus"), &["index"], 0, &["indexed"])
        .pinned_example(&["index"])
        .done(),
        op(
            "index.check",
            "Check the committed index against current inputs (the staleness gate)",
            &["004-codebase-index", "044-index-diagnostics-reach-a-gate"],
        )
        .cli("index check", "verdict-envelope", &["--slice=NAME", "--fail-on-unresolved", "--json"])
        .reads(TREE_READ)
        .authority(&["gate-verdict"])
        .finding(&["not-found", "stale", "validation"])
        .verdict()
        .facade("check_freshness_json", None, true)
        .example(
            "index-check-fresh",
            Some("corpus"),
            &["index", "check", "--json"],
            0,
            &["\"verb\": \"index.check\""],
        )
        .pinned_example(&["index", "check"])
        .done(),
        op(
            "index.coverage",
            "Report which source files no spec specifically claims (spec 029)",
            &["029-ownership-coverage"],
        )
        .cli("index coverage", "read-document", &["--json", "--fail-on-untraced", "--paths-from=FILE"])
        .reads(&["caller-file", "config", "corpus", "derived-ledger", "source-tree"])
        .executes(&["git"])
        .authority(&["gate-verdict"])
        .finding(&["validation"])
        .read("coverage")
        .facade("coverage_json", None, true)
        .facade("coverage_inventory_json", None, true)
        .example(
            "index-coverage",
            Some("corpus"),
            &["index", "coverage", "--json"],
            0,
            &["\"schemaVersion\""],
        )
        .example(
            "index-coverage-untraced",
            Some("corpus"),
            &["index", "coverage", "--fail-on-untraced"],
            1,
            &["unclaimed"],
        )
        .done(),
        op(
            "index.diagnostics",
            "List the diagnostics the committed index records (spec 044)",
            &["044-index-diagnostics-reach-a-gate"],
        )
        .cli("index diagnostics", "read-document", &["--json"])
        .finding(&["stale"])
        .read("index-diagnostics")
        .example(
            "index-diagnostics",
            Some("corpus"),
            &["index", "diagnostics", "--json"],
            0,
            &["\"schemaVersion\""],
        )
        .pinned_example(&["index", "diagnostics"])
        .done(),
        op(
            "index.orphans",
            "List orphaned specs from the committed index",
            &["004-codebase-index"],
        )
        .cli("index orphans", "read-document", &["--json"])
        .finding(&["stale"])
        .read("index-orphans")
        .facade("orphans_json", None, false)
        .example("index-orphans", Some("corpus"), &["index", "orphans", "--json"], 0, &["\"schemaVersion\""])
        .pinned_example(&["index", "orphans"])
        .done(),
        op(
            "index.owner",
            "Report which specs own one path, and how (spec 048)",
            &["048-the-ledger-answers-what-consumers-rebuild"],
        )
        .cli("index owner", "read-document", &["<path>=PATH!", "--json"])
        .authority(&["gate-verdict"])
        .finding(&["stale"])
        .read("index-owner")
        .example(
            "index-owner",
            Some("corpus"),
            &["index", "owner", "src/alpha.rs", "--json"],
            0,
            &["001-alpha"],
        )
        .pinned_example(&["index", "owner", "src/alpha.rs"])
        .done(),
        op(
            "index.render",
            "Render the committed index as markdown (a projection; never recomputes)",
            &["004-codebase-index"],
        )
        .cli("index render", "text", &[])
        .finding(&["stale"])
        .response(None, "unversioned", "index-markdown", None)
        .facade("render_json", None, false)
        .example("index-render", Some("corpus"), &["index", "render"], 0, &["001-alpha"])
        .pinned_example(&["index", "render"])
        .done(),
        op(
            "interface.verify",
            "Recompute every declared interface reference (spec 110) against local checkouts of the cited corpora. Exit 0 when every checked reference is `current` or `sections-current`, 1 when any is `stale`, `missing` or `unverified`, and 1 when the committed registry is stale (spec 132)",
            &["110-an-interface-reference-is-digest-pinned"],
        )
        .cli("interface verify", "read-document", &["--export=CORPUS=DIR*", "--spec=ID", "--json"])
        .reads(&["config", "corpus", "derived-ledger", "export-directory"])
        .pre("registry-fresh", &[1])
        .finding(&["not-found", "stale"])
        .read("interface-report")
        .facade("interface_verify_json", None, false)
        .example(
            "interface-verify-none",
            Some("corpus"),
            &["interface", "verify", "--json"],
            0,
            &["\"schemaVersion\""],
        )
        .pinned_example(&["interface", "verify"])
        .done(),
        op("lint", "Run the corpus conformance lint", &["003-conformance-lint"])
            .cli("lint", "verdict-envelope", &["--fail-on-warn", "--fail-on-info", "--json"])
            .authority(&["gate-verdict"])
            .finding(&["validation"])
            .verdict()
            .facade("lint_json", None, true)
            .example("lint-clean", Some("corpus"), &["lint", "--json"], 0, &["\"verb\": \"lint\""])
            .pinned_example(&["lint"])
            .done(),
        op(
            "registry.closure",
            "Resolve a context closure against the committed ledger (spec 107): every member's identity and one order-independent digest. Refuses a stale registry (exit 1)",
            &["107-a-context-closure-is-declared"],
        )
        .cli("registry closure", "read-document", &["--request=FILE!", "--json"])
        .reads(&["caller-file", "config", "corpus", "derived-ledger", "stdin"])
        .pre("registry-fresh", &[1])
        .finding(&["not-found", "stale", "validation"])
        .request_document()
        .read("context-closure")
        .facade("closure_json", None, true)
        .example_stdin(
            "registry-closure",
            Some("corpus"),
            &["registry", "closure", "--request", "-", "--json"],
            r#"{"specs":["001"],"rationale":"example"}"#,
            0,
            &["\"digest\"", "001-alpha"],
        )
        .pinned_example(&["registry", "closure", "--request", "-"])
        .done(),
        op(
            "registry.impacts",
            "Every declared impact and conflict (spec 109), inverted so the target side can see what was declared about it. `--target` is a spec id (every obligation it declares) or a qualified `<spec-id>#<obligation-id>` reference; `--declared-by` is a spec id; both compose by intersection",
            &["109-impact-and-conflict-are-declared"],
        )
        .cli("registry impacts", "read-document", &["--target=REF", "--declared-by=SPEC", "--json"])
        .finding(&["not-found", "stale", "validation"])
        .read("impacts")
        .facade("query_json", Some("impacts"), false)
        .example("registry-impacts", Some("corpus"), &["registry", "impacts", "--json"], 0, &["\"schemaVersion\""])
        .pinned_example(&["registry", "impacts"])
        .done(),
        op(
            "registry.list",
            "List specs (optionally filtered by status)",
            &["002-registry-query"],
        )
        .cli("registry list", "read-document", &["--status=STATUS", "--json", "--ids-only"])
        .authority(&["gate-verdict"])
        .finding(&["stale"])
        .read("registry-list")
        .facade("query_json", Some("list"), false)
        .example("registry-list", Some("corpus"), &["registry", "list", "--json"], 0, &["001-alpha", "002-beta"])
        .pinned_example(&["registry", "list"])
        .done(),
        op(
            "registry.moves",
            "Look up a path against every declared move (spec 111 §3.4). Without a path, lists every declaration, flattened and sorted: the derived path map. Answers from the committed registry, like `show`; refuses nothing about the working tree. Exits 1 on `ambiguous` or `cycle`",
            &["111-a-move-is-a-reviewed-mapping"],
        )
        .cli("registry moves", "read-document", &["<path>=PATH", "--json"])
        .finding(&["stale"])
        .read("moves")
        .facade("query_json", Some("moves"), false)
        .example("registry-moves", Some("corpus"), &["registry", "moves", "--json"], 0, &["\"schemaVersion\""])
        .pinned_example(&["registry", "moves"])
        .done(),
        op(
            "registry.obligation",
            "Resolve a qualified obligation reference, `<spec-id>#<obligation-id>` (spec 106). An unqualified id is refused, never resolved locally",
            &["106-obligations-are-declared-constraints"],
        )
        .cli("registry obligation", "read-document", &["<reference>=REFERENCE!", "--json"])
        .finding(&["not-found", "stale", "validation"])
        .read("obligation")
        .facade("query_json", Some("obligation"), false)
        .example(
            "registry-obligation",
            Some("corpus"),
            &["registry", "obligation", "001-alpha#R-1", "--json"],
            0,
            &["\"R-1\""],
        )
        .example(
            "registry-obligation-unknown",
            Some("corpus"),
            &["registry", "obligation", "001-alpha#R-9", "--json"],
            1,
            &["\"kind\": \"not-found\""],
        )
        .done(),
        op(
            "registry.plan",
            "Which specs can be worked on now, and what blocks the rest (spec 035)",
            &["035-registry-plan-ready-set"],
        )
        .cli("registry plan", "read-document", &["--json", "--next"])
        .finding(&["stale", "validation"])
        .read("plan")
        .facade("query_json", Some("plan"), false)
        .example("registry-plan", Some("corpus"), &["registry", "plan", "--json"], 0, &["\"schemaVersion\""])
        .pinned_example(&["registry", "plan"])
        .done(),
        op(
            "registry.relationships",
            "Show a spec's relationship neighborhood",
            &["002-registry-query"],
        )
        .cli("registry relationships", "read-document", &["<id>=ID!", "--json"])
        .finding(&["not-found", "stale"])
        .read("relationships")
        .facade("query_json", Some("relationships"), false)
        .example(
            "registry-relationships",
            Some("corpus"),
            &["registry", "relationships", "002", "--json"],
            0,
            &["001-alpha"],
        )
        .pinned_example(&["registry", "relationships", "002"])
        .done(),
        op("registry.show", "Show one spec by id", &["002-registry-query"])
            .cli("registry show", "read-document", &["<id>=ID!", "--json"])
            .finding(&["not-found", "stale"])
            .read("registry-show")
            .facade("query_json", Some("show"), false)
            .example("registry-show", Some("corpus"), &["registry", "show", "001", "--json"], 0, &["001-alpha"])
            .example(
                "registry-show-unknown",
                Some("corpus"),
                &["registry", "show", "999", "--json"],
                1,
                &["\"kind\": \"not-found\""],
            )
            .done(),
        op(
            "registry.status-report",
            "Counts of specs by status",
            &["002-registry-query"],
        )
        .cli("registry status-report", "read-document", &["--json", "--nonzero-only"])
        .finding(&["stale"])
        .read("status-report")
        .facade("query_json", Some("status-report"), false)
        .example(
            "registry-status-report",
            Some("corpus"),
            &["registry", "status-report", "--json"],
            0,
            &["\"schemaVersion\""],
        )
        .pinned_example(&["registry", "status-report"])
        .done(),
        op(
            "scope.compare",
            "Compare two declared scopes for conflicting intentions. Neither resolved against the committed ledger: a pure function of the two documents",
            &["108-a-work-scope-is-declared"],
        )
        .cli("scope compare", "read-document", &["<a>=A!", "<b>=B!", "--json"])
        .reads(&["caller-file", "config"])
        .finding(&["validation"])
        .request_document()
        .read("scope-comparison")
        .facade("scope_compare_json", None, false)
        .example(
            "scope-compare",
            Some("corpus"),
            &["scope", "compare", "scope-a.json", "scope-b.json", "--json"],
            0,
            &["\"schemaVersion\""],
        )
        .pinned_example(&["scope", "compare", "scope-a.json", "scope-b.json"])
        .done(),
        op(
            "scope.evaluate",
            "Resolve a declared scope's paths against the committed index and report where the declaration and the ownership disagree",
            &["108-a-work-scope-is-declared"],
        )
        .cli("scope evaluate", "read-document", &["--scope=FILE!", "--json"])
        .reads(&["caller-file", "config", "corpus", "derived-ledger", "stdin"])
        .finding(&["not-found", "stale", "validation"])
        .request_document()
        .read("scope-evaluation")
        .facade("scope_json", None, true)
        .example(
            "scope-evaluate",
            Some("corpus"),
            &["scope", "evaluate", "--scope", "scope-a.json", "--json"],
            0,
            &["\"schemaVersion\""],
        )
        .pinned_example(&["scope", "evaluate", "--scope", "scope-a.json"])
        .done(),
        op(
            "verify",
            "Run a spec's declared acceptance: the `verify:cli` commands under its `## Verification` heading, in order, stopping at the first failure",
            &["043-verify-declared-acceptance"],
        )
        .cli("verify", "verdict-envelope", verify_flags())
        .reads(&["config", "corpus"])
        .runs_declared_commands()
        .environment(&["SPEC_SPINE_VERIFY_STACK"])
        .finding(&["not-found", "validation"])
        .verdict()
        .example("verify-pass", Some("corpus"), &["verify", "001", "--json"], 0, &["\"verb\": \"verify\""])
        .example("verify-fail", Some("corpus"), &["verify", "002", "--json"], 1, &["\"outcome\": \"finding\""])
        .done(),
        op(
            "verify.affected",
            "Run a spec's declared acceptance: the `verify:cli` commands under its `## Verification` heading, in order, stopping at the first failure",
            &["158-the-affected-selection-is-a-governed-read"],
        )
        .cli("verify", "read-document", verify_flags())
        .reads(&["clock", "config", "corpus", "derived-ledger", "source-tree"])
        .writes(&["temporary"])
        .executes(&["git"])
        .pre("git-repository", &[4])
        .pre("refs-resolvable", &[1, 4])
        .finding(&["not-found", "validation"])
        .response(None, "read", "affected-selection", schema_id("affected.schema.json"))
        .facade("affected_json", None, false)
        .example(
            "verify-affected",
            Some("corpus"),
            &["verify", "--affected-by", "HEAD", "--plan", "--json"],
            0,
            &["\"selected\""],
        )
        .pinned_example(&["verify", "--affected-by", "HEAD", "--plan"])
        .done(),
        op(
            "verify.plan",
            "Run a spec's declared acceptance: the `verify:cli` commands under its `## Verification` heading, in order, stopping at the first failure",
            &["043-verify-declared-acceptance"],
        )
        .cli("verify", "verdict-envelope", verify_flags())
        .reads(&["config", "corpus"])
        .finding(&["not-found", "validation"])
        .verdict()
        .facade("verify_plan_json", None, true)
        .example("verify-plan", Some("corpus"), &["verify", "002", "--plan"], 0, &["exit 1"])
        .pinned_example(&["verify", "002", "--plan"])
        .done(),
        op(
            "verify-attestation",
            "Verify a corpus attestation by recompute and/or detached signature",
            &["021-ledger-seal", "039-per-spec-attestation", "070-an-authority-snapshot-says-what-it-read"],
        )
        .cli(
            "verify-attestation",
            "verdict-envelope",
            &[
                "--spec=ID",
                "--snapshot",
                "--recompute",
                "--signature",
                "--attestation=PATH",
                "--public-key=PATH",
                "--seal=PATH",
                "--json",
            ],
        )
        .reads(&["attestation", "config", "corpus", "derived-ledger", "source-tree"])
        .when("--signature", &["key-material"], &["signature-verification"])
        .finding(&["not-found", "stale", "validation"])
        .verdict()
        .facade("verify_attestation_json", None, true)
        .facade("verify_snapshot_attestation_json", None, true)
        .facade("verify_spec_attestation_json", None, true)
        .example(
            "verify-attestation-absent",
            Some("corpus"),
            &["verify-attestation", "--recompute", "--json"],
            4,
            &["\"kind\": \"io\""],
        )
        .pinned_example(&["verify-attestation", "--recompute"])
        .done(),
        library(
            "library.load-config",
            "Parse a spec-spine.toml and return the normalized configuration as JSON",
            &["001-compile-registry"],
        )
        .request_document()
        .response(None, "config", "config", None)
        .facade("load_config_json", None, false)
        .example_stdin(
            "library-load-config",
            None,
            &[],
            "[layout]\nspecs_dir = \"specs\"\n",
            0,
            &["\"specs_dir\""],
        )
        .example_stdin("library-load-config-unknown-key", None, &[], "[nope]\nx = 1\n", 2, &[])
        .done(),
        library(
            "library.scaffold-init",
            "Generate the governance scaffold for a configuration, as files-as-data the caller writes",
            &["092-the-engine-ships-governance-not-an-environment"],
        )
        .request_document()
        .response(None, "unversioned", "scaffold", None)
        .facade("scaffold_init_json", None, false)
        .example_stdin("library-scaffold-init", None, &[], "{}", 0, &["spec-spine.toml"])
        .example_stdin("library-scaffold-init-bad-config", None, &[], "{\"bogus\":1}", 2, &[])
        .done(),
        library(
            "library.scaffold-init-opts",
            "Generate the governance scaffold with an options document, as files-as-data the caller writes",
            &["131-a-scaffold-can-pin-its-producer"],
        )
        .request_document()
        .response(None, "unversioned", "scaffold", None)
        .facade("scaffold_init_opts_json", None, false)
        .example_stdin(
            "library-scaffold-init-opts",
            None,
            &[],
            r#"{"config":{},"options":{"pinExactVersion":true}}"#,
            0,
            &["required_version"],
        )
        .example_stdin(
            "library-scaffold-init-opts-unknown-option",
            None,
            &[],
            r#"{"config":{},"options":{"nope":true}}"#,
            2,
            &[],
        )
        .done(),
    ]
}

/// The catalog, sorted and digested (162 §3.1, §3.11). Pure: the same build
/// answers the same bytes everywhere.
pub fn capability_catalog() -> Result<CapabilityCatalog, Error> {
    let mut ops = operations();
    let axes = compatibility();
    for o in &mut ops {
        if let Some(r) = o
            .response
            .iter()
            .find(|r| r.axis != "unversioned" && !axes.contains_key(&r.axis))
        {
            return Err(Error::Internal(format!(
                "operation `{}` answers on axis `{}`, which compatibility() does not declare",
                o.name, r.axis
            )));
        }
        o.operation_digest = operation_digest(o)?;
    }
    ops.sort_by(|a, b| a.name.cmp(&b.name));
    let mut catalog = CapabilityCatalog {
        schema_version: CATALOG_SCHEMA_VERSION.to_string(),
        tool: spec_spine_types::verdict::TOOL.to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        compatibility: compatibility(),
        discovery: CatalogDiscovery {
            interface_references: InterfaceDiscovery {
                declared_by: "registry.show".to_string(),
                member: "interfaceReferences".to_string(),
                pin_sources: strings(&["contentHash", "sectionDigests"]),
                verified_by: "interface.verify".to_string(),
            },
            capabilities: CapabilitiesDiscovery {
                verified_by: "capabilities.verify".to_string(),
            },
        },
        operations: ops,
        catalog_digest: String::new(),
    };
    // 162 §3.11: discovery names operations, so each name must be one.
    let d = &catalog.discovery;
    for name in [
        &d.interface_references.declared_by,
        &d.interface_references.verified_by,
        &d.capabilities.verified_by,
    ] {
        if !catalog.operations.iter().any(|o| &o.name == name) {
            return Err(Error::Internal(format!(
                "discovery names `{name}`, which is not an operation in the catalog"
            )));
        }
    }
    catalog.catalog_digest = catalog_digest(&catalog)?;
    Ok(catalog)
}

/// One operation record by name, or [`Error::NotFound`].
pub fn capability_operation(name: &str) -> Result<Operation, Error> {
    capability_catalog()?
        .operations
        .into_iter()
        .find(|o| o.name == name)
        .ok_or_else(|| {
            Error::NotFound(format!(
                "no operation named `{name}` in this binary's catalog"
            ))
        })
}

/// Check pinned operation digests against this build (162 §3.11). A digest
/// that is not `sha256:` plus 64 lowercase hex digits is usage.
pub fn capability_verify(
    request: &CapabilityVerifyRequest,
) -> Result<CapabilityVerifyReport, Error> {
    if request.expect.is_empty() {
        return Err(Error::Usage(
            "no pin to verify: give at least one `<name>=sha256:<hex>`".to_string(),
        ));
    }
    for (name, digest) in &request.expect {
        if !is_digest(digest) {
            return Err(Error::Usage(format!(
                "pin for `{name}` is not `sha256:` plus 64 lowercase hex digits"
            )));
        }
    }
    let catalog = capability_catalog()?;
    let by_name: BTreeMap<&str, &Operation> = catalog
        .operations
        .iter()
        .map(|o| (o.name.as_str(), o))
        .collect();
    let results = request
        .expect
        .iter()
        .map(|(name, expected)| {
            let observed = by_name
                .get(name.as_str())
                .map(|o| o.operation_digest.clone());
            let outcome = match &observed {
                None => "missing",
                Some(d) if d == expected => "current",
                Some(_) => "changed",
            };
            PinResult {
                name: name.clone(),
                outcome: outcome.to_string(),
                expected: expected.clone(),
                observed,
            }
        })
        .collect();
    Ok(CapabilityVerifyReport {
        catalog_digest: catalog.catalog_digest,
        results,
    })
}

fn is_digest(s: &str) -> bool {
    s.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
