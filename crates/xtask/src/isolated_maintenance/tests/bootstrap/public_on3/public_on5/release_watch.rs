//! Release-watch authority is committed before freezing the genuine connected chain.
use super::*;

pub(in super::super) fn configure(f: &Fixture) {
    let release: Value = serde_json::from_str(include_str!(
        "../../../../../../tests/fixtures/depth-synthetic/onboarding/release-watch.json"
    ))
    .unwrap();
    let mut registry: Value = toml_edit::de::from_str(
        &fs::read_to_string(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap(),
    )
    .unwrap();
    let entry = registry["agents"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["agent_id"] == "depth_fixture")
        .unwrap();
    entry["maintenance"] = json!({"release_watch": release});
    write(
        f.root.path(),
        REGISTRY_RELATIVE_PATH,
        toml_edit::ser::to_string(&registry).unwrap(),
    );
    let mut approval: Value =
        toml_edit::de::from_str(&fs::read_to_string(f.root.path().join(APPROVAL)).unwrap())
            .unwrap();
    approval["descriptor"]["maintenance"] = json!({
        "mode": "release_watch_enrolled", "release_watch": release
    });
    let raw = toml_edit::ser::to_string(&approval).unwrap();
    let mut state = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
    write(f.root.path(), APPROVAL, &raw);
    state.approval_artifact_sha256 = hex::encode(Sha256::digest(raw.as_bytes()));
    write(
        f.root.path(),
        STATE,
        serde_json::to_vec_pretty(&state).unwrap(),
    );
    f.commit();
    local(f, &head(f));
}

pub(super) fn assert_settlement(closed: &Value) {
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../../../tests/fixtures/depth-synthetic/onboarding/release-watch-settlement.json"
    ))
    .unwrap();
    assert_eq!(closed["maintenance_settlement"], expected);
}

#[test]
fn public_on5_release_watch_prepared_closed_real_in1_and_observation() {
    connected_with(false, true, Some(public_on4::Phase::CloseoutReleaseWatch));
}
