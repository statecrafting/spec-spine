//! `spec-spine edit-plan` (spec 165): a digest-guarded, authority-classified
//! proposal for a governed edit, bound to one verified Git tree. Nothing
//! here, or anywhere in spec-spine, applies a plan.

use std::path::Path;

use spec_spine_core::{edit_plan, edit_plan_document};
use spec_spine_types::{EditPlanRequest, Error, PlanStatus, verdict::Verdict, verdict::verb};

use crate::cmd_content::{bind_snapshot, confirm_unchanged};
use crate::out;

/// Plan the request against the clean working tree at `HEAD`, or one exported
/// revision. A ready plan is the bare document at exit 0; a refused plan is a
/// finding whose `edit-plan` envelope carries it, at exit 1.
pub fn run(
    repo: &Path,
    request_path: &Path,
    repository: &str,
    revision: Option<&str>,
) -> Result<u8, Error> {
    let text = std::fs::read_to_string(request_path).map_err(|e| {
        Error::Io(format!(
            "read edit-plan request {}: {e}",
            request_path.display()
        ))
    })?;
    let request: EditPlanRequest = serde_json::from_str(&text)
        .map_err(|e| Error::Usage(format!("invalid edit-plan request: {e}")))?;

    let bound = bind_snapshot(repo, repository, revision)?;
    let plan = edit_plan(&bound.config, bound.tree(), &request, &bound.snapshot)?;
    confirm_unchanged(repo, &bound.snapshot, revision)?;

    if plan.status == PlanStatus::Refused {
        let value = serde_json::to_value(&plan).map_err(|e| Error::Internal(e.to_string()))?;
        out::verdict(
            &Verdict::report(verb::EDIT_PLAN, 1, value).with_summary(format!(
                "edit-plan: refused; {} refusal(s)",
                plan.refusals.len()
            )),
        )?;
        return Ok(1);
    }
    out!("{}", edit_plan_document(&plan)?);
    Ok(0)
}
