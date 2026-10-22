//! Pure verification against the report engine; no regenerated report is written.
use super::{
    models::{UnionSnapshotV2, WrapperCoverageV1},
    report, rules, wrapper,
};
use crate::agent_maintenance::support_audit::{surface_identity, ReportRowShape, SurfaceIdentity};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

type Claims = BTreeMap<SurfaceIdentity, BTreeSet<String>>;

pub(crate) fn validate(
    agent: &str,
    rules_bytes: &[u8],
    union: &Value,
    wrapper_value: &Value,
    version: &str,
    reports: &BTreeMap<String, Value>,
) -> Result<Claims, String> {
    let rules: rules::RulesFile = serde_json::from_slice(rules_bytes).map_err(|e| e.to_string())?;
    let raw: Value = serde_json::from_slice(rules_bytes).map_err(|e| e.to_string())?;
    for (pointer, expected) in [
        (
            "/wrapper_coverage/scope_semantics/defaults/no_scope_means",
            "all_expected_targets",
        ),
        (
            "/wrapper_coverage/scope_semantics/platforms_expand_using",
            "union.platform_mapping",
        ),
        (
            "/wrapper_coverage/scope_semantics/scope_set_resolution/mode",
            "union",
        ),
    ] {
        if raw.pointer(pointer).and_then(Value::as_str) != Some(expected) {
            return Err(format!("unsupported intake rule: {pointer}"));
        }
    }
    if raw
        .pointer("/wrapper_coverage/validation/disallow_overlapping_scopes")
        .and_then(Value::as_bool)
        != Some(true)
        || raw
            .pointer("/wrapper_coverage/scope_semantics/platforms_expand_to_expected_targets")
            .and_then(Value::as_bool)
            != Some(true)
    {
        return Err("unsupported wrapper scope/overlap rules".into());
    }
    rules::assert_supported_rules(&rules).map_err(|e| e.to_string())?;
    if rules.rules_schema_version != 1
        || rules
            .parity_exclusions
            .as_ref()
            .is_some_and(|p| p.schema_version != 1)
    {
        return Err("unsupported report rules schema".into());
    }
    let union: UnionSnapshotV2 =
        serde_json::from_value(union.clone()).map_err(|e| e.to_string())?;
    let wrapper: WrapperCoverageV1 =
        serde_json::from_value(wrapper_value.clone()).map_err(|e| e.to_string())?;
    for command in &wrapper.coverage {
        for scope in std::iter::once(command.scope.as_ref())
            .chain(command.flags.iter().map(|f| f.scope.as_ref()))
            .chain(command.args.iter().map(|a| a.scope.as_ref()))
            .flatten()
        {
            if scope
                .target_triples
                .iter()
                .flatten()
                .any(|t| !rules.union.expected_targets.contains(t))
                || scope
                    .platforms
                    .iter()
                    .flatten()
                    .any(|p| !rules.union.platform_mapping.values().any(|v| v == p))
            {
                return Err("wrapper scope names an unknown target/platform".into());
            }
        }
    }
    let targets: Vec<_> = union
        .inputs
        .iter()
        .map(|i| i.target_triple.clone())
        .collect();
    let mut filters = vec![("coverage.any.json".to_owned(), "any", None)];
    for target in &targets {
        filters.push((
            format!("coverage.{target}.json"),
            "exact_target",
            Some(target.as_str()),
        ));
    }
    if union.complete {
        filters.push(("coverage.all.json".into(), "all", None));
    }
    let expected: BTreeSet<_> = filters.iter().map(|(file, _, _)| file).collect();
    if reports.keys().collect::<BTreeSet<_>>() != expected {
        return Err("partial or unexpected coverage reports".into());
    }
    let exclusions = rules
        .parity_exclusions
        .as_ref()
        .map(report::build_parity_exclusions_index);
    let upstream = report::index_upstream(&union);
    let index = wrapper::index_wrapper(
        &rules.union.expected_targets,
        &rules.union.platform_mapping,
        &wrapper,
    );
    for entries in index
        .commands
        .values()
        .chain(index.flags.values())
        .chain(index.args.values())
    {
        let mut seen = BTreeSet::new();
        for entry in entries {
            if entry.targets.is_empty() || entry.targets.iter().any(|t| !seen.insert(t)) {
                return Err("wrapper scopes are empty or overlap".into());
            }
        }
    }
    for (file, mode, target) in filters {
        let actual = &reports[&file];
        let (filter, report_targets) = match target {
            Some(t) => (wrapper::FilterMode::ExactTarget(t), vec![t.to_owned()]),
            None if mode == "all" => (wrapper::FilterMode::All, union.expected_targets.clone()),
            None => (wrapper::FilterMode::Any, targets.clone()),
        };
        let derived = report::build_report(
            &rules,
            exclusions.as_ref(),
            version,
            mode,
            target,
            filter,
            &report_targets,
            &upstream,
            &wrapper,
            &index,
            actual["generated_at"]
                .as_str()
                .ok_or("missing report generated_at")?,
        )
        .map_err(|e| e.to_string())?;
        if *actual != serde_json::to_value(derived).map_err(|e| e.to_string())? {
            return Err(format!(
                "report differs from acquisition and wrapper claims: {file}"
            ));
        }
    }
    Ok(claims(agent, &index))
}

// The engine indexes every level, including passthrough/unsupported and unobserved claims.
fn claims(agent: &str, index: &wrapper::WrapperIndex) -> Claims {
    let mut out = Claims::new();
    for (path, entries) in &index.commands {
        insert(&mut out, agent, path, ReportRowShape::Command, "", entries);
    }
    for ((path, key), entries) in &index.flags {
        insert(&mut out, agent, path, ReportRowShape::Flag, key, entries);
    }
    for ((path, name), entries) in &index.args {
        insert(&mut out, agent, path, ReportRowShape::Arg, name, entries);
    }
    out
}

fn insert(
    out: &mut Claims,
    agent: &str,
    path: &[String],
    shape: ReportRowShape,
    leaf: &str,
    entries: &[wrapper::ScopedCoverage],
) {
    let targets = entries
        .iter()
        .flat_map(|e| e.targets.iter().cloned())
        .collect();
    out.insert(surface_identity(agent, path, shape, leaf), targets);
}
