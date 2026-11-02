use super::*;

// A wrapper-only agent that is not an agent_api runtime backend: no `backend_module`, no claims.
const NON_RUNTIME: &str = r#"
[[agents]]
agent_id = "depth_fixture"
display_name = "Depth Fixture"
crate_path = "crates/depth_fixture"
manifest_root = "cli_manifests/depth_fixture"
package_name = "unified-agent-api-depth-fixture"
canonical_targets = ["linux-x64"]

[agents.wrapper_coverage]
binding_kind = "generated_from_wrapper_crate"
source_path = "crates/depth_fixture"

[agents.capability_declaration]
always_on = []
backend_extensions = []

[agents.publication]
support_matrix_enabled = false
capability_matrix_enabled = false

[agents.release]
docs_release_track = "crates-io"

[agents.scaffold]
onboarding_pack_prefix = "depth-fixture-cli-onboarding"
"#;

// Appended after NON_RUNTIME, this table belongs to `depth_fixture`.
const WRAPPER_ONLY_DEPTH: &str = r#"
[agents.support_depth]
enabled_paths = ["maintenance"]

[[agents.support_depth.enrollments]]
lifecycle_path = "maintenance"
version = "0.1.0"
targets = ["linux-x64"]

[[agents.support_depth.enrollments.modes]]
selector = "run"
command_path = "depth_fixture run"
mode = "headless"

[[agents.support_depth.enrollments.operations]]
id = "run"
serves = ["run"]
templates = ["CORE"]
shared_mapping = "wrapper_only"
forwarding = false

[agents.support_depth.enrollments.operations.promise]
r1 = "r"
v1 = "v"
g1 = "g"
e1 = "e"
a1 = "a"
x1 = "x"
x2 = "x"
x3 = "x"
x4 = "x"
"#;

fn with_non_runtime(extra: &str) -> String {
    format!("{SEEDED_REGISTRY}{NON_RUNTIME}{extra}")
}

fn err(raw: &str) -> String {
    AgentRegistry::parse_unenforced(raw)
        .expect_err("registry must be rejected")
        .to_string()
}

#[test]
fn non_runtime_entry_parses_without_backend_module() {
    let registry = AgentRegistry::parse(&with_non_runtime("")).expect("parse");
    let entry = registry.find("depth_fixture").expect("entry");
    assert_eq!(entry.backend_module, None);
    assert_eq!(entry.backend_module_or_empty(), "");
    assert_eq!(
        registry
            .agents
            .iter()
            .filter(|a| a.backend_module.is_some())
            .count(),
        agent_registry::BUILT_IN_AGENTS.len()
    );
}

#[test]
fn non_runtime_entry_admits_wrapper_only_depth() {
    AgentRegistry::parse_unenforced(&with_non_runtime(WRAPPER_ONLY_DEPTH)).expect("parse");
}

#[test]
fn non_runtime_entry_refuses_every_capability_claim() {
    let mapped = WRAPPER_ONLY_DEPTH.replace("\"wrapper_only\"", "\"mapped\"")
        + "\n[[agents.support_depth.enrollments.capabilities]]\nid = \"agent_api.run\"\noperations = [\"run\"]\n";
    let cases = [
        (
            with_non_runtime("").replacen(
                "always_on = []\nbackend_extensions = []\n\n[agents.publication]\nsupport_matrix_enabled = false",
                "always_on = [\"agent_api.run\"]\nbackend_extensions = []\n\n[agents.publication]\nsupport_matrix_enabled = false",
                1,
            ),
            "declare capabilities",
        ),
        (
            with_non_runtime("").replacen(
                "backend_extensions = []\n\n[agents.publication]\nsupport_matrix_enabled = false",
                "backend_extensions = [\"depth_fixture.extra\"]\n\n[agents.publication]\nsupport_matrix_enabled = false",
                1,
            ),
            "declare capabilities",
        ),
        (
            with_non_runtime("").replacen(
                "support_matrix_enabled = false\ncapability_matrix_enabled = false",
                "support_matrix_enabled = false\ncapability_matrix_enabled = true",
                1,
            ),
            "enable capability publication",
        ),
        (with_non_runtime(&mapped), "map support-depth capabilities"),
        (
            with_non_runtime(
                &WRAPPER_ONLY_DEPTH.replace("\"wrapper_only\"", "\"proposed_extension\""),
            ),
            "`proposed_extension` operation",
        ),
    ];
    for (raw, expected) in cases {
        let message = err(&raw);
        assert!(
            message.contains("depth_fixture") && message.contains(expected),
            "expected `{expected}`, got: {message}"
        );
    }
}

#[test]
fn built_in_cannot_declare_itself_non_runtime() {
    let raw = SEEDED_REGISTRY.replacen(
        "backend_module = \"crates/agent_api/src/backends/codex\"\n",
        "",
        1,
    );
    let message = err(&raw);
    assert!(
        message.contains("`codex`") && message.contains("compiled built-in backend"),
        "{message}"
    );
}
