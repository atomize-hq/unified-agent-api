mod render;

use std::{fmt::Write as _, fs, io::Write, path::Path};

use crate::agent_registry::REGISTRY_RELATIVE_PATH;
use crate::proving_run_closeout::{
    load_validated_closeout_if_present, ProvingRunCloseout, ProvingRunCloseoutError,
    ProvingRunCloseoutExpected,
};
use crate::workspace_mutation::WorkspacePathJail;
use toml_edit::DocumentMut;

use self::render::{
    build_docs_preview as render_docs_preview, closeout_relative_path, release_touchpoint_lines,
    PacketPhase,
};
use super::{ConfigGate, DraftEntry, Error, TargetGate, RELEASE_DOC_PATH};

const RELEASE_DOC_START_MARKER: &str =
    "<!-- generated-by: xtask onboard-agent; section: crates-io-release -->";
const RELEASE_DOC_END_MARKER: &str =
    "<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->";
const WRAPPER_EVENTS_PACKAGE_NAME: &str = "unified-agent-api-wrapper-events";
const ROOT_AGENT_API_PACKAGE_NAME: &str = "unified-agent-api";

#[derive(Debug, Clone, Copy)]
pub(super) struct ApprovalRenderInput<'a> {
    pub(super) artifact_path: &'a str,
    pub(super) artifact_sha256: &'a str,
}

#[derive(Debug)]
pub(super) struct ReleasePreview {
    pub(super) lines: Vec<String>,
    pub(super) workspace_manifest: TextMutationPlan,
    pub(super) release_doc: TextMutationPlan,
}

#[derive(Debug, Clone)]
pub(super) struct TextMutationPlan {
    pub(super) path: String,
    pub(super) expected_before: String,
    pub(super) desired_after: String,
}

pub(super) fn build_release_preview(
    workspace_root: &Path,
    draft: &DraftEntry,
) -> Result<ReleasePreview, Error> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let cargo_toml_text = fs::read_to_string(&cargo_toml_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", cargo_toml_path.display())))?;
    let workspace_manifest =
        build_workspace_manifest_mutation(&cargo_toml_text, &draft.crate_path)?;

    let release_doc_path = workspace_root.join(RELEASE_DOC_PATH);
    let release_doc_text = fs::read_to_string(&release_doc_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", release_doc_path.display())))?;
    let publishable_packages =
        publishable_release_packages(workspace_root, draft, &workspace_manifest.desired_after)?;
    let release_doc_block = render_release_doc_block(&publishable_packages);
    let desired_release_doc = splice_release_doc_block(&release_doc_text, &release_doc_block);

    let lines = release_touchpoint_lines(draft);
    Ok(ReleasePreview {
        lines,
        workspace_manifest,
        release_doc: TextMutationPlan {
            path: RELEASE_DOC_PATH.to_string(),
            expected_before: release_doc_text,
            desired_after: desired_release_doc,
        },
    })
}

pub(super) fn load_proving_run_metrics(
    workspace_root: &Path,
    draft: &DraftEntry,
) -> Result<Option<ProvingRunCloseout>, Error> {
    let closeout_path = draft
        .docs_pack_root()
        .join("governance/proving-run-closeout.json");
    let jail = WorkspacePathJail::new(workspace_root)?;
    let resolved_closeout_path = jail.resolve(&closeout_path)?;
    let expected = ProvingRunCloseoutExpected {
        approval_path: draft.approval_identity().map(|(path, _)| Path::new(path)),
        onboarding_pack_prefix: &draft.onboarding_pack_prefix,
    };
    load_validated_closeout_if_present(
        workspace_root,
        Path::new(&closeout_relative_path(draft)),
        &resolved_closeout_path,
        expected,
    )
    .map_err(map_closeout_error)
}

fn map_closeout_error(err: ProvingRunCloseoutError) -> Error {
    match err {
        ProvingRunCloseoutError::Validation(message) => Error::Validation(message),
        ProvingRunCloseoutError::Internal(message) => Error::Internal(message),
    }
}

pub(super) fn write_input_summary<W: Write>(
    writer: &mut W,
    draft: &DraftEntry,
) -> Result<(), Error> {
    let approval = approval_render_input(draft);
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
    if let Some(approval) = approval {
        writeln!(writer, "approval_artifact_path: {}", approval.artifact_path)
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(
            writer,
            "approval_artifact_sha256: {}",
            approval.artifact_sha256
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
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
    closeout: Option<&ProvingRunCloseout>,
) -> Vec<(String, Option<String>)> {
    let approval = approval_render_input(draft);
    let phase = match closeout {
        Some(closeout) => PacketPhase::Closeout(closeout),
        None => PacketPhase::Execution,
    };
    render_docs_preview(draft, &release_preview.lines, phase, approval)
}

fn approval_render_input(draft: &DraftEntry) -> Option<ApprovalRenderInput<'_>> {
    let (artifact_path, artifact_sha256) = draft.approval_identity()?;
    Some(ApprovalRenderInput {
        artifact_path,
        artifact_sha256,
    })
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

fn render_current_json(draft: &DraftEntry) -> String {
    let targets = draft
        .canonical_targets
        .iter()
        .map(|target| format!("    \"{target}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n  \"expected_targets\": [\n{targets}\n  ],\n  \"inputs\": []\n}}\n")
}

pub(super) fn build_manual_follow_up(
    draft: &DraftEntry,
    closeout: Option<&ProvingRunCloseout>,
) -> Vec<String> {
    match closeout {
        Some(_) => vec![
            "No open runtime follow-up remains; the proving run is closed.".to_string(),
        ],
        None => vec![
            format!(
                "Next executable runtime step: implement the runtime-owned wrapper crate at `{}` and backend module `{}`.",
                draft.crate_path, draft.backend_module
            ),
            format!(
                "When the wrapper crate is crates.io-publishable, include crate-local `README.md`, `LICENSE-APACHE`, `LICENSE-MIT`, and set `readme = \"README.md\"` in `{}/Cargo.toml`.",
                draft.crate_path
            ),
            format!(
                "Author wrapper coverage input at `{}` for binding kind `{}`.",
                draft.wrapper_coverage_source_path, draft.wrapper_coverage_binding_kind
            ),
            format!(
                "Populate `{}/current.json`, pointers, versions, and reports from committed runtime evidence.",
                draft.manifest_root
            ),
            "Regenerate support and capability publication artifacts, then run `make preflight`."
                .to_string(),
        ],
    }
}

fn build_workspace_manifest_mutation(
    cargo_toml_text: &str,
    crate_path: &str,
) -> Result<TextMutationPlan, Error> {
    let mut doc = cargo_toml_text
        .parse::<DocumentMut>()
        .map_err(|err| Error::Internal(format!("parse Cargo.toml: {err}")))?;
    let members = doc["workspace"]["members"]
        .as_array_mut()
        .ok_or_else(|| Error::Internal("workspace.members must be an array".to_string()))?;
    let already_present = members
        .iter()
        .any(|member| member.as_str() == Some(crate_path));
    if !already_present {
        let insert_index = members
            .iter()
            .position(|member| member.as_str() == Some("crates/wrapper_events"))
            .or_else(|| {
                members
                    .iter()
                    .position(|member| member.as_str() == Some("crates/xtask"))
            })
            .unwrap_or(members.len());
        members.insert(insert_index, toml_edit::Value::from(crate_path));
    }
    let mut desired_after = doc.to_string();
    if !desired_after.ends_with('\n') {
        desired_after.push('\n');
    }

    Ok(TextMutationPlan {
        path: "Cargo.toml".to_string(),
        expected_before: cargo_toml_text.to_string(),
        desired_after,
    })
}

fn publishable_release_packages(
    workspace_root: &Path,
    draft: &DraftEntry,
    desired_workspace_manifest: &str,
) -> Result<Vec<String>, Error> {
    let doc = desired_workspace_manifest
        .parse::<DocumentMut>()
        .map_err(|err| Error::Internal(format!("parse desired Cargo.toml: {err}")))?;
    let members = doc["workspace"]["members"]
        .as_array()
        .ok_or_else(|| Error::Internal("workspace.members must be an array".to_string()))?;

    let mut leaf_packages = Vec::new();
    let mut wrapper_events = None;
    let mut root_agent_api = None;

    for member in members {
        let Some(member_path) = member.as_str() else {
            return Err(Error::Internal(
                "workspace.members entries must be strings".to_string(),
            ));
        };

        let package_name = if member_path == draft.crate_path {
            draft.package_name.clone()
        } else {
            let manifest_path = workspace_root.join(member_path).join("Cargo.toml");
            let manifest_text = match fs::read_to_string(&manifest_path) {
                Ok(text) => text,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => {
                    return Err(Error::Internal(format!(
                        "read {}: {err}",
                        manifest_path.display()
                    )));
                }
            };
            let manifest = manifest_text.parse::<DocumentMut>().map_err(|err| {
                Error::Internal(format!("parse {}: {err}", manifest_path.display()))
            })?;
            if package_publish_disabled(&manifest) {
                continue;
            }
            let Some(package_name) = manifest
                .get("package")
                .and_then(toml_edit::Item::as_table_like)
                .and_then(|package| package.get("name"))
                .and_then(toml_edit::Item::as_str)
            else {
                continue;
            };
            if !package_name.starts_with("unified-agent-api") {
                continue;
            }
            package_name.to_string()
        };

        if package_name == WRAPPER_EVENTS_PACKAGE_NAME {
            wrapper_events = Some(package_name);
        } else if package_name == ROOT_AGENT_API_PACKAGE_NAME {
            root_agent_api = Some(package_name);
        } else {
            leaf_packages.push(package_name);
        }
    }

    if let Some(package_name) = wrapper_events {
        leaf_packages.push(package_name);
    }
    if let Some(package_name) = root_agent_api {
        leaf_packages.push(package_name);
    }
    Ok(leaf_packages)
}

fn package_publish_disabled(doc: &DocumentMut) -> bool {
    doc.get("package")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|package| package.get("publish"))
        .and_then(toml_edit::Item::as_bool)
        == Some(false)
}

fn render_release_doc_block(packages: &[String]) -> String {
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

fn splice_release_doc_block(existing: &str, block: &str) -> String {
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
