//! Private onboarding freeze controller. No public entry calls this before roadmap E.6.
use super::*;
use crate::{
    agent_lifecycle::{self, LifecycleState},
    depth_record::onboarding::OnboardingApproval,
    git_inputs::GitInputs,
    isolated_maintenance::CapturedRegistry,
    runtime_advertising::source::Source,
    support_depth::FreezePlan,
};
use serde_json::Value;

mod approval;
mod integration;
mod packet;

struct Freeze {
    captured: CapturedRegistry,
    view: GitInputs,
    plan: FreezePlan,
    state: LifecycleState,
    state_path: String,
    state_before: Vec<u8>,
    approval: OnboardingApproval,
    approval_sha256: String,
}

fn plan(workspace: &Path, approval_path: &str) -> Result<Freeze, String> {
    let captured = crate::isolated_maintenance::route(workspace)?
        .ok_or("onboarding depth freeze requires an installed isolated route")?
        .capture(workspace)?;
    // This host exists ONLY for scope.rs's compiled synthetic subject, never the runtime pilot.
    let host = captured.evidence_host(workspace)?;
    // Pin every integration read to the authenticated object id through the hardened Git runner.
    // The route rejects grafts and object alternates on capture and recheck.
    let tip = Source::tip(workspace, host.tip())?;
    let mut view = GitInputs::new(workspace)?;
    let raw = view.required(approval_path)?;
    let approval = approval::read(&raw, approval_path, &captured)?;
    let approval_sha256 = view.reference(approval_path)?.sha256;
    let entry = captured
        .registry()
        .find(&approval.agent_id)
        .ok_or("unknown approval agent")?;
    let version = approval
        .support_depth_version
        .as_deref()
        .ok_or("charter rule 3 unresolved: approval states no depth version")?;
    let root = &entry.manifest_root;
    let state_path = agent_lifecycle::lifecycle_state_path(&approval.onboarding_pack_prefix);
    let state_before = view.required(&state_path)?;
    let state: LifecycleState = serde_json::from_slice(&state_before).map_err(|e| e.to_string())?;
    state
        .validate_in_workspace(workspace)
        .map_err(|e| e.to_string())?;
    if state.agent_id != approval.agent_id
        || state.onboarding_pack_prefix != approval.onboarding_pack_prefix
        || state.approval_artifact_path
            != agent_lifecycle::approval_artifact_path(&approval.onboarding_pack_prefix)
    {
        return Err("lifecycle attribution differs from the approval".into());
    }
    if state.lifecycle_stage != agent_lifecycle::LifecycleStage::Enrolled {
        return Err(format!("runtime-follow-on requires lifecycle stage `enrolled` (found `{}`); after runtime_integrated a new approval is required", state.lifecycle_stage.as_str()));
    }
    use crate::agent_registry::support_depth::LifecyclePath;
    let declaration = entry
        .support_depth
        .as_ref()
        .filter(|d| {
            d.enabled_paths
                .as_ref()
                .is_some_and(|p| p.contains(&LifecyclePath::Onboarding))
        })
        .and_then(|d| d.enrollments.as_ref())
        .and_then(|rows| {
            rows.iter().find(|row| {
                row.version == version && row.lifecycle_path == LifecyclePath::Onboarding
            })
        })
        .ok_or(
            "charter rule 3 unresolved: no enabled onboarding enrollment for the approved version",
        )?;
    rule_four(
        &mut view,
        &tip,
        &approval.agent_id,
        root,
        version,
        &declaration.targets,
    )?;
    complete_union(&mut view, root, version)?;
    let plan = FreezePlan::onboarding(&mut view, &approval, state.support_depth.as_deref())?;
    if let Some(obligations) = &plan.record.obligations {
        // The protected tip is the named integration ref. Never read a branch-local statement.
        let statement = integration::statement(
            &tip,
            &approval.onboarding_pack_prefix,
            &approval.agent_id,
            version,
        )?
        .ok_or_else(|| {
            format!(
                "acquisition statement missing at integration ref `{}`; provenance is undecided",
                host.tip()
            )
        })?;
        if let Some(why) = statement.mismatch(&obligations.acquired_inputs) {
            return Err(why);
        }
        let mut differing = Vec::new();
        for input in &obligations.acquired_inputs {
            let bytes = view.required(&input.path)?;
            if tip.optional_bytes(&input.path)?.as_ref() != Some(&bytes) {
                differing.push(input.path.clone());
            }
        }
        if !differing.is_empty() {
            return Err(format!(
                "acquired inputs differ from integration ref `{}`: {}",
                host.tip(),
                differing.join(", ")
            ));
        }
    }
    view.verify()?;
    captured.recheck(workspace)?;
    Ok(Freeze {
        captured,
        view,
        plan,
        state,
        state_path,
        state_before,
        approval,
        approval_sha256,
    })
}

fn rule_four(
    view: &mut GitInputs,
    tip: &Source,
    agent: &str,
    root: &str,
    version: &str,
    targets: &[String],
) -> Result<(), String> {
    let path = format!("{root}/versions/{version}.json");
    let metadata: Value =
        serde_json::from_slice(&view.required(&path)?).map_err(|e| format!("{path}: {e}"))?;
    if metadata["schema_version"] != 1 || metadata["semantic_version"].as_str() != Some(version) {
        return Err(format!(
            "charter rule 4: {path} does not name the approved exact upstream version"
        ));
    }
    let rules_path = format!("{root}/RULES.json");
    let rules: Value =
        serde_json::from_slice(&view.required(&rules_path)?).map_err(|e| e.to_string())?;
    if rules["storage"]["committed"]["reports"]["depth_record"].as_str()
        != Some(format!("{root}/reports/<version>/depth-record.json").as_str())
    {
        return Err(format!(
            "charter rule 4: {rules_path} does not adopt the canonical depth-record path"
        ));
    }
    let expected: Vec<String> = serde_json::from_value(rules["union"]["expected_targets"].clone())
        .map_err(|e| format!("charter rule 4: {rules_path}: {e}"))?;
    integration::targets(tip, agent, root, version, targets, &expected)?;
    let path = format!("{root}/VALIDATOR_SPEC.md");
    let spec = view.required(&path)?;
    let spec = text(&spec)?;
    if !spec.contains("## Depth records")
        || !spec.contains("Annex B")
        || !spec.contains("docs/specs/support-depth-contract.md")
    {
        return Err(format!(
            "charter rule 4: {path} does not adopt Annex B; adoption is undecided"
        ));
    }
    Ok(())
}

fn complete_union(view: &mut GitInputs, root: &str, version: &str) -> Result<(), String> {
    let path = format!("{root}/snapshots/{version}/union.json");
    let Some(raw) = view.optional(&path)? else {
        return Ok(());
    };
    let union: Value = serde_json::from_slice(&raw).map_err(|e| format!("{path}: {e}"))?;
    if union["complete"].as_bool() == Some(true) {
        return Ok(());
    }
    let expected = union["expected_targets"]
        .as_array()
        .ok_or("union expected_targets is undecided")?;
    let inputs = union["inputs"]
        .as_array()
        .ok_or("union inputs are undecided")?;
    let mut missing: Vec<_> = expected
        .iter()
        .filter_map(Value::as_str)
        .filter(|t| {
            !inputs
                .iter()
                .any(|i| i["target_triple"].as_str() == Some(t))
        })
        .map(str::to_owned)
        .collect();
    if let Some(named) = union["missing_targets"].as_array() {
        missing.extend(named.iter().filter_map(Value::as_str).map(str::to_owned));
    }
    missing.sort();
    missing.dedup();
    Err(format!("OF-P7: present union is incomplete; missing targets: {}. Partial-union admission is undecided", if missing.is_empty() { "undecided".into() } else { missing.join(", ") }))
}

fn text(raw: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(raw).map_err(|e| e.to_string())
}

/// No public dispatch calls this: ON2 remains refused until E.6. No executor is reachable here.
pub(super) fn dry_run(workspace: &Path, args: Args, writer: &mut impl Write) -> Result<(), Error> {
    if !args.dry_run || args.write {
        return Err(Error::Validation(
            "private depth freeze accepts only --dry-run".into(),
        ));
    }
    validate_args(&args, workspace)?;
    let run_id = args.run_id.clone().unwrap_or_else(generate_run_id);
    crate::runtime_evidence_run::validate_run_id(&run_id).map_err(Error::Validation)?;
    let mut freeze = plan(workspace, &args.approval).map_err(Error::Validation)?;
    let object = agent_lifecycle::LifecycleDepth {
        policy_identity: freeze.plan.record.policy.identity.clone(),
        obligations_identity: freeze
            .plan
            .record
            .obligations
            .as_ref()
            .map(|o| o.identity.clone()),
    };
    freeze.state.support_depth =
        Some(agent_lifecycle::Preserved::new(object).map_err(|e| Error::Internal(e.to_string()))?);
    freeze
        .state
        .validate_in_workspace(workspace)
        .map_err(|e| Error::Validation(e.to_string()))?;
    let record_path = format!(
        "{}/reports/{}/depth-record.json",
        freeze.approval.manifest_root,
        freeze
            .approval
            .support_depth_version
            .as_deref()
            .ok_or_else(|| Error::Validation("depth version undecided".into()))?
    );
    let mut mutations = packet::prepare(workspace, &args, &run_id, &freeze)?;
    use crate::workspace_mutation::PlannedMutation;
    let record = packet::json_bytes(&freeze.plan.record)?;
    mutations.push(match &freeze.plan.previous_record {
        Some(before) => PlannedMutation::replace(record_path, before.clone(), record),
        None => PlannedMutation::create(record_path, record),
    });
    mutations.push(PlannedMutation::replace(
        &freeze.state_path,
        freeze.state_before.clone(),
        packet::json_bytes(&freeze.state)?,
    ));
    // The owning LW1 path admits only this record/object and these reporting artifacts. The
    // production lifecycle writer remains refusing; no general depth-writing bypass is exposed.
    freeze.view.verify().map_err(Error::Validation)?;
    freeze
        .captured
        .recheck(workspace)
        .map_err(Error::Validation)?;
    crate::workspace_mutation::apply_admitted_mutations(workspace, &mutations)
        .map_err(|e| Error::Validation(e.to_string()))?;
    writeln!(
        writer,
        "OK: runtime-follow-on dry-run packet prepared.\nrun_id: {run_id}"
    )
    .map_err(|e| Error::Internal(e.to_string()))?;
    if freeze.plan.record.obligations.is_none() {
        writeln!(writer, "O not yet produced: a passing write run closes this generation's window to freeze O; depth acceptance then requires a new approval.")
            .map_err(|e| Error::Internal(e.to_string()))?;
    }
    Ok(())
}
