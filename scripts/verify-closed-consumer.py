#!/usr/bin/env python3
# Spec: specs/118-the-verdict-is-the-only-thing-on-stdout/spec.md
"""Run a command under a stderr consumer that closes, within a deadline.

Spec 118 D-4's acceptance lines need a consumer that reads the opening
transcript line and then closes the parent's stderr, and spec 118 D-6's need a
watchdog whose cleanup actually reaches the whole process tree.  Both were
written inline, three times over, in `## Verification`; this is the one copy,
because a supervision bug written three times is fixed three times or not at
all.

What it does, in order: spawn `cmd` in a process group of its own, arm the
watchdog *before* any read, take the first line of the child's stderr, close
that stream, read stdout to end-of-file on its own thread, reap the leader, and
check what was asked for.  Every check is an assertion, so the exit code is the
verdict.

The supervision contract
------------------------

**Termination authority belongs to the watchdog.**  Nothing else signals, and
nothing else reaps.

**Nothing signals a process group whose leader has been reaped.**  While the
leader is unreaped its pid cannot be recycled, so the negative pgid names this
tree and no other.  `Tree` holds that by putting the signal and the reap under
one lock and publishing the status inside the same critical section as the
`wait` that produced it, so no signal can follow a reap.

This replaces a `p.poll() is None and os.killpg(...)` guard, which reads like
the same invariant and is not.  `poll()` *reaps*: if the leader has exited while
a descendant still holds the pipes, the guard reaps the leader, concludes the
tree is gone, and skips the group signal, leaving the descendant running and the
reader blocked on a pipe it will never see closed.  Measured with a 0.2 s
deadline against a fixture whose descendant sleeps 2 s: the run returned after
2.01 s and the descendant's delayed side effect was present.

For the same reason `communicate()` is not used: it reaps implicitly, on a
thread the lock does not cover.

Exit codes: 0 every check passed, 1 a check failed, 2 usage.
"""

import argparse
import atexit
import json
import os
import signal
import subprocess
import sys
import threading
import time

POLL = 0.05


class Tree:
    """Termination authority over one process group.

    `kill_group` and `reap` take the same lock, and `reap` publishes the status
    inside the critical section that produced it, so a signal can never follow a
    reap.  `reap` holds the lock only for one bounded `wait` at a time and
    releases it between attempts, so a watchdog on another thread can always
    fire while a reap is in progress.
    """

    def __init__(self, proc):
        self._proc = proc
        self._lock = threading.Lock()
        self._status = None

    def kill_group(self):
        """Signal the whole tree.  True iff the tree is now terminated.

        The two errors are not the same answer.  `ProcessLookupError` means
        there was nothing left in the group to signal, so the postcondition
        already holds; `PermissionError` means the signal was *refused*, and
        returning True there would claim a termination that did not happen.
        """
        with self._lock:
            if self._status is not None:
                return False
            try:
                os.killpg(self._proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            except PermissionError:
                return False
            return True

    def reap(self, timeout=None):
        """Reap the leader and return its status, or None if `timeout` ran out."""
        deadline = None if timeout is None else time.monotonic() + timeout
        while True:
            with self._lock:
                if self._status is not None:
                    return self._status
                try:
                    self._status = self._proc.wait(timeout=POLL)
                    return self._status
                except subprocess.TimeoutExpired:
                    pass
            if deadline is not None and time.monotonic() >= deadline:
                return None

    def terminate(self):
        """Cleanup for any path out of this script, `atexit` included."""
        self.kill_group()
        self.reap(timeout=10)


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--deadline", type=float, required=True,
                    help="seconds before the watchdog terminates the tree")
    ap.add_argument("--transcript-prefix",
                    help="the first stderr line must start with this")
    ap.add_argument("--no-transcript", action="store_true",
                    help="expect no first stderr line at all (end-of-file)")
    ap.add_argument("--expect-returncode", type=int, required=True)
    ap.add_argument("--expect-outcome",
                    help="parse stdout as one verdict envelope and check report.outcome")
    ap.add_argument("--max-elapsed", type=float,
                    help="fail if the whole run took longer than this")
    ap.add_argument("cmd", nargs=argparse.REMAINDER)
    args = ap.parse_args(argv)

    cmd = args.cmd[1:] if args.cmd[:1] == ["--"] else args.cmd
    if not cmd:
        ap.error("a command is required after --")
    # `is not None`, not truthiness: `bool("")` is False, so an explicit
    # `--transcript-prefix ""` would read as an absent flag, slip past an
    # exactly-one guard written with `bool(...)`, and then assert nothing at all
    # because every byte string starts with `b""`.
    if (args.transcript_prefix is not None) == args.no_transcript:
        ap.error("exactly one of --transcript-prefix and --no-transcript")
    if args.transcript_prefix == "":
        ap.error("--transcript-prefix must not be empty: every line starts with it")

    started = time.monotonic()
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            stdin=subprocess.DEVNULL, start_new_session=True)
    tree = Tree(proc)
    atexit.register(tree.terminate)

    # Armed before any read, so start-up is inside the bound too: a fixture that
    # never writes a transcript line must be given up at the deadline, not
    # waited on for as long as it lives.
    watchdog = threading.Timer(args.deadline, tree.kill_group)
    watchdog.daemon = True   # a failing assertion is not made to wait it out
    watchdog.start()

    # Started before the transcript is awaited: a fixture that floods stdout
    # while the harness is still waiting would otherwise fill that pipe, block,
    # and be reported as a missing transcript.
    collected = {}
    reader = threading.Thread(target=lambda: collected.setdefault("out", proc.stdout.read()))
    reader.daemon = True
    reader.start()

    first = proc.stderr.readline()
    proc.stderr.close()
    # `join` returns only on end-of-file on the child's stdout, which needs
    # every member of the tree to have released that pipe.  So by the time
    # `reap` is reached the leader has either exited or been killed by the
    # watchdog, and the unbounded `reap` below cannot be the thing that hangs.
    reader.join()
    status = tree.reap()
    watchdog.cancel()
    elapsed = time.monotonic() - started

    if args.no_transcript:
        assert first == b"", "expected no transcript line, got %r" % first
    else:
        want = args.transcript_prefix.encode()
        assert first.startswith(want), "expected a line starting %r, got %r" % (want, first)
    assert status == args.expect_returncode, \
        "expected returncode %s, got %s" % (args.expect_returncode, status)
    if args.expect_outcome is not None:
        document = json.loads(collected.get("out", b""))
        got = document["report"]["outcome"]
        assert got == args.expect_outcome, "expected outcome %r, got %r in %s" % (
            args.expect_outcome, got, document)
    if args.max_elapsed is not None:
        assert elapsed < args.max_elapsed, \
            "the run took %.2fs, past the %.2fs bound" % (elapsed, args.max_elapsed)

    print("ok: returncode=%s elapsed=%.2fs" % (status, elapsed))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv[1:]))
    except AssertionError as failure:
        print("FAIL: %s" % failure, file=sys.stderr)
        sys.exit(1)
