use super::*;

#[test]
fn p_only_replacement_refuses_to_erase_carried_due_targets() {
    let (root, registry, mut args) = workspace("codex");
    fixture::acquire(root.path(), "codex");
    let plan = build_with_registry(root.path(), &args, &registry).unwrap();
    assert!(!record(&plan)
        .obligations
        .as_ref()
        .unwrap()
        .acceptance_work
        .is_empty());
    isolated_apply(root.path(), &plan, &registry).unwrap();
    git(root.path(), &["add", "."]);
    let path = root
        .path()
        .join("cli_manifests/codex/reports/1.2.3/depth-record.json");
    let bytes = fs::read(&path).unwrap();
    fs::remove_dir_all(root.path().join("cli_manifests/codex/snapshots/1.2.3")).unwrap();
    for entry in fs::read_dir(path.parent().unwrap()).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name().to_string_lossy().starts_with("coverage.") {
            fs::remove_file(entry.path()).unwrap();
        }
    }
    git(root.path(), &["add", "."]);
    args.request_commit = "fedcba0".into();
    let request_path = root.path().join(&plan.request.relative_path);
    let request = fs::read(&request_path).unwrap();
    let error = build_with_registry(root.path(), &args, &registry)
        .unwrap_err()
        .to_string();
    assert!(error.contains("prior targeted work"), "{error}");
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert_eq!(fs::read(request_path).unwrap(), request);
}

fn add_claude_enrollment(root: &Path) -> (AgentRegistry, Args) {
    let (_donor, registry, args) = workspace("claude_code");
    let depth = registry
        .find("claude_code")
        .unwrap()
        .support_depth
        .as_ref()
        .unwrap();
    let path = root.join(REGISTRY_RELATIVE_PATH);
    let mut doc: toml_edit::DocumentMut = fs::read_to_string(&path).unwrap().parse().unwrap();
    let entry = doc["agents"]
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
    fs::write(&path, doc.to_string()).unwrap();
    git(root, &["add", "."]);
    (
        AgentRegistry::parse_unenforced(&fs::read_to_string(path).unwrap()).unwrap(),
        args,
    )
}

#[test]
fn retained_claims_stay_unverified_and_positive_reporting_upgrades_still_refuse() {
    use crate::{
        agent_registry::support_depth::ObligationId,
        depth_record::{
            part_identity, DepthResult, ExecutionPart, FileRef, ObligationRef, ObligationResult,
        },
    };
    for (claim, obligation) in [
        (DepthResult::Verified, ObligationId::R1),
        (DepthResult::NotApplicable, ObligationId::X1),
    ] {
        for agent in ["codex", "claude_code"] {
            let (root, mut registry, invocation) = workspace("codex");
            let args = if agent == "claude_code" {
                let (r, a) = add_claude_enrollment(root.path());
                registry = r;
                a
            } else {
                invocation.clone()
            };
            if claim == DepthResult::NotApplicable {
                let declaration = registry
                    .agents
                    .iter_mut()
                    .find(|e| e.agent_id == agent)
                    .unwrap()
                    .support_depth
                    .as_mut()
                    .unwrap();
                declaration.enrollments.as_mut().unwrap()[0].operations[0]
                    .not_applicable_permitted
                    .push(ObligationId::X1);
                let path = root.path().join(REGISTRY_RELATIVE_PATH);
                let mut doc: toml_edit::DocumentMut =
                    fs::read_to_string(&path).unwrap().parse().unwrap();
                let entry = doc["agents"]
                    .as_array_of_tables_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|e| e["agent_id"].as_str() == Some(agent))
                    .unwrap();
                entry.insert(
                    "support_depth",
                    toml_edit::Item::Table(
                        toml_edit::ser::to_document(declaration)
                            .unwrap()
                            .as_table()
                            .clone(),
                    ),
                );
                fs::write(path, doc.to_string()).unwrap();
                git(root.path(), &["add", "."]);
            }
            fixture::acquire(root.path(), agent);
            let plan = build_with_registry(root.path(), &args, &registry).unwrap();
            isolated_apply(root.path(), &plan, &registry).unwrap();
            let mut rec = record(&plan);
            let target = rec.policy.selection.enrollment.targets[0].clone();
            let evidence = "fixtures/success.txt";
            let hash = hex::encode(Sha256::digest(b"success"));
            fixture::write(root.path(), evidence, "success");
            let mut execution = ExecutionPart {
                identity: String::new(),
                policy_identity: rec.policy.identity.clone(),
                obligations_identity: rec.obligations.as_ref().unwrap().identity.clone(),
                results: vec![ObligationResult {
                    obligation: ObligationRef {
                        operation: "run".into(),
                        id: Some(obligation),
                        capability: None,
                        target: target.clone(),
                    },
                    result: claim,
                    rationale: (claim == DepthResult::NotApplicable)
                        .then(|| "fixture reads no credentials".into()),
                    evidence: if claim == DepthResult::Verified {
                        vec![FileRef {
                            path: evidence.into(),
                            sha256: hash.clone(),
                        }]
                    } else {
                        vec![]
                    },
                }],
                reuse_bindings: vec![],
                debt_grants: vec![],
                debt_retired: vec![],
            };
            execution.identity = part_identity(&execution).unwrap();
            rec.execution = Some(execution);
            fixture::write(
                root.path(),
                &format!("cli_manifests/{agent}/reports/1.2.3/depth-record.json"),
                serde_json::to_vec_pretty(&rec).unwrap(),
            );
            git(root.path(), &["add", "."]);
            let facts =
                support_matrix::depth::derive_depth_facts(Some(&rec), true, &target, &|path| {
                    (path == evidence).then(|| hash.clone())
                })
                .unwrap()
                .unwrap();
            assert!(facts.promises.iter().all(|p| !p.qualified));
            let before = git(root.path(), &["status", "--porcelain=v1"]);
            if claim == DepthResult::Verified {
                assert!(facts
                    .promises
                    .iter()
                    .flat_map(|p| &p.obligations)
                    .all(|o| o.result == DepthResult::Unverified));
                // Retained hash-only claims can report truthful unverified facts, not a positive
                // result on this target or another agent's aggregate publication.
                build_with_registry(root.path(), &invocation, &registry).unwrap();
            } else {
                assert!(facts.promises[0]
                    .obligations
                    .iter()
                    .any(|o| o.result == DepthResult::NotApplicable));
                let error = build_with_registry(root.path(), &invocation, &registry)
                    .unwrap_err()
                    .to_string();
                assert!(
                    error.contains("deferred execution/debt verdict") && error.contains(agent),
                    "{error}"
                );
            }
            assert_eq!(git(root.path(), &["status", "--porcelain=v1"]), before);
            // An already published identical result may be retained; C still supplies no new verdict.
            let mut published: support_matrix::SupportMatrixArtifact = serde_json::from_slice(
                &fs::read(root.path().join(support_matrix::JSON_OUTPUT_PATH)).unwrap(),
            )
            .unwrap();
            let row = published
                .rows
                .iter_mut()
                .find(|r| r.agent == agent && r.version == "1.2.3" && r.target == target)
                .unwrap();
            row.depth = Some(facts);
            let md =
                fs::read_to_string(root.path().join(support_matrix::MARKDOWN_OUTPUT_PATH)).unwrap();
            let runtime = fs::read_to_string(
                root.path()
                    .join(support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH),
            )
            .unwrap();
            let bundle =
                support_matrix::render_publication_artifacts(&md, &published.rows, runtime)
                    .unwrap();
            fixture::write(root.path(), support_matrix::JSON_OUTPUT_PATH, bundle.json);
            fixture::write(
                root.path(),
                support_matrix::MARKDOWN_OUTPUT_PATH,
                bundle.markdown,
            );
            git(root.path(), &["add", "."]);
            let stable = build_with_registry(root.path(), &invocation, &registry).unwrap();
            isolated_apply(root.path(), &stable, &registry).unwrap();
        }
    }
}
