//! Check 9: semantic acceptance detection and current record-wide entry matching.
mod artifacts;
mod input;
mod manifest;
mod publication;
mod relay;
mod runtime;
use crate::runtime_advertising::source::Source;
use input::View;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Effect {
    pub version: String,
    pub target: Option<String>,
    pub path: String,
    pub kind: String,
}
#[derive(Debug)]
pub struct MissingEntry {
    pub version: String,
    pub digest: String,
    pub effects: Vec<Effect>,
}

/// Semantic effects remain observable even if ordinary freshness/qualification validation also fails.
#[derive(Debug, Default)]
pub struct Comparison {
    pub effects: Vec<Effect>,
    pub missing_entries: Vec<MissingEntry>,
    pub input_issues: Vec<String>,
}

pub fn compare(workspace: &Path, commit: &str, agent: &str) -> Result<Comparison, String> {
    let before = View::load(Source::tip(workspace, commit)?, agent)?;
    let after = View::load(Source::workspace(workspace)?, agent)?;
    let mut comparison = compare_views(&before, &after)?;
    if !before.records.is_empty() || !after.records.is_empty() {
        comparison.input_issues = publication::validate_current(workspace, &after)
            .err()
            .into_iter()
            .collect();
    }
    Ok(comparison)
}

/// Read-only preparation adapter. Supplied registry/proposed bytes confer no write admission.
/// Rejects effects themselves, including effects already carrying an acceptance entry.
pub fn check_reporting_preparation(
    workspace: &Path,
    registry: &crate::agent_registry::AgentRegistry,
    proposed: BTreeMap<String, String>,
) -> Result<(), String> {
    for entry in &registry.agents {
        let before = View::with_registry(Source::workspace(workspace)?, registry, &entry.agent_id)?;
        let after = View::with_registry(
            Source::proposed(workspace, proposed.clone())?,
            registry,
            &entry.agent_id,
        )?;
        let comparison = compare_views(&before, &after)?;
        if !comparison.effects.is_empty() {
            return Err(format!(
                "reporting-only preparation agent={} would create acceptance effects: {:?}",
                entry.agent_id, comparison.effects
            ));
        }
    }
    Ok(())
}

fn compare_views(before: &View, after: &View) -> Result<Comparison, String> {
    let agent = &after.entry.agent_id;
    if before.records.is_empty() && after.records.is_empty() {
        return Ok(Comparison::default());
    }
    if before.entry.manifest_root != after.entry.manifest_root {
        return Err("record root was relocated".into());
    }
    for version in before.records.keys() {
        if !after.records.contains_key(version) {
            return Err(format!("candidate record missing for {agent}/{version}"));
        }
    }
    let mut effects = BTreeSet::new();
    manifest::detect(before, after, &mut effects)?;
    artifacts::detect(before, after, &mut effects)?;
    publication::detect(before, after, &mut effects)?;
    let mut by_version: BTreeMap<String, Vec<Effect>> = BTreeMap::new();
    let effects: Vec<_> = effects
        .into_iter()
        .filter(|e| after.covered(&e.version, e.target.as_deref()))
        .collect();
    for effect in &effects {
        {
            by_version
                .entry(effect.version.clone())
                .or_default()
                .push(effect.clone());
        }
    }
    let mut missing = Vec::new();
    for (version, effects) in by_version {
        let record = after
            .records
            .get(&version)
            .ok_or("acceptance destination has no record")?;
        let digest = record.stated_sha256()?;
        if !record
            .acceptance_entries
            .iter()
            .any(|e| e.record_sha256() == digest)
        {
            missing.push(MissingEntry {
                version,
                digest,
                effects,
            });
        }
    }
    Ok(Comparison {
        effects,
        missing_entries: missing,
        input_issues: Vec::new(),
    })
}
fn effect(out: &mut BTreeSet<Effect>, version: &str, target: Option<&str>, path: &str, kind: &str) {
    out.insert(Effect {
        version: version.into(),
        target: target.map(str::to_owned),
        path: path.into(),
        kind: kind.into(),
    });
}

/// Read-only relay comparison; success supplies no write or integration authority.
pub fn check_reporting_relay(workspace: &Path, baseline: &str) -> Result<(), String> {
    relay::check(
        Source::tip(workspace, baseline)?,
        Source::workspace(workspace)?,
        workspace,
    )
}

/// Host-memory relay views; neither side is read from executor-writable Git objects.
pub(crate) fn check_captured_relay(
    before: Source,
    after: Source,
    workspace: &Path,
) -> Result<(), String> {
    relay::check(before, after, workspace)
}
