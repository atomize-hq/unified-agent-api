use std::{
    path::Path,
    process::{Command, Output},
};

const HOST_RUN_ID_ENV: &str = "XTASK_AGENT_MAINTENANCE_RUN_ID";
const HOST_RUN_ID: &str = "codex-0.156.1-run-20260925";

fn run_xtask(args: &[&str]) -> Output {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask crate lives under the workspace root");
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .env(HOST_RUN_ID_ENV, HOST_RUN_ID)
        .current_dir(workspace_root)
        .output()
        .expect("run xtask subprocess")
}

fn assert_nested_refusal(output: &Output, command: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "stderr: {stderr}");
    for expected in [command, HOST_RUN_ID, HOST_RUN_ID_ENV] {
        assert!(
            stderr.contains(expected),
            "stderr must contain `{expected}`: {stderr}"
        );
    }
}

#[test]
fn guarded_lifecycle_commands_refuse_inside_relay_session() {
    let cases: &[(&str, &[&str])] = &[
        (
            "prepare-agent-maintenance",
            &[
                "prepare-agent-maintenance",
                "--from-request",
                "missing-maintenance-request.toml",
                "--dry-run",
            ],
        ),
        (
            "execute-agent-maintenance",
            &[
                "execute-agent-maintenance",
                "--dry-run",
                "--request",
                "missing-maintenance-request.toml",
                "--codex-binary",
                "/nonexistent",
            ],
        ),
        (
            "refresh-agent",
            &[
                "refresh-agent",
                "--dry-run",
                "--request",
                "missing-maintenance-request.toml",
            ],
        ),
        (
            "close-agent-maintenance",
            &[
                "close-agent-maintenance",
                "--request",
                "missing-maintenance-request.toml",
                "--closeout",
                "missing-maintenance-closeout.json",
            ],
        ),
        (
            "prepare-agent-closeout",
            &[
                "prepare-agent-closeout",
                "--request",
                "missing-maintenance-request.toml",
                "--commit",
                "deadbeef",
                "--recorded-at",
                "2026-09-25T00:00:00Z",
            ],
        ),
    ];

    for (command, args) in cases {
        assert_nested_refusal(&run_xtask(args), command);
    }
}

#[test]
fn lifecycle_query_commands_are_not_blocked_inside_relay_session() {
    let audit = run_xtask(&[
        "maintenance-audit-status",
        "--request",
        "missing-maintenance-request.toml",
    ]);
    let audit_stderr = String::from_utf8_lossy(&audit.stderr);
    assert!(
        !audit_stderr.contains("running inside the `execute-agent-maintenance` session"),
        "audit status must not be blocked: {audit_stderr}"
    );

    let stand_down = run_xtask(&[
        "maintenance-stand-down-check",
        "--agent",
        "codex",
        "--target-version",
        "0.156.1",
    ]);
    let stand_down_stderr = String::from_utf8_lossy(&stand_down.stderr);
    assert!(
        !stand_down_stderr.contains("running inside the `execute-agent-maintenance` session"),
        "stand-down check must not be blocked: {stand_down_stderr}"
    );
}
