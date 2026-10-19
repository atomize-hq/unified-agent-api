//! Depth records: the checks the root's `VALIDATOR_SPEC.md` states in "Depth records", which are
//! checks 1, 3, 10 and 13 of `docs/agents/lifecycle/support-depth-validation-test-list.md`.
//! Checks 5, 6, 7, 8 and 12 are the support publication's, decided by `support-matrix --check`.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use xtask::{
    agent_lifecycle::{self, EvidenceId},
    agent_maintenance::closeout,
    agent_registry::AgentRegistry,
    depth_record::{DepthRecord, Event, RequestBinding, DEPTH_RECORD_FILE_NAME},
    proving_run_closeout,
};

use super::{rel_path, workspace_root_for, ValidateCtx, Violation};

/// The standing request, read only for what checks 1 and 3 compare. The validating loader
/// refuses a `[support_depth]` table while no depth enrollment can be declared, and refuses a
/// first-freeze placeholder, so this reader fails only on a file that is not TOML.
#[derive(Default, Deserialize)]
#[serde(default)]
struct StandingRequest {
    request_commit: String,
    request_recorded_at: String,
    trigger_kind: String,
    opened_from: String,
    detected_release: Option<DetectedRelease>,
    support_depth: Option<RequestTable>,
}

#[derive(Deserialize)]
struct DetectedRelease {
    target_version: Option<String>,
}

#[derive(Deserialize)]
struct RequestTable {
    policy_identity: Option<String>,
    obligations_identity: Option<String>,
}

pub(super) fn validate_depth_records(ctx: &ValidateCtx, violations: &mut Vec<Violation>) {
    // Canonical, as the workspace root is, so the root's path can be named relative to it.
    let root_dir = fs::canonicalize(&ctx.root).unwrap_or_else(|_| ctx.root.clone());
    let files = record_files(&root_dir);
    let workspace = workspace_root_for(&ctx.root);
    let base = workspace.clone().unwrap_or_else(|| root_dir.clone());
    let mut fail = |code: &'static str, path: &Path, message: String| {
        violations.push(violation(code, rel_path(&base, path), message));
    };

    // `RULES.json` adopts Annex B by naming where a record is held.
    if !files.is_empty() && ctx.depth_record_path.is_none() {
        for (_, path) in &files {
            let message = "RULES.json names no depth record, so this root does not adopt Annex B";
            fail("DEPTH_RECORD_UNADOPTED", path, message.to_string());
        }
        return;
    }
    let Some(workspace) = workspace else {
        for (_, path) in &files {
            let message = "outside a workspace no registry entry names the record's agent";
            fail("DEPTH_RECORD_UNANCHORED", path, message.to_string());
        }
        return;
    };
    let root = rel_path(&workspace, &root_dir);
    let expected = format!("{root}/reports/<version>/{DEPTH_RECORD_FILE_NAME}");
    if let Some(named) = ctx
        .depth_record_path
        .as_deref()
        .filter(|named| *named != expected)
    {
        let message = format!(
            "storage.committed.reports.depth_record names `{named}`; this root holds its records at `{expected}`"
        );
        let rules = fs::canonicalize(&ctx.rules_path).unwrap_or_else(|_| ctx.rules_path.clone());
        fail("DEPTH_RECORD_PATH_INVALID", &rules, message);
    }
    // A registry that does not load fails the support publication check already.
    let Ok(registry) = AgentRegistry::load(&workspace) else {
        return;
    };
    let Some(entry) = registry
        .agents
        .iter()
        .find(|entry| entry.manifest_root == root)
    else {
        for (_, path) in &files {
            let message = format!("no registry entry has the manifest root `{root}`");
            fail("DEPTH_RECORD_UNANCHORED", path, message);
        }
        return;
    };

    // Check 13: every identity is the digest of its part, and the record is readable at all.
    let mut records = BTreeMap::new();
    for (version, path) in &files {
        let parsed = fs::read_to_string(path)
            .map_err(|err| err.to_string())
            .and_then(|raw| DepthRecord::parse(&raw, &entry.agent_id, version));
        match parsed {
            Ok(record) => {
                records.insert(version.clone(), (path, record));
            }
            Err(err) => fail("DEPTH_RECORD_INVALID", path, err),
        }
    }

    // Checks 1 and 3, against the standing request. The onboarding path has no second statement
    // of P and O yet, so these checks are decided on the maintenance path only.
    let request_path = workspace.join(agent_lifecycle::maintenance_request_path(&entry.agent_id));
    // A request that is absent is no request. One that is there and cannot be read fails, as a
    // record that cannot be read does.
    let text = match fs::read_to_string(&request_path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            fail("DEPTH_REQUEST_UNREADABLE", &request_path, err.to_string());
            None
        }
        Ok(text) => Some(text),
    };
    if let Some(text) = text {
        match toml_edit::de::from_str::<StandingRequest>(&text) {
            Err(err) => fail("DEPTH_REQUEST_UNREADABLE", &request_path, err.to_string()),
            Ok(request) => {
                let version = request.detected_release.and_then(|r| r.target_version);
                let froze_p = request.support_depth.is_some();
                let table = request.support_depth.unwrap_or(RequestTable {
                    policy_identity: None,
                    obligations_identity: None,
                });
                let binding = RequestBinding {
                    event: Event::Maintenance {
                        request_commit: request.request_commit,
                        request_recorded_at: request.request_recorded_at,
                        trigger_kind: request.trigger_kind,
                        opened_from: request.opened_from,
                    },
                    policy_identity: table.policy_identity,
                    obligations_identity: table.obligations_identity,
                };
                let held = version
                    .as_ref()
                    .filter(|v| files.iter().any(|(held, _)| held == *v));
                match held {
                    Some(version) => {
                        if let Some((path, record)) = records.get(version) {
                            if let Some(why) = record.request_mismatch(&binding) {
                                let message = format!("{why} for version `{version}` (check 3)");
                                fail("DEPTH_RECORD_NOT_THE_REQUESTS", path, message);
                            }
                        }
                    }
                    None if froze_p => {
                        let version = version.as_deref().unwrap_or("none");
                        let message = format!(
                            "the request froze P for version `{version}`, which has no depth record (check 1)"
                        );
                        fail("DEPTH_RECORD_MISSING", &request_path, message);
                    }
                    None => {}
                }
            }
        }
    }

    // Check 10: a lifecycle stage or closeout evidence id reports a file the revision holds.
    if files.is_empty() {
        return;
    }
    let lifecycle = agent_lifecycle::lifecycle_state_path_for_entry(entry);
    let lifecycle_path = workspace.join(&lifecycle);
    if !lifecycle_path.is_file() {
        return;
    }
    match agent_lifecycle::load_lifecycle_state(&workspace, &lifecycle) {
        Err(err) => fail("DEPTH_LIFECYCLE_DANGLING", &lifecycle_path, err.to_string()),
        Ok(state) => {
            // A closeout is held when the file at its path is a closeout of its kind in form; the
            // lifecycle record names the proving-run closeout by path alone.
            let held = |path: &Path, form: fn(&str) -> Result<(), String>| {
                fs::read_to_string(path)
                    .map_err(|err| err.to_string())
                    .and_then(|text| form(&text))
            };
            let satisfied = |id| state.satisfied_evidence.contains(&id);
            let proving = state.closeout_baseline_path.as_deref();
            if satisfied(EvidenceId::ProvingRunCloseoutWritten) && proving.is_none() {
                let message = "`proving_run_closeout_written` names no proving-run closeout";
                fail(
                    "DEPTH_LIFECYCLE_DANGLING",
                    &lifecycle_path,
                    message.to_string(),
                );
            }
            if let Some(path) = proving {
                if let Err(err) = held(
                    &workspace.join(path),
                    proving_run_closeout::check_closeout_form,
                ) {
                    let message = format!("`{path}` is not a proving-run closeout: {err}");
                    fail("DEPTH_LIFECYCLE_DANGLING", &lifecycle_path, message);
                }
            }
            let closeout_path = request_path.with_file_name("maintenance-closeout.json");
            if satisfied(EvidenceId::MaintenanceCloseoutWritten) {
                if let Err(err) = held(&closeout_path, closeout::check_closeout_form) {
                    let path = rel_path(&workspace, &closeout_path);
                    let message = format!(
                        "`maintenance_closeout_written` reports `{path}`, which is not a maintenance closeout: {err}"
                    );
                    fail("DEPTH_LIFECYCLE_DANGLING", &lifecycle_path, message);
                }
            }
        }
    }
}

/// Every `reports/<version>/depth-record.json` under the root, by version.
fn record_files(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(root.join("reports")) else {
        return Vec::new();
    };
    let mut files: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path().join(DEPTH_RECORD_FILE_NAME))
        // Any entry at the name is a record, so one that cannot be read fails rather than hides.
        .filter(|path| fs::symlink_metadata(path).is_ok())
        .filter_map(|path| {
            let version = path.parent()?.file_name()?.to_string_lossy().into_owned();
            Some((version, path))
        })
        .collect();
    files.sort();
    files
}

fn violation(code: &'static str, path: String, message: String) -> Violation {
    Violation {
        code,
        path,
        json_pointer: None,
        message,
        unit: Some("depth_record"),
        command_path: None,
        key_or_name: None,
        field: None,
        target_triple: None,
        details: None,
    }
}
