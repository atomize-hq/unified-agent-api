//! Real proposal and committed-candidate refusals preserve every caller path.
use super::*;
use crate::{
    candidate_execution::tree::{self, Tree},
    depth_acceptance::onboarding::on5,
};
use std::path::PathBuf;
type Files = Vec<(PathBuf, Vec<u8>)>;

fn closed_draft(_: &Path, files: &mut Files) -> Result<(), String> {
    let (_, bytes) = files
        .iter_mut()
        .find(|(p, _)| p == Path::new(CLOSEOUT))
        .unwrap();
    let mut value: Value = serde_json::from_slice(bytes).unwrap();
    value["state"] = json!("closed");
    *bytes = serde_json::to_vec_pretty(&value).unwrap();
    Ok(())
}
fn pointer(_: &Path, files: &mut Files) -> Result<(), String> {
    files.push((
        format!("{ROOT}/min_supported.txt").into(),
        b"0.2.0\n".to_vec(),
    ));
    Ok(())
}
fn forged(root: &Path, files: &mut Files) -> Result<(), String> {
    let mut record: DepthRecord =
        serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
    record
        .acceptance_entries
        .push(AcceptanceEntry::ProvingRunCloseout {
            closeout: crate::depth_record::FileRef {
                path: CLOSEOUT.into(),
                sha256: "a".repeat(64),
            },
            record_sha256: record.stated_sha256().unwrap(),
        });
    files.push((RECORD.into(), serde_json::to_vec_pretty(&record).unwrap()));
    Ok(())
}
fn retained(_: &Path, files: &mut Files) -> Result<(), String> {
    let (_, bytes) = files
        .iter_mut()
        .find(|(p, _)| p == Path::new(RECORD))
        .unwrap();
    let mut record: DepthRecord = serde_json::from_slice(bytes).unwrap();
    let AcceptanceEntry::Publication { packet, .. } = &mut record.acceptance_entries[0] else {
        panic!("publication entry missing")
    };
    packet.sha256 = "a".repeat(64);
    *bytes = serde_json::to_vec_pretty(&record).unwrap();
    Ok(())
}
fn absent(_: &Path, files: &mut Files) -> Result<(), String> {
    let (_, bytes) = files
        .iter_mut()
        .find(|(p, _)| p == Path::new(RECORD))
        .unwrap();
    let mut record: DepthRecord = serde_json::from_slice(bytes).unwrap();
    record.acceptance_entries.pop().unwrap();
    *bytes = serde_json::to_vec_pretty(&record).unwrap();
    Ok(())
}

pub(super) fn prepared(root: &Path, before: &Tree) {
    for (hook, expected) in [
        (
            closed_draft as fn(&Path, &mut Files) -> Result<(), String>,
            "prepared ON5 requires its exact reporting-only closeout",
        ),
        (
            pointer,
            "onboarding cannot change pointer file cli_manifests/depth_fixture/min_supported.txt",
        ),
        (
            forged,
            "onboarding batch has an entry outside its owning path",
        ),
    ] {
        eprintln!("ON5 prepared proposal refusal expects {expected}");
        let error = on5::proposal_hooks::with_proposal(hook, || prepare(root, false))
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
        assert_eq!(&tree::capture(root, &Default::default()).unwrap(), before);
    }
    let error = on5::test_hooks::with_delivery_failure(1, || prepare(root, false))
        .unwrap_err()
        .to_string();
    assert_eq!(error, "injected repair delivery failure");
    assert_eq!(&tree::capture(root, &Default::default()).unwrap(), before);
}

pub(super) fn local(
    f: &Fixture,
    root: &Path,
    api: &super::super::super::super::fixture::Fake,
    human: &str,
) {
    let before = tree::capture(root, &Default::default()).unwrap();
    for (hook, expected) in [
        (
            retained as fn(&Path, &mut Files) -> Result<(), String>,
            "onboarding candidate changed retained entries",
        ),
        (
            absent,
            "onboarding proposal lacks its exact current owning entry",
        ),
    ] {
        eprintln!("ON5 final proposal refusal expects {expected}");
        let error = on5::proposal_hooks::with_proposal(hook, || close(root))
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
        assert_eq!(tree::capture(root, &Default::default()).unwrap(), before);
    }
    for case in ["acquired", "runtime", "drifted"] {
        if case == "acquired" {
            let record: DepthRecord =
                serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
            let input = &record.obligations.as_ref().unwrap().acquired_inputs[0].path;
            let mut bytes = fs::read(root.join(input)).unwrap();
            bytes.push(b'\n');
            write(root, input, bytes);
        } else if case == "drifted" {
            let mut state = life::load_lifecycle_state(root, STATE).unwrap();
            state.side_states.push(life::SideState::Drifted);
            write(root, STATE, serde_json::to_vec_pretty(&state).unwrap());
        } else {
            let packet: life::PublicationReadyPacket =
                serde_json::from_slice(&fs::read(root.join(PACKET)).unwrap()).unwrap();
            let path = packet
                .runtime_evidence_paths
                .iter()
                .find(|p| p.ends_with("validation-report.json"))
                .unwrap();
            let mut value: Value =
                serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap();
            value["status"] = json!("fail");
            write(root, path, serde_json::to_vec_pretty(&value).unwrap());
        }
        commit(root, api, &f.baseline, human);
        let changed = tree::capture(root, &Default::default()).unwrap();
        let prepared = prepare(root, true).unwrap_err().to_string();
        let final_error = close(root).unwrap_err().to_string();
        assert_eq!(
            prepared, final_error,
            "both ON5 commands must check the same frozen facts"
        );
        let expected = match case {
            "acquired" => "changed or missing frozen acquired inputs",
            "runtime" => "validation-report.json is not green",
            "drifted" => "onboarding cannot clear maintenance drift",
            _ => unreachable!(),
        };
        assert!(prepared.contains(expected), "{case}: {prepared}");
        assert_eq!(tree::capture(root, &Default::default()).unwrap(), changed);
        git(root, &["reset", "--hard", human]);
    }
}

pub(super) fn integration(
    f: &Fixture,
    root: &Path,
    api: &super::super::super::super::fixture::Fake,
    parent: &str,
    candidate: &str,
) {
    for case in [
        "entry",
        "retained",
        "closeout",
        "approval_source",
        "explicit_none_reason",
        "runtime",
        "written_paths",
        "evidence",
        "transfer",
        "maintenance",
    ] {
        eprintln!("ON5 IN1 refusal case {case}");
        match case {
            "entry" | "retained" => {
                let mut record: DepthRecord =
                    serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
                if case == "entry" {
                    let AcceptanceEntry::ProvingRunCloseout { closeout, .. } =
                        &mut record.acceptance_entries[1]
                    else {
                        panic!("proving entry missing")
                    };
                    closeout.sha256 = "a".repeat(64);
                } else {
                    record.acceptance_entries.remove(0);
                }
                write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
            }
            "closeout" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(root.join(CLOSEOUT)).unwrap()).unwrap();
                value["recorded_at"] = json!("2026-11-05T00:00:00Z");
                write(root, CLOSEOUT, serde_json::to_vec_pretty(&value).unwrap());
            }
            "approval_source" | "explicit_none_reason" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(root.join(CLOSEOUT)).unwrap()).unwrap();
                value[case] = json!(" \t\n ");
                let bytes = serde_json::to_vec_pretty(&value).unwrap();
                write(root, CLOSEOUT, &bytes);
                let mut record: DepthRecord =
                    serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
                let AcceptanceEntry::ProvingRunCloseout { closeout, .. } =
                    &mut record.acceptance_entries[1]
                else {
                    panic!("proving entry missing")
                };
                closeout.sha256 = hex::encode(Sha256::digest(&bytes));
                write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
            }
            "runtime" | "written_paths" => {
                let packet: life::PublicationReadyPacket =
                    serde_json::from_slice(&fs::read(root.join(PACKET)).unwrap()).unwrap();
                let name = if case == "runtime" {
                    "validation-report.json"
                } else {
                    "written-paths.json"
                };
                let path = packet
                    .runtime_evidence_paths
                    .iter()
                    .find(|p| p.ends_with(name))
                    .unwrap();
                if case == "runtime" {
                    let mut value: Value =
                        serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap();
                    value["status"] = json!("fail");
                    write(root, path, serde_json::to_vec_pretty(&value).unwrap());
                } else {
                    let mut bytes = fs::read(root.join(path)).unwrap();
                    bytes.push(b'\n');
                    write(root, path, bytes);
                }
            }
            "evidence" => {
                let record: DepthRecord =
                    serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
                let path = &record
                    .execution
                    .as_ref()
                    .unwrap()
                    .results
                    .iter()
                    .find(|row| !row.evidence.is_empty())
                    .unwrap()
                    .evidence[0]
                    .path;
                let mut bytes = fs::read(root.join(path)).unwrap();
                bytes.push(b'\n');
                write(root, path, bytes);
            }
            "transfer" => {
                let raw = fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH))
                    .unwrap()
                    .replace("\"onboarding\"", "\"maintenance\"");
                write(root, REGISTRY_RELATIVE_PATH, raw);
            }
            "maintenance" => {
                let mut record: DepthRecord =
                    serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
                record.acceptance_entries.pop();
                record
                    .acceptance_entries
                    .push(AcceptanceEntry::MaintenanceCloseout {
                        closeout: crate::depth_record::FileRef {
                            path: CLOSEOUT.into(),
                            sha256: hex::encode(Sha256::digest(
                                fs::read(root.join(CLOSEOUT)).unwrap(),
                            )),
                        },
                        record_sha256: record.stated_sha256().unwrap(),
                    });
                write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
            }
            _ => unreachable!(),
        }
        git(root, &["reset", "--soft", parent]);
        commit(root, api, &f.baseline, parent);
        let before = tree::capture(root, &Default::default()).unwrap();
        let error = check_integration(root, parent).unwrap_err();
        let expected = match case {
            "entry" | "closeout" | "maintenance" => {
                "integration proving lacks its exact current owning entry"
            }
            "approval_source" => "`approval_source` must not be empty",
            "explicit_none_reason" => "`explicit_none_reason` must not be empty",
            "retained" => "integration changed retained entries or shrank selection",
            "runtime" => "validation-report.json is not green",
            "evidence" => "onboarding obligation is not independently satisfied",
            "written_paths" => "written-paths.json differs from lifecycle written_paths_sha256",
            "transfer" => "isolated governing registry differs from protected staging",
            _ => unreachable!(),
        };
        assert!(error.contains(expected), "{case}: {error}");
        assert_eq!(tree::capture(root, &Default::default()).unwrap(), before);
        git(root, &["reset", "--hard", candidate]);
    }
}

/// Unchanged lifecycle still validates every prepared artifact semantic field at IN1.
pub(super) fn prepared_candidate(
    f: &Fixture,
    root: &Path,
    api: &super::super::super::super::fixture::Fake,
    parent: &str,
    prepared: &str,
) {
    let mut draft: Value = serde_json::from_slice(&fs::read(root.join(CLOSEOUT)).unwrap()).unwrap();
    draft["duration_seconds"] = json!(17);
    assert!(draft["duration_missing_reason"].is_string());
    write(root, CLOSEOUT, serde_json::to_vec_pretty(&draft).unwrap());
    git(root, &["reset", "--soft", parent]);
    commit(root, api, &f.baseline, parent);
    let before = tree::capture(root, &Default::default()).unwrap();
    let error = check_integration(root, parent).unwrap_err();
    assert!(
        error
            .contains("exactly one of `duration_seconds` or `duration_missing_reason` is required"),
        "{error}"
    );
    assert_eq!(tree::capture(root, &Default::default()).unwrap(), before);
    git(root, &["reset", "--hard", prepared]);
}
