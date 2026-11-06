//! Slice 0 of the onboarding freeze decisions: the isolated scope admits the onboarding selection
//! for the synthetic subject alone. It is a proof workspace, not onboarding path enablement.
use super::*;
use crate::agent_registry::support_depth::LifecyclePath;

const DEFERRED_ENTRY: &str =
    include_str!("../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml");
const APPROVAL: &str = include_str!(
    "../../../tests/fixtures/depth-synthetic/onboarding/governance/approved-agent.toml"
);
const STATEMENT: &str = include_str!(
    "../../../tests/fixtures/depth-synthetic/onboarding/governance/depth-acquisition/0.2.0.toml"
);
const APPROVAL_PATH: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/approved-agent.toml";
const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";

fn entry_index(doc: &toml_edit::DocumentMut, agent: &str) -> usize {
    doc["agents"]
        .as_array_of_tables()
        .unwrap()
        .iter()
        .position(|a| a["agent_id"].as_str() == Some(agent))
        .unwrap()
}

/// Install the canonical registry plus the deferred-maintenance onboarding entry, in place of the
/// maintenance entry, and commit it to the protected tip.
fn install_onboarding_entry(f: &Fixture) {
    let canonical = include_str!("../../../data/agent_registry.toml");
    let registry = format!("{canonical}\n{DEFERRED_ENTRY}");
    write(f.root.path(), REGISTRY_RELATIVE_PATH, registry);
    f.commit();
}

fn onboarding() -> Fixture {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    install_onboarding_entry(&f);
    f
}

fn edit_depth(f: &Fixture, agent: &str, edit: impl FnOnce(&mut toml_edit::Item)) {
    f.registry(|doc| {
        let index = entry_index(doc, agent);
        let agents = doc["agents"].as_array_of_tables_mut().unwrap();
        edit(&mut agents.get_mut(index).unwrap()["support_depth"]);
    });
}

fn paths(paths: &[&str]) -> toml_edit::Item {
    toml_edit::value(paths.iter().copied().collect::<toml_edit::Array>())
}

/// A retained record on the workspace's declared enrollment, under `event`.
fn record(f: &Fixture, event: serde_json::Value, path_enabled: bool) -> String {
    use crate::{
        agent_registry::support_depth::{ObligationId, ObligationTemplate},
        depth_record::{part_identity, PolicyPart, Selection, TemplateRule},
    };
    let raw = fs::read_to_string(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
    let registry = AgentRegistry::parse_unenforced(&raw).unwrap();
    let depth = registry
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .clone();
    let mut policy = PolicyPart {
        identity: "0".repeat(64),
        selection: Selection {
            agent_id: "depth_fixture".into(),
            enrollment: depth.unwrap().enrollments.unwrap()[0].clone(),
        },
        rules_revision: "1".into(),
        base: ObligationId::BASE.to_vec(),
        templates: vec![TemplateRule {
            template: ObligationTemplate::Core,
            obligations: vec![ObligationId::Lc2, ObligationId::Out3],
        }],
        path_enabled,
        debt_baseline: vec![],
    };
    policy
        .base
        .sort_by_key(|id| serde_json::to_string(id).unwrap());
    policy.identity = part_identity(&policy).unwrap();
    let raw = serde_json::json!({"schema_version":1,"event":event,"policy":policy,"obligations":null,"execution":null,"acceptance_entries":[]}).to_string();
    crate::depth_record::DepthRecord::parse(&raw, "depth_fixture", "0.2.0").unwrap();
    raw
}

fn onboarding_event() -> serde_json::Value {
    serde_json::json!({"lifecycle_path":"onboarding","approval_commit":"abc1234","approval_recorded_at":"2026-11-05T00:00:00Z"})
}

#[test]
fn onboarding_selection_resolves_for_the_synthetic_subject() {
    let f = onboarding();
    f.resolve().unwrap().recheck(f.root.path()).unwrap();
    // The onboarding Event's record on that selection is a retained generation of this scope.
    write(f.root.path(), RECORD, record(&f, onboarding_event(), true));
    f.commit();
    f.resolve().unwrap();
}

#[test]
fn onboarding_selection_is_admitted_only_under_the_installed_fixture() {
    let f = onboarding();
    let root = f.root.path();
    // Production: no route, the production load refuses depth, and the diagnostic refuses.
    assert!(route(root).unwrap().is_none());
    assert!(registry(root).is_err());
    assert!(check(root).unwrap_err().contains("not installed"));
    let installed = f.install();
    let route = route(root).unwrap().expect("installed route");
    let agent = route.registry(root).unwrap().find("depth_fixture").unwrap();
    let depth = agent.support_depth.as_ref().unwrap();
    assert_eq!(depth.enabled_paths, Some(vec![LifecyclePath::Onboarding]));
    let enrollments = depth.enrollments.as_ref().unwrap();
    assert_eq!(enrollments.len(), 1);
    assert_eq!(enrollments[0].lifecycle_path, LifecyclePath::Onboarding);
    assert!(
        agent.maintenance.release_watch.is_none(),
        "deferred variant has no release-watch"
    );
    drop(installed);
    assert!(super::route(root).unwrap().is_none());
}

#[test]
fn maintenance_selection_is_unchanged() {
    // The maintenance template still resolves, with release-watch, and its record still admits.
    let f = Fixture::new_for(&scope::SYNTHETIC);
    write(f.root.path(), RECORD, f.retained());
    f.commit();
    f.resolve().unwrap();
    // An onboarding-Event record does not belong to a maintenance workspace.
    let mut onboarding_record: serde_json::Value = serde_json::from_str(&f.retained()).unwrap();
    onboarding_record["event"] = onboarding_event();
    assert!(crate::depth_record::DepthRecord::parse(
        &onboarding_record.to_string(),
        "depth_fixture",
        "0.2.0"
    )
    .is_err());
}

#[test]
fn codex_subject_refuses_the_onboarding_path() {
    let f = Fixture::new();
    edit_depth(&f, "codex", |depth| {
        depth["enabled_paths"] = paths(&["onboarding"]);
        depth["enrollments"][0]["lifecycle_path"] = toml_edit::value("onboarding");
    });
    f.commit();
    let error = f.resolve().err().unwrap();
    assert!(error.contains("unsupported isolated selection"), "{error}");
    // The compiled selection check alone refuses it for Codex and admits it for the synthetic one.
    let raw = fs::read_to_string(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
    let registry = AgentRegistry::parse_unenforced(&raw).unwrap();
    let codex = registry.find("codex").unwrap().support_depth.clone();
    let enrollment = &codex.unwrap().enrollments.unwrap()[0];
    assert!(scope::selection(enrollment, &scope::CODEX).is_err());
    let f = onboarding();
    let raw = fs::read_to_string(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
    let registry = AgentRegistry::parse_unenforced(&raw).unwrap();
    let depth = registry
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .clone();
    let enrollment = &depth.unwrap().enrollments.unwrap()[0];
    scope::selection(enrollment, &scope::SYNTHETIC).unwrap();
    assert!(scope::selection(enrollment, &scope::CODEX).is_err());
}

#[test]
fn onboarding_scope_refuses_other_versions_targets_paths_and_agents() {
    let selection = "unsupported isolated selection";
    let one_path = "isolated scope supports exactly one lifecycle path";
    let disagree = "isolated enrollment is not on the enabled lifecycle path";
    let expected = [
        selection, selection, selection, one_path, disagree, disagree, one_path,
    ];
    for (edit, reason) in expected.into_iter().enumerate() {
        let f = onboarding();
        edit_depth(&f, "depth_fixture", |depth| {
            let d = &mut depth["enrollments"][0];
            match edit {
                0 => d["version"] = toml_edit::value("0.1.0"),
                1 => d["version"] = toml_edit::value("0.3.0"),
                2 => d["targets"][0] = toml_edit::value("x86_64-unknown-linux-musl"),
                // Both paths at once: one workspace proves one path.
                3 => depth["enabled_paths"] = paths(&["maintenance", "onboarding"]),
                // The enabled path and the enrollment's path disagree, either way round.
                4 => depth["enabled_paths"] = paths(&["maintenance"]),
                5 => d["lifecycle_path"] = toml_edit::value("maintenance"),
                // An onboarding enrollment on a path that is not enabled.
                _ => {
                    depth.as_table_mut().unwrap().remove("enabled_paths");
                }
            }
        });
        f.commit();
        let error = f.resolve().err().unwrap();
        assert!(error.contains(reason), "onboarding edit {edit}: {error}");
    }
    // A second agent's onboarding table is an extra registered scope.
    let f = onboarding();
    let codex: crate::agent_registry::support_depth::SupportDepth =
        serde_json::from_value(fixture::declaration()).unwrap();
    let codex = toml_edit::ser::to_document(&codex).unwrap();
    f.registry(|doc| {
        let index = entry_index(doc, "codex");
        let agents = doc["agents"].as_array_of_tables_mut().unwrap();
        let entry = agents.get_mut(index).unwrap();
        entry.insert(
            "support_depth",
            toml_edit::Item::Table(codex.as_table().clone()),
        );
        entry["support_depth"]["enabled_paths"] = paths(&["onboarding"]);
        entry["support_depth"]["enrollments"][0]["lifecycle_path"] = toml_edit::value("onboarding");
    });
    f.commit();
    assert!(
        f.resolve().is_err(),
        "extra Codex onboarding depth admitted"
    );
}

#[test]
fn retained_onboarding_records_follow_the_declared_path() {
    // A path-disabled P, and a retained version other than the subject's, refuse.
    let f = onboarding();
    write(f.root.path(), RECORD, record(&f, onboarding_event(), false));
    f.commit();
    let error = f.resolve().err().unwrap();
    assert!(
        error.contains("retained isolated selection differs"),
        "{error}"
    );
    let f = onboarding();
    let other = "cli_manifests/depth_fixture/reports/0.1.0/depth-record.json";
    write(f.root.path(), other, record(&f, onboarding_event(), true));
    f.commit();
    let error = f.resolve().err().unwrap();
    assert!(
        error.contains("unsupported retained isolated scope"),
        "{error}"
    );
    // A maintenance generation's record does not survive a move to the onboarding path.
    let f = Fixture::new_for(&scope::SYNTHETIC);
    write(f.root.path(), RECORD, f.retained());
    f.commit();
    install_onboarding_entry(&f);
    let error = f.resolve().err().unwrap();
    assert!(
        error.contains("retained isolated selection differs"),
        "{error}"
    );
}

#[test]
fn onboarding_templates_state_the_synthetic_subject() {
    // The deferred entry declares the reviewed selection, on the onboarding path, and nothing else.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let raw = fs::read_to_string(root.join("tests/fixtures/depth-synthetic/record.json")).unwrap();
    let reviewed = crate::depth_record::DepthRecord::parse(&raw, "depth_fixture", "0.2.0").unwrap();
    let mut expected = reviewed.policy.selection.enrollment;
    expected.lifecycle_path = LifecyclePath::Onboarding;
    let deferred = AgentRegistry::parse_unenforced(DEFERRED_ENTRY).unwrap();
    let entry = deferred.find("depth_fixture").unwrap();
    assert!(entry.maintenance.release_watch.is_none());
    let depth = entry.support_depth.as_ref().unwrap();
    assert_eq!(depth.enrollments.as_deref(), Some(&[expected][..]));

    // The approval approves exactly 0.2.0 and defers maintenance.
    let approval = APPROVAL
        .replace("{{APPROVAL_COMMIT}}", "abc1234")
        .replace("{{APPROVAL_RECORDED_AT}}", "2026-11-05T00:00:00Z");
    let attribution =
        crate::approval_artifact::parse_approval_attribution(&approval, APPROVAL_PATH).unwrap();
    assert_eq!(attribution.agent_id, "depth_fixture");
    assert_eq!(
        attribution.onboarding_pack_prefix,
        "depth-fixture-cli-onboarding"
    );
    assert_eq!(attribution.support_depth_version.as_deref(), Some("0.2.0"));
    let doc: toml_edit::DocumentMut = approval.parse().unwrap();
    let maintenance = &doc["descriptor"]["maintenance"];
    assert_eq!(maintenance["mode"].as_str(), Some("explicitly_deferred"));
    assert!(maintenance.get("release_watch").is_none());
    assert!(doc["descriptor"].get("backend_module").is_none());
    // Open question: the full loader still requires `backend_module`, so it refuses a non-runtime
    // approval. Keep that refusal until a later slice decides.
    let workspace = tempfile::tempdir().unwrap();
    write(workspace.path(), APPROVAL_PATH, &approval);
    let error = crate::approval_artifact::load_approval_artifact(workspace.path(), APPROVAL_PATH)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("backend_module"), "{error}");

    // The acquisition statement names exactly the inputs O binds, each awaiting its digest.
    let statement: toml_edit::DocumentMut = STATEMENT.parse().unwrap();
    let keys: Vec<_> = statement.iter().map(|(k, _)| k.to_owned()).collect();
    assert_eq!(keys, ["agent", "version", "inputs"]);
    assert_eq!(statement["agent"].as_str(), Some("depth_fixture"));
    assert_eq!(statement["version"].as_str(), Some("0.2.0"));
    let inputs = statement["inputs"].as_array_of_tables().unwrap();
    let mut listed = Vec::new();
    for input in inputs {
        let keys: Vec<_> = input.iter().map(|(k, _)| k).collect();
        assert_eq!(keys, ["path", "sha256", "command"]);
        assert_eq!(input["sha256"].as_str(), Some("{{SHA256}}"));
        assert!(!input["command"].as_str().unwrap().is_empty());
        listed.push(input["path"].as_str().unwrap().to_owned());
    }
    let m = "cli_manifests/depth_fixture";
    let t = "x86_64-unknown-linux-gnu";
    let mut expected = vec![
        format!("{m}/RULES.json"),
        format!("{m}/SCHEMA.json"),
        format!("{m}/snapshots/0.2.0/union.json"),
        format!("{m}/snapshots/0.2.0/{t}.json"),
        format!("{m}/wrapper_coverage.json"),
        format!("{m}/reports/0.2.0/coverage.any.json"),
        format!("{m}/reports/0.2.0/coverage.all.json"),
        format!("{m}/reports/0.2.0/coverage.{t}.json"),
    ];
    expected.sort();
    listed.sort();
    assert_eq!(listed, expected);
}
