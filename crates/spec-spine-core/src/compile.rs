//! The compile capability (spec 001): markdown corpus → deterministic registry.
//!
//! Pure function of `(config, file contents)`. Per-spec parse failures are
//! recorded as error-tier violations rather than aborting the run; `Err` is
//! reserved for I/O failures. The wall clock is never read here; it lives in
//! `build-meta.json`, written by the CLI.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_types::{
    Build, Config, Error, Frontmatter, FrontmatterIssue, REGISTRY_SCHEMA_VERSION, Registry,
    RegistrySpecShard, Severity, SpecRecord, Status, ValidationReport, Violation,
    parse_frontmatter_with, split_frontmatter,
};

use crate::index::Freshness;
use crate::{canonical_json, hash, markdown, shard};

/// Validation codes whose verdict depends on the whole corpus, not one spec:
/// duplicate id/prefix, dangling/unresolved edge targets, and a `depends_on`
/// cycle. They are NOT stored per registry shard (storing them would let a
/// sibling spec's PR stale this shard); they are recomputed from the assembled
/// record set on read.
const CROSS_SPEC_CODES: &[&str] = &["V-003", "V-004", "V-008", "V-010", "V-014"];

/// The cap on **undeclared** `extra_frontmatter` keys before `V-007` fires.
/// Keys listed in `frontmatter.extra_known_keys` are intentional and exempt;
/// the cap targets escape-hatch abuse (ported from OAP's ~8-entry V-002 cap).
pub const MAX_UNDECLARED_EXTRA_FRONTMATTER: usize = 8;

/// The result of a compile: the typed registry, its canonical JSON bytes, the
/// validation flag the CLI maps to an exit code, and the per-spec shard
/// projection the CLI writes to disk (spec 024).
///
/// `registry` + `json` are the aggregate in-memory view (consumed by `attest`,
/// the JSON facade, and the conformance test); `shards` is the committed form.
pub struct CompileOutcome {
    pub registry: Registry,
    pub json: String,
    pub validation_passed: bool,
    pub shards: RegistryShardSet,
}

/// The committed-form projection of a registry: one shard per spec (spec 024),
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
            // V-013 (spec 013 §3.3): a DECLARED extra key carrying a value
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

    // --- short-id resolution (spec 016): rewrite a depends_on / superseded_by
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
        // (spec 019), so the gate's predecessor→superseder transfer keys match.
        for item in &mut p.fm.supersedes {
            let resolved = resolve_spec_ref(item.spec(), &all_ids);
            item.set_spec(resolved);
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
        records.push(build_record(p.fm, p.spec_path, &p.body));
    }
    records.sort_by(|a, b| a.id.cmp(&b.id));
    // V-014 reads the records, not the parsed frontmatter, so it sees the
    // short-id-resolved `depends_on` (spec 016) rather than the authored text.
    detect_dependency_cycle(&records, &mut violations);

    // --- shard projection + aggregate content hash (spec 024) ---
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
    // V-011: a constrains item must scope something (spec 018): a code `unit`
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

/// V-003 (duplicate id) and V-004 (duplicate numeric prefix). `specs` is
/// `(id, spec_path)` pairs in discovery order.
fn detect_duplicates(specs: &[(String, String)], out: &mut Vec<Violation>) {
    let mut id_counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut prefix_owner: BTreeMap<&str, &str> = BTreeMap::new();
    for (id, spec_path) in specs {
        *id_counts.entry(id.as_str()).or_insert(0) += 1;
        let prefix = &id[..id.len().min(3)];
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

/// V-014 (error): a cycle in the `depends_on` graph (spec 033).
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
        unamendable: fm.unamendable,
        amendment_record: fm.amendment_record,
        origin: fm.origin,
        extra_frontmatter: fm.extra_frontmatter,
    }
}

// ===== committed-shard IO + assembly (spec 024) =====

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

/// The cap on how many stale shards a freshness report names (spec 031 §3.3),
/// so a corpus-wide restamp reports `and N more` instead of flooding a CI log.
const STALE_REPORT_CAP: usize = 20;

/// Registry-shard freshness (spec 031): does the committed shard tree equal what
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
/// byte (spec 031 §3.1).
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
/// spec 012 §3.3 for an index predating its slice config).
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
    // One line per stale shard (spec 031 §3.3), so a CI log stays greppable and
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

/// Assemble the aggregate [`Registry`] from the committed shard set (spec 024).
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

/// Re-derive the corpus-wide validation findings (V-003/004/008/010/014) from
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
    out
}

/// Resolve a short spec reference (`109`) to the full id (`109-slug`) by its
/// leading numeric segment, when exactly one spec matches. An exact id, an
/// ambiguous prefix, or no match returns the input unchanged (so V-008/V-010
/// still fire on a genuinely dangling reference). Mirrors the indexer's
/// `resolve_id` (spec 004) so compile-time and index-time resolution agree;
/// kept local to avoid coupling the compile gate (001) to the indexer's file.
fn resolve_spec_ref(short: &str, all_ids: &std::collections::BTreeSet<String>) -> String {
    if all_ids.contains(short) {
        return short.to_string();
    }
    let matches: Vec<&String> = all_ids
        .iter()
        .filter(|id| id.split('-').next() == Some(short))
        .collect();
    match matches.as_slice() {
        [only] => (*only).clone(),
        _ => short.to_string(),
    }
}

// --- small helpers ---

fn error(code: &str, message: String, path: Option<String>) -> Violation {
    Violation {
        code: code.to_string(),
        severity: Severity::Error,
        message,
        path,
    }
}

fn warning(code: &str, message: String, path: Option<String>) -> Violation {
    Violation {
        code: code.to_string(),
        severity: Severity::Warning,
        message,
        path,
    }
}

/// Repo-relative POSIX path of `file` under `repo_root` (forward slashes).
fn rel_posix(repo_root: &Path, file: &Path) -> String {
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
