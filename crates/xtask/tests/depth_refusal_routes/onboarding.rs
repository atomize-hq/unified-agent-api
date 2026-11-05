//! ON1–ON5: the onboarding write entrypoints refuse a depth workspace before any write.
//! Public depth refusal is workspace-wide (Annex A.1), so an approval of an agent other than the
//! depth subject is refused too.

use super::support::{Depth, Workspace};

const APPROVAL: &str =
    "docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml";
const CLOSEOUT: &str =
    "docs/agents/lifecycle/opencode-cli-onboarding/governance/proving-run-closeout.json";

fn refuses(invocations: &[(&str, &[&str])]) {
    for depth in Depth::ALL {
        let ws = Workspace::with(depth);
        for (route, args) in invocations {
            ws.assert_refuses(args, &format!("{route}: {}", depth.guard_reason()));
        }
    }
}

#[test]
fn on1_onboard_agent_write_refuses_depth_workspace() {
    refuses(&[(
        "onboard_agent",
        &["onboard-agent", "--write", "--approval", APPROVAL],
    )]);
}

#[test]
fn on2_runtime_follow_on_and_repair_runtime_evidence_writes_refuse_depth_workspace() {
    refuses(&[
        (
            "runtime_follow_on",
            &[
                "runtime-follow-on",
                "--write",
                "--approval",
                APPROVAL,
                "--run-id",
                "depth-refusal",
            ],
        ),
        (
            "repair_runtime_evidence",
            &["repair-runtime-evidence", "--write", "--approval", APPROVAL],
        ),
    ]);
}

#[test]
fn on3_prepare_publication_write_refuses_depth_workspace() {
    refuses(&[(
        "prepare_publication",
        &["prepare-publication", "--write", "--approval", APPROVAL],
    )]);
}

#[test]
fn on4_refresh_publication_write_refuses_depth_workspace() {
    refuses(&[(
        "publication_refresh",
        &["refresh-publication", "--write", "--approval", APPROVAL],
    )]);
}

#[test]
fn on5_proving_run_closeout_entrypoints_refuse_depth_workspace() {
    refuses(&[
        (
            "close_proving_run",
            &[
                "close-proving-run",
                "--approval",
                APPROVAL,
                "--closeout",
                CLOSEOUT,
            ],
        ),
        (
            "prepare_proving_run_closeout",
            &[
                "prepare-proving-run-closeout",
                "--write",
                "--approval",
                APPROVAL,
            ],
        ),
    ]);
}
