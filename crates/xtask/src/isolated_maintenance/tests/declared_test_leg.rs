//! Credential-free reader for the committed, depth-declared workspace test leg.
use super::{fixture::*, *};

#[test]
#[ignore = "launched explicitly by candidate-registry integration helpers"]
fn authenticated_registry_child() {
    let root = std::env::var_os("S10_DECLARED_REGISTRY_ROOT").expect("test-owned candidate root");
    let root = Path::new(&root);
    assert!(fixture::installed().is_none());
    assert!(route(root).unwrap().is_none());
    let head = git(root, &["rev-parse", "HEAD"]);
    let api = Arc::new(Fake::default());
    for (name, id, owner) in [("fixture/isolated", 10, 20), ("fixture/canonical", 30, 40)] {
        api.put(
            &format!("repos/{name}"),
            json!({"id":id,"owner":{"id":owner},"full_name":name}),
        );
        api.put(
            &format!("repos/{name}/branches/staging"),
            json!({"name":"staging","protected":true,"commit":{"sha":head}}),
        );
    }
    api.put(
        &format!("repos/fixture/isolated/compare/{head}...{head}"),
        json!({"status":"identical","merge_base_commit":{"sha":head}}),
    );
    api.put(
        &format!("repos/fixture/isolated/commits/{head}"),
        json!({"sha":head}),
    );
    let blob = git(
        root,
        &["rev-parse", &format!("HEAD:{REGISTRY_RELATIVE_PATH}")],
    );
    api.put(
        &format!("repos/fixture/isolated/contents/{REGISTRY_RELATIVE_PATH}?ref={head}"),
        json!({"path":REGISTRY_RELATIVE_PATH,"type":"file","sha":blob}),
    );
    let producer = git(root, &["rev-parse", "HEAD:producer/source.rs"]);
    api.put(
        &format!("repos/fixture/canonical/git/trees/{head}?recursive=1"),
        json!({"truncated":false,"tree":[
            {"path":"producer","type":"tree","mode":"040000","sha":"unused"},
            {"path":"producer/source.rs","type":"blob","mode":"100644","sha":producer}
        ]}),
    );
    let _installed = install(Box::new(move || {
        (profile_with(head.clone(), &scope::SYNTHETIC), api.clone())
    }));
    let route = route(root)
        .unwrap()
        .expect("independently installed fixture route");
    let registry = route.registry(root).unwrap();
    assert!(registry
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_some());
    let raw = fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH)).unwrap();
    assert!(AgentRegistry::parse(&raw)
        .unwrap_err()
        .to_string()
        .contains("agent `depth_fixture` carries `support_depth`, which no command enforces yet"));
    assert_eq!(AgentRegistry::parse_unenforced(&raw).unwrap(), *registry);
    let result = match std::env::var("S10_DECLARED_REGISTRY_OPERATION")
        .unwrap()
        .as_str()
    {
        "registry" => raw,
        "capability" => crate::capability_matrix::generate_markdown_for_workspace(root).unwrap(),
        "runtime-support-data" => {
            crate::support_matrix::render_agent_api_runtime_support_data(root).unwrap()
        }
        "runtime-check" => {
            crate::runtime_advertising::run_in_workspace(
                root,
                crate::runtime_advertising::Args { check: true },
            )
            .unwrap();
            "validated".to_string()
        }
        "claude-lifecycle" => {
            let entry = registry.find("claude_code").unwrap();
            crate::agent_lifecycle::load_lifecycle_state(
                root,
                &crate::agent_lifecycle::lifecycle_state_path_for_entry(entry),
            )
            .unwrap();
            "validated".to_string()
        }
        other => panic!("unsupported test operation {other}"),
    };
    route.recheck(root).unwrap();
    println!("S10_AUTHENTICATED_RESULT={}", hex::encode(result));
}
