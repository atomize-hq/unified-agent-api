#[cfg(test)]
#[path = "../tests/support/candidate_registry.rs"]
#[allow(clippy::duplicate_mod)]
mod candidate_registry;
#[cfg(test)]
#[path = "../tests/support/registry_fixture.rs"]
#[allow(clippy::duplicate_mod)]
mod registry_fixture;

use std::{fs, path::Path};

use crate::agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH};

pub const RELEASE_DOC_PATH: &str = "docs/crates-io-release.md";
pub const RELEASE_DOC_START_MARKER: &str =
    "<!-- generated-by: xtask onboard-agent; section: crates-io-release -->";
pub const RELEASE_DOC_END_MARKER: &str =
    "<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->";

const CRATES_IO_RELEASE_TRACK: &str = "crates-io";
const WRAPPER_EVENTS_PACKAGE_NAME: &str = "unified-agent-api-wrapper-events";
const ROOT_AGENT_API_PACKAGE_NAME: &str = "unified-agent-api";

pub fn render_release_doc(workspace_root: &Path) -> Result<String, String> {
    let existing = fs::read_to_string(workspace_root.join(RELEASE_DOC_PATH)).map_err(|err| {
        format!(
            "read {}: {err}",
            workspace_root.join(RELEASE_DOC_PATH).display()
        )
    })?;
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| format!("load {REGISTRY_RELATIVE_PATH}: {err}"))?;
    let packages = registry_release_packages(&registry, CRATES_IO_RELEASE_TRACK);
    let block = render_release_doc_block(&packages);
    Ok(splice_release_doc_block(&existing, &block))
}

pub fn registry_release_packages(registry: &AgentRegistry, release_track: &str) -> Vec<String> {
    let mut packages = registry
        .agents
        .iter()
        .filter(|entry| entry.release.docs_release_track == release_track)
        .map(|entry| entry.package_name.clone())
        .collect::<Vec<_>>();
    packages.push(WRAPPER_EVENTS_PACKAGE_NAME.to_string());
    packages.push(ROOT_AGENT_API_PACKAGE_NAME.to_string());
    packages
}

pub fn render_release_doc_block(packages: &[String]) -> String {
    let list = packages
        .iter()
        .map(|package| format!("- `{package}`"))
        .collect::<Vec<_>>()
        .join("\n");
    let order = packages
        .iter()
        .enumerate()
        .map(|(index, package)| format!("{}. `{package}`", index + 1))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "{RELEASE_DOC_START_MARKER}\n## Published crates\n\nThis repository publishes {} Rust packages for each root `VERSION` bump:\n\n{list}\n\n## Publish order\n\nAlways publish in this order:\n\n{order}\n{RELEASE_DOC_END_MARKER}",
        packages.len()
    )
}

pub fn splice_release_doc_block(existing: &str, block: &str) -> String {
    match (
        existing.find(RELEASE_DOC_START_MARKER),
        existing.find(RELEASE_DOC_END_MARKER),
    ) {
        (Some(start), Some(end)) if start <= end => {
            let before = &existing[..start];
            let after = &existing[end + RELEASE_DOC_END_MARKER.len()..];
            format!("{before}{block}{after}")
        }
        _ => {
            let trimmed = existing.trim_end();
            format!("{trimmed}\n\n{block}\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_registry::AgentRegistry;

    #[test]
    fn splice_release_doc_replaces_existing_generated_block() {
        let existing = "# Guide\n\nbefore\n\n<!-- generated-by: xtask onboard-agent; section: crates-io-release -->\nold\n<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->\n\nafter\n";
        let block = "<!-- generated-by: xtask onboard-agent; section: crates-io-release -->\nnew\n<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->";

        let updated = splice_release_doc_block(existing, block);

        assert!(updated.contains("before"));
        assert!(updated.contains("after"));
        assert!(updated.contains("\nnew\n"));
        assert!(!updated.contains("\nold\n"));
    }

    #[test]
    fn registry_release_packages_follow_registry_order_and_append_tail_packages() {
        let registry =
            AgentRegistry::parse(registry_fixture::undeclared_registry()).expect("parse registry");

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let candidate = candidate_registry::registry(&root);
        let mut expected: Vec<String> = candidate
            .agents
            .iter()
            .filter(|entry| entry.release.docs_release_track == "crates-io")
            .map(|entry| entry.package_name.clone())
            .collect();
        expected.extend([
            WRAPPER_EVENTS_PACKAGE_NAME.to_string(),
            ROOT_AGENT_API_PACKAGE_NAME.to_string(),
        ]);
        assert_eq!(registry_release_packages(&candidate, "crates-io"), expected);
        let committed = fs::read_to_string(root.join(RELEASE_DOC_PATH)).unwrap();
        assert!(committed.contains(&render_release_doc_block(&expected)));
        assert_eq!(candidate_registry::registry(&root), candidate);

        assert_eq!(
            registry_release_packages(&registry, "crates-io"),
            vec![
                "unified-agent-api-codex".to_string(),
                "unified-agent-api-claude-code".to_string(),
                "unified-agent-api-opencode".to_string(),
                "unified-agent-api-gemini-cli".to_string(),
                "unified-agent-api-aider".to_string(),
                "unified-agent-api-wrapper-events".to_string(),
                "unified-agent-api".to_string(),
            ]
        );
    }
}
