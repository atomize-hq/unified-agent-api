//! Preparatory policy resolution and read-only maintenance freeze planning. No command calls this
//! module yet.
//!
//! Inputs describe one candidate; callers remain responsible for acquiring that view. A resolved
//! policy is neither a freeze nor a verdict about classification, evidence, grants, route authority
//! or acceptance. Production registry loading deliberately continues to refuse depth declarations.

mod acquisition;
pub mod debt;
mod freeze;
use crate::git_inputs;
pub use freeze::{plan_freeze, plan_freeze_with_work, FreezePlan, FreezeRequest};

use std::{collections::BTreeSet, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    agent_maintenance::support_audit::{
        authorization::parse_debt_inventory, NON_TUI_SUPPORT_DEBT_PATH,
    },
    agent_registry::{
        support_depth::{ObligationId, GOVERNING_RULES_REVISION},
        AgentRegistry,
    },
    depth_record::{debt_rows, part_identity, DepthRecord, PolicyPart, Selection, TemplateRule},
};

/// Resolve one exact version from complete registry/RULES/inventory contents and its optional
/// retained record. `None` is remainder; missing policy for a retained selection is an error.
/// `prior_record=None` means actually absent, never an unreadable or unsupported record.
/// The inventory contains every agent's rows; P retains every row of this agent, not just gaps.
/// Aliases, predicate interpretation and mode exclusions need the later acquisition intake and
/// are not certified here. Predicate selectors and EXCLUDED currently refuse explicitly.
pub fn resolve_policy(
    registry: &str,
    agent: &str,
    version: &str,
    rules: &str,
    prior_record: Option<&str>,
    debt_inventory: &str,
) -> Result<Option<PolicyPart>, String> {
    resolve(
        registry,
        agent,
        version,
        rules,
        prior_record,
        debt_inventory,
    )
    .map_err(|error| format!("support depth agent={agent} version={version}: {error}"))
}

fn resolve(
    registry: &str,
    agent: &str,
    version: &str,
    rules: &str,
    prior_record: Option<&str>,
    debt_inventory: &str,
) -> Result<Option<PolicyPart>, String> {
    let parsed = semver::Version::parse(version).map_err(|e| e.to_string())?;
    if !parsed.pre.is_empty() || !parsed.build.is_empty() || parsed.to_string() != version {
        return Err("version must be one exact MAJOR.MINOR.PATCH".into());
    }
    let registry = AgentRegistry::parse_unenforced(registry).map_err(|e| e.to_string())?;
    let entry = registry.find(agent).ok_or("unknown registry agent")?;
    let prior = prior_record
        .map(|text| DepthRecord::parse(text, agent, version))
        .transpose()?;
    let table = entry.support_depth.as_ref();
    let declaration = table
        .and_then(|table| table.enrollments.as_ref())
        .and_then(|declarations| declarations.iter().find(|d| d.version == version));
    let Some(declaration) = declaration else {
        if prior.is_some() {
            return Err("missing policy for retained depth record".into());
        }
        return Ok(None);
    };
    let rules: Rules = serde_json::from_str(rules).map_err(|e| e.to_string())?;
    if rules.rules_schema_version != 1 {
        return Err("unsupported RULES schema".into());
    }
    let expected_path = format!(
        "{}/reports/<version>/depth-record.json",
        entry.manifest_root
    );
    if rules.storage.committed.reports.depth_record != expected_path {
        return Err("RULES does not adopt the canonical depth-record path".into());
    }
    let targets = rules.union.expected_targets;
    let unique: BTreeSet<_> = targets.iter().collect();
    if targets.is_empty()
        || unique.len() != targets.len()
        || targets.iter().any(|t| {
            t.is_empty()
                || !t
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || ".-_".contains(c))
        })
    {
        return Err("RULES union.expected_targets must contain unique plain target names".into());
    }
    for target in &declaration.targets {
        let retained = prior.as_ref().is_some_and(|p| {
            p.policy.selection.enrollment.modes.iter().any(|m| {
                m.targets
                    .as_ref()
                    .unwrap_or(&p.policy.selection.enrollment.targets)
                    .contains(target)
            })
        });
        if !targets.contains(target) && !retained {
            return Err(format!(
                "target `{target}` has no RULES or retained-record authority"
            ));
        }
    }
    // Without canonical acquisition facts, a free mode label or a flag spelling cannot prove
    // disjointness. Refuse unsupported predicates/exclusions, and all overlapping command scopes.
    for (index, mode) in declaration.modes.iter().enumerate() {
        if !mode.predicates.is_empty() {
            return Err(format!(
                "selector `{}` requires predicate/alias acquisition intake",
                mode.selector
            ));
        }
        for other in &declaration.modes[..index] {
            if mode.command_path == other.command_path
                && mode
                    .targets
                    .as_ref()
                    .unwrap_or(&declaration.targets)
                    .iter()
                    .any(|t| {
                        other
                            .targets
                            .as_ref()
                            .unwrap_or(&declaration.targets)
                            .contains(t)
                    })
            {
                return Err(format!(
                    "selectors `{}` and `{}` have unresolved overlapping command scope",
                    mode.selector, other.selector
                ));
            }
        }
    }
    if declaration.operations.iter().any(|o| o.is_excluded()) {
        return Err("mode exclusion requires classification/parity-exclusion intake".into());
    }
    let mut rows = parse_debt_inventory(debt_inventory, Path::new(NON_TUI_SUPPORT_DEBT_PATH))?;
    rows.retain(|row| row.agent_id == agent);
    let mut ids = BTreeSet::new();
    for row_id in debt_rows("policy.debt_baseline", &rows, agent)? {
        if !ids.insert(row_id) {
            return Err("debt baseline repeats an id".into());
        }
    }
    let templates: BTreeSet<_> = declaration
        .operations
        .iter()
        .flat_map(|o| o.templates.iter())
        .collect();
    let mut policy = PolicyPart {
        identity: String::new(),
        selection: Selection {
            agent_id: agent.into(),
            enrollment: declaration.clone(),
        },
        rules_revision: GOVERNING_RULES_REVISION.into(),
        base: canonical_set(ObligationId::BASE.to_vec())?,
        templates: canonical_set(
            templates
                .into_iter()
                .map(|template| {
                    Ok(TemplateRule {
                        template: *template,
                        obligations: canonical_set(template.obligations().to_vec())?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        )?,
        path_enabled: table
            .and_then(|t| t.enabled_paths.as_ref())
            .is_some_and(|paths| paths.contains(&declaration.lifecycle_path)),
        debt_baseline: canonical_set(rows)?,
    };
    policy.identity = part_identity(&policy)?;
    if let Some(mut previous) = prior {
        if previous.policy.selection.enrollment.lifecycle_path != declaration.lifecycle_path {
            return Err("retained enrollment cannot move to another lifecycle path".into());
        }
        let earlier = previous.clone();
        previous.policy = policy.clone();
        let missing = previous.lacking_from(&earlier);
        if !missing.is_empty() {
            return Err(format!(
                "policy loses retained coverage: {}",
                missing.join(", ")
            ));
        }
    }
    Ok(Some(policy))
}

// Typed projections retain duplicate-field/type rejection without copying unrelated RULES policy.
#[derive(Deserialize)]
struct Rules {
    rules_schema_version: u64,
    union: Targets,
    storage: Storage,
}
#[derive(Deserialize)]
struct Targets {
    expected_targets: Vec<String>,
}
#[derive(Deserialize)]
struct Storage {
    committed: Committed,
}
#[derive(Deserialize)]
struct Committed {
    reports: Reports,
}
#[derive(Deserialize)]
struct Reports {
    depth_record: String,
}

/// Compare a frozen policy with a freshly resolved one. This only decides P currentness;
/// it cannot disposition debt transitions or establish O/E/acceptance authority.
pub fn check_current_policy(frozen: &PolicyPart, resolved: &PolicyPart) -> Result<(), String> {
    for policy in [frozen, resolved] {
        if policy.rules_revision != GOVERNING_RULES_REVISION {
            return Err("policy uses an unsupported governing rules revision".into());
        }
        if policy.identity != part_identity(policy)? {
            return Err("policy identity does not match its contents".into());
        }
    }
    if frozen != resolved {
        return Err("frozen policy is no longer current; a new generation is required".into());
    }
    Ok(())
}

fn canonical_set<T: Serialize>(items: Vec<T>) -> Result<Vec<T>, String> {
    let mut keyed = items
        .into_iter()
        .map(|item| {
            let mut value = serde_json::to_value(&item).map_err(|e| e.to_string())?;
            value.sort_all_objects();
            let key = value.to_string();
            Ok((key, item))
        })
        .collect::<Result<Vec<_>, String>>()?;
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(keyed.into_iter().map(|(_, item)| item).collect())
}
