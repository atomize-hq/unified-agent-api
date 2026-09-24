use std::collections::BTreeSet;

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
        step.contains("REF: ${{ inputs.ref }}")
            && step.contains("EXPECTED_REF=\"automation/${AGENT_ID}-maintenance-${VERSION}\""),
        "the re-freeze must bind the dispatch ref to this packet generation"
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
fn c4_spec_acquisition_commit_and_bundle_name_the_same_paths() {
    let yml = read_repo_file(WORKFLOW);
    let commit = section_between(
        &yml,
        "- name: Commit the acquired artifacts onto the packet branch",
        "- name: Upload the committed artifact bundle",
        WORKFLOW,
    );
    assert!(
        commit.contains("\"docs/agents/lifecycle/${AGENT_ID}-maintenance\""),
        "the commit must stage regenerated request and packet docs"
    );

    let upload = section_between(
        &yml,
        "- name: Upload the committed artifact bundle",
        "- name: Record the maintenance audit verdict",
        WORKFLOW,
    );
    let committed = git_add_paths(commit);
    let bundled = upload_paths(upload);
    assert_eq!(
        bundled, committed,
        "the committed artifact bundle must name exactly the staged path set"
    );
}

fn git_add_paths(step: &str) -> BTreeSet<String> {
    let mut paths = Vec::new();
    let mut in_paths = false;
    for line in step.lines() {
        let trimmed = line.trim();
        if trimmed == "PATHS=(" {
            in_paths = true;
            continue;
        }
        if in_paths {
            if trimmed == ")" {
                in_paths = false;
            } else {
                paths.push(trimmed);
            }
            continue;
        }
        if let Some(path) = trimmed
            .strip_prefix("PATHS+=(")
            .and_then(|value| value.strip_suffix(')'))
        {
            paths.push(path);
        }
    }
    assert!(
        step.contains("git add \"${PATHS[@]}\""),
        "git add must stage PATHS"
    );
    assert!(!paths.is_empty(), "PATHS array");
    paths.into_iter().map(normalize_path).collect()
}

fn upload_paths(step: &str) -> BTreeSet<String> {
    step.lines()
        .skip_while(|line| line.trim() != "path: |")
        .skip(1)
        .take_while(|line| line.starts_with("            ") && !line.trim().is_empty())
        .map(|line| normalize_path(line.trim()))
        .collect()
}

fn normalize_path(path: &str) -> String {
    path.trim_end_matches('\\')
        .trim()
        .trim_matches('"')
        .replace("${{ needs.plan.outputs.manifest_root }}", "${ROOT}")
        .replace("${{ inputs.target_version }}", "${VERSION}")
        .replace("${{ inputs.agent_id }}", "${AGENT_ID}")
        .trim_end_matches('/')
        .to_string()
}
