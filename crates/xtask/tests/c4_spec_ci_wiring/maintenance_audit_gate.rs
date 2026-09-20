use super::*;

// Wiring guards for the maintenance audit gate inside `parity-acquire`: that it runs, what it is
// told, how its numeric exits are routed, and that a blocking verdict fails the job only after the
// commit and the artifact upload. The gate's runtime behaviour is exercised separately, by the
// extracted-step harness in `c4_spec_audit_gate_behavior.rs`.

#[test]
fn c4_spec_reusable_acquisition_routes_maintenance_audit_gate_by_numeric_exit_code() {
    let workflow = ".github/workflows/parity-acquire.yml";
    let yml = read_repo_file(workflow);
    let union_job_header = section_between(&yml, "  union:", "    steps:", workflow);
    let gate_section = section_between(
        &yml,
        "- name: Maintenance audit gate",
        "- name: Summarize the acquired union",
        workflow,
    );
    let upload_section = section_between(
        &yml,
        "- name: Upload the committed artifact bundle",
        "- name: Fail the job if the maintenance audit recorded a blocking verdict",
        workflow,
    );
    let final_failure_section = section_from(
        &yml,
        "- name: Fail the job if the maintenance audit recorded a blocking verdict",
        workflow,
    );

    for required in [
        "closeout_ready: ${{ steps.maintenance_audit.outputs.closeout_ready }}",
        "uplifts_required: ${{ steps.maintenance_audit.outputs.uplifts_required }}",
    ] {
        assert!(
            union_job_header.contains(required),
            "parity-acquire must retain maintenance audit union outputs: {required}"
        );
    }

    for required in [
        "cargo run -p xtask -- maintenance-audit-status",
        "--expect-target-version \"$VERSION\"",
        "docs/agents/lifecycle/${AGENT_ID}-maintenance/governance/maintenance-request.toml",
        "emit_gate_outputs()",
        "audit_failed",
        "audit_exit_code",
        "case \"$AUDIT_STATUS\" in",
    ] {
        assert!(
            gate_section.contains(required),
            "parity-acquire must retain maintenance audit wiring: {required}"
        );
    }

    assert_text_order(
        &yml,
        "Union → wrapper coverage → report → version metadata → validate",
        "- name: Maintenance audit gate",
        workflow,
    );
    assert_text_order(
        &yml,
        "- name: Maintenance audit gate",
        "Commit the acquired artifacts onto the packet branch",
        workflow,
    );
    assert_text_order(
        &yml,
        "Commit the acquired artifacts onto the packet branch",
        "Upload the committed artifact bundle",
        workflow,
    );
    assert_text_order(
        &yml,
        "Upload the committed artifact bundle",
        "Fail the job if the maintenance audit recorded a blocking verdict",
        workflow,
    );

    let uplifts_branch = section_between(
        gate_section,
        &format!("\n            {})", EXIT_UPLIFTS_REQUIRED),
        &format!("\n            {})", EXIT_INCOMPLETE_ACQUISITION),
        workflow,
    );
    assert!(
        uplifts_branch.contains("emit_gate_outputs \"false\" \"true\" \"false\" \"3\""),
        "exit {EXIT_UPLIFTS_REQUIRED} must mark the run as not closeout-ready with uplifts required"
    );
    assert!(
        gate_section.contains("Relay invocation placeholder (T3)")
            && uplifts_branch.contains("summarize_uplifts"),
        "exit {EXIT_UPLIFTS_REQUIRED} must leave a clearly marked relay placeholder for T3"
    );
    // Guarantees exit 3 stays advisory: the branch may record outputs and summaries, but it must
    // not terminate the shell or use a bare `false` statement to make the step fail.
    assert!(
        nonfatal_shell_branch_violations(uplifts_branch).is_empty(),
        "exit {EXIT_UPLIFTS_REQUIRED} is advisory and must not contain failing shell statements"
    );
    let scratch_uplifts_branch = format!("{uplifts_branch}\n              exit 1\n");
    let scratch_violations = nonfatal_shell_branch_violations(&scratch_uplifts_branch);
    assert!(
        scratch_violations.iter().any(|line| line.contains("exit")),
        "the advisory-branch guard must fail when a scratch copy inserts `exit 1`: {:?}",
        scratch_violations
    );

    let incomplete_branch = section_between(
        gate_section,
        &format!("\n            {})", EXIT_INCOMPLETE_ACQUISITION),
        "\n            *)",
        workflow,
    );
    assert!(
        incomplete_branch.contains("::error title=Incomplete acquisition::")
            && incomplete_branch.contains("missing targets")
            && incomplete_branch.contains("emit_gate_outputs \"false\" \"false\" \"true\""),
        "exit {EXIT_INCOMPLETE_ACQUISITION} must record a blocking verdict with a missing-targets error"
    );
    assert!(
        nonfatal_shell_branch_violations(incomplete_branch).is_empty(),
        "the gate step itself must not fail on exit {EXIT_INCOMPLETE_ACQUISITION}; failure happens in the final step"
    );

    let fallback_branch = section_between(
        gate_section,
        "\n            *)",
        "\n          esac",
        workflow,
    );
    assert!(
        fallback_branch.contains("::error title=Maintenance audit failed::")
            && fallback_branch.contains("emit_gate_outputs \"false\" \"false\" \"true\""),
        "fallback maintenance audit exits must record a blocking verdict"
    );
    assert!(
        nonfatal_shell_branch_violations(fallback_branch).is_empty(),
        "the gate step itself must not fail on non-advisory maintenance audit exits"
    );

    assert!(
        upload_section.contains("if: ${{ always() }}"),
        "the committed artifact bundle must upload even when a later step fails"
    );

    assert!(
        final_failure_section.contains(
            "if: ${{ always() && steps.maintenance_audit.outputs.audit_failed == 'true' }}"
        ) && final_failure_section.contains("exit \"$AUDIT_EXIT_CODE\""),
        "a recorded blocking verdict must fail the job only in the final post-upload step"
    );
}

#[test]
fn c4_spec_maintenance_audit_gate_passes_expected_target_version_to_xtask() {
    let workflow = ".github/workflows/parity-acquire.yml";
    let yml = read_repo_file(workflow);
    let gate_section = section_between(
        &yml,
        "- name: Maintenance audit gate",
        "- name: Summarize the acquired union",
        workflow,
    );

    assert!(
        gate_section.contains("--expect-target-version \"$VERSION\""),
        "the maintenance audit gate must pass the acquired VERSION to maintenance-audit-status"
    );
}

#[test]
fn c4_spec_maintenance_audit_uplifts_branch_is_nonfatal_and_detects_scratch_exit() {
    let workflow = ".github/workflows/parity-acquire.yml";
    let yml = read_repo_file(workflow);
    let gate_section = section_between(
        &yml,
        "- name: Maintenance audit gate",
        "- name: Summarize the acquired union",
        workflow,
    );
    let uplifts_branch = section_between(
        gate_section,
        &format!("\n            {})", EXIT_UPLIFTS_REQUIRED),
        &format!("\n            {})", EXIT_INCOMPLETE_ACQUISITION),
        workflow,
    );

    assert!(
        uplifts_branch.contains("emit_gate_outputs \"false\" \"true\" \"false\" \"3\"")
            && uplifts_branch.contains("summarize_uplifts"),
        "the advisory exit-3 branch must record its verdict without in-branch failure control flow"
    );
    assert!(
        nonfatal_shell_branch_violations(uplifts_branch).is_empty(),
        "the advisory exit-3 branch must not contain `exit` or bare `false` statements"
    );

    let scratch_uplifts_branch = format!("{uplifts_branch}\n              exit 1\n");
    let scratch_violations = nonfatal_shell_branch_violations(&scratch_uplifts_branch);
    assert!(
        scratch_violations.iter().any(|line| line.contains("exit")),
        "the advisory-branch guard must fail when a scratch copy inserts `exit 1`: {:?}",
        scratch_violations
    );
}

#[test]
fn c4_spec_maintenance_audit_artifact_bundle_upload_is_always() {
    let workflow = ".github/workflows/parity-acquire.yml";
    let yml = read_repo_file(workflow);
    let upload_section = section_between(
        &yml,
        "- name: Upload the committed artifact bundle",
        "- name: Fail the job if the maintenance audit recorded a blocking verdict",
        workflow,
    );

    assert!(
        upload_section.contains("if: ${{ always() }}"),
        "the committed artifact bundle upload must use if: always()"
    );
}

#[test]
fn c4_spec_maintenance_audit_blocking_verdict_fails_only_after_commit_and_upload() {
    let workflow = ".github/workflows/parity-acquire.yml";
    let yml = read_repo_file(workflow);
    let final_failure_section = section_from(
        &yml,
        "- name: Fail the job if the maintenance audit recorded a blocking verdict",
        workflow,
    );

    assert_text_order(
        &yml,
        "Commit the acquired artifacts onto the packet branch",
        "Upload the committed artifact bundle",
        workflow,
    );
    assert_text_order(
        &yml,
        "Upload the committed artifact bundle",
        "Fail the job if the maintenance audit recorded a blocking verdict",
        workflow,
    );
    assert!(
        final_failure_section.contains(
            "if: ${{ always() && steps.maintenance_audit.outputs.audit_failed == 'true' }}"
        ) && final_failure_section.contains("exit \"$AUDIT_EXIT_CODE\""),
        "a blocking maintenance audit verdict must fail the job only in the final post-upload step"
    );
}
