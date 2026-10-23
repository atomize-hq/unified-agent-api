use super::*;

const WORKFLOW: &str = ".github/workflows/parity-acquire.yml";
const REFREEZE: &str = "- name: Re-freeze the maintenance request against the acquired artifacts";

#[test]
fn c4_spec_acquisition_refreezes_between_union_and_audit() {
    let yml = read_repo_file(WORKFLOW);
    assert_text_order(
        &yml,
        "Union → wrapper coverage → report → version metadata → validate",
        REFREEZE,
        WORKFLOW,
    );
    assert_text_order(&yml, REFREEZE, "- name: Maintenance audit gate", WORKFLOW);

    let step = section_between(&yml, REFREEZE, "- name: Maintenance audit gate", WORKFLOW);
    assert!(
        step.contains("id: refreeze") && step.contains("if: ${{ inputs.commit }}"),
        "request re-freeze must be commit-only and addressable"
    );
    for (first, second) in [
        (
            "git fetch --depth=1 --no-tags origin staging",
            "maintenance-stand-down-check",
        ),
        (
            "maintenance-stand-down-check",
            "prepare-agent-maintenance --from-request",
        ),
    ] {
        assert_text_order(step, first, second, WORKFLOW);
    }
    assert!(
        step.contains("--from-ref FETCH_HEAD"),
        "stand-down must read the freshly fetched staging base"
    );
    assert!(
        step.contains("REF: ${{ needs.plan.outputs.branch_name }}")
            && step.contains("EXPECTED_REF=\"automation/${AGENT_ID}-maintenance-${VERSION}\""),
        "the re-freeze must bind either admitted ref spelling to this packet generation"
    );

    for command in [
        "maintenance-stand-down-check",
        "prepare-agent-maintenance --from-request",
    ] {
        assert_text_order(
            step,
            "if [ \"$REF\" != \"$EXPECTED_REF\" ]",
            command,
            WORKFLOW,
        );
    }
    let ref_mismatch = section_between(
        step,
        "if [ \"$REF\" != \"$EXPECTED_REF\" ]; then",
        "\n          fi",
        WORKFLOW,
    );
    assert!(
        ref_mismatch.contains("::notice title=Re-freeze skipped::")
            && ref_mismatch.contains("refrozen=false")
            && ref_mismatch.contains("exit 0")
            && !ref_mismatch.contains("prepare-agent-maintenance"),
        "a dispatch for another packet branch must exit cleanly without re-freezing"
    );

    let stood_down = section_between(step, "\n            3)", "\n            *)", WORKFLOW);
    assert!(
        stood_down.contains("refrozen=false") && stood_down.contains("exit 0"),
        "exit 3 must skip re-freeze cleanly"
    );
    let guard_failure = section_between(step, "\n            *)", "\n          esac", WORKFLOW);
    assert!(
        guard_failure.contains("::error") && guard_failure.contains("exit 1"),
        "every other non-zero guard status must fail closed"
    );
}

#[test]
fn c4_spec_acquisition_commit_and_bundle_use_the_same_checked_delivery() {
    let yml = read_repo_file(WORKFLOW);
    let stage = section_between(
        &yml,
        "- name: Stage the complete checked delivery",
        "- name: Maintenance audit gate",
        WORKFLOW,
    );
    assert!(stage.contains("id: delivery"));
    assert!(stage.contains("stage-delivery --plan _ci_tmp/acquisition-plan.json"));
    assert!(stage.contains("--input-state \"$TRANSPORT_DIR/inputs.json\""));
    assert!(stage.contains("--preparation \"$PREPARATION_REPORT\""));
    assert!(stage.contains("--bundle-dir \"$TRANSPORT_DIR/bundle\""));
    let commit = section_between(
        &yml,
        "- name: Commit the acquired artifacts onto the packet branch",
        "- name: Upload the committed artifact bundle",
        WORKFLOW,
    );
    assert!(commit.contains("verify --state \"$TRANSPORT_DIR/delivery.json\""));
    assert!(
        !commit.contains("git add"),
        "commit cannot recapture or broaden the checked path set"
    );
    let upload = section_between(
        &yml,
        "- name: Upload the committed artifact bundle",
        "- name: Record the maintenance audit verdict",
        WORKFLOW,
    );
    assert!(upload.contains("path: ${{ runner.temp }}/maintenance-transport/bundle/"));
    assert!(upload.contains("always() && steps.delivery.outcome == 'success'"));
}
