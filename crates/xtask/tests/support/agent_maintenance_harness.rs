use std::path::Path;

use crate::{
    harness::{seed_gemini_approval_artifact, seed_release_touchpoints, write_text},
    release_doc, support_matrix,
};

pub fn seed_opencode_basis(root: &Path) {
    seed_release_touchpoints(root);
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nManual contract text.\n",
    );
    seed_publishable_workspace_member(root, "crates/gemini_cli", "unified-agent-api-gemini-cli");
    seed_cli_manifest_root(
        root,
        "cli_manifests/codex",
        &["x86_64-unknown-linux-musl"],
        &[(&["mcp", "list"], &["x86_64-unknown-linux-musl"])],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/claude_code",
        &["linux-x64"],
        &[(&["mcp", "list"], &["linux-x64"])],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/opencode",
        &["linux-x64", "darwin-arm64", "win32-x64"],
        &[],
    );
    seed_cli_manifest_root(root, "cli_manifests/gemini_cli", &["darwin-arm64"], &[]);
    seed_cli_manifest_root(root, "cli_manifests/aider", &["darwin-arm64"], &[]);

    let support_bundle =
        support_matrix::generate_publication_artifacts(root).expect("generate support publication");
    write_text(
        &root.join("cli_manifests/support_matrix/current.json"),
        &support_bundle.json,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        &support_bundle.markdown,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/capability-matrix.md"),
        &default_capability_matrix_markdown(),
    );

    seed_gemini_approval_artifact(
        root,
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml",
        "gemini-cli-onboarding",
    );
    seed_clean_governance_closeouts(root);

    let release_doc = release_doc::render_release_doc(root).expect("render release doc");
    write_text(&root.join(release_doc::RELEASE_DOC_PATH), &release_doc);
}

pub fn overwrite_opencode_governance_with_stale_claim(root: &Path) {
    write_text(
        &root.join(
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
        "# Closeout\n\n- capability advertisement is intentionally conservative and now matches the landed backend contract and generated capability inventory:\n  <!-- xtask-governance-check:opencode-capabilities:start -->\n  `agent_api.events`, `agent_api.events.live`, `agent_api.run`\n  <!-- xtask-governance-check:opencode-capabilities:end -->\n  are the claimed OpenCode v1 capability ids under the current runtime evidence\n",
    );
}

fn seed_clean_governance_closeouts(root: &Path) {
    write_text(
        &root.join(
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
        "# Closeout\n\n- capability advertisement is intentionally conservative and now matches the landed backend contract and generated capability inventory:\n  <!-- xtask-governance-check:opencode-capabilities:start -->\n  `agent_api.config.model.v1`, `agent_api.events`, `agent_api.events.live`, `agent_api.run`, `agent_api.session.fork.v1`, `agent_api.session.resume.v1`\n  <!-- xtask-governance-check:opencode-capabilities:end -->\n  are the claimed OpenCode v1 capability ids under the current runtime evidence\n",
    );
    write_text(
        &root.join(
            "docs/integrations/opencode/governance/seam-3-closeout.md",
        ),
        "# Closeout\n\n- the support publication artifacts now show OpenCode as manifest-supported only where committed root evidence justifies it, while\n  <!-- xtask-governance-check:opencode-support:start -->\n  backend_support = supported\n  uaa_support = supported\n  <!-- xtask-governance-check:opencode-support:end -->\n  under the current backend evidence and pointer posture\n",
    );
}

fn seed_publishable_workspace_member(root: &Path, member_path: &str, package_name: &str) {
    write_text(
        &root.join(member_path).join("Cargo.toml"),
        &format!("[package]\nname = \"{package_name}\"\nversion = \"0.2.3\"\nedition = \"2021\"\n"),
    );
}

fn seed_cli_manifest_root(
    root: &Path,
    manifest_root: &str,
    canonical_targets: &[&str],
    commands: &[(&[&str], &[&str])],
) {
    let current = serde_json::json!({
        "expected_targets": canonical_targets,
        "inputs": canonical_targets
            .iter()
            .map(|target| serde_json::json!({
                "target_triple": target,
                "binary": { "semantic_version": "1.0.0" }
            }))
            .collect::<Vec<_>>(),
        "commands": commands
            .iter()
            .map(|(path, available_on)| serde_json::json!({
                "path": path,
                "available_on": available_on,
            }))
            .collect::<Vec<_>>(),
    });
    write_text(
        &root.join(manifest_root).join("current.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&current).expect("serialize current manifest")
        ),
    );

    let version = serde_json::json!({
        "semantic_version": "1.0.0",
        "status": "supported",
        "coverage": { "supported_targets": canonical_targets },
    });
    write_text(
        &root.join(manifest_root).join("versions/1.0.0.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&version).expect("serialize version metadata")
        ),
    );

    for target in canonical_targets {
        write_text(
            &root
                .join(manifest_root)
                .join(format!("pointers/latest_supported/{target}.txt")),
            "1.0.0\n",
        );
        write_text(
            &root
                .join(manifest_root)
                .join(format!("pointers/latest_validated/{target}.txt")),
            "1.0.0\n",
        );
        let report = serde_json::json!({
            "inputs": { "upstream": { "targets": [target] } },
            "deltas": {
                "missing_commands": [],
                "missing_flags": [],
                "missing_args": [],
                "intentionally_unsupported": [],
                "wrapper_only_commands": [],
                "wrapper_only_flags": [],
                "wrapper_only_args": [],
            }
        });
        write_text(
            &root
                .join(manifest_root)
                .join(format!("reports/1.0.0/coverage.{target}.json")),
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&report).expect("serialize report")
            ),
        );
    }
}

fn default_capability_matrix_markdown() -> String {
    include_str!("../../../../docs/specs/unified-agent-api/capability-matrix.md").to_string()
}
