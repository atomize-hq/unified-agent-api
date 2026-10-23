use super::{transport::*, *};
use crate::{
    agent_registry::support_depth::ObligationId,
    depth_ci::{self, Witness},
    runtime_advertising::source::Source,
};
use std::process::Command;

pub(super) const WORKFLOW: &str = ".github/workflows/ci.yml";
pub(super) const EXECUTOR: &str = "github-actions:rust-tests/isolated-channel-v1";
pub(super) const TRUST: &str = "github-actions-channel-v1";
pub(super) const REVIEW: &str = "maintainer-mapping-v1";

pub(super) fn original(source: &Source) -> Result<DepthRecord, String> {
    let record = depth_ci::record(&source.read(depth_ci::FIXTURE)?)?;
    if record.execution.is_some() || !record.acceptance_entries.is_empty() {
        return Err("fixture is not frozen P/O".into());
    }
    Ok(record)
}
pub(super) fn scope(record: &DepthRecord) -> Scope {
    Scope {
        agent_id: "codex".into(),
        version: "0.0.0".into(),
        event: record.event.clone(),
        enrollment_origin: EnrollmentOrigin::Isolated,
        obligation: ObligationRef {
            operation: "controlled_channel".into(),
            id: Some(ObligationId::V1),
            capability: None,
            target: depth_ci::TARGET.into(),
        },
        policy_identity: record.policy.identity.clone(),
        obligations_identity: record.obligations.as_ref().unwrap().identity.clone(),
        rules_revision: record.policy.rules_revision.clone(),
        trust_revision: TRUST.into(),
        review_revision: REVIEW.into(),
    }
}
/// Whole compiled producer and workflow closure. This narrow profile refuses changes until the
/// implementation is reviewed on protected staging. A candidate never approves its own hash.
pub(super) fn approved(
    root: &Path,
    built: &str,
    workflow: &str,
    anchor: &str,
) -> Result<(), String> {
    for commit in [built, workflow, anchor] {
        source(root, commit)?;
    }
    for path in [
        "crates",
        "Cargo.toml",
        "Cargo.lock",
        WORKFLOW,
        depth_ci::ARCHIVE,
    ] {
        let oid = |commit: &str| -> Result<Vec<u8>, String> {
            crate::depth_ci::output(
                Command::new("git")
                    .arg("--no-replace-objects")
                    .arg("-C")
                    .arg(root)
                    .args([
                        "rev-parse",
                        "--verify",
                        "--end-of-options",
                        &format!("{commit}:{path}"),
                    ]),
            )
        };
        if oid(built)? != oid(anchor)? || oid(workflow)? != oid(anchor)? {
            return Err(format!(
                "producer closure is not approved on protected staging: {path}"
            ));
        }
    }
    Ok(())
}
pub(super) fn validate_witness(
    root: &Path,
    w: &Witness,
    run: &Value,
    attempt: u64,
    anchor: &str,
) -> Result<DepthRecord, String> {
    if w.schema_version != 1
        || w.run_id != run["id"].as_u64().ok_or("missing host run")?
        || w.run_attempt != attempt
    {
        return Err("witness run/attempt mismatch".into());
    }
    let head = sha(required_string(run, "head_sha")?)?;
    let built = source(root, &w.build.source_commit)?;
    // PR workflow checkout is GitHub's two-parent test merge, while head_sha names the PR head.
    // Other supported events build their exact host head. No arbitrary descendant qualifies.
    if w.build.source_commit != head {
        if run["event"] != "pull_request" {
            return Err("checkout does not match host head".into());
        }
        let parents = crate::depth_ci::output(
            Command::new("git")
                .arg("--no-replace-objects")
                .arg("-C")
                .arg(root)
                .args(["show", "-s", "--format=%P", &w.build.source_commit]),
        )?;
        let parents = std::str::from_utf8(&parents)
            .map_err(|e| e.to_string())?
            .split_whitespace()
            .collect::<Vec<_>>();
        if parents.len() != 2 || parents[1] != head {
            return Err("checkout is not the host PR test merge".into());
        }
    }
    // The workflow itself must be the exact checkout implementation for this fixed CI profile.
    if w.workflow_commit != w.build.source_commit || w.build.source_tree != built.tree()? {
        return Err("workflow/build source mismatch".into());
    }
    approved(root, &w.build.source_commit, &w.workflow_commit, anchor)?;
    let original = original(&built)?;
    let names = w
        .cases
        .requested
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>();
    if names != depth_ci::CASES
        || w.cases.requested.is_empty()
        || w.cases
            .requested
            .iter()
            .any(|c| c.executable_sha256 != w.cases.requested[0].executable_sha256)
    {
        return Err("wrong exact test executable/case population".into());
    }
    w.graph.closure()?;
    let profile: Value = serde_json::from_str(
        &built.read("crates/xtask/tests/fixtures/depth-ci/build-profile.json")?,
    )
    .map_err(|e| e.to_string())?;
    if serde_json::to_value(&w.graph.packages).map_err(|e| e.to_string())? != profile["packages"]
        || w.graph.inputs != w.build.covered_inputs
        || w.build.host_target != depth_ci::TARGET
        || w.environment.target != depth_ci::TARGET
        || !w.build.features.is_empty()
        || !w.environment.features.is_empty()
    {
        return Err("witness build graph/environment disagrees with reviewed profile".into());
    }
    for (reference, bytes) in payloads(w)? {
        if depth_ci::reference(&reference.path, &bytes) != reference {
            return Err("witness payload digest mismatch".into());
        }
    }
    Ok(original)
}
pub(super) fn payloads(w: &Witness) -> Result<Vec<(FileRef, Vec<u8>)>, String> {
    Ok(vec![
        (
            w.build.toolchain.clone(),
            w.toolchain_report.as_bytes().to_vec(),
        ),
        (
            w.build.dependency_graph.clone(),
            serde_json::to_vec(&w.graph).map_err(|e| e.to_string())?,
        ),
        (
            w.build.environment.clone(),
            serde_json::to_vec(&w.environment).map_err(|e| e.to_string())?,
        ),
        (w.cases.raw_output.clone(), w.raw_output.as_bytes().to_vec()),
    ])
}
pub(super) fn mapping(w: &Witness, reviewer: &str) -> Result<AdequacyMapping, String> {
    let input = |path| {
        w.graph
            .inputs
            .iter()
            .find(|f| f.path == path)
            .cloned()
            .ok_or("missing assertion/production graph input")
    };
    Ok(AdequacyMapping{obligation:scope(&DepthRecord::parse(include_str!("../../../../tests/fixtures/depth-ci/record.json"),"codex","0.0.0")?).obligation,reviewer_identity:reviewer.into(),executor_identity:EXECUTOR.into(),assertions:w.cases.requested.iter().map(|case| Ok(AssertionBinding{case:case.clone(),assertion:input(depth_ci::ASSERTION)?,production_inputs:vec![input(depth_ci::PRODUCTION)?],counterfactual:"If the production validator accepts a forbidden value, rejects an allowed value or changes an accepted value, the corresponding bound grammar assertion fails. Redundant implementation guards need not independently fail.".into()})).collect::<Result<_,String>>()?})
}
