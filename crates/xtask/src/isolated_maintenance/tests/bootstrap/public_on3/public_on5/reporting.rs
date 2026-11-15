//! Published reporting can reduce claims while retaining its historical owning entry.
use super::*;

pub(super) fn check(
    f: &Fixture,
    root: &Path,
    api: &super::super::super::super::fixture::Fake,
    parent: &str,
) {
    let packet = fs::read(root.join(PACKET)).unwrap();
    let state = fs::read(root.join(STATE)).unwrap();
    let original: crate::depth_record::DepthRecord =
        serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
    for case in ["downgrade", "qualification", "entry"] {
        eprintln!("ON5 reporting regression case {case}");
        let mut record = original.clone();
        let execution = record.execution.as_mut().unwrap();
        for row in &mut execution.results {
            row.result = crate::depth_record::DepthResult::Unverified;
            row.evidence.clear();
        }
        execution.identity = crate::depth_record::part_identity(execution).unwrap();
        if case == "entry" {
            // An appended current entry must re-enter acceptance even with no qualifications.
            let mut entry = original.acceptance_entries[1].clone();
            let crate::depth_record::AcceptanceEntry::ProvingRunCloseout { record_sha256, .. } =
                &mut entry
            else {
                panic!("missing proving entry")
            };
            *record_sha256 = record.stated_sha256().unwrap();
            record.acceptance_entries.push(entry);
        }
        write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
        if case != "qualification" {
            // Standalone readers require a clean committed candidate before rendering.
            commit(root, api, &f.baseline, parent);
            let bundle = crate::support_matrix::generate_publication_artifacts(root).unwrap();
            write(root, crate::support_matrix::JSON_OUTPUT_PATH, bundle.json);
            write(
                root,
                crate::support_matrix::MARKDOWN_OUTPUT_PATH,
                bundle.markdown,
            );
            write(
                root,
                crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
                bundle.runtime_support_data,
            );
            let artifact: Value = serde_json::from_slice(
                &fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
            )
            .unwrap();
            // The wrapper-only enrollment declares one promise and no capability mappings.
            assert_downgraded(&artifact, &["run"], &[]);
        }
        commit(root, api, &f.baseline, parent);
        assert_eq!(fs::read(root.join(PACKET)).unwrap(), packet);
        assert_eq!(fs::read(root.join(STATE)).unwrap(), state);
        let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
        let outcome = check_integration(root, parent);
        if case == "downgrade" {
            assert_eq!(record.acceptance_entries, original.acceptance_entries);
            outcome.unwrap();
        } else {
            let error = outcome.unwrap_err();
            assert!(
                error.contains("onboarding obligation is not independently satisfied"),
                "{case}: {error}"
            );
        }
        assert_eq!(
            crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
            before
        );
        git(root, &["reset", "--hard", parent]);
    }
}

/// Pin the subject and complete inventory before evaluating qualification flags.
fn assert_downgraded(artifact: &Value, expected_promises: &[&str], expected_capabilities: &[&str]) {
    assert!(!expected_promises.is_empty());
    let rows: Vec<_> = artifact["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["agent"] == "depth_fixture")
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "expected the synthetic subject's support row"
    );
    let depth = &rows[0]["depth"];
    let promises = depth["promises"].as_array().unwrap();
    let capabilities = depth["capabilities"].as_array().unwrap();
    let actual_promises: Vec<_> = promises
        .iter()
        .map(|p| p["operation"].as_str().unwrap())
        .collect();
    let actual_capabilities: Vec<_> = capabilities
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(actual_promises, expected_promises);
    assert_eq!(actual_capabilities, expected_capabilities);
    for promise in promises {
        assert_eq!(promise["qualified"], false);
    }
    for capability in capabilities {
        assert_eq!(capability["depth_qualified"], false);
    }
}

#[test]
fn public_on5_downgrade_assertions_require_subject_inventory_and_flags() {
    let complete = json!({"rows": [{"agent": "depth_fixture", "depth": {
        "promises": [{"operation": "run", "qualified": false}],
        "capabilities": [{"id": "agent_api.run", "depth_qualified": false}]
    }}]});
    assert_downgraded(&complete, &["run"], &["agent_api.run"]);
    for case in [
        "row",
        "depth",
        "promises",
        "capabilities",
        "qualified",
        "depth_qualified",
    ] {
        let mut artifact = complete.clone();
        let row = &mut artifact["rows"][0];
        match case {
            "row" => row["agent"] = json!("other"),
            "depth" => row["depth"] = Value::Null,
            "promises" | "capabilities" => row["depth"][case] = json!([]),
            "qualified" => row["depth"]["promises"][0][case] = json!(true),
            "depth_qualified" => row["depth"]["capabilities"][0][case] = json!(true),
            _ => unreachable!(),
        }
        assert!(
            std::panic::catch_unwind(|| {
                assert_downgraded(&artifact, &["run"], &["agent_api.run"])
            })
            .is_err(),
            "{case} escaped the downgrade assertions"
        );
    }
}
