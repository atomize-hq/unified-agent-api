//! `uaa-0039`: the closeout checks that enforce the support-audit baseline.
//!
//! Two rules in `docs/specs/maintenance-request-contract-v1.md` gate a packet close and neither had
//! an enforcer. Under "Hidden upstream surfaces and wrapper-only rows", every wrapper-only row — a
//! surface the wrapper claims that the union does not show — must be sorted into one of four
//! categories, and a row sorted obsolete must contract publication truth in the same run. Under
//! field invariant 6, `unmatched_debt_surface` must be empty.
//!
//! Every check re-derives from the repository instead of reading the artifact's own claims. An
//! artifact recording an empty unmatched-debt list cannot establish that the live list is empty,
//! and a disposition that says "obsolete" cannot establish that a contraction happened. The one
//! thing the artifact is trusted for is *which* report it adjudicated, because a correctly
//! contracted row is gone from the live report by the time this runs — and that trust is bounded by
//! requiring the named report to be the one the audit would select today.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use serde::Deserialize;

use crate::agent_registry::AgentRegistry;

use super::super::{
    request,
    support_audit::{derive_support_surface_audit, wrapper_only_baseline, SurfaceIdentity},
};
use super::{MaintenanceCloseout, MaintenanceCloseoutError};

/// The four categories the contract names, in its own order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WrapperOnlyCategory {
    HiddenUpstreamSupported,
    OlderUpstreamOnly,
    Obsolete,
    DiscoveryBug,
}

impl WrapperOnlyCategory {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "hidden_upstream_supported" => Some(Self::HiddenUpstreamSupported),
            "older_upstream_only" => Some(Self::OlderUpstreamOnly),
            "obsolete" => Some(Self::Obsolete),
            "discovery_bug" => Some(Self::DiscoveryBug),
            _ => None,
        }
    }

    pub fn as_id(self) -> &'static str {
        match self {
            Self::HiddenUpstreamSupported => "hidden_upstream_supported",
            Self::OlderUpstreamOnly => "older_upstream_only",
            Self::Obsolete => "obsolete",
            Self::DiscoveryBug => "discovery_bug",
        }
    }
}

/// One adjudicated wrapper-only row.
///
/// The evidence each category owes differs, because the categories assert different things. All
/// four owe an `evidence_ref` that resolves to a file in the repository and a non-empty `note`.
/// `older_upstream_only` additionally owes `last_supported_version`, which is the whole content of
/// the claim. `discovery_bug` owes `follow_on`, because a discovery bug is work the packet is
/// deferring and an untracked deferral is the failure this repository already guards elsewhere.
/// `obsolete` owes no extra field and instead owes a fact: the surface must be gone from the live
/// report, which is what "contract publication truth in the same run" means once the wrapper claim
/// is withdrawn and the report regenerated.
#[derive(Debug, Clone)]
pub struct WrapperOnlyDisposition {
    pub surface: SurfaceIdentity,
    pub category: WrapperOnlyCategory,
    pub evidence_ref: String,
    pub note: String,
    pub last_supported_version: Option<String>,
    pub follow_on: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawWrapperOnlyDisposition {
    surface_kind: String,
    command_path: String,
    surface_id: String,
    category: String,
    evidence_ref: String,
    note: String,
    last_supported_version: Option<String>,
    follow_on: Option<String>,
}

/// Read the dispositions a previous closeout recorded, without validating the rest of that file.
///
/// T6 carries adjudications forward rather than re-asking for them, so it needs the prior
/// `wrapper_only_dispositions` from an artifact that is otherwise stale: its `request_sha256`
/// belongs to an older request generation and its `commit` to an older packet, both of which
/// `load_linked_closeout` would rightly reject. Only this one array survives a version change,
/// because a disposition is a judgement about a *surface*, not about a release.
///
/// The dispositions themselves are held to the full bar — `validate_dispositions` is the same
/// function the validator calls — so a carried row that no longer resolves its `evidence_ref`
/// fails here rather than being copied forward on trust. A missing file and a file with no
/// dispositions are both an empty list: neither is an error, because the first closeout for an
/// agent has no predecessor and a packet with no wrapper-only rows records none.
pub fn read_recorded_dispositions(
    workspace_root: &Path,
    closeout_path: &Path,
) -> Result<Vec<WrapperOnlyDisposition>, MaintenanceCloseoutError> {
    let resolved = workspace_root.join(closeout_path);
    let text = match std::fs::read_to_string(&resolved) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(MaintenanceCloseoutError::Internal(format!(
                "read {}: {err}",
                closeout_path.display()
            )))
        }
    };

    let document = serde_json::from_str::<serde_json::Value>(&text).map_err(|err| {
        MaintenanceCloseoutError::Validation(format!(
            "parse {}: {err}. The recorded closeout must be readable before its dispositions can \
             be carried forward; repair or remove it.",
            closeout_path.display()
        ))
    })?;
    let Some(rows) = document.get("wrapper_only_dispositions") else {
        return Ok(Vec::new());
    };
    let raw =
        serde_json::from_value::<Vec<RawWrapperOnlyDisposition>>(rows.clone()).map_err(|err| {
            MaintenanceCloseoutError::Validation(format!(
                "{}: `wrapper_only_dispositions` is not readable: {err}",
                closeout_path.display()
            ))
        })?;
    validate_dispositions(workspace_root, closeout_path, raw)
}

pub(super) fn validate_dispositions(
    workspace_root: &Path,
    closeout_path: &Path,
    raw: Vec<RawWrapperOnlyDisposition>,
) -> Result<Vec<WrapperOnlyDisposition>, MaintenanceCloseoutError> {
    raw.into_iter()
        .enumerate()
        .map(|(index, row)| validate_disposition(workspace_root, closeout_path, index, row))
        .collect()
}

fn validate_disposition(
    workspace_root: &Path,
    closeout_path: &Path,
    index: usize,
    raw: RawWrapperOnlyDisposition,
) -> Result<WrapperOnlyDisposition, MaintenanceCloseoutError> {
    let field = |name: &str| format!("wrapper_only_dispositions[{index}].{name}");
    let category = WrapperOnlyCategory::parse(&raw.category).ok_or_else(|| {
        invalid(
            closeout_path,
            &field("category"),
            &format!(
                "`{}` is not one of hidden_upstream_supported, older_upstream_only, obsolete, discovery_bug",
                raw.category
            ),
        )
    })?;

    for (name, value) in [
        ("surface_kind", &raw.surface_kind),
        ("command_path", &raw.command_path),
        ("surface_id", &raw.surface_id),
        ("evidence_ref", &raw.evidence_ref),
        ("note", &raw.note),
    ] {
        if value.trim().is_empty() {
            return Err(invalid(closeout_path, &field(name), "must not be empty"));
        }
    }

    // A dangling evidence path is indistinguishable from no evidence at all.
    let evidence_ref = raw.evidence_ref.trim().to_string();
    if !workspace_root.join(&evidence_ref).exists() {
        return Err(invalid(
            closeout_path,
            &field("evidence_ref"),
            &format!("`{evidence_ref}` does not resolve to a file in the repository"),
        ));
    }

    // Each optional field belongs to exactly one category. Requiring it where it is owed keeps the
    // claim honest; refusing it elsewhere stops an obsolete row from carrying a `follow_on` that
    // implies tracking the category never established.
    let last_supported_version = require_for(
        closeout_path,
        &field("last_supported_version"),
        raw.last_supported_version.as_deref(),
        category == WrapperOnlyCategory::OlderUpstreamOnly,
        "older_upstream_only",
    )?;
    let follow_on = require_for(
        closeout_path,
        &field("follow_on"),
        raw.follow_on.as_deref(),
        category == WrapperOnlyCategory::DiscoveryBug,
        "discovery_bug",
    )?;

    Ok(WrapperOnlyDisposition {
        surface: SurfaceIdentity::new(
            raw.surface_kind.trim().to_string(),
            raw.command_path.trim().to_string(),
            raw.surface_id.trim().to_string(),
        ),
        category,
        evidence_ref,
        note: raw.note.trim().to_string(),
        last_supported_version,
        follow_on,
    })
}

fn require_for(
    closeout_path: &Path,
    field: &str,
    value: Option<&str>,
    owed: bool,
    owed_by: &str,
) -> Result<Option<String>, MaintenanceCloseoutError> {
    match (owed, value.map(str::trim).filter(|v| !v.is_empty())) {
        (true, Some(value)) => Ok(Some(value.to_string())),
        (true, None) => Err(invalid(
            closeout_path,
            field,
            &format!("is required for category `{owed_by}`"),
        )),
        (false, Some(_)) => Err(invalid(
            closeout_path,
            field,
            &format!("is only allowed for category `{owed_by}`"),
        )),
        (false, None) => Ok(None),
    }
}

/// Reconciles the recorded dispositions against what the repository says right now.
pub(crate) fn validate_live_support_audit_truth(
    workspace_root: &Path,
    closeout_path: &Path,
    agent_id: &str,
    detected_release: Option<&request::DetectedRelease>,
    closeout: &MaintenanceCloseout,
) -> Result<(), MaintenanceCloseoutError> {
    // Without a detected release there is no target version, so no coverage report is bound and no
    // wrapper-only obligation is derivable. Dispositions must then be absent rather than unchecked:
    // an unbound disposition asserts an adjudication nothing can verify.
    let Some(detected_release) = detected_release else {
        return reject_unbound(
            closeout_path,
            closeout,
            "the request declares no detected release",
        );
    };

    let registry = AgentRegistry::load(workspace_root).map_err(|err| {
        MaintenanceCloseoutError::Internal(format!(
            "{}: load agent registry: {err}",
            closeout_path.display()
        ))
    })?;
    let entry = registry.find(agent_id).ok_or_else(|| {
        MaintenanceCloseoutError::Validation(format!(
            "{}: `{agent_id}` is not an enrolled agent",
            closeout_path.display()
        ))
    })?;

    let baseline = wrapper_only_baseline(workspace_root, entry, &detected_release.target_version)
        .map_err(|message| {
        MaintenanceCloseoutError::Validation(format!(
            "{}: live wrapper-only re-derivation failed for `{agent_id}`: {message}",
            closeout_path.display()
        ))
    })?;
    let Some((live_report_ref, live_surfaces)) = baseline else {
        return reject_unbound(
            closeout_path,
            closeout,
            &format!(
                "`{agent_id}` has no coverage report for {}",
                detected_release.target_version
            ),
        );
    };

    let adjudication_owed =
        !live_surfaces.is_empty() || !closeout.wrapper_only_dispositions.is_empty();
    check_baseline_ref(closeout_path, closeout, &live_report_ref, adjudication_owed)?;
    check_dispositions_cover_live(closeout_path, closeout, &live_surfaces)?;
    check_unmatched_debt_is_empty(workspace_root, closeout_path, entry, detected_release)
}

fn reject_unbound(
    closeout_path: &Path,
    closeout: &MaintenanceCloseout,
    why: &str,
) -> Result<(), MaintenanceCloseoutError> {
    if !closeout.wrapper_only_dispositions.is_empty()
        || closeout.wrapper_only_baseline_ref.is_some()
    {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `wrapper_only_dispositions` and `wrapper_only_baseline_ref` must be absent because {why}",
            closeout_path.display()
        )));
    }
    Ok(())
}

/// The baseline must be the report the audit would select today. A packet that adjudicated
/// `coverage.all.json` while `coverage.any.json` exists judged a narrower set than the one that
/// governs, and `select_report_path` is the single authority on which that is.
///
/// The field is owed only when there is something to bind: a packet whose agent has no wrapper-only
/// row and records no disposition has nothing to point at, and the emptiness is re-derived here
/// rather than taken from the artifact, so demanding the ref anyway would buy nothing.
fn check_baseline_ref(
    closeout_path: &Path,
    closeout: &MaintenanceCloseout,
    live_report_ref: &str,
    adjudication_owed: bool,
) -> Result<(), MaintenanceCloseoutError> {
    match closeout.wrapper_only_baseline_ref.as_deref() {
        Some(recorded) if recorded == live_report_ref => Ok(()),
        Some(recorded) => Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `wrapper_only_baseline_ref` is `{recorded}` but the support audit selects `{live_report_ref}`",
            closeout_path.display()
        ))),
        None if adjudication_owed => Err(MaintenanceCloseoutError::Validation(format!(
            "{}: missing required field `wrapper_only_baseline_ref` (expected `{live_report_ref}`)",
            closeout_path.display()
        ))),
        None => Ok(()),
    }
}

fn check_dispositions_cover_live(
    closeout_path: &Path,
    closeout: &MaintenanceCloseout,
    live_surfaces: &BTreeSet<SurfaceIdentity>,
) -> Result<(), MaintenanceCloseoutError> {
    let mut by_surface = BTreeMap::<&SurfaceIdentity, &WrapperOnlyDisposition>::new();
    for disposition in &closeout.wrapper_only_dispositions {
        if by_surface
            .insert(&disposition.surface, disposition)
            .is_some()
        {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "{}: `wrapper_only_dispositions` carries more than one entry for {}",
                closeout_path.display(),
                disposition.surface.describe()
            )));
        }
    }

    for surface in live_surfaces {
        if !by_surface.contains_key(surface) {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "{}: live wrapper-only row {} has no entry in `wrapper_only_dispositions`",
                closeout_path.display(),
                surface.describe()
            )));
        }
    }

    for (surface, disposition) in &by_surface {
        let still_claimed = live_surfaces.contains(*surface);
        match (disposition.category, still_claimed) {
            // Sorted obsolete and still claimed: the contraction this category owes did not happen.
            (WrapperOnlyCategory::Obsolete, true) => {
                return Err(MaintenanceCloseoutError::Validation(format!(
                    "{}: {} is sorted `obsolete` but still appears in the live wrapper-only report; \
                     the publication claim must be contracted in the same run",
                    closeout_path.display(),
                    surface.describe()
                )));
            }
            // Any other category naming a surface the report no longer lists is adjudicating
            // something that is not there. Only `obsolete` explains a row's absence.
            (category, false) if category != WrapperOnlyCategory::Obsolete => {
                return Err(MaintenanceCloseoutError::Validation(format!(
                    "{}: {} is sorted `{}` but is not a live wrapper-only row",
                    closeout_path.display(),
                    surface.describe(),
                    category.as_id()
                )));
            }
            _ => {}
        }
    }

    Ok(())
}

/// Field invariant 6. Derived here rather than read from the request, because a frozen request that
/// records the same unmatched rows as the live audit reconciles `exact` and would otherwise close.
fn check_unmatched_debt_is_empty(
    workspace_root: &Path,
    closeout_path: &Path,
    entry: &crate::agent_registry::AgentRegistryEntry,
    detected_release: &request::DetectedRelease,
) -> Result<(), MaintenanceCloseoutError> {
    let audit = derive_support_surface_audit(workspace_root, entry, detected_release).map_err(
        |message| {
            MaintenanceCloseoutError::Validation(format!(
                "{}: live support-surface audit failed for `{}`: {message}",
                closeout_path.display(),
                entry.agent_id
            ))
        },
    )?;
    if audit.unmatched_debt_surface.is_empty() {
        return Ok(());
    }
    let rows = audit
        .unmatched_debt_surface
        .iter()
        .map(|row| {
            format!(
                "{} {} ({})",
                row.command_path, row.surface_id, row.observation
            )
        })
        .collect::<Vec<_>>();
    Err(MaintenanceCloseoutError::Validation(format!(
        "{}: live `unmatched_debt_surface` must be empty before a packet closes; {} row(s) remain: {}",
        closeout_path.display(),
        rows.len(),
        rows.join(", ")
    )))
}

fn invalid(closeout_path: &Path, field: &str, problem: &str) -> MaintenanceCloseoutError {
    MaintenanceCloseoutError::Validation(format!(
        "{}: `{field}` {problem}",
        closeout_path.display()
    ))
}
