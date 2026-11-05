//! Governing data that `onboard-agent` scaffolds in a new manifest root: `RULES.json`,
//! `SCHEMA.json`, `VERSION_METADATA_SCHEMA.json` and `VALIDATOR_SPEC.md`.
//!
//! The templates follow the shape of the committed roots (the synthetic subject's governing
//! templates in `crates/xtask/tests/fixtures/depth-synthetic/governing/`). Only agent identity,
//! the manifest root and the canonical targets vary. The scaffold declares no parity exclusions,
//! no acquisition block and no depth-record path, so the root does not adopt Annex B of the
//! support-depth contract and cannot hold a depth record.

use std::path::Path;

use super::super::DraftEntry;

const RULES_TEMPLATE: &str = include_str!("templates/RULES.json");
const SCHEMA_TEMPLATE: &str = include_str!("templates/SCHEMA.json");
const VERSION_METADATA_SCHEMA_TEMPLATE: &str =
    include_str!("templates/VERSION_METADATA_SCHEMA.json");
const VALIDATOR_SPEC_TEMPLATE: &str = include_str!("templates/VALIDATOR_SPEC.md");

/// The governing files, as `(repo-relative path, contents)` in a fixed order.
pub(super) fn governing_files(draft: &DraftEntry) -> Vec<(String, String)> {
    let values = TemplateValues::from_draft(draft);
    [
        ("RULES.json", RULES_TEMPLATE),
        ("SCHEMA.json", SCHEMA_TEMPLATE),
        ("VALIDATOR_SPEC.md", VALIDATOR_SPEC_TEMPLATE),
        (
            "VERSION_METADATA_SCHEMA.json",
            VERSION_METADATA_SCHEMA_TEMPLATE,
        ),
    ]
    .into_iter()
    .map(|(name, template)| {
        let path = Path::new(&draft.manifest_root)
            .join(name)
            .display()
            .to_string();
        let contents = if name.ends_with(".md") {
            values.render_markdown(template)
        } else {
            values.render_json(template)
        };
        (path, contents)
    })
    .collect()
}

/// The `union.tool_name` a root declares and its snapshots must carry: the agent id with `_`
/// replaced by `-`, with a `-cli` suffix unless it already ends in one (`codex` -> `codex-cli`,
/// `claude_code` -> `claude-code-cli`, `gemini_cli` -> `gemini-cli`).
pub(super) fn tool_name(agent_id: &str) -> String {
    let base = agent_id.replace('_', "-");
    if base == "cli" || base.ends_with("-cli") {
        base
    } else {
        format!("{base}-cli")
    }
}

/// The platform label a canonical target maps to under `union.platform_mapping`. Known
/// operating-system spellings group into `linux`, `macos` and `windows`; any other target is its
/// own platform, so the scaffold never groups targets it cannot classify.
pub(super) fn platform_label(target: &str) -> String {
    let lower = target.to_ascii_lowercase();
    let has = |token: &str| lower.split(['-', '_']).any(|part| part == token);
    if has("linux") {
        "linux".to_string()
    } else if has("darwin") || has("apple") || has("macos") {
        "macos".to_string()
    } else if has("windows") || has("win32") || has("win64") {
        "windows".to_string()
    } else {
        target.to_string()
    }
}

struct TemplateValues {
    agent_id: String,
    display_name: String,
    manifest_root: String,
    tool_name: String,
    targets: Vec<String>,
}

impl TemplateValues {
    fn from_draft(draft: &DraftEntry) -> Self {
        Self {
            agent_id: draft.agent_id.clone(),
            display_name: draft.display_name.clone(),
            manifest_root: draft.manifest_root.trim_end_matches('/').to_string(),
            tool_name: tool_name(&draft.agent_id),
            targets: draft.canonical_targets.clone(),
        }
    }

    fn required_target(&self) -> &str {
        self.targets.first().map(String::as_str).unwrap_or_default()
    }

    fn render_json(&self, template: &str) -> String {
        let expected = render_json_array(&self.targets, 4);
        let mapping = render_platform_mapping(&self.targets, 4);
        template
            .replace("\"{{EXPECTED_TARGETS}}\"", &expected)
            .replace("\"{{PLATFORM_MAPPING}}\"", &mapping)
            .replace("{{REQUIRED_TARGET}}", &json_inner(self.required_target()))
            .replace("{{MANIFEST_ROOT}}", &json_inner(&self.manifest_root))
            .replace("{{TOOL_NAME}}", &json_inner(&self.tool_name))
            .replace("{{DISPLAY_NAME}}", &json_inner(&self.display_name))
            .replace("{{AGENT_ID}}", &json_inner(&self.agent_id))
    }

    fn render_markdown(&self, template: &str) -> String {
        let target_list = self
            .targets
            .iter()
            .map(|target| format!("  - `{target}` (platform `{}`)", platform_label(target)))
            .collect::<Vec<_>>()
            .join("\n");
        template
            .replace("{{TARGET_LIST}}", &target_list)
            .replace("{{REQUIRED_TARGET}}", self.required_target())
            .replace("{{MANIFEST_ROOT}}", &self.manifest_root)
            .replace("{{TOOL_NAME}}", &self.tool_name)
            .replace("{{DISPLAY_NAME}}", &self.display_name)
            .replace("{{AGENT_ID}}", &self.agent_id)
    }
}

/// A string's JSON encoding without the surrounding quotes, for use inside a quoted template slot.
fn json_inner(value: &str) -> String {
    let encoded = serde_json::to_string(value).expect("serialize string");
    encoded[1..encoded.len() - 1].to_string()
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("serialize string")
}

/// A pretty-printed JSON array whose key sits at `indent` spaces, matching the template layout.
fn render_json_array(values: &[String], indent: usize) -> String {
    if values.is_empty() {
        return "[]".to_string();
    }
    let item_pad = " ".repeat(indent + 2);
    let items = values
        .iter()
        .map(|value| format!("{item_pad}{}", json_string(value)))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("[\n{items}\n{}]", " ".repeat(indent))
}

fn render_platform_mapping(targets: &[String], indent: usize) -> String {
    if targets.is_empty() {
        return "{}".to_string();
    }
    let item_pad = " ".repeat(indent + 2);
    let items = targets
        .iter()
        .map(|target| {
            format!(
                "{item_pad}{}: {}",
                json_string(target),
                json_string(&platform_label(target))
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n{items}\n{}}}", " ".repeat(indent))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_names_follow_the_committed_roots() {
        assert_eq!(tool_name("codex"), "codex-cli");
        assert_eq!(tool_name("claude_code"), "claude-code-cli");
        assert_eq!(tool_name("opencode"), "opencode-cli");
        assert_eq!(tool_name("gemini_cli"), "gemini-cli");
        assert_eq!(tool_name("depth_fixture"), "depth-fixture-cli");
    }

    #[test]
    fn platform_labels_follow_the_committed_roots() {
        for (target, label) in [
            ("x86_64-unknown-linux-musl", "linux"),
            ("x86_64-unknown-linux-gnu", "linux"),
            ("linux-x64", "linux"),
            ("aarch64-apple-darwin", "macos"),
            ("darwin-arm64", "macos"),
            ("x86_64-pc-windows-msvc", "windows"),
            ("win32-x64", "windows"),
            ("wasm32-wasip1", "wasm32-wasip1"),
        ] {
            assert_eq!(platform_label(target), label, "{target}");
        }
    }

    #[test]
    fn json_arrays_and_mappings_render_in_target_order() {
        let targets = vec!["linux-x64".to_string(), "darwin-arm64".to_string()];
        assert_eq!(
            render_json_array(&targets, 4),
            "[\n      \"linux-x64\",\n      \"darwin-arm64\"\n    ]"
        );
        assert_eq!(
            render_platform_mapping(&targets, 4),
            "{\n      \"linux-x64\": \"linux\",\n      \"darwin-arm64\": \"macos\"\n    }"
        );
    }
}
