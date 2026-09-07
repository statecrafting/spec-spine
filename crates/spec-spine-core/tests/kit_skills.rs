// Spec: specs/048-kit-ships-the-governed-loop-skills/spec.md
//! Kit skill tests (spec 048): the kit ships one repository-invariant skill
//! set for the governed loop, and this repository runs the same set on
//! itself. Four adopters had each rewritten the same five loop skills the
//! kit lacked, and the kit's own copies had drifted (a renamed tool, a rule
//! format Claude Code does not read, a "read-only" review that wrote). These
//! tests pin the set, the frontmatter contract, the project-layer section
//! every skill must end with, and the read-only forms the read skills use.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The fifteen skills, the loop first, in the order "Working the backlog"
/// runs them.
const SKILLS: &[&str] = &[
    "init",
    "setup",
    "next",
    "build",
    "verify",
    "ship",
    "shepherd",
    "spec",
    "commit",
    "code-review",
    "validate-and-fix",
    "cleanup",
    "implement-plan",
    "research",
    "refactor-claude-md",
];

/// Skills that read and must never run a writing `spec-spine` verb.
const READ_ONLY: &[&str] = &["init", "next", "verify", "code-review"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn skill_dirs() -> [(&'static str, PathBuf); 2] {
    let root = repo_root();
    [
        ("kit", root.join("kit/.claude/skills")),
        ("self", root.join(".claude/skills")),
    ]
}

fn skill_names(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("SKILL.md").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}

fn read_skill(dir: &Path, name: &str) -> String {
    let p = dir.join(name).join("SKILL.md");
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// The YAML frontmatter block as `key: value` lines.
fn frontmatter(body: &str) -> Vec<(String, String)> {
    let mut lines = body.lines();
    assert_eq!(lines.next(), Some("---"), "frontmatter must open the file");
    lines
        .take_while(|l| *l != "---")
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

fn field<'a>(fm: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fm.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// Every `spec-spine <verb...>` invocation in a body, as the verb words up
/// to the first shell metacharacter or line end.
fn spec_spine_verbs(body: &str) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for line in body.lines() {
        let mut rest = line;
        while let Some(i) = rest.find("spec-spine ") {
            let after = &rest[i + "spec-spine ".len()..];
            let words: Vec<String> = after
                .split(|c: char| "|&;)`\"'".contains(c))
                .next()
                .unwrap_or("")
                .split_whitespace()
                .map(str::to_string)
                .collect();
            if !words.is_empty() && !words[0].starts_with('-') {
                out.push(words);
            }
            rest = after;
        }
    }
    out
}

fn is_write(verb: &[String]) -> bool {
    match verb.first().map(String::as_str) {
        Some("compile") => !verb.iter().any(|w| w == "--check"),
        Some("index") => verb.get(1).is_none_or(|w| w.starts_with('-')),
        Some("init") | Some("attest") => true,
        _ => false,
    }
}

#[test]
fn the_kit_and_this_repository_ship_the_same_fifteen_skills() {
    let want: BTreeSet<String> = SKILLS.iter().map(|s| s.to_string()).collect();
    for (label, dir) in skill_dirs() {
        assert_eq!(
            skill_names(&dir),
            want,
            "{label}: skill set differs from spec 048 3.1"
        );
    }
}

#[test]
fn the_skills_are_byte_identical_between_the_kit_and_this_repository() {
    let [(_, kit), (_, own)] = skill_dirs();
    for name in SKILLS {
        assert_eq!(
            read_skill(&kit, name),
            read_skill(&own, name),
            "{name}: the kit copy and this repository's copy differ (spec 048 3.3)"
        );
    }
}

#[test]
fn every_skill_declares_name_description_and_allowed_tools() {
    let [(_, kit), _] = skill_dirs();
    for name in SKILLS {
        let fm = frontmatter(&read_skill(&kit, name));
        assert_eq!(field(&fm, "name"), Some(*name), "{name}: frontmatter name");
        assert!(
            field(&fm, "description").is_some_and(|d| d.len() > 20),
            "{name}: needs a description"
        );
        assert!(
            field(&fm, "allowed-tools").is_some_and(|t| !t.is_empty()),
            "{name}: needs an allowed-tools list (spec 048 3.2)"
        );
    }
}

#[test]
fn every_skill_ends_with_a_project_layer_section() {
    let [(_, kit), _] = skill_dirs();
    for name in SKILLS {
        let body = read_skill(&kit, name);
        assert!(
            body.contains("\n## Project layer\n"),
            "{name}: missing the `## Project layer` section (spec 048 3.3)"
        );
    }
}

#[test]
fn no_skill_carries_a_project_specific_or_stale_reference() {
    let [(_, kit), _] = skill_dirs();
    let banned = [
        ("\u{2014}", "an em dash"),
        ("`Task`", "the renamed Task tool (it is Agent)"),
        ("allowed-tools: Task", "the renamed Task tool"),
        (
            "globs:",
            "a rule frontmatter key Claude Code does not read (use paths:)",
        ),
        (
            "imports:",
            "a rule frontmatter key Claude Code does not read",
        ),
        (
            "/tmp/",
            "a hardcoded temp directory (use the scratchpad or state_dir)",
        ),
        ("registry.json", "the pre-024 monolithic registry path"),
        ("index.json", "the pre-024 monolithic index path"),
        ("make spine", "one adopter's composite name"),
        ("hqgit", "an adopter's name"),
        ("aicortex", "an adopter's name"),
        ("rahi", "an adopter's name"),
        ("butler", "an adopter's name"),
        ("claude-observatory", "an adopter's name"),
        (
            "<your build command>",
            "a placeholder; skills read the gate from AGENTS.md",
        ),
    ];
    for name in SKILLS {
        let body = read_skill(&kit, name);
        for (needle, why) in banned {
            assert!(!body.contains(needle), "{name}: contains {needle:?}, {why}");
        }
    }
}

#[test]
fn read_skills_never_run_a_writing_verb() {
    let [(_, kit), _] = skill_dirs();
    for name in READ_ONLY {
        let body = read_skill(&kit, name);
        for verb in spec_spine_verbs(&body) {
            assert!(
                !is_write(&verb),
                "{name}: read-only skill invokes `spec-spine {}` (spec 048 3.4)",
                verb.join(" ")
            );
        }
    }
}

#[test]
fn the_loop_skills_wrap_the_tool_verbs_they_exist_for() {
    let [(_, kit), _] = skill_dirs();
    let must = [
        ("next", "registry plan --json"),
        ("build", "registry plan --json"),
        ("build", "implementation: in-progress"),
        ("verify", "spec-spine verify"),
        ("ship", "Spec-Drift-Waiver"),
        ("shepherd", "headRefOid"),
        ("spec", "registry list --ids-only"),
        ("spec", "status: draft"),
        ("init", "compile --check"),
        ("setup", "registry plan"),
        ("code-review", "compile --check"),
        ("commit", "session_"),
        ("commit", "U+2014"),
    ];
    for (name, needle) in must {
        assert!(
            read_skill(&kit, name).contains(needle),
            "{name}: must mention {needle:?} (spec 048 3.1)"
        );
    }
}

/// The script still ships for adopters pinned below 0.15.0, but spec 051 3.2
/// moved the harness onto `spec-spine verify`: no skill may call it, and the
/// kit README must say it is deprecated. Deleting it is 051 4's follow-on,
/// gated on the adopters upgrading, not on this spec merging.
#[test]
fn the_verify_script_ships_deprecated_and_no_skill_calls_it() {
    let root = repo_root();
    let kit = fs::read_to_string(root.join("kit/scripts/verify-spec.sh")).unwrap();
    let own = fs::read_to_string(root.join("scripts/verify-spec.sh")).unwrap();
    assert_eq!(kit, own, "scripts/verify-spec.sh differs from the kit copy");
    assert!(kit.starts_with("#!/usr/bin/env bash"));
    assert!(kit.contains("verify:cli") && kit.contains("verify:browser"));
    assert!(kit.contains("not-declared"), "an honest zero, not a pass");

    let readme = fs::read_to_string(root.join("kit/README.md")).unwrap();
    assert!(
        readme.contains("DEPRECATED"),
        "kit/README.md must mark the script deprecated (spec 051 3.2)"
    );

    for (label, dir) in skill_dirs() {
        for name in SKILLS {
            let body = read_skill(&dir, name);
            for line in body.lines() {
                assert!(
                    !line.contains("scripts/verify-spec.sh") || line.contains("Do NOT"),
                    "{label}/{name}: calls the deprecated script (spec 051 3.1)"
                );
            }
        }
    }
}

#[test]
fn the_kit_agents_carry_the_legitimate_edit_rule_and_no_em_dash() {
    let root = repo_root();
    for agent in ["architect", "explorer", "implementer", "reviewer"] {
        let p = root.join(format!("kit/.claude/agents/{agent}.md"));
        let body = fs::read_to_string(&p).unwrap();
        assert!(!body.contains('\u{2014}'), "{agent}: em dash");
    }
    let reviewer = fs::read_to_string(root.join("kit/.claude/agents/reviewer.md")).unwrap();
    assert!(
        reviewer.contains("legitimate mid-build edits"),
        "reviewer polices spec 047 3.2"
    );
    assert!(
        reviewer.contains("Gate Evidence"),
        "reviewer runs the gate as evidence"
    );
    let implementer = fs::read_to_string(root.join("kit/.claude/agents/implementer.md")).unwrap();
    assert!(implementer.contains("`establishes` list in the same change"));
}

#[test]
fn the_write_scanner_recognises_writes() {
    let w = |s: &str| s.split_whitespace().map(str::to_string).collect::<Vec<_>>();
    assert!(is_write(&w("compile")));
    assert!(is_write(&w("index")));
    assert!(is_write(&w("index --repo x")));
    assert!(!is_write(&w("compile --check")));
    assert!(!is_write(&w("index check")));
    assert!(!is_write(&w("index coverage --fail-on-untraced")));
    assert!(!is_write(&w("registry plan --json")));
    assert!(!is_write(&w("lint --fail-on-warn")));
    assert!(!is_write(&w("couple --base origin/main --head HEAD")));
}

// --- spec 051 3.3: one gate list ------------------------------------------

/// The gate chain, as `standards/spec/contract.md` defines it. Both the CI
/// scanner and the parse guard read this one list: two copies could drift, and
/// a verb missing from either makes the subset assertion vacuous for that verb
/// without failing. Adding a verb to the chain means adding it here.
const GOVERNANCE_VERBS: &[&str] = &["compile", "index", "lint", "couple"];

/// The leading words of a command, up to the first flag: `index check
/// --fail-on-unresolved` has the verb path `index check`.
fn verb_path(cmd: &str) -> String {
    cmd.split_whitespace()
        .take_while(|w| !w.starts_with('-'))
        .collect::<Vec<_>>()
        .join(" ")
}

fn fail_flags(cmd: &str) -> Vec<String> {
    cmd.split_whitespace()
        .filter(|w| w.starts_with("--fail-on-"))
        .map(str::to_string)
        .collect()
}

/// The `spec-spine ...` lines inside the fenced block under "Run the gate
/// before every commit".
fn agents_md_gate_commands(root: &Path) -> Vec<String> {
    let text = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    // Anchoring on the first occurrence is only safe while there is exactly
    // one. A usage example quoting the phrase would silently move the anchor,
    // and a decoy fence of governance verbs would then pass the guard below.
    assert_eq!(
        text.matches("Run the gate before every commit").count(),
        1,
        "AGENTS.md must name the gate step exactly once, or this parse anchors on the wrong one"
    );
    let start = text
        .find("Run the gate before every commit")
        .expect("AGENTS.md names the gate step");
    let tail = &text[start..];
    let open = tail
        .find("```sh")
        .expect("the gate list is a fenced sh block");
    let body = &tail[open + "```sh".len()..];
    let close = body.find("```").expect("the fence closes");
    body[..close]
        .lines()
        .filter_map(|l| l.trim().strip_prefix("spec-spine "))
        .map(|c| c.trim().to_string())
        .collect()
}

/// The governance verbs CI actually runs, however the binary is spelled there.
fn ci_governance_commands(root: &Path) -> Vec<String> {
    let text = fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(i) = trimmed.find("spec-spine ") {
            let cmd = trimmed[i + "spec-spine ".len()..].trim();
            let head = cmd.split_whitespace().next().unwrap_or("");
            if GOVERNANCE_VERBS.contains(&head) {
                out.push(cmd.to_string());
            }
        }
    }
    out
}

/// Spec 051 3.3. Every skill tells its reader to run "the gate as `AGENTS.md`
/// lists it", so a step CI enforces and that list omits is a step every session
/// skips. That is the drift this spec was filed for: `index coverage
/// --fail-on-untraced` was enforced in CI and absent from the list four skills
/// defer to, leaving the skills more correct than their own authority.
#[test]
fn agents_md_gate_list_names_every_step_ci_enforces() {
    let root = repo_root();
    let listed = agents_md_gate_commands(&root);
    assert!(
        !listed.is_empty(),
        "AGENTS.md must carry a fenced gate list"
    );
    // Guard the parse itself: `find` takes the first match, so a fence added
    // above the gate list would be read as the gate list, and every assertion
    // below would go vacuous without failing. Every line of the real block is
    // a governance verb, so anything else means we read the wrong fence.
    for cmd in &listed {
        let head = cmd.split_whitespace().next().unwrap_or("");
        assert!(
            GOVERNANCE_VERBS.contains(&head),
            "AGENTS.md gate-list parse read the wrong block: got `{cmd}`"
        );
    }

    for ci in ci_governance_commands(&root) {
        // CI runs `compile --check` where a local session runs `compile`: a
        // gate must never repair the tree it is judging. Comparing verb paths
        // makes the two the same step, which is what they are.
        let path = verb_path(&ci);
        let matching: Vec<&String> = listed.iter().filter(|l| verb_path(l) == path).collect();
        assert!(
            !matching.is_empty(),
            "CI runs `spec-spine {ci}` but AGENTS.md's gate list names no `{path}` step"
        );
        for flag in fail_flags(&ci) {
            assert!(
                matching.iter().any(|l| l.contains(&flag)),
                "CI runs `spec-spine {ci}` but AGENTS.md's `{path}` step omits {flag}"
            );
        }
    }
}

/// The other direction of 3.3, as a subset relation: a skill may name fewer
/// steps than the full gate, but never a gating flag its authority omits.
#[test]
fn no_skill_names_a_gate_flag_agents_md_omits() {
    let root = repo_root();
    let listed = agents_md_gate_commands(&root);
    for (label, dir) in skill_dirs() {
        for name in SKILLS {
            let body = read_skill(&dir, name);
            for verb in spec_spine_verbs(&body) {
                let cmd = verb.join(" ");
                for flag in fail_flags(&cmd) {
                    assert!(
                        listed.iter().any(|l| l.contains(&flag)),
                        "{label}/{name}: names {flag}, which AGENTS.md's gate list omits"
                    );
                }
            }
        }
    }
}
