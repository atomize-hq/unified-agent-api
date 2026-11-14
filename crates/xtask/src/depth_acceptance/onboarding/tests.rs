//! Private fixtures prove owning predicates and aggregate detectors. Connected proof below
//! consumes actual ON3 bytes and the independently authenticated slice-6 population.
use super::*;
mod readonly_gates;
use crate::{
    agent_lifecycle as life,
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::{due, part_identity},
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
const AGENT: &str = "depth_fixture";
const VERSION: &str = "0.2.0";
const ROOT: &str = "cli_manifests/depth_fixture";
const APPROVAL: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/approved-agent.toml";
const STATE: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/lifecycle-state.json";
const PACKET: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/publication-ready.json";
const CLOSEOUT: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/proving-run-closeout.json";
const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
const EVIDENCE: &str = "docs/s7-private-evidence.json";
const STAMP: &str = "2026-11-05T00:00:00Z";
fn encode(value: &impl serde::Serialize) -> String {
    serde_json::to_string_pretty(value).unwrap() + "\n"
}
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn source(files: &BTreeMap<String, String>) -> Source {
    Source::proposed(&repo(), files.clone()).unwrap()
}
fn reseal(record: &mut DepthRecord) {
    record.policy.identity = part_identity(&record.policy).unwrap();
    let o = record.obligations.as_mut().unwrap();
    o.policy_identity = record.policy.identity.clone();
    o.identity = part_identity(o).unwrap();
    if let Some(e) = &mut record.execution {
        e.results
            .sort_by_key(|result| serde_json::to_string(result).unwrap());
        e.policy_identity = record.policy.identity.clone();
        e.obligations_identity = o.identity.clone();
        e.identity = part_identity(e).unwrap();
    }
}
struct Fixture {
    before: BTreeMap<String, String>,
    after: BTreeMap<String, String>,
    record: DepthRecord,
    kind: Kind,
}
impl Fixture {
    fn new(kind: Kind) -> Self {
        let registry = format!(
            "{}\n{}",
            include_str!("../../../data/agent_registry.toml"),
            include_str!(
                "../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml"
            )
        );
        let approval = include_str!(
            "../../../tests/fixtures/depth-synthetic/onboarding/governance/approved-agent.toml"
        )
        .replace("{{APPROVAL_COMMIT}}", "abc1234")
        .replace("{{APPROVAL_RECORDED_AT}}", STAMP);
        let mut record: DepthRecord = serde_json::from_str(include_str!(
            "../../../tests/fixtures/depth-synthetic/record.json"
        ))
        .unwrap();
        record.event = Event::Onboarding {
            approval_commit: "abc1234".into(),
            approval_recorded_at: STAMP.into(),
        };
        record.policy.selection.enrollment.lifecycle_path = LifecyclePath::Onboarding;
        record.obligations.as_mut().unwrap().acquired_inputs.clear();
        reseal(&mut record);
        let results: Vec<_> = due(&record).unwrap().into_iter().map(|reference| json!({"obligation": reference, "result":"verified", "evidence":[{"path":EVIDENCE,"sha256":artifacts::digest("private evidence")}]})).collect();
        record.execution = Some(serde_json::from_value(json!({"identity":"", "policy_identity":"", "obligations_identity":"", "results": results, "reuse_bindings":[], "debt_grants":[], "debt_retired":[]})).unwrap());
        reseal(&mut record);
        let summary = json!({"requested_runtime_profile":"minimal", "achieved_runtime_profile":"minimal", "primary_template":"codex", "template_lineage":["codex"], "landed_surfaces":[], "deferred_surfaces":[], "minimal_profile_justification":"Private predicate fixture"});
        let mut state: LifecycleState = serde_json::from_value(json!({
            "schema_version":"1", "agent_id":AGENT, "onboarding_pack_prefix":"depth-fixture-cli-onboarding",
            "approval_artifact_path":APPROVAL, "approval_artifact_sha256":artifacts::digest(&approval),
            "lifecycle_stage":"publication_ready", "support_tier":"baseline_runtime", "side_states":[],
            "current_owner_command":"prepare-publication --write", "expected_next_command":"refresh-publication --write",
            "last_transition_at":STAMP, "last_transition_by":"private predicate fixture",
            "required_evidence":life::required_evidence_for_stage(LifecycleStage::PublicationReady),
            "satisfied_evidence":life::required_evidence_for_stage(LifecycleStage::PublicationReady),
            "blocking_issues":[], "retryable_failures":[], "active_runtime_evidence_run_id":null,
            "implementation_summary":summary, "publication_packet_path":null,"publication_packet_sha256":null, "closeout_baseline_path":null,
            "support_depth":{"policy_identity":record.policy.identity, "obligations_identity":record.obligations.as_ref().unwrap().identity},
            "written_paths_sha256":"a".repeat(64),
        })).unwrap();
        state.validate().unwrap();
        let packet = json!({"schema_version":"1", "agent_id":AGENT, "approval_artifact_path":APPROVAL, "approval_artifact_sha256":state.approval_artifact_sha256,
            "lifecycle_state_path":STATE, "lifecycle_state_sha256":artifacts::digest(&encode(&state)), "lifecycle_stage":"publication_ready", "support_tier_at_emit":"baseline_runtime",
            "manifest_root":ROOT, "expected_targets":["x86_64-unknown-linux-gnu"], "capability_publication_enabled":false, "support_publication_enabled":true, "capability_matrix_target":null,
            "required_commands":life::REQUIRED_PUBLICATION_COMMANDS, "required_publication_outputs":[], "runtime_evidence_paths":[], "publication_owned_paths":[], "blocking_issues":[], "implementation_summary":state.implementation_summary});
        let mut before = BTreeMap::from([
            (REGISTRY_RELATIVE_PATH.into(), registry),
            (APPROVAL.into(), approval),
            (STATE.into(), encode(&state)),
            (PACKET.into(), encode(&packet)),
            (RECORD.into(), encode(&record)),
            (EVIDENCE.into(), "private evidence".into()),
            (
                crate::agent_maintenance::support_audit::NON_TUI_SUPPORT_DEBT_PATH.into(),
                "### `support-debt-authorization-contract-target-version-v1`\n".into(),
            ),
        ]);
        advance(&mut state, LifecycleStage::Published);
        state.publication_packet_path = Some(PACKET.into());
        state.publication_packet_sha256 = Some(artifacts::digest(&encode(&packet)));
        if kind == Kind::Proving {
            let publication = AcceptanceEntry::Publication {
                packet: FileRef {
                    path: PACKET.into(),
                    sha256: artifacts::digest(&encode(&packet)),
                },
                record_sha256: record.stated_sha256().unwrap(),
            };
            record.acceptance_entries.push(publication);
            before.insert(STATE.into(), encode(&state));
            before.insert(RECORD.into(), encode(&record));
            advance(&mut state, LifecycleStage::ClosedBaseline);
            state.closeout_baseline_path = Some(CLOSEOUT.into());
        }
        before.insert(
            format!("{ROOT}/RULES.json"),
            include_str!(
                "../../../tests/fixtures/depth-synthetic/governing/manifest-root/RULES.json"
            )
            .into(),
        );
        before.insert(
            format!("{ROOT}/current.json"),
            crate::onboard_agent::render_current_json(&["x86_64-unknown-linux-gnu".into()]),
        );
        for pointer in [
            "latest_validated.txt",
            "min_supported.txt",
            "pointers/latest_supported/x86_64-unknown-linux-gnu.txt",
            "pointers/latest_validated/x86_64-unknown-linux-gnu.txt",
        ] {
            before.insert(format!("{ROOT}/{pointer}"), "none\n".into());
        }
        let mut after = before.clone();
        after.insert(STATE.into(), encode(&state));
        if kind == Kind::Proving {
            after.insert(CLOSEOUT.into(), encode(&json!({"state":"closed","approval_ref":APPROVAL,"approval_sha256":state.approval_artifact_sha256,"approval_source":"private fixture","manual_control_plane_edits":0,"partial_write_incidents":0,"ambiguous_ownership_incidents":0,"duration_seconds":1,"explicit_none_reason":"Private predicate proof","preflight_passed":true,"recorded_at":STAMP,"commit":"abc1234"})));
        }
        if kind == Kind::Proving {
            let maintenance =
                crate::approval_artifact::parse_approval_maintenance(&before[APPROVAL], APPROVAL)
                    .unwrap();
            let crate::approval_artifact::ApprovalMaintenanceMode::ExplicitlyDeferred {
                deferral_sha256,
                ..
            } = maintenance.mode
            else {
                panic!("expected deferral")
            };
            let mut closeout: Value = serde_json::from_str(&after[CLOSEOUT]).unwrap();
            closeout["maintenance_settlement"] = json!({"mode":"explicitly_deferred","approval_section_sha256":maintenance.section_sha256,"release_watch_sha256":null,"deferral_sha256":deferral_sha256});
            after.insert(CLOSEOUT.into(), encode(&closeout));
        }
        Self {
            before,
            after,
            record,
            kind,
        }
    }
    fn view(&self) -> View {
        let src = source(&self.after);
        let registry =
            AgentRegistry::parse_unenforced(&src.read(REGISTRY_RELATIVE_PATH).unwrap()).unwrap();
        View::with_registry(src, &registry, AGENT).unwrap()
    }
    fn accept(&self) -> Result<AcceptanceEntry, String> {
        acceptance(
            &self.view(),
            &source(&self.before),
            &self.record,
            self.kind,
            || {
                Ok((
                    self.record.policy.clone(),
                    self.record.obligations.clone().unwrap(),
                ))
            },
            || Ok(crate::depth_evidence::test_results(Some(&self.record))),
        )
    }
    fn save(&mut self) {
        self.after.insert(RECORD.into(), encode(&self.record));
    }
    fn replace_frozen(&mut self) {
        reseal(&mut self.record);
        let digest = self.record.stated_sha256().unwrap();
        for entry in &mut self.record.acceptance_entries {
            if let AcceptanceEntry::Publication { record_sha256, .. } = entry {
                *record_sha256 = digest.clone();
            }
        }
        self.before.insert(RECORD.into(), encode(&self.record));
        for files in [&mut self.before, &mut self.after] {
            let mut state: LifecycleState = serde_json::from_str(&files[STATE]).unwrap();
            state.support_depth = Some(
                life::Preserved::new(crate::depth_record::onboarding::LifecycleDepth {
                    policy_identity: self.record.policy.identity.clone(),
                    obligations_identity: Some(
                        self.record.obligations.as_ref().unwrap().identity.clone(),
                    ),
                })
                .unwrap(),
            );
            files.insert(STATE.into(), encode(&state));
        }
        let mut packet: Value = serde_json::from_str(&self.before[PACKET]).unwrap();
        if self.kind == Kind::Publication {
            packet["lifecycle_state_sha256"] = json!(artifacts::digest(&self.before[STATE]));
        }
        let raw = encode(&packet);
        self.before.insert(PACKET.into(), raw.clone());
        self.after.insert(PACKET.into(), raw.clone());
        let mut state: LifecycleState = serde_json::from_str(&self.after[STATE]).unwrap();
        state.publication_packet_sha256 = Some(artifacts::digest(&raw));
        self.after.insert(STATE.into(), encode(&state));
        self.save();
    }
    fn refuse(&self, expected: &str) {
        let files = self.after.clone();
        assert_eq!(self.accept().unwrap_err(), expected);
        assert_eq!(self.after, files);
    }
}
fn advance(state: &mut LifecycleState, stage: LifecycleStage) {
    state.lifecycle_stage = stage;
    state.support_tier = life::SupportTier::PublicationBacked;
    state.required_evidence = life::required_evidence_for_stage(stage).to_vec();
    state.satisfied_evidence = state.required_evidence.clone();
}
#[test]
fn private_publication_and_proving_bind_exact_artifact_and_record() {
    for kind in [Kind::Publication, Kind::Proving] {
        let f = Fixture::new(kind);
        let entry = f.accept().unwrap();
        assert_eq!(entry.record_sha256(), f.record.stated_sha256().unwrap());
        match entry {
            AcceptanceEntry::Publication { packet, .. } => {
                assert_eq!(packet.path, PACKET);
                assert_eq!(packet.sha256, artifacts::digest(&f.after[PACKET]));
            }
            AcceptanceEntry::ProvingRunCloseout { closeout, .. } => {
                assert_eq!(closeout.path, CLOSEOUT);
                assert_eq!(closeout.sha256, artifacts::digest(&f.after[CLOSEOUT]));
            }
            _ => panic!("wrong owning entry"),
        }
    }
}
#[test]
fn each_due_obligation_missing_has_its_own_bounded_refusal() {
    for (id, debug_id) in [
        ("A1", "A1"),
        ("E1", "E1"),
        ("G1", "G1"),
        ("LC2", "Lc2"),
        ("OUT3", "Out3"),
        ("R1", "R1"),
        ("V1", "V1"),
        ("X1", "X1"),
        ("X2", "X2"),
        ("X3", "X3"),
        ("X4", "X4"),
    ] {
        for kind in [Kind::Publication, Kind::Proving] {
            let mut f = Fixture::new(kind);
            f.record
                .execution
                .as_mut()
                .unwrap()
                .results
                .retain(|r| serde_json::to_value(r.obligation.id).unwrap() != id);
            reseal(&mut f.record);
            f.save();
            let reason = format!("onboarding is missing a due depth obligation: ObligationRef {{ operation: \"run\", id: Some({debug_id}), capability: None, target: \"x86_64-unknown-linux-gnu\" }}");
            f.refuse(&reason);
        }
    }
}
#[test]
fn wrong_path_event_packet_and_lifecycle_refuse() {
    for kind in [Kind::Publication, Kind::Proving] {
        let mut f = Fixture::new(kind);
        f.record.policy.selection.enrollment.lifecycle_path = LifecyclePath::Maintenance;
        f.record.event = Event::Maintenance {
            request_commit: "abc1234".into(),
            request_recorded_at: STAMP.into(),
            trigger_kind: "fixture".into(),
            opened_from: "docs/basis.md".into(),
        };
        reseal(&mut f.record);
        f.save();
        f.refuse("onboarding has no lifecycle-path authority");
        let mut f = Fixture::new(kind);
        f.record.event = Event::Onboarding {
            approval_commit: "bbbbbbb".into(),
            approval_recorded_at: STAMP.into(),
        };
        f.save();
        f.refuse("onboarding approval is not the exact current Event/version");
        let mut f = Fixture::new(kind);
        f.after.insert(
            PACKET.into(),
            f.after[PACKET].replace("baseline_runtime", "publication_backed"),
        );
        assert!(f.accept().unwrap_err().contains("support_tier_at_emit"));
        let mut f = Fixture::new(kind);
        f.after.insert(
            STATE.into(),
            f.after[STATE].replace("a".repeat(64).as_str(), "b".repeat(64).as_str()),
        );
        f.refuse("onboarding lifecycle is not the exact approval/P/O binding");
    }
}
#[test]
fn stale_evidence_and_pre_entry_verdicts_cannot_satisfy_onboarding() {
    let mut f = Fixture::new(Kind::Publication);
    f.after.insert(EVIDENCE.into(), "edited receipt".into());
    f.refuse("onboarding obligation is not independently satisfied: ObligationRef { operation: \"run\", id: Some(R1), capability: None, target: \"x86_64-unknown-linux-gnu\" }");
    let mut f = Fixture::new(Kind::Publication);
    let old = crate::depth_evidence::test_results(Some(&f.record));
    f.record.acceptance_entries.push(f.accept().unwrap());
    f.save();
    let error = acceptance(
        &f.view(),
        &source(&f.before),
        &f.record,
        f.kind,
        || {
            Ok((
                f.record.policy.clone(),
                f.record.obligations.clone().unwrap(),
            ))
        },
        || Ok(old),
    )
    .unwrap_err();
    assert_eq!(error,"onboarding obligation is not independently satisfied: ObligationRef { operation: \"run\", id: Some(R1), capability: None, target: \"x86_64-unknown-linux-gnu\" }");
    f.accept().unwrap();
}

#[test]
fn classification_carried_work_and_debt_refuse() {
    for kind in [Kind::Publication, Kind::Proving] {
        for classification in [true, false] {
            let mut f = Fixture::new(kind);
            let mut record = f.record.clone();
            let mut value = serde_json::to_value(&record).unwrap();
            let surface = json!({"surface_kind":"flags","command_path":"depth_fixture run","surface_id":"--value"});
            if classification {
                value["obligations"]["classification_required"] =
                    json!([{"operation":"run","surface":surface}]);
            } else {
                value["obligations"]["acceptance_work"] =
                    json!([{"surface":surface,"targets":["x86_64-unknown-linux-gnu"]}]);
            }
            record = serde_json::from_value(value).unwrap();
            f.record = record;
            f.replace_frozen();
            f.refuse(if classification {
                "onboarding has classification_required units"
            } else {
                "onboarding due name-level work lacks independent semantic corroboration"
            });
        }
        let mut f = Fixture::new(kind);
        f.record
            .execution
            .as_mut()
            .unwrap()
            .debt_retired
            .push("legacy".into());
        reseal(&mut f.record);
        f.save();
        f.refuse("onboarding debt grants, retirements and baseline debt are unsupported");
        let mut f = Fixture::new(kind);
        let path = crate::agent_maintenance::support_audit::NON_TUI_SUPPORT_DEBT_PATH;
        f.after
            .insert(path.into(), f.after[path].clone() + "\nchanged inventory\n");
        f.refuse("onboarding debt inventory drift is unsupported");
    }
}

#[test]
fn aggregate_effect_allowlists_refuse_pointers_status_passed_targets_and_marker_retirement() {
    for kind in [Kind::Publication, Kind::Proving] {
        for (case, expected) in [
            ("pointer", "latest_validated_by_target"),
            ("status", "status"),
            ("passed", "validation.passed_targets"),
            ("marker", "marker retirement"),
        ] {
            let mut f = Fixture::new(kind);
            let path = format!("{ROOT}/versions/{VERSION}.json");
            let metadata = json!({"schema_version":1,"semantic_version":VERSION,"status":"snapshotted","validation":{"passed_targets":[]}});
            f.before.insert(path.clone(), encode(&metadata));
            f.after.insert(path.clone(), encode(&metadata));
            match case {
                "pointer" => {
                    f.after.insert(
                        format!("{ROOT}/pointers/latest_validated/x86_64-unknown-linux-gnu.txt"),
                        "0.2.0\n".into(),
                    );
                }
                "status" => {
                    let mut value = metadata;
                    value["status"] = json!("validated");
                    f.after.insert(path, encode(&value));
                }
                "passed" => {
                    let mut value = metadata;
                    value["validation"]["passed_targets"] = json!(["x86_64-unknown-linux-gnu"]);
                    f.after.insert(path, encode(&value));
                }
                _ => {
                    f.before.insert("docs/agents/lifecycle/depth_fixture-maintenance/governance/automation-stand-down/0.2.0.toml".into(),"marker\n".into());
                }
            }
            // Invoke the actual raw detector and the owning aggregate allowlist.
            let view = f.view();
            let error = effects::check(
                &source(&f.before),
                &source(&f.after),
                &view,
                &f.record,
                kind,
            )
            .unwrap_err();
            assert_eq!(
                error,
                format!("onboarding {kind:?} forbids acceptance effect {expected}")
            );
        }
    }
}

#[test]
fn outside_generation_effect_and_foreign_entry_refuse_the_entire_batch() {
    let mut f = Fixture::new(Kind::Publication);
    f.after.insert(
        format!("{ROOT}/pointers/latest_supported/x86_64-unknown-linux-gnu.txt"),
        "0.3.0\n".into(),
    );
    assert_eq!(
        effects::check(
            &source(&f.before),
            &source(&f.after),
            &f.view(),
            &f.record,
            f.kind
        )
        .unwrap_err(),
        "onboarding batch has acceptance outside its owned generation"
    );
    let mut f = Fixture::new(Kind::Publication);
    f.record
        .acceptance_entries
        .push(AcceptanceEntry::Promotion {
            targets: vec!["x86_64-unknown-linux-gnu".into()],
            record_sha256: f.record.stated_sha256().unwrap(),
        });
    f.save();
    assert_eq!(
        effects::check(
            &source(&f.before),
            &source(&f.after),
            &f.view(),
            &f.record,
            f.kind
        )
        .unwrap_err(),
        "onboarding batch has an entry outside its owning path"
    );
}

#[test]
fn maintenance_and_onboarding_authorities_cannot_substitute() {
    let mut f = Fixture::new(Kind::Publication);
    f.record.acceptance_entries.push(f.accept().unwrap());
    f.save();
    let request_path = life::maintenance_request_path(AGENT);
    let request = format!("agent_id = \"{AGENT}\"\nrequest_commit = \"abc1234\"\nrequest_recorded_at = \"{STAMP}\"\ntrigger_kind = \"fixture\"\nopened_from = \"docs/basis.md\"\n[detected_release]\ntarget_version = \"{VERSION}\"\n");
    let commit = crate::git_inputs::command(&repo())
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(commit.status.success());
    let closeout = encode(
        &json!({"request_ref":request_path, "request_sha256":artifacts::digest(&request), "preflight_passed":true, "recorded_at":STAMP, "commit":String::from_utf8(commit.stdout).unwrap().trim(), "explicit_none_reason":"Private ownership proof"}),
    );
    crate::agent_maintenance::closeout::check_closeout_form(&closeout).unwrap();
    f.after.insert(request_path, request);
    f.after.insert(
        format!("docs/agents/lifecycle/{AGENT}-maintenance/governance/maintenance-closeout.json"),
        closeout,
    );
    // Control: the destination resolves this exact version when its Event has the right owner.
    let mut maintenance = f.record.clone();
    maintenance.event = Event::Maintenance {
        request_commit: "abc1234".into(),
        request_recorded_at: STAMP.into(),
        trigger_kind: "fixture".into(),
        opened_from: "docs/basis.md".into(),
    };
    maintenance.policy.selection.enrollment.lifecycle_path = LifecyclePath::Maintenance;
    reseal(&mut maintenance);
    let marker = crate::agent_maintenance::stand_down::marker_relative_path(
        &format!("docs/agents/lifecycle/{AGENT}-maintenance"),
        VERSION,
    );
    let marker_raw =
        crate::agent_maintenance::stand_down::render_marker_toml(AGENT, VERSION, STAMP);
    f.before.insert(marker.clone(), marker_raw.clone());
    f.after.insert(marker, marker_raw);
    let mut control = f.after.clone();
    control.insert(RECORD.into(), encode(&maintenance));
    let control = View::with_registry(source(&control), &f.view().registry, AGENT).unwrap();
    assert_eq!(
        artifacts::maintenance_owner(&control).unwrap().as_deref(),
        Some(VERSION)
    );
    struct MaintenanceControl(DepthRecord);
    impl super::super::manual::Host for MaintenanceControl {
        fn bindings(
            &self,
            _: &Path,
            _: &DepthRecord,
        ) -> Result<(PolicyPart, ObligationsPart), String> {
            Ok((self.0.policy.clone(), self.0.obligations.clone().unwrap()))
        }
        fn evidence(&self, _: &Path, _: &DepthRecord) -> Result<EvidenceVerdicts, String> {
            Ok(crate::depth_evidence::test_results(Some(&self.0)))
        }
    }
    assert!(matches!(
        super::super::manual::acceptance(
            &repo(),
            &control,
            &maintenance,
            &source(&f.before),
            &MaintenanceControl(maintenance.clone())
        )
        .unwrap(),
        AcceptanceEntry::MaintenanceCloseout { .. }
    ));
    struct Unreachable;
    impl super::super::manual::Host for Unreachable {
        fn bindings(
            &self,
            _: &Path,
            _: &DepthRecord,
        ) -> Result<(PolicyPart, ObligationsPart), String> {
            panic!("wrong owner reached bindings")
        }
        fn evidence(&self, _: &Path, _: &DepthRecord) -> Result<EvidenceVerdicts, String> {
            panic!("wrong owner reached evidence")
        }
    }
    // An onboarding record and even its exact Publication entry cannot satisfy the maintenance owner.
    let error = super::super::manual::acceptance(
        &repo(),
        &f.view(),
        &f.record,
        &source(&f.before),
        &Unreachable,
    )
    .unwrap_err();
    assert_eq!(error, "request Event contradicts recorded generation");
    let mut f = Fixture::new(Kind::Publication);
    f.record.event = Event::Maintenance {
        request_commit: "abc1234".into(),
        request_recorded_at: STAMP.into(),
        trigger_kind: "fixture".into(),
        opened_from: "docs/basis.md".into(),
    };
    f.record.policy.selection.enrollment.lifecycle_path = LifecyclePath::Maintenance;
    reseal(&mut f.record);
    f.save();
    f.refuse("onboarding has no lifecycle-path authority");
}

#[cfg(target_os = "linux")]
mod connected;
#[cfg(target_os = "linux")]
pub(crate) use connected::connected_publication;
mod adversarial;
mod review_round1;
