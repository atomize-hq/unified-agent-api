use super::*;

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
