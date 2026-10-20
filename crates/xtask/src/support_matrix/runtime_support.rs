//! The runtime support projection: the `latest_validated` version of each expected target, as
//! `crates/agent_api/src/runtime_support_data.rs` embeds it for the families that embed one.

use std::{collections::BTreeSet, fmt::Write as _};

use agent_api::RuntimeSupportRecord;

use super::derive::LoadedAgentRoot;

pub(super) const EMBEDDED_RUNTIME_SUPPORT_FAMILIES: &[&str] = &["codex"];

pub(super) fn derive_validated_runtime_support_for_loaded_roots(
    roots: &[LoadedAgentRoot],
) -> Result<Vec<RuntimeSupportRecord>, String> {
    let mut records = derive_runtime_support_projection_for_loaded_roots(roots)?
        .into_iter()
        .filter_map(|record| {
            record.latest_validated.map(|version| RuntimeSupportRecord {
                runtime_family: record.runtime_family,
                target_triple: record.target_triple,
                version,
            })
        })
        .collect::<Vec<_>>();

    records.sort_by(|left, right| {
        left.runtime_family
            .cmp(&right.runtime_family)
            .then(left.target_triple.cmp(&right.target_triple))
            .then(left.version.cmp(&right.version))
    });
    Ok(records)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RuntimeSupportProjectionRecord {
    runtime_family: String,
    target_triple: String,
    latest_validated: Option<String>,
}

pub(super) fn derive_runtime_support_projection_for_loaded_roots(
    roots: &[LoadedAgentRoot],
) -> Result<Vec<RuntimeSupportProjectionRecord>, String> {
    let mut records = Vec::new();
    for root in roots {
        records.extend(derive_runtime_support_projection_for_loaded_root(root)?);
    }

    records.sort_by(|left, right| {
        left.runtime_family
            .cmp(&right.runtime_family)
            .then(left.target_triple.cmp(&right.target_triple))
            .then_with(|| left.latest_validated.cmp(&right.latest_validated))
    });
    Ok(records)
}

fn derive_runtime_support_projection_for_loaded_root(
    root: &LoadedAgentRoot,
) -> Result<Vec<RuntimeSupportProjectionRecord>, String> {
    let known_versions = root
        .versions
        .iter()
        .map(|metadata| metadata.semantic_version.as_str())
        .collect::<BTreeSet<_>>();
    let mut records = Vec::new();

    for target in &root.posture.expected_targets {
        let latest_validated = match root
            .pointers
            .latest_validated
            .get(target)
            .and_then(|value| value.as_deref())
        {
            Some(version) => {
                if !known_versions.contains(version) {
                    return Err(format!(
                        "agent `{}` target `{}` latest_validated pointer references unknown version `{version}`",
                        root.agent, target
                    ));
                }
                Some(version.to_string())
            }
            None => None,
        };

        records.push(RuntimeSupportProjectionRecord {
            runtime_family: root.agent.clone(),
            target_triple: target.clone(),
            latest_validated,
        });
    }

    Ok(records)
}

pub(super) fn render_agent_api_runtime_support_data_from_projection(
    projection: &[RuntimeSupportProjectionRecord],
    runtime_families: &[&str],
) -> String {
    let mut rendered = String::from(
        "// This file is derived from committed repo truth.\n\
         // Validate it with `cargo test -p xtask --all-targets`.\n\n",
    );

    for runtime_family in runtime_families {
        let symbol = runtime_family_symbol(runtime_family);
        let _ = writeln!(
            rendered,
            "const {symbol}: &[EmbeddedRuntimeSupportRecord] = &["
        );

        for record in projection
            .iter()
            .filter(|record| record.runtime_family == *runtime_family)
        {
            let latest_validated = match &record.latest_validated {
                Some(version) => format!("Some(\"{version}\")"),
                None => "None".to_string(),
            };
            let _ = writeln!(
                rendered,
                "    EmbeddedRuntimeSupportRecord {{\n        target_triple: \"{}\",\n        latest_validated: {},\n    }},",
                record.target_triple, latest_validated
            );
        }

        rendered.push_str("];\n");
    }

    rendered
}

fn runtime_family_symbol(runtime_family: &str) -> String {
    format!(
        "{}_RUNTIME_SUPPORT",
        runtime_family
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect::<String>()
    )
}
