// Spec: specs/100-one-source-generates-the-agent-trees/spec.md
//
// Spec 100 3.6: the generated agent instruction trees are asserted against the
// bytes on disk, never against a re-run of `scripts/gen-agent-trees.py`. A test
// that regenerates and compares the output to itself passes on an empty
// repository and proves nothing about what is committed, which is the failure
// mode the committed `.derived/` trees exist to avoid.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Spec 081 3.1's set, which spec 100 3.4 makes the generated trees' set too.
const SKILLS: &[&str] = &[
    "prime",
    "setup",
    "next",
    "build",
    "verify",
    "ship",
    "shepherd",
    "spec",
    "commit",
    "code-review",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Directory entry names holding the file named by `holds`.
fn entries(dir: &Path, holds: &str) -> BTreeSet<String> {
    fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .filter(|e| {
            if holds.is_empty() {
                e.path().is_file()
            } else {
                e.path().join(holds).is_file()
            }
        })
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}

/// The three trees spec 100 3.1 maps `kit/.claude/skills/` onto.
fn skill_trees() -> [(&'static str, PathBuf); 3] {
    let root = repo_root();
    [
        ("kit/.claude/skills", root.join("kit/.claude/skills")),
        (".claude/skills", root.join(".claude/skills")),
        (".agents/skills", root.join(".agents/skills")),
    ]
}

#[test]
fn every_generated_skill_tree_is_byte_identical_to_the_kit() {
    let [(_, kit), (b_label, b), (c_label, c)] = skill_trees();
    for name in SKILLS {
        let source = read(&kit.join(name).join("SKILL.md"));
        for (label, dir) in [(b_label, &b), (c_label, &c)] {
            assert_eq!(
                source,
                read(&dir.join(name).join("SKILL.md")),
                "{label}/{name}/SKILL.md differs from the kit source \
                 (spec 100 3.1; run `python3 scripts/gen-agent-trees.py`)"
            );
        }
    }
}

#[test]
fn the_generated_skill_trees_hold_exactly_the_kits_skill_set() {
    let expected: BTreeSet<String> = SKILLS.iter().map(|s| (*s).to_owned()).collect();
    for (label, dir) in skill_trees() {
        assert_eq!(
            entries(&dir, "SKILL.md"),
            expected,
            "{label}: skill set differs from spec 081 3.1, which spec 100 3.4 \
             extends to the generated trees. A skill removed from the kit must be \
             removed from every tree."
        );
    }
}

#[test]
fn every_claude_agent_has_a_codex_projection() {
    let root = repo_root();
    let sources = entries(&root.join(".claude/agents"), "");
    let projections = entries(&root.join(".codex/agents"), "");
    let expected: BTreeSet<String> = sources
        .iter()
        .filter(|n| n.ends_with(".md"))
        .map(|n| n.replace(".md", ".toml"))
        .collect();
    assert!(!expected.is_empty(), ".claude/agents holds no .md source");
    assert_eq!(
        projections, expected,
        ".codex/agents does not mirror .claude/agents (spec 100 3.1, 3.3)"
    );
}

#[test]
fn the_codex_projection_round_trips_the_claude_source() {
    let root = repo_root();
    for name in entries(&root.join(".claude/agents"), "")
        .iter()
        .filter(|n| n.ends_with(".md"))
    {
        let stem = name.trim_end_matches(".md");
        let source = read(&root.join(".claude/agents").join(name));
        let (fields, body) = split_frontmatter(&source, name);

        let projected = read(&root.join(".codex/agents").join(format!("{stem}.toml")));
        let parsed: toml::Table = projected
            .parse()
            .unwrap_or_else(|e| panic!(".codex/agents/{stem}.toml: not valid TOML: {e}"));

        for key in ["name", "description"] {
            assert_eq!(
                parsed.get(key).and_then(|v| v.as_str()),
                Some(fields(key).as_str()),
                ".codex/agents/{stem}.toml: `{key}` is not the source frontmatter's (spec 100 3.3)"
            );
        }
        // Spec 100 3.6: the DECODED value, not the file text. A body carrying a
        // backslash or a `\"\"\"` produces a file that still looks right and
        // decodes wrong, which is the whole reason 3.3 refuses those bodies.
        assert_eq!(
            parsed
                .get("developer_instructions")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!(".codex/agents/{stem}.toml: no developer_instructions")),
            body.trim_end_matches('\n'),
            ".codex/agents/{stem}.toml: developer_instructions does not round-trip \
             the body of .claude/agents/{name} (spec 100 3.3, 3.6)"
        );
        // Spec 100 3.3: the refusals the round-trip above depends on.
        assert!(
            !body.contains('\\') && !body.contains("\"\"\""),
            ".claude/agents/{name}: a body TOML would reinterpret reached a projection \
             (spec 100 3.3 refuses it at the generator)"
        );
    }
}

#[test]
fn no_generated_tree_carries_the_substitution() {
    let root = repo_root();
    // Read the tree rather than `SKILLS`: a skill present in `.agents/skills`
    // but absent from the constant would otherwise be skipped here. The set
    // test above would catch the divergence, but only when it is also run, and
    // each `#[test]` can be run alone.
    let mut files: Vec<PathBuf> = entries(&root.join(".agents/skills"), "SKILL.md")
        .iter()
        .map(|s| root.join(".agents/skills").join(s).join("SKILL.md"))
        .collect();
    files.extend(
        entries(&root.join(".codex/agents"), "")
            .iter()
            .map(|n| root.join(".codex/agents").join(n)),
    );
    for path in files {
        let body = read(&path);
        for banned in [".Codex/", "Codex.ai"] {
            assert!(
                !body.contains(banned),
                "{}: contains `{banned}`, the blind `claude` to `Codex` substitution \
                 spec 100 3.2 forbids. A generated tree is a projection of its source, \
                 not a search and replace of it.",
                path.display()
            );
        }
    }
}

/// The frontmatter scalars and the body after the closing `---`, matching
/// `scripts/gen-agent-trees.py`.
fn split_frontmatter(text: &str, label: &str) -> (impl Fn(&str) -> String + use<>, String) {
    let lines: Vec<&str> = text.split('\n').collect();
    assert_eq!(lines.first(), Some(&"---"), "{label}: no frontmatter block");
    let end = lines[1..]
        .iter()
        .position(|l| *l == "---")
        .unwrap_or_else(|| panic!("{label}: frontmatter block is not closed"))
        + 1;
    let fields: Vec<(String, String)> = lines[1..end]
        .iter()
        .filter(|l| !l.starts_with([' ', '\t', '-']))
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect();
    let body = lines[end + 1..]
        .join("\n")
        .trim_start_matches('\n')
        .to_owned();
    let label = label.to_owned();
    let get = move |key: &str| {
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("{label}: frontmatter has no `{key}`"))
    };
    (get, body)
}
