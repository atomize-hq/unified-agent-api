use super::*;

const CALLER_WORKFLOW: &str = ".github/workflows/agent-maintenance-open-pr.yml";
const PUBLISH_STEP: &str = "Comment the verdict on the packet PR";

// What this job writes is the only thing a maintainer sees about an acquisition, so the body it
// renders is the contract. Each case below is a sentence that would be wrong in a way nobody could
// catch by reading the PR: a missing verdict read as clean, a blocking verdict shown with a
// pasteable relay command, a superseded run overwriting a current one.

struct PublishRun {
    _temp: TempDir,
    body: String,
    stdout: String,
    calls: String,
}

fn run_publisher(verdict: Option<&str>, existing_comments: &str, run_id: &str) -> PublishRun {
    let temp = TempDir::new().expect("publisher tempdir");
    let bin = temp.path().join("bin");
    fs::create_dir_all(&bin).expect("create stub bin");

    // `gh` is stubbed so the test observes what would have been sent. The first argument set is
    // recorded, and a comment list is served from a fixture.
    fs::write(temp.path().join("comments-fixture.json"), existing_comments)
        .expect("write comments fixture");
    write_executable(
        &bin.join("gh"),
        b"#!/bin/bash\n\
printf '%s\\n' \"$*\" >> \"$STUB_CALLS\"\n\
for arg in \"$@\"; do\n\
  case \"$arg\" in\n\
    body=@*) cp \"${arg#body=@}\" \"$STUB_BODY\" ;;\n\
  esac\n\
done\n\
case \"$*\" in\n\
  *comments\\ --paginate*) cat \"$STUB_COMMENTS\" ;;\n\
esac\n\
exit 0\n",
    );

    if let Some(verdict) = verdict {
        let dir = temp.path().join("_verdict");
        fs::create_dir_all(&dir).expect("create verdict dir");
        fs::write(dir.join("verdict.json"), verdict).expect("write verdict");
    }

    let body_path = temp.path().join("captured-body.md");
    let calls_path = temp.path().join("gh-calls.txt");
    fs::write(&calls_path, "").expect("seed calls");
    let script_path = temp.path().join("publish.sh");
    fs::write(
        &script_path,
        extract_run_block_from(CALLER_WORKFLOW, PUBLISH_STEP),
    )
    .expect("write publish script");

    let mut command = runner_command(&script_path, temp.path());
    command
        .env("PATH", path_with_stub(&bin))
        .env("STUB_BODY", &body_path)
        .env("STUB_CALLS", &calls_path)
        .env("STUB_COMMENTS", temp.path().join("comments-fixture.json"));
    add_step_env_from(
        &mut command,
        CALLER_WORKFLOW,
        PUBLISH_STEP,
        &[
            ("${{ github.token }}", "stub-token"),
            ("${{ github.repository }}", "owner/repo"),
            (
                "${{ needs.open-pr.outputs.pull_request_number }}",
                "211",
            ),
            ("${{ inputs.agent_id }}", "codex"),
            ("${{ inputs.target_version }}", "1.2.3"),
            ("${{ github.run_id }}", run_id),
            ("${{ github.run_attempt }}", "1"),
            (
                "${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}/attempts/${{ github.run_attempt }}",
                "https://example.invalid/run",
            ),
            ("${{ needs.acquire.result }}", "failure"),
        ],
    );
    let output = command.output().expect("run publish script");
    assert!(
        output.status.success(),
        "publish step failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    PublishRun {
        body: fs::read_to_string(&body_path).unwrap_or_default(),
        calls: fs::read_to_string(&calls_path).unwrap_or_default(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        _temp: temp,
    }
}

fn verdict_json(audit: &str, extra: &str) -> String {
    format!(
        r#"{{"schema_version":1,
            "packet":{{"agent_id":"codex","target_version":"1.2.3","commit_mode":true}},
            "producer":{{"run_id":"7","run_attempt":1}},
            "audit":{audit},
            "delivery":{{"attempted":true,"committed":true,"head_sha":"deadbee"}}
            {extra}}}"#
    )
}

#[test]
fn a_missing_verdict_is_reported_as_unavailable_and_never_as_clean() {
    let run = run_publisher(None, "[]", "7");
    assert!(
        run.body.contains("**unavailable**") && run.body.contains("not a clean audit"),
        "a missing verdict must say so in both directions: {}",
        run.body
    );
    assert!(
        run.body.contains("`failure`"),
        "the producing job's own result is the only other fact available: {}",
        run.body
    );
    assert!(!run.body.contains("execute-agent-maintenance"));
}

#[test]
fn an_uplift_verdict_carries_the_relay_and_a_blocking_one_does_not() {
    let uplifts = run_publisher(
        Some(&verdict_json(
            r#"{"observed":true,"exit_code":3,"blocking":false,"closeout_ready":false,
                "uplifts_required":true,"title":"","message":""}"#,
            r#","relay":{"dry_run":"cargo run -p xtask -- execute-agent-maintenance --dry-run --request R",
                        "write":"cargo run -p xtask -- execute-agent-maintenance --write --request R --run-id RUN_ID_FROM_DRY_RUN"}"#,
        )),
        "[]",
        "7",
    );
    assert!(uplifts.body.contains("Uplifts are required"));
    assert!(uplifts.body.contains("--dry-run --request R"));
    assert!(uplifts.body.contains("RUN_ID_FROM_DRY_RUN"));

    // Exit 2 writes no projection, so the message is the only carrier for the offending rows —
    // and a pasteable relay command here would be an instruction to do the wrong thing. The
    // fixture carries a relay block the producer would not have written, because a negative
    // assertion proves nothing when the fixture makes the positive case impossible.
    let blocking = run_publisher(
        Some(&verdict_json(
            r#"{"observed":true,"exit_code":2,"blocking":true,"closeout_ready":false,
                "uplifts_required":false,"title":"Maintenance audit failed",
                "message":"debt row `codex exec` matches no live gap"}"#,
            r#","relay":{"dry_run":"cargo run -p xtask -- execute-agent-maintenance --dry-run --request R",
                        "write":"cargo run -p xtask -- execute-agent-maintenance --write --request R --run-id RUN_ID_FROM_DRY_RUN"}"#,
        )),
        "[]",
        "7",
    );
    assert!(blocking.body.contains("Maintenance audit failed"));
    assert!(blocking
        .body
        .contains("debt row `codex exec` matches no live gap"));
    assert!(
        !blocking.body.contains("execute-agent-maintenance"),
        "a blocking verdict must not render a relay invocation: {}",
        blocking.body
    );
}

#[test]
fn an_unobserved_audit_is_not_reported_as_a_clean_one() {
    let run = run_publisher(Some(&verdict_json(r#"{"observed":false}"#, "")), "[]", "7");
    assert!(run.body.contains("**did not run**"), "{}", run.body);
    assert!(!run.body.contains("Audit clean"));
}

#[test]
fn a_newer_run_is_never_overwritten_by_an_older_one() {
    let existing = r#"[{"id":99,"body":"<!-- maintenance-audit-verdict agent=codex version=1.2.3 run=900 attempt=1 -->\nold"}]"#;
    // Completion order is not generation order. Run 7 finishing after run 900 must stand down
    // rather than replace a current verdict with its own valid but superseded one.
    let stale = run_publisher(
        Some(&verdict_json(
            r#"{"observed":true,"exit_code":0,"blocking":false,"closeout_ready":true,"uplifts_required":false,"title":"","message":""}"#,
            "",
        )),
        existing,
        "7",
    );
    assert!(
        stale.stdout.contains("Superseded audit verdict"),
        "an older run must stand down: {}",
        stale.stdout
    );
    assert!(
        !stale.calls.contains("PATCH") && !stale.calls.contains("POST"),
        "a superseded run must write nothing: {}",
        stale.calls
    );

    // A newer run updates the one managed comment in place rather than appending another.
    let fresh = run_publisher(
        Some(&verdict_json(
            r#"{"observed":true,"exit_code":0,"blocking":false,"closeout_ready":true,"uplifts_required":false,"title":"","message":""}"#,
            "",
        )),
        existing,
        "1000",
    );
    assert!(
        fresh
            .calls
            .contains("PATCH repos/owner/repo/issues/comments/99"),
        "a newer run must edit the managed comment, not add a second: {}",
        fresh.calls
    );
}
