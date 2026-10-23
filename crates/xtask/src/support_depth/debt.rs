//! Conditional, read-only debt planning. Reports are claims, not proof of a continuing blocker,
//! semantic coverage/exclusion or upstream removal. No returned value is writer/publisher authority.
pub use crate::agent_maintenance::support_audit::authorization::DebtInventoryRow;
use crate::{
    agent_maintenance::support_audit::{
        authorization::{
            parse_debt_inventory, validate_authorization_evidence_with, validate_row_set,
        },
        surfaces_from_report_deltas, NON_TUI_SUPPORT_DEBT_PATH,
    },
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::{part_identity, DepthRecord, Event, FileRef},
    git_inputs::GitInputs,
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Proof obligations for the later owning route. These are intentionally unresolved: neither
/// caller-supplied claims nor content hashes can establish the truth of a semantic assessment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebtRequirement {
    Renewal {
        row_id: String,
        frozen_row_sha256: String,
        report: FileRef,
        target_reports: Vec<FileRef>,
    },
    Retirement {
        row_id: String,
        frozen_row_sha256: String,
        target: String,
        report: FileRef,
    },
}

#[derive(Debug, Clone)]
pub struct DebtPlan {
    /// Identity of the candidate being inspected, not an authorization receipt.
    pub record_sha256: String,
    /// Complete materialized rows of this agent, including unselected versions and targets.
    pub rows: Vec<DebtInventoryRow>,
    /// Corroborate the reports/currentness and the full frozen row. Renewal additionally owes
    /// unchanged-blocker/owner/milestone/follow-on proof; each retirement target owes live
    /// coverage/exclusion or explicit upstream-removal proof. Help absence never discharges it.
    pub requirements: Vec<DebtRequirement>,
}
impl DebtPlan {
    pub fn requires_corroboration(&self) -> bool {
        !self.requirements.is_empty()
    }
}

/// Reconcile a proposed E/inventory pair without changing either. `reports` supplies candidate
/// coverage JSON by repository-relative path; this pure entry does not authenticate those bytes.
/// Success is a structurally permissible conditional candidate, never verified debt/admission.
pub fn plan_debt_transition(
    record: &DepthRecord,
    inventory: &str,
    reports: &BTreeMap<String, String>,
) -> Result<DebtPlan, String> {
    let agent = &record.policy.selection.agent_id;
    let version = &record.policy.selection.enrollment.version;
    DepthRecord::parse(
        &serde_json::to_string(record).map_err(|e| e.to_string())?,
        agent,
        version,
    )?;
    if !matches!(record.event, Event::Maintenance { .. }) {
        return Err("debt planning requires the maintenance lifecycle".into());
    }
    if record
        .obligations
        .as_ref()
        .is_some_and(|o| o.policy_identity != record.policy.identity)
    {
        return Err("debt O is not bound to the stated P".into());
    }
    if let Some(e) = &record.execution {
        let o = record
            .obligations
            .as_ref()
            .ok_or("debt execution requires frozen O")?;
        if e.policy_identity != record.policy.identity || e.obligations_identity != o.identity {
            return Err("debt E is not bound to the stated P/O".into());
        }
    }
    let baseline = &record.policy.debt_baseline;
    validate_row_set(baseline)?;
    let all = parse_debt_inventory(inventory, Path::new(NON_TUI_SUPPORT_DEBT_PATH))?;
    let actual: Vec<_> = all.into_iter().filter(|r| r.agent_id == *agent).collect();
    validate_row_set(&actual)?;
    let mut rows: BTreeMap<_, _> = baseline
        .iter()
        .map(|r| (r.row_id.clone(), r.clone()))
        .collect();
    let mut requirements = Vec::new();
    if let Some(e) = &record.execution {
        validate_row_set(&e.debt_grants)?;
        let root = format!("cli_manifests/{agent}");
        for grant in &e.debt_grants {
            let old = rows
                .get(&grant.row_id)
                .ok_or("new debt requires maintainer P supersession/refreeze")?;
            let mut immutable = grant.clone();
            immutable.scope_target_triples = old.scope_target_triples.clone();
            immutable.authorized_at_version = old.authorized_at_version.clone();
            immutable.authorization_evidence_ref = old.authorization_evidence_ref.clone();
            if immutable != *old
                || !grant
                    .scope_target_triples
                    .is_subset(&old.scope_target_triples)
            {
                return Err(format!(
                    "debt row {} changes frozen identity/authority",
                    grant.row_id
                ));
            }
            validate_authorization_evidence_with(&root, grant, &|path| read_report(reports, path))?;
            let mut target_reports = Vec::new();
            for target in &grant.scope_target_triples {
                let (path, json) = target_report(reports, agent, version, target)?;
                if !has_gap(agent, &path, &json, grant)? {
                    return Err("renewed debt is not a current same-surface target gap".into());
                }
                // Exact-target rows may retain the union's broader availability list, but
                // the selected target must actually be among those same-surface observations.
                let mut per_target = grant.clone();
                per_target.scope_target_triples = BTreeSet::from([target.clone()]);
                per_target.authorization_evidence_ref = path.clone();
                validate_authorization_evidence_with(&root, &per_target, &|p| {
                    read_report(reports, p)
                })?;
                target_reports.push(reference(&path, reports)?);
            }
            requirements.push(DebtRequirement::Renewal {
                row_id: grant.row_id.clone(),
                frozen_row_sha256: row_sha(old)?,
                report: reference(&grant.authorization_evidence_ref, reports)?,
                target_reports,
            });
            rows.insert(grant.row_id.clone(), grant.clone());
        }
        for id in &e.debt_retired {
            let old = rows.remove(id).ok_or("retirement names no frozen row")?;
            for target in &old.scope_target_triples {
                let (path, json) = target_report(reports, agent, version, target)?;
                if has_gap(agent, &path, &json, &old)? {
                    return Err("cannot retire a row with an unresolved target gap".into());
                }
                requirements.push(DebtRequirement::Retirement {
                    row_id: id.clone(),
                    frozen_row_sha256: row_sha(&old)?,
                    target: target.clone(),
                    report: reference(&path, reports)?,
                });
            }
        }
    }
    let rows: Vec<_> = rows.into_values().collect();
    validate_row_set(&rows)?;
    let actual: BTreeMap<_, _> = actual.iter().map(|r| (&r.row_id, r)).collect();
    let expected: BTreeMap<_, _> = rows.iter().map(|r| (&r.row_id, r)).collect();
    if actual != expected {
        return Err(
            "candidate inventory differs from frozen baseline plus exact E transitions".into(),
        );
    }
    Ok(DebtPlan {
        record_sha256: record.stated_sha256()?,
        rows,
        requirements,
    })
}

/// Read candidate inputs through the existing coherent stage-0 reader. This remains a diagnostic
/// plan: semantic proof requirements are not discharged and production enrollment is not enabled.
pub fn evaluate_debt_workspace(
    workspace: &Path,
    agent: &str,
    version: &str,
) -> Result<DebtPlan, String> {
    let mut view = GitInputs::new(workspace)?;
    let registry = AgentRegistry::parse_unenforced(&text(view.required(REGISTRY_RELATIVE_PATH)?)?)
        .map_err(|e| e.to_string())?;
    let entry = registry.find(agent).ok_or("unknown debt agent")?;
    let raw = text(view.required(&format!(
        "{}/reports/{version}/depth-record.json",
        entry.manifest_root
    ))?)?;
    let record = DepthRecord::parse(&raw, agent, version)?;
    let inventory = text(view.required(NON_TUI_SUPPORT_DEBT_PATH)?)?;
    let mut paths = BTreeSet::new();
    if let Some(e) = &record.execution {
        for row in &e.debt_grants {
            paths.insert(row.authorization_evidence_ref.clone());
            for t in &row.scope_target_triples {
                paths.insert(format!(
                    "{}/reports/{version}/coverage.{t}.json",
                    entry.manifest_root
                ));
            }
        }
        for id in &e.debt_retired {
            let row = record
                .policy
                .debt_baseline
                .iter()
                .find(|r| r.row_id == *id)
                .ok_or("unknown retired row")?;
            for t in &row.scope_target_triples {
                paths.insert(format!(
                    "{}/reports/{version}/coverage.{t}.json",
                    entry.manifest_root
                ));
            }
        }
    }
    let mut reports = BTreeMap::new();
    for path in paths {
        reports.insert(path.clone(), text(view.required(&path)?)?);
    }
    let plan = plan_debt_transition(&record, &inventory, &reports)?;
    view.verify()?;
    Ok(plan)
}
fn text(bytes: Vec<u8>) -> Result<String, String> {
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
fn row_sha(row: &DebtInventoryRow) -> Result<String, String> {
    part_identity(row)
}

fn reference(path: &str, reports: &BTreeMap<String, String>) -> Result<FileRef, String> {
    let raw = reports
        .get(path)
        .ok_or_else(|| format!("missing current debt report: {path}"))?;
    Ok(FileRef {
        path: path.into(),
        sha256: hex::encode(Sha256::digest(raw.as_bytes())),
    })
}
fn read_report(
    reports: &BTreeMap<String, String>,
    path: &str,
) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        reports
            .get(path)
            .ok_or_else(|| format!("missing current debt report: {path}"))?,
    )
    .map_err(|e| e.to_string())
}
fn target_report(
    reports: &BTreeMap<String, String>,
    agent: &str,
    version: &str,
    target: &str,
) -> Result<(String, serde_json::Value), String> {
    let path = format!("cli_manifests/{agent}/reports/{version}/coverage.{target}.json");
    let json = read_report(reports, &path)?;
    if json
        .pointer("/inputs/upstream/semantic_version")
        .and_then(|v| v.as_str())
        != Some(version)
        || json
            .pointer("/platform_filter/mode")
            .and_then(|v| v.as_str())
            != Some("exact_target")
        || json
            .pointer("/platform_filter/target_triple")
            .and_then(|v| v.as_str())
            != Some(target)
        || json.pointer("/inputs/upstream/targets") != Some(&serde_json::json!([target]))
    {
        return Err(format!("debt report is not exact version/target: {path}"));
    }
    Ok((path, json))
}
fn has_gap(
    agent: &str,
    path: &str,
    json: &serde_json::Value,
    row: &DebtInventoryRow,
) -> Result<bool, String> {
    let deltas = json
        .get("deltas")
        .and_then(|d| d.as_object())
        .ok_or("debt report has no deltas")?;
    Ok(surfaces_from_report_deltas(agent, Path::new(path), deltas)?.contains(&row.identity()))
}
