use super::*;

#[test]
fn request_outside_maintenance_root_rejected() {
    let fixture = fixture_root("agent-maintenance-request-path");
    seed_publication_inputs(&fixture);

    let invalid_request =
        "docs/agents/lifecycle/opencode-cli-onboarding/governance/maintenance-request.toml";
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
