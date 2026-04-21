use std::{collections::BTreeSet, fmt::Write as _, fs, io::Write, path::Path};

use crate::agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH};
use toml_edit::DocumentMut;

use super::{
    ConfigGate, DraftEntry, Error, TargetGate, CHECK_PUBLISH_READINESS_SCRIPT_PATH,
    OWNERSHIP_MARKER, PUBLISH_SCRIPT_PATH, PUBLISH_WORKFLOW_PATH, RELEASE_DOC_PATH,
    VALIDATE_PUBLISH_SCRIPT_PATH,
};

#[derive(Debug)]
pub(super) struct ReleasePreview {
    pub(super) lines: Vec<String>,
}

pub(super) fn build_release_preview(
    workspace_root: &Path,
    registry: &AgentRegistry,
    draft: &DraftEntry,
) -> Result<ReleasePreview, Error> {
    let seeded_package_names = registry
        .agents
        .iter()
        .map(|entry| entry.package_name.as_str())
        .collect::<BTreeSet<_>>();

    if seeded_package_names.contains(draft.package_name.as_str()) {
        return Ok(ReleasePreview {
            lines: vec!["NO RELEASE CHANGES".to_string()],
        });
    }

    let mut lines = Vec::new();
    if !workspace_membership_contains(workspace_root, &draft.crate_path)? {
        lines.push(format!(
            "FUTURE M2: Cargo.toml must add workspace member `{}`.",
            draft.crate_path
        ));
    }
    lines.push(format!(
        "FUTURE M2: {RELEASE_DOC_PATH} must add `{}` on release track `{}`.",
        draft.package_name, draft.docs_release_track
    ));
    lines.push(format!(
        "Workflow and script files remain unchanged in M1: {PUBLISH_WORKFLOW_PATH}, {PUBLISH_SCRIPT_PATH}, {VALIDATE_PUBLISH_SCRIPT_PATH}, {CHECK_PUBLISH_READINESS_SCRIPT_PATH}."
    ));
    Ok(ReleasePreview { lines })
}

pub(super) fn write_input_summary<W: Write>(
    writer: &mut W,
    draft: &DraftEntry,
) -> Result<(), Error> {
    writeln!(writer, "== INPUT SUMMARY ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "agent_id: {}", draft.agent_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "display_name: {}", draft.display_name)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "crate_path: {}", draft.crate_path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "backend_module: {}", draft.backend_module)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "manifest_root: {}", draft.manifest_root)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "package_name: {}", draft.package_name)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write_list(writer, "canonical_targets", &draft.canonical_targets)?;
    writeln!(
        writer,
        "wrapper_coverage_binding_kind: {}",
        draft.wrapper_coverage_binding_kind
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "wrapper_coverage_source_path: {}",
        draft.wrapper_coverage_source_path
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write_list(
        writer,
        "always_on_capabilities",
        &draft.always_on_capabilities,
    )?;
    write_gate_summaries(
        writer,
        &draft.target_gated_capabilities,
        &draft.config_gated_capabilities,
    )?;
    write_list(writer, "backend_extensions", &draft.backend_extensions)?;
    writeln!(
        writer,
        "support_matrix_enabled: {}",
        draft.support_matrix_enabled
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "capability_matrix_enabled: {}",
        draft.capability_matrix_enabled
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "docs_release_track: {}", draft.docs_release_track)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "onboarding_pack_prefix: {}",
        draft.onboarding_pack_prefix
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn write_registry_preview<W: Write>(writer: &mut W, preview: &str) -> Result<(), Error> {
    writeln!(writer, "== REGISTRY ENTRY PREVIEW ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "Path: {REGISTRY_RELATIVE_PATH}")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write_code_block(writer, "toml", preview)?;
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn write_docs_preview<W: Write>(
    writer: &mut W,
    previews: &[(String, Option<String>)],
) -> Result<(), Error> {
    writeln!(writer, "== DOCS SCAFFOLD PREVIEW ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for (path, contents) in previews {
        writeln!(writer, "Path: {path}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        if let Some(contents) = contents {
            write_code_block(writer, "md", contents)?;
        } else {
            writeln!(writer, "(empty file)")
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        }
    }
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn write_manifest_preview<W: Write>(
    writer: &mut W,
    previews: &[(String, Option<String>)],
) -> Result<(), Error> {
    writeln!(writer, "== MANIFEST ROOT PREVIEW ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for (path, contents) in previews {
        writeln!(writer, "Path: {path}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        if let Some(contents) = contents {
            write_code_block(writer, "json", contents)?;
        } else {
            writeln!(writer, "(empty file)")
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        }
    }
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn write_release_preview<W: Write>(
    writer: &mut W,
    preview: &ReleasePreview,
) -> Result<(), Error> {
    writeln!(writer, "== RELEASE/PUBLICATION TOUCHPOINTS ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for line in &preview.lines {
        writeln!(writer, "{line}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn write_manual_follow_up<W: Write>(
    writer: &mut W,
    lines: &[String],
) -> Result<(), Error> {
    writeln!(writer, "== MANUAL FOLLOW-UP ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for line in lines {
        writeln!(writer, "- {line}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn render_registry_entry_preview(draft: &DraftEntry) -> String {
    let mut out = String::new();
    writeln!(&mut out, "[[agents]]").expect("write String");
    writeln!(&mut out, "agent_id = {:?}", draft.agent_id).expect("write String");
    writeln!(&mut out, "display_name = {:?}", draft.display_name).expect("write String");
    writeln!(&mut out, "crate_path = {:?}", draft.crate_path).expect("write String");
    writeln!(&mut out, "backend_module = {:?}", draft.backend_module).expect("write String");
    writeln!(&mut out, "manifest_root = {:?}", draft.manifest_root).expect("write String");
    writeln!(&mut out, "package_name = {:?}", draft.package_name).expect("write String");
    writeln!(
        &mut out,
        "canonical_targets = {}",
        render_string_array(&draft.canonical_targets)
    )
    .expect("write String");
    writeln!(&mut out).expect("write String");

    writeln!(&mut out, "[agents.wrapper_coverage]").expect("write String");
    writeln!(
        &mut out,
        "binding_kind = {:?}",
        draft.wrapper_coverage_binding_kind
    )
    .expect("write String");
    writeln!(
        &mut out,
        "source_path = {:?}",
        draft.wrapper_coverage_source_path
    )
    .expect("write String");
    writeln!(&mut out).expect("write String");

    writeln!(&mut out, "[agents.capability_declaration]").expect("write String");
    writeln!(
        &mut out,
        "always_on = {}",
        render_string_array(&draft.always_on_capabilities)
    )
    .expect("write String");
    writeln!(
        &mut out,
        "backend_extensions = {}",
        render_string_array(&draft.backend_extensions)
    )
    .expect("write String");
    if !draft.target_gated_capabilities.is_empty() {
        writeln!(&mut out).expect("write String");
    }

    for target_gate in &draft.target_gated_capabilities {
        writeln!(&mut out, "[[agents.capability_declaration.target_gated]]").expect("write String");
        writeln!(&mut out, "capability_id = {:?}", target_gate.capability_id)
            .expect("write String");
        writeln!(
            &mut out,
            "targets = {}",
            render_string_array(&target_gate.targets)
        )
        .expect("write String");
        writeln!(&mut out).expect("write String");
    }

    for config_gate in &draft.config_gated_capabilities {
        writeln!(&mut out, "[[agents.capability_declaration.config_gated]]").expect("write String");
        writeln!(&mut out, "capability_id = {:?}", config_gate.capability_id)
            .expect("write String");
        writeln!(&mut out, "config_key = {:?}", config_gate.config_key).expect("write String");
        if let Some(targets) = &config_gate.targets {
            writeln!(&mut out, "targets = {}", render_string_array(targets)).expect("write String");
        }
        writeln!(&mut out).expect("write String");
    }

    writeln!(&mut out, "[agents.publication]").expect("write String");
    writeln!(
        &mut out,
        "support_matrix_enabled = {}",
        draft.support_matrix_enabled
    )
    .expect("write String");
    writeln!(
        &mut out,
        "capability_matrix_enabled = {}",
        draft.capability_matrix_enabled
    )
    .expect("write String");
    writeln!(&mut out).expect("write String");

    writeln!(&mut out, "[agents.release]").expect("write String");
    writeln!(
        &mut out,
        "docs_release_track = {:?}",
        draft.docs_release_track
    )
    .expect("write String");
    writeln!(&mut out).expect("write String");

    writeln!(&mut out, "[agents.scaffold]").expect("write String");
    writeln!(
        &mut out,
        "onboarding_pack_prefix = {:?}",
        draft.onboarding_pack_prefix
    )
    .expect("write String");
    out
}

pub(super) fn build_docs_preview(
    draft: &DraftEntry,
    release_preview: &ReleasePreview,
) -> Vec<(String, Option<String>)> {
    let docs_root = draft.docs_pack_root();
    let docs_root_display = docs_root.display().to_string();
    let release_touchpoints = release_preview
        .lines
        .iter()
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    vec![
        (
            docs_root.join("README.md").display().to_string(),
            Some(render_markdown_file(format!(
                "# {} onboarding pack\n\nThis preview seeds the control-plane onboarding packet for `{}`.\n\n- Agent id: `{}`\n- Wrapper crate: `{}`\n- Backend module: `{}`\n- Manifest root: `{}`\n",
                draft.display_name,
                draft.display_name,
                draft.agent_id,
                draft.crate_path,
                draft.backend_module,
                draft.manifest_root
            ))),
        ),
        (
            docs_root.join("scope_brief.md").display().to_string(),
            Some(render_markdown_file(format!(
                "# Scope brief\n\nControl-plane-owned preview outputs:\n\n- Registry enrollment preview in `{REGISTRY_RELATIVE_PATH}`\n- Docs scaffold preview in `{docs_root_display}`\n- Manifest-root scaffold preview in `{}`\n\nRuntime-owned implementation remains manual in M1.\n",
                draft.manifest_root
            ))),
        ),
        (
            docs_root.join("seam_map.md").display().to_string(),
            Some(render_markdown_file(format!(
                "# Seam map\n\n- Declaration seam: registry entry for `{}`\n- Docs seam: onboarding pack `{docs_root_display}`\n- Manifest seam: `{}` skeleton\n- Runtime seam: wrapper crate `{}` and backend module `{}`\n",
                draft.agent_id,
                draft.manifest_root,
                draft.crate_path,
                draft.backend_module
            ))),
        ),
        (
            docs_root.join("threading.md").display().to_string(),
            Some(render_markdown_file(
                "# Threading\n\n1. Approve the dry-run preview.\n2. Materialize runtime-owned wrapper and backend work outside M1.\n3. Populate manifest evidence after runtime artifacts exist.\n4. Apply any future M2 release/doc mutations listed in this packet.\n".to_string(),
            )),
        ),
        (
            docs_root.join("review_surfaces.md").display().to_string(),
            Some(render_markdown_file(format!(
                "# Review surfaces\n\n- `{REGISTRY_RELATIVE_PATH}`\n- `{docs_root_display}`\n- `{}`\n- `{RELEASE_DOC_PATH}`\n- `{PUBLISH_WORKFLOW_PATH}` remains unchanged in M1\n",
                draft.manifest_root
            ))),
        ),
        (
            docs_root
                .join("governance/remediation-log.md")
                .display()
                .to_string(),
            Some(render_markdown_file(
                "# Remediation log\n\nNo mutations are applied in M1 dry-run mode. Record follow-up decisions here once runtime-owned work starts.\n".to_string(),
            )),
        ),
        (
            docs_root.join("HANDOFF.md").display().to_string(),
            Some(render_markdown_file(render_handoff_body(
                draft,
                &release_touchpoints,
                &build_manual_follow_up(draft),
            ))),
        ),
    ]
}

pub(super) fn build_manifest_preview(draft: &DraftEntry) -> Vec<(String, Option<String>)> {
    vec![
        (
            Path::new(&draft.manifest_root)
                .join("current.json")
                .display()
                .to_string(),
            Some(render_current_json(draft)),
        ),
        (
            Path::new(&draft.manifest_root)
                .join("versions/.gitkeep")
                .display()
                .to_string(),
            None,
        ),
        (
            Path::new(&draft.manifest_root)
                .join("pointers/latest_supported/.gitkeep")
                .display()
                .to_string(),
            None,
        ),
        (
            Path::new(&draft.manifest_root)
                .join("pointers/latest_validated/.gitkeep")
                .display()
                .to_string(),
            None,
        ),
        (
            Path::new(&draft.manifest_root)
                .join("reports/.gitkeep")
                .display()
                .to_string(),
            None,
        ),
    ]
}

pub(super) fn build_manual_follow_up(draft: &DraftEntry) -> Vec<String> {
    vec![
        format!(
            "Create the wrapper crate at `{}` and keep any file edits runtime-owned.",
            draft.crate_path
        ),
        format!(
            "Implement backend behavior under `{}` and ensure backend-owned capability extensions match the preview.",
            draft.backend_module
        ),
        format!(
            "Author wrapper coverage input at `{}` for binding kind `{}`.",
            draft.wrapper_coverage_source_path, draft.wrapper_coverage_binding_kind
        ),
        format!(
            "Populate `{}/current.json`, pointers, versions, and reports from committed runtime evidence once the agent exists.",
            draft.manifest_root
        ),
        "Re-run `xtask onboard-agent --dry-run` after runtime-owned work changes the proposed artifact set."
            .to_string(),
    ]
}

fn workspace_membership_contains(workspace_root: &Path, crate_path: &str) -> Result<bool, Error> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let text = fs::read_to_string(&cargo_toml_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", cargo_toml_path.display())))?;
    let doc = text
        .parse::<DocumentMut>()
        .map_err(|err| Error::Internal(format!("parse {}: {err}", cargo_toml_path.display())))?;
    let members = doc["workspace"]["members"]
        .as_array()
        .ok_or_else(|| Error::Internal("workspace.members must be an array".to_string()))?;
    let contains = members
        .iter()
        .any(|member| member.as_str() == Some(crate_path));
    Ok(contains)
}

fn write_list<W: Write>(writer: &mut W, label: &str, values: &[String]) -> Result<(), Error> {
    writeln!(writer, "{label}:").map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if values.is_empty() {
        writeln!(writer, "- (none)")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        return Ok(());
    }
    for value in values {
        writeln!(writer, "- {value}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    Ok(())
}

fn write_gate_summaries<W: Write>(
    writer: &mut W,
    target_gated: &[TargetGate],
    config_gated: &[ConfigGate],
) -> Result<(), Error> {
    writeln!(writer, "target_gated_capabilities:")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if target_gated.is_empty() {
        writeln!(writer, "- (none)")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    } else {
        for gate in target_gated {
            writeln!(
                writer,
                "- {} => {}",
                gate.capability_id,
                gate.targets.join(",")
            )
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        }
    }
    writeln!(writer, "config_gated_capabilities:")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if config_gated.is_empty() {
        writeln!(writer, "- (none)")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    } else {
        for gate in config_gated {
            let suffix = gate
                .targets
                .as_ref()
                .map(|targets| format!(" => {}", targets.join(",")))
                .unwrap_or_default();
            writeln!(
                writer,
                "- {}:{}{}",
                gate.capability_id, gate.config_key, suffix
            )
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        }
    }
    Ok(())
}

fn write_code_block<W: Write>(writer: &mut W, language: &str, contents: &str) -> Result<(), Error> {
    writeln!(writer, "```{language}")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write!(writer, "{contents}").map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if !contents.ends_with('\n') {
        writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    writeln!(writer, "```").map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn render_string_array(values: &[String]) -> String {
    let rendered = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>();
    format!("[{}]", rendered.join(", "))
}

fn render_handoff_body(
    draft: &DraftEntry,
    release_touchpoints: &str,
    manual_follow_up: &[String],
) -> String {
    format!(
        "# Handoff\n\nThis packet previews the next executable control-plane artifacts for `{}`.\n\n## Release touchpoints\n\n{}\n\n## Manual Runtime Follow-Up\n\n{}\n",
        draft.agent_id,
        release_touchpoints,
        manual_follow_up
            .iter()
            .map(|line| format!("- {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn render_markdown_file(body: String) -> String {
    format!("{OWNERSHIP_MARKER}\n\n{body}")
}

fn render_current_json(draft: &DraftEntry) -> String {
    let targets = draft
        .canonical_targets
        .iter()
        .map(|target| format!("    \"{target}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n  \"expected_targets\": [\n{targets}\n  ],\n  \"inputs\": []\n}}\n")
}
