//! SA-P3: the binding selects only between compiled subjects. The synthetic subject is a
//! machinery proof (MP-P1); it never widens or replaces the Codex qualification scope.
use super::*;

fn edit_entry(f: &Fixture, agent: &str, edit: impl FnOnce(&mut toml_edit::Table)) {
    f.registry(|doc| {
        let entry = doc["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some(agent))
            .unwrap();
        edit(entry);
    });
}

#[test]
fn synthetic_wrapper_only_subject_resolves_without_a_capability() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    f.resolve().unwrap().recheck(f.root.path()).unwrap();
    let path = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
    write(f.root.path(), path, f.retained());
    f.commit();
    f.resolve().unwrap();
}

#[test]
fn compiled_subjects_never_cross() {
    // A Codex declaration does not satisfy the synthetic profile, and the reverse.
    for (declared, profile) in [
        (&scope::CODEX, &scope::SYNTHETIC),
        (&scope::SYNTHETIC, &scope::CODEX),
    ] {
        let mut f = Fixture::new_for(declared);
        f.subject = profile;
        assert!(f.resolve().is_err(), "{} admitted", declared.agent_id);
    }
}

#[test]
fn synthetic_selection_rejects_scope_expansion() {
    for edit in 0..10 {
        let f = Fixture::new_for(&scope::SYNTHETIC);
        edit_entry(&f, "depth_fixture", |agent| {
            let d = &mut agent["support_depth"]["enrollments"][0];
            match edit {
                // The baseline release is not the maintained version.
                0 => d["version"] = toml_edit::value("0.1.0"),
                1 => d["targets"][0] = toml_edit::value("x86_64-unknown-linux-musl"),
                2 => d["modes"][0]["command_path"] = toml_edit::value("depth_fixture exec"),
                3 => {
                    d["modes"][0]["selector"] = toml_edit::value("run");
                    d["operations"][0]["serves"][0] = toml_edit::value("run");
                }
                4 => d["modes"][0]["predicates"][0]["flag"] = toml_edit::value("--model"),
                // 5 and 6 are refused by the registry contract itself (SA-P2: a non-runtime entry
                // declares no mapping; CORE never forwards); the scope check stays as defense.
                5 => d["operations"][0]["shared_mapping"] = toml_edit::value("mapped"),
                6 => d["operations"][0]["forwarding"] = toml_edit::value(true),
                8 => d["modes"][0]["mode"] = toml_edit::value("interactive"),
                7 => {
                    d["operations"][0]["not_applicable_permitted"] =
                        toml_edit::value(["X4"].into_iter().collect::<toml_edit::Array>())
                }
                _ => d["modes"][0]["predicates"][0]["value"] = toml_edit::value("true"),
            }
        });
        f.commit();
        assert!(f.resolve().is_err(), "accepted synthetic expansion {edit}");
    }
}

#[test]
fn synthetic_profile_refuses_codex_depth_and_foreign_retained_roots() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    // A second depth declaration, here on Codex, is an extra registered scope.
    let codex: crate::agent_registry::support_depth::SupportDepth =
        serde_json::from_value(fixture::declaration()).unwrap();
    let codex = toml_edit::ser::to_document(&codex).unwrap();
    edit_entry(&f, "codex", |agent| {
        agent.insert(
            "support_depth",
            toml_edit::Item::Table(codex.as_table().clone()),
        );
    });
    f.commit();
    assert!(f.resolve().is_err(), "extra Codex depth admitted");

    let f = Fixture::new_for(&scope::SYNTHETIC);
    for path in [
        "cli_manifests/codex/reports/0.156.1/depth-record.json",
        "cli_manifests/depth_fixture/reports/0.1.0/depth-record.json",
        "reports/0.2.0/depth-record.json",
    ] {
        write(f.root.path(), path, f.retained());
        f.commit();
        assert!(f.resolve().is_err(), "accepted {path}");
        fs::remove_file(f.root.path().join(path)).unwrap();
        f.commit();
    }
    edit_entry(&f, "depth_fixture", |agent| {
        agent.remove("support_depth");
    });
    f.commit();
    assert!(f.resolve().is_err());
}

#[test]
fn synthetic_subject_must_stay_non_runtime() {
    // A backend makes the entry a runtime built-in, which then may carry capability claims.
    let f = Fixture::new_for(&scope::SYNTHETIC);
    edit_entry(&f, "depth_fixture", |agent| {
        agent.insert(
            "backend_module",
            toml_edit::value("crates/agent_api/src/backends/depth_fixture"),
        );
        agent["capability_declaration"]["always_on"] =
            toml_edit::value(["agent_api.run"].into_iter().collect::<toml_edit::Array>());
    });
    f.commit();
    assert!(f
        .resolve()
        .err()
        .unwrap()
        .contains("runtime membership differs"));
}

#[test]
fn frozen_synthetic_profile_record_matches_the_compiled_subject() {
    // The evidence profile's frozen P selects exactly what the binding admits, and Codex's does not.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let raw = fs::read_to_string(root.join("tests/fixtures/depth-synthetic/record.json")).unwrap();
    let record = crate::depth_record::DepthRecord::parse(&raw, "depth_fixture", "0.2.0").unwrap();
    let enrollment = &record.policy.selection.enrollment;
    scope::selection(enrollment, &scope::SYNTHETIC).unwrap();
    assert!(scope::selection(enrollment, &scope::CODEX).is_err());
}

#[test]
fn only_the_synthetic_context_names_an_evidence_host() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let host = f.resolve().unwrap().evidence_host(f.root.path()).unwrap();
    assert_eq!(host.repository(), "fixture/isolated");
    assert_eq!(host.tip(), git(f.root.path(), &["rev-parse", "HEAD"]));
    let f = Fixture::new();
    let error = f
        .resolve()
        .unwrap()
        .evidence_host(f.root.path())
        .err()
        .unwrap();
    assert!(error.contains("only the synthetic subject"), "{error}");
}
