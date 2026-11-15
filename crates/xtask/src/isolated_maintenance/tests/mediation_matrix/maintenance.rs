//! Public MA intake/apply boundaries, using an otherwise working ordinary maintenance packet.
use super::*;
use crate::agent_maintenance::{
    closeout, execute,
    prepare::{apply_prepare_plan, build_prepare_plan},
    prepare_closeout, refresh,
};

const REQUEST: &str =
    "docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml";
const CLOSEOUT: &str =
    "docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-closeout.json";

#[test]
fn ma_intake_and_apply_refuse_onboarding_and_later_version_displacement() {
    let (f, mut args) = super::super::prepare::workspace();
    let root = f.root.path();
    f.registry(|doc| {
        doc["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap()
            .remove("support_depth")
            .unwrap();
    });
    super::super::prepare::acquire(root, "0.2.0");
    f.commit();
    // Ordinary MA1/MA2 and MA3 planning must reach their actual normal entry points.
    let ordinary = build_prepare_plan(root, &args).unwrap();
    apply_prepare_plan(root, &ordinary).unwrap();
    use sha2::{Digest, Sha256};
    let closeout = serde_json::to_vec_pretty(&json!({
        "request_ref": REQUEST,
        "request_sha256": hex::encode(Sha256::digest(fs::read(root.join(REQUEST)).unwrap())),
        "resolved_findings": [], "explicit_none_reason": "No findings in the ordinary matrix control",
        "preflight_passed": true, "recorded_at": "2026-11-05T00:00:00Z",
        "commit": git(root, &["rev-parse", "HEAD"]),
    })).unwrap();
    closeout::check_closeout_form(std::str::from_utf8(&closeout).unwrap()).unwrap();
    write(root, CLOSEOUT, closeout);
    f.commit();
    let cached = build_prepare_plan(root, &args).unwrap();
    let refresh_plan = refresh::build_refresh_plan(root, Path::new(REQUEST)).unwrap();
    assert!(!cached.files.is_empty());
    assert!(!refresh_plan.files.is_empty());
    refresh::apply_refresh_plan(root, &refresh_plan).unwrap();
    let head = git(root, &["rev-parse", "HEAD"]);
    git(root, &["update-ref", "refs/remotes/origin/staging", &head]);
    let promotion = crate::manifest_promotion::Args {
        agent: "depth_fixture".into(),
        version: "0.2.0".into(),
        passed_targets: vec!["x86_64-unknown-linux-gnu".into()],
        validation_commit: head,
        write: true,
    };
    // The ordinary candidate is valid before inserting the retained onboarding record
    // with its lifecycle binding absent. Pin the resulting canonical binding violation.
    crate::manifest_promotion::build_plan(root, &promotion).unwrap();
    let ordinary_registry = fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH)).unwrap();
    let onboarding = AgentRegistry::parse_unenforced(DECLARED).unwrap();
    let table =
        toml_edit::ser::to_document(onboarding.agents[0].support_depth.as_ref().unwrap()).unwrap();
    let record = guard_record::record();
    for carrier in ["declared", "omitted", "disabled", "removed_record"] {
        write(root, REGISTRY_RELATIVE_PATH, &ordinary_registry);
        if carrier != "omitted" {
            f.registry(|doc| {
                let entry = doc["agents"]
                    .as_array_of_tables_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
                    .unwrap();
                entry.insert(
                    "support_depth",
                    toml_edit::Item::Table(table.as_table().clone()),
                );
                if carrier == "disabled" {
                    entry["support_depth"]
                        .as_table_mut()
                        .unwrap()
                        .remove("enabled_paths")
                        .unwrap();
                }
            });
        }
        if carrier == "removed_record" {
            fs::remove_file(root.join(RECORD)).unwrap();
        } else {
            write(root, RECORD, &record);
        }
        f.commit();
        let reason = if carrier == "omitted" {
            "production depth preparation is not enabled: retained record cli_manifests/depth_fixture/reports/0.2.0/depth-record.json"
        } else {
            "carries `support_depth`, which no command enforces yet"
        };
        for version in ["0.2.0", "0.3.0"] {
            args.target_version = version.into();
            unchanged(root, reason, || {
                build_prepare_plan(root, &args).err().unwrap().to_string()
            });
        }
        unchanged(root, reason, || {
            apply_prepare_plan(root, &cached).unwrap_err().to_string()
        });
        let guard = if carrier == "omitted" {
            "unsupported depth write (retained record)"
        } else {
            "unsupported depth write (declared enrollment)"
        };
        unchanged(root, &format!("refresh apply: {guard}"), || {
            refresh::apply_refresh_plan(root, &refresh_plan)
                .unwrap_err()
                .to_string()
        });
        let request_reason = if carrier == "omitted" {
            "support_depth: missing policy for retained depth record"
        } else {
            "carries `support_depth`, which no command enforces yet"
        };
        unchanged(root, request_reason, || {
            execute::run_in_workspace(
                root,
                execute::Args {
                    request: REQUEST.into(),
                    dry_run: false,
                    write: true,
                    run_id: Some("s11-matrix".into()),
                    codex_binary: Some("absent-executor".into()),
                },
                &mut Vec::new(),
            )
            .unwrap_err()
            .to_string()
        });
        unchanged(root, request_reason, || {
            closeout::write_closeout_outputs(root, Path::new(REQUEST), Path::new(CLOSEOUT))
                .unwrap_err()
                .to_string()
        });
        unchanged(root, request_reason, || {
            closeout::run_in_workspace(
                root,
                closeout::Args {
                    request: REQUEST.into(),
                    closeout: CLOSEOUT.into(),
                },
                &mut Vec::new(),
            )
            .unwrap_err()
            .to_string()
        });
        unchanged(root, request_reason, || {
            prepare_closeout::run_in_workspace_with_fetcher(
                root,
                prepare_closeout::Args {
                    request: REQUEST.into(),
                    commit: git(root, &["rev-parse", "HEAD"]),
                    recorded_at: "2026-11-05T00:00:00Z".into(),
                    write: true,
                },
                &mut Vec::new(),
                |_| panic!("onboarding refusal reached a network fetch"),
            )
            .unwrap_err()
            .to_string()
        });
        let head = git(root, &["rev-parse", "HEAD"]);
        git(root, &["update-ref", "refs/remotes/origin/staging", &head]);
        let promotion_reason = if carrier == "omitted" {
            "DEPTH_LIFECYCLE_BINDING_MISSING"
        } else {
            request_reason
        };
        unchanged(root, promotion_reason, || {
            crate::manifest_promotion::build_plan(
                root,
                &crate::manifest_promotion::Args {
                    agent: "depth_fixture".into(),
                    version: "0.2.0".into(),
                    passed_targets: vec!["x86_64-unknown-linux-gnu".into()],
                    validation_commit: head,
                    write: true,
                },
            )
            .err()
            .unwrap()
        });
    }
}

#[test]
fn on2_dry_run_depth_approval_cannot_borrow_a_maintenance_route() {
    use clap::Parser;
    let (f, _) = super::super::prepare::workspace();
    let root = f.root.path();
    let approval = "docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml";
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(approval);
    let raw = format!(
        "{}\n[support_depth]\nversion = \"0.2.0\"\n",
        fs::read_to_string(source).unwrap()
    );
    write(root, approval, raw);
    let comparison = "docs/agents/selection/cli-agent-selection-packet.md";
    write(
        root,
        comparison,
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(comparison),
        )
        .unwrap(),
    );
    f.commit();
    let _installed = f.install();
    // A structurally valid foreign depth approval establishes no owning ON2 route.
    crate::approval_artifact::load_approval_artifact(root, approval).unwrap();
    unchanged(
        root,
        "runtime_follow_on: unsupported depth write (declared enrollment)",
        || {
            crate::runtime_follow_on::run_in_workspace(
                root,
                crate::runtime_follow_on::Args::parse_from([
                    "runtime-follow-on",
                    "--dry-run",
                    "--approval",
                    approval,
                    "--run-id",
                    "s11-foreign-route",
                ]),
                &mut Vec::new(),
            )
            .unwrap_err()
            .to_string()
        },
    );
}
