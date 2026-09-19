use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use tempfile::TempDir;
use xtask::agent_maintenance::audit_status::{
    EXIT_INCOMPLETE_ACQUISITION, EXIT_TARGET_VERSION_MISMATCH, EXIT_UPLIFTS_REQUIRED,
};

const WORKFLOW: &str = ".github/workflows/parity-acquire.yml";
const MATERIALIZE_STEP: &str = "Materialize snapshots and assert the required target is present";
const GATE_STEP: &str = "Maintenance audit gate";
const TERMINAL_STEP: &str = "Fail the job if the maintenance audit recorded a blocking verdict";

#[derive(Debug)]
struct GateRun {
    output: Output,
    raw_output: String,
    outputs: BTreeMap<String, String>,
    summary: String,
}

#[derive(Clone, Copy)]
struct GateCase {
    status: i32,
    commit: bool,
    closeout_ready: &'static str,
    uplifts_required: &'static str,
    audit_failed: &'static str,
    title: &'static str,
    message: &'static str,
    summary_status: &'static str,
    annotation: &'static str,
}

#[test]
fn maintenance_audit_gate_routes_every_exit_by_behavior() {
    let cases = [
        GateCase {
            status: 0,
            commit: true,
            closeout_ready: "true",
            uplifts_required: "false",
            audit_failed: "false",
            title: "",
            message: "",
            summary_status: "clean",
            annotation: "",
        },
        GateCase {
            status: EXIT_UPLIFTS_REQUIRED,
            commit: true,
            closeout_ready: "false",
            uplifts_required: "true",
            audit_failed: "false",
            title: "",
            message: "",
            summary_status: "uplifts required",
            annotation: "",
        },
        GateCase {
            status: EXIT_INCOMPLETE_ACQUISITION,
            commit: true,
            closeout_ready: "false",
            uplifts_required: "false",
            audit_failed: "true",
            title: "Incomplete acquisition",
            message: "maintenance-audit-status reported that the acquisition union this run produced is incomplete; missing targets: linux-arm64, win32-x64",
            summary_status: "incomplete acquisition",
            annotation: "::error title=Incomplete acquisition::",
        },
        GateCase {
            status: EXIT_TARGET_VERSION_MISMATCH,
            commit: false,
            closeout_ready: "false",
            uplifts_required: "false",
            audit_failed: "false",
            title: "Target version mismatch",
            message: "mismatch detail",
            summary_status: "target version mismatch (dry run, not blocking)",
            annotation: "::notice title=Target version mismatch::mismatch detail",
        },
        GateCase {
            status: EXIT_TARGET_VERSION_MISMATCH,
            commit: true,
            closeout_ready: "false",
            uplifts_required: "false",
            audit_failed: "true",
            title: "Target version mismatch",
            message: "mismatch detail",
            summary_status: "target version mismatch",
            annotation: "::error title=Target version mismatch::mismatch detail",
        },
        failure_case(2),
        failure_case(1),
        failure_case(101),
    ];

    for case in cases {
        let run = run_gate(case.status, case.commit, true, case.message);
        assert!(
            run.output.status.success(),
            "gate must exit zero for xtask exit {}: {}",
            case.status,
            String::from_utf8_lossy(&run.output.stderr)
        );
        assert_gate_outputs(&run.outputs, &case);
        assert!(
            run.summary
                .contains(&format!("- status: {}", case.summary_status)),
            "summary for exit {} was:\n{}",
            case.status,
            run.summary
        );
        let stderr = String::from_utf8_lossy(&run.output.stderr);
        if !case.annotation.is_empty() {
            assert!(
                stderr.contains(case.annotation),
                "stderr for exit {} was:\n{}",
                case.status,
                stderr
            );
        }
        if case.status == EXIT_TARGET_VERSION_MISMATCH {
            let forbidden = if case.commit {
                "::notice title=Target version mismatch"
            } else {
                "::error title=Target version mismatch"
            };
            assert!(
                !stderr.contains(forbidden),
                "stderr for exit {} unexpectedly contained `{forbidden}`:\n{stderr}",
                case.status
            );
        }
        if case.status == EXIT_UPLIFTS_REQUIRED {
            assert!(run.summary.contains("Relay invocation placeholder (T3)"));
        }
    }
}

#[test]
fn maintenance_audit_gate_records_a_missing_request_without_running_cargo() {
    let run = run_gate(0, true, false, "unused");
    assert!(run.output.status.success());
    let expected = GateCase {
        status: 2,
        commit: true,
        closeout_ready: "false",
        uplifts_required: "false",
        audit_failed: "true",
        title: "Maintenance audit failed",
        message: "parity-acquire planned acquisition for codex, but the maintenance request path docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml is missing; the request must exist for the version under acquisition.",
        summary_status: "failed",
        annotation: "::error title=Maintenance audit failed::",
    };
    assert_gate_outputs(&run.outputs, &expected);
    let mismatch = run_gate(EXIT_TARGET_VERSION_MISMATCH, false, true, "mismatch detail");
    assert_eq!(mismatch.outputs["audit_failed"], "false");
}

#[test]
fn maintenance_audit_gate_uses_the_last_nonblank_stderr_line_for_mismatch() {
    let run = run_gate(
        EXIT_TARGET_VERSION_MISMATCH,
        false,
        true,
        "first detail\n\nfinal mismatch detail",
    );
    assert!(run.output.status.success());
    assert_eq!(run.outputs["audit_message"], "final mismatch detail");
    assert!(String::from_utf8_lossy(&run.output.stderr)
        .contains("::notice title=Target version mismatch::final mismatch detail"));
}

#[test]
fn maintenance_audit_gate_heredoc_has_no_fixed_delimiter_collision() {
    let first = run_gate(2, true, true, "__AUDIT_MESSAGE__");
    let second = run_gate(2, true, true, "__AUDIT_MESSAGE__");
    for run in [&first, &second] {
        assert!(run.output.status.success());
        assert_eq!(run.outputs["audit_message"], "__AUDIT_MESSAGE__");
        assert_eq!(run.outputs["audit_failed"], "true");
        assert_eq!(run.outputs["audit_exit_code"], "2");
    }
    let first_delimiter = audit_message_delimiter(&first.raw_output);
    let second_delimiter = audit_message_delimiter(&second.raw_output);
    assert_ne!(first_delimiter, second_delimiter);
    assert_ne!(first_delimiter, "__AUDIT_MESSAGE__");
    assert_ne!(second_delimiter, "__AUDIT_MESSAGE__");
}

#[test]
fn terminal_step_propagates_only_valid_blocking_exit_codes() {
    for code in ["1", "2", "4", "5", "255"] {
        let output = run_terminal(code);
        assert_eq!(
            output.status.code(),
            Some(code.parse::<i32>().expect("numeric test code")),
            "stderr for {code}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stderr)
            .contains("Invalid maintenance audit exit code"));
    }
    assert_eq!(run_terminal("0").status.code(), Some(1));
}

#[test]
fn terminal_step_rejects_nonblocking_or_invalid_exit_codes() {
    for code in ["0", "", "not-a-number", "-1", "256"] {
        let output = run_terminal(code);
        assert_eq!(output.status.code(), Some(1), "invalid code `{code}`");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("::error title=Invalid maintenance audit exit code::")
                && stderr.contains("was invalid"),
            "stderr for `{code}` was:\n{stderr}"
        );
    }
}

#[test]
fn union_job_runs_after_snapshot_failure_but_not_plan_failure_or_cancellation() {
    let workflow = read_workflow();
    let header = section_between(&workflow, "  union:\n", "    steps:\n");
    assert!(header.contains("needs: [plan, snapshot]"));
    assert!(header.contains("if: ${{ !cancelled() && needs.plan.result == 'success' }}"));
}

#[test]
fn failed_snapshot_leg_uploads_raw_help_evidence_but_never_its_snapshot() {
    let workflow = read_workflow();
    let raw_help_step = "      - name: Upload the raw help capture (never committed)\n";
    // A step-level `if:` here could let the union download a snapshot from a failed capture.
    let snapshot_upload = section_between(
        &workflow,
        "      - name: Upload the per-target snapshot\n",
        raw_help_step,
    );
    assert!(!snapshot_upload
        .lines()
        .any(|l| l.starts_with("        if:")));
    let raw_help_upload = section_between(&workflow, raw_help_step, "  union:\n");
    assert!(raw_help_upload.starts_with("        if: ${{ !cancelled() }}\n"));
}

#[test]
fn materialize_snapshots_removes_stale_planned_target_before_reporting_it_missing() {
    let run = run_materialize(&["linux-x64", "linux-arm64"], &["linux-x64"], "linux-x64");

    assert!(
        run.output.status.success(),
        "materialize step failed: {}",
        String::from_utf8_lossy(&run.output.stderr)
    );
    assert!(run.destination.join("linux-x64.json").is_file());
    assert!(
        !run.destination.join("linux-arm64.json").exists(),
        "a planned target with no artifact from this run must not retain its stale snapshot"
    );
    assert!(
        String::from_utf8_lossy(&run.output.stdout)
            .contains("::warning title=Missing target snapshot::linux-arm64 produced no snapshot"),
        "missing target was not reported: {}",
        String::from_utf8_lossy(&run.output.stdout)
    );
    assert!(String::from_utf8_lossy(&run.output.stdout).contains("missing target snapshots: 1"));
}

#[test]
fn materialize_snapshots_preserves_a_complete_acquisition() {
    let run = run_materialize(
        &["linux-x64", "linux-arm64"],
        &["linux-x64", "linux-arm64"],
        "linux-x64",
    );

    assert!(
        run.output.status.success(),
        "materialize step failed: {}",
        String::from_utf8_lossy(&run.output.stderr)
    );
    assert_eq!(
        fs::read_to_string(run.destination.join("linux-x64.json")).expect("fresh x64 snapshot"),
        "fresh linux-x64\n"
    );
    assert_eq!(
        fs::read_to_string(run.destination.join("linux-arm64.json")).expect("fresh arm64 snapshot"),
        "fresh linux-arm64\n"
    );
    assert!(String::from_utf8_lossy(&run.output.stdout).contains("missing target snapshots: 0"));
}

struct MaterializeRun {
    _temp: TempDir,
    destination: PathBuf,
    output: Output,
}

fn run_materialize(planned: &[&str], fresh: &[&str], required: &str) -> MaterializeRun {
    let temp = TempDir::new().expect("materialize tempdir");
    let plan = temp.path().join("_ci_tmp/acquisition-plan.json");
    fs::create_dir_all(plan.parent().expect("plan parent")).expect("create plan dir");
    let include = planned
        .iter()
        .map(|target| format!(r#"{{"target_triple":"{target}"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    fs::write(&plan, format!(r#"{{"include":[{include}]}}"#)).expect("write plan");

    let destination = temp.path().join("manifest/snapshots/1.2.3");
    fs::create_dir_all(&destination).expect("create destination");
    for target in planned {
        fs::write(
            destination.join(format!("{target}.json")),
            format!("stale {target}\n"),
        )
        .expect("write stale snapshot");
    }

    let snapshots = temp.path().join("_snapshots");
    fs::create_dir_all(&snapshots).expect("create downloaded snapshots dir");
    for target in fresh {
        fs::write(
            snapshots.join(format!("{target}.json")),
            format!("fresh {target}\n"),
        )
        .expect("write fresh snapshot");
    }

    let script = temp.path().join("materialize.sh");
    fs::write(&script, extract_run_block(MATERIALIZE_STEP)).expect("write materialize script");
    let mut command = runner_command(&script, temp.path());
    command.env("PATH", std::env::var("PATH").expect("PATH"));
    add_step_env(
        &mut command,
        MATERIALIZE_STEP,
        &[
            ("${{ inputs.target_version }}", "1.2.3"),
            ("${{ needs.plan.outputs.manifest_root }}", "manifest"),
            ("${{ needs.plan.outputs.required_target }}", required),
        ],
    );
    let output = command.output().expect("run materialize script");

    MaterializeRun {
        _temp: temp,
        destination,
        output,
    }
}

fn failure_case(status: i32) -> GateCase {
    GateCase {
        status,
        commit: true,
        closeout_ready: "false",
        uplifts_required: "false",
        audit_failed: "true",
        title: "Maintenance audit failed",
        message: "failure detail",
        summary_status: "failed",
        annotation: "::error title=Maintenance audit failed::failure detail",
    }
}

fn assert_gate_outputs(actual: &BTreeMap<String, String>, expected: &GateCase) {
    let expected = BTreeMap::from([
        (
            "closeout_ready".to_string(),
            expected.closeout_ready.to_string(),
        ),
        (
            "uplifts_required".to_string(),
            expected.uplifts_required.to_string(),
        ),
        (
            "audit_failed".to_string(),
            expected.audit_failed.to_string(),
        ),
        ("audit_exit_code".to_string(), expected.status.to_string()),
        ("audit_title".to_string(), expected.title.to_string()),
        ("audit_message".to_string(), expected.message.to_string()),
    ]);
    assert_eq!(actual, &expected);
}

fn run_gate(status: i32, commit: bool, request_exists: bool, stderr: &str) -> GateRun {
    let temp = TempDir::new().expect("gate tempdir");
    let bin = temp.path().join("bin");
    fs::create_dir_all(&bin).expect("create stub bin");
    write_executable(
        &bin.join("cargo"),
        b"#!/bin/bash\nprintf '%s\\n' \"$STUB_STDERR\" >&2\nexit \"$STUB_STATUS\"\n",
    );
    if request_exists {
        let request = temp
            .path()
            .join("docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml");
        fs::create_dir_all(request.parent().expect("request parent")).expect("create request dir");
        fs::write(request, "request stub\n").expect("write request");
    }
    let union = temp.path().join("manifest/snapshots/1.2.3/union.json");
    fs::create_dir_all(union.parent().expect("union parent")).expect("create union dir");
    fs::write(union, r#"{"missing_targets":["linux-arm64","win32-x64"]}"#).expect("write union");

    let output_path = temp.path().join("github-output");
    let summary_path = temp.path().join("github-summary");
    let script_path = temp.path().join("gate.sh");
    fs::write(&script_path, extract_run_block(GATE_STEP)).expect("write gate script");
    let mut command = runner_command(&script_path, temp.path());
    command
        .env("PATH", path_with_stub(&bin))
        .env("GITHUB_OUTPUT", &output_path)
        .env("GITHUB_STEP_SUMMARY", &summary_path)
        .env("STUB_STATUS", status.to_string())
        .env("STUB_STDERR", stderr);
    add_step_env(
        &mut command,
        GATE_STEP,
        &[
            ("${{ inputs.target_version }}", "1.2.3"),
            ("${{ needs.plan.outputs.manifest_root }}", "manifest"),
            ("${{ inputs.agent_id }}", "codex"),
            (
                "${{ inputs.commit }}",
                if commit { "true" } else { "false" },
            ),
        ],
    );
    let output = command.output().expect("run gate script");
    let raw_output = fs::read_to_string(output_path).expect("read outputs");

    GateRun {
        output,
        outputs: parse_github_output(&raw_output),
        raw_output,
        summary: fs::read_to_string(summary_path).expect("read summary"),
    }
}

fn run_terminal(code: &str) -> Output {
    let temp = TempDir::new().expect("terminal tempdir");
    let script = temp.path().join("terminal.sh");
    fs::write(&script, extract_run_block(TERMINAL_STEP)).expect("write terminal script");
    let output_path = temp.path().join("github-output");
    let summary_path = temp.path().join("github-summary");
    let mut command = runner_command(&script, temp.path());
    command
        .env("PATH", std::env::var("PATH").expect("PATH"))
        .env("GITHUB_OUTPUT", output_path)
        .env("GITHUB_STEP_SUMMARY", summary_path);
    add_step_env(
        &mut command,
        TERMINAL_STEP,
        &[
            (
                "${{ steps.maintenance_audit.outputs.audit_exit_code }}",
                code,
            ),
            (
                "${{ steps.maintenance_audit.outputs.audit_title }}",
                "Recorded verdict",
            ),
            (
                "${{ steps.maintenance_audit.outputs.audit_message }}",
                "recorded message",
            ),
        ],
    );
    command.output().expect("run terminal script")
}

fn runner_command(script: &Path, current_dir: &Path) -> Command {
    let mut command = Command::new("/bin/bash");
    command
        .args(["--noprofile", "--norc", "-eo", "pipefail"])
        .arg(script)
        .current_dir(current_dir)
        .env_clear();
    command
}

fn add_step_env(command: &mut Command, step_name: &str, values: &[(&str, &str)]) {
    for (key, expression) in extract_step_env(step_name) {
        let value = values
            .iter()
            .find_map(|(known_expression, value)| {
                (*known_expression == expression).then_some(*value)
            })
            .unwrap_or_else(|| {
                panic!("unknown env expression `{expression}` in step `{step_name}`")
            });
        command.env(key, value);
    }
}

fn write_executable(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("write executable");
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod executable");
}

fn path_with_stub(bin: &Path) -> String {
    format!("{}:{}", bin.display(), std::env::var("PATH").expect("PATH"))
}

fn read_workflow() -> String {
    fs::read_to_string(repo_root().join(WORKFLOW)).expect("read workflow")
}

fn extract_run_block(step_name: &str) -> String {
    let workflow = read_workflow();
    let marker = format!("      - name: {step_name}\n");
    let step = workflow
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing step `{step_name}`"))
        .1;
    let body = step
        .split_once("        run: |\n")
        .unwrap_or_else(|| panic!("step `{step_name}` has no run block"))
        .1;
    let mut script = String::new();
    for line in body.lines() {
        if let Some(line) = line.strip_prefix("          ") {
            script.push_str(line);
            script.push('\n');
        } else if line.trim().is_empty() {
            script.push('\n');
        } else {
            break;
        }
    }
    script
}

fn extract_step_env(step_name: &str) -> Vec<(String, String)> {
    let workflow = read_workflow();
    let marker = format!("      - name: {step_name}\n");
    let step = workflow
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing step `{step_name}`"))
        .1;
    let body = step
        .split_once("        env:\n")
        .unwrap_or_else(|| panic!("step `{step_name}` has no env block"))
        .1;
    body.lines()
        .take_while(|line| line.starts_with("          "))
        .map(|line| {
            line.trim_start()
                .split_once(": ")
                .map(|(key, expression)| (key.to_string(), expression.to_string()))
                .unwrap_or_else(|| panic!("invalid env binding `{line}` in step `{step_name}`"))
        })
        .collect()
}

fn audit_message_delimiter(output: &str) -> &str {
    output
        .lines()
        .find_map(|line| line.strip_prefix("audit_message<<"))
        .expect("audit_message delimiter")
}

fn parse_github_output(text: &str) -> BTreeMap<String, String> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut parsed = BTreeMap::new();
    let mut index = 0;
    while index < lines.len() {
        if let Some((name, delimiter)) = lines[index].split_once("<<") {
            index += 1;
            let start = index;
            while index < lines.len() && lines[index] != delimiter {
                index += 1;
            }
            assert!(index < lines.len(), "unterminated output `{name}`");
            parsed.insert(name.to_string(), lines[start..index].join("\n"));
        } else {
            let (name, value) = lines[index]
                .split_once('=')
                .unwrap_or_else(|| panic!("invalid output line `{}`", lines[index]));
            parsed.insert(name.to_string(), value.to_string());
        }
        index += 1;
    }
    parsed
}

fn section_between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let after = text
        .split_once(start)
        .unwrap_or_else(|| panic!("missing `{start}`"))
        .1;
    after
        .split_once(end)
        .unwrap_or_else(|| panic!("missing `{end}` after `{start}`"))
        .0
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}
