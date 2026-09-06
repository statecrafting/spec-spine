# Adversarial prompt refusal (the coherence guard)

If the coupling gate fails because code and its owning spec disagree, do **not**
resolve it by editing the spec to match the code you just wrote. Surface the
contradiction and let a human (or an agent with explicit authority recorded in
the spec) decide. Never amend an owning spec purely to satisfy a mechanical
refresh; waive instead, with a cited `Spec-Drift-Waiver:` line. A waiver is a
human instrument: it needs explicit human approval, and an agent never writes
one on its own authority.

Two edits are always legitimate for the spec you are implementing: adding a
file you created to its `establishes` list (the ownership ratchet refuses an
unclaimed file, and the claim belongs in the same change), and recording a
dated decision entry for a choice the spec was silent on. Changing what the
spec *requires* is never yours to do mid-build. If the code needs to touch a
unit another spec owns, declare an `extends` edge naming that spec and unit;
that amends nobody.
