#!/usr/bin/env python3
"""Generate this repository's agent instruction trees from one source (spec 100).

Four agents read instructions out of this repository and each wants them at a
different path. Until spec 100 the non-Claude trees were hand-copied once, by a
blind `claude` -> `Codex` substitution that invented a `.Codex/rules/` directory
existing on no filesystem, and then drifted for a week with nothing detecting it
(spec 100 1.1, 1.2).

    .claude/skills/<name>/SKILL.md   <- kit/.claude/skills/<name>/SKILL.md
    .agents/skills/<name>/SKILL.md   <- kit/.claude/skills/<name>/SKILL.md
    .codex/agents/<name>.toml        <- .claude/agents/<name>.md

The skill trees are identity copies. Nothing is rewritten, ever: a path is a
fact about a filesystem and not a brand, and the rules every agent reads are at
`.claude/rules/` (spec 100 3.2). The Codex agent projection is structural only,
because TOML is not markdown (spec 100 3.3).

    python3 scripts/gen-agent-trees.py            # write
    python3 scripts/gen-agent-trees.py --check    # verify, write nothing

`crates/spec-spine-core/tests/agent_trees.rs` asserts the same thing against the
committed bytes, so a tree cannot drift without a red test.
"""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

SKILL_SOURCE = ROOT / "kit/.claude/skills"
SKILL_DESTS = [ROOT / ".claude/skills", ROOT / ".agents/skills"]

AGENT_SOURCE = ROOT / ".claude/agents"
AGENT_DEST = ROOT / ".codex/agents"

# Frontmatter keys a Codex agent definition has a use for. The rest (`tools`,
# `model`, `safety_tier`, `mutation`, `memory`) are dropped rather than
# translated: spec 100 3.3 declines to invent a Codex meaning for a Claude
# field.
TOML_KEYS = ("name", "description")


def split_frontmatter(text: str, src: Path) -> tuple[dict[str, str], str]:
    """The `key: value` scalars of the YAML block, and the body after it.

    Only scalars are read. A block sequence (`tools:` and its `- Read` lines)
    parses as a key with an empty value followed by lines this function ignores,
    which is exactly right: no sequence key is in TOML_KEYS.
    """
    lines = text.split("\n")
    if not lines or lines[0] != "---":
        sys.exit(f"{src}: expected a YAML frontmatter block opening with `---`")
    try:
        end = lines.index("---", 1)
    except ValueError:
        sys.exit(f"{src}: the frontmatter block is not closed")
    fields = {}
    for line in lines[1:end]:
        if line.startswith((" ", "\t", "-")) or ":" not in line:
            continue
        key, _, value = line.partition(":")
        fields[key.strip()] = value.strip()
    # The body starts after the closing `---` and its newline.
    return fields, "\n".join(lines[end + 1 :]).lstrip("\n")


def toml_basic(value: str, src: Path, key: str) -> str:
    """A TOML basic string. Refuses rather than mangling what it cannot encode."""
    if '"' in value or "\\" in value:
        sys.exit(f"{src}: `{key}` contains a quote or backslash, which spec 100 3.3 does not specify an encoding for")
    return f'"{value}"'


def project_agent(src: Path) -> str:
    fields, body = split_frontmatter(src.read_text(encoding="utf-8"), src)
    for key in TOML_KEYS:
        if not fields.get(key):
            sys.exit(f"{src}: the frontmatter has no `{key}`")
    # Spec 100 3.3 and D-4: refuse a body TOML cannot carry verbatim rather than
    # emit a silent escape whose meaning depends on where the quotes fall. A
    # multi-line BASIC string processes escapes, so a lone backslash is the
    # subtler hazard of the three: `\n` in the source decodes to a newline and a
    # backslash before a line ending swallows the line break entirely.
    if "\\" in body:
        sys.exit(f"{src}: the body contains a backslash, which a TOML basic string would decode as an escape (spec 100 3.3)")
    # The only quote sequence a multi-line basic string cannot carry. TOML 1.0
    # allows one or two quotes immediately before the closing delimiter, so a
    # body ending in `"` or `""` is valid and is NOT refused; three in a row
    # ends the string early wherever they appear, which is what this catches.
    if '"""' in body:
        sys.exit(f'{src}: the body contains `"""`, which spec 100 3.3 refuses rather than escapes')
    out = [f"{key} = {toml_basic(fields[key], src, key)}" for key in TOML_KEYS]
    out.append(f'developer_instructions = """\n{body.rstrip(chr(10))}"""')
    return "\n".join(out) + "\n"


def generated() -> dict[Path, str]:
    """Every destination path and the bytes it must hold."""
    if not SKILL_SOURCE.is_dir():
        sys.exit(f"{SKILL_SOURCE}: not a directory")
    out: dict[Path, str] = {}
    for skill in sorted(p for p in SKILL_SOURCE.iterdir() if (p / "SKILL.md").is_file()):
        text = (skill / "SKILL.md").read_text(encoding="utf-8")
        for dest in SKILL_DESTS:
            out[dest / skill.name / "SKILL.md"] = text
    for agent in sorted(AGENT_SOURCE.glob("*.md")):
        out[AGENT_DEST / f"{agent.stem}.toml"] = project_agent(agent)
    return out


def stale_destinations(wanted: dict[Path, str]) -> list[Path]:
    """Committed files under a destination tree that no source maps to.

    Spec 100 3.1: a skill deleted from the kit is deleted from every generated
    tree. Without this the five skills spec 081 removed would still be here.
    """
    found = []
    for dest in SKILL_DESTS:
        found += [p for p in dest.glob("*/SKILL.md")] if dest.is_dir() else []
    found += [p for p in AGENT_DEST.glob("*.toml")] if AGENT_DEST.is_dir() else []
    return sorted(p for p in found if p not in wanted)


def main() -> int:
    check = "--check" in sys.argv[1:]
    wanted = generated()
    stale = stale_destinations(wanted)

    differing = [p for p, text in wanted.items() if not p.is_file() or p.read_text(encoding="utf-8") != text]

    if check:
        for path in differing:
            print(f"differs: {path.relative_to(ROOT)}", file=sys.stderr)
        for path in stale:
            print(f"no source maps to: {path.relative_to(ROOT)}", file=sys.stderr)
        if differing or stale:
            print(
                f"{len(differing) + len(stale)} generated file(s) out of date; run `python3 scripts/gen-agent-trees.py`",
                file=sys.stderr,
            )
            return 1
        print(f"{len(wanted)} generated file(s) up to date")
        return 0

    for path, text in wanted.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    for path in stale:
        path.unlink()
        # A skill directory exists only to hold its SKILL.md.
        if path.parent != AGENT_DEST and not any(path.parent.iterdir()):
            path.parent.rmdir()
    print(f"wrote {len(wanted)} file(s), removed {len(stale)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
