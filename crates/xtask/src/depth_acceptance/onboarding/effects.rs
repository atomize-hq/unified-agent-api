//! Aggregate mediation includes destinations with no retained record or covered selection.
use super::*;
use std::collections::BTreeSet;

pub(super) fn check(
    before: &Source,
    after: &Source,
    owner: &View,
    record: &DepthRecord,
    kind: Kind,
) -> Result<(), String> {
    let old_registry = crate::agent_registry::AgentRegistry::parse_unenforced(
        &before.read(crate::agent_registry::REGISTRY_RELATIVE_PATH)?,
    )
    .map_err(|e| e.to_string())?;
    let registry = &owner.registry;
    if registry != &old_registry {
        return Err("onboarding cannot change registry authority".into());
    }
    let version = &record.policy.selection.enrollment.version;
    let packet = agent_lifecycle::publication_ready_path_for_entry(&owner.entry);
    let closeout = format!(
        "docs/agents/lifecycle/{}/governance/proving-run-closeout.json",
        owner.entry.scaffold.onboarding_pack_prefix
    );
    for entry in &registry.agents {
        let old = View::with_registry(before.clone(), registry, &entry.agent_id)?;
        let new = View::with_registry(after.clone(), registry, &entry.agent_id)?;
        let mut effects = BTreeSet::new();
        // Use the raw detectors, before coverage filtering can hide an outside destination.
        let rules = format!("{}/RULES.json", entry.manifest_root);
        if !old.records.is_empty()
            || !new.records.is_empty()
            || before.optional(&rules)?.is_some()
            || after.optional(&rules)?.is_some()
        {
            super::super::manifest::detect(&old, &new, &mut effects)?;
        }
        let prefix = format!("{}/versions/", entry.manifest_root);
        let mut versions = BTreeSet::new();
        for path in before
            .paths(prefix.trim_end_matches('/'))?
            .into_iter()
            .chain(after.paths(prefix.trim_end_matches('/'))?)
        {
            if let Some(version) = path
                .strip_prefix(&prefix)
                .and_then(|p| p.strip_suffix(".json"))
                .filter(|v| !v.contains('/'))
            {
                versions.insert(version.to_owned());
            }
        }
        super::super::manifest::detect_versions(&old, &new, &mut effects, &versions)?;
        artifacts::detect(&old, &new, &mut effects)?;
        super::super::publication::detect(&old, &new, &mut effects)?;
        for effect in effects {
            if entry.agent_id != owner.entry.agent_id || effect.version != *version {
                return Err("onboarding batch has acceptance outside its owned generation".into());
            }
            if effect.target.as_ref().is_some_and(|target| {
                !crate::support_matrix::depth::covered_targets(record).contains(&target.as_str())
            }) {
                return Err("onboarding batch has acceptance outside its owned target".into());
            }
            let allowed = match kind {
                Kind::Publication => {
                    (effect.kind == "published" && effect.path == packet)
                        || super::super::publication::is_qualification(&effect)
                        || (effect.path == crate::capability_matrix::DEFAULT_OUT_PATH
                            && effect.kind.starts_with("published capability "))
                        || (effect.path == "crates/agent_api/src/runtime_advertising/data.rs"
                            && effect.kind.starts_with("runtime advertising "))
                }

                Kind::Proving => {
                    [
                        "closed_baseline",
                        "proving_run_closeout_written",
                        "closeout semantic change",
                    ]
                    .contains(&effect.kind.as_str())
                        && effect.path == closeout
                }
            };
            if !allowed {
                return Err(format!(
                    "onboarding {:?} forbids acceptance effect {}",
                    kind, effect.kind
                ));
            }
        }
        // Pointer clearing and min_supported changes carry no positive detector effect,
        // but onboarding has no pointer ownership in either direction.
        let pointer_root = format!("{}/pointers", entry.manifest_root);
        let mut pointers = BTreeSet::from([
            format!("{}/min_supported.txt", entry.manifest_root),
            format!("{}/latest_validated.txt", entry.manifest_root),
        ]);
        pointers.extend(
            before
                .paths(&pointer_root)?
                .into_iter()
                .filter(|path| path.ends_with(".txt")),
        );
        pointers.extend(
            after
                .paths(&pointer_root)?
                .into_iter()
                .filter(|path| path.ends_with(".txt")),
        );
        for path in pointers {
            if before.optional_bytes(&path)? != after.optional_bytes(&path)? {
                return Err(format!("onboarding cannot change pointer file {path}"));
            }
        }
        let markers = format!(
            "docs/agents/lifecycle/{}-maintenance/governance/automation-stand-down",
            entry.agent_id
        );
        for path in before
            .paths(&markers)?
            .into_iter()
            .filter(|path| path.ends_with(".toml"))
        {
            if before.optional_bytes(&path)?.is_some() && after.optional_bytes(&path)?.is_none() {
                return Err("onboarding cannot retire maintenance markers".into());
            }
        }
        for (retained_version, prior) in &old.records {
            if !new.records.contains_key(retained_version) {
                return Err("onboarding batch removed a retained record".into());
            }
            if (entry.agent_id != owner.entry.agent_id || retained_version != version)
                && new.records[retained_version] != *prior
            {
                return Err(
                    "onboarding batch changed a record outside its owned generation".into(),
                );
            }
        }
        for (v, candidate) in &new.records {
            if (entry.agent_id != owner.entry.agent_id || v != version)
                && old.records.get(v) != Some(candidate)
            {
                return Err(
                    "onboarding batch changed a record outside its owned generation".into(),
                );
            }
            let prior = old
                .records
                .get(v)
                .map(|r| r.acceptance_entries.as_slice())
                .unwrap_or_default();
            if !candidate.acceptance_entries.starts_with(prior) {
                return Err("onboarding batch changed retained entries".into());
            }
            for added in &candidate.acceptance_entries[prior.len()..] {
                if entry.agent_id != owner.entry.agent_id || v != version {
                    return Err(
                        "onboarding batch has acceptance outside its owned generation".into(),
                    );
                }
                if *added != entry_reference(owner, before, record, kind)? {
                    return Err("onboarding batch has an entry outside its owning path".into());
                }
            }
        }
    }
    Ok(())
}
