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

pub(super) fn validate_onboarding_binding(
    workspace: &Path,
    entry: &AgentRegistryEntry,
    records: &BTreeMap<String, (&PathBuf, DepthRecord)>,
    fail: &mut dyn FnMut(&'static str, &Path, String),
) {
    let lifecycle = agent_lifecycle::lifecycle_state_path_for_entry(entry);
    let lifecycle_path = workspace.join(&lifecycle);
    // An absent lifecycle record is no onboarding pack, as check 10 reads it, so there is no object
    // to compare; the merge check decides retained records. One that is there and cannot be read
    // is check 10's to report.
    if fs::symlink_metadata(&lifecycle_path).is_err() {
        return;
    }
    let Ok(state) = agent_lifecycle::load_lifecycle_state(workspace, &lifecycle) else {
        return;
    };
    let object = state.support_depth.as_ref();
    let onboarding = records
        .values()
        .any(|(_, record)| matches!(record.event, Event::Onboarding { .. }));
    // No record and no object: P is not yet frozen, or the root is not depth-enrolled at all.
    if object.is_none() && !onboarding {
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
