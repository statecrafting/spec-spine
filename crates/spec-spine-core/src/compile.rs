//! The compile capability (spec 001): markdown corpus → deterministic registry.
//!
//! Pure function of `(config, file contents)`. Per-spec parse failures are
//! recorded as error-tier violations rather than aborting the run; `Err` is
//! reserved for I/O failures. The wall clock is never read here; it lives in
//! `build-meta.json`, written by the CLI.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use spec_spine_types::{
    Build, Config, Error, Frontmatter, FrontmatterIssue, REGISTRY_SCHEMA_VERSION, Registry,
    RegistrySpecShard, Severity, SpecRecord, Status, Unit, ValidationReport, Violation,
    parse_frontmatter_with, split_frontmatter,
};

use crate::index::Freshness;
use crate::{canonical_json, hash, markdown, shard};

/// Validation codes whose verdict depends on the whole corpus, not one spec:
/// duplicate id/prefix, dangling/unresolved edge targets, and a `depends_on`
/// cycle. They are NOT stored per registry shard (storing them would let a
/// sibling spec's PR stale this shard); they are recomputed from the assembled
/// record set on read.
const CROSS_SPEC_CODES: &[&str] = &[
    "V-003", "V-004", "V-008", "V-010", "V-014",
    // Spec 063 §3.5: the three planned-territory collisions. Each needs to know
    // something about a spec other than the one being validated, so each is
    // corpus-wide and none belongs on a single spec's shard.
    "V-015", "V-016", "V-017",
    // Spec 109 §3.4: an impact/conflict reference that must resolve a spec id
    // against the corpus (dangling, self-referencing, duplicate once short ids
    // are normalized, and `settled_by`).
    "V-028", "V-029", "V-030", "V-031",
    // Spec 111 §3.2: a move's `answered_by` that names no spec in the corpus.
    // A warning, like `depends_on`'s V-010, not an error (D-3): informational,
    // never authority-moving.
    "V-041",
];

/// The cap on **undeclared** `extra_frontmatter` keys before `V-007` fires.
/// Keys listed in `frontmatter.extra_known_keys` are intentional and exempt;
/// the cap targets escape-hatch abuse (ported from OAP's ~8-entry V-002 cap).
pub const MAX_UNDECLARED_EXTRA_FRONTMATTER: usize = 8;

/// The result of a compile: the typed registry, its canonical JSON bytes, the
/// validation flag the CLI maps to an exit code, and the per-spec shard
/// projection the CLI writes to disk (spec 022).
///
/// `registry` + `json` are the aggregate in-memory view (consumed by `attest`,
/// the JSON facade, and the conformance test); `shards` is the committed form.
pub struct CompileOutcome {
    pub registry: Registry,
    pub json: String,
    pub validation_passed: bool,
    pub shards: RegistryShardSet,
}

impl CompileOutcome {
    /// How many warning-tier violations this compile produced (spec 064 §3.2).
    ///
    /// Exposed here rather than left to each caller's own filter so the CLI,
    /// the facade and `check_report` read one number. `validation_passed` is
    /// deliberately untouched by this: spec 001 §3.2 fixes it as "false iff any
    /// error-tier violation is present", and `--fail-on-warn` gates the exit
    /// code, never the verdict field or an emitted byte.
    pub fn warning_count(&self) -> usize {
        self.registry
            .validation
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Warning)
            .count()
    }
}

/// The committed-form projection of a registry: one shard per spec (spec 022),
/// sorted by id for determinism.
pub struct RegistryShardSet {
    pub spec_shards: Vec<RegistrySpecShard>,
}

/// Compile the spec corpus under `repo_root` into a registry.
///
/// Returns `Err` only on I/O failure (unreadable specs dir / file). Validation
/// failures are carried inside `registry.validation` with
/// `validation_passed == false`.
pub fn compile(cfg: &Config, repo_root: &Path) -> Result<CompileOutcome, Error> {
    let specs_dir = repo_root.join(&cfg.layout.specs_dir);
    let mut violations: Vec<Violation> = Vec::new();
    let mut hash_pieces: Vec<(String, String)> = Vec::new();

    // --- discover NNN-slug/spec.md, sorted by directory name ---
    let mut spec_files: Vec<(String, PathBuf)> = Vec::new();
    let entries = fs::read_dir(&specs_dir).map_err(|e| {
        Error::Io(format!(
            "cannot read specs dir {}: {e}",
            specs_dir.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| Error::Io(e.to_string()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let spec_md = path.join("spec.md");
        if spec_md.is_file() {
            spec_files.push((entry.file_name().to_string_lossy().into_owned(), spec_md));
        }
    }
    spec_files.sort();

    // --- parse pass (records every file into the content hash) ---
    struct Parsed {
        dirname: String,
        spec_path: String,
        fm: Frontmatter,
        body: String,
    }
    let mut parsed: Vec<Parsed> = Vec::new();

    for (dirname, spec_md) in &spec_files {
        let raw = fs::read_to_string(spec_md)
            .map_err(|e| Error::Io(format!("read {}: {e}", spec_md.display())))?;
        let spec_path = rel_posix(repo_root, spec_md);
        hash_pieces.push((spec_path.clone(), raw.clone()));

        match parse_frontmatter_with(&raw, &cfg.frontmatter.extra_known_keys) {
            Ok(fm) => {
                let body = split_frontmatter(&raw).map(|(_, b)| b).unwrap_or_default();
                parsed.push(Parsed {
                    dirname: dirname.clone(),
                    spec_path,
                    fm,
                    body,
                });
            }
            // V-013 (spec 012 §3.3): a DECLARED extra key carrying a value
            // JSON cannot represent. Same skip-and-continue semantics as
            // V-002 (001 §3.1).
            Err(FrontmatterIssue::UnrepresentableDeclared { key, detail }) => {
                violations.push(error(
                    "V-013",
                    format!(
                        "declared extra-frontmatter key '{key}' carries an unrepresentable YAML value: {detail}"
                    ),
                    Some(spec_path),
                ));
            }
            Err(FrontmatterIssue::Malformed(m)) => violations.push(error(
                "V-002",
                format!("malformed frontmatter: {m}"),
                Some(spec_path),
            )),
        }
    }

    // --- cross-spec sets ---
    let all_ids: std::collections::BTreeSet<String> =
        parsed.iter().map(|p| p.fm.id.clone()).collect();
    let id_paths: Vec<(String, String)> = parsed
        .iter()
        .map(|p| (p.fm.id.clone(), p.spec_path.clone()))
        .collect();
    detect_duplicates(&id_paths, &mut violations);

    // --- short-id resolution (spec 015): rewrite a depends_on / superseded_by
    // reference that names a spec by its leading number (`109`) to the full id
    // (`109-slug`), before validation (V-008/V-010) and record construction see
    // it. A genuinely dangling or ambiguous reference is left unchanged, so its
    // V-code still fires. Resolution is a pure function of the id set, so the
    // registry stays deterministic.
    for p in &mut parsed {
        for dep in &mut p.fm.depends_on {
            *dep = resolve_spec_ref(dep, &all_ids);
        }
        if let Some(by) = p.fm.superseded_by.as_mut() {
            *by = resolve_spec_ref(by, &all_ids);
        }
        // The predecessor named by a `supersedes` item may use a short id too
        // (spec 018), so the gate's predecessor→superseder transfer keys match.
        for item in &mut p.fm.supersedes {
            let resolved = resolve_spec_ref(item.spec(), &all_ids);
            item.set_spec(resolved);
        }
        // Spec 109 §3.7: an impact/conflict `obligation` reference's spec half
        // normalizes the same way; an unqualified reference, or one whose spec
        // half does not resolve, is left unchanged (`V-025`/`V-028` report
        // those). `settled_by` is a bare spec id, normalized like `depends_on`.
        for imp in &mut p.fm.impacts {
            imp.obligation = normalize_obligation_ref(&imp.obligation, &all_ids);
        }
        for c in &mut p.fm.conflicts {
            c.obligation = normalize_obligation_ref(&c.obligation, &all_ids);
            if let Some(by) = c.settled_by.as_mut() {
                *by = resolve_spec_ref(by, &all_ids);
            }
        }
        // Spec 111 §3.2: `answered_by` is a bare spec id, normalized the same
        // way as `depends_on` and `settled_by`.
        for mv in &mut p.fm.moves {
            if let Some(by) = mv.answered_by.as_mut() {
                *by = resolve_spec_ref(by, &all_ids);
            }
        }
    }

    // --- per-spec validation + record construction ---
    let raw_by_path: BTreeMap<String, String> = hash_pieces.iter().cloned().collect();
    let mut records: Vec<SpecRecord> = Vec::new();
    for p in parsed {
        validate_spec(
            cfg,
            &p.dirname,
            &p.spec_path,
            &p.fm,
            &all_ids,
            &mut violations,
        );
        validate_obligations(&p.spec_path, &p.fm, &p.body, &mut violations);
        validate_impacts_local(&p.spec_path, &p.fm, &mut violations);
        validate_interface_references(&p.spec_path, &p.fm, &mut violations);
        validate_intent(&p.spec_path, &p.fm, &mut violations);
        validate_moves_local(&p.spec_path, &p.fm, &mut violations);
        records.push(build_record(p.fm, p.spec_path, &p.body));
    }
    records.sort_by(|a, b| a.id.cmp(&b.id));
    // V-014 reads the records, not the parsed frontmatter, so it sees the
    // short-id-resolved `depends_on` (spec 015) rather than the authored text.
    detect_dependency_cycle(&records, &mut violations);
    // Spec 082 3.1, 3.3.
    detect_amends_verification(&records, &mut violations);
    // Spec 063 §3.5, over DECLARED units at compile time rather than over the
    // resolved graph. Naming the stage matters, because the obvious reading is
    // wrong: the existing duplicate-ownership machinery operates on
    // `TraceMapping`, and §3.2 excludes a planned unit from ever producing one,
    // so a collision between two planned claims would have nothing to ride on.
    // A rule that cannot fire is worse than no rule, because the spec would
    // promise a refusal that never happens.
    validate_planned_territory(&records, &mut violations);
    // Spec 109 §3.4 rules 2-4, and the `settled_by` rule of §3.3: the
    // impact/conflict checks that must resolve a spec id against the corpus.
    detect_impact_cross_spec(&records, &mut violations);
    // Spec 111 §3.2: a move's `answered_by` that must resolve a spec id
    // against the corpus.
    detect_move_cross_spec(&records, &mut violations);

    // --- shard projection + aggregate content hash (spec 022) ---
    // One shard per spec, each carrying its compiled record, its corpus-
    // independent ("local") violations, and a hash over its `spec.md` (the
    // registry's only hashed input, matching the pre-shard `build.contentHash`).
    // The aggregate `build.contentHash` is the fold of those per-shard hashes,
    // recomputable on read from the shard set and never committed to one file.
    let mut spec_shards: Vec<RegistrySpecShard> = Vec::new();
    let mut keyed: Vec<(String, String)> = Vec::new();
    for record in &records {
        let raw = raw_by_path
            .get(&record.spec_path)
            .cloned()
            .unwrap_or_default();
        let shard_hash = hash::content_hash(vec![(record.spec_path.clone(), raw)]);
        let local_violations: Vec<Violation> = violations
            .iter()
            .filter(|v| {
                !CROSS_SPEC_CODES.contains(&v.code.as_str())
                    && v.path.as_deref() == Some(record.spec_path.as_str())
            })
            .cloned()
            .collect();
        keyed.push((format!("spec:{}", record.id), shard_hash.clone()));
        spec_shards.push(RegistrySpecShard {
            spec_version: REGISTRY_SCHEMA_VERSION.to_string(),
            shard_hash,
            record: record.clone(),
            local_violations,
        });
    }

    // --- assemble registry ---
    let validation = ValidationReport::from_violations(violations);
    let validation_passed = validation.passed;
    let registry = Registry {
        spec_version: REGISTRY_SCHEMA_VERSION.to_string(),
        build: Build {
            compiler_id: cfg.branding.compiler_id.clone(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            input_root: ".".to_string(),
            content_hash: shard::aggregate_content_hash(&keyed),
        },
        specs: records,
        validation,
    };
    let json = canonical_json::to_string(&registry)?;

    Ok(CompileOutcome {
        registry,
        json,
        validation_passed,
        shards: RegistryShardSet { spec_shards },
    })
}

/// Per-spec validation codes V-001/005/006/007/008/009/010/011/012 (V-013 is
/// emitted at the parse stage above).
fn validate_spec(
    cfg: &Config,
    dirname: &str,
    spec_path: &str,
    fm: &Frontmatter,
    all_ids: &std::collections::BTreeSet<String>,
    out: &mut Vec<Violation>,
) {
    let at = || Some(spec_path.to_string());

    // V-012: id pattern.
    if !valid_id(&fm.id) {
        out.push(error(
            "V-012",
            format!(
                "id '{}' does not match ^[0-9]{{3}}-[a-z0-9]+(-[a-z0-9]+)*$",
                fm.id
            ),
            at(),
        ));
    }
    // V-001: directory name equals id.
    if dirname != fm.id {
        out.push(error(
            "V-001",
            format!("directory '{dirname}' does not equal id '{}'", fm.id),
            at(),
        ));
    }
    // V-005: domain allowlist (only when configured non-empty).
    if let Some(domain) = &fm.domain {
        if !cfg.domains.permits(domain) {
            out.push(error(
                "V-005",
                format!("domain '{domain}' is not in domains.allowed"),
                at(),
            ));
        }
    }
    // V-006: kind allowlist (only when configured non-empty).
    if let Some(kind) = &fm.kind {
        if !cfg.kind.permits(kind) {
            out.push(error(
                "V-006",
                format!("kind '{kind}' is not in kind.allowed"),
                at(),
            ));
        }
    }
    // V-007: undeclared extra_frontmatter count cap.
    let undeclared = fm
        .extra_frontmatter
        .keys()
        .filter(|k| !cfg.frontmatter.extra_known_keys.contains(k))
        .count();
    if undeclared > MAX_UNDECLARED_EXTRA_FRONTMATTER {
        out.push(error(
            "V-007",
            format!(
                "{undeclared} undeclared extra-frontmatter keys exceed the cap of {MAX_UNDECLARED_EXTRA_FRONTMATTER} (declare them in frontmatter.extra_known_keys or model them)"
            ),
            at(),
        ));
    }
    // V-008 / V-009: lifecycle requirements.
    if fm.status == Status::Superseded {
        match &fm.superseded_by {
            Some(by) if all_ids.contains(by) => {}
            Some(by) => out.push(error(
                "V-008",
                format!("superseded_by '{by}' does not resolve to an existing spec"),
                at(),
            )),
            None => out.push(error(
                "V-008",
                "status is 'superseded' but superseded_by is missing".into(),
                at(),
            )),
        }
    }
    if fm.status == Status::Retired && fm.retirement_rationale.is_none() {
        out.push(error(
            "V-009",
            "status is 'retired' but retirement_rationale is missing".into(),
            at(),
        ));
    }
    // V-010 (warning): dangling depends_on.
    for dep in &fm.depends_on {
        if !all_ids.contains(dep) {
            out.push(warning(
                "V-010",
                format!("depends_on '{dep}' does not resolve to an existing spec"),
                at(),
            ));
        }
    }
    // V-011: a constrains item must scope something (spec 017): a code `unit`
    // (path-scoped, e.g. invariant-freeze) or `target_specs` (spec-scoped, e.g.
    // a sequencing plan). An item with neither asserts an invariant over nothing.
    for c in &fm.constrains {
        if c.unit.is_none() && c.target_specs.is_empty() {
            out.push(error(
                "V-011",
                "constrains item declares neither a unit nor target_specs".into(),
                at(),
            ));
        }
    }
}

/// The leading run of ASCII decimal digits of `id`, as a sub-slice of it.
///
/// This is the `NNN` of spec 001 §3.2, read by character. Slicing `&id[..3]`
/// outright panics when byte 3 falls inside a multi-byte character, which is
/// the defect spec 059 fixes; counting ASCII digits can only ever stop on a
/// character boundary, because every byte it accepts is a one-byte character.
/// An id with no leading digit yields `""`, which spec 059 §3.2 defines as
/// having no numeric prefix at all. `lint.rs::ordinal` reads the same run and
/// parses it, because ordering is a different question about these characters.
fn numeric_prefix(id: &str) -> &str {
    let end = id.bytes().take_while(u8::is_ascii_digit).count();
    &id[..end]
}

/// V-003 (duplicate id) and V-004 (duplicate numeric prefix). `specs` is
/// `(id, spec_path)` pairs in discovery order.
fn detect_duplicates(specs: &[(String, String)], out: &mut Vec<Violation>) {
    let mut id_counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut prefix_owner: BTreeMap<&str, &str> = BTreeMap::new();
    for (id, spec_path) in specs {
        *id_counts.entry(id.as_str()).or_insert(0) += 1;
        // Spec 059 §3.2: an id with no numeric prefix shares one with nothing,
        // so it is not a V-004 candidate. V-012 is what refuses it.
        let prefix = numeric_prefix(id);
        if prefix.is_empty() {
            continue;
        }
        match prefix_owner.get(prefix) {
            Some(other) if *other != id.as_str() => out.push(error(
                "V-004",
                format!("numeric prefix '{prefix}' is shared by '{other}' and '{id}'"),
                Some(spec_path.clone()),
            )),
            _ => {
                prefix_owner.entry(prefix).or_insert(id.as_str());
            }
        }
    }
    for (id, count) in id_counts {
        if count > 1 {
            out.push(error(
                "V-003",
                format!("duplicate spec id '{id}' ({count} specs)"),
                None,
            ));
        }
    }
}

/// Spec 063 §3.5: planned territory is subject to the ownership rules that
/// already exist, and this spec introduces no second set.
///
/// All three checks are **corpus-wide**, decided with every spec's frontmatter
/// loaded, because each needs to know something about a spec other than the one
/// being validated: whether another spec plans the same unit, already owns it,
/// or is extending a unit this spec marked planned.
///
/// - **`V-015`** two specs plan the same unit, without `co_authority`: the
///   duplicate-ownership refusal, with the same escape as any shared claim.
/// - **`V-016`** a spec plans a unit another spec already owns. The correct
///   declaration there is an `extends` edge naming that spec and unit, which is
///   what crossing into owned territory has always meant.
/// - **`V-017`** an `extends` edge names a planned unit. There is nothing to
///   extend: the unit does not exist and its planner does not yet own it.
fn validate_planned_territory(specs: &[SpecRecord], out: &mut Vec<Violation>) {
    use std::collections::{BTreeMap, BTreeSet};

    // Subjects, so the flag is never part of the identity being compared.
    let key = |u: &Unit| unit_identity(&u.subject());

    // Keyed by unit identity, valued by the specs making the claim. `owners`
    // holds unplanned claims and carries the claimant, because `extends` is
    // itself an ownership-bearing edge: without knowing WHO owns a unit, the
    // V-017 check below would be defeated by the very edge it is examining.
    let mut planners: BTreeMap<String, Vec<&SpecRecord>> = BTreeMap::new();
    let mut owners: BTreeMap<String, Vec<&SpecRecord>> = BTreeMap::new();
    let mut shared: BTreeSet<String> = BTreeSet::new();
    for spec in specs {
        for c in &spec.co_authority {
            shared.insert(key(&c.unit));
        }
        for unit in owning_units(spec) {
            let k = key(&unit);
            if unit.is_planned() {
                planners.entry(k).or_default().push(spec);
            } else {
                owners.entry(k).or_default().push(spec);
            }
        }
    }

    for (unit_key, plans) in &planners {
        let at = |s: &SpecRecord| Some(s.spec_path.clone());
        if plans.len() > 1 && !shared.contains(unit_key) {
            let ids: Vec<&str> = plans.iter().map(|s| s.id.as_str()).collect();
            out.push(error(
                "V-015",
                format!(
                    "unit '{unit_key}' is planned by more than one spec ({}): declare \
                     co_authority if the claim is genuinely shared",
                    ids.join(", ")
                ),
                at(plans[0]),
            ));
        }
        if let Some(existing) = owners.get(unit_key)
            && !shared.contains(unit_key)
        {
            let owner_ids: Vec<&str> = existing.iter().map(|s| s.id.as_str()).collect();
            out.push(error(
                "V-016",
                format!(
                    "spec '{}' plans unit '{unit_key}', which spec(s) {} already own: \
                     declare an extends edge naming the owning spec and unit instead",
                    plans[0].id,
                    owner_ids.join(", ")
                ),
                at(plans[0]),
            ));
        }
    }

    // V-017 reads `extends` alone: it is the edge whose meaning is "cross into
    // territory another spec owns", and a planned unit is territory nobody owns
    // yet.
    for spec in specs {
        for item in &spec.extends {
            let Some(unit) = &item.unit else { continue };
            let k = key(unit);
            // "Owned by someone else": the extending spec's own edge is what
            // put it in `owners`, so counting it would make this rule silent
            // in exactly the case it exists for.
            let owned_elsewhere = owners
                .get(&k)
                .is_some_and(|os| os.iter().any(|o| o.id != spec.id));
            if planners.contains_key(&k) && !owned_elsewhere {
                out.push(error(
                    "V-017",
                    format!(
                        "spec '{}' extends unit '{k}', which spec '{}' has only planned: \
                         there is nothing to extend until it is written",
                        spec.id, planners[&k][0].id
                    ),
                    Some(spec.spec_path.clone()),
                ));
            }
        }
    }
}

/// Every unit a spec claims through an ownership-bearing edge, for the §3.5
/// collision checks. `references` is excluded (spec 031: a cited file is not a
/// claimed one), and so is `amends`, whose subject is a spec rather than a unit.
fn owning_units(spec: &SpecRecord) -> Vec<Unit> {
    let mut units: Vec<Unit> = spec.establishes.clone();
    units.extend(spec.extends.iter().filter_map(|i| i.unit.clone()));
    units.extend(spec.refines.iter().filter_map(|i| i.unit.clone()));
    units.extend(spec.supersedes.iter().filter_map(|i| match i {
        spec_spine_types::SupersedeItem::Scoped(s) => s.unit.clone(),
        _ => None,
    }));
    units.extend(spec.co_authority.iter().map(|i| i.unit.clone()));
    units.extend(spec.constrains.iter().filter_map(|i| i.unit.clone()));
    units
}

/// A unit's identity as a comparable, human-readable key.
fn unit_identity(unit: &Unit) -> String {
    match unit {
        Unit::File { path, .. } => format!("file:{path}"),
        Unit::Section { file, anchor, .. } => format!("section:{file}#{anchor}"),
        Unit::Symbol { id, .. } => format!("symbol:{id}"),
        Unit::Directory { path, .. } => format!("directory:{path}"),
        Unit::Crate { id, .. } => format!("crate:{id}"),
        Unit::Module { id, .. } => format!("module:{id}"),
    }
}

/// V-014 (error): a cycle in the `depends_on` graph (spec 030).
///
/// spec-spine attaches no scheduling semantics to `depends_on`, but a cycle is
/// a corpus defect under any reading: the edge says one spec's authority rests
/// on another's, and a cycle asserts that of every spec on it at once. It is
/// also the one `depends_on` defect its sibling check cannot see, because every
/// id on a cycle resolves; V-010 fires only on a dangling target.
///
/// Iterative DFS over an explicit stack, so a deep corpus cannot overflow the
/// real one. Edges leaving the corpus are skipped (V-010's finding, not this
/// one). Start order is `records` order, which both callers sort by id, and
/// child order is authored order, so which cycle is found is a pure function of
/// the corpus. One cycle is reported per compile: the path names every spec on
/// it, and breaking it is what reveals any other.
/// Spec 082 §3.1 and §3.3: `amends_verification` must be declarable without
/// ambiguity.
///
/// Three refusals, all errors, because each leaves `verify` with a question it
/// would otherwise have to answer by guessing:
///
/// - `V-018`: an entry that is not also in `amends`. Replacing what a spec
///   accepts changes what that spec requires of the tree, which is an amendment,
///   and spec 037 §3.3 makes the edge set the authoritative record of one. A
///   spec taking over another's acceptance while `amended_by (incoming)` stayed
///   silent would be exactly the undiscoverable amendment 040 exists to prevent.
/// - `V-019`: two live specs naming the same target. Picking one by ordinal is a
///   rule that always answers and never gives a reason; constitution V makes a
///   disagreement about authority a question for a person.
/// - `V-020`: a cycle in the replacement chain, which has no fixed point to
///   resolve to.
///
/// A `superseded` or `retired` amender is skipped throughout (§3.2, D-5): its
/// acceptance is no longer the corpus's, so it neither claims a target nor
/// occupies one.
fn detect_amends_verification(records: &[SpecRecord], out: &mut Vec<Violation>) {
    let live = |r: &SpecRecord| !matches!(r.status, Status::Superseded | Status::Retired);
    let all_ids: std::collections::BTreeSet<&str> = records.iter().map(|r| r.id.as_str()).collect();

    // V-018, and the claim map V-019 reads. Records are id-sorted, so the first
    // claimant of a target is stable and the message names the pair in a fixed
    // order.
    let mut claimed: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    for r in records.iter().filter(|r| live(r)) {
        for target in &r.amends_verification {
            if !r.amends.iter().any(|a| a == target) {
                out.push(error(
 "V-018",
 format!(
 "amends_verification names '{target}', which is not in amends: replacing a spec's `## Verification` block amends that spec, so the edge that records the amendment must be declared too (spec 082 3.1)"
 ),
 Some(r.spec_path.clone()),
 ));
            }
            if !all_ids.contains(target.as_str()) {
                continue; // V-008/V-010 territory; not this rule's to restate.
            }
            match claimed.get(target.as_str()) {
 Some(first) => out.push(error(
 "V-019",
 format!(
 "spec '{}' and spec '{first}' both replace the `## Verification` block of '{target}'; only one spec may hold another's acceptance, and which one is a question about authority rather than a tie to break (spec 082 3.3)",
 r.id
 ),
 Some(r.spec_path.clone()),
 )),
 None => {
 claimed.insert(target.as_str(), r.id.as_str());
 }
 }
        }
    }

    // V-020: a cycle in the chain `target -> holder`. Walking from every target
    // is enough; the map is small and the walk is bounded by its size.
    //
    // One report per cycle, the convention `detect_dependency_cycle` already
    // sets: every node of a cycle is a start, so an unguarded walk reports a
    // two-node cycle twice and an n-node cycle n times, which buries the one
    // fact the reader needs under n copies of it.
    let mut reported: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for start in claimed.keys() {
        if reported.contains(*start) {
            continue;
        }
        let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        let mut at: &str = start;
        while let Some(next) = claimed.get(at) {
            if !seen.insert(at) {
                reported.extend(seen.iter().copied());
                let path = records
                    .iter()
                    .find(|r| r.id == *start)
                    .map(|r| r.spec_path.clone());
                out.push(error(
 "V-020",
 format!(
 "the amends_verification chain starting at '{start}' is a cycle, so it resolves to no block (spec 082 3.3)"
 ),
 path,
 ));
                break;
            }
            at = next;
        }
    }
}

fn detect_dependency_cycle(records: &[SpecRecord], out: &mut Vec<Violation>) {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let ids: std::collections::BTreeSet<&str> = records.iter().map(|r| r.id.as_str()).collect();
    let mut deps: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut spec_paths: BTreeMap<&str, &str> = BTreeMap::new();
    for r in records {
        deps.insert(
            r.id.as_str(),
            r.depends_on
                .iter()
                .map(String::as_str)
                .filter(|d| ids.contains(d))
                .collect(),
        );
        spec_paths.insert(r.id.as_str(), r.spec_path.as_str());
    }
    let mut color: BTreeMap<&str, u8> = ids.iter().map(|id| (*id, WHITE)).collect();

    for r in records {
        // `color` is seeded from `ids`, which is collected from these same
        // `records` four lines up, and no later insert adds a key (every
        // `child` is filtered through `ids`). The fallback is therefore
        // unreachable. It defaults to WHITE rather than BLACK so that if it
        // ever became reachable the node would still be walked: a redundant
        // walk is harmless, whereas defaulting to BLACK would skip the node
        // and miss a cycle rooted there, which is the one outcome a gate must
        // never produce silently.
        debug_assert!(
            color.contains_key(r.id.as_str()),
            "dependency-cycle detector: record id absent from the color map"
        );
        if color.get(r.id.as_str()).copied().unwrap_or(WHITE) != WHITE {
            continue;
        }
        // (spec id, index of the next depends_on entry to walk from it).
        let mut stack: Vec<(&str, usize)> = vec![(r.id.as_str(), 0)];
        color.insert(r.id.as_str(), GREY);
        while let Some(&(node, at)) = stack.last() {
            let Some(child) = deps.get(node).and_then(|d| d.get(at)).copied() else {
                color.insert(node, BLACK);
                stack.pop();
                continue;
            };
            if let Some(top) = stack.last_mut() {
                top.1 = at + 1;
            }
            match color.get(child).copied().unwrap_or(BLACK) {
                WHITE => {
                    color.insert(child, GREY);
                    stack.push((child, 0));
                }
                GREY => {
                    // `child` is still on the current path, so everything from
                    // its position up to the top of the stack is the cycle;
                    // naming it once more closes the path into a loop.
                    //
                    // GREY holds exactly while a node is on the current stack:
                    // set on push, cleared to BLACK on pop. The search
                    // therefore always hits, and `debug_assert` makes a broken
                    // invariant fail `cargo test` loudly. Release does not
                    // panic (that would abort the gate outside its documented
                    // exit codes) and does not invent a path either: reaching
                    // this arm proves a cycle through `child` exists, so the
                    // fallback reports that much and drops only the precision
                    // it can no longer vouch for.
                    let entry = stack.iter().position(|&(n, _)| n == child);
                    debug_assert!(
                        entry.is_some(),
                        "dependency-cycle detector: GREY node absent from the stack"
                    );
                    // `child` is the spec the cycle re-enters, so it is on the
                    // cycle whichever branch below runs.
                    let at_path = spec_paths.get(child).map(|p| p.to_string());
                    let message = match entry {
                        Some(from) => {
                            let mut cycle: Vec<&str> =
                                stack[from..].iter().map(|&(n, _)| n).collect();
                            cycle.push(child);
                            format!("depends_on cycle: {}", cycle.join(" -> "))
                        }
                        None => format!("depends_on cycle through '{child}'"),
                    };
                    out.push(error("V-014", message, at_path));
                    return;
                }
                _ => {}
            }
        }
    }
}

/// Build a `SpecRecord` from parsed frontmatter, copying `extra_frontmatter`
/// verbatim so downstream-specific keys reach `registry.json` (the overlay seam).
fn build_record(fm: Frontmatter, spec_path: String, body: &str) -> SpecRecord {
    let spec_path_for_digests = spec_path.clone();
    SpecRecord {
        id: fm.id,
        title: fm.title,
        status: fm.status,
        created: fm.created,
        summary: fm.summary,
        spec_path,
        authors: fm.authors,
        owner: fm.owner,
        kind: fm.kind,
        domain: fm.domain,
        risk: fm.risk,
        implementation: fm.implementation,
        depends_on: fm.depends_on,
        code_aliases: fm.code_aliases,
        feature_branch: fm.feature_branch,
        section_headings: markdown::section_headings(body),
        establishes: fm.establishes,
        extends: fm.extends,
        refines: fm.refines,
        supersedes: fm.supersedes,
        amends: fm.amends,
        co_authority: fm.co_authority,
        constrains: fm.constrains,
        references: fm.references,
        superseded_by: fm.superseded_by,
        retirement_rationale: fm.retirement_rationale,
        amends_sections: fm.amends_sections,
        amends_verification: fm.amends_verification,
        unamendable: fm.unamendable,
        amendment_record: fm.amendment_record,
        origin: fm.origin,
        section_digests: section_digests(&spec_path_for_digests, body),
        obligations: fm.obligations,
        impacts: fm.impacts,
        conflicts: fm.conflicts,
        interface_references: fm.interface_references,
        intent: fm.intent.map(Into::into),
        moves: fm.moves,
        extra_frontmatter: fm.extra_frontmatter,
    }
}

/// Spec 114 §3.4: the one intent rule serde cannot see. A missing `goal`, a
/// wrong type or an unknown member is already malformed frontmatter (`V-002`);
/// what is left is a string that says nothing. `V-039` judges whether the
/// declaration is well formed, never whether it is true (§3.5).
fn validate_intent(spec_path: &str, fm: &Frontmatter, out: &mut Vec<Violation>) {
    let Some(intent) = &fm.intent else {
        return;
    };
    let empty = |member: &str| {
        error(
            "V-039",
            format!(
                "spec '{}' declares an intent whose {member} is empty; a declared \
                 goal or non-goal that says nothing is refused, and the key is optional",
                fm.id
            ),
            Some(spec_path.to_string()),
        )
    };
    if intent.goal.trim().is_empty() {
        out.push(empty("goal"));
    }
    for (i, ng) in intent.non_goals.iter().enumerate() {
        if ng.trim().is_empty() {
            out.push(empty(&format!("non_goals[{i}]")));
        }
    }
}

/// Every body section's digest (spec 106 §3.5), keyed by anchor. The first
/// heading with a given anchor wins, which is the section `resolve_section`
/// answers for that anchor. One hash construction (spec 077): the name is
/// `<specPath>#<anchor>`, so equal text in two specs is two identities.
pub(crate) fn section_digests(spec_path: &str, body: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (anchor, text) in crate::sections::markdown_section_texts(body) {
        if anchor.is_empty() || out.contains_key(&anchor) {
            continue;
        }
        let digest = hash::content_hash(vec![(format!("{spec_path}#{anchor}"), text)]);
        out.insert(anchor, digest);
    }
    out
}

/// Spec 106 §3.3, 3.4, 3.7: the obligation rules that are a pure function of
/// one spec. Malformed members were already refused at parse (`V-002`).
fn validate_obligations(spec_path: &str, fm: &Frontmatter, body: &str, out: &mut Vec<Violation>) {
    if fm.obligations.is_empty() {
        return;
    }
    let at = || Some(spec_path.to_string());
    let mut anchor_counts: BTreeMap<String, usize> = BTreeMap::new();
    for (anchor, _) in crate::sections::markdown_section_texts(body) {
        *anchor_counts.entry(anchor).or_default() += 1;
    }
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for ob in &fm.obligations {
        if !spec_spine_types::valid_obligation_id(&ob.id) {
            out.push(error(
                "V-021",
                format!(
                    "obligation id '{}' does not match ^[A-Za-z][A-Za-z0-9]*(-[A-Za-z0-9]+)*$",
                    ob.id
                ),
                at(),
            ));
        } else if !seen.insert(ob.id.as_str()) {
            out.push(error(
                "V-021",
                format!(
                    "obligation id '{}' is declared twice; an id is unique within its spec, \
                     and a withdrawn obligation keeps its id (withdrawn: true), so it cannot be reused",
                    ob.id
                ),
                at(),
            ));
        }
        match anchor_counts.get(ob.anchor.as_str()).copied().unwrap_or(0) {
            1 => {}
            0 => out.push(error(
                "V-022",
                format!(
                    "obligation '{}' anchor '{}' names no heading in this spec's body",
                    ob.id, ob.anchor
                ),
                at(),
            )),
            n => out.push(error(
                "V-022",
                format!(
                    "obligation '{}' anchor '{}' is ambiguous: {n} headings in this spec's body share it",
                    ob.id, ob.anchor
                ),
                at(),
            )),
        }
        let is_verification = ob.kind == spec_spine_types::ObligationKind::Verification;
        if is_verification && ob.inputs.is_empty() {
            out.push(error(
                "V-023",
                format!(
                    "verification obligation '{}' declares no inputs; its evidence is declared, never inferred",
                    ob.id
                ),
                at(),
            ));
        } else if !is_verification && !ob.inputs.is_empty() {
            out.push(error(
                "V-023",
                format!(
                    "obligation '{}' declares inputs, and only a verification obligation may",
                    ob.id
                ),
                at(),
            ));
        }
        if ob.text.trim().is_empty() {
            out.push(error(
                "V-024",
                format!("obligation '{}' has empty text", ob.id),
                at(),
            ));
        }
    }
}

/// Normalize a qualified obligation reference's spec half to its full id
/// (spec 109 §3.7), mirroring `resolve_spec_ref`'s short-id resolution for
/// `depends_on`. An unqualified reference, or one whose spec half does not
/// resolve, is returned unchanged: `V-025` and `V-028` are what report those,
/// and normalizing a reference that will be refused anyway would hide what the
/// author actually wrote from the violation message.
fn normalize_obligation_ref(
    reference: &str,
    all_ids: &std::collections::BTreeSet<String>,
) -> String {
    match spec_spine_types::split_obligation_ref(reference) {
        Some((spec_part, ob_id)) => format!("{}#{ob_id}", resolve_spec_ref(spec_part, all_ids)),
        None => reference.to_string(),
    }
}

/// Spec 109 §3.4 rule 1, §3.2's successor rules, and §3.3's `reason` rule: the
/// impact/conflict checks that are a pure function of one spec. Malformed
/// members were already refused at parse (`V-002`).
fn validate_impacts_local(spec_path: &str, fm: &Frontmatter, out: &mut Vec<Violation>) {
    if fm.impacts.is_empty() && fm.conflicts.is_empty() {
        return;
    }
    let at = || Some(spec_path.to_string());
    // Live (non-withdrawn) ids this spec declares, for the successor rule
    // (§3.2): a successor must be one of them.
    let live_ids: std::collections::BTreeSet<&str> = fm
        .obligations
        .iter()
        .filter(|o| !o.withdrawn)
        .map(|o| o.id.as_str())
        .collect();
    let withdrawn_ids: std::collections::BTreeSet<&str> = fm
        .obligations
        .iter()
        .filter(|o| o.withdrawn)
        .map(|o| o.id.as_str())
        .collect();

    for imp in &fm.impacts {
        if spec_spine_types::split_obligation_ref(&imp.obligation).is_none() {
            out.push(error(
                "V-025",
                format!(
                    "impact references '{}', which is not a qualified obligation reference \
                     (<spec-id>#<obligation-id>): an unqualified reference is refused, never \
                     resolved locally",
                    imp.obligation
                ),
                at(),
            ));
            // One mistake, one violation (D-8): the successor rules describe a
            // target, and an unqualified reference names none.
            continue;
        }
        match (imp.nature, &imp.successor) {
            (spec_spine_types::ImpactNature::Supersedes, None) => out.push(error(
                "V-026",
                format!(
                    "impact on '{}' is nature 'supersedes' and names no successor: a \
                     supersession with no successor is a deletion wearing a different word",
                    imp.obligation
                ),
                at(),
            )),
            (spec_spine_types::ImpactNature::Supersedes, Some(succ)) => {
                if !live_ids.contains(succ.as_str()) {
                    let why = if withdrawn_ids.contains(succ.as_str()) {
                        "is withdrawn"
                    } else {
                        "is not an obligation this spec declares"
                    };
                    out.push(error(
                        "V-026",
                        format!(
                            "impact on '{}' names successor '{succ}', which {why}",
                            imp.obligation
                        ),
                        at(),
                    ));
                }
            }
            (_, Some(succ)) => out.push(error(
                "V-026",
                format!(
                    "impact on '{}' names successor '{succ}', and only a 'supersedes' impact \
                     may: a member that means nothing on any other nature is a member somebody \
                     will misread",
                    imp.obligation
                ),
                at(),
            )),
            (_, None) => {}
        }
    }

    for c in &fm.conflicts {
        if spec_spine_types::split_obligation_ref(&c.obligation).is_none() {
            out.push(error(
                "V-025",
                format!(
                    "conflict references '{}', which is not a qualified obligation reference \
                     (<spec-id>#<obligation-id>): an unqualified reference is refused, never \
                     resolved locally",
                    c.obligation
                ),
                at(),
            ));
        }
        if c.reason.trim().is_empty() {
            out.push(error(
                "V-027",
                format!("conflict on '{}' has empty reason", c.obligation),
                at(),
            ));
        }
    }
}

/// Spec 110 §3.2: interface-reference rules that are a pure function of one
/// spec (no other spec is consulted, unlike spec 109's impact/conflict
/// resolution, because the cited spec lives in another corpus this compile
/// cannot see). Malformed members were already refused at parse (`V-002`).
fn validate_interface_references(spec_path: &str, fm: &Frontmatter, out: &mut Vec<Violation>) {
    if fm.interface_references.is_empty() {
        return;
    }
    let at = || Some(spec_path.to_string());
    let mut pairs_seen: std::collections::BTreeSet<(&str, &str)> =
        std::collections::BTreeSet::new();
    for r in &fm.interface_references {
        let label = format!("{}#{}", r.corpus, r.spec);

        if !spec_spine_types::valid_corpus_name(&r.corpus) {
            out.push(error(
                "V-032",
                format!(
                    "interface reference '{label}' has corpus '{}', which does not match \
                     ^[a-z0-9][a-z0-9._-]*$",
                    r.corpus
                ),
                at(),
            ));
        }

        if !valid_id(&r.spec) {
            if r.spec.bytes().all(|b| b.is_ascii_digit()) {
                out.push(error(
                    "V-033",
                    format!(
                        "interface reference '{label}' names spec '{}', a short id: a short id \
                         cannot be resolved against another corpus, and a reference must name \
                         the cited spec's full id",
                        r.spec
                    ),
                    at(),
                ));
            } else {
                out.push(error(
                    "V-033",
                    format!(
                        "interface reference '{label}' names spec '{}', which does not match \
                         ^[0-9]{{3}}-[a-z0-9]+(-[a-z0-9]+)*$: a reference must name the cited \
                         spec's full id",
                        r.spec
                    ),
                    at(),
                ));
            }
        }

        if !spec_spine_types::valid_digest(&r.digest) {
            out.push(error(
                "V-034",
                unsupported_digest_message(&format!("interface reference '{label}'"), &r.digest),
                at(),
            ));
        }

        let mut anchors_seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for s in &r.sections {
            if !spec_spine_types::valid_digest(&s.digest) {
                out.push(error(
                    "V-035",
                    unsupported_digest_message(
                        &format!("interface reference '{label}' section '{}'", s.anchor),
                        &s.digest,
                    ),
                    at(),
                ));
            }
            if !anchors_seen.insert(s.anchor.as_str()) {
                out.push(error(
                    "V-037",
                    format!(
                        "interface reference '{label}' pins anchor '{}' twice: anchors are \
                         unique within one reference",
                        s.anchor
                    ),
                    at(),
                ));
            }
        }

        if !spec_spine_types::valid_obtained_date(&r.obtained) {
            out.push(error(
                "V-036",
                format!(
                    "interface reference '{label}' has obtained '{}', which does not match \
                     ^[0-9]{{4}}-[0-9]{{2}}-[0-9]{{2}}$",
                    r.obtained
                ),
                at(),
            ));
        }

        if !pairs_seen.insert((r.corpus.as_str(), r.spec.as_str())) {
            out.push(error(
                "V-038",
                format!(
                    "interface reference '{label}' is declared twice: a spec names one \
                     reference per (corpus, spec) pair"
                ),
                at(),
            ));
        }
    }
}

/// A move path (spec 111 §3.2): non-empty, repo-relative (no leading `/`) and
/// no `..` segment, the same grammar spec 108's scope paths use.
fn check_move_path(p: &str) -> Option<&'static str> {
    if p.is_empty() {
        return Some("is empty");
    }
    if p.starts_with('/') {
        return Some("is absolute: paths are repo-relative");
    }
    if p.split('/').any(|seg| seg == "..") {
        return Some("carries a '..' segment");
    }
    None
}

/// Spec 111 §3.2: shape rules that are a pure function of one spec (no other
/// spec consulted). A `kind` other than the four is already refused at parse
/// (`V-002`), the way an unknown `ImpactNature`/`ObligationKind` is: `kind` is
/// a closed, typed enum, not a raw string this function re-validates.
fn validate_moves_local(spec_path: &str, fm: &Frontmatter, out: &mut Vec<Violation>) {
    if fm.moves.is_empty() {
        return;
    }
    let at = || Some(spec_path.to_string());
    for mv in &fm.moves {
        let from_paths = mv.from.paths();
        let to_paths: Vec<&str> = mv.to.as_ref().map(|t| t.paths()).unwrap_or_default();
        let kind_label = mv.kind.label();

        // Arity vs kind (§3.1).
        let arity_ok = match mv.kind {
            spec_spine_types::MoveKind::Relocated => from_paths.len() == 1 && to_paths.len() == 1,
            spec_spine_types::MoveKind::Split => from_paths.len() == 1 && to_paths.len() >= 2,
            spec_spine_types::MoveKind::Merged => from_paths.len() >= 2 && to_paths.len() == 1,
            spec_spine_types::MoveKind::Removed => from_paths.len() == 1 && mv.to.is_none(),
        };
        if !arity_ok {
            out.push(error(
                "V-040",
                format!(
                    "move (kind '{kind_label}') names {} 'from' path(s) and {} 'to' path(s), \
                     which does not match its kind: relocated is one to one, split is one to \
                     two or more, merged is two or more to one, and removed names one 'from' \
                     and 'to: null'",
                    from_paths.len(),
                    to_paths.len()
                ),
                at(),
            ));
        }

        // Path grammar (§3.2): non-empty, repo-relative, no `..` segment.
        for p in from_paths.iter().chain(to_paths.iter()) {
            if let Some(why) = check_move_path(p) {
                out.push(error("V-040", format!("move path '{p}' {why}"), at()));
            }
        }

        // The same path on both sides of one entry (§3.2).
        for p in &from_paths {
            if to_paths.contains(p) {
                out.push(error(
                    "V-040",
                    format!("move (kind '{kind_label}') names '{p}' on both sides of one entry"),
                    at(),
                ));
            }
        }

        // `answered_by` only under `removed` (§3.1, §3.2).
        if mv.answered_by.is_some() && mv.kind != spec_spine_types::MoveKind::Removed {
            out.push(error(
                "V-040",
                format!(
                    "move (kind '{kind_label}') declares answered_by, which is meaningful \
                     only for a 'removed' entry"
                ),
                at(),
            ));
        }
    }
}

/// Spec 111 §3.2 (D-3): a move's `answered_by` that must resolve a spec id
/// against the corpus, exactly as `depends_on`'s dangling check (`V-010`)
/// does, and at the same warning tier: informational, never authority-moving
/// (§3.5), so a dangling reference here is judged the way `depends_on` and
/// every edge target are, not the way `superseded_by` (`V-008`) is. Corpus-
/// wide, so this code is in [`CROSS_SPEC_CODES`] and
/// [`recompute_cross_spec_violations`] re-derives it from the assembled
/// record set on read.
fn detect_move_cross_spec(records: &[SpecRecord], out: &mut Vec<Violation>) {
    let all_ids: std::collections::BTreeSet<&str> = records.iter().map(|r| r.id.as_str()).collect();
    for r in records {
        for mv in &r.moves {
            let Some(by) = &mv.answered_by else {
                continue;
            };
            if !all_ids.contains(by.as_str()) {
                out.push(warning(
                    "V-041",
                    format!(
                        "spec '{}' move answered_by '{by}' does not resolve to an existing spec",
                        r.id
                    ),
                    Some(r.spec_path.clone()),
                ));
            }
        }
    }
}

/// The digest-grammar refusal message (spec 110 §3.1, §3.2), shared by the
/// reference-level and section-level checks: an unrecognized algorithm prefix
/// ("only sha256: is supported") is a different mistake than a malformed
/// sha256 digest, and a reader correcting the frontmatter needs to know which
/// one they made.
fn unsupported_digest_message(what: &str, digest: &str) -> String {
    match digest.split_once(':') {
        Some(("sha256", _)) => {
            format!("{what} has digest '{digest}', which does not match ^sha256:[0-9a-f]{{64}}$")
        }
        Some((algo, _)) => format!(
            "{what} has digest '{digest}' under algorithm '{algo}': only sha256 is supported"
        ),
        None => format!(
            "{what} has digest '{digest}' with no algorithm prefix: the form is \
             sha256:<64 lowercase hex digits>"
        ),
    }
}

/// Spec 109 §3.4 rules 2-4, and the `settled_by` rule of §3.3: the
/// impact/conflict checks that must resolve a spec id against the corpus,
/// exactly as `depends_on`'s dangling check (`V-010`) does. Corpus-wide, so
/// these codes are in [`CROSS_SPEC_CODES`] and [`recompute_cross_spec_violations`]
/// re-derives them from the assembled records on read.
fn detect_impact_cross_spec(records: &[SpecRecord], out: &mut Vec<Violation>) {
    let by_id: std::collections::BTreeMap<&str, &SpecRecord> =
        records.iter().map(|r| (r.id.as_str(), r)).collect();

    for r in records {
        let at = Some(r.spec_path.clone());

        let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for imp in &r.impacts {
            check_obligation_reference(&imp.obligation, r, &by_id, "impact", out, at.clone());
            if !seen.insert(imp.obligation.as_str()) {
                out.push(error(
                    "V-030",
                    format!(
                        "spec '{}' names '{}' twice in impacts",
                        r.id, imp.obligation
                    ),
                    at.clone(),
                ));
            }
        }

        let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for c in &r.conflicts {
            check_obligation_reference(&c.obligation, r, &by_id, "conflict", out, at.clone());
            if !seen.insert(c.obligation.as_str()) {
                out.push(error(
                    "V-030",
                    format!(
                        "spec '{}' names '{}' twice in conflicts",
                        r.id, c.obligation
                    ),
                    at.clone(),
                ));
            }
            match (c.resolution, &c.settled_by) {
                (spec_spine_types::ConflictResolution::Pending, None) => out.push(error(
                    "V-031",
                    format!(
                        "spec '{}' conflict on '{}' is resolution 'pending' and names no \
                         settled_by",
                        r.id, c.obligation
                    ),
                    at.clone(),
                )),
                (spec_spine_types::ConflictResolution::Pending, Some(by)) => {
                    if !by_id.contains_key(by.as_str()) {
                        out.push(error(
                            "V-031",
                            format!(
                                "spec '{}' conflict on '{}' names settled_by '{by}', which does \
                                 not resolve to an existing spec",
                                r.id, c.obligation
                            ),
                            at.clone(),
                        ));
                    }
                }
                (_, Some(by)) => out.push(error(
                    "V-031",
                    format!(
                        "spec '{}' conflict on '{}' names settled_by '{by}', and only a \
                         'pending' conflict may",
                        r.id, c.obligation
                    ),
                    at.clone(),
                )),
                (_, None) => {}
            }
        }
    }
}

/// One impact/conflict `obligation` reference against the assembled corpus
/// (spec 109 §3.4 rules 2-3): dangling (`V-028`, the spec or the obligation on
/// it does not exist) or self-referencing (`V-029`, withdrawal and
/// supersession inside one spec are what the tombstone is for). Already
/// unqualified references are skipped: `V-025` reported those, and this would
/// only restate it under a different code.
fn check_obligation_reference(
    reference: &str,
    r: &SpecRecord,
    by_id: &std::collections::BTreeMap<&str, &SpecRecord>,
    kind: &str,
    out: &mut Vec<Violation>,
    at: Option<String>,
) {
    let Some((spec_part, ob_id)) = spec_spine_types::split_obligation_ref(reference) else {
        return;
    };
    if spec_part == r.id {
        out.push(error(
            "V-029",
            format!(
                "spec '{}' {kind} '{reference}' names an obligation of the declaring spec \
                 itself: withdrawal and supersession inside one spec are what the tombstone is \
                 for",
                r.id
            ),
            at,
        ));
        return;
    }
    match by_id.get(spec_part) {
        None => out.push(error(
            "V-028",
            format!(
                "spec '{}' {kind} '{reference}' does not resolve: no spec '{spec_part}'",
                r.id
            ),
            at,
        )),
        Some(target) => {
            if !target.obligations.iter().any(|o| o.id == ob_id) {
                out.push(error(
                    "V-028",
                    format!(
                        "spec '{}' {kind} '{reference}' does not resolve: '{spec_part}' \
                         declares no obligation '{ob_id}'",
                        r.id
                    ),
                    at,
                ));
            }
        }
    }
}

// ===== spec 049: validate one spec, write nothing =====

/// One spec's validation verdict (spec 049 §3.2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecCheckReport {
    /// The resolved full id, so a caller that passed `056` sees what it named.
    pub spec_id: String,
    pub spec_path: String,
    /// This spec's violations: the corpus-independent ones, plus the cross-spec
    /// ones this spec is party to. Never another spec's.
    pub violations: Vec<Violation>,
    /// False iff an error-tier violation is present.
    pub passed: bool,
}

/// Validate exactly one spec against the committed registry, writing nothing
/// (spec 049).
///
/// The verb exists because an author with an unfinished draft has had no way to
/// ask whether it parses. `compile` writes, so asking commits the draft to the
/// ledger as a side effect of the question; `compile --check` refuses, because a
/// new draft has no committed shard and the whole run reads as stale. What was
/// left was a `mktemp -d` ritual two adopters wrote down, whose documented
/// gotcha (a `references` edge into `docs/` not resolving in a corpus copy that
/// has no `docs/`) is the tell that a workaround teaching you to disbelieve its
/// own output is a missing feature with extra steps.
///
/// The named spec is read from where it lives, so every path in it resolves the
/// way it will resolve after it lands. Its siblings come from the **committed**
/// registry: the author is asking whether the draft fits the corpus as it
/// stands, and what stands is what was committed.
pub fn compile_spec(cfg: &Config, repo_root: &Path, id: &str) -> Result<SpecCheckReport, Error> {
    let specs_dir = repo_root.join(&cfg.layout.specs_dir);
    // Spec 067 3.4: the one policy, over the ids this verb already reads. A
    // string comparison against the listing, never `specs_dir.join(id)`, so a
    // path-shaped argument is refused rather than walked to (067 3.2, D-6).
    let dirname = crate::spec_id::resolve_spec_id(id, crate::spec_id::spec_dir_ids(&specs_dir)?)?;
    let spec_md = specs_dir.join(&dirname).join("spec.md");
    let raw = fs::read_to_string(&spec_md)
        .map_err(|e| Error::Io(format!("read {}: {e}", spec_md.display())))?;
    let spec_path = rel_posix(repo_root, &spec_md);

    let mut violations: Vec<Violation> = Vec::new();
    let fm = match parse_frontmatter_with(&raw, &cfg.frontmatter.extra_known_keys) {
        Ok(fm) => fm,
        // The same two skip-and-report arms `compile` uses: a spec whose
        // frontmatter did not parse has no record to build, and that is the
        // whole finding.
        Err(FrontmatterIssue::UnrepresentableDeclared { key, detail }) => {
            violations.push(error(
                "V-013",
                format!(
                    "declared extra-frontmatter key '{key}' carries an unrepresentable YAML value: {detail}"
                ),
                Some(spec_path.clone()),
            ));
            return Ok(finish_spec_report(dirname, spec_path, violations));
        }
        Err(FrontmatterIssue::Malformed(m)) => {
            violations.push(error(
                "V-002",
                format!("malformed frontmatter: {m}"),
                Some(spec_path.clone()),
            ));
            return Ok(finish_spec_report(dirname, spec_path, violations));
        }
    };

    // Siblings as committed. A registry that has never been compiled is an
    // empty corpus rather than an error: a first spec in a fresh repository is
    // exactly the case this verb is for.
    let committed: Vec<SpecRecord> = load_committed_registry(cfg, repo_root)
        .map(|r| r.specs)
        .unwrap_or_default();

    let mut fm = fm;
    let mut all_ids: std::collections::BTreeSet<String> =
        committed.iter().map(|r| r.id.clone()).collect();
    all_ids.insert(fm.id.clone());
    // Short-id resolution (spec 015) before validation sees the references, so
    // `V-008` / `V-010` judge the resolved value exactly as `compile` does.
    for dep in &mut fm.depends_on {
        *dep = resolve_spec_ref(dep, &all_ids);
    }
    if let Some(by) = fm.superseded_by.as_mut() {
        *by = resolve_spec_ref(by, &all_ids);
    }
    for item in &mut fm.supersedes {
        let resolved = resolve_spec_ref(item.spec(), &all_ids);
        item.set_spec(resolved);
    }

    validate_spec(cfg, &dirname, &spec_path, &fm, &all_ids, &mut violations);

    let body = split_frontmatter(&raw).map(|(_, b)| b).unwrap_or_default();
    let record = build_record(fm, spec_path.clone(), &body);
    let mut records: Vec<SpecRecord> = committed
        .into_iter()
        .filter(|r| r.id != record.id)
        .collect();
    records.push(record);
    records.sort_by(|a, b| a.id.cmp(&b.id));

    // The cross-spec codes are not dropped: `V-004` (a duplicate ordinal) and
    // `V-014` (a cycle) are answerable only against the assembled record set,
    // and a duplicate ordinal is one of the two mistakes a new draft actually
    // makes. Only this spec's are kept, and each message already names the
    // other spec involved, because an author told "duplicate numeric prefix
    // '056'" without being told who holds it has been given a puzzle.
    //
    // `V-008` and `V-010`, the other two cross-spec codes, are NOT recomputed
    // here: `validate_spec` above already judged them against the same
    // `all_ids`, and adding them a second time would report one mistake twice.
    let id = records
        .iter()
        .find(|r| r.spec_path == spec_path)
        .map(|r| r.id.clone())
        .unwrap_or_else(|| dirname.clone());
    let mut cross: Vec<Violation> = Vec::new();
    let id_paths: Vec<(String, String)> = records
        .iter()
        .map(|r| (r.id.clone(), r.spec_path.clone()))
        .collect();
    detect_duplicates(&id_paths, &mut cross);
    detect_dependency_cycle(&records, &mut cross);
    detect_amends_verification(&records, &mut cross);
    for v in cross {
        if v.path.as_deref() == Some(spec_path.as_str()) || v.message.contains(&id) {
            violations.push(v);
        }
    }

    Ok(finish_spec_report(dirname, spec_path, violations))
}

fn finish_spec_report(
    spec_id: String,
    spec_path: String,
    mut violations: Vec<Violation>,
) -> SpecCheckReport {
    violations.sort_by(|a, b| a.code.cmp(&b.code));
    let passed = !violations.iter().any(|v| v.severity == Severity::Error);
    SpecCheckReport {
        spec_id,
        spec_path,
        violations,
        passed,
    }
}

// ===== committed-shard IO + assembly (spec 022) =====

/// The committed spec-registry directory: `<derived>/spec-registry`.
pub fn registry_dir(cfg: &Config, repo_root: &Path) -> PathBuf {
    repo_root
        .join(&cfg.layout.derived_dir)
        .join("spec-registry")
}

/// Serialize each registry shard to its canonical-JSON file, `(<id>.json,
/// content)`. Canonical (sorted keys, 2-space, trailing LF) so a shard is
/// byte-identical across platforms. The CLI writes these under
/// `<registry_dir>/by-spec/`.
pub fn registry_shard_files(shards: &RegistryShardSet) -> Result<Vec<(String, String)>, Error> {
    shards
        .spec_shards
        .iter()
        .map(|s| {
            Ok((
                format!("{}.json", s.record.id),
                canonical_json::to_string(s)?,
            ))
        })
        .collect()
}

/// The cap on how many stale shards a freshness report names (spec 028 §3.3),
/// so a corpus-wide restamp reports `and N more` instead of flooding a CI log.
const STALE_REPORT_CAP: usize = 20;

/// Registry-shard freshness (spec 028): does the committed shard tree equal what
/// the current corpus compiles to? Compiles in memory and delegates to
/// [`compare_committed_registry`]; **never writes**.
///
/// Reports staleness only. Validation is a separate verdict carried on
/// [`CompileOutcome`]; a caller that needs both (the CLI, which must rank
/// validation above staleness) should call [`compile`] and
/// [`compare_committed_registry`] directly rather than compiling twice.
pub fn check_registry_freshness(cfg: &Config, repo_root: &Path) -> Result<Freshness, Error> {
    let outcome = compile(cfg, repo_root)?;
    compare_committed_registry(cfg, repo_root, &outcome.shards)
}

/// Compare a freshly compiled shard set against the committed one, byte for
/// byte (spec 028 §3.1).
///
/// The comparison is over the serialized shard *bytes*, not the `shardHash`
/// field: canonical emission makes a fresh compile reproducible, so an exact
/// comparison is both the simplest and the strictest check, catching a stale
/// hash, a hand-edited body, and a schema restamp alike. (The index side
/// compares recomputed hashes instead only because re-resolving code spans is
/// expensive; a registry shard is cheap to re-emit.)
///
/// Three drift classes are reported together: `modified` (bytes differ),
/// `missing` (a spec with no committed shard), and `orphaned` (a committed
/// shard with no spec). The latter two are set-membership failures, invisible
/// to a content comparison alone, which is why this compares sets.
///
/// An unbuilt registry reads as `Stale`, not [`Error::Io`]: a registry that was
/// never built is by definition not vouching for the corpus (same reasoning as
/// spec 011 §3.3 for an index predating its slice config).
pub fn compare_committed_registry(
    cfg: &Config,
    repo_root: &Path,
    shards: &RegistryShardSet,
) -> Result<Freshness, Error> {
    let expected: BTreeMap<String, String> = registry_shard_files(shards)?.into_iter().collect();
    let by_spec = registry_dir(cfg, repo_root).join(shard::BY_SPEC_DIR);
    // `read_shard_files` yields an empty list for a missing dir, which is what
    // turns an unbuilt registry into "every shard missing" (stale) rather than
    // an I/O error.
    let committed: BTreeMap<String, Vec<u8>> =
        shard::read_shard_files(&by_spec)?.into_iter().collect();

    let mut stale: Vec<String> = Vec::new();
    for (name, content) in &expected {
        match committed.get(name) {
            None => stale.push(format!("missing {name}")),
            Some(bytes) if bytes.as_slice() != content.as_bytes() => {
                stale.push(format!("modified {name}"));
            }
            Some(_) => {}
        }
    }
    for name in committed.keys() {
        if !expected.contains_key(name) {
            stale.push(format!("orphaned {name}"));
        }
    }

    if stale.is_empty() {
        return Ok(Freshness::Fresh);
    }
    stale.sort();
    let total = stale.len();
    // One line per stale shard (spec 028 §3.3), so a CI log stays greppable and
    // each finding is its own line rather than one wrapped blob.
    let mut lines: Vec<String> = stale
        .iter()
        .take(STALE_REPORT_CAP)
        .map(|s| format!("  {s}"))
        .collect();
    if total > STALE_REPORT_CAP {
        lines.push(format!("  and {} more", total - STALE_REPORT_CAP));
    }
    Ok(Freshness::Stale {
        expected: format!("{} shard(s) matching the corpus", expected.len()),
        actual: format!("{total} stale shard(s):\n{}", lines.join("\n")),
    })
}

/// Read and parse the committed registry shards (gating each shard's MAJOR at
/// the read boundary). An unbuilt registry (no `by-spec` dir) yields an empty
/// vector.
fn read_committed_registry_shards(
    cfg: &Config,
    repo_root: &Path,
) -> Result<Vec<RegistrySpecShard>, Error> {
    let base = registry_dir(cfg, repo_root);
    if !base.exists() {
        return Err(Error::Io(format!(
            "read {} (run `spec-spine compile` first?): not found",
            base.display()
        )));
    }
    let dir = base.join(shard::BY_SPEC_DIR);
    let mut shards = Vec::new();
    for (name, bytes) in shard::read_shard_files(&dir)? {
        let sh: RegistrySpecShard = serde_json::from_slice(&bytes)
            .map_err(|e| Error::Parse(format!("invalid registry shard {name}: {e}")))?;
        shard::check_major("registry", &sh.spec_version, REGISTRY_SCHEMA_VERSION)?;
        shards.push(sh);
    }
    Ok(shards)
}

/// Assemble the aggregate [`Registry`] from the committed shard set (spec 022).
/// The aggregate validation and content hash are recomputed on read: per-shard
/// "local" violations are merged, the corpus-wide checks (duplicate id/prefix,
/// dangling edges) are re-derived from the assembled records, and the content
/// hash is the fold of the shard hashes. This is the single committed-registry
/// reader; the `registry` queries and the coupling gate route through it, so
/// they keep their `&Registry` contract unchanged.
pub fn load_committed_registry(cfg: &Config, repo_root: &Path) -> Result<Registry, Error> {
    let shards = read_committed_registry_shards(cfg, repo_root)?;
    let mut records: Vec<SpecRecord> = Vec::new();
    let mut violations: Vec<Violation> = Vec::new();
    let mut keyed: Vec<(String, String)> = Vec::new();
    for sh in &shards {
        keyed.push((format!("spec:{}", sh.record.id), sh.shard_hash.clone()));
        violations.extend(sh.local_violations.iter().cloned());
        records.push(sh.record.clone());
    }
    records.sort_by(|a, b| a.id.cmp(&b.id));
    violations.extend(recompute_cross_spec_violations(&records));
    Ok(Registry {
        spec_version: REGISTRY_SCHEMA_VERSION.to_string(),
        build: Build {
            compiler_id: cfg.branding.compiler_id.clone(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            input_root: ".".to_string(),
            content_hash: shard::aggregate_content_hash(&keyed),
        },
        specs: records,
        validation: ValidationReport::from_violations(violations),
    })
}

/// Re-derive the corpus-wide validation findings (V-003/004/008/010/014, and,
/// by linkage through [`detect_amends_verification`] and
/// [`detect_impact_cross_spec`], V-018/019/020 and V-028/029/030/031) from
/// the assembled records. A local mirror of the cross-spec checks in
/// [`compile`] (kept equal by value for V-003/004/008/010, and by linkage for
/// V-014, which both paths reach through [`detect_dependency_cycle`]), so the
/// assembled registry's validation matches a fresh compile's for a corpus that
/// has any. Records are id-sorted,
/// which equals compile's discovery order, so V-004's "shared by" pairing agrees.
fn recompute_cross_spec_violations(records: &[SpecRecord]) -> Vec<Violation> {
    let mut out: Vec<Violation> = Vec::new();
    let id_paths: Vec<(String, String)> = records
        .iter()
        .map(|r| (r.id.clone(), r.spec_path.clone()))
        .collect();
    detect_duplicates(&id_paths, &mut out);
    let all_ids: std::collections::BTreeSet<&str> = records.iter().map(|r| r.id.as_str()).collect();
    for r in records {
        if r.status == Status::Superseded {
            match &r.superseded_by {
                Some(by) if all_ids.contains(by.as_str()) => {}
                Some(by) => out.push(error(
                    "V-008",
                    format!("superseded_by '{by}' does not resolve to an existing spec"),
                    Some(r.spec_path.clone()),
                )),
                None => out.push(error(
                    "V-008",
                    "status is 'superseded' but superseded_by is missing".into(),
                    Some(r.spec_path.clone()),
                )),
            }
        }
        for dep in &r.depends_on {
            if !all_ids.contains(dep.as_str()) {
                out.push(warning(
                    "V-010",
                    format!("depends_on '{dep}' does not resolve to an existing spec"),
                    Some(r.spec_path.clone()),
                ));
            }
        }
    }
    detect_dependency_cycle(records, &mut out);
    detect_amends_verification(records, &mut out);
    detect_impact_cross_spec(records, &mut out);
    detect_move_cross_spec(records, &mut out);
    out
}

/// Resolve a short spec reference (`109`) to the full id (`109-slug`) by its
/// leading numeric segment, when exactly one spec matches. An exact id, an
/// ambiguous prefix, or no match returns the input unchanged (so V-008/V-010
/// still fire on a genuinely dangling reference). Mirrors the indexer's
/// `resolve_id` (spec 004) so compile-time and index-time resolution agree;
/// kept local to avoid coupling the compile gate (001) to the indexer's file.
fn resolve_spec_ref(short: &str, all_ids: &std::collections::BTreeSet<String>) -> String {
    crate::spec_id::resolve_spec_ref(short, all_ids)
}

// --- small helpers ---

fn error(code: &str, message: String, path: Option<String>) -> Violation {
    Violation::new(code, Severity::Error, message).at_opt(path)
}

fn warning(code: &str, message: String, path: Option<String>) -> Violation {
    Violation::new(code, Severity::Warning, message).at_opt(path)
}

/// Repo-relative POSIX path of `file` under `repo_root` (forward slashes).
pub(crate) fn rel_posix(repo_root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(repo_root).unwrap_or(file);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// `^[0-9]{3}-[a-z0-9]+(-[a-z0-9]+)*$`, matched without a regex dependency.
fn valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    if bytes.len() < 5 || !bytes[..3].iter().all(u8::is_ascii_digit) || bytes[3] != b'-' {
        return false;
    }
    let slug = &id[4..];
    if slug.is_empty() || slug.starts_with('-') || slug.ends_with('-') || slug.contains("--") {
        return false;
    }
    slug.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::valid_id;

    #[test]
    fn id_pattern() {
        assert!(valid_id("000-spec-spine-bootstrap"));
        assert!(valid_id("042-x"));
        assert!(!valid_id("42-x"), "needs 3 digits");
        assert!(!valid_id("000-"), "needs a slug");
        assert!(!valid_id("000-Foo"), "lowercase only");
        assert!(!valid_id("000--x"), "no double hyphen");
        assert!(!valid_id("000-x-"), "no trailing hyphen");
    }
}
