use super::*;

#[test]
fn every_previously_covered_mode_target_and_capability_must_remain() {
    let mut rec = record();
    let enrollment = &mut rec["policy"]["selection"]["enrollment"];
    enrollment["targets"] = json!([REQUIRED_TARGET, "aarch64-unknown-linux-musl"]);
    enrollment["modes"].as_array_mut().unwrap().push(json!({ "selector": "inspect", "command_path": "codex inspect", "mode": "headless", "predicates": [] }));
    enrollment["operations"][0]["serves"] = json!(["exec-json", "inspect"]);
    enrollment["operations"][0]["shared_mapping"] = json!("mapped");
    enrollment["capabilities"] = json!([{ "id": "agent_api.run", "operations": ["exec"] }]);
    seal(&mut rec);
    let (temp, root, tip) = baseline(Some(&rec));
    let (status, report) = compare(&root, &tip);
    assert_eq!(status, 0, "{report}");
    for change in ["mode", "target", "capability"] {
        let mut changed = rec.clone();
        let e = &mut changed["policy"]["selection"]["enrollment"];
        match change {
            "mode" => {
                e["modes"].as_array_mut().unwrap().pop();
                e["operations"][0]["serves"] = json!(["exec-json"]);
            }
            "target" => e["targets"] = json!([REQUIRED_TARGET]),
            _ => {
                e["capabilities"] = json!([]);
                e["operations"][0]["shared_mapping"] = json!("wrapper_only");
            }
        }
        seal(&mut changed);
        write_record(&root, &changed);
        freeze(temp.path(), &changed);
        fails(&root, &tip, "DEPTH_SELECTION_SHRUNK");
    }
}

#[test]
fn operation_renames_and_later_generations_preserve_coverage_and_entries() {
    let mut rec = record();
    rec["acceptance_entries"] = json!([{ "backs": "promotion", "targets": [REQUIRED_TARGET], "record_sha256": "a".repeat(64) }]);
    let (temp, root, tip) = baseline(Some(&rec));
    rec["policy"]["selection"]["enrollment"]["operations"][0]["id"] = json!("renamed");
    rec["event"]["request_recorded_at"] = json!("2026-10-20T00:00:00Z");
    seal(&mut rec);
    write_record(&root, &rec);
    freeze(temp.path(), &rec);
    let (status, report) = compare(&root, &tip);
    assert_eq!(status, 0, "{report}");
    let (status, report) = compare_args(&root, &tip, "codex-validate", &[]);
    assert_eq!(status, 0, "alias: {report}");
}

#[test]
fn missing_table_wrong_version_and_other_lifecycle_path_are_not_freezes() {
    let (temp, root, tip) = baseline(Some(&record()));
    let mut rec = record();
    rec["policy"]["rules_revision"] = json!("2");
    seal(&mut rec);
    write_record(&root, &rec);
    write_request(temp.path(), COMMIT, None);
    fails(&root, &tip, "DEPTH_RECORD_WITHOUT_FREEZE");
    freeze(temp.path(), &rec);
    edit_request(temp.path(), |text| text.replace(VERSION, "0.62.0"));
    fails(&root, &tip, "DEPTH_RECORD_WITHOUT_FREEZE");

    // An onboarding record cannot use even a matching version's maintenance freeze.
    rec["event"] = json!({ "lifecycle_path": "onboarding", "approval_commit": COMMIT, "approval_recorded_at": RECORDED_AT });
    rec["policy"]["selection"]["enrollment"]["lifecycle_path"] = json!("onboarding");
    seal(&mut rec);
    write_record(&root, &rec);
    fails(&root, &tip, "DEPTH_RECORD_WITHOUT_FREEZE");
}

fn approval(workspace: &Path, version: &str) -> String {
    let text = fs::read_to_string(
        workspace_root()
            .join(GOVERNANCE)
            .join("approved-agent.toml"),
    )
    .unwrap();
    write_text(
        &workspace.join("docs/agents/selection/cli-agent-selection-packet.md"),
        "# Selection\n",
    );
    format!("{text}\n[descriptor.maintenance]\nmode = \"explicitly_deferred\"\n[descriptor.maintenance.deferral]\nreason = \"fixture\"\nfollow_up = \"fixture\"\napproved_scope = \"create_lane_closeout\"\n[support_depth]\nversion = \"{version}\"\n")
}

#[test]
fn onboarding_creation_requires_a_loadable_agent_root_and_version_approval() {
    let (temp, root, tip) = baseline(None);
    let mut rec = record();
    rec["event"] = json!({ "lifecycle_path": "onboarding", "approval_commit": COMMIT, "approval_recorded_at": RECORDED_AT });
    rec["policy"]["selection"]["enrollment"]["lifecycle_path"] = json!("onboarding");
    seal(&mut rec);
    write_record(&root, &rec);
    fails(&root, &tip, "DEPTH_RECORD_WITHOUT_FREEZE");
    let path = temp.path().join(GOVERNANCE).join("approved-agent.toml");
    let text = approval(temp.path(), VERSION);
    write_text(&path, &text);
    let (status, report) = compare(&root, &tip);
    assert_eq!(status, 0, "{report}");
    for bad in [
        text.replace(&format!("version = \"{VERSION}\""), "version = \"0.62.0\""),
        text.replace(
            "manifest_root = \"cli_manifests/codex\"",
            "manifest_root = \"cli_manifests/aider\"",
        ),
        text.replace("agent_id = \"codex\"", "agent_id = \"aider\""),
    ] {
        write_text(&path, &bad);
        fails(&root, &tip, "DEPTH_RECORD_WITHOUT_FREEZE");
    }
    write_text(&path, "not toml !!!");
    fails(&root, &tip, "DEPTH_APPROVAL_UNREADABLE");
}

#[test]
fn an_unchanged_onboarding_statement_needs_no_surviving_approval() {
    let (temp, root, _tip) = baseline(None);
    let mut rec = record();
    rec["event"] = json!({ "lifecycle_path": "onboarding", "approval_commit": COMMIT, "approval_recorded_at": RECORDED_AT });
    rec["policy"]["selection"]["enrollment"]["lifecycle_path"] = json!("onboarding");
    seal(&mut rec);
    rec["acceptance_entries"] = json!([{ "backs": "promotion", "targets": [REQUIRED_TARGET], "record_sha256": "a".repeat(64) }]);
    write_record(&root, &rec);
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-qm", "onboarding record"]);
    let tip = git(temp.path(), &["rev-parse", "HEAD"]);
    write_record(&root, &rec);
    let (status, report) = compare(&root, &tip);
    assert_eq!(status, 0, "{report}");
}

#[test]
fn baseline_is_an_immutable_git_tree_and_unreadable_records_never_disappear() {
    let (temp, root, _tip) = baseline(Some(&record()));
    let path = root.join(format!("reports/{VERSION}/depth-record.json"));
    for directory in [false, true] {
        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
        if directory {
            write_text(&path.join("nested"), "not a record");
        } else {
            write_text(&path, "not JSON");
        }
        git(temp.path(), &["add", "."]);
        git(temp.path(), &["commit", "-qm", "bad tip"]);
        let bad_tip = git(temp.path(), &["rev-parse", "HEAD"]);
        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        }
        write_record(&root, &record());
        fails(&root, &bad_tip, "DEPTH_TIP_UNREADABLE");
    }
}

#[test]
fn losing_registry_and_adoption_anchors_does_not_hide_tip_records() {
    let (temp, root, tip) = baseline(Some(&record()));
    let registry = temp.path().join("crates/xtask/data/agent_registry.toml");
    let original = fs::read_to_string(&registry).unwrap();
    write_text(
        &registry,
        &original.replace(
            "manifest_root = \"cli_manifests/codex\"",
            "manifest_root = \"cli_manifests/old_codex\"",
        ),
    );
    fails(&root, &tip, "DEPTH_RECORD_UNANCHORED");
    fs::remove_dir_all(root.join(format!("reports/{VERSION}"))).unwrap();
    fails(&root, &tip, "DEPTH_RECORD_REMOVED");
    write_text(&registry, &original);
    write_record(&root, &record());
    let path = root.join("RULES.json");
    let mut rules: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    rules["storage"]["committed"]["reports"]
        .as_object_mut()
        .unwrap()
        .remove("depth_record");
    write_json(&path, &rules);
    fails(&root, &tip, "DEPTH_RECORD_UNADOPTED");
}

#[test]
fn relocating_a_registered_root_through_a_symlink_cannot_hide_its_tip() {
    let (temp, root, tip) = baseline(Some(&record()));
    let relocated = temp.path().join("cli_manifests/relocated");
    fs::rename(&root, &relocated).unwrap();
    std::os::unix::fs::symlink("relocated", &root).unwrap();
    let registry = temp.path().join("crates/xtask/data/agent_registry.toml");
    let text = fs::read_to_string(&registry).unwrap().replace(
        "manifest_root = \"cli_manifests/codex\"",
        "manifest_root = \"cli_manifests/relocated\"",
    );
    write_text(&registry, &text);
    fs::remove_dir_all(relocated.join(format!("reports/{VERSION}"))).unwrap();
    fs::remove_file(temp.path().join(request_path())).unwrap();
    let rules = relocated.join("RULES.json");
    let text = fs::read_to_string(&rules).unwrap().replace(
        "cli_manifests/codex/reports/",
        "cli_manifests/relocated/reports/",
    );
    write_text(&rules, &text);
    fails(&root, &tip, "DEPTH_RECORD_UNANCHORED");
    fails(&root, &tip, "DEPTH_RECORD_REMOVED");
}

#[test]
fn new_records_without_a_candidate_registry_cannot_skip_freeze_checks() {
    let (temp, root, tip) = baseline(None);
    write_record(&root, &record());
    fs::remove_file(temp.path().join("crates/xtask/data/agent_registry.toml")).unwrap();
    fails(&root, &tip, "DEPTH_RECORD_UNANCHORED");
}

#[test]
fn symlinking_a_record_or_its_report_directories_cannot_hide_removal() {
    for component in ["file", "version", "reports"] {
        let (_temp, root, tip) = baseline(Some(&record()));
        let path = match component {
            "file" => root.join(format!("reports/{VERSION}/depth-record.json")),
            "version" => root.join(format!("reports/{VERSION}")),
            _ => root.join("reports"),
        };
        let saved = root.join(format!("saved-{component}"));
        fs::rename(&path, &saved).unwrap();
        std::os::unix::fs::symlink(&saved, &path).unwrap();
        fails(&root, &tip, "DEPTH_RECORD_INVALID");
    }
}

#[test]
fn merge_checks_the_result_against_the_integration_parent_not_the_stale_branch() {
    let (temp, root, base) = baseline(Some(&record()));
    git(temp.path(), &["switch", "-qc", "candidate"]);
    write_text(&temp.path().join("candidate.txt"), "candidate\n");
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-qm", "candidate"]);
    git(temp.path(), &["switch", "-qc", "integration", &base]);
    let mut rec = record();
    rec["acceptance_entries"] = json!([{ "backs": "promotion", "targets": [REQUIRED_TARGET], "record_sha256": "a".repeat(64) }]);
    write_record(&root, &rec);
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-qm", "accept"]);
    git(
        temp.path(),
        &["merge", "--no-ff", "-qm", "test merge", "candidate"],
    );
    let (status, report) = compare(&root, "HEAD^1");
    assert_eq!(status, 0, "Git kept the tip's entry: {report}");
    write_record(&root, &record());
    fails(&root, "HEAD^1", "DEPTH_ACCEPTANCE_ENTRIES_CHANGED");
}

#[test]
fn unchanged_identities_cannot_hide_policy_obligations_or_execution_edits() {
    let mut rec = record_with_obligations();
    rec["execution"] = json!({ "identity": "", "policy_identity": rec["policy"]["identity"], "obligations_identity": rec["obligations"]["identity"], "results": [], "reuse_bindings": [], "debt_grants": [], "debt_retired": [] });
    seal(&mut rec);
    let (temp, root, tip) = baseline(Some(&rec));
    for part in ["policy", "obligations", "execution"] {
        let mut changed = rec.clone();
        match part {
            "policy" => changed[part]["rules_revision"] = json!("changed"),
            "obligations" => {
                changed[part]["acquired_inputs"] =
                    json!([{ "path": "new-input", "sha256": "a".repeat(64) }])
            }
            _ => changed[part]["debt_retired"] = json!(["changed"]),
        }
        write_record(&root, &changed);
        fails(&root, &tip, "DEPTH_RECORD_INVALID");
        git(temp.path(), &["add", "."]);
        git(temp.path(), &["commit", "-qm", "invalid identity"]);
        let invalid_tip = git(temp.path(), &["rev-parse", "HEAD"]);
        write_record(&root, &rec);
        fails(&root, &invalid_tip, "DEPTH_TIP_UNREADABLE");
    }
}
