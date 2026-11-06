use super::*;
use crate::{
    agent_lifecycle::LifecycleDepth, depth_record::onboarding::OnboardingApproval,
    git_inputs::GitInputs, support_depth::FreezePlan,
};

const APPROVAL_PATH: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/approved-agent.toml";

#[test]
fn runtime_follow_on_policy_resolves_onboarding_and_preserves_p_on_refreeze() {
    let (f, _) = super::super::prepare::workspace();
    let root = f.root.path();
    write(
        root,
        REGISTRY_RELATIVE_PATH,
        format!(
            "{}\n{}",
            include_str!("../../../data/agent_registry.toml"),
            include_str!(
                "../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml"
            )
        ),
    );
    let raw = include_str!(
        "../../../tests/fixtures/depth-synthetic/onboarding/governance/approved-agent.toml"
    )
    .replace("{{APPROVAL_COMMIT}}", "abc1234")
    .replace("{{APPROVAL_RECORDED_AT}}", "2026-11-05T00:00:00Z");
    let approval = OnboardingApproval::parse(&raw, APPROVAL_PATH).unwrap();
    f.commit();
    let _installed = f.install();
    crate::isolated_maintenance::route(root).unwrap().unwrap();
    let plan = FreezePlan::onboarding(&mut GitInputs::new(root).unwrap(), &approval, None).unwrap();
    assert_eq!(plan.record.event, approval.event);
    assert!(plan.record.obligations.is_none());
    let object = LifecycleDepth {
        policy_identity: plan.record.policy.identity.clone(),
        obligations_identity: None,
    };
    let record_path = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
    write(
        root,
        record_path,
        serde_json::to_vec_pretty(&plan.record).unwrap(),
    );
    f.commit();
    let repeated =
        FreezePlan::onboarding(&mut GitInputs::new(root).unwrap(), &approval, Some(&object))
            .unwrap();
    assert_eq!(plan.record, repeated.record);
}
