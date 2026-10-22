use super::*;

#[test]
fn apply_refuses_changed_outputs_and_modified_plans_without_writes() {
    for changed_output in [false, true] {
        let fixture = fixture_root("prepare-guarded-apply");
        seed_registry(&fixture);
        seed_support_files(&fixture);
        let mut plan = build_prepare_plan(&fixture, &args()).unwrap();
        if changed_output {
            write_text(
                &fixture.join(&plan.files[0].relative_path),
                "intervening edit\n",
            );
        } else {
            plan.files[0].contents = b"caller supplied bytes\n".to_vec();
        }
        assert!(apply_prepare_plan(&fixture, &plan).is_err());
        assert!(!fixture
            .join("docs/agents/lifecycle/codex-maintenance/HANDOFF.md")
            .exists());
        if changed_output {
            assert_eq!(
                fs::read(fixture.join(&plan.files[0].relative_path)).unwrap(),
                b"intervening edit\n"
            );
        }
    }
}

#[test]
fn apply_refuses_cross_workspace_and_changed_inputs() {
    let fixture = fixture_root("prepare-input-guard");
    seed_registry(&fixture);
    seed_support_files(&fixture);
    let plan = build_prepare_plan(&fixture, &args()).unwrap();
    let other = fixture_root("prepare-other-workspace");
    seed_registry(&other);
    seed_support_files(&other);
    assert!(apply_prepare_plan(&other, &plan).is_err());
    let rules = fixture.join("cli_manifests/codex/RULES.json");
    let mut bytes = fs::read(&rules).unwrap();
    bytes.push(b'\n');
    fs::write(&rules, bytes).unwrap();
    assert!(apply_prepare_plan(&fixture, &plan).is_err());
    assert!(!fixture.join(&plan.files[0].relative_path).exists());
}

#[test]
fn apply_independently_refuses_new_retained_depth_scope() {
    let fixture = fixture_root("prepare-retained-guard");
    seed_registry(&fixture);
    seed_support_files(&fixture);
    let plan = build_prepare_plan(&fixture, &args()).unwrap();
    write_text(
        &fixture.join("cli_manifests/codex/reports/0.1.0/depth-record.json"),
        "{}",
    );
    assert!(apply_prepare_plan(&fixture, &plan).is_err());
    assert!(!fixture.join(&plan.files[0].relative_path).exists());
}

#[test]
fn ordinary_plan_cannot_survive_a_new_registry_enrollment() {
    let fixture = fixture_root("prepare-declaration-guard");
    seed_registry(&fixture);
    seed_support_files(&fixture);
    let plan = build_prepare_plan(&fixture, &args()).unwrap();
    let path = fixture.join(agent_registry::REGISTRY_RELATIVE_PATH);
    let mut document: toml_edit::DocumentMut = fs::read_to_string(&path).unwrap().parse().unwrap();
    let entry = document["agents"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["agent_id"].as_str() == Some("codex"))
        .unwrap();
    let table: toml_edit::DocumentMut = "enabled_paths = [\"maintenance\"]\n".parse().unwrap();
    entry.insert(
        "support_depth",
        toml_edit::Item::Table(table.as_table().clone()),
    );
    fs::write(path, document.to_string()).unwrap();
    assert!(apply_prepare_plan(&fixture, &plan).is_err());
    assert!(!fixture.join(&plan.request.relative_path).exists());
}
