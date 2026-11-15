use super::*;

/// MA4's actual final checker must reject these effects even on an admitted ON3/ON5 record.
pub(super) fn relay(root: &Path) {
    let parent = git(root, &["rev-parse", "HEAD"]);
    for (path, bytes, reason) in [
        (
            RECORD.to_owned(),
            None,
            "relay cannot add/remove a depth generation",
        ),
        (
            REGISTRY_RELATIVE_PATH.to_owned(),
            Some(b"changed registry authority".to_vec()),
            "relay cannot change registry authority",
        ),
    ] {
        let original = fs::read(root.join(&path)).unwrap();
        match bytes {
            Some(bytes) => write(root, &path, bytes),
            None => fs::remove_file(root.join(&path)).unwrap(),
        }
        unchanged(root, reason, || {
            crate::depth_acceptance::check_reporting_relay(root, &parent).unwrap_err()
        });
        write(root, &path, original);
    }
}

/// Real all-root IN1, plus the ordinary-root --tip adapter for a foreign removed record.
pub(super) fn committed(root: &Path, parent: &str, commit: impl Fn()) {
    for (case, expected) in [
        ("removed_record", "candidate record missing for depth_fixture/0.2.0"),
        ("omitted_declaration", "isolated governing registry differs from protected staging"),
        ("disabled_declaration", "isolated governing registry differs from protected staging"),
        ("malformed_record", "depth record: schema_version must be 1"),
        ("deleted_binding", "missing bound input: docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/lifecycle-state.json"),
    ] {
        match case {
            "removed_record" => fs::remove_file(root.join(RECORD)).unwrap(),
            "omitted_declaration" | "disabled_declaration" => {
                let mut doc: toml_edit::DocumentMut = fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH)).unwrap().parse().unwrap();
                let entry = doc["agents"].as_array_of_tables_mut().unwrap().iter_mut()
                    .find(|a| a["agent_id"].as_str() == Some("depth_fixture")).unwrap();
                if case == "omitted_declaration" {
                    entry.remove("support_depth").unwrap();
                } else {
                    entry["support_depth"].as_table_mut().unwrap().remove("enabled_paths").unwrap();
                }
                write(root, REGISTRY_RELATIVE_PATH, doc.to_string());
            }
            "malformed_record" => {
                let mut record: serde_json::Value = serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
                record["schema_version"] = json!(2);
                write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
            }
            "deleted_binding" => fs::remove_file(root.join(STATE)).unwrap(),
            _ => unreachable!(),
        }
        commit();
        eprintln!("E.8 real IN1 refusal {case}");
        unchanged(root, expected, || crate::depth_acceptance::check_integration(root, parent).unwrap_err());
        if case == "removed_record" {
            let before = super::super::unsupported_routes::tree(root);
            let codes = crate::manifest_validate::violation_codes(crate::manifest_validate::Args {
                root: Some(root.join("cli_manifests/codex")), rules: None, schema: None,
                version_schema: None, mode: crate::manifest_validate::Mode::Check,
                json: false, tip: Some(parent.into()),
            }).unwrap();
            assert!(codes.contains(&"DEPTH_ADMISSION_REFUSED"), "ordinary requested root lost all-root admission: {codes:?}");
            assert_eq!(super::super::unsupported_routes::tree(root), before);
        }
        git(root, &["reset", "--hard", parent]);
    }
    write(
        root,
        "cli_manifests/depth_fixture/pointers/latest_validated/x86_64-unknown-linux-gnu.txt",
        "0.2.0\n",
    );
    commit();
    unchanged(root, "relay acceptance effects forbidden:", || {
        crate::depth_acceptance::check_reporting_relay(root, parent).unwrap_err()
    });
    git(root, &["reset", "--hard", parent]);
}
