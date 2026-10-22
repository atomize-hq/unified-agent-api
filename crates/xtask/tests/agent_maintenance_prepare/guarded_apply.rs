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

#[test]
fn binary_report_and_governance_inputs_bind_raw_bytes() {
    let fixture = fixture_root("prepare-binary-inputs");
    seed_registry(&fixture);
    seed_support_files(&fixture);
    let report = fixture.join("cli_manifests/codex/reports/0.1.0/raw-evidence.bin");
    let governance =
        fixture.join("docs/agents/lifecycle/codex-maintenance/governance/raw-evidence.bin");
    for path in [&report, &governance] {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, [0xff, 0x00, 0x80]).unwrap();
    }
    let plan = build_prepare_plan(&fixture, &args()).unwrap();
    fs::write(&report, [0xfe, 0x00, 0x80]).unwrap();
    assert!(apply_prepare_plan(&fixture, &plan).is_err());
    assert!(!fixture.join(&plan.request.relative_path).exists());
    let fresh = build_prepare_plan(&fixture, &args()).unwrap();
    apply_prepare_plan(&fixture, &fresh).unwrap();
    assert_eq!(fs::read(&report).unwrap(), [0xfe, 0x00, 0x80]);
    assert_eq!(fs::read(&governance).unwrap(), [0xff, 0x00, 0x80]);
}

#[test]
fn index_only_addition_under_a_bound_directory_invalidates_an_ordinary_plan() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let fixture = fixture_root("prepare-index-membership");
    seed_registry(&fixture);
    seed_support_files(&fixture);
    for args in [vec!["init", "-q"], vec!["add", "."]] {
        assert!(Command::new("git")
            .arg("-C")
            .arg(&fixture)
            .args(args)
            .status()
            .unwrap()
            .success());
    }
    let plan = build_prepare_plan(&fixture, &args()).unwrap();
    let mut child = Command::new("git")
        .arg("-C")
        .arg(&fixture)
        .args(["hash-object", "-w", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"index-only input")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let oid = String::from_utf8(out.stdout).unwrap();
    let path = "cli_manifests/codex/reports/0.98.0/coverage.index-only.json";
    assert!(Command::new("git")
        .arg("-C")
        .arg(&fixture)
        .args([
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("100644,{},{}", oid.trim(), path)
        ])
        .status()
        .unwrap()
        .success());
    assert!(!fixture.join(path).exists());
    assert!(apply_prepare_plan(&fixture, &plan).is_err());
    assert!(!fixture.join(&plan.request.relative_path).exists());
}
