//! Isolated retained-depth fixtures through the real CLI guard/finalization/copy-back boundary.
use super::*;
use crate::maintenance_harness as mh;
const NOTE: &str = "docs/agents/lifecycle/codex-maintenance/runtime-note.md";
const EVIDENCE: &str = "cli_manifests/codex/reports/0.61.0/evidence.json";

fn fixture(action: &str, evidence: &str) -> (tempfile::TempDir, PathBuf, PathBuf, Value) {
    let rec = record_with_obligations();
    let (temp, root, _) = baseline(Some(&rec));
    let seed = mh::prepare_execute_fixture("relay-boundary-seed");
    copy_dir_recursive(
        &seed.join("docs/agents/lifecycle/codex-maintenance"),
        &temp.path().join("docs/agents/lifecycle/codex-maintenance"),
    );
    for path in [
        ".github/workflows/agent-maintenance-open-pr.yml",
        "gate-command.sh",
        "cli_manifests/codex/versions/0.98.0.json",
    ] {
        write_text(
            &temp.path().join(path),
            &fs::read_to_string(seed.join(path)).unwrap(),
        );
    }
    write_text(&temp.path().join("docs/specs/unified-agent-api/non-tui-support-debt.md"), "# Non-TUI Support Debt Inventory\n\n### `support-debt-authorization-contract-target-version-v1`\n\n## Inventory\n");
    let request = temp.path().join(request_path());
    let raw = fs::read_to_string(&request).unwrap();
    write_text(&request, &raw.replace("  \"cli_manifests/codex/versions/0.98.0.json\",", "  \"cli_manifests/**\",\n  \"docs/specs/unified-agent-api/**\",\n  \"crates/agent_api/**\","));
    let registry = xtask::agent_registry::AgentRegistry::load(temp.path()).unwrap();
    let targets = &registry.find("codex").unwrap().canonical_targets;
    let mut report = json!({"inputs":{"upstream":{"semantic_version":"0.98.0","targets":targets}}, "platform_filter":{"mode":"any"}, "deltas":{"missing_commands":[],"missing_flags":[],"missing_args":[],"intentionally_unsupported":[]}});
    write_json(&root.join("reports/0.98.0/coverage.any.json"), &report);
    for target in targets {
        report["inputs"]["upstream"]["targets"] = json!([target]);
        report["platform_filter"] = json!({"mode":"exact_target", "target_triple":target});
        write_json(
            &root.join(format!("reports/0.98.0/coverage.{target}.json")),
            &report,
        );
    }
    let receipt = include_str!("../../../fixtures/depth-evidence-v1.json");
    xtask::depth_evidence::EvidenceReceipt::parse(receipt).unwrap();
    write_text(&temp.path().join("receipt-input.json"), receipt);
    let mut next = rec;
    next["execution"] = json!({"identity":"", "policy_identity":next["policy"]["identity"], "obligations_identity":next["obligations"]["identity"],
        "results":[{"obligation":{"operation":"exec","id":"V1","target":REQUIRED_TARGET}, "result":"verified", "evidence":[{"path":EVIDENCE,"sha256":sha(receipt)}]}],
        "reuse_bindings":[], "debt_grants":[], "debt_retired":[]});
    seal(&mut next);
    write_json(&temp.path().join("next-depth.json"), &next);
    let binary = mh::fake_execute_codex_binary(temp.path());
    let raw = fs::read_to_string(&binary).unwrap();
    write_text(
        &binary,
        &raw.replace(
            "  success)",
            &format!(
                "  boundary)\n    printf reporting > '{NOTE}'\n    {action}\n    ;;\n  success)"
            ),
        ),
    );
    write_text(
        &root.join(format!("pointers/latest_supported/{REQUIRED_TARGET}.txt")),
        "none\n",
    );
    republish(&root);
    commit_all(temp.path());
    if evidence == "staged" {
        write_text(&temp.path().join(EVIDENCE), receipt);
        git(temp.path(), &["add", EVIDENCE]);
    }
    if evidence == "untracked" {
        write_text(&temp.path().join(EVIDENCE), receipt);
    }
    let dry = mh::run_execute_cli(mh::execute_args("--dry-run", Some(&binary)), temp.path());
    assert_eq!(dry.exit_code, 0, "{}", dry.stderr);
    mh::write_fake_execute_codex_scenario(temp.path(), "boundary");
    (temp, root, binary, next)
}
#[test]
fn reporting_e_delivers_nonempty_bound_evidence_and_preserves_caller_index() {
    let (temp, root, binary, next) = fixture(
        &format!("cp next-depth.json '{}'; git add .", record_path()),
        "staged",
    );
    let index = fs::read(temp.path().join(".git/index")).unwrap();
    let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
    assert_eq!(out.exit_code, 0, "{}", out.stderr);
    assert_eq!(fs::read(temp.path().join(".git/index")).unwrap(), index);
    assert_eq!(
        serde_json::from_slice::<Value>(
            &fs::read(root.join(format!("reports/{VERSION}/depth-record.json"))).unwrap()
        )
        .unwrap(),
        next
    );
    let record =
        xtask::depth_record::DepthRecord::parse(&next.to_string(), "codex", VERSION).unwrap();
    assert!(xtask::depth_record::bound_evidence(temp.path(), [&record])
        .unwrap()
        .contains_key(EVIDENCE));
    let artifact: Value = serde_json::from_slice(
        &fs::read(
            temp.path()
                .join("cli_manifests/support_matrix/current.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let row = artifact["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["agent"] == "codex" && r["version"] == VERSION && r["target"] == REQUIRED_TARGET
        })
        .unwrap();
    assert_eq!(
        row["depth"]["promises"][0]["qualified"], false,
        "raw verified claim conferred authority"
    );
}
#[test]
fn candidate_only_staging_cannot_admit_new_or_preexisting_untracked_receipt() {
    for initial in ["absent", "untracked"] {
        let action = format!(
            "cp next-depth.json '{}'; cp receipt-input.json '{EVIDENCE}'; git add .",
            record_path()
        );
        let (temp, root, binary, _) = fixture(&action, initial);
        let before = fs::read(root.join(format!("reports/{VERSION}/depth-record.json"))).unwrap();
        let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
        assert_ne!(out.exit_code, 0);
        assert!(
            out.stderr.contains("untracked bound input"),
            "{}",
            out.stderr
        );
        assert_eq!(
            fs::read(root.join(format!("reports/{VERSION}/depth-record.json"))).unwrap(),
            before
        );
        assert!(!temp.path().join(NOTE).exists());
        assert_eq!(temp.path().join(EVIDENCE).exists(), initial == "untracked");
    }
}
#[test]
fn ordinary_later_request_cannot_promote_retained_tuple_through_actual_relay() {
    let action = format!("printf '{VERSION}\\n' > cli_manifests/codex/pointers/latest_supported/{REQUIRED_TARGET}.txt");
    let (temp, root, binary, _) = fixture(&action, "absent");
    let path = root.join(format!("pointers/latest_supported/{REQUIRED_TARGET}.txt"));
    let before = fs::read(&path).unwrap();
    let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
    assert_ne!(out.exit_code, 0);
    assert!(
        out.stderr.contains("acceptance effects forbidden"),
        "{}",
        out.stderr
    );
    assert_eq!(fs::read(path).unwrap(), before);
    assert!(!temp.path().join(NOTE).exists());
}

#[test]
fn ordinary_run_rejects_other_agent_aggregate_qualification() {
    let action = "cp aggregate.json cli_manifests/support_matrix/current.json; cp aggregate.md docs/specs/unified-agent-api/support-matrix.md";
    let (temp, root, binary, _) = fixture(action, "absent");
    let mut other = record_with_obligations();
    other["policy"]["selection"]["agent_id"] = json!("claude_code");
    other["policy"]["selection"]["enrollment"]["modes"][0]["command_path"] =
        json!("claude_code exec");
    seal(&mut other);
    other["obligations"]["policy_identity"] = other["policy"]["identity"].clone();
    seal(&mut other);
    write_json(
        &temp.path().join(format!(
            "cli_manifests/claude_code/reports/{VERSION}/depth-record.json"
        )),
        &other,
    );
    let rules_path = temp.path().join("cli_manifests/claude_code/RULES.json");
    let mut rules: Value = serde_json::from_slice(
        &fs::read(workspace_root().join("cli_manifests/claude_code/RULES.json")).unwrap(),
    )
    .unwrap();
    rules["storage"]["committed"]["reports"]["depth_record"] =
        json!("cli_manifests/claude_code/reports/<version>/depth-record.json");
    write_json(&rules_path, &rules);
    let other_root = temp.path().join("cli_manifests/claude_code");
    for (path, fields) in [
        (
            other_root.join("current.json"),
            json!({"snapshot_schema_version":2,"mode":"union"}),
        ),
        (
            other_root.join(format!("versions/{VERSION}.json")),
            json!({"schema_version":1}),
        ),
    ] {
        let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        write_json(&path, &value);
    }
    xtask::support_matrix::derive_rows(temp.path()).unwrap();
    republish(&root);
    let path = temp
        .path()
        .join("cli_manifests/support_matrix/current.json");
    let before = fs::read(&path).unwrap();
    let mut artifact: xtask::support_matrix::SupportMatrixArtifact =
        serde_json::from_slice(&before).unwrap();
    let row = artifact
        .rows
        .iter_mut()
        .find(|r| r.agent == "claude_code" && r.version == VERSION && r.target == REQUIRED_TARGET)
        .unwrap();
    row.depth.as_mut().unwrap().promises[0].qualified = true;
    let bundle =
        xtask::support_matrix::render_publication_bundle(&artifact.rows, String::new()).unwrap();
    write_text(&temp.path().join("aggregate.json"), &bundle.json);
    write_text(
        &temp.path().join("aggregate.md"),
        &format!(
            "<!-- support-matrix-published:start -->\n{}<!-- support-matrix-published:end -->\n",
            bundle.markdown
        ),
    );
    let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
    assert_ne!(out.exit_code, 0);
    assert!(
        out.stderr.contains("acceptance effects forbidden"),
        "{}",
        out.stderr
    );
    assert_eq!(fs::read(path).unwrap(), before);
    assert!(!temp.path().join(NOTE).exists());
}
