//! Check 9: semantic acceptance detection and current record-wide entry matching.
mod artifacts;
mod input;
mod manifest;
mod publication;
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
    manifest::detect(&before, &after, &mut effects)?;
    artifacts::detect(&before, &after, &mut effects)?;
    publication::detect(workspace, commit, &before, &after, &mut effects)?;
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
    let input_issues = publication::validate_current(workspace, &after)
        .err()
        .into_iter()
        .collect();
    Ok(Comparison {
        effects,
        missing_entries: missing,
        input_issues,
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
