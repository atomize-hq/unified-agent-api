//! Isolated retained-depth fixtures through the real CLI guard/finalization/copy-back boundary.
use super::*;
use crate::maintenance_harness as mh;
const NOTE: &str = "docs/agents/lifecycle/codex-maintenance/runtime-note.md";
/// The agent's evidence destination; MA4 delivery refuses bound evidence anywhere else.
const EVIDENCE_DIR: &str = "cli_manifests/codex/reports/0.61.0/evidence";
const EVIDENCE: &str = "cli_manifests/codex/reports/0.61.0/evidence/receipt.json";
/// A receipt path contract revision 1 allows but MA4 delivery refuses: outside `evidence/`.
const OUTSIDE_EVIDENCE: &str = "cli_manifests/codex/reports/0.61.0/evidence.json";
/// A tracked receipt under a directory capture otherwise skips.
const TARGET_EVIDENCE: &str = "target/receipt.json";
/// A raw-output payload outside any `evidence/` destination, referenced only by the receipt.
const PAYLOAD: &str = "cli_manifests/codex/reports/0.61.0/results.txt";

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
    let evidence_path = match evidence {
        "target" => TARGET_EVIDENCE,
        "outside" => OUTSIDE_EVIDENCE,
        _ => EVIDENCE,
    };
    // The fixture receipt's payloads, inside the evidence destination.
    let mut value: Value =
        serde_json::from_str(include_str!("../../../fixtures/depth-evidence-v1.json")).unwrap();
    for (pointer, name) in [
        ("/build/toolchain/path", "toolchain.txt"),
        ("/build/environment/path", "environment.json"),
        ("/build/dependency_graph/path", "dependencies.json"),
        ("/cases/raw_output/path", "results.txt"),
    ] {
        *value.pointer_mut(pointer).unwrap() = json!(format!("{EVIDENCE_DIR}/{name}"));
    }
    let mut receipt = serde_json::to_string_pretty(&value).unwrap();
    let payload = if evidence == "rundir" {
        run_dir_payload()
    } else {
        PAYLOAD.to_owned()
    };
    if matches!(evidence, "committed" | "rundir" | "target") {
        if evidence != "target" {
            value["cases"]["raw_output"] = json!({"path": payload, "sha256": sha("raw")});
        }
        receipt = serde_json::to_string_pretty(&value).unwrap();
        let mut retargeted = value.clone();
        retargeted["cases"]["raw_output"] =
            json!({"path": run_dir_payload(), "sha256": sha("substituted")});
        write_text(
            &temp.path().join("retargeted-receipt.json"),
            &serde_json::to_string_pretty(&retargeted).unwrap(),
        );
        value["build"]["toolchain"] =
            json!({"path": "cli_manifests/codex/reports/0.61.0/extra.txt", "sha256": sha("x")});
        write_text(
            &temp.path().join("extended-receipt.json"),
            &serde_json::to_string_pretty(&value).unwrap(),
        );
    }
    let receipt = receipt.as_str();
    xtask::depth_evidence::EvidenceReceipt::parse(receipt).unwrap();
    write_text(&temp.path().join("receipt-input.json"), receipt);
    let mut next = rec;
    next["execution"] = json!({"identity":"", "policy_identity":next["policy"]["identity"], "obligations_identity":next["obligations"]["identity"],
        "results":[{"obligation":{"operation":"exec","id":"V1","target":REQUIRED_TARGET}, "result":"verified", "evidence":[{"path":evidence_path,"sha256":sha(receipt)}]}],
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
    if matches!(evidence, "committed" | "rundir" | "target") {
        write_json(&temp.path().join(record_path()), &next);
        write_text(&temp.path().join(evidence_path), receipt);
        if evidence == "target" {
            git(temp.path(), &["add", "-f", TARGET_EVIDENCE]);
        } else {
            write_text(&temp.path().join(&payload), "raw");
        }
    }
    republish(&root);
    commit_all(temp.path());
    if evidence == "staged" || evidence == "outside" {
        write_text(&temp.path().join(evidence_path), receipt);
        git(temp.path(), &["add", evidence_path]);
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
            "cp next-depth.json '{}'; mkdir -p '{EVIDENCE_DIR}'; cp receipt-input.json '{EVIDENCE}'; git add .",
            record_path()
        );
        let (temp, root, binary, _) = fixture(&action, initial);
        let before = fs::read(root.join(format!("reports/{VERSION}/depth-record.json"))).unwrap();
        let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
        assert_ne!(out.exit_code, 0);
        // Production stages no evidence, so a candidate-made bound receipt is refused by the
        // delivery byte binding (or, failing that, as an untracked bound input).
        assert!(
            out.stderr.contains("not the bytes the host staged")
                || out.stderr.contains("untracked bound input"),
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

#[test]
fn inherited_alternate_index_cannot_supply_missing_caller_evidence_binding() {
    let action = format!("cp next-depth.json '{}'; git add .", record_path());
    let (temp, root, binary, _) = fixture(&action, "untracked");
    let alternate = tempfile::tempdir().unwrap();
    let index = alternate.path().join("index");
    for args in [vec!["read-tree", "HEAD"], vec!["add", EVIDENCE]] {
        let out = Command::new("git")
            .current_dir(temp.path())
            .env("GIT_INDEX_FILE", &index)
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
    }
    let before = fs::read(root.join(format!("reports/{VERSION}/depth-record.json"))).unwrap();
    let caller_index = fs::read(temp.path().join(".git/index")).unwrap();
    let alternate_index = fs::read(&index).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(temp.path())
        .env("GIT_INDEX_FILE", &index)
        .args(
            mh::execute_args("--write", Some(&binary))
                .into_iter()
                .skip(1),
        )
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "alternate index supplied caller authority"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("untracked bound input"), "{stderr}");
    assert_eq!(
        fs::read(root.join(format!("reports/{VERSION}/depth-record.json"))).unwrap(),
        before
    );
    assert_eq!(
        fs::read(temp.path().join(".git/index")).unwrap(),
        caller_index
    );
    assert_eq!(fs::read(index).unwrap(), alternate_index);
    assert!(!temp.path().join(NOTE).exists());
}

/// Production stages no evidence: with genuine E, its receipt and an external payload committed,
/// substituting the payload behind the unchanged receipt, or the receipt newly referencing
/// another external file, refuses the whole delivery with caller files and index unchanged.
#[test]
fn committed_evidence_payloads_outside_evidence_destinations_cannot_change() {
    for action in [
        format!("printf substituted > '{PAYLOAD}'"),
        format!(
            "cp extended-receipt.json '{EVIDENCE}'; printf x > 'cli_manifests/codex/reports/0.61.0/extra.txt'"
        ),
    ] {
        let (temp, _root, binary, _) = fixture(&action, "committed");
        let files = [EVIDENCE, PAYLOAD, NOTE].map(|p| fs::read(temp.path().join(p)).ok());
        let index = fs::read(temp.path().join(".git/index")).unwrap();
        let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
        assert_ne!(out.exit_code, 0, "{action}");
        assert!(
            out.stderr.contains("outside its evidence destination"),
            "{action}: {}",
            out.stderr
        );
        assert_eq!(
            [EVIDENCE, PAYLOAD, NOTE].map(|p| fs::read(temp.path().join(p)).ok()),
            files,
            "{action}"
        );
        assert!(!temp
            .path()
            .join("cli_manifests/codex/reports/0.61.0/extra.txt")
            .exists());
        assert_eq!(fs::read(temp.path().join(".git/index")).unwrap(), index);
    }
}

/// The payload's path inside this run's own diagnostics directory.
fn run_dir_payload() -> String {
    format!(
        "docs/agents/.uaa-temp/agent-maintenance/runs/{}/raw.txt",
        mh::EXECUTE_WRITE_RUN_ID
    )
}

/// Production: a committed receipt binding a payload inside the run directory refuses the whole
/// delivery, and the substituted payload is not copied back as a diagnostic.
#[test]
fn committed_evidence_bound_into_the_run_directory_refuses_without_copy_back() {
    let payload = run_dir_payload();
    let (temp, _root, binary, _) = fixture(&format!("printf substituted > '{payload}'"), "rundir");
    let index = fs::read(temp.path().join(".git/index")).unwrap();
    let files = [EVIDENCE, NOTE].map(|p| fs::read(temp.path().join(p)).ok());
    let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
    assert_ne!(out.exit_code, 0);
    assert!(
        out.stderr.contains("may not live in an MA4 run directory"),
        "{}",
        out.stderr
    );
    assert_eq!(fs::read(temp.path().join(&payload)).unwrap(), b"raw");
    assert_eq!(
        [EVIDENCE, NOTE].map(|p| fs::read(temp.path().join(p)).ok()),
        files
    );
    assert_eq!(fs::read(temp.path().join(".git/index")).unwrap(), index);
}

/// Diagnostic copy-back fails closed: the executor retargets the committed receipt into its run
/// directory, substitutes that payload, then plants something a full candidate capture refuses
/// (a FIFO, or a literal-backslash name, in a sibling run directory). No substituted byte reaches
/// the caller; only host-authored diagnostics are delivered.
#[cfg(unix)]
#[test]
fn diagnostic_copy_back_fails_closed_when_the_candidate_cannot_be_captured() {
    let payload = run_dir_payload();
    let sibling = "docs/agents/.uaa-temp/agent-maintenance/runs/sibling";
    for plant in [
        format!("mkdir -p {sibling}; mkfifo {sibling}/fifo"),
        format!("mkdir -p {sibling}; printf x > '{sibling}/a\\b'"),
    ] {
        let action = format!(
            "cp retargeted-receipt.json '{EVIDENCE}'; printf substituted > '{payload}'; {plant}"
        );
        let (temp, _root, binary, _) = fixture(&action, "committed");
        let files = [EVIDENCE, PAYLOAD, NOTE].map(|p| fs::read(temp.path().join(p)).ok());
        let index = fs::read(temp.path().join(".git/index")).unwrap();
        let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
        assert_ne!(out.exit_code, 0, "{plant}");
        assert!(
            !temp.path().join(&payload).exists(),
            "{plant}: substituted payload copied back"
        );
        assert!(!temp.path().join(sibling).exists(), "{plant}");
        assert_eq!(
            [EVIDENCE, PAYLOAD, NOTE].map(|p| fs::read(temp.path().join(p)).ok()),
            files,
            "{plant}"
        );
        assert_eq!(fs::read(temp.path().join(".git/index")).unwrap(), index);
        // Host diagnostics still reach the caller.
        let run = payload.rsplit_once('/').unwrap().0;
        assert!(
            temp.path()
                .join(run)
                .join("validation-report.json")
                .is_file(),
            "{plant}: {}",
            out.stderr
        );
    }
}

/// MA4 delivery refuses an E that binds a receipt outside its evidence destination (contract
/// revision 1 allows it; only delivery refuses), with caller files and index unchanged.
#[test]
fn reporting_e_binding_outside_its_evidence_destination_is_refused() {
    let (temp, root, binary, _) = fixture(
        &format!("cp next-depth.json '{}'; git add .", record_path()),
        "outside",
    );
    let record = fs::read(root.join(format!("reports/{VERSION}/depth-record.json"))).unwrap();
    let index = fs::read(temp.path().join(".git/index")).unwrap();
    let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
    assert_ne!(out.exit_code, 0);
    assert!(
        out.stderr.contains("outside its evidence destination")
            && out.stderr.contains(OUTSIDE_EVIDENCE),
        "{}",
        out.stderr
    );
    assert_eq!(
        fs::read(root.join(format!("reports/{VERSION}/depth-record.json"))).unwrap(),
        record
    );
    assert_eq!(fs::read(temp.path().join(".git/index")).unwrap(), index);
}

/// A tracked receipt under `target/` (which capture otherwise skips), bound by an unchanged
/// committed E. Whether the executor only substitutes a payload, or also retargets the receipt
/// into its run directory, delivery is refused and nothing the executor wrote reaches the caller.
#[test]
fn a_tracked_target_receipt_bound_by_unchanged_e_is_refused() {
    let payload = run_dir_payload();
    let results = format!("{EVIDENCE_DIR}/results.txt");
    for action in [
        format!("mkdir -p '{EVIDENCE_DIR}'; printf substituted > '{results}'"),
        format!("cp retargeted-receipt.json '{TARGET_EVIDENCE}'; printf substituted > '{payload}'"),
    ] {
        let (temp, _root, binary, _) = fixture(&action, "target");
        let files = [TARGET_EVIDENCE, NOTE].map(|p| fs::read(temp.path().join(p)).ok());
        let index = fs::read(temp.path().join(".git/index")).unwrap();
        let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
        assert_ne!(out.exit_code, 0, "{action}");
        assert!(
            out.stderr.contains("outside its evidence destination")
                || out.stderr.contains("write boundary violation"),
            "{action}: {}",
            out.stderr
        );
        assert!(!temp.path().join(&payload).exists(), "{action}");
        assert!(!temp.path().join(&results).exists(), "{action}");
        assert_eq!(
            [TARGET_EVIDENCE, NOTE].map(|p| fs::read(temp.path().join(p)).ok()),
            files,
            "{action}"
        );
        assert_eq!(fs::read(temp.path().join(".git/index")).unwrap(), index);
    }
}

/// On failure the host serializes every diagnostic itself: an executor-written
/// `codex-execution.json` never reaches the caller, even when the executor also breaks the
/// host's own log writes and full candidate capture.
#[cfg(unix)]
#[test]
fn an_executor_written_execution_record_is_never_delivered_on_failure() {
    let run = run_dir_payload().rsplit_once('/').unwrap().0.to_owned();
    let sibling = "docs/agents/.uaa-temp/agent-maintenance/runs/sibling";
    let action = format!(
        "printf forged > '{run}/codex-execution.json'; mkdir -p '{run}/codex-stdout.log'; \
         mkdir -p {sibling}; printf x > '{sibling}/a\\b'; exit 1"
    );
    let (temp, _root, binary, _) = fixture(&action, "absent");
    let index = fs::read(temp.path().join(".git/index")).unwrap();
    let out = mh::run_execute_cli(mh::execute_args("--write", Some(&binary)), temp.path());
    assert_ne!(out.exit_code, 0);
    let record = temp.path().join(&run).join("codex-execution.json");
    if let Ok(bytes) = fs::read(&record) {
        assert!(
            !String::from_utf8_lossy(&bytes).contains("forged"),
            "forged record delivered"
        );
    }
    assert!(!temp.path().join(&run).join("codex-stdout.log").is_dir());
    assert!(!temp.path().join(sibling).exists());
    assert!(
        temp.path()
            .join(&run)
            .join("validation-report.json")
            .is_file(),
        "{}",
        out.stderr
    );
    assert_eq!(fs::read(temp.path().join(".git/index")).unwrap(), index);
}
