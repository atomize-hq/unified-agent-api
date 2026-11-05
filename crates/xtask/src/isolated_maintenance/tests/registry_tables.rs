//! Roadmap B.8: registry parsing admits a `support_depth` table only for the installed isolated
//! profile's compiled subject. Production parsing keeps refusing every table.
use super::*;
use crate::isolated_maintenance::{registry as routed, RegistryError};

const PRODUCTION_REFUSAL: &str =
    "agent `depth_fixture` carries `support_depth`, which no command enforces yet";

fn raw(f: &Fixture) -> String {
    fs::read_to_string(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap()
}
fn tables(subject: &'static scope::Subject) -> DepthTables {
    DepthTables { subject }
}
fn agent<'a>(doc: &'a mut toml_edit::DocumentMut, id: &str) -> &'a mut toml_edit::Table {
    doc["agents"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
        .find(|a| a["agent_id"].as_str() == Some(id))
        .unwrap()
}

#[test]
fn the_installed_subject_table_is_accepted_and_production_still_refuses_it() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let raw = raw(&f);
    let parsed = AgentRegistry::parse_isolated(&raw, &tables(&scope::SYNTHETIC)).unwrap();
    assert!(parsed
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_some());
    // The accepting parse validates exactly what the unenforced parse validates.
    assert_eq!(parsed, AgentRegistry::parse_unenforced(&raw).unwrap());
    let refusal = AgentRegistry::parse(&raw).unwrap_err().to_string();
    assert!(refusal.contains(PRODUCTION_REFUSAL), "{refusal}");
    // Another compiled subject's admission does not admit this table.
    let other = AgentRegistry::parse_isolated(&raw, &tables(&scope::CODEX)).unwrap_err();
    assert!(other.to_string().contains(PRODUCTION_REFUSAL), "{other}");

    // Without an installed profile the routed load is the production load, with its refusal.
    match routed(f.root.path()) {
        Err(RegistryError::Production(error)) => {
            assert!(error.contains(PRODUCTION_REFUSAL), "{error}")
        }
        other => panic!("expected the production refusal, got {other:?}"),
    }
    // Installed, the same workspace resolves through the accepting parse.
    let _installed = f.install();
    assert_eq!(routed(f.root.path()).unwrap(), parsed);
}

#[test]
fn malformed_or_foreign_tables_refuse_even_for_the_installed_subject() {
    type Edit = fn(&mut toml_edit::DocumentMut);
    let edits: [(&str, Edit); 5] = [
        ("unknown field", |doc| {
            agent(doc, "depth_fixture")["support_depth"]["unexpected"] = toml_edit::value(true);
        }),
        ("missing promise", |doc| {
            agent(doc, "depth_fixture")["support_depth"]["enrollments"][0]["operations"][0]
                .as_table_like_mut()
                .unwrap()
                .remove("promise");
        }),
        ("empty targets", |doc| {
            agent(doc, "depth_fixture")["support_depth"]["enrollments"][0]["targets"] =
                toml_edit::value(toml_edit::Array::new());
        }),
        ("mapped operation on a non-runtime entry", |doc| {
            agent(doc, "depth_fixture")["support_depth"]["enrollments"][0]["operations"][0]
                ["shared_mapping"] = toml_edit::value("mapped");
        }),
        ("table on another agent", |doc| {
            let table = agent(doc, "depth_fixture")["support_depth"].clone();
            agent(doc, "claude_code").insert("support_depth", table);
        }),
    ];
    for (name, edit) in edits {
        let f = Fixture::new_for(&scope::SYNTHETIC);
        f.registry(edit);
        assert!(
            AgentRegistry::parse_isolated(&raw(&f), &tables(&scope::SYNTHETIC)).is_err(),
            "accepting parse admitted: {name}"
        );
        assert!(
            AgentRegistry::parse(&raw(&f)).is_err(),
            "production: {name}"
        );
        // Even protected by the isolated host, the route refuses it and never falls back.
        f.commit();
        let _installed = f.install();
        assert!(
            matches!(routed(f.root.path()), Err(RegistryError::Isolated(_))),
            "route admitted: {name}"
        );
    }
}

#[test]
fn a_failing_route_never_falls_back_to_a_registry_production_would_accept() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    // Remove the subject's table: production parsing accepts the result, the route must not.
    f.registry(|doc| {
        agent(doc, "depth_fixture").remove("support_depth");
    });
    f.commit();
    AgentRegistry::parse(&raw(&f)).unwrap();
    {
        let _installed = f.install();
        match routed(f.root.path()) {
            Err(RegistryError::Isolated(error)) => {
                assert!(
                    error.contains("missing isolated depth declaration"),
                    "{error}"
                )
            }
            other => panic!("a failing route fell back: {other:?}"),
        }
    }
    // Host identity drift fails an otherwise valid route the same way.
    let f = Fixture::new_for(&scope::SYNTHETIC);
    f.api.put(
        "repos/fixture/isolated",
        json!({"id":11,"owner":{"id":20},"full_name":"fixture/isolated"}),
    );
    let _installed = f.install();
    assert!(matches!(
        routed(f.root.path()),
        Err(RegistryError::Isolated(_))
    ));
}

#[test]
fn the_committed_production_registry_is_unchanged_and_carries_no_table() {
    let committed = include_str!("../../../data/agent_registry.toml");
    let production = AgentRegistry::parse(committed).unwrap();
    assert!(production.agents.iter().all(|a| a.support_depth.is_none()));
    assert!(production.find("depth_fixture").is_none());
    // Admitting either compiled subject changes nothing about a registry without tables.
    for subject in [&scope::CODEX, &scope::SYNTHETIC] {
        assert_eq!(
            AgentRegistry::parse_isolated(committed, &tables(subject)).unwrap(),
            production
        );
    }
    // And the shipped production load is the same registry.
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert_eq!(AgentRegistry::load(&workspace).unwrap(), production);
}
