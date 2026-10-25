use super::{transport::*, *};
use crate::depth_ci::profile::Profile;
use crate::{
    depth_ci::{self, Witness},
    runtime_advertising::source::Source,
};
use std::process::Command;

pub(super) const OPTIONAL: [&str; 4] = [
    ".cargo/config",
    ".cargo/config.toml",
    "rust-toolchain",
    "rust-toolchain.toml",
];
pub(super) const WORKFLOW: &str = ".github/workflows/ci.yml";
pub(super) const EXECUTOR: &str = "github-actions:rust-tests/isolated-channel-v1";
pub(super) const REVIEW: &str = "maintainer-mapping-v1";

pub(super) fn original_for(source: &Source, p: Profile) -> Result<DepthRecord, String> {
    let record = p.record(&source.read(p.fixture())?)?;
    if record.execution.is_some() || !record.acceptance_entries.is_empty() {
        return Err("fixture is not frozen P/O".into());
    }
    for input in &record
        .obligations
        .as_ref()
        .ok_or("missing frozen obligations")?
        .acquired_inputs
    {
        if depth_ci::reference(
            &input.path,
            &source
                .optional_bytes(&input.path)?
                .ok_or("missing acquired profile input")?,
        ) != *input
        {
            return Err("acquired profile provenance differs from frozen O".into());
        }
    }
    Ok(record)
}
pub(super) fn scope(record: &DepthRecord) -> Scope {
    scope_for(
        record,
        Profile::Channel,
        Profile::Channel.obligations(record).remove(0),
    )
}
pub(super) fn scope_for(record: &DepthRecord, p: Profile, obligation: ObligationRef) -> Scope {
    Scope {
        agent_id: "codex".into(),
        version: p.version().into(),
        event: record.event.clone(),
        enrollment_origin: EnrollmentOrigin::Isolated,
        obligation,
        policy_identity: record.policy.identity.clone(),
        obligations_identity: record.obligations.as_ref().unwrap().identity.clone(),
        rules_revision: record.policy.rules_revision.clone(),
        trust_revision: p.trust().into(),
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
    p: Profile,
) -> Result<(), String> {
    for commit in [built, workflow, anchor] {
        let source = source(root, commit)?;
        for path in OPTIONAL {
            if source.optional_bytes(path)?.is_some() {
                return Err(
                    "initial producer requires absent root Cargo/toolchain configuration".into(),
                );
            }
        }
    }
    for path in [
        "crates",
        "Cargo.toml",
        "Cargo.lock",
        WORKFLOW,
        depth_ci::ARCHIVE,
    ]
    .into_iter()
    .chain(p.requirements())
    {
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
    p: Profile,
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
    approved(root, &w.build.source_commit, &w.workflow_commit, anchor, p)?;
    let original = original_for(&built, p)?;
    let names = w
        .cases
        .requested
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>();
    let raw = built.optional(&format!("{}/cases.json", p.directory()))?;
    let expected = p.case_names(raw.as_deref())?;
    if names != expected
        || w.cases.requested.is_empty()
        || w.cases
            .requested
            .iter()
            .any(|c| c.executable_sha256 != w.cases.requested[0].executable_sha256)
    {
        return Err("wrong exact test executable/case population".into());
    }
    w.graph.closure()?;
    if let Some(scope) = p.requirements() {
        let prefix = format!("{scope}/");
        let claimed: std::collections::BTreeSet<_> = w
            .graph
            .inputs
            .iter()
            .filter(|f| f.path.starts_with(&prefix))
            .map(|f| f.path.clone())
            .collect();
        let expected = built.committed_files(scope)?.into_iter().collect();
        if !w.graph.scopes.iter().any(|s| s == scope) || claimed != expected {
            return Err("CORE witness lacks complete canonical requirement scope".into());
        }
    }
    let profile: Value =
        serde_json::from_str(&built.read(&p.build_profile())?).map_err(|e| e.to_string())?;
    if serde_json::to_value(&w.graph.packages).map_err(|e| e.to_string())? != profile["packages"]
        || w.graph.inputs != w.build.covered_inputs
        || w.build.host_target != depth_ci::TARGET
        || w.environment.target != depth_ci::TARGET
        || w.build.features != p.features()
        || w.environment.features != p.features()
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

pub(super) fn mapping_for(
    root: &Path,
    w: &Witness,
    reviewer: &str,
    p: Profile,
    obligation: &ObligationRef,
) -> Result<AdequacyMapping, String> {
    if p == Profile::Channel {
        return mapping(w, reviewer);
    }
    let built = source(root, &w.build.source_commit)?;
    let specs: std::collections::BTreeMap<String, Vec<crate::depth_ci::profile::CaseSpec>> =
        serde_json::from_str(&built.read(&format!("{}/cases.json", p.directory()))?)
            .map_err(|e| e.to_string())?;
    let key = obligation
        .id
        .map(|id| {
            serde_json::to_value(id)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .unwrap_or_else(|| "M".into());
    let input = |path: &str| {
        w.graph
            .inputs
            .iter()
            .find(|f| f.path == path)
            .cloned()
            .ok_or_else(|| format!("missing mapped input {path}"))
    };
    let assertions = specs
        .get(&key)
        .ok_or("missing due obligation mapping")?
        .iter()
        .map(|spec| {
            Ok(AssertionBinding {
                case: w
                    .cases
                    .requested
                    .iter()
                    .find(|c| c.name == spec.name)
                    .cloned()
                    .ok_or("mapped case absent from collected population")?,
                assertion: input(&spec.assertion)?,
                production_inputs: spec
                    .production_inputs
                    .iter()
                    .map(|s| input(s))
                    .collect::<Result<_, String>>()?,
                counterfactual: spec.counterfactual.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    if assertions.is_empty() {
        return Err("empty obligation mapping".into());
    }
    Ok(AdequacyMapping {
        obligation: obligation.clone(),
        reviewer_identity: reviewer.into(),
        executor_identity: p.executor().into(),
        assertions,
    })
}
