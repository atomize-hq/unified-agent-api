use std::process::Command;

use super::*;

const WORKFLOW: &str = ".github/workflows/parity-acquire.yml";
const GUARD: &str = "- name: Require an automation token for committing acquisition";

fn plan(yml: &str) -> &str {
    section_between(yml, "\n  plan:\n", "\n  snapshot:\n", WORKFLOW)
}

fn guard(plan: &str) -> &str {
    section_between(plan, GUARD, "      - uses: actions/checkout@", WORKFLOW)
}

#[test]
fn committing_acquisition_requires_a_token_before_any_acquisition_work() {
    let yml = read_repo_file(WORKFLOW);
    let plan = plan(&yml);
    assert!(
        plan.split_once("    steps:\n")
            .expect("plan steps")
            .1
            .trim_start()
            .starts_with(GUARD),
        "credential admission must run before checkout, build or upstream acquisition"
    );
    let guard = guard(plan);
    assert!(guard.contains("COMMIT_ARTIFACTS: ${{ inputs.commit }}"));
    assert!(guard.contains("HAS_AUTOMATION_TOKEN: ${{ secrets.AUTOMATION_TOKEN != '' }}"));
    assert!(
        !guard.lines().any(|line| {
            let line = line.trim_start();
            line.starts_with("if:") || line.starts_with("continue-on-error:")
        }),
        "the credential guard must neither be skipped nor allow a failure to continue"
    );
    assert!(
        yml.contains("    needs: plan\n"),
        "snapshots depend on admission"
    );
    let union = section_between(&yml, "\n  union:\n", "    steps:\n", WORKFLOW);
    assert!(union.contains("needs: [plan, snapshot]"));
    assert!(union.contains("needs.plan.result == 'success'"));
}

#[test]
fn acquisition_token_admission_runs_the_workflow_shell_for_both_delivery_modes() {
    let yml = read_repo_file(WORKFLOW);
    let guard = guard(plan(&yml));
    let body = guard.split_once("        run: |\n").expect("guard shell").1;
    let script = body
        .lines()
        .map(|line| line.strip_prefix("          ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");
    for commit in ["false", "true"] {
        for has_token in ["false", "true", ""] {
            let output = Command::new("bash")
                .args(["--noprofile", "--norc", "-c", &script])
                .env_clear()
                .env("COMMIT_ARTIFACTS", commit)
                .env("HAS_AUTOMATION_TOKEN", has_token)
                .output()
                .expect("run credential guard");
            let admitted = commit == "false" || has_token == "true";
            assert_eq!(
                output.status.success(),
                admitted,
                "commit={commit}, token present={has_token}: {output:?}"
            );
            if !admitted {
                assert_eq!(output.status.code(), Some(1));
                assert!(String::from_utf8_lossy(&output.stderr)
                    .contains("::error title=Missing automation token::"));
            }
        }
    }
}

#[test]
fn only_committing_acquisition_checks_out_with_the_delivery_token() {
    let yml = read_repo_file(WORKFLOW);
    let union = section_between(
        &yml,
        "\n  union:\n",
        "      - uses: dtolnay/rust-toolchain@stable",
        WORKFLOW,
    );
    assert!(
        union.contains("token: ${{ inputs.commit && secrets.AUTOMATION_TOKEN || github.token }}")
    );
    assert!(union.contains("persist-credentials: ${{ inputs.commit }}"));
    let caller = read_repo_file(".github/workflows/agent-maintenance-open-pr.yml");
    let acquisition = section_between(
        &caller,
        "uses: ./.github/workflows/parity-acquire.yml",
        "\n  publish-verdict:",
        ".github/workflows/agent-maintenance-open-pr.yml",
    );
    assert!(acquisition.contains("AUTOMATION_TOKEN: ${{ secrets.AUTOMATION_TOKEN }}"));
}
