use super::*;
use crate::{
    agent_maintenance::closeout_proposal::Proposal,
    depth_record::{part_identity, FileRef},
};
use serde_json::{json, Value};
use std::{fs, process::Command};
const REQUEST: &str = "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml";
const CLOSEOUT: &str =
    "docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json";
const RECORD: &str = "cli_manifests/codex/reports/1.2.3/depth-record.json";
const MARKER: &str =
    "docs/agents/lifecycle/codex-maintenance/governance/automation-stand-down/1.2.3.toml";
const EVIDENCE: &str = "crates/codex/tests/fixture-evidence.json";
const LINUX: &str = "x86_64-unknown-linux-gnu";
const STAMP: &str = "2026-10-23T12:00:00Z";
fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn write(root: &Path, path: &str, raw: impl AsRef<[u8]>) {
    fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
    fs::write(root.join(path), raw).unwrap();
}
fn canonical(value: &mut Value, key: &str) {
    match value {
        Value::Object(o) => {
            for (k, v) in o {
                if k != "enrollment" {
                    canonical(v, k);
                }
            }
        }
        Value::Array(a) => {
            for v in &mut *a {
                canonical(v, key);
            }
            if key != "acceptance_entries" {
                a.sort_by_key(Value::to_string);
            }
        }
        _ => (),
    }
}
fn seal(mut value: Value) -> DepthRecord {
    canonical(&mut value, "");
    value["policy"]["identity"] = json!(part_identity(&value["policy"]).unwrap());
    value["obligations"]["policy_identity"] = value["policy"]["identity"].clone();
    value["obligations"]["identity"] = json!(part_identity(&value["obligations"]).unwrap());
    value["execution"]["policy_identity"] = value["policy"]["identity"].clone();
    value["execution"]["obligations_identity"] = value["obligations"]["identity"].clone();
    value["execution"]["identity"] = json!(part_identity(&value["execution"]).unwrap());
    DepthRecord::parse(&value.to_string(), "codex", "1.2.3").unwrap()
}
fn record() -> DepthRecord {
    let ids = [
        "A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4", "LC2", "OUT3",
    ];
    let file = FileRef {
        path: EVIDENCE.into(),
        sha256: artifacts::digest("private producer fixture"),
    };
    let mut results: Vec<_> = ids.iter().map(|id| json!({"obligation":{"operation":"run", "id":id, "target":LINUX}, "result":"verified", "evidence":[file]})).collect();
    results.push(json!({"obligation":{"operation":"run", "capability":"agent_api.run", "target":LINUX}, "result":"verified", "evidence":[file]}));
    seal(json!({
        "schema_version":1,
        "event":{"lifecycle_path":"maintenance", "request_commit":"abc", "request_recorded_at":STAMP, "trigger_kind":"upstream_release_detected", "opened_from":"docs/basis.md"},
        "policy":{"identity":"", "selection":{"agent_id":"codex", "enrollment":{
            "lifecycle_path":"maintenance", "version":"1.2.3", "targets":[LINUX],
            "modes":[{"selector":"exec", "command_path":"codex exec", "mode":"headless", "predicates":[]}],
            "operations":[{"id":"run", "serves":["exec"], "templates":["CORE"], "shared_mapping":"mapped", "forwarding":false, "not_applicable_permitted":["X1"],
                "promise":{"r1":"r", "v1":"v", "g1":"g", "e1":"e", "a1":"a", "x1":"x", "x2":"x", "x3":"x", "x4":"x"}}],
            "capabilities":[{"id":"agent_api.run", "operations":["run"]}]}},
            "rules_revision":"1", "base":["A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4"], "templates":[{"template":"CORE", "obligations":["LC2", "OUT3"]}], "path_enabled":true, "debt_baseline":[]},
        "obligations":{"identity":"", "policy_identity":"", "targets":[LINUX], "acquired_inputs":[], "operations":[{"id":"run", "surfaces":[], "obligations":ids}], "classification_required":[], "acceptance_work":[]},
        "execution":{"identity":"", "policy_identity":"", "obligations_identity":"", "results":results, "reuse_bindings":[], "debt_grants":[], "debt_retired":[]}, "acceptance_entries":[]
    }))
}
// This private test producer is fixed independently of each proposed record. Its sealed verdicts
// exercise the owning writer's use of the evidence boundary; they are not production/CI proof.
struct Producer {
    current: DepthRecord,
    proved: bool,
}
impl Host for Producer {
    fn bindings(&self, _: &Path, _: &DepthRecord) -> Result<(PolicyPart, ObligationsPart), String> {
        Ok((
            self.current.policy.clone(),
            self.current.obligations.clone().unwrap(),
        ))
    }
    fn evidence(&self, _: &Path, record: &DepthRecord) -> Result<EvidenceVerdicts, String> {
        // Re-evaluate the independently fixed substantive state for this exact query. Entries
        // do not establish evidence; owning admission checks them separately. A prior opaque
        // set remains unusable after entry changes, so every query constructs a fresh set.
        let mut evaluated = self.current.clone();
        evaluated.acceptance_entries = record.acceptance_entries.clone();
        Ok(if self.proved && evaluated == *record {
            crate::depth_evidence::test_results(Some(&evaluated))
        } else {
            EvidenceVerdicts::default()
        })
    }
}
struct Fixture {
    root: tempfile::TempDir,
    record: DepthRecord,
    closeout: Vec<u8>,
    producer: Producer,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let r = root.path();
        write(
            r,
            REGISTRY_RELATIVE_PATH,
            include_str!("../../../data/agent_registry.toml"),
        );
        write(r, REQUEST, format!("agent_id = \"codex\"\nrequest_commit = \"abc\"\nrequest_recorded_at = \"{STAMP}\"\ntrigger_kind = \"upstream_release_detected\"\nopened_from = \"docs/basis.md\"\n[detected_release]\ntarget_version = \"1.2.3\"\n"));
        write(r, EVIDENCE, "private producer fixture");
        write(r, "cli_manifests/codex/RULES.json", "{}");
        write(
            r,
            NON_TUI_SUPPORT_DEBT_PATH,
            "### `support-debt-authorization-contract-target-version-v1`\n",
        );
        let record = record();
        write(r, RECORD, serde_json::to_vec_pretty(&record).unwrap());
        write(
            r,
            MARKER,
            stand_down::render_marker_toml("codex", "1.2.3", STAMP),
        );
        git(r, &["init", "-q"]);
        git(r, &["config", "user.name", "Fixture"]);
        git(r, &["config", "user.email", "fixture@example.invalid"]);
        git(r, &["add", "."]);
        git(r, &["commit", "-qm", "frozen generation"]);
        git(r, &["update-ref", "refs/remotes/origin/staging", "HEAD"]);
        let closeout = serde_json::to_vec_pretty(&json!({"request_ref":REQUEST, "request_sha256":artifacts::digest(&fs::read_to_string(r.join(REQUEST)).unwrap()), "resolved_findings":[{"category_id":"governance_doc_drift", "summary":"fixture", "surfaces":["docs/basis.md"]}], "explicit_none_reason":"fixture", "preflight_passed":true, "recorded_at":STAMP, "commit":git(r, &["rev-parse", "HEAD"])})).unwrap();
        let producer = Producer {
            current: record.clone(),
            proved: true,
        };
        Self {
            root,
            record,
            closeout,
            producer,
        }
    }
    fn save(&self) {
        write(
            self.root.path(),
            RECORD,
            serde_json::to_vec_pretty(&self.record).unwrap(),
        );
    }
    fn plan(&self) -> Result<Vec<crate::workspace_mutation::PlannedMutation>, String> {
        let proposal = Proposal::capture(self.root.path())?;
        proposal.plan_with(
            self.root.path(),
            vec![(CLOSEOUT.into(), self.closeout.clone())],
            |root, before, after, paths| plan_with(root, before, after, paths, &self.producer),
        )
    }
    fn refuse(&self, expected: &str) {
        let prior = fs::read(self.root.path().join(RECORD)).unwrap();
        let error = self.plan().expect_err("must refuse");
        assert!(error.contains(expected), "{error}");
        assert!(!self.root.path().join(CLOSEOUT).exists());
        assert_eq!(fs::read(self.root.path().join(RECORD)).unwrap(), prior);
    }
}
#[test]
fn positive_closeout_and_exact_entry_are_delivered_in_one_checked_batch() {
    let mut f = Fixture::new();
    f.record
        .acceptance_entries
        .push(AcceptanceEntry::Promotion {
            targets: vec![LINUX.into()],
            record_sha256: "e".repeat(64),
        });
    f.save();
    f.producer.current = f.record.clone();
    let plans = f.plan().unwrap();
    assert_eq!(plans.len(), 2);
    crate::workspace_mutation::apply_admitted_mutations(f.root.path(), &plans).unwrap();
    assert_eq!(fs::read(f.root.path().join(CLOSEOUT)).unwrap(), f.closeout);
    let accepted = DepthRecord::parse(
        &fs::read_to_string(f.root.path().join(RECORD)).unwrap(),
        "codex",
        "1.2.3",
    )
    .unwrap();
    assert_eq!(
        accepted.acceptance_entries[0],
        f.record.acceptance_entries[0]
    );
    assert_eq!(
        accepted.stated_sha256().unwrap(),
        f.record.stated_sha256().unwrap()
    );
    assert_eq!(accepted.acceptance_entries.len(), 2);
    assert_eq!(
        accepted.acceptance_entries[1],
        AcceptanceEntry::MaintenanceCloseout {
            closeout: FileRef {
                path: CLOSEOUT.into(),
                sha256: artifacts::digest(std::str::from_utf8(&f.closeout).unwrap())
            },
            record_sha256: f.record.stated_sha256().unwrap()
        }
    );
}
#[test]
fn no_public_claim_or_existing_entry_can_replace_evaluated_evidence() {
    let mut f = Fixture::new();
    f.producer.proved = false;
    f.record
        .acceptance_entries
        .push(AcceptanceEntry::Promotion {
            targets: vec![LINUX.into()],
            record_sha256: f.record.stated_sha256().unwrap(),
        });
    f.save();
    f.refuse("not independently satisfied");
}
#[test]
fn missing_m_result_and_missing_base_result_both_refuse() {
    for capability in [true, false] {
        let mut f = Fixture::new();
        let mut value = serde_json::to_value(&f.record).unwrap();
        value["execution"]["results"]
            .as_array_mut()
            .unwrap()
            .retain(|r| {
                if capability {
                    r["obligation"]["capability"].is_null()
                } else {
                    r["obligation"]["id"] != "R1"
                }
            });
        f.record = seal(value);
        f.save();
        f.producer.current = f.record.clone();
        f.refuse("missing a due depth obligation");
    }
}
#[test]
fn changed_p_and_o_cannot_be_resealed_into_currentness() {
    for policy in [true, false] {
        let mut f = Fixture::new();
        let mut value = serde_json::to_value(&f.record).unwrap();
        if policy {
            value["policy"]["selection"]["enrollment"]["operations"][0]["promise"]["r1"] =
                json!("different promise");
        } else {
            value["obligations"]["acquired_inputs"] =
                json!([{ "path":"docs/new-input.md", "sha256":"a".repeat(64) }]);
        }
        f.record = seal(value);
        f.save();
        f.refuse(if policy {
            "no longer current"
        } else {
            "O is no longer current"
        });
    }
}
#[test]
fn classification_and_carried_due_work_each_refuse() {
    for classification in [true, false] {
        let mut f = Fixture::new();
        let mut value = serde_json::to_value(&f.record).unwrap();
        let surface =
            json!({"surface_kind":"flags", "command_path":"codex exec", "surface_id":"--model"});
        if classification {
            value["obligations"]["classification_required"] =
                json!([{ "operation":"run", "surface":surface }]);
        } else {
            value["obligations"]["acceptance_work"] =
                json!([{ "surface":surface, "targets":[LINUX] }]);
        }
        f.record = seal(value);
        f.save();
        f.producer.current = f.record.clone();
        f.refuse(if classification {
            "classification_required"
        } else {
            "due name-level work"
        });
    }
}
#[test]
fn stale_e_binding_and_stale_evidence_refuse() {
    for binding in [true, false] {
        let mut f = Fixture::new();
        if binding {
            let e = f.record.execution.as_mut().unwrap();
            e.obligations_identity = "a".repeat(64);
            e.identity = part_identity(e).unwrap();
            f.save();
        } else {
            write(f.root.path(), EVIDENCE, "changed dependency/receipt");
        }
        f.refuse(if binding {
            "P/O/E binding is stale"
        } else {
            "not independently satisfied"
        });
    }
}
#[test]
fn wrong_generation_marker_refuses_even_when_the_version_is_protected() {
    let f = Fixture::new();
    write(
        f.root.path(),
        MARKER,
        stand_down::render_marker_toml("codex", "1.2.3", "2026-10-22T12:00:00Z"),
    );
    git(f.root.path(), &["add", MARKER]);
    git(
        f.root.path(),
        &["commit", "-qm", "different generation marker"],
    );
    git(
        f.root.path(),
        &["update-ref", "refs/remotes/origin/staging", "HEAD"],
    );
    f.refuse("another generation");
}
#[test]
fn marker_only_on_candidate_refuses() {
    let f = Fixture::new();
    git(f.root.path(), &["rm", "-q", MARKER]);
    git(
        f.root.path(),
        &["commit", "-qm", "integration without marker"],
    );
    git(
        f.root.path(),
        &["update-ref", "refs/remotes/origin/staging", "HEAD"],
    );
    write(
        f.root.path(),
        MARKER,
        stand_down::render_marker_toml("codex", "1.2.3", STAMP),
    );
    f.refuse("missing advertising input");
}
#[test]
fn hash_only_marker_requires_exact_request_identity() {
    for valid in [true, false] {
        let f = Fixture::new();
        let digest = if valid {
            artifacts::digest(&fs::read_to_string(f.root.path().join(REQUEST)).unwrap())
        } else {
            "a".repeat(64)
        };
        write(f.root.path(), MARKER, format!("schema_version = 1\nagent_id = \"codex\"\ntarget_version = \"1.2.3\"\nreason = \"fixture\"\nrequest_sha256 = \"{digest}\"\n"));
        git(f.root.path(), &["add", MARKER]);
        git(f.root.path(), &["commit", "-qm", "hash marker"]);
        git(
            f.root.path(),
            &["update-ref", "refs/remotes/origin/staging", "HEAD"],
        );
        if valid {
            f.plan().unwrap();
        } else {
            f.refuse("another generation");
        }
    }
}
#[test]
fn concurrent_input_edit_is_preserved_and_no_closeout_or_entry_is_applied() {
    let f = Fixture::new();
    let proposal = Proposal::capture(f.root.path()).unwrap();
    let error = proposal
        .plan_with(
            f.root.path(),
            vec![(CLOSEOUT.into(), f.closeout.clone())],
            |root, before, after, paths| {
                let output = plan_with(root, before, after, paths, &f.producer)?;
                write(root, REQUEST, "concurrent maintainer edit");
                Ok(output)
            },
        )
        .unwrap_err();
    assert!(error.contains("inputs changed"), "{error}");
    assert_eq!(
        fs::read_to_string(f.root.path().join(REQUEST)).unwrap(),
        "concurrent maintainer edit"
    );
    assert!(!f.root.path().join(CLOSEOUT).exists());
    assert_eq!(
        fs::read(f.root.path().join(RECORD)).unwrap(),
        serde_json::to_vec_pretty(&f.record).unwrap()
    );
}
#[test]
fn concurrent_output_edit_between_plan_and_apply_is_preserved() {
    let f = Fixture::new();
    let plans = f.plan().unwrap();
    write(f.root.path(), CLOSEOUT, "concurrent closeout");
    let prior = fs::read(f.root.path().join(RECORD)).unwrap();
    crate::workspace_mutation::apply_admitted_mutations(f.root.path(), &plans).unwrap_err();
    assert_eq!(
        fs::read_to_string(f.root.path().join(CLOSEOUT)).unwrap(),
        "concurrent closeout"
    );
    assert_eq!(fs::read(f.root.path().join(RECORD)).unwrap(), prior);
}

#[path = "tests/regressions.rs"]
mod regressions;

mod promotions;

mod integration;

#[test]
fn private_producer_freshly_seals_entries_but_keeps_substantive_state_fixed() {
    let f = Fixture::new();
    let obligation = f.record.execution.as_ref().unwrap().results[0]
        .obligation
        .clone();
    let old = f.producer.evidence(f.root.path(), &f.record).unwrap();
    let mut with_entry = f.record.clone();
    with_entry
        .acceptance_entries
        .push(AcceptanceEntry::Promotion {
            targets: vec![LINUX.into()],
            record_sha256: f.record.stated_sha256().unwrap(),
        });
    assert_eq!(
        old.result(&with_entry, &obligation),
        DepthResult::Unverified
    );
    let fresh = f.producer.evidence(f.root.path(), &with_entry).unwrap();
    assert_eq!(
        fresh.result(&with_entry, &obligation),
        DepthResult::Verified
    );
    with_entry.execution.as_mut().unwrap().identity = "f".repeat(64);
    let changed = f.producer.evidence(f.root.path(), &with_entry).unwrap();
    assert_eq!(
        changed.result(&with_entry, &obligation),
        DepthResult::Unverified
    );
}
