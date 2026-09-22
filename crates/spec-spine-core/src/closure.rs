// Spec: specs/107-a-context-closure-is-declared/spec.md
//! ContextClosure (spec 107): a consumer's declared context for one piece of
//! work, resolved against the committed ledger and content-addressed.
//!
//! The closure itself lives in the consumer's record (spec 107 §3.1, spec
//! 092). This module only resolves one: [`resolve_closure`] is pure over its
//! arguments, and [`closure`] is the IO wrapper that checks freshness and reads
//! the committed ledger.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use spec_spine_types::{
    Config, Error, ObligationKind, REGISTRY_SCHEMA_VERSION, Registry, RegistrySpecShard,
    split_obligation_ref,
};

use crate::index::Freshness;
use crate::{canonical_json, compile, hash, shard, spec_id};

/// A section named by spec and anchor (spec 107 §3.2).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SectionRef {
    pub spec: String,
    pub anchor: String,
}

/// The request a consumer holds (spec 107 §3.1). Unknown members are refused:
/// a misspelt list would silently digest less than its author named.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClosureRequest {
    #[serde(default)]
    pub specs: Vec<String>,
    #[serde(default)]
    pub sections: Vec<SectionRef>,
    #[serde(default)]
    pub obligations: Vec<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

/// One resolved member (spec 107 §3.3), tagged by `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ClosureMember {
    #[serde(rename_all = "camelCase")]
    Spec { spec: String, content_hash: String },
    #[serde(rename_all = "camelCase")]
    Section {
        spec: String,
        anchor: String,
        digest: String,
    },
    #[serde(rename_all = "camelCase")]
    Obligation {
        spec: String,
        id: String,
        obligation_kind: ObligationKind,
        text: String,
        anchor: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        inputs: Vec<String>,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        withdrawn: bool,
        section_digest: String,
    },
}

impl ClosureMember {
    /// The digest piece name (spec 107 §3.4), which is also the member's
    /// identity: two members with one name are one member.
    fn piece_name(&self) -> String {
        match self {
            ClosureMember::Spec { spec, .. } => format!("spec:{spec}"),
            ClosureMember::Section { spec, anchor, .. } => format!("section:{spec}#{anchor}"),
            ClosureMember::Obligation { spec, id, .. } => format!("obligation:{spec}#{id}"),
        }
    }

    /// What the piece digests: the identity the member carries.
    fn piece_content(&self) -> Result<String, Error> {
        match self {
            ClosureMember::Spec { content_hash, .. } => Ok(content_hash.clone()),
            ClosureMember::Section { digest, .. } => Ok(digest.clone()),
            ClosureMember::Obligation { .. } => canonical_json::to_string(self),
        }
    }
}

/// A resolved closure (spec 107 §3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedClosure {
    pub digest: String,
    pub members: Vec<ClosureMember>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

/// Resolve a closure request against a registry and its specs' content hashes
/// (spec 107 §3.2 - 3.4). Pure: reads nothing but its arguments.
///
/// Refusals, in order: an empty request or an unqualified obligation reference
/// is [`Error::Parse`] (exit 3); every reference that does not resolve is
/// collected into one [`Error::NotFound`] (exit 1) naming each.
pub fn resolve_closure(
    registry: &Registry,
    content_hashes: &BTreeMap<String, String>,
    request: &ClosureRequest,
) -> Result<ResolvedClosure, Error> {
    if request.specs.is_empty() && request.sections.is_empty() && request.obligations.is_empty() {
        return Err(Error::Parse(
            "a closure names at least one spec, section or obligation".into(),
        ));
    }
    let mut obligation_refs = Vec::new();
    for reference in &request.obligations {
        let parts = split_obligation_ref(reference).ok_or_else(|| {
            Error::Parse(format!(
                "closure obligation '{reference}' is not qualified: a qualified form \
                 <spec-id>#<obligation-id> is required (spec 106)"
            ))
        })?;
        obligation_refs.push((reference.as_str(), parts));
    }

    let ids = || registry.specs.iter().map(|s| s.id.as_str());
    let record = |id: &str| registry.specs.iter().find(|s| s.id == id);
    let mut missing: Vec<String> = Vec::new();
    let mut members: BTreeMap<String, ClosureMember> = BTreeMap::new();
    let mut add = |m: ClosureMember| {
        members.insert(m.piece_name(), m);
    };

    for arg in &request.specs {
        match spec_id::resolve_spec_id(arg, ids()) {
            Ok(id) => match content_hashes.get(&id) {
                Some(h) => add(ClosureMember::Spec {
                    spec: id,
                    content_hash: h.clone(),
                }),
                None => missing.push(format!("spec '{id}' has no committed shard")),
            },
            Err(e) => missing.push(format!("spec '{arg}': {e}")),
        }
    }
    for s in &request.sections {
        match spec_id::resolve_spec_id(&s.spec, ids()) {
            Ok(id) => match record(&id).and_then(|r| r.section_digests.get(&s.anchor)) {
                Some(d) => add(ClosureMember::Section {
                    spec: id,
                    anchor: s.anchor.clone(),
                    digest: d.clone(),
                }),
                None => missing.push(format!("section '{id}#{}': no such heading", s.anchor)),
            },
            Err(e) => missing.push(format!("section spec '{}': {e}", s.spec)),
        }
    }
    for (reference, (spec_part, ob_id)) in obligation_refs {
        let id = match spec_id::resolve_spec_id(spec_part, ids()) {
            Ok(id) => id,
            Err(e) => {
                missing.push(format!("obligation '{reference}': {e}"));
                continue;
            }
        };
        let Some(rec) = record(&id) else {
            missing.push(format!("obligation '{reference}': no spec '{id}'"));
            continue;
        };
        let found = rec.obligations.iter().find(|o| o.id == ob_id);
        // Spec 106 R-4 makes an obligation's anchor resolve at compile, so its
        // digest is always present in a valid ledger. If it is not, the ledger
        // broke that invariant, and an empty digest would hide it.
        let digest = found.and_then(|ob| rec.section_digests.get(&ob.anchor).cloned());
        match (found, digest) {
            (Some(ob), None) => missing.push(format!(
                "obligation '{reference}': its anchor '{}' has no section digest in the ledger",
                ob.anchor
            )),
            (Some(ob), Some(section_digest)) => add(ClosureMember::Obligation {
                spec: id.clone(),
                id: ob.id.clone(),
                obligation_kind: ob.kind,
                text: ob.text.clone(),
                anchor: ob.anchor.clone(),
                inputs: ob.inputs.clone(),
                withdrawn: ob.withdrawn,
                section_digest,
            }),
            (None, _) => missing.push(format!(
                "obligation '{reference}': '{id}' declares no '{ob_id}'"
            )),
        }
    }

    if !missing.is_empty() {
        return Err(Error::NotFound(format!(
            "closure references that do not resolve: {}",
            missing.join("; ")
        )));
    }

    let mut pieces = Vec::with_capacity(members.len());
    for (name, member) in &members {
        pieces.push((name.clone(), member.piece_content()?));
    }
    // The member map is keyed by piece name, whose prefixes sort `obligation`,
    // `section`, `spec`: kind first, then identity, as §3.3 requires.
    Ok(ResolvedClosure {
        digest: hash::content_hash(pieces),
        members: members.into_values().collect(),
        rationale: request.rationale.clone(),
    })
}

/// Every committed registry shard's `shardHash`, by spec id: the spec's full
/// content hash, read and never recomputed (spec 048 §3.3).
pub fn committed_content_hashes(
    cfg: &Config,
    repo_root: &Path,
) -> Result<BTreeMap<String, String>, Error> {
    let dir = compile::registry_dir(cfg, repo_root).join(shard::BY_SPEC_DIR);
    let mut out = BTreeMap::new();
    for (name, bytes) in shard::read_shard_files(&dir)? {
        let sh: RegistrySpecShard = serde_json::from_slice(&bytes)
            .map_err(|e| Error::Parse(format!("invalid registry shard {name}: {e}")))?;
        shard::check_major("registry", &sh.spec_version, REGISTRY_SCHEMA_VERSION)?;
        out.insert(sh.record.id, sh.shard_hash);
    }
    Ok(out)
}

/// Resolve a closure against the committed ledger (spec 107 §3.5, 3.6). A
/// stale registry is refused with [`Error::Stale`] (exit 2) before anything is
/// digested.
pub fn closure(
    cfg: &Config,
    repo_root: &Path,
    request: &ClosureRequest,
) -> Result<ResolvedClosure, Error> {
    if let Freshness::Stale { expected, actual } =
        compile::check_registry_freshness(cfg, repo_root)?
    {
        return Err(Error::Stale { expected, actual });
    }
    let registry = compile::load_committed_registry(cfg, repo_root)?;
    let hashes = committed_content_hashes(cfg, repo_root)?;
    resolve_closure(&registry, &hashes, request)
}
