//! Observe invocation clocks from the actual public ON2 output for independent fake authority.
use super::*;

pub(in super::super) fn freeze(
    f: &Fixture,
) -> crate::depth_acceptance::admission::freeze::authority::Writer {
    crate::runtime_follow_on::run_in_workspace(
        f.root.path(),
        crate::runtime_follow_on::Args::parse_from([
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            APPROVAL,
            "--run-id",
            "fr2-freeze",
        ]),
        &mut Vec::new(),
    )
    .unwrap();
    let packet = f
        .root
        .path()
        .join("docs/agents/.uaa-temp/runtime-follow-on/runs/fr2-freeze");
    let read = |name: &str| -> Value {
        serde_json::from_slice(&fs::read(packet.join(name)).unwrap()).unwrap()
    };
    let contract = read("input-contract.json");
    let status = read("run-status.json");
    assert_eq!(status["run_id"], "fr2-freeze");
    assert_eq!(status["status"], "dry_run_ready");
    crate::depth_acceptance::admission::freeze::authority::Writer::Onboarding {
        run_id: "fr2-freeze".into(),
        contract_generated_at: contract["generated_at"].as_str().unwrap().into(),
        packet_generated_at: status["generated_at"].as_str().unwrap().into(),
        requested_tier: crate::runtime_follow_on::RequestedTier::Default,
        minimal_justification_file: None,
        allow_rich_surface: vec![],
    }
}
