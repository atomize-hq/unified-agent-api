//! T5 — closeout finding derivation.
//!
//! The closeout carries two finding sets with opposite meanings, and the frozen validator checks
//! them against a live drift re-derivation with no tolerance in either direction:
//!
//! - `resolved_findings` — what the packet fixed. Its signatures must **not** appear in live drift.
//! - `deferred_findings` — what is still outstanding. Its signatures must equal the live set
//!   **exactly**, both directions, or `explicit_none_reason` when live drift is clean.
//!
//! So the deferred half is not a judgement at all: the validator already decided it, and anything
//! this module does other than transcribe the live report is a disagreement it will lose. The only
//! real decision is the resolved half, and the risk there is the inverse of the usual one — the
//! validator checks that a resolved signature is *absent* from live drift, which a finding over
//! surfaces that could never drift satisfies vacuously. A derivation that invents its own surface
//! list would pass while asserting nothing.
//!
//! Both halves are therefore derived from something already authoritative:
//!
//! - **Resolved surfaces come from the packet's own write envelope** — the version-scoped manifest
//!   artifacts `contract_policy` permits a packet to write, filtered to the ones that exist. The
//!   closeout cannot claim a surface the packet was never allowed to touch, and cannot claim one
//!   that was never written.
//! - **Deferred findings come from `drift::check_agent_drift`**, transcribed one to one.
//!
//! Three refusals, each of which the obvious implementation gets wrong:
//!
//! - **A drift check that errors is not a clean drift check.** A missing, malformed or unreadable
//!   input must refuse. Falling through to `explicit_none_reason` would record "nothing is
//!   outstanding" on the strength of not having looked.
//! - **A live category the closeout schema cannot express must refuse.**
//!   [`drift::DriftCategory`] has six variants and `MaintenanceDriftCategory` has five:
//!   `runtime_evidence_drift` has no counterpart. The validator demands every live signature appear
//!   in `deferred_findings`, and `validate_finding` rejects that category id, so such a packet has
//!   no valid closeout at all. Refusing names the repair; emitting anything else would be a
//!   generator writing a file it knows the validator rejects.
//! - **Claiming a category resolved while live drift still reports it is a contradiction.** Exact
//!   signature collision is unlikely — live surfaces differ from the envelope — so the validator
//!   would let it through. This module refuses on the category alone, because the packet's work in
//!   that category is by definition not finished.

use std::{collections::BTreeSet, path::Path};

use crate::{agent_registry::AgentRegistry, support_matrix};

use super::{
    super::drift,
    types::{
        DeferredFindingsTruth, MaintenanceCloseoutError, MaintenanceDriftCategory,
        MaintenanceFinding,
    },
};

/// What T5 derives, ready for T6 to place in the artifact.
#[derive(Debug, Clone)]
pub struct DerivedFindings {
    pub resolved_findings: Vec<MaintenanceFinding>,
    pub deferred_findings: DeferredFindingsTruth,
}

pub fn derive_findings(
    workspace_root: &Path,
    agent_id: &str,
    target_version: &str,
) -> Result<DerivedFindings, MaintenanceCloseoutError> {
    let live = check_live_drift(workspace_root, agent_id)?;
    derive_findings_from_report(workspace_root, agent_id, target_version, &live)
}

/// The half that does not run the drift check, mirroring `validate_live_drift_report`: the
/// derivation and the validator can then be driven from one report, which is what "the derivation
/// satisfies the validator" has to mean to be worth asserting.
pub fn derive_findings_from_report(
    workspace_root: &Path,
    agent_id: &str,
    target_version: &str,
    live: &drift::AgentDriftReport,
) -> Result<DerivedFindings, MaintenanceCloseoutError> {
    let resolved_findings = derive_resolved(workspace_root, agent_id, target_version)?;

    // A category cannot be both fixed and outstanding. The validator compares whole signatures and
    // would not catch this, because the live surfaces are whatever drifted and the resolved ones are
    // the write envelope.
    let live_categories: BTreeSet<&str> = live.findings.iter().map(|f| f.category_id()).collect();
    for finding in &resolved_findings {
        if live_categories.contains(finding.category_id.as_id()) {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "cannot record `{}` as resolved for `{agent_id}`: live drift still reports that \
                 category, so the packet's work in it is not finished. Complete it, or close with \
                 the finding deferred rather than resolved.",
                finding.category_id.as_id()
            )));
        }
    }

    Ok(DerivedFindings {
        resolved_findings,
        deferred_findings: derive_deferred(agent_id, live)?,
    })
}

/// An error here is never evidence of cleanliness.
fn check_live_drift(
    workspace_root: &Path,
    agent_id: &str,
) -> Result<drift::AgentDriftReport, MaintenanceCloseoutError> {
    drift::check_agent_drift(workspace_root, agent_id).map_err(|err| match err {
        drift::DriftCheckError::Validation(message) => MaintenanceCloseoutError::Validation(
            format!("live drift check for `{agent_id}` did not complete, so nothing can be recorded as deferred or absent: {message}"),
        ),
        drift::DriftCheckError::Internal(message) => MaintenanceCloseoutError::Internal(format!(
            "live drift check for `{agent_id}` did not complete: {message}"
        )),
    })
}

fn derive_deferred(
    agent_id: &str,
    live: &drift::AgentDriftReport,
) -> Result<DeferredFindingsTruth, MaintenanceCloseoutError> {
    if live.findings.is_empty() {
        return Ok(DeferredFindingsTruth::ExplicitNone(format!(
            "`check-agent-drift --agent {agent_id}` reports status: clean, so no maintenance drift \
             finding remains deferred. Derived from the live report at closeout time, not carried \
             forward from the request."
        )));
    }

    let mut deferred = Vec::with_capacity(live.findings.len());
    for finding in &live.findings {
        let Some(category_id) = MaintenanceDriftCategory::parse(finding.category_id()) else {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "live drift for `{agent_id}` reports category `{}`, which the closeout schema \
                 cannot express — `MaintenanceDriftCategory` has no counterpart for it. The \
                 validator requires every live finding to appear in `deferred_findings` and \
                 rejects this category id, so no valid closeout exists while it is present. \
                 Repair the underlying condition and re-run; do not close around it.",
                finding.category_id()
            )));
        };
        if finding.surfaces.is_empty() {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "live drift for `{agent_id}` reports category `{}` with no surfaces; \
                 `deferred_findings[].surfaces` must not be empty.",
                finding.category_id()
            )));
        }
        deferred.push(MaintenanceFinding {
            category_id,
            summary: finding.summary.clone(),
            surfaces: finding.surfaces.clone(),
        });
    }
    Ok(DeferredFindingsTruth::Findings(deferred))
}

/// The version-scoped manifest artifacts the packet materialized, plus the publication surfaces it
/// regenerated — each included only when it exists on disk.
fn derive_resolved(
    workspace_root: &Path,
    agent_id: &str,
    target_version: &str,
) -> Result<Vec<MaintenanceFinding>, MaintenanceCloseoutError> {
    let registry = AgentRegistry::load(workspace_root).map_err(|err| {
        MaintenanceCloseoutError::Validation(format!("load agent registry: {err}"))
    })?;
    let entry = registry.find(agent_id).ok_or_else(|| {
        MaintenanceCloseoutError::Validation(format!(
            "`{agent_id}` is not in the agent registry, so its write envelope is unknown"
        ))
    })?;
    let manifest_root = entry.manifest_root.clone();

    // Version-scoped first, and on their own: these are the artifacts that exist only because this
    // packet landed. `wrapper_coverage.json` and `artifacts.lock.json` are version-agnostic and are
    // present for every past version, so counting them toward "the packet materialized something"
    // would let a version with no artifacts at all derive a resolved finding — the vacuous claim
    // this module exists to prevent.
    let mut manifest_surfaces = existing_files_under(
        workspace_root,
        &format!("{manifest_root}/snapshots/{target_version}"),
    );
    manifest_surfaces.extend(existing_files_under(
        workspace_root,
        &format!("{manifest_root}/reports/{target_version}"),
    ));
    let versions_file = format!("{manifest_root}/versions/{target_version}.json");
    if workspace_root.join(&versions_file).is_file() {
        manifest_surfaces.push(versions_file);
    }

    if manifest_surfaces.is_empty() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "no version-scoped manifest artifact exists for `{agent_id}` {target_version} under \
             `{manifest_root}`. A closeout with nothing resolved is either the wrong checkout — \
             these are committed to the packet branch — or an acquisition that never completed."
        )));
    }

    // Only now are the version-agnostic surfaces evidence of *this* packet.
    for candidate in [
        format!("{manifest_root}/wrapper_coverage.json"),
        format!("{manifest_root}/artifacts.lock.json"),
    ] {
        if workspace_root.join(&candidate).is_file() {
            manifest_surfaces.push(candidate);
        }
    }

    let publication_surfaces: Vec<String> = [
        support_matrix::JSON_OUTPUT_PATH,
        support_matrix::MARKDOWN_OUTPUT_PATH,
    ]
    .into_iter()
    .filter(|path| workspace_root.join(path).is_file())
    .map(str::to_string)
    .collect();

    let mut findings = vec![MaintenanceFinding {
        category_id: MaintenanceDriftCategory::RegistryManifest,
        summary: format!(
            "The {agent_id} {target_version} packet materialized the version-scoped manifest \
             artifacts required by the live maintenance request."
        ),
        surfaces: manifest_surfaces,
    }];
    if !publication_surfaces.is_empty() {
        findings.push(MaintenanceFinding {
            category_id: MaintenanceDriftCategory::SupportPublication,
            summary: format!(
                "Support-matrix publication was regenerated to match the landed {agent_id} \
                 {target_version} manifest truth."
            ),
            surfaces: publication_surfaces,
        });
    }
    Ok(findings)
}

/// Sorted repo-relative paths of the regular files directly under `relative_dir`.
///
/// A missing directory yields nothing rather than an error: whether its absence is fatal is
/// `derive_resolved`'s decision, made once over the whole envelope.
fn existing_files_under(workspace_root: &Path, relative_dir: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(workspace_root.join(relative_dir)) else {
        return Vec::new();
    };
    let mut out: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .map(|name| format!("{relative_dir}/{name}"))
        .collect();
    out.sort();
    out
}
