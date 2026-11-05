//! Governing data that `onboard-agent` scaffolds in a new manifest root: `RULES.json`,
//! `SCHEMA.json`, `VERSION_METADATA_SCHEMA.json`, `VALIDATOR_SPEC.md`, the root version pointers
//! (`min_supported.txt`, `latest_validated.txt` and each target's pointer files, all `none`) and a
//! `wrapper_coverage.json` that claims no coverage.
//!
//! The templates follow the shape of the committed roots (the synthetic subject's governing
//! templates in `crates/xtask/tests/fixtures/depth-synthetic/governing/`). Only agent identity,
//! the manifest root and the canonical target vary. The scaffold declares no parity exclusions,
//! no acquisition block and no depth-record path, so the root does not adopt Annex B of the
//! support-depth contract and cannot hold a depth record.

use std::path::Path;

use super::super::{DraftEntry, Error};

const RULES_TEMPLATE: &str = include_str!("templates/RULES.json");
const SCHEMA_TEMPLATE: &str = include_str!("templates/SCHEMA.json");
const VERSION_METADATA_SCHEMA_TEMPLATE: &str =
    include_str!("templates/VERSION_METADATA_SCHEMA.json");
const VALIDATOR_SPEC_TEMPLATE: &str = include_str!("templates/VALIDATOR_SPEC.md");
/// The value of a version pointer before any version is known (`root_pointers_allow_none`).
const NO_VERSION: &str = "none\n";
/// Coverage before the wrapper's generator has run: a schema-valid file that claims nothing.
const EMPTY_WRAPPER_COVERAGE: &str = "{\n  \"schema_version\": 1,\n  \"coverage\": []\n}\n";

/// Refuse a manifest root the scaffold cannot write truthfully, before any preview or write.
///
/// - A root with more than one canonical target needs a required target. `manifest-promote`
///   advances `current.json` only once that target passes, and the registry declares targets as
///   an ordered list with no required-target field, so order would silently decide it. Which
///   target is required is an undecided policy, so the scaffold refuses rather than choose.
/// - An approval whose `[support_depth]` table states an exact upstream version asks for version
///   metadata for that version. The version-metadata schema has no status for a version whose
///   surface was never acquired, and onboarding depth enablement is not wired (roadmap E.6), so
///   the scaffold refuses rather than write metadata that claims a snapshot.
pub(crate) fn refuse_unscaffoldable(draft: &DraftEntry) -> Result<(), Error> {
    if draft.canonical_targets.len() != 1 {
        return Err(Error::Validation(format!(
            "onboard-agent scaffolds a manifest root for exactly one canonical target; `{}` declares {} ({}). \
             No required-target policy decides which target must pass before promotion, so a \
             multi-target root is refused rather than chosen by target order",
            draft.agent_id,
            draft.canonical_targets.len(),
            draft.canonical_targets.join(", ")
        )));
    }
    if let Some(version) = draft.approval_support_depth_version.as_deref() {
        return Err(Error::Validation(format!(
            "approval states `[support_depth].version = \"{version}\"`: onboard-agent cannot write \
             version metadata for `{version}` before its surface is acquired, and onboarding depth \
             enablement is not available; refusing"
        )));
    }
    Ok(())
}

/// The governing files, as `(repo-relative path, contents)` in a fixed order.
pub(super) fn governing_files(draft: &DraftEntry) -> Vec<(String, String)> {
    let values = TemplateValues::from_draft(draft);
    let root = Path::new(&draft.manifest_root);
    let mut files = [
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
        let contents = if name.ends_with(".md") {
            values.render_markdown(template)
        } else {
            values.render_json(template)
        };
        (root.join(name).display().to_string(), contents)
    })
    .collect::<Vec<_>>();
    files.push((
        root.join("wrapper_coverage.json").display().to_string(),
        EMPTY_WRAPPER_COVERAGE.to_string(),
    ));
    let mut pointers = vec![
        "min_supported.txt".to_string(),
        "latest_validated.txt".to_string(),
    ];
    for target in &draft.canonical_targets {
        pointers.push(format!("pointers/latest_supported/{target}.txt"));
        pointers.push(format!("pointers/latest_validated/{target}.txt"));
    }
    files.extend(pointers.into_iter().map(|pointer| {
        (
            root.join(pointer).display().to_string(),
            NO_VERSION.to_string(),
        )
    }));
    files
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
        template
            .replace(
                "{{REQUIRED_PLATFORM}}",
                &platform_label(self.required_target()),
            )
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
