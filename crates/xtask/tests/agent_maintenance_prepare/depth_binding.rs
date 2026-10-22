use super::*;

#[test]
fn depth_request_rendering_adds_only_the_binding_table_and_from_request_stays_transport() {
    let fixture = fixture_root("prepare-depth-binding-render");
    seed_registry(&fixture);
    seed_support_files(&fixture);
    let args = args();
    let plan = build_prepare_plan(&fixture, &args).unwrap();
    assert!(plan.request.support_depth.is_none());
    let registry = agent_registry::AgentRegistry::load(&fixture).unwrap();
    let request = &plan.request;
    let contract = contract_policy::build_execution_contract(
        &fixture,
        registry.find("codex").unwrap(),
        &request.relative_path,
        &request.maintenance_root,
        &request.opened_from,
        &args.target_version,
        &args.branch_name,
    )
    .unwrap();
    let ordinary = String::from_utf8(
        plan.files
            .iter()
            .find(|f| f.relative_path == request.relative_path)
            .unwrap()
            .contents
            .clone(),
    )
    .unwrap();
    assert!(!ordinary.contains("[support_depth]"));
    for with_o in [false, true] {
        let binding = request::SupportDepthBinding {
            policy_identity: "a".repeat(64),
            obligations_identity: with_o.then(|| "b".repeat(64)),
        };
        let rendered = prepare::render_request_toml(
            &args,
            &request.basis_ref,
            request.detected_release.as_ref().unwrap(),
            request.support_surface_audit.as_ref().unwrap(),
            &contract,
            Some(&binding),
        )
        .unwrap();
        let fragment = binding.render().unwrap();
        assert_eq!(rendered.replace(&fragment, ""), ordinary);
        let doc: toml_edit::DocumentMut = rendered.parse().unwrap();
        assert_eq!(doc["artifact_version"].as_str(), Some("2"));
        assert_eq!(
            doc["support_depth"].as_table().unwrap().len(),
            if with_o { 2 } else { 1 }
        );
        let file = fixture.join(&request.relative_path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, rendered).unwrap();
        // This reader transports Event metadata; it does not validate enrollment/bindings.
        let recovered = prepare::args_from_request_in_workspace(
            &fixture,
            Path::new(&request.relative_path),
            true,
            false,
        )
        .unwrap();
        let mut expected = args.clone();
        expected.dispatch_workflow = Some(
            request
                .detected_release
                .as_ref()
                .unwrap()
                .dispatch_workflow
                .clone(),
        );
        assert_eq!(recovered, expected);
        let error = request::load_request_envelope(&fixture, Path::new(&request.relative_path))
            .unwrap_err()
            .to_string();
        assert!(error.contains("table must be absent"), "{error}");
    }
}
