#!/usr/bin/env python3
# Spec: specs/112-nothing-reruns-a-merged-acceptance/spec.md
"""Run the four post-ratification sweep regressions in disposable repositories.

Build target/release/spec-spine first, then run python3 scripts/test-verify-sweep.py.
Use --sweep-script PATH to test an exported historical script as a negative
control. Requires Python's standard library, Bash, Git and the in-tree binary;
SPEC_SPINE_BIN can supply another already-built binary explicitly.
"""

import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest


SOURCE = Path(__file__).resolve().parents[1]
BIN = Path(os.environ.get("SPEC_SPINE_BIN", SOURCE / "target/release/spec-spine")).resolve()
SCRIPT = SOURCE / "scripts/verify-sweep.sh"


def run(args, cwd=None, **kwargs):
    return subprocess.run(
        [str(arg) for arg in args], cwd=cwd, text=True, capture_output=True, **kwargs
    )


def checked(args, cwd=None):
    result = run(args, cwd)
    if result.returncode:
        raise AssertionError(
            f"{args}: exit {result.returncode}\n{result.stdout}\n{result.stderr}"
        )
    return result.stdout.strip()


class SweepRegressions(unittest.TestCase):
    def setUp(self):
        # Include a space to exercise quoting. Every fixture, worktree, ledger,
        # report and marker lives until assertions finish, including on failure.
        scratch = tempfile.TemporaryDirectory(prefix="spec112 regressions-")
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name).resolve()
        self.repo = self.root / "repo"
        self.out = self.root / "run"

    def fixture(self, specs):
        self.repo.mkdir()
        (self.repo / "spec-spine.toml").write_text(
            '[layout]\nspecs_dir = "specs"\n', encoding="utf-8"
        )
        (self.repo / "sentinel").write_bytes(b"original\n")
        for sid, commands in specs.items():
            path = self.repo / "specs" / sid / "spec.md"
            path.parent.mkdir(parents=True)
            body = "\n".join([
                "---", f'id: "{sid}"', f'title: "{sid}"', "status: draft",
                "implementation: pending", 'created: "2026-09-17"',
                'summary: "Fixture"', "---", f"# {sid}", "",
            ])
            if commands:
                body += "\n## Verification\n\n```verify:cli\n"
                body += "\n".join(commands) + "\n```\n"
            path.write_text(body, encoding="utf-8")
        checked([BIN, "--repo", self.repo, "compile"])
        checked([BIN, "--repo", self.repo, "index"])
        checked(["git", "init", "-q", "-b", "main", self.repo])
        checked(["git", "config", "user.name", "Fixture"], self.repo)
        checked(["git", "config", "user.email", "fixture@example.invalid"], self.repo)

    def commit(self):
        checked(["git", "add", "-A"], self.repo)
        checked(["git", "commit", "-qm", "fixture"], self.repo)
        self.sha = checked(["git", "rev-parse", "HEAD"], self.repo)

    def sweep(self, ledger_text="", extra=()):
        ledger = self.root / "exempt.txt"
        ledger.write_text(ledger_text, encoding="utf-8")
        result = run([
            "bash", SCRIPT, "--repo", self.repo, "--trusted-ref", "main",
            "--exempt-file", ledger, "--out", self.out, "--timeout", "5", *extra,
        ], env=dict(os.environ, SPEC_SPINE_BIN=str(BIN)))
        self.assertFalse((self.out / "tree").exists(), result.stderr)
        self.assertEqual(checked(["git", "status", "--porcelain"], self.repo), "")
        self.assertEqual(checked(["git", "rev-parse", "HEAD"], self.repo), self.sha)
        self.assertEqual((self.repo / "sentinel").read_bytes(), b"original\n")
        return result

    def report(self, counts):
        report = json.loads((self.out / "sweep.json").read_text(encoding="utf-8"))
        expected = dict.fromkeys(("passed", "failed", "not-declared", "exempt", "not-run"), 0)
        expected.update(counts)
        self.assertEqual(report["counts"], expected)
        self.assertEqual(report["revision"], self.sha)
        self.assertEqual(report["trustedRef"], "main")
        self.assertEqual(report["binaryOrigin"], "SPEC_SPINE_BIN")
        self.assertEqual(report["binaryVersion"], checked([BIN, "--version"]))
        self.assertEqual(report["ledgerOrigin"], str(self.root / "exempt.txt"))
        self.assertEqual(report["ledgerClosedAt"], 48)
        for row in report["specs"]:
            if row["log"] is not None:
                self.assertTrue((self.out / row["log"]).is_file(), row)
        return report

    def test_unreadable_exemption_is_not_run(self):
        self.fixture({"000-legacy": [], "001-green": ["true"]})
        # Preserve the compiled registry's member while removing its document.
        (self.repo / "specs/000-legacy/spec.md").unlink()
        self.commit()
        plan = run([BIN, "--repo", self.repo, "verify", "000-legacy", "--plan"])
        self.assertNotEqual(plan.returncode, 0)
        result = self.sweep("000-legacy\n", ("--only", "000"))
        self.assertEqual(result.returncode, 1, result.stderr)
        report = self.report({"not-run": 1})
        self.assertEqual(report["selection"], "000")
        self.assertEqual(len(report["specs"]), 1)
        row = report["specs"][0]
        self.assertEqual(row["id"], "000-legacy")
        self.assertEqual(row["outcome"], "not-run")
        self.assertEqual(row["commands"], 0)
        self.assertEqual(row["exitCode"], plan.returncode)
        self.assertEqual(row["failure"], f"no verdict: verify --plan exited {plan.returncode}")
        self.assertFalse(row["leftTreeDirty"])
        self.assertIsNone(row["log"])
        self.assertEqual(list((self.out / "logs").iterdir()), [])
        summary = (self.out / "sweep.md").read_text(encoding="utf-8")
        self.assertIn("**000-legacy** not-run", summary)
        self.assertNotIn("logs/000-legacy.log", summary)

    def residue_case(self, mutation):
        # The next spec checks exact bytes in all three layers, the revision,
        # detached state and cleanliness. A truthful dirty flag alone is not
        # evidence that restoration succeeded.
        check_clean = (
            "import pathlib, subprocess; "
            "git = lambda *args: subprocess.check_output(['git', *args]); "
            "assert pathlib.Path('sentinel').read_bytes() == b'original\\n'; "
            "assert git('show', ':sentinel') == b'original\\n'; "
            "assert git('show', 'HEAD:sentinel') == b'original\\n'; "
            "assert git('rev-parse', 'HEAD') == git('rev-parse', 'main'); "
            "assert git('rev-parse', '--abbrev-ref', 'HEAD').strip() == b'HEAD'; "
            "assert git('status', '--porcelain') == b''; "
            "print('clean revision bytes observed')"
        )
        self.fixture({
            "001-dirty": [mutation],
            "002-clean": ["python3 -c " + shlex.quote(check_clean)],
        })
        self.commit()
        result = self.sweep()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = self.report({"passed": 2})
        self.assertEqual(report["selection"], "all")
        self.assertEqual([row["id"] for row in report["specs"]], ["001-dirty", "002-clean"])
        for row, dirty in zip(report["specs"], (True, False)):
            self.assertEqual(row["outcome"], "passed")
            self.assertEqual(row["commands"], 1)
            self.assertEqual(row["exitCode"], 0)
            self.assertEqual(row["failure"], "")
            self.assertIs(row["leftTreeDirty"], dirty)
            self.assertEqual(row["log"], f"logs/{row['id']}.log")
        self.assertIn("mutation observed", (self.out / "logs/001-dirty.log").read_text())
        self.assertIn("clean revision bytes observed", (self.out / "logs/002-clean.log").read_text())

    def test_staged_residue_restored_before_next_spec(self):
        self.residue_case(
            "printf 'changed\\n' > sentinel && git add sentinel && "
            "test \"$(git show :sentinel)\" = changed && "
            "printf 'mutation observed\\n'"
        )

    def test_committed_residue_restored_before_next_spec(self):
        self.residue_case(
            "printf 'changed\\n' > sentinel && git add sentinel && "
            "git commit -qm residue && test -z \"$(git status --porcelain)\" && "
            "test \"$(git rev-parse HEAD)\" != \"$(git rev-parse main)\" && "
            "printf 'mutation observed\\n'"
        )

    def test_restore_failure_prevents_next_spec_execution(self):
        marker = self.root / "next-ran"
        self.fixture({
            "001-lock": [
                'touch residue.txt && touch "$(git rev-parse --git-path index.lock)" '
                "&& printf 'restore lock created\\n'"
            ],
            "002-next": ["touch " + shlex.quote(str(marker))],
        })
        self.commit()
        result = self.sweep()
        self.assertEqual(result.returncode, 3, result.stderr)
        self.assertIn(f"cannot restore the isolated worktree to {self.sha} after 001-lock", result.stderr)
        self.assertIn("restore lock created", (self.out / "logs/001-lock.log").read_text())
        self.assertFalse(marker.exists(), "the next acceptance block executed")
        self.assertFalse((self.out / "logs/002-next.log").exists())
        self.assertFalse((self.out / "sweep.json").exists())
        self.assertFalse((self.out / "sweep.md").exists())


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sweep-script", type=Path, default=SCRIPT)
    args = parser.parse_args()
    SCRIPT = args.sweep_script.resolve()
    if not BIN.is_file() or not os.access(BIN, os.X_OK):
        parser.error("build the in-tree binary: cargo build --release --locked -p spec-spine-cli")
    if not SCRIPT.is_file():
        parser.error(f"sweep script not found: {SCRIPT}")
    unittest.main(argv=[__file__], verbosity=2)
