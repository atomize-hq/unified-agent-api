use super::*;

use serde_json::Value;

const VERDICT_STEP: &str = "Record the maintenance audit verdict";

// The verdict artifact is the only channel that survives the `union` job's own failure, so what it
// records has to be exact. Two distinctions carry the weight: an audit that never ran is not a
// clean audit, and a blocking verdict is not a failed delivery. Everything here pins one of those.

struct VerdictRun {
    _temp: TempDir,
    verdict: Value,
}

fn run_verdict(overrides: &[(&str, &str)]) -> VerdictRun {
    let mut values: Vec<(&str, &str)> = vec![
        ("${{ inputs.agent_id }}", "codex"),
        ("${{ inputs.target_version }}", "1.2.3"),
        ("${{ inputs.ref }}", "automation/codex-maintenance-1.2.3"),
        ("${{ inputs.commit }}", "true"),
        (
            "${{ steps.maintenance_audit.outputs.audit_exit_code }}",
            "0",
        ),
        (
            "${{ steps.maintenance_audit.outputs.audit_failed }}",
            "false",
        ),
        ("${{ steps.maintenance_audit.outputs.audit_title }}", ""),
        ("${{ steps.maintenance_audit.outputs.audit_message }}", ""),
        (
            "${{ steps.maintenance_audit.outputs.closeout_ready }}",
            "true",
        ),
        (
            "${{ steps.maintenance_audit.outputs.uplifts_required }}",
            "false",
        ),
        ("${{ steps.commit_artifacts.outputs.committed }}", "true"),
        ("${{ steps.commit_artifacts.outputs.head_sha }}", "abc1234"),
        ("${{ github.run_id }}", "42"),
        ("${{ github.run_attempt }}", "1"),
    ];
    for (expression, value) in overrides {
        let slot = values
            .iter_mut()
            .find(|(known, _)| known == expression)
            .unwrap_or_else(|| panic!("unknown override `{expression}`"));
        slot.1 = value;
    }

    let temp = TempDir::new().expect("verdict tempdir");
    let output_path = temp.path().join("github-output");
    fs::write(&output_path, "").expect("seed github output");
    let script_path = temp.path().join("verdict.sh");
    fs::write(&script_path, extract_run_block(VERDICT_STEP)).expect("write verdict script");

    let mut command = runner_command(&script_path, temp.path());
    command
        .env("PATH", std::env::var("PATH").expect("PATH"))
        .env("GITHUB_OUTPUT", &output_path)
        .env("RUNNER_TEMP", temp.path());
    add_step_env(&mut command, VERDICT_STEP, &values);
    let output = command.output().expect("run verdict script");
    assert!(
        output.status.success(),
        "verdict step failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let raw_output = fs::read_to_string(&output_path).expect("read github output");
    let path = parse_github_output(&raw_output)
        .remove("path")
        .expect("the verdict step must export the path it wrote");
    let verdict = serde_json::from_slice(&fs::read(&path).expect("read verdict json"))
        .expect("verdict must be valid json");
    VerdictRun {
        _temp: temp,
        verdict,
    }
}

#[test]
fn a_clean_run_records_an_observed_verdict_and_a_delivered_commit() {
    let run = run_verdict(&[]);
    assert_eq!(run.verdict["schema_version"], 1);
    assert_eq!(run.verdict["packet"]["agent_id"], "codex");
    assert_eq!(run.verdict["packet"]["commit_mode"], true);
    assert_eq!(run.verdict["producer"]["run_attempt"], 1);
    assert_eq!(run.verdict["audit"]["observed"], true);
    assert_eq!(run.verdict["audit"]["exit_code"], 0);
    assert_eq!(run.verdict["audit"]["blocking"], false);
    assert_eq!(run.verdict["audit"]["closeout_ready"], true);
    assert_eq!(run.verdict["delivery"]["committed"], true);
    assert_eq!(run.verdict["delivery"]["head_sha"], "abc1234");
}

#[test]
fn a_blocking_verdict_keeps_its_diagnostic_and_reports_delivery_separately() {
    // Exit 2 writes no projection, so this message is the only place the offending debt rows are
    // named. It is also multiline, which is why the step passes it to `jq` as an argument rather
    // than interpolating it.
    let message = "debt row `codex exec` matches no live gap\nand the debt baseline moved";
    let run = run_verdict(&[
        (
            "${{ steps.maintenance_audit.outputs.audit_exit_code }}",
            "2",
        ),
        (
            "${{ steps.maintenance_audit.outputs.audit_failed }}",
            "true",
        ),
        (
            "${{ steps.maintenance_audit.outputs.audit_title }}",
            "Maintenance audit failed",
        ),
        (
            "${{ steps.maintenance_audit.outputs.audit_message }}",
            message,
        ),
        (
            "${{ steps.maintenance_audit.outputs.closeout_ready }}",
            "false",
        ),
    ]);
    assert_eq!(run.verdict["audit"]["observed"], true);
    assert_eq!(run.verdict["audit"]["exit_code"], 2);
    assert_eq!(run.verdict["audit"]["blocking"], true);
    assert_eq!(run.verdict["audit"]["message"], message);
    // The run still committed. A blocking verdict and a failed push are different facts and the
    // artifact has to keep them apart, or the PR reports one as the other.
    assert_eq!(run.verdict["delivery"]["committed"], true);
}

#[test]
fn a_gate_that_never_recorded_an_exit_is_not_a_clean_run() {
    let run = run_verdict(&[
        ("${{ steps.maintenance_audit.outputs.audit_exit_code }}", ""),
        ("${{ steps.maintenance_audit.outputs.audit_failed }}", ""),
        ("${{ steps.maintenance_audit.outputs.closeout_ready }}", ""),
        (
            "${{ steps.maintenance_audit.outputs.uplifts_required }}",
            "",
        ),
    ]);
    assert_eq!(run.verdict["audit"]["observed"], false);
    assert!(
        run.verdict["audit"]["exit_code"].is_null(),
        "an unobserved audit must not carry an exit code: {}",
        run.verdict["audit"]
    );
    assert!(
        run.verdict["audit"]["closeout_ready"].is_null(),
        "an unobserved audit must not look closeout-ready: {}",
        run.verdict["audit"]
    );
}

#[test]
fn a_dry_run_records_no_delivery_attempt() {
    let run = run_verdict(&[
        ("${{ inputs.commit }}", "false"),
        ("${{ steps.commit_artifacts.outputs.committed }}", ""),
        ("${{ steps.commit_artifacts.outputs.head_sha }}", ""),
    ]);
    assert_eq!(run.verdict["packet"]["commit_mode"], false);
    assert_eq!(run.verdict["delivery"]["attempted"], false);
    assert!(run.verdict["delivery"]["committed"].is_null());
}

#[test]
fn a_commit_step_that_reported_nothing_is_not_a_successful_delivery() {
    // The commit step is skipped when an earlier one fails, so its outputs arrive empty. That is
    // not the same as "committed nothing", and defaulting either way would be a lie.
    let run = run_verdict(&[
        ("${{ steps.commit_artifacts.outputs.committed }}", ""),
        ("${{ steps.commit_artifacts.outputs.head_sha }}", ""),
    ]);
    assert_eq!(run.verdict["delivery"]["attempted"], true);
    assert_eq!(run.verdict["delivery"]["committed"], false);
    assert_eq!(run.verdict["delivery"]["unreported"], true);
    assert!(run.verdict["delivery"]["head_sha"].is_null());
}
