#!/usr/bin/env python3
"""Generate the spec 103 verifier fixture set.

Documented generator (spec 103 3.9). Run from the repository root with a built
binary; it writes crates/spec-spine-core/fixtures/verifier/ and nothing else.
"""
import io, json, os, shutil, subprocess, sys, tempfile, hashlib

ROOT = os.getcwd()
BIN = os.path.join(ROOT, "target/release/spec-spine")
OUT = os.path.join(ROOT, "crates/spec-spine-core/fixtures/verifier")

CORPUS_SPEC = """---
id: "000-bootstrap"
title: "Bootstrap"
status: approved
created: "2026-01-01"
summary: "The one-spec corpus a verifier fixture recomputes against."
---

# 000: Bootstrap

## 1. Purpose

A minimal corpus so a fixture can be reproduced from its own directory.
"""

def sh(args, cwd):
    r = subprocess.run(args, cwd=cwd, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"{args} exited {r.returncode}: {r.stderr}")
    return r.stdout

def build_producer():
    tmp = tempfile.mkdtemp(prefix="spec103-")
    os.makedirs(os.path.join(tmp, "specs/000-bootstrap"))
    io.open(os.path.join(tmp, "specs/000-bootstrap/spec.md"), "w").write(CORPUS_SPEC)
    sh([BIN, "compile"], tmp)
    sh([BIN, "attest"], tmp)
    payload = io.open(os.path.join(tmp, ".derived/attestation/attestation.json"), "rb").read()
    return tmp, payload

def write_case(cid, payload_bytes, case, corpus_src=None):
    d = os.path.join(OUT, cid)
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    io.open(os.path.join(d, "payload.json"), "wb").write(payload_bytes)
    io.open(os.path.join(d, "case.json"), "w").write(
        json.dumps(case, indent=2, sort_keys=True) + "\n"
    )
    if corpus_src:
        os.makedirs(os.path.join(d, "corpus/specs/000-bootstrap"))
        io.open(os.path.join(d, "corpus/specs/000-bootstrap/spec.md"), "w").write(CORPUS_SPEC)

def reserialize(obj):
    # A mutated payload is re-serialized after the change; 3.2 permits that and
    # forbids only calling the result producer output.
    return (json.dumps(obj, indent=2, sort_keys=True) + "\n").encode()

def main():
    tmp, producer = build_producer()
    obj = json.loads(producer)
    tool_version = obj["tool"]["version"]
    attestation_hash = hashlib.sha256(producer).hexdigest()
    subject = {"kind": "corpus", "attestationHash": f"sha256:{attestation_hash}"}
    none_subject = {"kind": "none"}
    PT = "spec-spine/corpus-attestation"
    schema = obj["schemaVersion"]

    def base(cid, kind, reason, exit_code, needs_corpus, **extra):
        c = {
            "schemaVersion": "0.1.0",
            "id": cid,
            "payloadType": PT,
            "payloadSchemaVersion": extra.pop("payloadSchemaVersion", schema),
            "bytes": kind,
            "subject": extra.pop("subject", subject),
            "needsCorpus": needs_corpus,
            "toolVersion": extra.pop("toolVersion", tool_version),
            "expect": {"outcome": "refused" if reason else "accepted",
                       "exit": exit_code},
        }
        if reason:
            c["expect"]["reason"] = reason
        c.update(extra)
        return c

    # --- the positive control: producer bytes, verbatim -----------------------
    write_case("control-untampered", producer,
               dict(base("control-untampered", "producer", None, 0, True),
                    producedBy="spec-spine attest, over this case's corpus/"),
               corpus_src=True)

    M = lambda cid, mutation, **kw: dict(
        base(cid, "mutated", kw.pop("reason"), kw.pop("exit"), kw.pop("needs", False), **kw),
        derivedFrom="control-untampered", mutation=mutation)

    d = json.loads(producer); d["prCouple"] = 1
    write_case("unknown-member-top", reserialize(d),
               M("unknown-member-top", "added top-level member `prCouple`",
                 reason="unknown-member", exit=3))

    d = json.loads(producer); d["tool"]["extra"] = 1
    write_case("unknown-member-nested", reserialize(d),
               M("unknown-member-nested", "added member `extra` to the nested `tool` object",
                 reason="unknown-member", exit=3))

    # Reformatted: identical values, different whitespace. Needs the corpus,
    # because the byte comparison happens only after a value match.
    write_case("reformatted-same-values",
               (json.dumps(json.loads(producer), indent=4, sort_keys=True) + "\n").encode(),
               M("reformatted-same-values", "re-indented from 2 spaces to 4; every value identical",
                 reason="non-canonical-bytes", exit=1, needs=True),
               corpus_src=True)

    d = json.loads(producer); d["schemaVersion"] = "9.0.0"
    write_case("unsupported-major", reserialize(d),
               M("unsupported-major", "schemaVersion MAJOR raised to 9.0.0",
                 reason="unsupported-major", exit=3, payloadSchemaVersion="9.0.0"))

    d = json.loads(producer); d["schemaVersion"] = "0.9.0"
    write_case("minor-ahead-content-mismatch", reserialize(d),
               M("minor-ahead-content-mismatch", "schemaVersion MINOR raised to 0.9.0, MAJOR unchanged",
                 reason="content-mismatch", exit=1, needs=True, payloadSchemaVersion="0.9.0"),
               corpus_src=True)

    # Duplicate key: authored, because no JSON serializer emits one.
    dup = producer.decode().replace('"schemaVersion"', '"schemaVersion": "0.1.0",\n  "schemaVersion"', 1)
    write_case("duplicate-key", dup.encode(),
               dict(base("duplicate-key", "authored", "duplicate-key", 3, False),
                    authoredBecause="no JSON serializer emits a duplicate key, so this cannot be "
                                    "produced by mutating through a JSON library"))

    d = json.loads(producer); d["tool"]["version"] = "0.0.1-fixture"
    write_case("tool-version-changed", reserialize(d),
               M("tool-version-changed", "tool.version set to 0.0.1-fixture",
                 reason="version-mismatch", exit=1, needs=True,
                 toolVersion="0.0.1-fixture"),
               corpus_src=True)

    d = json.loads(producer)
    d["verdicts"]["lint"]["ok"] = not d["verdicts"]["lint"]["ok"]
    write_case("flipped-verdict", reserialize(d),
               M("flipped-verdict", "verdicts.lint.ok negated",
                 reason="content-mismatch", exit=1, needs=True),
               corpus_src=True)

    d = json.loads(producer); del d["schemaVersion"]
    write_case("missing-schema-version", reserialize(d),
               M("missing-schema-version", "the schemaVersion member removed",
                 reason="missing-schema-version", exit=3,
                 payloadSchemaVersion="(absent)"))

    write_case("unreadable-json", b"{ this is not json\n",
               dict(base("unreadable-json", "authored", "unreadable-json", 3, False,
                         subject=none_subject, payloadSchemaVersion="(absent)"),
                    authoredBecause="attest cannot emit invalid JSON, so this payload has no "
                                    "producer form to derive from"))

    cases = sorted(n for n in os.listdir(OUT) if os.path.isdir(os.path.join(OUT, n)))
    index = {
        "schemaVersion": "0.1.0",
        "payloadTypes": ["spec-spine/corpus-attestation", "spec-spine/spec-attestation"],
        "reasons": ["unreadable-json", "missing-schema-version", "unsupported-major",
                    "unknown-member", "duplicate-key", "non-canonical-bytes",
                    "content-mismatch", "version-mismatch"],
        "cases": cases,
    }
    io.open(os.path.join(OUT, "index.json"), "w").write(
        json.dumps(index, indent=2, sort_keys=True) + "\n")
    shutil.rmtree(tmp, ignore_errors=True)
    print(f"wrote {len(cases)} case(s) to {OUT}")

main()
