# Bindings plan (napi / pyo3 / cgo)

> **Design only: no binding code exists in this repo, by mandate (spec 115).** This
> document describes how npm, Python, and Go bindings will wrap `spec-spine-core`
> later. The library is built to make this a thin, mechanical exercise: every
> operation already has a `&str → Result<String, Error>` facade
> (see [api.md](api.md) §7), which is the single seam every binding wraps.

## Why the JSON facade is the seam (not the typed API)

The typed Rust API is ergonomic for Rust callers but crosses an FFI boundary
poorly: owned structs, enums, and `Result` do not map cleanly to a C ABI or a
JS/Python object without per-type marshalling. The JSON facade collapses the
entire surface to one signature:

```rust
fn op(input_json: &str) -> Result<String, Error>
```

A binding only needs to: (1) pass a UTF-8 string in, (2) get a UTF-8 string (or
an error) out, (3) wrap that in the host language's idiom. No per-DTO
marshalling, no lifetime/generic/trait-object handling; the boundary is already
FFI-friendly *by construction* (no lifetimes, generics, or trait objects;
`unsafe_code = "forbid"`; a single `Error` enum).

The facade functions to wrap:

| Facade fn | Wraps |
|---|---|
| `compile_json(config_json, repo_root)` | `compile` |
| `index_json(config_json, repo_root)` | `index` |
| `lint_json(config_json, repo_root)` | `lint` |
| `check_freshness_json(config_json, repo_root)` | `check_index_freshness` |
| `couple_json(request_json)` | `couple` |
| `query_json(request_json)` | `list` / `list_ids` / `show` / `status_report` / `relationships` |
| `render_json(config_json, index_json)` | `render_markdown` (spec 010) |
| `orphans_json(index_json)` | `orphans` (spec 010) |
| `load_config_json(toml_src)` | `load_config` |
| `scaffold_init_json(config_json)` | `scaffold_init` |

## The uniform envelope

In Rust the facade returns `Result<String, Error>`. The binding layer maps that
to one envelope shape across all languages, so callers handle success and
failure uniformly. Spec 132 fixes that shape as the same envelope `--json`
already writes on the CLI, rather than a separate `{ok, data, error}` binding
convention: a consumer reading documents from both the CLI and a binding
branches on one header, not two.

```jsonc
// success
{ "schemaVersion": "1.0.0", "tool": "spec-spine", "verb": "compile.check",
  "outcome": "ok", "exitCode": 0, "summary": "compile.check: ok",
  "report": <the facade's returned JSON, parsed> }
// failure
{ "schemaVersion": "1.0.0", "tool": "spec-spine", "verb": "compile.check",
  "outcome": "refused", "exitCode": 2, "summary": "config error: …",
  "error": { "kind": "validation" | "stale" | "not-found" | "drift"
                   | "refused" | "config" | "io" | "schema" | "usage" | "internal",
             "message": "…" } }
```

`outcome` is derived from `exitCode` and never passed separately, so the two
cannot disagree (`0` ok, `1` finding, `2` refused, `3` usage, `4` failed).
`error.kind` is the `Error` variant's stable token (`crate::error_kind`), a
closed set; a binding should treat an unknown `kind` as a generic failure,
since the set can only grow under a MAJOR bump to the envelope schema. This is
the only mapping logic a binding must implement; it is identical across
napi/pyo3/cgo. Before spec 132, the sketch here was a `{ok, data, error}`
shape with `error.code` the bare variant name and `exitCode` one of `1 | 2 |
3`; nothing built against that sketch, so there is no compatibility surface to
carry forward.

---

## napi-rs (Node / npm): sketch (do not build yet)

A `spec-spine-napi` crate using [`napi-rs`](https://napi.rs):

```rust
// crates/spec-spine-napi/src/lib.rs  (illustrative only, not in this repo)
#[napi]
pub fn compile(config_json: String, repo_root: String) -> napi::Result<String> {
    spec_spine_core::compile_json(&config_json, &repo_root).map_err(to_napi_err)
}
// … one #[napi] fn per facade fn; to_napi_err builds the envelope's `error` member.
```

- Ships as a prebuilt `.node` per platform via napi-rs's GitHub Actions matrix:
  the same triple matrix as the binary release.
- The published npm package wraps each export to parse the returned JSON and
  expose idiomatic JS (`await specSpine.compile(config, repoRoot)` returning
  the parsed `report`, throwing a typed `SpecSpineError` carrying
  `outcome`/`exitCode`/`error.kind`).

## pyo3 (Python): sketch

A `spec-spine-py` crate using [`pyo3`](https://pyo3.rs) +
[`maturin`](https://maturin.rs):

```rust
// illustrative only
#[pyfunction]
fn compile(config_json: &str, repo_root: &str) -> PyResult<String> {
    spec_spine_core::compile_json(config_json, repo_root).map_err(to_py_err)
}
#[pymodule]
fn spec_spine(m: &Bound<PyModule>) -> PyResult<()> { m.add_function(wrap_pyfunction!(compile, m)?)? /* … */ }
```

- `maturin` builds wheels per platform (the same triple matrix); publishes to
  PyPI.
- A thin Python layer parses the JSON and raises `SpecSpineError(kind,
  exit_code, message)` when `exitCode != 0`, returning `dict`/dataclasses (the
  `report` member) on success.

## cgo (Go): sketch

A `cdylib`/`staticlib` crate exposing a C ABI, consumed from Go via cgo:

```rust
// illustrative only: the one place `unsafe`/`extern "C"` is permitted (a binding crate, not core)
#[no_mangle]
pub extern "C" fn spec_spine_compile(config_json: *const c_char, repo_root: *const c_char) -> *mut c_char { … }
#[no_mangle]
pub extern "C" fn spec_spine_string_free(p: *mut c_char) { … }   // caller frees returned strings
```

- The C header is generated (`cbindgen`); the Go package wraps each `extern "C"`
  fn, marshals strings across cgo, and frees them via `spec_spine_string_free`.
- Go callers get `func Compile(configJSON, repoRoot string) (Registry, error)`,
  decoding the envelope and returning a typed error whenever `exitCode != 0`.

---

## Design rules these bindings rely on (already guaranteed by the core)

- **Pure functions of `(config, file bytes)`**, no ambient clock/env, so a
  binding can call from any host without surprise side effects (the sole
  wall-clock value, `build-meta.json.builtAt`, is written by the CLI, not the
  facade).
- **Owned, `serde`-serializable DTOs**: everything crossing the boundary is
  already JSON-representable.
- **A single, stable `Error` enum → stable exit codes** (spec 132's family
  contract): the envelope's `outcome`/`exitCode`/`error.kind` are a direct,
  stable projection.
- **No `unsafe` in core**: the only `extern "C"`/`unsafe` lives in the (future)
  cgo binding crate, never in `spec-spine-core`.
- **`publish = false` is set on none of the shipped crates**: bindings can
  depend on the published `spec-spine-core` from crates.io.

## Repository shape when bindings land (future)

```
crates/
├─ spec-spine-types/   (published)
├─ spec-spine-core/    (published; bindings depend on this)
├─ spec-spine-cli/     (published)
├─ spec-spine-napi/    (future; napi-rs → npm)
├─ spec-spine-py/      (future; pyo3/maturin → PyPI)
└─ spec-spine-ffi/     (future; cdylib + cbindgen → Go/C/others)
```

Each binding crate is a thin shell over the facade; the engine and its
guarantees stay in `spec-spine-core`. **Nothing in this list is built yet.**
