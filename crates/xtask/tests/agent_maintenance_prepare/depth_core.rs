use super::*;
use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::DepthRecord,
    support_matrix,
};
use contract_policy::{dispatch_kind_str, dispatch_workflow_value, opened_from_path};
use prepare::{
    isolated_apply, isolated_build as build_with_registry, Error, PreparePlan, PreparedFile,
};
use serde_json::json;
use std::{fs, path::PathBuf, process::Command};
#[path = "depth_core/fixture.rs"]
mod fixture;

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}
fn copy(source: &Path, destination: &Path) {
    if source.is_dir() {
        fs::create_dir_all(destination).unwrap();
        for e in fs::read_dir(source).unwrap() {
            let e = e.unwrap();
            copy(&e.path(), &destination.join(e.file_name()));
        }
    } else {
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source, destination).unwrap();
    }
}
fn workspace(agent: &str) -> (tempfile::TempDir, AgentRegistry, Args) {
    let root = fixture::workspace(agent);
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in [
        "Cargo.toml",
        "crates/agent_api/src",
        "crates/agent_api/Cargo.toml",
        "docs/specs/unified-agent-api/capability-matrix.md",
    ] {
        copy(&repo.join(path), &root.path().join(path));
    }
    let registry = AgentRegistry::parse_unenforced(
        &fs::read_to_string(root.path().join(REGISTRY_RELATIVE_PATH)).unwrap(),
    )
    .unwrap();
    let mut projection = Vec::new();
    for entry in registry.support_matrix_entries() {
        let base = &entry.manifest_root;
        let rules: serde_json::Value = fs::read(repo.join(base).join("RULES.json"))
            .ok()
            .map(|raw| serde_json::from_slice(&raw).unwrap())
            .unwrap_or_else(|| json!({"union":{"expected_targets":entry.canonical_targets}}));
        fixture::write(
            root.path(),
            &format!("{base}/RULES.json"),
            serde_json::to_vec(&rules).unwrap(),
        );
        let targets = rules["union"]["expected_targets"].as_array().unwrap();
        fixture::write(root.path(), &format!("{base}/current.json"), json!({"snapshot_schema_version":2,"mode":"union","expected_targets":targets,"inputs":[]}).to_string());
        fixture::write(
            root.path(),
            &format!("{base}/latest_validated.txt"),
            "0.0.1\n",
        );
        fs::create_dir_all(root.path().join(base).join("versions")).unwrap();
        for target in targets {
            let target = target.as_str().unwrap();
            for kind in ["latest_supported", "latest_validated"] {
                fixture::write(
                    root.path(),
                    &format!("{base}/pointers/{kind}/{target}.txt"),
                    "none\n",
                );
            }
            if entry.agent_id == "codex" {
                projection.push(support_matrix::RuntimeSupportProjectionRecord {
                    runtime_family: "codex".into(),
                    target_triple: target.into(),
                    latest_validated: None,
                });
            }
        }
    }
    projection.sort_by(|a, b| a.target_triple.cmp(&b.target_triple));
    let bundle=support_matrix::render_publication_artifacts("Normative introduction.\n\n<!-- support-matrix-published:start -->\n<!-- support-matrix-published:end -->\n", &[], support_matrix::render_agent_api_runtime_support_data_from_projection(&projection, &["codex"])).unwrap();
    for (path, bytes) in [
        (support_matrix::JSON_OUTPUT_PATH, bundle.json),
        (support_matrix::MARKDOWN_OUTPUT_PATH, bundle.markdown),
        (
            support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            bundle.runtime_support_data,
        ),
    ] {
        fixture::write(root.path(), path, bytes);
    }
    git(root.path(), &["add", "."]);
    let entry = registry.find(agent).unwrap();
    let release = entry.maintenance.release_watch.as_ref().unwrap();
    let workflow = dispatch_workflow_value(agent, release).unwrap();
    let args = Args {
        agent: agent.into(),
        current_version: "0.0.1".into(),
        latest_stable: "1.2.3".into(),
        target_version: "1.2.3".into(),
        opened_from: PathBuf::from(opened_from_path(&workflow)),
        detected_by: "fixture".into(),
        dispatch_kind: dispatch_kind_str(release.dispatch_kind).into(),
        dispatch_workflow: Some(workflow),
        branch_name: format!("automation/{agent}-maintenance-1.2.3"),
        request_recorded_at: "2026-10-22T00:00:00Z".into(),
        request_commit: "abcdef0".into(),
        dry_run: false,
        write: true,
    };
    copy(
        &repo.join(&args.opened_from),
        &root.path().join(&args.opened_from),
    );
    git(root.path(), &["add", "."]);
    (root, registry, args)
}
fn record(plan: &PreparePlan) -> DepthRecord {
    let file = plan
        .files
        .iter()
        .find(|f| f.relative_path.ends_with("/depth-record.json"))
        .unwrap();
    DepthRecord::parse(
        std::str::from_utf8(&file.contents).unwrap(),
        &plan.request.agent_id,
        &plan
            .request
            .detected_release
            .as_ref()
            .unwrap()
            .target_version,
    )
    .unwrap()
}

#[test]
fn private_preparation_plans_p_only_and_coherent_o_for_every_adopted_root() {
    for agent in ["codex", "claude_code", "opencode"] {
        let (root, registry, args) = workspace(agent);
        let before = git(root.path(), &["status", "--porcelain=v1"]);
        let p = build_with_registry(root.path(), &args, &registry).unwrap();
        assert!(record(&p).obligations.is_none());
        assert_eq!(git(root.path(), &["status", "--porcelain=v1"]), before);
        assert!(
            build_prepare_plan(root.path(), &args).is_err(),
            "production dispatch must refuse"
        );
        assert!(
            apply_prepare_plan(root.path(), &p).is_err(),
            "public direct apply must refuse"
        );
        isolated_apply(root.path(), &p, &registry).unwrap();
        git(root.path(), &["add", "."]);
        fixture::acquire(root.path(), agent);
        let o = build_with_registry(root.path(), &args, &registry).unwrap();
        assert!(record(&o).obligations.is_some());
        assert_eq!(record(&o).policy, record(&p).policy);
        for path in [
            support_matrix::JSON_OUTPUT_PATH,
            support_matrix::MARKDOWN_OUTPUT_PATH,
            support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
        ] {
            assert!(o.planned_paths().contains(&path));
        }
        let markdown = &o
            .files
            .iter()
            .find(|f| f.relative_path == support_matrix::MARKDOWN_OUTPUT_PATH)
            .unwrap()
            .contents;
        assert!(std::str::from_utf8(markdown)
            .unwrap()
            .starts_with("Normative introduction."));
        isolated_apply(root.path(), &o, &registry).unwrap();
        git(root.path(), &["add", "."]);
        let again = build_with_registry(root.path(), &args, &registry).unwrap();
        assert_eq!(again.files, o.files);
        assert_eq!(
            isolated_apply(root.path(), &again, &registry)
                .unwrap()
                .written,
            0
        );
    }
}

#[test]
fn refreeze_retains_due_work_entries_and_clears_e_only_when_o_changes() {
    use crate::depth_record::{part_identity, AcceptanceEntry, ExecutionPart};
    let (root, registry, mut args) = workspace("codex");
    fixture::acquire(root.path(), "codex");
    let first = build_with_registry(root.path(), &args, &registry).unwrap();
    let mut prior = record(&first);
    assert!(!prior
        .obligations
        .as_ref()
        .unwrap()
        .acceptance_work
        .is_empty());
    isolated_apply(root.path(), &first, &registry).unwrap();
    let mut e = ExecutionPart {
        identity: String::new(),
        policy_identity: prior.policy.identity.clone(),
        obligations_identity: prior.obligations.as_ref().unwrap().identity.clone(),
        results: vec![],
        reuse_bindings: vec![],
        debt_grants: vec![],
        debt_retired: vec![],
    };
    e.identity = part_identity(&e).unwrap();
    prior.execution = Some(e);
    prior.acceptance_entries.push(AcceptanceEntry::Promotion {
        record_sha256: prior.stated_sha256().unwrap(),
        targets: prior.policy.selection.enrollment.targets.clone(),
    });
    let path = "cli_manifests/codex/reports/1.2.3/depth-record.json";
    fixture::write(
        root.path(),
        path,
        serde_json::to_vec_pretty(&prior).unwrap(),
    );
    git(root.path(), &["add", "."]);
    let stable = build_with_registry(root.path(), &args, &registry).unwrap();
    assert_eq!(record(&stable), prior);
    let wrapper = "cli_manifests/codex/wrapper_coverage.json";
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(root.path().join(wrapper)).unwrap()).unwrap();
    value["coverage"][0]["note"] = json!("changed bound wrapper description");
    value["coverage"][0]["args"][0]["level"] = json!("explicit");
    fixture::write(root.path(), wrapper, value.to_string());
    fixture::regenerate(root.path(), "codex");
    let changed = build_with_registry(root.path(), &args, &registry).unwrap();
    let now = record(&changed);
    assert!(now.execution.is_none());
    assert_eq!(now.event, prior.event);
    assert_eq!(now.policy, prior.policy);
    assert_eq!(now.acceptance_entries, prior.acceptance_entries);
    assert_eq!(
        now.obligations.as_ref().unwrap().acceptance_work,
        prior.obligations.as_ref().unwrap().acceptance_work
    );
    assert_eq!(
        changed
            .request
            .support_surface_audit
            .as_ref()
            .unwrap()
            .required_uplifts_this_run,
        first
            .request
            .support_surface_audit
            .as_ref()
            .unwrap()
            .required_uplifts_this_run
    );
    isolated_apply(root.path(), &changed, &registry).unwrap();
    git(root.path(), &["add", "."]);
    args.request_commit = "fedcba0".into();
    let generation = record(&build_with_registry(root.path(), &args, &registry).unwrap());
    assert_ne!(generation.event, now.event);
    assert_eq!(generation.policy, now.policy);
    assert_eq!(generation.acceptance_entries, now.acceptance_entries);
    assert!(generation.execution.is_none());
}

#[test]
fn depth_apply_refuses_stale_bound_bytes_index_membership_and_publication() {
    for change in ["bytes", "index", "membership", "publication"] {
        let (root, registry, args) = workspace("codex");
        let plan = build_with_registry(root.path(), &args, &registry).unwrap();
        match change {
            "bytes" => {
                let p = root.path().join("cli_manifests/codex/RULES.json");
                let mut bytes = fs::read(&p).unwrap();
                bytes.push(b'\n');
                fs::write(p, bytes).unwrap();
                git(root.path(), &["add", "cli_manifests/codex/RULES.json"]);
            }
            "index" => {
                git(
                    root.path(),
                    &[
                        "update-index",
                        "--chmod=+x",
                        "cli_manifests/codex/RULES.json",
                    ],
                );
            }
            "membership" => {
                fixture::write(
                    root.path(),
                    "cli_manifests/codex/reports/1.2.3/new-evidence.txt",
                    "unrelated new file",
                );
            }
            _ => {
                let p = root.path().join(support_matrix::MARKDOWN_OUTPUT_PATH);
                let mut bytes = fs::read(&p).unwrap();
                bytes.extend_from_slice(b"\nIntervening normative edit.\n");
                fs::write(p, bytes).unwrap();
            }
        }
        assert!(
            isolated_apply(root.path(), &plan, &registry).is_err(),
            "{change}"
        );
        assert!(!root.path().join(&plan.request.relative_path).exists());
        assert!(!root
            .path()
            .join("cli_manifests/codex/reports/1.2.3/depth-record.json")
            .exists());
    }
}

#[test]
fn modified_owned_paths_and_request_are_not_an_apply_authority() {
    let (root, _, args) = workspace("codex");
    let registry_path = root.path().join(REGISTRY_RELATIVE_PATH);
    let mut document: toml_edit::DocumentMut =
        fs::read_to_string(&registry_path).unwrap().parse().unwrap();
    for entry in document["agents"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
    {
        entry.remove("support_depth");
    }
    fs::write(&registry_path, document.to_string()).unwrap();
    let plan = build_prepare_plan(root.path(), &args).unwrap();
    for change in ["path", "extra", "removed", "request"] {
        let mut forged = plan.clone();
        match change {
            "path" => forged.files[0].relative_path = "unowned.txt".into(),
            "extra" => forged.files.push(PreparedFile {
                relative_path: "unowned.txt".into(),
                contents: b"injected".to_vec(),
            }),
            "removed" => {
                forged.files.pop();
            }
            _ => forged.request.request_commit = "fedcba0".into(),
        }
        assert!(
            apply_prepare_plan(root.path(), &forged).is_err(),
            "{change}"
        );
        assert!(!root.path().join("unowned.txt").exists());
        assert!(!root.path().join(&plan.request.relative_path).exists());
    }
}

#[test]
fn a_new_version_displaces_the_request_without_changing_the_old_record() {
    let (root, registry, mut args) = workspace("codex");
    let first = build_with_registry(root.path(), &args, &registry).unwrap();
    isolated_apply(root.path(), &first, &registry).unwrap();
    git(root.path(), &["add", "."]);
    let old_path = root
        .path()
        .join("cli_manifests/codex/reports/1.2.3/depth-record.json");
    let old_bytes = fs::read(&old_path).unwrap();
    let mut depth = registry
        .find("codex")
        .unwrap()
        .support_depth
        .clone()
        .unwrap();
    let mut enrollment = depth.enrollments.as_ref().unwrap()[0].clone();
    enrollment.version = "1.2.4".into();
    depth.enrollments.as_mut().unwrap().push(enrollment);
    let path = root.path().join(REGISTRY_RELATIVE_PATH);
    let mut document: toml_edit::DocumentMut = fs::read_to_string(&path).unwrap().parse().unwrap();
    let entry = document["agents"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["agent_id"].as_str() == Some("codex"))
        .unwrap();
    entry.insert(
        "support_depth",
        toml_edit::Item::Table(
            toml_edit::ser::to_document(&depth)
                .unwrap()
                .as_table()
                .clone(),
        ),
    );
    fs::write(&path, document.to_string()).unwrap();
    git(root.path(), &["add", "."]);
    let registry = AgentRegistry::parse_unenforced(&fs::read_to_string(&path).unwrap()).unwrap();
    args.target_version = "1.2.4".into();
    args.latest_stable = "1.2.4".into();
    args.branch_name = "automation/codex-maintenance-1.2.4".into();
    assert!(build_with_registry(root.path(), &args, &registry)
        .unwrap_err()
        .to_string()
        .contains("same Event"));
    args.request_commit = "fedcba0".into();
    let next = build_with_registry(root.path(), &args, &registry).unwrap();
    isolated_apply(root.path(), &next, &registry).unwrap();
    assert_eq!(fs::read(&old_path).unwrap(), old_bytes);
    assert_eq!(
        next.request
            .detected_release
            .as_ref()
            .unwrap()
            .target_version,
        "1.2.4"
    );
    let published: support_matrix::SupportMatrixArtifact = serde_json::from_slice(
        &next
            .files
            .iter()
            .find(|f| f.relative_path == support_matrix::JSON_OUTPUT_PATH)
            .unwrap()
            .contents,
    )
    .unwrap();
    assert!(published
        .rows
        .iter()
        .any(|r| r.agent == "codex" && r.version == "1.2.3"));
    assert!(published
        .rows
        .iter()
        .any(|r| r.agent == "codex" && r.version == "1.2.4"));
}

#[test]
fn reporting_adapter_rejects_another_agents_new_qualification_even_with_an_old_entry() {
    use crate::depth_record::AcceptanceEntry;
    let (root, registry, args) = workspace("codex");
    let first = build_with_registry(root.path(), &args, &registry).unwrap();
    isolated_apply(root.path(), &first, &registry).unwrap();
    git(root.path(), &["add", "."]);
    let (_other, other_registry, other_args) = workspace("claude_code");
    let depth = other_registry
        .find("claude_code")
        .unwrap()
        .support_depth
        .as_ref()
        .unwrap();
    let path = root.path().join(REGISTRY_RELATIVE_PATH);
    let mut document: toml_edit::DocumentMut = fs::read_to_string(&path).unwrap().parse().unwrap();
    let entry = document["agents"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["agent_id"].as_str() == Some("claude_code"))
        .unwrap();
    entry.insert(
        "support_depth",
        toml_edit::Item::Table(
            toml_edit::ser::to_document(depth)
                .unwrap()
                .as_table()
                .clone(),
        ),
    );
    fs::write(&path, document.to_string()).unwrap();
    git(root.path(), &["add", "."]);
    let registry = AgentRegistry::parse_unenforced(&fs::read_to_string(path).unwrap()).unwrap();
    let other = build_with_registry(root.path(), &other_args, &registry).unwrap();
    isolated_apply(root.path(), &other, &registry).unwrap();
    let mut rec = record(&other);
    rec.acceptance_entries.push(AcceptanceEntry::Promotion {
        record_sha256: rec.stated_sha256().unwrap(),
        targets: rec.policy.selection.enrollment.targets.clone(),
    });
    fixture::write(
        root.path(),
        "cli_manifests/claude_code/reports/1.2.3/depth-record.json",
        serde_json::to_vec_pretty(&rec).unwrap(),
    );
    git(root.path(), &["add", "."]);
    let plan = build_with_registry(root.path(), &args, &registry).unwrap();
    let mut published: support_matrix::SupportMatrixArtifact = serde_json::from_slice(
        &plan
            .files
            .iter()
            .find(|f| f.relative_path == support_matrix::JSON_OUTPUT_PATH)
            .unwrap()
            .contents,
    )
    .unwrap();
    let row = published
        .rows
        .iter_mut()
        .find(|r| r.agent == "claude_code")
        .unwrap();
    row.depth.as_mut().unwrap().promises[0].qualified = true;
    let md = String::from_utf8(
        plan.files
            .iter()
            .find(|f| f.relative_path == support_matrix::MARKDOWN_OUTPUT_PATH)
            .unwrap()
            .contents
            .clone(),
    )
    .unwrap();
    let bundle =
        support_matrix::render_publication_artifacts(&md, &published.rows, String::new()).unwrap();
    let mut proposed = plan
        .files
        .iter()
        .map(|f| {
            (
                f.relative_path.clone(),
                String::from_utf8(f.contents.clone()).unwrap(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    proposed.insert(support_matrix::JSON_OUTPUT_PATH.into(), bundle.json);
    proposed.insert(support_matrix::MARKDOWN_OUTPUT_PATH.into(), bundle.markdown);
    let error =
        crate::depth_acceptance::check_reporting_preparation(root.path(), &registry, proposed)
            .unwrap_err();
    assert!(
        error.contains("acceptance effects") && error.contains("claude_code"),
        "{error}"
    );
}

#[path = "depth_core/repair_cases.rs"]
mod repair_cases;
