//! Checks 1 and 3 on the onboarding path: the depth record against the second statement of its P
//! and O, the `support_depth` object of the onboarding pack's lifecycle state (OF-P2 of
//! `docs/agents/lifecycle/support-depth-onboarding-freeze-decisions.md`), compared as a record is
//! compared with a maintenance request's table, with the approval's onboarding Event.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use xtask::{
    agent_lifecycle,
    agent_registry::AgentRegistryEntry,
    depth_record::{
        onboarding::{onboarding_binding, OnboardingApproval},
        DepthRecord, Event,
    },
};

/// `check_10_reports` is whether check 10 reports a lifecycle state that cannot be read, as it
/// does while the root holds a record file; otherwise this check reports it, since such a state
/// may hold an object no record states.
pub(super) fn validate_onboarding_binding(
    workspace: &Path,
    entry: &AgentRegistryEntry,
    records: &BTreeMap<String, (&PathBuf, DepthRecord)>,
    check_10_reports: bool,
    fail: &mut dyn FnMut(&'static str, &Path, String),
) {
    let lifecycle = agent_lifecycle::lifecycle_state_path_for_entry(entry);
    let lifecycle_path = workspace.join(&lifecycle);
    let onboarding: Vec<(&String, &PathBuf)> = records
        .iter()
        .filter(|(_, (_, record))| matches!(record.event, Event::Onboarding { .. }))
        .map(|(version, (path, _))| (version, *path))
        .collect();

    // A deleted lifecycle state deletes the object with it: every onboarding record is then a
    // record without an object. Only a file that is not there is absent; any other failure to
    // look, such as a symlink loop or a parent that is not a directory, is reported. A dangling
    // symlink is there, and is read below.
    match fs::symlink_metadata(&lifecycle_path) {
        Ok(_) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            for (version, path) in &onboarding {
                let message = format!(
                    "the record states an onboarding generation, but `{lifecycle}` does not \
                     exist, so no `support_depth` object states it: a deleted binding for \
                     version `{version}` (check 1)"
                );
                fail("DEPTH_LIFECYCLE_BINDING_MISSING", path, message);
            }
            return;
        }
        Err(err) => {
            let message = format!("`{lifecycle}` cannot be looked up: {err}");
            fail("DEPTH_LIFECYCLE_UNREADABLE", &lifecycle_path, message);
            return;
        }
    }
    // Read the typed state before following workspace links. A non-depth scaffold may name an
    // approval that has not been installed yet. Local form errors still fail: unreadable or
    // malformed depth fields cannot be treated as absence. With any record file or an object,
    // retain the full loader's path, digest and linked-artifact checks.
    let state = fs::read(&lifecycle_path)
        .map_err(|err| err.to_string())
        .and_then(|bytes| {
            let state: agent_lifecycle::LifecycleState =
                serde_json::from_slice(&bytes).map_err(|err| err.to_string())?;
            state.validate().map_err(|err| err.to_string())?;
            if !check_10_reports && state.support_depth.is_none() {
                return Ok(None);
            }
            agent_lifecycle::load_lifecycle_state(workspace, &lifecycle)
                .map(Some)
                .map_err(|err| err.to_string())
        });
    let state = match state {
        Ok(Some(state)) => state,
        Ok(None) => return,
        Err(err) => {
            if !check_10_reports {
                let message = format!("{err}; the lifecycle state may state a depth object");
                fail("DEPTH_LIFECYCLE_UNREADABLE", &lifecycle_path, message);
            }
            return;
        }
    };
    let object = state.support_depth.as_deref();
    // No record and no object: P is not yet frozen, or the root is not depth-enrolled at all.
    if object.is_none() && onboarding.is_empty() {
        return;
    }

    let prefix = &entry.scaffold.onboarding_pack_prefix;
    if state.agent_id != entry.agent_id || &state.onboarding_pack_prefix != prefix {
        let message = format!(
            "the lifecycle state names agent `{}` and pack `{}`, not `{}` and `{prefix}`, the \
             registry entry's",
            state.agent_id, state.onboarding_pack_prefix, entry.agent_id
        );
        fail("DEPTH_LIFECYCLE_NOT_THE_AGENTS", &lifecycle_path, message);
        return;
    }
    let approval_path = &state.approval_artifact_path;
    let approval = fs::read_to_string(workspace.join(approval_path))
        .map_err(|err| err.to_string())
        .and_then(|text| OnboardingApproval::parse(&text, approval_path));
    let approval = match approval {
        Ok(approval) => approval,
        Err(err) => {
            fail(
                "DEPTH_APPROVAL_UNREADABLE",
                &workspace.join(approval_path),
                err,
            );
            return;
        }
    };
    if let Some(why) = approval.attribution_mismatch(&entry.agent_id, &entry.manifest_root, prefix)
    {
        fail(
            "DEPTH_APPROVAL_NOT_THE_AGENTS",
            &workspace.join(approval_path),
            why,
        );
        return;
    }

    let held: BTreeMap<String, DepthRecord> = records
        .iter()
        .map(|(version, (_, record))| (version.clone(), record.clone()))
        .collect();
    if let Err(faults) = onboarding_binding(object, &approval, &held) {
        for fault in faults {
            let path = fault
                .version
                .as_ref()
                .and_then(|version| records.get(version))
                .map(|(path, _)| path.as_path())
                .unwrap_or(&lifecycle_path);
            let message = match &fault.version {
                Some(version) => {
                    format!("{} for version `{version}` (checks 1 and 3)", fault.message)
                }
                None => format!("{} (check 1)", fault.message),
            };
            fail(fault.code, path, message);
        }
    }
}
