// Spec: specs/114-authoring-adapters-and-intent-are-separated/spec.md
//! Intent (spec 114 §3.3): a spec's standing goal and its deliberate
//! exclusions, declared in frontmatter and carried into the registry.
//!
//! Two shapes, because the two sides of the compiler spell members
//! differently: frontmatter keys are snake_case (`non_goals`) and registry
//! members are camelCase (`nonGoals`). Each shape accepts exactly its own
//! spelling, so an author writing `nonGoals` is refused rather than silently
//! accepted in a second dialect.
//!
//! An intent is a declaration. Nothing here, and nothing that reads it, judges
//! whether it is true, achieved or wise (§3.4, §3.5).

use serde::{Deserialize, Serialize};

/// The authored `intent` key. `deny_unknown_fields`: a member other than
/// `goal` and `non_goals` (an attempt's `approach`, say) is malformed
/// frontmatter, never silently dropped (§3.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntentDeclaration {
    /// What the spec is for. Required, and non-empty after trimming (§3.4).
    pub goal: String,
    /// What a reader would reasonably expect and is deliberately excluded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_goals: Vec<String>,
}

/// The intent as the registry records it (§3.3): the strings exactly as
/// authored after YAML scalar parsing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Intent {
    pub goal: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_goals: Vec<String>,
}

impl From<IntentDeclaration> for Intent {
    fn from(d: IntentDeclaration) -> Self {
        Self {
            goal: d.goal,
            non_goals: d.non_goals,
        }
    }
}
