//! Maintenance observations retain IN1 admission when the parent lacks a declaration.
use super::*;

fn observation(parent_has_entry: bool, frozen_without_record: bool) {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let registry = fs::read(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
    f.registry(|doc| {
        let entries = doc["agents"].as_array_of_tables_mut().unwrap();
        let index = entries
            .iter()
            .position(|entry| entry["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap();
        if parent_has_entry {
            entries.get_mut(index).unwrap().remove("support_depth");
        } else {
            entries.remove(index);
        }
    });
    f.commit();
    let parent = head(&f);
    write(f.root.path(), REGISTRY_RELATIVE_PATH, registry);
    // An ordinary maintenance effect is outside the onboarding checkpoint's finite paths.
    write(
        f.root.path(),
        "docs/ordinary-maintenance.txt",
        "maintenance\n",
    );
    if frozen_without_record {
        write(
            f.root.path(),
            &crate::agent_lifecycle::maintenance_request_path("depth_fixture"),
            "[detected_release]\ntarget_version = \"0.2.0\"\n[support_depth]\npolicy_identity = \"frozen\"\n",
        );
    }
    let next = candidate(&f, &parent);
    f.host();
    local(&f, &next);
    let _installed = f.install();
    let before = snapshot(f.root.path());
    let result = check_integration(f.root.path(), &parent);
    if frozen_without_record {
        assert_eq!(
            result.unwrap_err(),
            "depth_fixture: request froze P without its depth record"
        );
    } else {
        result.unwrap();
        refused(&f, &next);
    }
    assert_eq!(snapshot(f.root.path()), before);
}

#[test]
fn maintenance_observation_accepts_parent_without_registry_entry() {
    observation(false, false);
}

#[test]
fn maintenance_observation_accepts_parent_without_depth_table() {
    observation(true, false);
}

#[test]
fn maintenance_observation_refuses_unbacked_freeze_with_parent_without_registry_entry() {
    observation(false, true);
}

#[test]
fn maintenance_observation_refuses_unbacked_freeze_with_parent_without_depth_table() {
    observation(true, true);
}
