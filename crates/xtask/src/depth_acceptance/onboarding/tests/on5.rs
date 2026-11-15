//! Settlement facts are identical in the writer and independent proving predicate.
use super::*;
use crate::close_proving_run::settlement::settle_maintenance_readiness;

#[test]
fn on5_release_watch_settlement_matches_writer_and_refuses_mismatched_registry() {
    let mut f = Fixture::new(Kind::Proving);
    let release: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/depth-synthetic/onboarding/release-watch.json"
    ))
    .unwrap();
    let mut approval: Value = toml_edit::de::from_str(&f.after[APPROVAL]).unwrap();
    approval["descriptor"]["maintenance"] =
        json!({"mode":"release_watch_enrolled", "release_watch":release});
    let raw = toml_edit::ser::to_string(&approval).unwrap();
    f.after.insert(APPROVAL.into(), raw.clone());
    let mut entry = f.view().entry;
    entry.maintenance.release_watch = Some(serde_json::from_value(release.clone()).unwrap());
    // Source proposes the matching typed registry while retaining every other fixture field.
    let mut registry_value: Value =
        toml_edit::de::from_str(&f.after[REGISTRY_RELATIVE_PATH]).unwrap();
    let row = registry_value["agents"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["agent_id"] == AGENT)
        .unwrap();
    row["maintenance"] = json!({"release_watch": release});
    f.after.insert(
        REGISTRY_RELATIVE_PATH.into(),
        toml_edit::ser::to_string(&registry_value).unwrap(),
    );
    let maintenance = crate::approval_artifact::parse_approval_maintenance(&raw, APPROVAL).unwrap();
    let settled = settle_maintenance_readiness(&entry, &maintenance, Path::new(CLOSEOUT)).unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/depth-synthetic/onboarding/release-watch-settlement.json"
    ))
    .unwrap();
    assert_eq!(settled.mode.as_str(), expected["mode"]);
    assert_eq!(
        settled.approval_section_sha256,
        expected["approval_section_sha256"]
    );
    assert_eq!(
        settled.release_watch_sha256.as_deref(),
        expected["release_watch_sha256"].as_str()
    );
    assert_eq!(settled.deferral_sha256, None);
    let mut state: LifecycleState = serde_json::from_str(&f.after[STATE]).unwrap();
    state.approval_artifact_sha256 = artifacts::digest(&raw);
    f.after.insert(STATE.into(), encode(&state));
    let mut closeout: Value = serde_json::from_str(&f.after[CLOSEOUT]).unwrap();
    closeout["approval_sha256"] = json!(state.approval_artifact_sha256);
    closeout["maintenance_settlement"] = expected;
    f.after.insert(CLOSEOUT.into(), encode(&closeout));
    binding::proving(&f.view(), &state, CLOSEOUT).unwrap();
    for missing in [true, false] {
        let mut view = f.view();
        if missing {
            view.entry.maintenance.release_watch = None;
        } else {
            view.entry
                .maintenance
                .release_watch
                .as_mut()
                .unwrap()
                .upstream
                .owner = Some("different-owner".into());
        }
        let writer = settle_maintenance_readiness(&view.entry, &maintenance, Path::new(CLOSEOUT))
            .unwrap_err()
            .to_string();
        let integration = binding::proving(&view, &state, CLOSEOUT).unwrap_err();
        assert_eq!(writer, integration);
        let expected = if missing {
            "requires registry `maintenance.release_watch`"
        } else {
            "approval and registry maintenance.release_watch truth diverge"
        };
        assert!(writer.contains(expected), "{writer}");
    }
    closeout["maintenance_settlement"]["release_watch_sha256"] = json!("a".repeat(64));
    f.after.insert(CLOSEOUT.into(), encode(&closeout));
    assert_eq!(
        binding::proving(&f.view(), &state, CLOSEOUT).unwrap_err(),
        "onboarding proving maintenance settlement differs from exact approval"
    );
}

#[test]
fn on5_prepared_cannot_replace_retained_proving_authority() {
    let mut f = Fixture::new(Kind::Proving);
    let retained = f.accept().unwrap();
    f.record.acceptance_entries.push(retained);
    f.before.insert(RECORD.into(), encode(&f.record));
    let mut draft: Value = serde_json::from_str(&f.after[CLOSEOUT]).unwrap();
    draft["state"] = json!("prepared");
    draft["maintenance_settlement"] = Value::Null;
    f.after = f.before.clone();
    f.after.insert(CLOSEOUT.into(), encode(&draft));
    let view = f.view();
    let before = f.after.clone();
    assert_eq!(
        super::super::on5::prepared_batch(source(&f.before), source(&f.after), &view.registry)
            .unwrap_err(),
        "prepared ON5 cannot replace a retained proving closeout"
    );
    assert_eq!(f.after, before);
}

#[test]
fn on5_commit_identity_ignores_inherited_git_redirects() {
    const CHILD: &str = "XTASK_ON5_HEAD_IDENTITY_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let root = tempfile::tempdir().unwrap();
        for args in [
            vec!["init", "--template=", "-b", "main"],
            vec![
                "-c",
                "user.name=ON5",
                "-c",
                "user.email=on5@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "--allow-empty",
                "-qm",
                "ON5 identity",
            ],
        ] {
            assert!(crate::git_inputs::command(root.path())
                .args(args)
                .output()
                .unwrap()
                .status
                .success());
        }
        let actual = super::super::on5::head(root.path()).unwrap();
        let expected = std::fs::read_to_string(root.path().join(".git/refs/heads/main")).unwrap();
        assert_eq!(actual, expected.trim());
        return;
    }
    let foreign = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "depth_acceptance::onboarding::tests::on5::on5_commit_identity_ignores_inherited_git_redirects", "--nocapture"])
        .env(CHILD, "1").env("GIT_DIR", foreign.path()).env("GIT_WORK_TREE", foreign.path())
        .env("GIT_INDEX_FILE", foreign.path().join("forged-index")).output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
