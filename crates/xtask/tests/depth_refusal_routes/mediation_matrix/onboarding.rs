use super::*;

const APPROVAL: &str =
    "docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml";
const CLOSEOUT: &str =
    "docs/agents/lifecycle/opencode-cli-onboarding/governance/proving-run-closeout.json";

#[test]
fn every_on_entry_refuses_onboarding_carriers_without_scope_authority() {
    let routes: [(&str, Vec<&str>); 7] = [
        (
            "onboard_agent",
            vec!["onboard-agent", "--write", "--approval", APPROVAL],
        ),
        (
            "runtime_follow_on",
            vec![
                "runtime-follow-on",
                "--write",
                "--approval",
                APPROVAL,
                "--run-id",
                "s11-matrix",
            ],
        ),
        (
            "repair_runtime_evidence",
            vec!["repair-runtime-evidence", "--write", "--approval", APPROVAL],
        ),
        (
            "prepare_publication",
            vec!["prepare-publication", "--write", "--approval", APPROVAL],
        ),
        (
            "publication_refresh",
            vec!["refresh-publication", "--write", "--approval", APPROVAL],
        ),
        (
            "prepare_proving_run_closeout",
            vec![
                "prepare-proving-run-closeout",
                "--write",
                "--approval",
                APPROVAL,
            ],
        ),
        (
            "close_proving_run",
            vec![
                "close-proving-run",
                "--approval",
                APPROVAL,
                "--closeout",
                CLOSEOUT,
            ],
        ),
    ];
    assert_eq!(routes.len(), 7);
    for carrier in Carrier::ALL {
        let ws = workspace(carrier);
        for (route, args) in &routes {
            ws.assert_refuses(args, &format!("{route}: {}", carrier.reason()));
        }
    }
}

#[test]
fn ordinary_neutral_command_reaches_its_production_result() {
    let ws = Workspace::plain();
    let output = ws.xtask(&["capability-matrix", "--out", "ordinary/matrix.md"]);
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{error}");
    let written = fs::read_to_string(ws.root().join("ordinary/matrix.md")).unwrap();
    assert!(written.contains("# Capability matrix"));
}
