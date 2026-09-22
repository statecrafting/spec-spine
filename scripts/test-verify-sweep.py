#!/usr/bin/env python3
# Spec: specs/089-nothing-reruns-a-merged-acceptance/spec.md
"""Run the sweep regressions in disposable repositories.

The first four are spec 089's post-ratification review regressions. Then spec
119's: the release verdict, the lifecycle it reads, and the default run
directory. The last are spec 121's: the Acceptance workflow's report step,
driven by real release-mode sweeps.

Build target/release/spec-spine first, then run python3 scripts/test-verify-sweep.py.
Use --sweep-script PATH to test an exported historical script as a negative
control. Requires Python's standard library, Bash, Git and the in-tree binary;
SPEC_SPINE_BIN can supply another already-built binary explicitly.
"""

import argparse
import json
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import subprocess
import tempfile
import unittest


SOURCE = Path(__file__).resolve().parents[1]
BIN = Path(os.environ.get("SPEC_SPINE_BIN", SOURCE / "target/release/spec-spine")).resolve()
SCRIPT = SOURCE / "scripts/verify-sweep.sh"
REPORTER = SOURCE / "scripts/acceptance-report.py"


def ledger_closed_at(script):
    # Read from the script under test rather than pinned: spec 095 moved it from
    # 48 to 43, and a literal here went stale without anything running it.
    match = re.search(r"^readonly LEDGER_CLOSED_AT=([0-9]+)$", script.read_text(), re.M)
    if not match:
        raise AssertionError(f"{script}: no LEDGER_CLOSED_AT")
    return int(match.group(1))


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

    def fixture(self, specs, lifecycle=None):
        # `lifecycle` maps an id to (status, implementation); implementation
        # None omits the key. Unlisted specs are draft / pending, as before.
        lifecycle = lifecycle or {}
        self.repo.mkdir()
        (self.repo / "spec-spine.toml").write_text(
            '[layout]\nspecs_dir = "specs"\n', encoding="utf-8"
        )
        (self.repo / "sentinel").write_bytes(b"original\n")
        for sid, commands in specs.items():
            path = self.repo / "specs" / sid / "spec.md"
            path.parent.mkdir(parents=True)
            status, impl = lifecycle.get(sid, ("draft", "pending"))
            head = ["---", f'id: "{sid}"', f'title: "{sid}"', f"status: {status}"]
            if impl is not None:
                head.append(f"implementation: {impl}")
            body = "\n".join(head + [
                'created: "2026-09-17"', 'summary: "Fixture"', "---", f"# {sid}", "",
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

    def sweep(self, ledger_text="", extra=(), out=True, env=None, bin=None):
        ledger = self.root / "exempt.txt"
        ledger.write_text(ledger_text, encoding="utf-8")
        args = ["bash", SCRIPT, "--repo", self.repo, "--trusted-ref", "main",
                "--exempt-file", ledger, "--timeout", "5", *extra]
        if out:
            args += ["--out", self.out]
        result = run(args, env=dict(env or os.environ, SPEC_SPINE_BIN=str(bin or BIN)))
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
        self.assertEqual(report["ledgerClosedAt"], ledger_closed_at(SCRIPT))
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


    # --- spec 119: the release verdict --------------------------------------

    def release_report(self):
        report = json.loads((self.out / "sweep.json").read_text(encoding="utf-8"))
        self.assertEqual(report["schemaVersion"], "1.1.0")
        # 1.1.0 is additive: the corpus counts keep exactly their five keys.
        self.assertEqual(
            sorted(report["counts"]),
            ["exempt", "failed", "not-declared", "not-run", "passed"],
        )
        return report, {row["id"]: row for row in report["specs"]}

    def test_pending_work_is_visible_and_outside_the_release_verdict(self):
        self.fixture(
            {
                "001-built": ["true"],
                "002-pending-red": ["false"],
                "003-in-progress-red": ["false"],
                "004-deferred-green": ["true"],
            },
            {
                "001-built": ("approved", "complete"),
                "002-pending-red": ("draft", "pending"),
                "003-in-progress-red": ("draft", "in-progress"),
                "004-deferred-green": ("approved", "deferred"),
            },
        )
        self.commit()
        # Without --release, nothing changed: two blocks fail, exit 1.
        corpus = self.sweep()
        self.assertEqual(corpus.returncode, 1, corpus.stderr)
        report, rows = self.release_report()
        self.assertEqual(report["mode"], "corpus")
        self.assertEqual(report["counts"]["failed"], 2)
        self.assertEqual(report["verdicts"], {"corpus": "not-clean", "release": "clean"})
        # With --release, the same run exits on the release verdict.
        release = self.sweep(extra=("--release",))
        self.assertEqual(release.returncode, 0, release.stderr)
        report, rows = self.release_report()
        self.assertEqual(report["mode"], "release")
        self.assertEqual(report["counts"], {
            "passed": 2, "failed": 2, "not-declared": 0, "exempt": 0, "not-run": 0,
        })
        self.assertEqual(report["releaseCounts"], {
            "passed": 1, "failed": 0, "not-declared": 0, "exempt": 0,
            "not-run": 0, "pending": 3,
        })
        # Pending work keeps its real outcome and lifecycle; a green deferred
        # block is not `passed` for the release.
        self.assertEqual(rows["002-pending-red"]["outcome"], "failed")
        self.assertEqual(rows["002-pending-red"]["releaseOutcome"], "pending")
        self.assertEqual(rows["003-in-progress-red"]["lifecycle"],
                         {"status": "draft", "implementation": "in-progress", "class": "pending"})
        self.assertEqual(rows["004-deferred-green"]["outcome"], "passed")
        self.assertEqual(rows["004-deferred-green"]["releaseOutcome"], "pending")
        self.assertEqual(rows["001-built"]["releaseOutcome"], "passed")
        summary = (self.out / "sweep.md").read_text(encoding="utf-8")
        self.assertIn("## Release verdict", summary)
        self.assertIn("- 002-pending-red: status draft, implementation pending; block failed", summary)
        self.assertIn("release verdict: clean", release.stderr)

    def test_an_implemented_draft_that_fails_fails_the_release(self):
        self.fixture(
            {"001-draft-built-red": ["false"], "002-approved-green": ["true"]},
            {
                "001-draft-built-red": ("draft", "complete"),
                "002-approved-green": ("approved", "complete"),
            },
        )
        self.commit()
        result = self.sweep(extra=("--release",))
        self.assertEqual(result.returncode, 1, result.stderr)
        report, rows = self.release_report()
        self.assertEqual(rows["001-draft-built-red"]["releaseOutcome"], "failed")
        self.assertEqual(report["verdicts"]["release"], "not-clean")
        self.assertIn("**001-draft-built-red** failed", (self.out / "sweep.md").read_text())

    def test_an_absent_implementation_defers_to_status(self):
        # Spec 042: absent on a draft reads as pending; absent on anything
        # ratified is settled, so it is judged.
        self.fixture(
            {"001-draft-absent": ["false"], "002-approved-absent": ["false"]},
            {
                "001-draft-absent": ("draft", None),
                "002-approved-absent": ("approved", None),
            },
        )
        self.commit()
        result = self.sweep(extra=("--release",))
        self.assertEqual(result.returncode, 1, result.stderr)
        _, rows = self.release_report()
        self.assertEqual(rows["001-draft-absent"]["lifecycle"],
                         {"status": "draft", "implementation": None, "class": "pending"})
        self.assertEqual(rows["001-draft-absent"]["releaseOutcome"], "pending")
        self.assertEqual(rows["002-approved-absent"]["lifecycle"]["class"], "implemented")
        self.assertEqual(rows["002-approved-absent"]["releaseOutcome"], "failed")

    def test_pending_work_is_never_exempt_and_an_unreadable_plan_is_never_pending(self):
        self.fixture(
            {"000-legacy-pending": [], "001-ghost-pending": ["true"], "002-green": ["true"]},
            {"002-green": ("approved", "complete")},
        )
        (self.repo / "specs/001-ghost-pending/spec.md").unlink()
        self.commit()
        result = self.sweep("000-legacy-pending\n", ("--release",))
        self.assertEqual(result.returncode, 1, result.stderr)
        report, rows = self.release_report()
        # On the ledger and pending: the corpus outcome is `exempt`, the
        # release outcome is `pending`, never an exemption of unbuilt work.
        self.assertEqual(rows["000-legacy-pending"]["outcome"], "exempt")
        self.assertEqual(rows["000-legacy-pending"]["releaseOutcome"], "pending")
        # A plan that cannot be read is `not-run` for the release too, even on
        # a pending spec: missing evidence is not deferred work.
        self.assertEqual(rows["001-ghost-pending"]["outcome"], "not-run")
        self.assertEqual(rows["001-ghost-pending"]["releaseOutcome"], "not-run")
        self.assertEqual(report["releaseCounts"]["not-run"], 1)
        self.assertEqual(report["releaseCounts"]["exempt"], 0)

    def test_a_timed_out_implemented_block_is_not_run_for_the_release(self):
        self.fixture({"001-slow": ["sleep 30"]}, {"001-slow": ("approved", "complete")})
        self.commit()
        result = self.sweep(extra=("--release",))
        self.assertEqual(result.returncode, 1, result.stderr)
        _, rows = self.release_report()
        self.assertEqual(rows["001-slow"]["outcome"], "not-run")
        self.assertEqual(rows["001-slow"]["releaseOutcome"], "not-run")

    def test_a_spec_the_lifecycle_read_does_not_answer_for_is_not_run(self):
        # Defense in depth: the ids and the lifecycle come from two reads of
        # one ledger and cannot disagree today, so a stub binary makes them.
        # A spec with no lifecycle is not judged a release success, however
        # its block ends.
        self.fixture(
            {"001-green": ["true"], "002-green": ["true"]},
            {"001-green": ("approved", "complete"), "002-green": ("approved", "complete")},
        )
        self.commit()
        stub = self.root / "stub-spec-spine"
        stub.write_text(
            "#!/usr/bin/env bash\n"
            'case " $* " in *" registry list --json "*)\n'
            f'  {shlex.quote(str(BIN))} "$@" | python3 -c \''
            "import json,sys; d=json.load(sys.stdin); "
            'd[\"items\"]=[i for i in d[\"items\"] if i[\"id\"]!=\"001-green\"]; '
            "json.dump(d,sys.stdout)'\n"
            "  exit ;;\n"
            "esac\n"
            f'exec {shlex.quote(str(BIN))} "$@"\n',
            encoding="utf-8",
        )
        stub.chmod(0o755)
        result = self.sweep(extra=("--release",), bin=stub)
        self.assertEqual(result.returncode, 1, result.stderr)
        report = json.loads((self.out / "sweep.json").read_text(encoding="utf-8"))
        rows = {row["id"]: row for row in report["specs"]}
        self.assertEqual(rows["001-green"]["outcome"], "passed")
        self.assertEqual(rows["001-green"]["lifecycle"],
                         {"status": None, "implementation": None, "class": "unknown"})
        self.assertEqual(rows["001-green"]["releaseOutcome"], "not-run")
        self.assertEqual(rows["002-green"]["releaseOutcome"], "passed")

    # --- spec 119: the default run directory --------------------------------

    def test_the_default_run_directory_is_new_per_run_and_outside_tmp(self):
        self.fixture({"001-green": ["true"]}, {"001-green": ("approved", "complete")})
        self.commit()
        cache = self.root / "cache"
        env = dict(os.environ, XDG_CACHE_HOME=str(cache))
        first = self.sweep(out=False, env=env)
        second = self.sweep(out=False, env=env)
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(second.returncode, 0, second.stderr)
        runs = sorted((cache / "spec-spine/sweeps").iterdir())
        self.assertEqual(len(runs), 2, runs)
        short = checked(["git", "rev-parse", "--short", "HEAD"], self.repo)
        for run_dir in runs:
            self.assertTrue(run_dir.name.startswith(short + "-"), run_dir)
            # The second run cleared nothing of the first.
            self.assertTrue((run_dir / "sweep.json").is_file(), run_dir)
            self.assertFalse((run_dir / "tree").exists(), run_dir)
        # Without XDG_CACHE_HOME the default is under $HOME/.cache.
        home = self.root / "home"
        env = {k: v for k, v in os.environ.items() if k != "XDG_CACHE_HOME"}
        env["HOME"] = str(home)
        third = self.sweep(out=False, env=env)
        self.assertEqual(third.returncode, 0, third.stderr)
        self.assertEqual(len(list((home / ".cache/spec-spine/sweeps").iterdir())), 1)

    def test_an_explicit_out_under_the_purged_temp_tree_is_warned_on_macos(self):
        self.fixture({"001-green": ["true"]}, {"001-green": ("approved", "complete")})
        self.commit()
        result = self.sweep()
        self.assertEqual(result.returncode, 0, result.stderr)
        tmp = Path(os.environ.get("TMPDIR", "/tmp")).resolve()
        under_tmp = str(self.out.resolve()).startswith(str(tmp) + os.sep)
        warned = "which macOS purges of files older" in result.stderr
        self.assertEqual(warned, platform.system() == "Darwin" and under_tmp, result.stderr)


    # --- spec 121: the Acceptance workflow's report step ---------------------

    def render(self, code, report=None):
        # The report step exactly as the workflow calls it, with the summary
        # sent to a file. Returns (exit, annotations by level, summary text).
        summary = self.root / "summary.md"
        if summary.exists():
            summary.unlink()
        report = report or self.out / "sweep.json"
        result = run(["python3", REPORTER, "--report", report, "--sweep-exit", str(code),
                      "--summary", summary])
        notes = {"error": [], "warning": [], "notice": []}
        for line in result.stdout.splitlines():
            for level in notes:
                if line.startswith(f"::{level} "):
                    notes[level].append(line)
        text = summary.read_text(encoding="utf-8") if summary.exists() else ""
        return result, notes, text

    def test_a_pending_draft_conceals_no_implemented_failure_and_no_unreadable_result(self):
        self.fixture(
            {
                "001-pending-red": ["false"],
                "002-built-red": ["false"],
                "003-ghost-built": ["true"],
                "004-built-green": ["true"],
            },
            {
                "001-pending-red": ("draft", "pending"),
                "002-built-red": ("approved", "complete"),
                "003-ghost-built": ("approved", "complete"),
                "004-built-green": ("approved", "complete"),
            },
        )
        # The registry keeps 003's member; its document, and so its plan, is gone.
        (self.repo / "specs/003-ghost-built/spec.md").unlink()
        self.commit()
        result = self.sweep(extra=("--release",))
        self.assertEqual(result.returncode, 1, result.stderr)
        rendered, notes, text = self.render(result.returncode)
        self.assertEqual(rendered.returncode, 0, rendered.stdout + rendered.stderr)
        errors = "\n".join(notes["error"])
        self.assertEqual(len(notes["error"]), 2, notes)
        self.assertIn("acceptance 002-built-red::failed for the release", errors)
        self.assertIn("acceptance 003-ghost-built::not-run for the release", errors)
        self.assertEqual(len(notes["warning"]), 1, notes)
        self.assertIn("pending acceptance 001-pending-red::block failed", notes["warning"][0])
        self.assertIn("**Release verdict: NOT CLEAN**", text)
        self.assertIn("Corpus verdict (raw, every selected block): **NOT CLEAN**", text)
        # The sweep's own report follows, unmodified.
        self.assertIn((self.out / "sweep.md").read_text(encoding="utf-8"), text)

    def test_a_pending_draft_alone_leaves_the_release_clean_and_the_corpus_visible(self):
        self.fixture(
            {"001-pending-red": ["false"], "002-built-green": ["true"],
             "003-pending-green": ["true"]},
            {"001-pending-red": ("draft", "pending"),
             "002-built-green": ("approved", "complete"),
             "003-pending-green": ("draft", "in-progress")},
        )
        self.commit()
        result = self.sweep(extra=("--release",))
        self.assertEqual(result.returncode, 0, result.stderr)
        report, _ = self.release_report()
        self.assertEqual(report["verdicts"], {"corpus": "not-clean", "release": "clean"})
        rendered, notes, text = self.render(result.returncode)
        self.assertEqual(rendered.returncode, 0, rendered.stdout + rendered.stderr)
        self.assertEqual(notes["error"], [])
        self.assertEqual(len(notes["warning"]), 1, notes)
        self.assertIn("001-pending-red", notes["warning"][0])
        # A fail-first block passing before its build is a finding, not silence.
        self.assertEqual(len(notes["notice"]), 1, notes)
        self.assertIn("003-pending-green", notes["notice"][0])
        self.assertIn("**Release verdict: CLEAN**", text)
        self.assertIn("Corpus verdict (raw, every selected block): **NOT CLEAN**", text)
        self.assertIn("failed 1", text)

    def test_the_report_step_refuses_evidence_it_cannot_vouch_for(self):
        self.fixture({"001-built-green": ["true"], "002-pending-red": ["false"]},
                     {"001-built-green": ("approved", "complete")})
        self.commit()
        # A corpus-mode report: the workflow must have asked for --release.
        corpus = self.sweep()
        self.assertEqual(corpus.returncode, 1, corpus.stderr)
        rendered, notes, _ = self.render(1)
        self.assertEqual(rendered.returncode, 1)
        self.assertIn("not 'release'", notes["error"][0])
        release = self.sweep(extra=("--release",))
        self.assertEqual(release.returncode, 0, release.stderr)
        # Agreement passes; both directions of disagreement refuse.
        self.assertEqual(self.render(0)[0].returncode, 0)
        rendered, notes, _ = self.render(1)
        self.assertEqual(rendered.returncode, 1)
        self.assertIn("exited 1 but its report's release verdict is 'clean'", notes["error"][0])
        # No exit code recorded, a refusal and an interruption, report or not.
        for code, words in (("", "recorded no exit code"), ("3", "refused (exit 3)"),
                            ("130", "was interrupted (exit 130)")):
            rendered, notes, text = self.render(code)
            self.assertEqual(rendered.returncode, 1, code)
            self.assertIn(words, notes["error"][0])
            self.assertIn("no verdict this run can vouch for", text)
        # Missing, unparseable, and older than 1.1.0.
        rendered, notes, _ = self.render(0, self.root / "absent.json")
        self.assertEqual(rendered.returncode, 1)
        self.assertIn("wrote no report", notes["error"][0])
        garbled = self.root / "garbled.json"
        garbled.write_text("{not json", encoding="utf-8")
        rendered, notes, _ = self.render(0, garbled)
        self.assertEqual(rendered.returncode, 1)
        self.assertIn("unreadable", notes["error"][0])
        old = json.loads((self.out / "sweep.json").read_text(encoding="utf-8"))
        for key in ("mode", "verdicts", "releaseCounts"):
            old.pop(key)
        old["schemaVersion"] = "1.0.0"
        legacy = self.root / "legacy.json"
        legacy.write_text(json.dumps(old), encoding="utf-8")
        rendered, notes, _ = self.render(0, legacy)
        self.assertEqual(rendered.returncode, 1)
        self.assertIn("(1.0.0) is not a 1.x report carrying both verdicts", notes["error"][0])
        # And the other direction of disagreement, on a real not-clean report.
        self.fixture_reset()
        self.fixture({"001-built-red": ["false"]}, {"001-built-red": ("approved", "complete")})
        self.commit()
        red = self.sweep(extra=("--release",))
        self.assertEqual(red.returncode, 1, red.stderr)
        # The report read below is this second sweep's, not the first's.
        second = json.loads((self.out / "sweep.json").read_text(encoding="utf-8"))
        self.assertEqual(second["revision"], self.sha)
        self.assertEqual([row["id"] for row in second["specs"]], ["001-built-red"])
        rendered, notes, _ = self.render(0)
        self.assertEqual(rendered.returncode, 1)
        self.assertIn("exited 0 but its report's release verdict is 'not-clean'", notes["error"][0])

    def fixture_reset(self):
        # A second fixture in the same test: the repository is rebuilt from
        # scratch at the same path, and the next sweep clears and rewrites
        # self.out (spec 089 3.6), which the caller then checks.
        shutil.rmtree(self.repo)


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
