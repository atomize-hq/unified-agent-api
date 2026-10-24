use super::*;
use xtask::{
    agent_registry::AgentRegistry,
    runtime_advertising::{projection, DATA},
};

fn claim(record: &mut Value, cap: &str) {
    record["policy"]["selection"]["enrollment"]["operations"][0]["shared_mapping"] =
        json!("mapped");
    record["policy"]["selection"]["enrollment"]["capabilities"] =
        json!([{"id":cap,"operations":["exec"]}]);
    seal(record);
}
fn regenerate(temp: &Path) {
    let registry = AgentRegistry::load(temp).unwrap();
    write_text(
        &temp.join(DATA),
        &projection::render(&projection::project(&registry).unwrap()),
    );
}
fn change(temp: &Path, path: &str, from: &str, to: &str) {
    let path = temp.join(path);
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains(from), "{from}");
    write_text(&path, &text.replacen(from, to, 1));
}
#[test]
fn opt_in_runtime_advertising_and_default_feature_additions_need_current_entries() {
    for default_feature in [false, true] {
        let mut rec = record();
        claim(&mut rec, "agent_api.tools.mcp.add.v1");
        let (temp, root, tip) = baseline(Some(&rec));
        assert!(requirements(temp.path(), &tip).is_empty());
        if default_feature {
            change(
                temp.path(),
                "crates/agent_api/Cargo.toml",
                "default = []",
                "default = [\"codex\"]",
            );
        } else {
            change(
                temp.path(),
                "crates/xtask/data/agent_registry.toml",
                "config_key = \"allow_mcp_write\"",
                "config_key = \"allow_external_sandbox_exec\"",
            );
            regenerate(temp.path());
        }
        let report = requirements(temp.path(), &tip);
        assert!(report[0]
            .effects
            .iter()
            .any(|e| e.kind == "runtime advertising agent_api.tools.mcp.add.v1"));
        let (status, diagnostic) = compare_as_published(&root, &tip, "manifest-validate", &[]);
        assert_eq!(status, 2, "{diagnostic}");
        assert!(
            diagnostic.contains("DEPTH_ACCEPTANCE_ENTRY_MISSING"),
            "{diagnostic}"
        );
        matching(&mut rec);
        write_record(&root, &rec);
        assert!(requirements(temp.path(), &tip).is_empty());
        let (status, diagnostic) = compare_as_published(&root, &tip, "manifest-validate", &[]);
        assert_eq!(status, 2, "{diagnostic}");
        assert!(
            diagnostic.contains("DEPTH_ADMISSION_REFUSED"),
            "{diagnostic}"
        );
    }
}
#[test]
fn canonical_capability_cells_charge_every_claiming_version_without_target_subset_filtering() {
    let mut rec = record();
    claim(&mut rec, "agent_api.run");
    let (temp, root, _) = baseline(Some(&rec));
    let mut second = rec.clone();
    second["policy"]["selection"]["enrollment"]["version"] = json!("0.60.0");
    seal(&mut second);
    write_json(&root.join("reports/0.60.0/depth-record.json"), &second);
    let mut metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join(format!("versions/{VERSION}.json"))).unwrap(),
    )
    .unwrap();
    metadata["semantic_version"] = json!("0.60.0");
    write_json(&root.join("versions/0.60.0.json"), &metadata);
    republish(&root);
    let tip = commit_all(temp.path());
    // Publisher-stage transition regenerates the actual canonical matrix.
    lifecycle::setup(temp.path(), &root, false);
    let missing = requirements(temp.path(), &tip);
    assert_eq!(missing.len(), 2);
    assert!(missing.iter().all(|r| r
        .effects
        .iter()
        .any(|e| e.kind == "published capability agent_api.run")));
    for (version, record) in [(VERSION, &mut rec), ("0.60.0", &mut second)] {
        let parsed =
            xtask::depth_record::DepthRecord::parse(&record.to_string(), "codex", version).unwrap();
        record["acceptance_entries"] = json!([{"backs":"promotion","targets":["unrelated"],"record_sha256":parsed.stated_sha256().unwrap()}]);
        write_json(
            &root.join(format!("reports/{version}/depth-record.json")),
            record,
        );
    }
    republish(&root);
    assert!(requirements(temp.path(), &tip).is_empty());
}
#[test]
fn qualification_additions_are_detected_even_when_continuous_freshness_also_fails() {
    let mut rec = record();
    claim(&mut rec, "agent_api.run");
    let (temp, root, tip) = baseline(Some(&rec));
    let path = temp
        .path()
        .join("cli_manifests/support_matrix/current.json");
    let mut artifact: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let row = artifact["rows"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| {
            r["agent"] == "codex" && r["version"] == VERSION && r["target"] == REQUIRED_TARGET
        })
        .unwrap();
    row["depth"]["promises"][0]["qualified"] = json!(true);
    row["depth"]["capabilities"][0]["depth_qualified"] = json!(true);
    write_json(&path, &artifact);
    let parsed: xtask::support_matrix::SupportMatrixArtifact =
        serde_json::from_value(artifact).unwrap();
    let markdown = xtask::support_matrix::render_markdown_projection(&parsed.rows);
    write_text(&temp.path().join(xtask::support_matrix::MARKDOWN_OUTPUT_PATH), &format!("<!-- support-matrix-published:start -->\n{markdown}\n<!-- support-matrix-published:end -->\n"));
    let report = xtask::depth_acceptance::compare(temp.path(), &tip, "codex").unwrap();
    assert!(!report.input_issues.is_empty());
    assert_eq!(report.missing_entries.len(), 1);
    assert!(report.effects.iter().any(|e| e.kind == "promise exec"));
    assert!(report
        .effects
        .iter()
        .any(|e| e.kind == "capability agent_api.run"));
    matching(&mut rec);
    write_record(&root, &rec);
    // Entry removes check 9 failure even though the unrelated fabricated qualification stays invalid.
    assert!(xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
        .unwrap()
        .missing_entries
        .is_empty());
}
#[test]
fn malformed_immutable_publications_cannot_be_repaired_by_the_candidate() {
    let (temp, root, _) = baseline(Some(&record()));
    let path = temp
        .path()
        .join("cli_manifests/support_matrix/current.json");
    let valid = fs::read_to_string(&path).unwrap();
    write_text(&path, &valid.replacen('{', "{\"schema_version\":1,", 1));
    let tip = commit_all(temp.path());
    republish(&root);
    assert!(xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
        .unwrap_err()
        .contains("duplicate JSON"));
}

fn regression_remove_host_gates(temp: &Path) {
    let path = temp.join(xtask::agent_registry::REGISTRY_RELATIVE_PATH);
    let mut doc = fs::read_to_string(&path)
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    for agent in doc["agents"].as_array_of_tables_mut().unwrap().iter_mut() {
        let caps = agent["capability_declaration"].as_table_mut().unwrap();
        caps.remove("target_gated");
        caps.remove("config_gated");
    }
    fs::write(path, doc.to_string()).unwrap();
    let registry = xtask::agent_registry::AgentRegistry::load(temp).unwrap();
    let source = xtask::runtime_advertising::projection::render(
        &xtask::runtime_advertising::projection::project(&registry).unwrap(),
    );
    write_text(&temp.join(xtask::runtime_advertising::DATA), &source);
}
fn regression_claimed_record() -> Value {
    let mut rec = record();
    rec["policy"]["selection"]["enrollment"]["operations"][0]["shared_mapping"] = json!("mapped");
    rec["policy"]["selection"]["enrollment"]["capabilities"] =
        json!([{ "id":"agent_api.run", "operations":["exec"] }]);
    seal(&mut rec);
    rec
}
#[test]
fn regression_other_host_default_feature_addition_must_require_entry() {
    use xtask::runtime_advertising::revision::{Context, Revision};
    let rec = regression_claimed_record();
    let (temp, root, _) = baseline(Some(&rec));
    regression_remove_host_gates(temp.path());
    republish(&root);
    let tip = commit_all(temp.path());
    let path = temp.path().join("crates/agent_api/Cargo.toml");
    let mut doc = fs::read_to_string(&path)
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    let mut defaults = toml_edit::Array::new();
    defaults.push("codex");
    doc["features"]["default"] = toml_edit::value(defaults);
    fs::write(path, doc.to_string()).unwrap();
    let before = Revision::tip(temp.path(), &tip).unwrap();
    let after = Revision::workspace(temp.path()).unwrap();
    let context = Context {
        os: "linux",
        arch: "x86_64",
        write: false,
        sandbox: false,
        requested: 0,
        default_features: true,
        default_config: true,
    };
    assert!(!before
        .capabilities("codex", context)
        .unwrap()
        .contains("agent_api.run"));
    assert!(after
        .capabilities("codex", context)
        .unwrap()
        .contains("agent_api.run"));
    let additions = after.additions(&before).unwrap();
    assert!(additions.iter().any(|a| a.agent == "codex"
        && a.capability == "agent_api.run"
        && a.os == "other"
        && a.arch == "other"));
    let report = xtask::depth_acceptance::compare(temp.path(), &tip, "codex").unwrap();
    println!(
        "effect={:?} issues={:?}",
        report.effects, report.input_issues
    );
    assert_eq!(
        report.missing_entries.len(),
        1,
        "actual runtime addition needs entry"
    );
}
#[test]
fn regression_malformed_baseline_metadata_must_be_rejected() {
    let mut rec = record();
    matching(&mut rec);
    let (temp, root, _) = baseline(Some(&rec));
    let path = root.join(format!("versions/{VERSION}.json"));
    let original: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut invalid = original.clone();
    invalid["status"] = json!({"not":"a status"});
    write_json(&path, &invalid);
    let tip = commit_all(temp.path());
    write_json(&path, &original);
    let report = xtask::depth_acceptance::compare(temp.path(), &tip, "codex");
    println!("malformed baseline result={report:?}");
    assert!(
        report.is_err(),
        "a malformed old comparison input cannot become normal prior state"
    );
}

#[test]
fn runtime_additions_map_existing_arm_and_x64_target_aliases_and_reject_unknown_labels() {
    let mut rec = record();
    claim(&mut rec, "agent_api.run");
    rec["policy"]["selection"]["enrollment"]["targets"] =
        json!(["linux-arm64", "darwin-x64", "win32-arm64"]);
    seal(&mut rec);
    let (temp, root, tip) = baseline(Some(&rec));
    change(
        temp.path(),
        "crates/agent_api/Cargo.toml",
        "default = []",
        "default = [\"codex\"]",
    );
    let missing = requirements(temp.path(), &tip);
    for target in ["linux-arm64", "darwin-x64", "win32-arm64"] {
        assert!(missing[0]
            .effects
            .iter()
            .any(|e| e.kind == "runtime advertising agent_api.run"
                && e.target.as_deref() == Some(target)));
    }
    matching(&mut rec);
    write_record(&root, &rec);
    republish(&root);
    assert!(requirements(temp.path(), &tip).is_empty());
    rec["policy"]["selection"]["enrollment"]["targets"] = json!(["unknown-host"]);
    seal(&mut rec);
    write_record(&root, &rec);
    republish(&root);
    assert!(xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
        .unwrap_err()
        .contains("unresolved advertising target label"));
}
#[test]
fn corrupt_immutable_capability_format_is_not_an_empty_generated_document() {
    let (temp, root, _) = baseline(Some(&record()));
    write_text(
        &temp.path().join(xtask::capability_matrix::DEFAULT_OUT_PATH),
        "# Capability matrix\nmalformed\n",
    );
    let tip = commit_all(temp.path());
    republish(&root);
    assert!(xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
        .unwrap_err()
        .contains("capability matrix preamble"));
}
