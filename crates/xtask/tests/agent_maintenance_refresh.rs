#![allow(dead_code, unused_imports, clippy::enum_variant_names)]

use std::{collections::BTreeSet, fs, path::Path};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

mod agent_registry {
    pub use xtask::agent_registry::*;
}
mod capability_publication {
    pub use xtask::capability_publication::*;
}
mod publication_refresh {
    pub use xtask::publication_refresh::*;
}
#[path = "../src/capability_matrix.rs"]
mod capability_matrix;
#[path = "../src/capability_projection.rs"]
mod capability_projection;
#[path = "../src/agent_maintenance/docs.rs"]
mod docs;
#[path = "../src/agent_maintenance/refresh.rs"]
mod refresh;
#[path = "../src/release_doc.rs"]
mod release_doc;
#[path = "../src/agent_maintenance/request.rs"]
mod request;
#[path = "../src/root_intake_layout.rs"]
mod root_intake_layout;
#[path = "../src/support_matrix.rs"]
mod support_matrix;
#[path = "../src/workspace_mutation.rs"]
mod workspace_mutation;
use harness::{fixture_root, seed_release_touchpoints, snapshot_files, write_text};
use refresh::{apply_refresh_plan, build_refresh_plan};
use request::load_request;

#[test]
fn request_outside_maintenance_root_rejected() {
    let fixture = fixture_root("agent-maintenance-request-path");
    seed_publication_inputs(&fixture);

    let invalid_request =
        "docs/project_management/next/opencode-implementation/governance/maintenance-request.toml";
    write_text(
        &fixture.join(invalid_request),
        &request_toml("opencode", &["packet_doc_refresh"], false, &[]),
    );

    let err = load_request(&fixture, Path::new(invalid_request))
        .expect_err("request outside maintenance root should fail");
    assert!(err.to_string().contains("maintenance-request.toml"));
}

#[test]
fn runtime_owned_actions_rejected() {
    let fixture = fixture_root("agent-maintenance-runtime-owned-actions");
    seed_publication_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &request_toml(
            "opencode",
            &["packet_doc_refresh", "runtime_code_refresh"],
            false,
            &[],
        ),
    );

    let err = build_refresh_plan(&fixture, Path::new(request_path))
        .expect_err("runtime-owned actions should fail validation");
    assert!(err
        .to_string()
        .contains("runtime-owned or unsupported action"));
    assert!(err.to_string().contains("runtime_code_refresh"));
}

#[test]
fn missing_basis_ref_is_rejected() {
    let fixture = fixture_root("agent-maintenance-missing-basis-ref");
    seed_publication_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &request_toml_with_refs(
            "opencode",
            "docs/agents/lifecycle/opencode-maintenance/governance/missing-basis.md",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
            &["packet_doc_refresh"],
            false,
            &[],
        ),
    );

    let err = build_refresh_plan(&fixture, Path::new(request_path)).expect_err("missing basis ref");
    assert!(err.to_string().contains("field `basis_ref`"));
    assert!(err.to_string().contains("must point to an existing file"));
}

#[test]
fn missing_opened_from_is_rejected() {
    let fixture = fixture_root("agent-maintenance-missing-opened-from");
    seed_publication_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &request_toml_with_refs(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
            "docs/agents/lifecycle/opencode-maintenance/governance/missing-opened-from.md",
            &["packet_doc_refresh"],
            false,
            &[],
        ),
    );

    let err =
        build_refresh_plan(&fixture, Path::new(request_path)).expect_err("missing opened_from");
    assert!(err.to_string().contains("field `opened_from`"));
    assert!(err.to_string().contains("must point to an existing file"));
}

#[test]
fn release_doc_refresh_uses_registry_order_instead_of_workspace_member_order() {
    let fixture = fixture_root("agent-maintenance-release-doc-registry-order");
    seed_publication_inputs(&fixture);
    write_text(
        &fixture.join("Cargo.toml"),
        "[workspace]\nmembers = [\n  \"crates/agent_api\",\n  \"crates/opencode\",\n  \"crates/codex\",\n  \"crates/claude_code\",\n  \"crates/gemini_cli\",\n  \"crates/aider\",\n  \"crates/wrapper_events\",\n  \"crates/xtask\",\n]\n",
    );

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &request_toml("opencode", &["release_doc_refresh"], false, &[]),
    );

    let plan = build_refresh_plan(&fixture, Path::new(request_path)).expect("build refresh plan");
    let release_doc = plan
        .files
        .iter()
        .find(|file| file.relative_path == release_doc::RELEASE_DOC_PATH)
        .expect("release doc planned");
    let markdown = String::from_utf8(release_doc.contents.clone()).expect("utf8 release doc");

    let codex_position = markdown
        .find("1. `unified-agent-api-codex`")
        .expect("codex order");
    let opencode_position = markdown
        .find("3. `unified-agent-api-opencode`")
        .expect("opencode order");
    assert!(codex_position < opencode_position);
}

#[test]
fn dry_run_write_plan_identity_and_no_write_vs_write_parity() {
    let fixture = fixture_root("agent-maintenance-plan-parity");
    seed_publication_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &request_toml(
            "opencode",
            &[
                "packet_doc_refresh",
                "capability_matrix_refresh",
                "release_doc_refresh",
            ],
            true,
            &["Update the runtime-owned stale capability statement in the implementation pack."],
        ),
    );

    let before = snapshot_files(&fixture);
    let dry_run_plan =
        build_refresh_plan(&fixture, Path::new(request_path)).expect("build dry-run plan");
    let write_plan =
        build_refresh_plan(&fixture, Path::new(request_path)).expect("build write plan");
    let after_build = snapshot_files(&fixture);

    assert_eq!(dry_run_plan, write_plan, "plan build must be identical");
    assert_eq!(before, after_build, "plan build must not write files");

    let summary = apply_refresh_plan(&fixture, &write_plan).expect("apply refresh plan");
    assert_eq!(summary.total, write_plan.files.len());
    assert_eq!(summary.written, write_plan.files.len());

    let changed_paths = diff_paths(&before, &snapshot_files(&fixture));
    let planned_paths: BTreeSet<String> = write_plan
        .planned_paths()
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(
        changed_paths, planned_paths,
        "write mode should apply the same planned files"
    );
}

#[test]
fn onboarding_and_implementation_historical_roots_untouched() {
    let fixture = fixture_root("agent-maintenance-historical-roots");
    seed_publication_inputs(&fixture);

    let onboarding_root = fixture.join("docs/project_management/next/opencode-cli-onboarding");
    let implementation_closeout =
        fixture.join("docs/integrations/opencode/governance/seam-2-closeout.md");
    write_text(
        &onboarding_root.join("README.md"),
        "# Historical onboarding packet\n",
    );
    write_text(
        &implementation_closeout,
        "# Historical implementation closeout\n",
    );

    let onboarding_before =
        fs::read_to_string(onboarding_root.join("README.md")).expect("read onboarding");
    let implementation_before =
        fs::read_to_string(&implementation_closeout).expect("read implementation");

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &request_toml(
            "opencode",
            &["packet_doc_refresh", "capability_matrix_refresh"],
            false,
            &[],
        ),
    );

    let plan = build_refresh_plan(&fixture, Path::new(request_path)).expect("build refresh plan");
    apply_refresh_plan(&fixture, &plan).expect("apply refresh plan");

    let onboarding_after =
        fs::read_to_string(onboarding_root.join("README.md")).expect("read onboarding after");
    let implementation_after =
        fs::read_to_string(&implementation_closeout).expect("read implementation after");
    assert_eq!(
        onboarding_before, onboarding_after,
        "onboarding packet root must remain untouched"
    );
    assert_eq!(
        implementation_before, implementation_after,
        "historical implementation docs must remain untouched"
    );
}

#[test]
fn identical_replay_is_noop() {
    let fixture = fixture_root("agent-maintenance-replay-noop");
    seed_publication_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &request_toml(
            "opencode",
            &[
                "packet_doc_refresh",
                "capability_matrix_refresh",
                "release_doc_refresh",
            ],
            false,
            &[],
        ),
    );

    let first_plan =
        build_refresh_plan(&fixture, Path::new(request_path)).expect("build initial plan");
    let first = apply_refresh_plan(&fixture, &first_plan).expect("apply initial plan");
    assert_eq!(first.written, first_plan.files.len());

    let replay_plan =
        build_refresh_plan(&fixture, Path::new(request_path)).expect("build replay plan");
    assert_eq!(first_plan, replay_plan, "replay plan must stay stable");
    let replay = apply_refresh_plan(&fixture, &replay_plan).expect("apply replay plan");
    assert_eq!(
        replay.written, 0,
        "identical replay should not rewrite files"
    );
    assert_eq!(
        replay.identical,
        replay_plan.files.len(),
        "identical replay should classify every planned file as identical"
    );
}

#[test]
fn publication_refresh_actions_match_shared_publication_planner_bytes() {
    let fixture = fixture_root("agent-maintenance-publication-planner-parity");
    seed_publication_inputs(&fixture);
    normalize_support_matrix_fixture(&fixture);

    for (actions, support_enabled, capability_enabled) in [
        (&["support_matrix_refresh"][..], true, false),
        (&["capability_matrix_refresh"][..], false, true),
        (
            &["support_matrix_refresh", "capability_matrix_refresh"][..],
            true,
            true,
        ),
    ] {
        let request_path =
            "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
        write_text(
            &fixture.join(request_path),
            &request_toml("opencode", actions, false, &[]),
        );

        let plan =
            build_refresh_plan(&fixture, Path::new(request_path)).expect("build refresh plan");
        let maintenance_publication_files = plan
            .files
            .iter()
            .filter(|file| {
                matches!(
                    file.relative_path.as_str(),
                    publication_refresh::SUPPORT_MATRIX_JSON_OUTPUT_PATH
                        | publication_refresh::SUPPORT_MATRIX_MARKDOWN_OUTPUT_PATH
                        | publication_refresh::CAPABILITY_MATRIX_OUTPUT_PATH
                )
            })
            .map(|file| (file.relative_path.clone(), file.contents.clone()))
            .collect::<Vec<_>>();

        let shared_publication_files = publication_refresh::build_publication_artifact_plan(
            &fixture,
            support_enabled,
            capability_enabled,
        )
        .expect("build shared publication plan")
        .into_iter()
        .map(|file| (file.relative_path, file.contents))
        .collect::<Vec<_>>();

        assert_eq!(
            maintenance_publication_files, shared_publication_files,
            "maintenance refresh should reuse shared publication bytes for {:?}",
            actions
        );
    }
}

fn normalize_support_matrix_fixture(root: &Path) {
    for manifest_root in [
        "cli_manifests/codex",
        "cli_manifests/claude_code",
        "cli_manifests/opencode",
        "cli_manifests/gemini_cli",
        "cli_manifests/aider",
    ] {
        let current_path = root.join(manifest_root).join("current.json");
        let mut current: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&current_path).expect("read current.json"))
                .expect("parse current.json");
        let expected_targets = current["expected_targets"]
            .as_array()
            .expect("expected_targets array")
            .iter()
            .map(|value| value.as_str().expect("target string").to_string())
            .collect::<Vec<_>>();
        current["inputs"] = serde_json::Value::Array(
            expected_targets
                .iter()
                .map(|target| {
                    serde_json::json!({
                        "target_triple": target,
                        "binary": { "semantic_version": "1.0.0" }
                    })
                })
                .collect(),
        );
        write_text(
            &current_path,
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&current).expect("serialize current.json")
            ),
        );

        let version_path = root.join(manifest_root).join("versions/1.0.0.json");
        let mut version: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&version_path).expect("read version.json"))
                .expect("parse version.json");
        version["status"] = serde_json::Value::String("validated".to_string());
        write_text(
            &version_path,
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&version).expect("serialize version.json")
            ),
        );
    }
}

fn seed_publication_inputs(root: &Path) {
    seed_release_touchpoints(root);
    write_text(
        &root.join("docs/integrations/opencode/governance/seam-2-closeout.md"),
        "# Closeout\n\nThis stale capability claim triggered maintenance.\n",
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nManual contract text.\n",
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/capability-matrix.md"),
        "# Capability matrix\n\nStale publication.\n",
    );
    seed_publishable_workspace_member(root, "crates/gemini_cli", "unified-agent-api-gemini-cli");
    seed_cli_manifest_root(
        root,
        "cli_manifests/codex",
        &["x86_64-unknown-linux-musl"],
        &[
            (&["mcp", "list"], &["x86_64-unknown-linux-musl"]),
            (&["mcp", "get"], &["x86_64-unknown-linux-musl"]),
            (&["mcp", "add"], &["x86_64-unknown-linux-musl"]),
            (&["mcp", "remove"], &["x86_64-unknown-linux-musl"]),
        ],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/claude_code",
        &["linux-x64", "darwin-arm64", "win32-x64"],
        &[(
            &["mcp", "list"],
            &["linux-x64", "darwin-arm64", "win32-x64"],
        )],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/opencode",
        &["linux-x64", "darwin-arm64", "win32-x64"],
        &[],
    );
    seed_cli_manifest_root(root, "cli_manifests/gemini_cli", &["darwin-arm64"], &[]);
    seed_cli_manifest_root(root, "cli_manifests/aider", &["darwin-arm64"], &[]);
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
        "inputs": [{
            "target_triple": canonical_targets[0],
            "binary": { "semantic_version": "1.0.0" }
        }],
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
        "status": "latest_validated",
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
                serde_json::to_string_pretty(&report).expect("serialize support report")
            ),
        );
    }
}

fn request_toml(
    agent_id: &str,
    actions: &[&str],
    runtime_required: bool,
    runtime_items: &[&str],
) -> String {
    request_toml_with_refs(
        agent_id,
        "docs/integrations/opencode/governance/seam-2-closeout.md",
        "docs/integrations/opencode/governance/seam-2-closeout.md",
        actions,
        runtime_required,
        runtime_items,
    )
}

fn request_toml_with_refs(
    agent_id: &str,
    basis_ref: &str,
    opened_from: &str,
    actions: &[&str],
    runtime_required: bool,
    runtime_items: &[&str],
) -> String {
    let actions_block = actions
        .iter()
        .map(|action| format!("  \"{action}\","))
        .collect::<Vec<_>>()
        .join("\n");
    let runtime_items_block = if runtime_items.is_empty() {
        String::new()
    } else {
        runtime_items
            .iter()
            .map(|item| format!("  \"{item}\","))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        concat!(
            "artifact_version = \"1\"\n",
            "agent_id = \"{agent_id}\"\n",
            "trigger_kind = \"drift_detected\"\n",
            "basis_ref = \"{basis_ref}\"\n",
            "opened_from = \"{opened_from}\"\n",
            "requested_control_plane_actions = [\n",
            "{actions_block}\n",
            "]\n",
            "request_recorded_at = \"2026-04-22T01:15:00Z\"\n",
            "request_commit = \"1adb8f1\"\n",
            "\n",
            "[runtime_followup_required]\n",
            "required = {runtime_required}\n",
            "items = [\n",
            "{runtime_items_block}\n",
            "]\n"
        ),
        agent_id = agent_id,
        basis_ref = basis_ref,
        opened_from = opened_from,
        actions_block = actions_block,
        runtime_required = if runtime_required { "true" } else { "false" },
        runtime_items_block = runtime_items_block
    )
}

fn diff_paths(
    before: &std::collections::BTreeMap<String, Vec<u8>>,
    after: &std::collections::BTreeMap<String, Vec<u8>>,
) -> BTreeSet<String> {
    before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| before.get(path) != after.get(path))
        .collect()
}
