# Declared obligation traceability

Spec 169. An author can connect one obligation (spec 106) to the unit, test,
invariant, documentation, or interface that relates to it. spec-spine records
that connection and reports whether its target binds structurally. It does not
infer connections and does not treat one as proof.

## What a declaration says

A `traceability` entry in a spec's frontmatter records one authored relation:

```yaml
traceability:
  - id: "R-1-impl"
    obligation: "169-declared-obligation-traceability#R-1"
    relation: implemented-by
    target: { kind: unit, spec: "169-declared-obligation-traceability", unit: "crates/spec-spine-core/src/traceability.rs" }
```

The relation's identity is `<declaring-spec>#trace:<id>`. It never changes,
and once a spec carrying it is approved, the only compatible retirement is
`withdrawn: true` in place.

| Target kind | Members | Relation |
|---|---|---|
| `unit` | `spec`, `unit` (an authority unit, as the edge grammar accepts) | `implemented-by` |
| `test` | `selector` (a spec 155 test selector id) | `tested-by` |
| `invariant` | `obligation` (qualified, of kind `invariant`) | `enforced-by` |
| `documentation` | `selector` (a spec 155 selector of any kind but `test`) | `documented-by` |
| `interface` | `role`, `spec`, and exactly one of `unit` or `corpus` | `produced-by` (producer), `consumed-by` (consumer) |

A cross-corpus interface (`corpus` plus `spec`) must name exactly one of the
declaring spec's `interface_references`. It copies no digest and fetches
nothing; the reference already carries the pin.

Compile refuses a malformed entry: an unknown member, relation, or target kind
(`V-002`); a bad or repeated id, an unqualified source, a disallowed pairing,
or an interface whose role or form is wrong (`V-044`); a source obligation that
does not resolve, is ambiguous, or is withdrawn while the relation is live
(`V-045`); and the same normalized `(obligation, relation, target)` twice
(`V-046`). Every spec reference is normalized to its full id in the registry.

The member is optional. A corpus that declares none compiles exactly as
before. No tool warns because an obligation has no relations, and absence never
means unimplemented, untested, or undocumented.

## What the read answers

```text
spec-spine registry traceability
    [--declared-by <spec>]
    [--obligation <spec-id>#<obligation-id>]
    [--state <state>]
    [--json]
```

Each relation is reported with its identity, normalized source and target, the
canonical target identity, and one state:

| State | Meaning |
|---|---|
| `resolved` | The target binds to exactly one structural target. |
| `unresolved` | The form is supported, and nothing binds (a missing file, a planned unit not yet written, a unit the named spec does not claim, an obligation that is not an invariant). |
| `ambiguous` | The form binds more than one target (for example a symbol declared twice). |
| `unsupported` | The form is outside the resolver's matrix. Every `test` target is unsupported until a measured resolver spec adds a test kind to spec 155's matrix. |
| `unknown` | Resolution needs an input that is unavailable, such as a fresh committed codebase index for a unit target. |
| `withdrawn` | The relation, its source, or its target obligation is withdrawn. |

Every state but `resolved` carries a `detail` naming identities only. The read
exits `0` whatever the states are; filters intersect; an unqualified
`--obligation` or an unknown `--state` exits `3`, and an unknown spec exits
`1`. The JSON form is a read document (read axis `0.11.0`) that also carries
`traceabilityVersion` (`1.0.0`), the document's own axis. A reader accepts any
`1.x` and refuses another major, and it never downcasts an unknown state. A
new relation kind, target kind, or state is a new major, so a `1.x` reader never
meets one.

The library call is `traceability(&Config, repo_root, &Registry,
Option<&CodebaseIndex>, &TraceabilityRequest)`; the JSON facade is
`traceability_json(config_json, repo_root, request_json)`.

## What it never means

`resolved` means only that the declared target binds. It does not mean the code
ran, a test passed, evidence was admitted, an implementation was accepted, or
anything was released, adopted, or observed. Those are later stages owned by
other systems, and no state here is inferred from another.

Ownership is not a relation. A spec that claims a file is not thereby declared
to implement, test, or document anything. The resolver never infers a relation
from names, paths, imports, call graphs, proximity, or prose, and never
substitutes a nearby target.

The read executes nothing, fetches nothing, and writes nothing. Selector
targets are bound through spec 155's resolver and its containment rules;
selected content never appears in the output.
