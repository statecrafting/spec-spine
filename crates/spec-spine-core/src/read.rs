//! The read-document emitter (spec 074).
//!
//! A read document is the JSON a verb emits when it answers a question rather
//! than rendering a verdict: `registry list`, `show`, `status-report`,
//! `relationships` and `plan` (with their projections), `index owner`,
//! `coverage`, `diagnostics` and `orphans`, and `config show`, plus the facade
//! functions that return the same answers. Before this module each call site
//! serialized with `serde_json::to_string_pretty`, so a document's key order was
//! whatever order its Rust struct declared, and none carried a version a
//! consumer could dispatch on.
//!
//! Every read document goes through [`read_document`]: sorted keys, canonical
//! layout, object form, and `schemaVersion` = [`READ_SCHEMA_VERSION`] unless the
//! caller says the document already names its own version. The verdict verbs
//! keep spec 034's envelope and do not come here.

use serde::Serialize;
use serde_json::Value;
use spec_spine_types::{Error, READ_SCHEMA_VERSION};

use crate::canonical_json;

/// How a read document is versioned (spec 074 §3.2). An argument, never
/// inferred from member names: an emitter that looked for a member called
/// `version` would silently exempt the first document that grew one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Versioning {
    /// Insert `schemaVersion` = [`READ_SCHEMA_VERSION`] at the top level.
    Stamp,
    /// Insert nothing: the document already carries a version its own spec
    /// declares, under the named member. `config show` (`config_version`,
    /// spec 047) is the only such document. The member must be present, so the
    /// exemption cannot be claimed by a document with nothing to exempt.
    Preexisting(&'static str),
}

/// The member the stamped version sits under.
pub const READ_VERSION_MEMBER: &str = "schemaVersion";

/// The member a top-level array is wrapped under (spec 074 §3.3).
pub const ITEMS_MEMBER: &str = "items";

/// Emit one read document (spec 074 §3.2, §3.3).
///
/// 1. serialize `value`;
/// 2. bring it to object form: an array is wrapped as `{ "items": [...] }` in
///    the same order; a top-level `null` or scalar is refused as an internal
///    error, since an absent answer is a member the **caller** names (for
///    example `{ "next": null }`) and no read emits a bare scalar;
/// 3. apply `mode`;
/// 4. write through the canonical writer: sorted keys at every depth, two-space
///    indent, LF, one trailing newline.
///
/// A pure function of `(value, mode)`: no clock, environment or filesystem.
pub fn read_document<T: Serialize + ?Sized>(value: &T, mode: Versioning) -> Result<String, Error> {
    let value = serde_json::to_value(value).map_err(|e| Error::Schema(e.to_string()))?;
    let mut object = match value {
        Value::Object(map) => map,
        Value::Array(items) => {
            let mut map = serde_json::Map::new();
            map.insert(ITEMS_MEMBER.to_string(), Value::Array(items));
            map
        }
        Value::Null => {
            return Err(Error::Schema(
                "internal: a read document cannot be a bare null; the caller names the \
                 member an absent answer sits under (spec 074 §3.3)"
                    .to_string(),
            ));
        }
        other => {
            return Err(Error::Schema(format!(
                "internal: a read document must be an object, got a bare {} (spec 074 §3.3)",
                scalar_kind(&other)
            )));
        }
    };
    match mode {
        Versioning::Stamp => {
            // A document that already names `schemaVersion` is not one to stamp:
            // overwriting would silently replace a version some other contract
            // put there. The caller must say `Preexisting` instead, and saying
            // the wrong thing is an internal error rather than a corrupted
            // document.
            if object.contains_key(READ_VERSION_MEMBER) {
                return Err(Error::Schema(format!(
                    "internal: a read document to be stamped already carries `{READ_VERSION_MEMBER}`; stamping would overwrite it (spec 074 §3.2)"
                )));
            }
            object.insert(
                READ_VERSION_MEMBER.to_string(),
                Value::String(READ_SCHEMA_VERSION.to_string()),
            );
        }
        Versioning::Preexisting(member) => {
            if !object.contains_key(member) {
                return Err(Error::Schema(format!(
                    "internal: a read document exempted from stamping must carry its own \
                     version member `{member}` (spec 074 §3.2)"
                )));
            }
        }
    }
    canonical_json::to_string(&Value::Object(object))
}

fn scalar_kind(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        _ => "value",
    }
}
