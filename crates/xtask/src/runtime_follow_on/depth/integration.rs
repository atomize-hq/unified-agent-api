//! Integration authority uses one authenticated tip, with Source's git_inputs::command runner.
use crate::{
    agent_lifecycle::LifecycleStage,
    depth_record::{acquisition, DepthRecord},
    runtime_advertising::source::Source,
    support_matrix::depth::covered_targets,
};

/// Callers establish the same frozen approval generation before checking its progression.
pub(crate) fn check_progression(prior: LifecycleStage, next: LifecycleStage) -> Result<(), String> {
    if next < prior {
        return Err(format!(
            "onboarding lifecycle progression cannot move backward from `{}` to `{}` within a frozen generation",
            prior.as_str(), next.as_str()
        ));
    }
    Ok(())
}

/// Keep the statement reader's complete-directory validation and shared parser, while ensuring
/// both enumeration and blob reads use the same hardened view as bound-input comparisons.
pub(super) fn statement(
    tip: &Source,
    pack: &str,
    agent: &str,
    version: &str,
) -> Result<Option<acquisition::AcquisitionStatement>, String> {
    let directory = acquisition::statement_dir(pack);
    let mut found = None;
    for path in tip.committed_files(&directory)? {
        let name = path
            .strip_prefix(&format!("{directory}/"))
            .filter(|name| !name.contains('/'))
            .ok_or_else(|| {
                format!("{path}: acquisition statement must be a direct directory entry")
            })?;
        if tip.executable(&path)? {
            return Err(format!(
                "{path}: acquisition statement must have mode 100644"
            ));
        }
        let statement = acquisition::AcquisitionStatement::parse(&tip.read(&path)?, agent, name)?;
        if statement.version == version {
            found = Some(statement);
        }
    }
    Ok(found)
}

/// Charter rule 4 adopts the registry targets rule, including coverage already integrated for
/// this version. A record in the onboarding branch alone cannot grant target authority.
pub(super) fn targets(
    tip: &Source,
    agent: &str,
    root: &str,
    version: &str,
    declared: &[String],
    expected: &[String],
) -> Result<(), String> {
    if declared.iter().all(|target| expected.contains(target)) {
        return Ok(());
    }
    let path = format!("{root}/reports/{version}/depth-record.json");
    let record = tip
        .optional(&path)?
        .map(|raw| DepthRecord::parse(&raw, agent, version))
        .transpose()?;
    let covered = record.as_ref().map(covered_targets).unwrap_or_default();
    let missing: Vec<_> = declared
        .iter()
        .filter(|target| !expected.contains(target) && !covered.contains(target.as_str()))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "charter rule 4: targets have neither RULES.json union.expected_targets nor \
             depth-record coverage at the authenticated integration tip: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}

/// IN1 re-establishes reporting freeze inputs independently, using its pinned parent for
/// acquisition authority even during post-tip observation. Command labels are never authority.
pub(crate) fn check_freeze(
    root: &std::path::Path,
    parent: &Source,
    prior: Option<&DepthRecord>,
    record: &DepthRecord,
    entry: &crate::agent_registry::AgentRegistryEntry,
) -> Result<(), String> {
    use crate::agent_lifecycle::{self, LifecycleStage, LifecycleState};
    let path = agent_lifecycle::approval_artifact_path_for_entry(entry);
    let freeze = super::plan(root, &path, Some(parent))?;
    let prior_state: LifecycleState =
        serde_json::from_str(&parent.read(&freeze.state_path)?).map_err(|e| e.to_string())?;
    let previous_records = prior
        .map(|record| {
            (
                record.policy.selection.enrollment.version.clone(),
                record.clone(),
            )
        })
        .into_iter()
        .collect();
    crate::depth_record::onboarding::onboarding_binding(
        prior_state.support_depth.as_deref(),
        &freeze.approval,
        &previous_records,
    )
    .map_err(|faults| {
        faults
            .into_iter()
            .map(|f| f.message)
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    if prior_state.lifecycle_stage != LifecycleStage::Enrolled {
        return Err(
            "onboarding reporting freeze cannot reopen the window after implementation".into(),
        );
    }
    if parent.optional_bytes(&path)?.as_ref() != Some(&freeze.view.clone().required(&path)?) {
        return Err("onboarding freeze approval differs from its pinned parent".into());
    }
    let object = freeze
        .state
        .support_depth
        .as_ref()
        .ok_or("onboarding freeze lacks lifecycle support_depth object")?;
    if let Some(why) = record.request_mismatch(&object.binding(freeze.approval.event.clone())) {
        return Err(format!("onboarding reporting freeze: {why}"));
    }
    // Re-derivation binds current registry policy and all acquired inputs. An asserted hash or
    // a hand-assembled P/O object does not substitute for the shared planner.
    if freeze.plan.record != *record {
        return Err("onboarding reporting freeze differs from independently derived P/O".into());
    }
    let retained_execution = prior
        .filter(|p| p.event == record.event && p.obligations == record.obligations)
        .and_then(|p| p.execution.clone());
    if record.execution != retained_execution {
        return Err("onboarding freeze cannot manufacture execution evidence".into());
    }
    let carried = prior
        .and_then(|p| p.obligations.as_ref())
        .map(|o| o.acceptance_work.as_slice())
        .unwrap_or_default();
    let proposed_work = record
        .obligations
        .as_ref()
        .map(|o| o.acceptance_work.as_slice())
        .unwrap_or_default();
    if carried != proposed_work {
        return Err("onboarding freeze changed carried acceptance work".into());
    }
    if let Some(prior) = prior {
        if record.acceptance_entries != prior.acceptance_entries
            || !record.lacking_from(prior).is_empty()
        {
            return Err("onboarding freeze changed entries or shrank selection".into());
        }
        if prior.event == record.event && prior.policy != record.policy {
            return Err("onboarding re-freeze changed P".into());
        }
    } else if !record.acceptance_entries.is_empty() {
        return Err("onboarding freeze cannot add acceptance entries".into());
    }
    let mut expected = prior_state;
    expected.support_depth = freeze.state.support_depth.clone();
    if serde_json::to_value(expected).map_err(|e| e.to_string())?
        != serde_json::to_value(&freeze.state).map_err(|e| e.to_string())?
    {
        return Err("onboarding freeze changed lifecycle outside its P/O object".into());
    }
    freeze.view.verify()?;
    freeze.captured.recheck(root)
}
