//! The spec frontmatter grammar: the typed `Frontmatter` struct, the value
//! enums, the `extra_frontmatter` overflow, and the parse entry points.
//!
//! Parsing is pure. The `---`-delimited block is split out
//! ([`split_frontmatter`]), the known keys are deserialized into [`Frontmatter`],
//! and every key not in [`KNOWN_KEYS`] overflows into `extra_frontmatter` as a
//! `serde_json::Value`. The value domain splits on declaration (spec 012):
//! a key listed in `config.frontmatter.extra_known_keys` (passed to
//! [`parse_frontmatter_with`]) carries **any JSON-representable YAML value**,
//! transported verbatim under canonical-JSON normalization; an undeclared key
//! keeps the original scalar / string-list restriction (the anti-bulk-YAML
//! guard, ported from OAP/aide `spec-types`). [`parse_frontmatter`] is the
//! declared-nothing form, byte-compatible with pre-013 behavior.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::edges::{
    CoAuthorityItem, ConstrainItem, ExtendItem, Origin, ReferenceItem, RefineItem, SupersedeItem,
};
use crate::error::{Error, Result};
use crate::impact::{Conflict, Impact};
use crate::intent::IntentDeclaration;
use crate::interface::InterfaceReference;
use crate::moves::MoveDeclaration;
use crate::obligation::Obligation;
use crate::unit::Unit;
use crate::yaml::YamlValue;

/// Lifecycle status of a spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Draft,
    Approved,
    Superseded,
    Retired,
}

/// Risk level (optional descriptive metadata).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Low,
    Medium,
    High,
    Critical,
}

/// Implementation progress (optional descriptive metadata).
///
/// The canonical "not applicable" spelling is `n-a` (kebab-case, the family the
/// other variants share); `n/a` is accepted as a deserialize-only alias (spec
/// 014) for the predecessor dialect, and normalizes back to `n-a` on emission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Implementation {
    Pending,
    InProgress,
    Complete,
    #[serde(rename = "n-a", alias = "n/a")]
    Na,
    Deferred,
}

/// A failed frontmatter parse, classified for the compiler's V-code mapping
/// (spec 012 §3.3).
#[derive(Clone, Debug)]
pub enum FrontmatterIssue {
    /// Malformed YAML or a grammar violation: the V-002 class.
    Malformed(String),
    /// A DECLARED extra key whose value JSON cannot represent (non-string
    /// mapping key, YAML tag, non-finite number): the V-013 class.
    UnrepresentableDeclared { key: String, detail: String },
}

impl From<FrontmatterIssue> for Error {
    fn from(issue: FrontmatterIssue) -> Self {
        match issue {
            FrontmatterIssue::Malformed(m) => Error::Parse(m),
            FrontmatterIssue::UnrepresentableDeclared { key, detail } => Error::Parse(format!(
                "declared extra-frontmatter key '{key}' carries an unrepresentable YAML value: {detail}"
            )),
        }
    }
}

/// Every frontmatter key modeled as a struct field. Keys outside this set
/// overflow into `extra_frontmatter`.
pub const KNOWN_KEYS: &[&str] = &[
    // required + descriptive
    "id",
    "title",
    "status",
    "created",
    "summary",
    "authors",
    "owner",
    "kind",
    "domain",
    "risk",
    "implementation",
    "depends_on",
    "code_aliases",
    "feature_branch",
    // typed edges (8)
    "establishes",
    "extends",
    "refines",
    "supersedes",
    "amends",
    "co_authority",
    "constrains",
    "references",
    // lifecycle / amendment
    "superseded_by",
    "retirement_rationale",
    "amends_sections",
    // Spec 082 3.1: the amended specs whose `## Verification` block this
    // spec's own block replaces. A subset of `amends`.
    "amends_verification",
    "unamendable",
    "amendment_record",
    // bootstrap marker
    "origin",
    // Spec 106 3.1: declared constraints.
    "obligations",
    // Spec 109 3.1: declared impact and conflict against another spec's
    // obligations.
    "impacts",
    "conflicts",
    // Spec 110 3.1: cross-corpus interface references.
    "interface_references",
    // Spec 114 3.3: a standing goal and its deliberate exclusions.
    "intent",
    // Spec 111 3.1: declared move (relocation, split, merge, removal).
    "moves",
    // Spec 142 3.2: declared section relocations.
    "relocates",
    // Spec 169 3.1: declared obligation traceability.
    "traceability",
];

/// The typed, parsed frontmatter of a `spec.md`.
///
/// Field names are `snake_case` to match the authored YAML. Unknown keys are
/// **not** captured by serde (unknown fields are ignored on deserialize); they
/// are collected separately into `extra_frontmatter` by [`parse_frontmatter`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Frontmatter {
    // --- required ---
    pub id: String,
    pub title: String,
    pub status: Status,
    pub created: String,
    pub summary: String,

    // --- optional descriptive ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<Risk>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<Implementation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code_aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_branch: Option<String>,

    // --- typed edges (8) ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub establishes: Vec<Unit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extends: Vec<ExtendItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refines: Vec<RefineItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supersedes: Vec<SupersedeItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amends: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub co_authority: Vec<CoAuthorityItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constrains: Vec<ConstrainItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<ReferenceItem>,

    // --- lifecycle / amendment ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retirement_rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amends_sections: Vec<String>,
    /// Spec 082 3.1: amended specs whose `## Verification` block this spec's own
    /// block replaces, so `verify <amended-id>` runs this spec's commands.
    /// Every entry MUST also appear in `amends` (`V-018`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amends_verification: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unamendable: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment_record: Option<String>,

    // --- bootstrap marker ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,

    // --- declared constraints (spec 106) ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligations: Vec<Obligation>,

    // --- declared impact and conflict against another spec's obligations
    // (spec 109) ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impacts: Vec<Impact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<Conflict>,

    // --- cross-corpus interface references (spec 110) ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interface_references: Vec<InterfaceReference>,

    // --- declared intent (spec 114) ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<IntentDeclaration>,

    // --- declared moves (spec 111) ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub moves: Vec<MoveDeclaration>,

    // --- declared section relocations (spec 142) ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relocates: Vec<crate::Relocation>,

    // --- declared obligation traceability (spec 169) ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traceability: Vec<crate::TraceDeclaration>,

    // --- overflow (populated by parse_frontmatter, never by serde) ---
    #[serde(skip)]
    pub extra_frontmatter: BTreeMap<String, serde_json::Value>,
}

/// Split a `spec.md` source into its `(frontmatter_yaml, body)` halves.
///
/// Strips a leading UTF-8 BOM, requires the file to open with a `---` fence, and
/// reads to the next line that is exactly `---`. Line-ending agnostic (uses
/// [`str::lines`], which handles both `\n` and `\r\n`). Returns owned strings.
pub fn split_frontmatter(src: &str) -> Result<(String, String)> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let mut lines = src.lines();

    match lines.next() {
        Some(first) if first.trim_end() == "---" => {}
        _ => {
            return Err(Error::Parse(
                "spec.md must begin with a YAML frontmatter block delimited by '---'".into(),
            ));
        }
    }

    let mut frontmatter = String::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line.trim_end() == "---" {
            closed = true;
            break;
        }
        frontmatter.push_str(line);
        frontmatter.push('\n');
    }
    if !closed {
        return Err(Error::Parse(
            "unterminated frontmatter block (missing closing '---')".into(),
        ));
    }

    let mut body = String::new();
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }

    Ok((frontmatter, body))
}

/// Parse the frontmatter block of a `spec.md` into a typed [`Frontmatter`],
/// treating every extra key as undeclared (pre-013 behavior, kept for
/// config-free callers).
///
/// Returns [`Error::Parse`] for a malformed block, a missing required key, an
/// invalid enum value, or a non-scalar value under an unknown (overflow) key.
pub fn parse_frontmatter(src: &str) -> Result<Frontmatter> {
    parse_frontmatter_with(src, &[]).map_err(Into::into)
}

/// Parse with declared-key awareness (spec 012): a key listed in `declared`
/// (the adopter's `frontmatter.extra_known_keys`) carries any
/// JSON-representable YAML value, transported verbatim; an undeclared key
/// keeps the scalar / string-list restriction. A top-level `null` value drops
/// the key on either path.
pub fn parse_frontmatter_with(
    src: &str,
    declared: &[String],
) -> std::result::Result<Frontmatter, FrontmatterIssue> {
    let malformed = |m: String| FrontmatterIssue::Malformed(m);
    let (yaml, _body) = split_frontmatter(src).map_err(|e| {
        malformed(match e {
            Error::Parse(m) => m,
            other => other.to_string(),
        })
    })?;

    let value = crate::yaml::parse(&yaml)
        .map_err(|e| malformed(format!("invalid YAML frontmatter: {e}")))?;

    let mapping = value
        .as_mapping()
        .ok_or_else(|| malformed("frontmatter must be a YAML mapping".into()))?;

    // Known keys (unknown keys are ignored here; collected below).
    let mut frontmatter: Frontmatter = crate::yaml::from_value(&value)
        .map_err(|e| malformed(format!("invalid frontmatter: {e}")))?;

    // Overflow: every key not in KNOWN_KEYS becomes an extra_frontmatter entry.
    for (k, v) in mapping {
        let key = match k.as_str() {
            Some(s) => s,
            None => return Err(malformed("frontmatter keys must be strings".into())),
        };
        if KNOWN_KEYS.contains(&key) {
            continue;
        }
        let json = if declared.iter().any(|d| d == key) {
            yaml_to_json(v).map_err(|detail| FrontmatterIssue::UnrepresentableDeclared {
                key: key.to_string(),
                detail,
            })?
        } else {
            yaml_to_extra(v).map_err(malformed)?
        };
        if json.is_null() {
            continue;
        }
        frontmatter.extra_frontmatter.insert(key.to_string(), json);
    }

    // `paths:` sugar on extends/refines items (spec 013): expanded here, in
    // the shared parse path, so every consumer (compile, index, lint, couple)
    // sees only single-unit edges.
    frontmatter.extends =
        crate::edges::expand_extend_paths(std::mem::take(&mut frontmatter.extends))
            .map_err(malformed)?;
    frontmatter.refines =
        crate::edges::expand_refine_paths(std::mem::take(&mut frontmatter.refines))
            .map_err(malformed)?;
    // Full-scope supersedes (`{ scope: full }` / bare id) collapse to the
    // bare-string form so the wire stays byte-identical for full-only corpora
    // (spec 018).
    frontmatter.supersedes =
        crate::edges::normalize_supersedes(std::mem::take(&mut frontmatter.supersedes));

    Ok(frontmatter)
}

/// The UNDECLARED-key path: scalars and string lists only (`Null` drops the
/// key); a nested map or mixed list is a grammar violation, exactly the
/// pre-013 guard. (A tag never reaches here: the parser refuses it, spec 196.)
fn yaml_to_extra(v: &YamlValue) -> std::result::Result<serde_json::Value, String> {
    match v {
        YamlValue::Null => Ok(serde_json::Value::Null),
        YamlValue::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        YamlValue::Int(i) => Ok(serde_json::Value::from(*i)),
        // Past `i64`, the pre-196 reader fell back to a float; so does this.
        YamlValue::UInt(u) => match i64::try_from(*u) {
            Ok(i) => Ok(serde_json::Value::from(i)),
            Err(_) => serde_json::Number::from_f64(*u as f64)
                .map(serde_json::Value::Number)
                .ok_or_else(|| "unsupported numeric extra-frontmatter value".to_string()),
        },
        YamlValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| "unsupported numeric extra-frontmatter value".to_string()),
        YamlValue::String(s) => Ok(serde_json::Value::String(s.clone())),
        YamlValue::Sequence(seq) => {
            let mut list = Vec::with_capacity(seq.len());
            for item in seq {
                match item.as_str() {
                    Some(s) => list.push(serde_json::Value::String(s.to_string())),
                    None => {
                        return Err("extra-frontmatter lists must contain only strings".to_string());
                    }
                }
            }
            Ok(serde_json::Value::Array(list))
        }
        YamlValue::Mapping(_) => Err(
            "extra-frontmatter values must be scalars or string lists, not nested maps".to_string(),
        ),
    }
}

/// The DECLARED-key path (spec 012 §3.2): full YAML → JSON conversion.
/// Mappings require string keys and numbers must be finite. Map key order is
/// canonicalized by the sorted `serde_json::Map` (authoring order is not
/// preserved: the price of byte-identical registries).
fn yaml_to_json(v: &YamlValue) -> std::result::Result<serde_json::Value, String> {
    v.to_json().map_err(|e| {
        if e == "mapping keys must be strings" {
            "non-string mapping key is not JSON-representable".to_string()
        } else {
            e
        }
    })
}
