//! The conformance lint (spec 003): corpus convention checks (`L-` codes),
//! disjoint from compile's structural `V-` codes. Severity gating
//! (error always / warning under `--fail-on-warn` / info under `--fail-on-info`)
//! is applied by the CLI; this layer just produces the diagnostics.

use std::collections::BTreeSet;
use std::path::Path;

use spec_spine_types::{Config, Error, Severity, SpecRecord, Status, Unit, Violation};

use crate::compile::compile;

/// The result of a lint run.
pub struct LintReport {
    pub violations: Vec<Violation>,
}

impl LintReport {
    /// Count of violations at a given severity.
    pub fn count(&self, severity: Severity) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .count()
    }
}

/// Lint the corpus under `repo_root`: compile it, then run conformance checks.
pub fn lint(cfg: &Config, repo_root: &Path) -> Result<LintReport, Error> {
    let registry = compile(cfg, repo_root)?.registry;
    let ids: BTreeSet<&str> = registry.specs.iter().map(|s| s.id.as_str()).collect();
    let domains_enabled = !cfg.domains.allowed.is_empty();
    let kind_enabled = !cfg.kind.allowed.is_empty();

    let mut violations = Vec::new();
    for spec in &registry.specs {
        let at = || Some(spec.spec_path.clone());

        // L-001: ordinary spec claims no territory.
        //
        // Spec 092 §3.11: a `superseded` or `retired` spec is not an ordinary
        // spec. L-001 exists to catch an author who wrote a spec and forgot to
        // say what it governs; a document whose authority has been transferred
        // by an explicit `supersedes` edge, or withdrawn, is making a correct
        // statement when it claims nothing, and holding it to L-001 would leave
        // a corpus with exactly two ways to answer: delete the history, or
        // invent a claim for it. Neither is the lint's business.
        //
        // This exempts the DECLARATION, never a claim: a superseded spec that
        // still names a unit is still held to every diagnostic about that unit,
        // because the unit is still written down.
        //
        // Spec 116: a spec marked `implementation: deferred` is the fourth case
        // of the same exemption. The contract's lifecycle table keeps
        // `deferred` as a decision not to schedule, and the only honest
        // territory for a contract nobody is building is none: a speculative
        // `establishes` raises an unresolved unit that `check
        // --fail-on-unresolved` refuses, and `planned: true` on a file that
        // already exists raises `L-012`.
        //
        // A ratchet, not a hole: nothing remembers the deferral, so any other
        // `implementation` value, or none, re-arms the warning. `n-a` is
        // deliberately not included (spec 116 §3.3). Like the exemptions
        // beside it, this covers the declaration and never a claim.
        let withdrawn = matches!(spec.status, Status::Superseded | Status::Retired);
        let retroactive = spec.origin.as_ref().is_some_and(|o| o.retroactive);
        let deferred = spec.implementation == Some(spec_spine_types::Implementation::Deferred);
        if !retroactive && !withdrawn && !deferred && !has_ownership_edge(spec) {
            violations.push(warn(
                "L-001",
                format!(
                    "spec '{}' declares no ownership edge (claims no territory)",
                    spec.id
                ),
                at(),
            ));
        }
        // L-002 / L-003: unclassified under an enabled taxonomy.
        if domains_enabled && spec.domain.is_none() {
            violations.push(warn(
                "L-002",
                format!("spec '{}' has no domain", spec.id),
                at(),
            ));
        }
        if kind_enabled && spec.kind.is_none() {
            violations.push(warn(
                "L-003",
                format!("spec '{}' has no kind", spec.id),
                at(),
            ));
        }
        // L-013: the title heading names another spec (spec 098 §3.4).
        //
        // The only citation class a lint can decide. A document states its own
        // ordinal twice, in its directory and in its title, and two statements
        // of one fact can be compared; nothing states the answer for `spec 050`
        // in a comment, which is why the other two classes spec 098 repairs get
        // a form in `compact` instead of a code here.
        //
        // Silent when either side carries no ordinal. The corpus does not
        // require numeric ids (`V-001` requires only that the directory equal
        // the id), and holding a corpus to a convention it never claimed is
        // spec 046 §3.3's mistake.
        if let Some(own) = spec_ordinal_str(&spec.id)
            && let Ok(text) = std::fs::read_to_string(repo_root.join(&spec.spec_path))
            && let Some(heading) = title_heading_ordinal(&text)
            && heading != own
        {
            violations.push(warn(
                "L-013",
                format!(
                    "spec '{}' is titled '{heading}': its title heading names another \
                     spec's ordinal. The directory and the heading state the same fact, \
                     and they disagree",
                    spec.id
                ),
                at(),
            ));
        }
        // L-004: dangling edge target.
        for target in edge_targets(spec) {
            if !ids.contains(target.as_str()) {
                violations.push(warn(
                    "L-004",
                    format!("spec '{}' references unknown spec '{target}'", spec.id),
                    at(),
                ));
            }
        }
        // L-006: a unit claimed inside the declared, ungoverned state root
        // (spec 036 3.4). Error tier: neither the claim nor the bypass wins,
        // because letting the claim win would reintroduce spec 008's override
        // into a directory whose whole purpose is to be ungoverned, and letting
        // the bypass win would silently discard a unit an author wrote
        // deliberately. Both are wrong, so the corpus is told instead.
        for unit_path in claimed_paths(spec) {
            if cfg.layout.is_state_path(&unit_path) {
                violations.push(error(
                    "L-006",
                    format!(
                        "spec '{}' claims '{unit_path}', which is inside the ungoverned \
                         layout.state_dir '{}': move the file out of the state root, or \
                         stop claiming it",
                        spec.id, cfg.layout.state_dir
                    ),
                    at(),
                ));
            }
        }

        // L-007: a `depends_on` entry that does not point backward in filing
        // order (spec 046 §3.2). Opt-in: when the knob is off nothing is
        // emitted at all, not emitted-and-filtered, so a corpus that has not
        // opted in sees byte-identical output before and after spec 046.
        //
        // Error tier, matching `L-006`: the knob alone decides whether the
        // corpus is held to this, and an adopter who turned it on turned it on
        // to be refused. A warning would have meant two knobs (this one and
        // `--fail-on-warn`) for one decision.
        if cfg.lint.require_ordinal_monotonic_depends_on {
            for target in &spec.depends_on {
                if let Some((mine, theirs)) = ordinal_pair(&spec.id, target)
                    && theirs >= mine
                {
                    violations.push(error(
                        "L-007",
                        format!(
                            "spec '{}' depends_on '{target}', which is not a lower \
                                 ordinal ({theirs:03} >= {mine:03}): a dependency points \
                                 backward in filing order",
                            spec.id
                        ),
                        at(),
                    ));
                }
            }
        }

        // L-009 (spec 051 §3.2): a heading that is a near miss for the defects
        // anchor. Info tier, the tier this corpus reserves for a nudge on
        // otherwise valid prose (`L-005` is the other): it surfaces under
        // `--fail-on-info`, which no gate here runs, so a corpus is told
        // without being refused.
        //
        // The anchor comes from `sections::anchor_of`, the indexer's own slug
        // rule, so the lint cannot disagree with what a section unit would
        // resolve to. Quoting a heading and meaning an anchor is the defect
        // this spec closes; carrying a second slug rule would reopen it.
        for heading in &spec.section_headings {
            let anchor = crate::sections::anchor_of(heading);
            if crate::sections::is_near_miss_defects_anchor(&anchor) {
                violations.push(info(
                    "L-009",
                    format!(
                        "spec '{}' has heading '{heading}' (anchor '{anchor}'), which is \
                         not the defects anchor: a consumer looking for '{}' will not \
                         find this section",
                        spec.id,
                        crate::sections::DEFECTS_ANCHOR
                    ),
                    at(),
                ));
            }
        }

        // L-005: stub (no body sections).
        if spec.section_headings.is_empty() {
            violations.push(info(
                "L-005",
                format!("spec '{}' has no body sections", spec.id),
                at(),
            ));
        }

        // L-014 (spec 109 §3.3): an `unresolved` conflict, once per entry. A
        // corpus is allowed to know it contradicts itself; what it is not
        // allowed to do is know silently, so this is a warning, not an error,
        // and `--fail-on-warn` is what turns it into a refusal.
        for c in &spec.conflicts {
            if c.resolution == spec_spine_types::ConflictResolution::Unresolved {
                violations.push(warn(
                    "L-014",
                    format!(
                        "spec '{}' declares an unresolved conflict with '{}'",
                        spec.id, c.obligation
                    ),
                    at(),
                ));
            }
        }

        // L-015 / L-016 (spec 111 §3.3): what a move declaration says about
        // the working tree, checked against it directly. Warning tier: these
        // describe the tree at a point in time, not the declaration's own
        // well-formedness (that is `V-040`, at compile), and they MUST NOT
        // attempt to verify that content moved (§3.6). A path `V-040`
        // refuses (empty, absolute, a `..` segment), or one carrying a
        // platform root or prefix, is never joined onto the repository root:
        // `Path::join` with an absolute argument discards the root, so the
        // stat would read outside the tree and answer about another file
        // (111 D-12). A path `check_move_path` refuses is also a `V-040` at
        // compile; one only the component scan rejects (a Windows drive
        // prefix, say) is skipped here and is not a compile error.
        let in_tree = |p: &str| {
            crate::compile::check_move_path(p).is_none()
                && std::path::Path::new(p).components().all(|c| {
                    matches!(
                        c,
                        std::path::Component::Normal(_) | std::path::Component::CurDir
                    )
                })
        };
        for mv in &spec.moves {
            let kind_label = mv.kind.label();
            match (&mv.to, mv.kind) {
                (Some(to), _) => {
                    // L-015: a relocated/split/merged `to` path that does not
                    // exist, naming the declaring spec and both paths. One
                    // warning per missing `to` (a `split` entry checks each
                    // branch independently); the `from` side is named as
                    // declared, joined, rather than repeated per `to`.
                    let from_label = mv.from.paths().join(", ");
                    for to_path in to.paths() {
                        if in_tree(to_path) && !repo_root.join(to_path).exists() {
                            violations.push(warn(
                                "L-015",
                                format!(
                                    "spec '{}' declares a '{kind_label}' move from '{from_label}' \
                                     to '{to_path}', and '{to_path}' does not exist",
                                    spec.id
                                ),
                                at(),
                            ));
                        }
                    }
                }
                (None, spec_spine_types::MoveKind::Removed) => {
                    // L-016: the other direction. `from` has exactly one path
                    // under `removed` (V-040 enforces the arity).
                    for from_path in mv.from.paths() {
                        if in_tree(from_path) && repo_root.join(from_path).exists() {
                            violations.push(warn(
                                "L-016",
                                format!(
                                    "spec '{}' declares '{from_path}' removed, and it still exists",
                                    spec.id
                                ),
                                at(),
                            ));
                        }
                    }
                }
                (None, _) => {}
            }
        }
    }

    // L-017 (spec 142 §3.4): an `amends_sections` entry that names no section of
    // any spec the declaring spec amends. The list is one flat list shared by
    // every target (spec 132's note), so an entry is judged against all of
    // them: it names a section when it is a heading's number (`3.1` for
    // `### 3.1 The rule`) or its anchor, in at least one amended spec. Before
    // 142 the key was accepted with any text, so a typo pointed a reader at a
    // section that does not exist.
    let by_id: std::collections::BTreeMap<&str, &SpecRecord> =
        registry.specs.iter().map(|s| (s.id.as_str(), s)).collect();
    for spec in &registry.specs {
        for entry in &spec.amends_sections {
            let named = spec
                .amends
                .iter()
                .filter_map(|a| by_id.get(a.as_str()))
                .any(|t| {
                    t.section_headings.iter().any(|h| {
                        heading_number(h) == Some(entry.as_str())
                            || crate::sections::anchor_of(h) == *entry
                    })
                });
            if !named {
                violations.push(warn(
                    "L-017",
                    format!(
                        "spec '{}' amends_sections names '{entry}', which is the number or anchor \
                         of no section in the spec(s) it amends ({})",
                        spec.id,
                        spec.amends.join(", ")
                    ),
                    Some(spec.spec_path.clone()),
                ));
            }
        }
    }

    // L-018 (spec 142 §3.5): two live specs that both `establishes` one unit.
    // Before 142 a split could leave the unit claimed by the old spec and the
    // new one alike, and both cleared the gate. The hand-off that makes one of
    // them the owner is a partial `supersedes` (§3.1), which removes the
    // predecessor's claim, so a pair joined by one is not reported.
    let mut establishers: std::collections::BTreeMap<String, Vec<&SpecRecord>> =
        std::collections::BTreeMap::new();
    for spec in registry
        .specs
        .iter()
        .filter(|s| !matches!(s.status, Status::Superseded | Status::Retired))
    {
        for unit in &spec.establishes {
            if let Ok(key) = serde_json::to_string(&unit.subject()) {
                establishers.entry(key).or_default().push(spec);
            }
        }
    }
    for (key, specs) in &establishers {
        if specs.len() < 2 {
            continue;
        }
        let handed_over = |a: &SpecRecord, b: &SpecRecord| {
            a.supersedes.iter().any(|s| {
                s.spec() == b.id
                    && s.partial_unit()
                        .and_then(|u| serde_json::to_string(&u.subject()).ok())
                        .is_some_and(|k| k == *key)
            })
        };
        let unresolved: Vec<&str> = specs
            .iter()
            .filter(|a| !specs.iter().any(|b| handed_over(b, a)))
            .map(|s| s.id.as_str())
            .collect();
        if unresolved.len() >= 2 {
            for id in &unresolved {
                violations.push(warn(
                    "L-018",
                    format!(
                        "unit {key} is established by more than one live spec ({}); a split \
                         hands a unit over with a partial `supersedes` naming it, which leaves \
                         one owner",
                        unresolved.join(", ")
                    ),
                    by_id.get(id).map(|s| s.spec_path.clone()),
                ));
            }
        }
    }

    // L-008 (spec 050): a claimed path that exists and that no content hash
    // covers. Its contents can be rewritten end to end with `index check` and
    // `compile --check` both reporting fresh, which is the sentence this
    // diagnostic qualifies.
    //
    // Read from the **committed** index rather than recomputed: `lint` is a
    // read verb and indexing here would make it write-shaped and slow. A corpus
    // with no committed index is silent, which is right: there is no ledger yet
    // for a claim to be invisible to.
    if let Ok(index) = crate::index::load_committed_index(cfg, repo_root) {
        for claim in crate::index::unwitnessed_claims(cfg, repo_root, &index)
            .into_iter()
            .filter(|c| !c.allowed)
        {
            let spec_path = registry
                .specs
                .iter()
                .find(|s| s.id == claim.spec_id)
                .map(|s| s.spec_path.clone());
            violations.push(warn(
                "L-008",
                // Spec 061 3.2, conformance with 057 3.x: the two remedies are
                // genuinely different choices and the message must say how, or
                // it reads as a pick-either. It sent an adopter to the wrong
                // one. Since spec 141 a glob records the file in the index's
                // inputs record, so an edit rewrites that one file and makes
                // the change a governance change: right for a handful of
                // governance files, wrong for a tree of source. A section or
                // symbol unit is hashed through its span and stales only the
                // claiming spec's shard.
                format!(
                    "spec '{}' claims '{}', which is in no content hash: its contents can \
                     change without staling any shard. Either add a covering glob to \
                     [index] extra_hashed_inputs, which records the file in the index's \
                     inputs record (codebase-index/inputs.json), so every edit to it is a \
                     governance change that rewrites that record (right for a few \
                     governance files, wrong for a tree of source), or claim a section or \
                     symbol unit, which is hashed through its span and stales only this \
                     spec's shard",
                    claim.spec_id, claim.path
                ),
                spec_path,
            ));
        }
    }

    // L-011 / L-012 (spec 063 3.3, 3.4): the `planned` flag cannot outlive the
    // work, in either direction.
    //
    // Read from the **committed** index for the resolved half, by the same
    // rule `L-008` follows: `lint` is a read verb, and indexing here would make
    // it write-shaped and slow. A corpus with no committed index is silent
    // about `L-012`, which is right: there is no ledger yet to say a claim has
    // landed. `L-011` needs no ledger at all, being a contradiction inside one
    // file's frontmatter.
    let resolved_paths: std::collections::BTreeSet<String> =
        match crate::index::load_committed_index(cfg, repo_root) {
            Ok(index) => index
                .traceability
                .mappings
                .iter()
                .flat_map(|t| t.resolved_units.iter())
                .filter(|ru| !ru.locations.is_empty())
                .map(|ru| unit_key(&ru.unit))
                .collect(),
            Err(_) => Default::default(),
        };
    for spec in &registry.specs {
        let at = || Some(spec.spec_path.clone());
        for unit in owned_units(spec).into_iter().filter(|u| u.is_planned()) {
            // L-011: completion asserts the work is done and a planned unit
            // asserts it is not. Error tier, not warning: the two together are
            // a contradiction inside the spec's own frontmatter rather than a
            // gap someone might hold deliberately. Spec 038 established that
            // `complete` ends the in-flight window; this extends the same
            // principle to the new field, which is what keeps `planned` from
            // becoming a state nobody owns the exit from.
            if spec.implementation == Some(spec_spine_types::Implementation::Complete) {
                violations.push(error(
                    "L-011",
                    format!(
                        "spec '{}' is implementation: complete while unit '{}' is still \
                         marked planned: completion asserts the work is done and a planned \
                         unit asserts it is not. Drop the flag, or the completion",
                        spec.id,
                        unit_label(&unit)
                    ),
                    at(),
                ));
            }
            // L-012: the other direction. The file landed, the claim is
            // satisfied, and without this nothing notices that the spec still
            // describes it as future work. Warning tier, so `--fail-on-warn`
            // refuses it in a corpus running the gate and a corpus that does
            // not is merely told.
            if resolved_paths.contains(&unit_key(&unit.subject())) {
                violations.push(warn(
                    "L-012",
                    format!(
                        "spec '{}' marks unit '{}' planned, and it now resolves: drop the \
                         flag. The claim is unaffected either way, this is a signal about \
                         the frontmatter",
                        spec.id,
                        unit_label(&unit)
                    ),
                    at(),
                ));
            }
        }
    }

    // L-010 (spec 061 3.1): an `[index] extra_hashed_inputs` pattern ending in
    // `/**`. In the `glob` crate `dir/**` enumerates DIRECTORIES and the hasher
    // keeps only entries that are files, so such a pattern can never contribute
    // a byte to any content hash, whatever the tree contains.
    //
    // Two adopters hit this on one day: one wrote `crates/**`, measured no
    // effect and concluded the key was inert; the other found `standards/**`
    // and `.github/workflows/**` inherited from a scaffold and confirmed they
    // "had never contributed to any content hash". Spec 058 fixed the default
    // and could not fix a value already written into an adopter's own file.
    //
    // The check is on the PATTERN, not on whether it currently matches. A
    // pattern matching nothing today may be a legitimate forward-looking entry
    // in a specify-first corpus, which is the false positive spec 058 4 named
    // when it deferred this lint. A pattern ending `/**` is inert under EVERY
    // tree, so refusing the form is decidable from the config alone and has no
    // legitimate counter-example: an adopter who wants to match nothing writes
    // no entry.
    //
    // Warning tier, so `lint --fail-on-warn` refuses it. Unlike `L-008`, which
    // flags a state a corpus may hold deliberately, this one never is.
    //
    // Second table (spec 065 3.1): `[index.slices]` carries pattern lists with
    // `extra_hashed_inputs` semantics and the slice walk keeps only files, so
    // `dir/**` is equally inert there. One code, not `L-011` (065 D-2): the
    // defect and the remedy are identical, only the sentence about what is
    // lost differs. An adopter audited on 2026-09-09 carried eight of these in
    // slices after a fix pass had cleaned the table this lint already read.
    //
    // Each message names its table, and for a slice the slice, on one line
    // (065 3.2). The slice message speaks about the slice's own hash, the one
    // `index check --slice <name>` gates, and never about a content hash:
    // slices are independent of `contentHash` by spec 011's design, so the
    // `extra_hashed_inputs` sentence would be a false statement under a true
    // code (065 3.3). One emission site for both tables, so the code stays
    // unique by construction.
    let global_dead = cfg
        .index
        .extra_hashed_inputs
        .iter()
        .filter(|p| p.ends_with("/**"))
        .map(|pattern| {
            format!(
                "[index] extra_hashed_inputs pattern '{pattern}' matches \
                 directories only, so it can contribute no bytes to any \
                 content hash. Write it as '{pattern}/*' to match the files \
                 under it"
            )
        });
    let slice_dead = cfg.index.slices.iter().flat_map(|(name, patterns)| {
        patterns
            .iter()
            .filter(|p| p.ends_with("/**"))
            .map(move |pattern| {
                format!(
                    "[index.slices] '{name}' pattern '{pattern}' matches \
                     directories only, so it can contribute no bytes to the \
                     '{name}' slice hash that `index check --slice {name}` \
                     gates. Write it as '{pattern}/*' to match the files under \
                     it"
                )
            })
    });
    for message in global_dead.chain(slice_dead) {
        violations.push(warn("L-010", message, Some("spec-spine.toml".to_string())));
    }

    Ok(LintReport { violations })
}

/// Both ids' ordinals, or `None` when either lacks one (spec 046 §3.3).
///
/// Silence is the honest answer when the order is undefined. The corpus does
/// not require numeric ids: `V-001` requires only that the directory equal the
/// id, so `auth-login` is well-formed, and telling such a corpus that its ids
/// have no ordinal would be telling it that it does not hold a convention it
/// never claimed.
fn ordinal_pair(declaring: &str, target: &str) -> Option<(u64, u64)> {
    Some((ordinal(declaring)?, ordinal(target)?))
}

/// The leading decimal digit run of an id, as an integer.
///
/// Numeric rather than lexical, so a corpus that outgrows three digits and
/// files `1001-foo` orders above `999-bar` instead of below it. Iterating
/// `char`s rather than slicing bytes keeps this safe on a non-ASCII id, which
/// is the defect `detect_duplicates` carries and which spec 046 §4 declines to
/// fix from inside a lint change.
fn ordinal(id: &str) -> Option<u64> {
    let digits: String = id.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// The leading three-digit ordinal of an id, as written. `None` when the id
/// does not open with exactly three digits (spec 098 §3.4).
fn spec_ordinal_str(id: &str) -> Option<&str> {
    let head = id.get(..3)?;
    if head.as_bytes().iter().all(u8::is_ascii_digit)
        && !id[3..].starts_with(|c: char| c.is_ascii_digit())
    {
        Some(head)
    } else {
        None
    }
}

/// The ordinal of the first `#{1,6} NNN: ` heading AFTER the frontmatter.
///
/// After, because a `#` comment inside YAML frontmatter is prose and several in
/// this corpus open with an ordinal: reading one as a title would report a
/// citation as a wrong title.
fn title_heading_ordinal(src: &str) -> Option<&str> {
    let mut lines = src.lines();
    let mut body: Box<dyn Iterator<Item = &str>> = if src.starts_with("---") {
        let _ = lines.next();
        let mut seen_close = false;
        Box::new(lines.by_ref().skip_while(move |l| {
            if seen_close {
                false
            } else {
                seen_close = l.trim_end() == "---";
                true
            }
        }))
    } else {
        Box::new(lines)
    };
    body.find_map(|line| {
        let hashes = line.len() - line.trim_start_matches('#').len();
        if !(1..=6).contains(&hashes) || !line[hashes..].starts_with(' ') {
            return None;
        }
        let ord = line.get(hashes + 1..hashes + 4)?;
        if ord.as_bytes().iter().all(u8::is_ascii_digit) && line[hashes + 4..].starts_with(':') {
            Some(ord)
        } else {
            None
        }
    })
}

/// The leading section number of a heading's text (`3.1` for `3.1 The rule`,
/// `4` for `4. Out of scope`), without a trailing dot, if it has one.
fn heading_number(heading: &str) -> Option<&str> {
    let end = heading
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(heading.len());
    let number = heading[..end].trim_end_matches('.');
    (!number.is_empty() && number.starts_with(|c: char| c.is_ascii_digit())).then_some(number)
}

fn has_ownership_edge(spec: &SpecRecord) -> bool {
    !spec.establishes.is_empty()
        || !spec.extends.is_empty()
        || !spec.refines.is_empty()
        || !spec.supersedes.is_empty()
        || !spec.amends.is_empty()
        || !spec.co_authority.is_empty()
        || !spec.constrains.is_empty()
}

/// Every spec id this spec names across its relationship edges.
fn edge_targets(spec: &SpecRecord) -> Vec<String> {
    let mut targets = Vec::new();
    targets.extend(spec.supersedes.iter().map(|s| s.spec().to_string()));
    targets.extend(spec.amends.iter().cloned());
    targets.extend(spec.extends.iter().map(|e| e.spec.clone()));
    targets.extend(spec.refines.iter().flat_map(|r| r.refines_specs.clone()));
    targets.extend(spec.co_authority.iter().flat_map(|c| c.with_specs.clone()));
    targets.extend(spec.constrains.iter().flat_map(|c| c.target_specs.clone()));
    targets
}

/// Every repo-relative path a spec claims through an ownership-bearing edge.
///
/// All six of them: `establishes`, `extends`, `refines`, `supersedes`,
/// `co_authority` and `constrains`. A partial `supersedes` item carries the unit
/// whose authority transfers (spec 018), so it claims a path exactly as the
/// others do; omitting it would let a superseding spec hold a claim inside the
/// state root that no diagnostic ever named, which is the contradiction `L-006`
/// exists to surface.
///
/// `references` is excluded: spec 031 settled that a cited file is not a claimed
/// one, so citing something inside the state root is not that contradiction.
/// `amends` is excluded too, because its subject is the amended spec's `spec.md`
/// rather than an arbitrary unit, and a `spec.md` lives under `specs_dir`, which
/// `state_dir` may not overlap.
///
/// Section and file units carry a path; symbol, crate and module units are
/// resolved by id and have none to test here.
fn claimed_paths(spec: &SpecRecord) -> Vec<String> {
    let mut paths = Vec::new();
    let mut push = |unit: &Unit| {
        if let Some(p) = unit_path(unit) {
            paths.push(p);
        }
    };
    for unit in &spec.establishes {
        push(unit);
    }
    for item in &spec.extends {
        if let Some(u) = &item.unit {
            push(u);
        }
    }
    for item in &spec.refines {
        if let Some(u) = &item.unit {
            push(u);
        }
    }
    for item in &spec.supersedes {
        if let spec_spine_types::SupersedeItem::Scoped(scoped) = item
            && let Some(u) = &scoped.unit
        {
            push(u);
        }
    }
    for item in &spec.co_authority {
        push(&item.unit);
    }
    for item in &spec.constrains {
        if let Some(u) = &item.unit {
            push(u);
        }
    }
    paths
}

/// The repo-relative path a unit names, for the unit kinds that carry one.
fn unit_path(unit: &Unit) -> Option<String> {
    match unit {
        Unit::File { path, .. } | Unit::Directory { path, .. } => Some(path.clone()),
        Unit::Section { file, .. } => Some(file.clone()),
        Unit::Symbol { .. } | Unit::Crate { .. } | Unit::Module { .. } => None,
    }
}

fn error(code: &str, message: String, path: Option<String>) -> Violation {
    Violation::new(code, Severity::Error, message).at_opt(path)
}

fn warn(code: &str, message: String, path: Option<String>) -> Violation {
    Violation::new(code, Severity::Warning, message).at_opt(path)
}

fn info(code: &str, message: String, path: Option<String>) -> Violation {
    Violation::new(code, Severity::Info, message).at_opt(path)
}

/// A unit's identity as a comparable key, ignoring the `planned` flag (spec
/// 063 §3.4: the flag is not part of a unit's identity).
fn unit_key(unit: &Unit) -> String {
    match unit.subject() {
        Unit::File { path, .. } => format!("file:{path}"),
        Unit::Section { file, anchor, .. } => format!("section:{file}#{anchor}"),
        Unit::Symbol { id, .. } => format!("symbol:{id}"),
        Unit::Directory { path, .. } => format!("directory:{path}"),
        Unit::Crate { id, .. } => format!("crate:{id}"),
        Unit::Module { id, .. } => format!("module:{id}"),
    }
}

/// A unit as a reader sees it in a diagnostic: the path or id it names.
fn unit_label(unit: &Unit) -> String {
    match unit {
        Unit::File { path, .. } | Unit::Directory { path, .. } => path.clone(),
        Unit::Section { file, anchor, .. } => format!("{file}#{anchor}"),
        Unit::Symbol { id, .. } | Unit::Crate { id, .. } | Unit::Module { id, .. } => id.clone(),
    }
}

/// Every unit a spec claims through an ownership-bearing edge.
///
/// The same six edges `claimed_paths` walks, returning the units rather than
/// their paths, because spec 063's checks are about the unit (a symbol or crate
/// unit has no path and can be planned exactly as a file can). `references` is
/// excluded for the reason spec 031 gave: a cited file is not a claimed one, so
/// a spec cannot plan territory by citing it.
fn owned_units(spec: &SpecRecord) -> Vec<Unit> {
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
