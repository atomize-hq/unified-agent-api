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
            for row in artifact["rows"].as_array().unwrap() {
                if let Some(depth) = row.get("depth").filter(|d| !d.is_null()) {
                    for promise in depth["promises"].as_array().unwrap() {
                        assert_eq!(promise["qualified"], false);
                    }
                    for capability in depth["capabilities"].as_array().unwrap() {
                        assert_eq!(capability["depth_qualified"], false);
                    }
                }
            }
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
