use super::*;

fn request_text(table: &str) -> String {
    format!(
        r#"artifact_version = "2"
agent_id = "codex"
trigger_kind = "upstream_release_detected"
basis_ref = "cli_manifests/codex/latest_validated.txt"
opened_from = ".github/workflows/agent-maintenance-open-pr.yml"
requested_control_plane_actions = ["packet_doc_refresh"]
request_recorded_at = "2026-10-22T00:00:00Z"
request_commit = "abcdef0"
[runtime_followup_required]
required = false
items = []
{table}
"#
    )
}

#[test]
fn depth_binding_table_is_part_of_the_shared_request_shape() {
    let text = request_text(&format!(
        "[support_depth]\npolicy_identity = \"{}\"\n",
        "a".repeat(64)
    ));
    toml_edit::de::from_str::<raw::RawMaintenanceRequest>(&text)
        .expect("the two-field support-depth table must parse");
}

use crate::{
    agent_registry::support_depth::SupportDepth,
    depth_record::{part_identity, DepthRecord, Event, ObligationsPart, PolicyPart},
};
use serde_json::json;

const VERSION: &str = "1.2.3";

fn event() -> Event {
    Event::Maintenance {
        request_commit: "abcdef0".into(),
        request_recorded_at: "2026-10-22T00:00:00Z".into(),
        trigger_kind: "upstream_release_detected".into(),
        opened_from: ".github/workflows/agent-maintenance-open-pr.yml".into(),
    }
}

fn fixture(agent: &str, with_o: bool) -> (tempfile::TempDir, AgentRegistry, DepthRecord) {
    let root = tempfile::tempdir().unwrap();
    let mut registry =
        AgentRegistry::parse_unenforced(include_str!("../../../data/agent_registry.toml")).unwrap();
    let entry = registry
        .agents
        .iter_mut()
        .find(|e| e.agent_id == agent)
        .unwrap();
    let target = entry.canonical_targets[0].clone();
    let depth: SupportDepth = serde_json::from_value(json!({"enabled_paths":["maintenance"],"enrollments":[{
        "lifecycle_path":"maintenance","version":VERSION,"targets":[target],
        "modes":[{"selector":"run","command_path":format!("{agent} run"),"mode":"headless","predicates":[]}],
        "operations":[{"id":"run","serves":["run"],"templates":["CORE"],"shared_mapping":"wrapper_only","forwarding":false,
        "promise":{"r1":"r","v1":"v","g1":"g","e1":"e","a1":"a","x1":"x","x2":"x","x3":"x","x4":"x"}}]}]})).unwrap();
    entry.support_depth = Some(depth.clone());
    let mut policy: PolicyPart = serde_json::from_value(json!({
        "identity":"a".repeat(64),"selection":{"agent_id":agent,"enrollment":depth.enrollments.unwrap()[0]},
        "rules_revision":"1","base":["A1","E1","G1","R1","V1","X1","X2","X3","X4"],
        "templates":[{"template":"CORE","obligations":["LC2","OUT3"]}],"path_enabled":true,"debt_baseline":[]
    })).unwrap();
    policy.identity = part_identity(&policy).unwrap();
    let obligations = with_o.then(|| {
        let mut o: ObligationsPart = serde_json::from_value(json!({
            "identity":"a".repeat(64),"policy_identity":policy.identity,"targets":[target],
            "acquired_inputs":[{"path":"fixtures/input.json","sha256":"a".repeat(64)}],
            "operations":[],"classification_required":[],"acceptance_work":[]
        }))
        .unwrap();
        o.identity = part_identity(&o).unwrap();
        o
    });
    let record = DepthRecord {
        schema_version: 1,
        event: event(),
        policy,
        obligations,
        execution: None,
        acceptance_entries: vec![],
    };
    save_record(root.path(), agent, &record);
    (root, registry, record)
}

fn save_record(root: &Path, agent: &str, record: &DepthRecord) {
    let raw = serde_json::to_string(record).unwrap();
    DepthRecord::parse(&raw, agent, VERSION).expect("fixture is an actual valid Annex B record");
    let path = root.join(format!(
        "cli_manifests/{agent}/reports/{VERSION}/depth-record.json"
    ));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, raw).unwrap();
}

fn table(record: &DepthRecord) -> SupportDepthBinding {
    SupportDepthBinding {
        policy_identity: record.policy.identity.clone(),
        obligations_identity: record.obligations.as_ref().map(|o| o.identity.clone()),
    }
}

fn validate(
    root: &Path,
    registry: &AgentRegistry,
    agent: &str,
    event: Event,
    table: Option<&SupportDepthBinding>,
) -> Result<(), MaintenanceRequestError> {
    depth::validate_binding(
        &WorkspacePathJail::new(root).unwrap(),
        Path::new("request.toml"),
        registry.find(agent).unwrap(),
        "2",
        Some(VERSION),
        event,
        table,
    )
}

#[test]
fn depth_bindings_round_trip_p_only_and_p_o_for_all_adopted_roots() {
    for agent in ["codex", "claude_code", "opencode"] {
        for with_o in [false, true] {
            let (root, registry, record) = fixture(agent, with_o);
            let binding = table(&record);
            let rendered = binding.render().unwrap();
            assert_eq!(rendered.contains("obligations_identity"), with_o);
            let raw: raw::RawMaintenanceRequest =
                toml_edit::de::from_str(&request_text(&rendered)).unwrap();
            assert_eq!(raw.support_depth, Some(binding.clone()));
            validate(
                root.path(),
                &registry,
                agent,
                event(),
                raw.support_depth.as_ref(),
            )
            .unwrap();
        }
    }
}

#[test]
fn malformed_depth_tables_and_identities_are_rejected() {
    for fields in [
        "",
        "policy_identity = 12",
        "policy_identity = []",
        "policy_identity = 'a'\nextra = 'x'",
        "policy_identity = 'a'\npolicy_identity = 'b'",
        "policy_identity = 'a'\nobligations_identity = false",
    ] {
        assert!(
            toml_edit::de::from_str::<raw::RawMaintenanceRequest>(&request_text(&format!(
                "[support_depth]\n{fields}\n"
            )))
            .is_err(),
            "{fields}"
        );
    }
    let (root, registry, record) = fixture("codex", true);
    for value in [
        "".to_string(),
        "a".repeat(63),
        "A".repeat(64),
        "g".repeat(64),
    ] {
        for obligations in [false, true] {
            let mut binding = table(&record);
            if obligations {
                binding.obligations_identity = Some(value.clone());
            } else {
                binding.policy_identity = value.clone();
            }
            assert!(binding.render().unwrap_err().contains("64 lowercase hex"));
            assert!(
                validate(root.path(), &registry, "codex", event(), Some(&binding))
                    .unwrap_err()
                    .to_string()
                    .contains("64 lowercase hex")
            );
        }
    }
}

#[test]
fn enrollment_presence_and_retained_record_fail_closed() {
    let (root, mut registry, record) = fixture("codex", false);
    let binding = table(&record);
    assert!(validate(root.path(), &registry, "codex", event(), None)
        .unwrap_err()
        .to_string()
        .contains("table is required"));
    registry
        .agents
        .iter_mut()
        .find(|e| e.agent_id == "codex")
        .unwrap()
        .support_depth = None;
    assert!(validate(root.path(), &registry, "codex", event(), None)
        .unwrap_err()
        .to_string()
        .contains("retained depth record"));
    let path = root.path().join(format!(
        "cli_manifests/codex/reports/{VERSION}/depth-record.json"
    ));
    fs::remove_file(path).unwrap();
    assert!(
        validate(root.path(), &registry, "codex", event(), Some(&binding))
            .unwrap_err()
            .to_string()
            .contains("must be absent")
    );
    validate(root.path(), &registry, "codex", event(), None).unwrap();
}

#[test]
fn every_event_member_and_both_frozen_identities_must_agree() {
    for with_o in [false, true] {
        let (root, registry, record) = fixture("codex", with_o);
        let binding = table(&record);
        for index in 0..4 {
            let mut changed = event();
            let Event::Maintenance {
                request_commit,
                request_recorded_at,
                trigger_kind,
                opened_from,
            } = &mut changed
            else {
                unreachable!()
            };
            [
                request_commit,
                request_recorded_at,
                trigger_kind,
                opened_from,
            ][index]
                .push('x');
            assert!(
                validate(root.path(), &registry, "codex", changed, Some(&binding))
                    .unwrap_err()
                    .to_string()
                    .contains("Event")
            );
        }
        let mut changed = binding.clone();
        changed.policy_identity = "b".repeat(64);
        assert!(
            validate(root.path(), &registry, "codex", event(), Some(&changed))
                .unwrap_err()
                .to_string()
                .contains("record's P")
        );
        let mut changed = binding.clone();
        changed.obligations_identity = if with_o { None } else { Some("b".repeat(64)) };
        assert!(
            validate(root.path(), &registry, "codex", event(), Some(&changed))
                .unwrap_err()
                .to_string()
                .contains("record's O")
        );
        if with_o {
            changed.obligations_identity = Some("b".repeat(64));
            assert!(
                validate(root.path(), &registry, "codex", event(), Some(&changed))
                    .unwrap_err()
                    .to_string()
                    .contains("record's O")
            );
        }
    }
}

#[test]
fn missing_corrupt_and_symlinked_records_never_count_as_absence() {
    let (root, registry, record) = fixture("codex", false);
    let binding = table(&record);
    let path = root.path().join(format!(
        "cli_manifests/codex/reports/{VERSION}/depth-record.json"
    ));
    fs::write(&path, "{}").unwrap();
    assert!(
        validate(root.path(), &registry, "codex", event(), Some(&binding))
            .unwrap_err()
            .to_string()
            .contains("schema_version")
    );
    fs::remove_file(&path).unwrap();
    assert!(
        validate(root.path(), &registry, "codex", event(), Some(&binding))
            .unwrap_err()
            .to_string()
            .contains("record is missing")
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.path().join("absent"), &path).unwrap();
        assert!(
            validate(root.path(), &registry, "codex", event(), Some(&binding))
                .unwrap_err()
                .to_string()
                .contains("symlink")
        );
    }
}

fn seed_loader_request(
    root: &Path,
    registry: &AgentRegistry,
    table: Option<&SupportDepthBinding>,
) -> PathBuf {
    let mut registry_doc: toml_edit::DocumentMut =
        include_str!("../../../data/agent_registry.toml")
            .parse()
            .unwrap();
    let codex = registry.find("codex").unwrap();
    let entry = registry_doc["agents"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["agent_id"].as_str() == Some("codex"))
        .unwrap();
    entry.insert(
        "support_depth",
        toml_edit::Item::Table(
            toml_edit::ser::to_document(codex.support_depth.as_ref().unwrap())
                .unwrap()
                .as_table()
                .clone(),
        ),
    );
    let registry_path = root.join(REGISTRY_RELATIVE_PATH);
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    fs::write(registry_path, registry_doc.to_string()).unwrap();
    for path in [
        "cli_manifests/codex/latest_validated.txt",
        ".github/workflows/agent-maintenance-open-pr.yml",
    ] {
        let full = root.join(path);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(full, "fixture\n").unwrap();
    }
    let derived = super::super::contract_policy::derive_detected_release_fields(
        "codex",
        codex.maintenance.release_watch.as_ref().unwrap(),
    )
    .unwrap();
    let text = request_text(&format!(
        r#"{}
[detected_release]
detected_by = "fixture"
current_validated = "1.2.2"
target_version = "1.2.3"
latest_stable = "1.2.4"
version_policy = "{}"
source_kind = "{}"
source_ref = "{}"
dispatch_kind = "{}"
dispatch_workflow = "{}"
branch_name = "automation/codex-maintenance-1.2.3"
"#,
        table.map(|t| t.render().unwrap()).unwrap_or_default(),
        derived.version_policy,
        derived.source_kind,
        derived.source_ref,
        derived.dispatch_kind,
        derived.dispatch_workflow
    ));
    let path = PathBuf::from(
        "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml",
    );
    fs::create_dir_all(root.join(&path).parent().unwrap()).unwrap();
    fs::write(root.join(&path), text).unwrap();
    path
}

#[test]
fn both_loader_policies_check_depth_bindings_before_audit_drift_and_keep_production_refusal() {
    let (root, registry, record) = fixture("codex", true);
    for policy in [AuditDriftPolicy::Reject, AuditDriftPolicy::Tolerate] {
        for case in ["missing", "p", "o", "event", "matching"] {
            let mut binding = table(&record);
            if case == "p" {
                binding.policy_identity = "b".repeat(64);
            }
            if case == "o" {
                binding.obligations_identity = None;
            }
            let path = seed_loader_request(
                root.path(),
                &registry,
                (case != "missing").then_some(&binding),
            );
            if case == "event" {
                let file = root.path().join(&path);
                fs::write(
                    &file,
                    fs::read_to_string(&file)
                        .unwrap()
                        .replace("abcdef0", "abcdef1"),
                )
                .unwrap();
            }
            let error = load_request_with_registry(root.path(), &path, policy, |root| {
                AgentRegistry::parse_unenforced(
                    &fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH)).unwrap(),
                )
            })
            .unwrap_err()
            .to_string();
            let expected = match case {
                "missing" => "table is required",
                "p" => "record's P",
                "o" => "record's O",
                "event" => "Event",
                // The binding succeeds; the ordinary missing-audit rule still prevents readiness.
                "matching" => "support_surface_audit",
                _ => unreachable!(),
            };
            assert!(error.contains(expected), "case={case}: {error}");
            let request_before = fs::read(root.path().join(&path)).unwrap();
            let record_path = root
                .path()
                .join("cli_manifests/codex/reports/1.2.3/depth-record.json");
            let record_before = fs::read(&record_path).unwrap();
            let public_error =
                load_request_envelope_validated_with_policy(root.path(), &path, policy)
                    .unwrap_err()
                    .to_string();
            assert!(
                public_error.contains("which no command enforces yet"),
                "{public_error}"
            );
            assert_eq!(fs::read(root.path().join(&path)).unwrap(), request_before);
            assert_eq!(fs::read(&record_path).unwrap(), record_before);
        }
    }
}

#[test]
fn private_loader_core_preserves_the_typed_binding_when_the_ordinary_audit_is_valid() {
    let (root, registry, record) = fixture("codex", true);
    let binding = table(&record);
    let path = seed_loader_request(root.path(), &registry, Some(&binding));
    let entry = registry.find("codex").unwrap();
    let rules = root.path().join("cli_manifests/codex/RULES.json");
    fs::write(
        rules,
        json!({"union":{"expected_targets":entry.canonical_targets}}).to_string(),
    )
    .unwrap();
    let reports = root.path().join("cli_manifests/codex/reports/1.2.3");
    let mut report = json!({"inputs":{"upstream":{"semantic_version":VERSION,"targets":entry.canonical_targets}},
        "platform_filter":{"mode":"any"},"deltas":{"missing_commands":[],"missing_flags":[],"missing_args":[],"intentionally_unsupported":[]}});
    fs::write(reports.join("coverage.any.json"), report.to_string()).unwrap();
    for target in &entry.canonical_targets {
        report["platform_filter"] = json!({"mode":"exact_target","target_triple":target});
        fs::write(
            reports.join(format!("coverage.{target}.json")),
            report.to_string(),
        )
        .unwrap();
    }
    let debt = root
        .path()
        .join("docs/specs/unified-agent-api/non-tui-support-debt.md");
    fs::create_dir_all(debt.parent().unwrap()).unwrap();
    fs::write(
        debt,
        "### `support-debt-authorization-contract-target-version-v1`\n",
    )
    .unwrap();
    let request_path = root.path().join(&path);
    let mut text = fs::read_to_string(&request_path).unwrap();
    use super::super::support_audit::{allowed_deferrals, excluded_surface_kinds, surface_kinds};
    text.push_str(&format!("\n[support_surface_audit]\nrequired = true\nsurface_kinds = {}\nexcluded_surface_kinds = {}\nallowed_deferrals = {}\npre_run_debt_count = 0\nexpected_post_run_debt_count = 0\n",
        serde_json::to_string(&surface_kinds()).unwrap(), serde_json::to_string(&excluded_surface_kinds()).unwrap(), serde_json::to_string(&allowed_deferrals()).unwrap()));
    fs::write(&request_path, &text).unwrap();
    for policy in [AuditDriftPolicy::Reject, AuditDriftPolicy::Tolerate] {
        let loaded = load_request_with_registry(root.path(), &path, policy, |root| {
            AgentRegistry::parse_unenforced(
                &fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH)).unwrap(),
            )
        })
        .expect("ordinary audit and depth bindings both validate in the isolated core");
        assert_eq!(
            loaded.envelope.request.support_depth.as_ref(),
            Some(&binding)
        );
        assert_eq!(loaded.envelope.request.request_commit, "abcdef0");
        assert_eq!(fs::read_to_string(&request_path).unwrap(), text);
    }
}
