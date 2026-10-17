use super::*;

// Appended to the seeded registry, this table belongs to its last agent entry, `aider`.
const SUPPORT_DEPTH: &str = r#"
[agents.support_depth]
enabled_paths = ["maintenance"]

[[agents.support_depth.enrollments]]
lifecycle_path = "maintenance"
version = "1.2.3"
targets = ["linux-x64", "darwin-arm64"]

[[agents.support_depth.enrollments.modes]]
selector = "run-json"
command_path = "aider run"
mode = "headless"
predicates = [{ flag = "--output-format", value = "json" }]

[[agents.support_depth.enrollments.modes]]
selector = "tui"
command_path = "aider"
mode = "interactive"
targets = ["linux-x64"]

[[agents.support_depth.enrollments.operations]]
id = "run"
serves = ["run-json"]
templates = ["CORE", "SESSION"]
shared_mapping = "mapped"
forwarding = false
not_applicable_permitted = ["X1", "OUT3"]

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

[[agents.support_depth.enrollments.operations]]
id = "tui-excluded"
serves = ["tui"]
templates = ["EXCLUDED"]
rationale = "interactive only"

[[agents.support_depth.enrollments.capabilities]]
id = "agent_api.run"
operations = ["run"]
"#;

const EXTRA_MODE: &str = "\n[[agents.support_depth.enrollments.modes]]\nselector = \"extra\"\n\
                          command_path = \"aider extra\"\nmode = \"headless\"\n";

fn with_support_depth(edit: impl Fn(&str) -> String) -> Result<AgentRegistry, String> {
    AgentRegistry::parse_unenforced(&format!("{SEEDED_REGISTRY}{}", edit(SUPPORT_DEPTH)))
        .map_err(|err| err.to_string())
}

#[test]
fn seeded_registry_carries_no_support_depth() {
    let registry = AgentRegistry::parse(SEEDED_REGISTRY).expect("parse seeded registry");
    assert!(registry.agents.iter().all(|a| a.support_depth.is_none()));
    assert_eq!(registry.agents.last().unwrap().agent_id, "aider");
}

#[test]
fn support_depth_declaration_parses_but_no_command_loads_it() {
    let registry = with_support_depth(str::to_string).expect("parse support_depth");
    let table = registry
        .agents
        .last()
        .unwrap()
        .support_depth
        .as_ref()
        .unwrap();
    let declaration = &table.enrollments.as_ref().unwrap()[0];
    assert_eq!(declaration.version, "1.2.3");
    assert_eq!(declaration.operations[0].obligations().len(), 12);
    assert_eq!(declaration.operations[1].obligations().len(), 1);

    let refused = AgentRegistry::parse(&format!("{SEEDED_REGISTRY}{SUPPORT_DEPTH}"));
    let message = refused.unwrap_err().to_string();
    assert!(message.contains("no command enforces yet"), "{message}");
}

#[test]
fn support_depth_templates_match_the_contract_table() {
    use agent_registry::support_depth::{ObligationId, ObligationTemplate};
    // docs/specs/support-depth-contract.md, "Obligation templates".
    let table = [
        ("CORE", "OUT3 LC2"),
        ("SESSION", "S1"),
        ("QUERY", "OUT2 LC1"),
        ("ADMIN", "OUT1 LC1 FX1"),
        ("DIAG", "OUT1 LC1 B1"),
        ("OPAQUE", "OUT1 LC1"),
        ("LAUNCH", "OUT4 LC3"),
        ("MANAGED", "OUT4 OUT3 LC4"),
        ("MCP", "MCP1"),
        ("REMOTE", "N1"),
        ("EXCLUDED", "EX1"),
    ];
    let id = |name: &str| -> ObligationId {
        serde_json::from_value(serde_json::Value::String(name.to_string())).expect(name)
    };
    for (template, obligations) in table {
        let template: ObligationTemplate =
            serde_json::from_value(serde_json::Value::String(template.to_string())).unwrap();
        let expected: Vec<_> = obligations.split(' ').map(id).collect();
        assert_eq!(template.obligations(), expected.as_slice(), "{template:?}");
    }
    for base in "R1 V1 G1 E1 A1 X1 X2 X3 X4".split(' ') {
        assert!(ObligationId::BASE.contains(&id(base)));
    }
}

#[test]
fn support_depth_accepts_what_the_registry_alone_does_not_decide() {
    let cases: &[(&str, &str)] = &[
        // Rule 8: a declaration on a path `enabled_paths` does not list is still a declaration.
        (
            "enabled_paths = [\"maintenance\"]",
            "enabled_paths = [\"onboarding\"]",
        ),
        ("enabled_paths = [\"maintenance\"]\n", ""),
        // One command path under two mode names.
        ("command_path = \"aider\"", "command_path = \"aider run\""),
        (
            "targets = [\"linux-x64\"]",
            "targets = [\"linux-x64\", \"darwin-arm64\"]",
        ),
        // Whether forwarding suffices for a template other than `MANAGED` is not decided here.
        ("forwarding = false", "forwarding = true"),
    ];
    for (from, to) in cases {
        assert_eq!(SUPPORT_DEPTH.matches(from).count(), 1, "ambiguous `{from}`");
        with_support_depth(|table| table.replace(from, to))
            .unwrap_or_else(|err| panic!("`{to}` must load: {err}"));
    }
    let paths_only = "\n[agents.support_depth]\nenabled_paths = [\"onboarding\"]\n";
    with_support_depth(|_| paths_only.to_string()).expect("paths only");
}

#[test]
fn support_depth_rejects_invalid_tables() {
    const TARGETS: &str = "targets = [\"linux-x64\", \"darwin-arm64\"]";
    const PATHS: &str = "enabled_paths = [\"maintenance\"]";
    const MODE_TARGETS: &str = "targets = [\"linux-x64\"]";
    const RUN_SERVES: &str = "serves = [\"run-json\"]";
    const TUI_SERVES: &str = "serves = [\"tui\"]";
    const TEMPLATES: &str = "templates = [\"CORE\", \"SESSION\"]";
    const NA: &str = "not_applicable_permitted = [\"X1\", \"OUT3\"]";
    const CAP_OPS: &str = "operations = [\"run\"]";
    const PREDICATES: &str = "predicates = [{ flag = \"--output-format\", value = \"json\" }]";
    const EXCLUDED: &str = "templates = [\"EXCLUDED\"]";
    let cases: &[(&str, &str, &str)] = &[
        (PATHS, "enabled_paths = []", "must be omitted when no path"),
        (PATHS, "enabled_paths = [\"maintenance\", \"maintenance\"]", "must not repeat"),
        (PATHS, "enabled_paths = [\"nightly\"]", "unknown variant"),
        ("lifecycle_path = \"maintenance\"", "lifecycle_path = \"nightly\"", "unknown variant"),
        (PATHS, "enabled_paths = [\"maintenance\"]\noverrides = []", "unknown field"),
        ("version = \"1.2.3\"", "version = \"v1.2.3\"", "exact semantic version"),
        ("version = \"1.2.3\"", "version = \"^1.2\"", "exact semantic version"),
        ("version = \"1.2.3\"", "version = \"1.2.3-rc.1\"", "exact semantic version"),
        ("version = \"1.2.3\"", "version = \"1.2.3+build\"", "exact semantic version"),
        (TARGETS, "targets = []", "at least one entry"),
        (TARGETS, "targets = [\"linux-x64\", \"linux-x64\"]", "without repeats"),
        (TARGETS, "targets = [\"linux-x64\", \" \"]", "hold whitespace"),
        (TARGETS, "targets = [\"linux-x64\", \"\"]", "may be empty"),
        (PREDICATES, "targets = [\"linux-x64\"]", "no mode is selected on"),
        (TARGETS, "targets = [\"linux-x64\", \"darwin-arm64\"]\nunknown = 1", "unknown field"),
        (MODE_TARGETS, "targets = [\"win32-x64\"]", "do not list"),
        (MODE_TARGETS, "targets = []", "at least one entry"),
        (MODE_TARGETS, "targets = [\"linux-x64\", \"linux-x64\"]", "without repeats"),
        (MODE_TARGETS, "targets = [\"linux-x64\"]\nexcluded = true", "unknown field"),
        ("selector = \"tui\"", "selector = \"Tui\"", "lowercase"),
        ("selector = \"tui\"", "selector = \"\"", "lowercase"),
        ("selector = \"tui\"", "selector = \"tuI\"", "lowercase"),
        ("selector = \"tui\"", "selector = \"t ui\"", "lowercase"),
        ("selector = \"tui\"", "selector = \"run-json\"", "declared more than once"),
        ("mode = \"interactive\"", "mode = \"Interactive\"", "lowercase"),
        ("command_path = \"aider\"", "command_path = \"aider \"", "single spaces"),
        ("command_path = \"aider\"", "command_path = \"aider\\trun\"", "single spaces"),
        ("command_path = \"aider\"", "command_path = \"aider run\\tx\"", "single spaces"),
        ("command_path = \"aider\"", "command_path = \"claude\"", "must be `aider`"),
        ("command_path = \"aider\"", "command_path = \"aider --json\"", "none of them a flag"),
        (PREDICATES, "predicates = [{ flag = \"output-format\" }]", "start with `-`"),
        (PREDICATES, "predicates = [{ flag = \"--\" }]", "name a flag"),
        (PREDICATES, "predicates = [{ flag = \"--a=b\" }]", "whitespace or `=`"),
        (PREDICATES, "predicates = [{ flag = \"--a b\" }]", "whitespace or `=`"),
        (PREDICATES, "predicates = [{ flag = \"--a\", value = \" \" }]", "must not be empty"),
        (PREDICATES, "predicates = [{ flag = \"--a\" }, { flag = \"--a\", value = \"1\" }]", "more than once"),
        (PREDICATES, "predicates = [{ flag = \"--a\", absent = true }]", "unknown field"),
        ("id = \"run\"", "id = \"Run\"", "lowercase"),
        ("id = \"run\"", "id = \"tui-excluded\"", "declared more than once"),
        (RUN_SERVES, "serves = []", "at least one entry"),
        (RUN_SERVES, "serves = [\"run-json\", \"run-json\"]", "without repeats"),
        (RUN_SERVES, "serves = [\"nope\"]", "undeclared mode"),
        (RUN_SERVES, "serves = [\"run-json\", \"tui\"]", "excluded and also served"),
        (TUI_SERVES, "serves = [\"tui\", \"run-json\"]", "exactly one mode"),
        (TEMPLATES, "templates = []", "at least one template"),
        (TEMPLATES, "templates = [\"CORE\", \"CORE\"]", "without repeats"),
        (TEMPLATES, "templates = [\"SESSION\"]", "output obligation"),
        (TEMPLATES, "templates = [\"CORE\", \"TUI\"]", "unknown variant"),
        (TEMPLATES, "templates = [\"CORE\"]\nrationale = \"x\"", "mode exclusion only"),
        (TEMPLATES, "templates = [\"CORE\"]\noverride = \"x\"", "unknown field"),
        (EXCLUDED, "templates = [\"EXCLUDED\", \"CORE\"]", "mode exclusion"),
        (EXCLUDED, "templates = [\"EXCLUDED\"]\nshared_mapping = \"wrapper_only\"", "mode exclusion"),
        (EXCLUDED, "templates = [\"EXCLUDED\"]\nforwarding = false", "mode exclusion"),
        (EXCLUDED, "templates = [\"EXCLUDED\"]\nnot_applicable_permitted = [\"EX1\"]", "mode exclusion"),
        (EXCLUDED, "templates = [\"EXCLUDED\"]\npromise = { r1 = \"r\", v1 = \"v\", g1 = \"g\", e1 = \"e\", a1 = \"a\", x1 = \"x\", x2 = \"x\", x3 = \"x\", x4 = \"x\" }", "mode exclusion"),
        ("rationale = \"interactive only\"\n", "", "rationale is required"),
        ("rationale = \"interactive only\"", "rationale = \" \"", "must not be empty"),
        (NA, "not_applicable_permitted = [\"FX1\"]", "got `FX1`"),
        (NA, "not_applicable_permitted = [\"X1\", \"X1\"]", "without repeats"),
        (NA, "not_applicable_permitted = [\"M\"]", "unknown variant"),
        ("shared_mapping = \"mapped\"\n", "", "shared_mapping is required"),
        ("forwarding = false\n", "", "forwarding is required"),
        ("forwarding = false", "forwarding = \"no\"", "invalid type"),
        (
            "templates = [\"CORE\", \"SESSION\"]\nshared_mapping = \"mapped\"\nforwarding = false",
            "templates = [\"MANAGED\"]\nshared_mapping = \"mapped\"\nforwarding = true",
            "cannot be true for a promise that selects `MANAGED`",
        ),
        ("shared_mapping = \"mapped\"", "shared_mapping = \"wrapper_only\"", "not a `mapped` operation"),
        ("x4 = \"x\"", "x4 = \"\"", "must not be empty"),
        ("r1 = \"r\"", "r1 = \"\"", "must not be empty"),
        ("v1 = \"v\"", "v1 = \"\"", "must not be empty"),
        ("g1 = \"g\"", "g1 = \"\"", "must not be empty"),
        ("e1 = \"e\"", "e1 = \"\"", "must not be empty"),
        ("a1 = \"a\"", "a1 = \"\"", "must not be empty"),
        ("x1 = \"x\"", "x1 = \" \"", "must not be empty"),
        ("x2 = \"x\"", "x2 = \" \"", "must not be empty"),
        ("x3 = \"x\"", "x3 = \" \"", "must not be empty"),
        ("x4 = \"x\"\n", "", "missing field `x4`"),
        ("x4 = \"x\"", "x4 = \"x\"\nm = \"x\"", "unknown field"),
        ("id = \"agent_api.run\"", "id = \"Agent API\"", "lowercase"),
        (CAP_OPS, "operations = []", "at least one entry"),
        (CAP_OPS, "operations = [\"run\", \"run\"]", "without repeats"),
        (CAP_OPS, "operations = [\"nope\"]", "undeclared operation"),
        (CAP_OPS, "operations = [\"tui-excluded\"]", "not a `mapped` operation"),
        (CAP_OPS, "operations = [\"run\"]\nclaimed = true", "unknown field"),
    ];
    let mut failures = Vec::new();
    for (from, to, expected) in cases {
        assert_eq!(SUPPORT_DEPTH.matches(from).count(), 1, "ambiguous `{from}`");
        match with_support_depth(|table| table.replace(from, to)) {
            Ok(_) => failures.push(format!("`{to}` loaded")),
            Err(err) if !err.contains(expected) => {
                failures.push(format!("`{to}`: expected `{expected}` in `{err}`"))
            }
            Err(err) => assert!(
                err.contains("agent `aider`") || err.contains("TOML"),
                "{err}"
            ),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));

    // Whole-table cases.
    let declaration = SUPPORT_DEPTH
        .split_once("[[agents.support_depth.enrollments]]")
        .unwrap()
        .1;
    let whole: &[(String, &str)] = &[
        ("\n[agents.support_depth]\n".into(), "must be omitted"),
        (
            "\n[agents.support_depth]\nenrollments = []\n".into(),
            "no declaration",
        ),
        (
            SUPPORT_DEPTH
                .split("[[agents.support_depth.enrollments.capabilities]]")
                .next()
                .unwrap()
                .to_string(),
            "no capability mapping names it",
        ),
        (
            format!("{SUPPORT_DEPTH}{EXTRA_MODE}"),
            "served by no operation",
        ),
        (
            SUPPORT_DEPTH
                .replace("[agents.support_depth.enrollments.operations.promise]", "")
                .replace(
                    "r1 = \"r\"\nv1 = \"v\"\ng1 = \"g\"\ne1 = \"e\"\na1 = \"a\"\n",
                    "",
                )
                .replace("x1 = \"x\"\nx2 = \"x\"\nx3 = \"x\"\nx4 = \"x\"\n", ""),
            "promise is required",
        ),
        (
            SUPPORT_DEPTH
                .replace("command_path = \"aider\"", "command_path = \"aider run\"")
                .replace("mode = \"interactive\"", "mode = \"headless\""),
            "overlaps another selector",
        ),
        // Rule 6 holds across lifecycle paths.
        (
            format!(
                "{SUPPORT_DEPTH}\n[[agents.support_depth.enrollments]]{}",
                declaration.replace("\"maintenance\"", "\"onboarding\"")
            ),
            "more than once",
        ),
        (
            format!(
                "{SUPPORT_DEPTH}\n[[agents.support_depth.enrollments.capabilities]]\n\
                 id = \"agent_api.run\"\noperations = [\"run\"]\n"
            ),
            "declared more than once",
        ),
        // Only mode exclusions: nothing is selected.
        (
            SUPPORT_DEPTH
                .split("[[agents.support_depth.enrollments.modes]]\nselector = \"tui\"")
                .next()
                .unwrap()
                .replace(PREDICATES, "")
                + "\n[[agents.support_depth.enrollments.operations]]\nid = \"x\"\n\
                   serves = [\"run-json\"]\ntemplates = [\"EXCLUDED\"]\nrationale = \"r\"\n",
            "not a mode exclusion",
        ),
        (
            "\n[agents.support_depth]\n[[agents.support_depth.enrollments]]\n\
             lifecycle_path = \"maintenance\"\nversion = \"1.2.3\"\ntargets = [\"a\"]\n\
             modes = []\noperations = []\n"
                .into(),
            "at least one mode",
        ),
    ];
    for (table, expected) in whole {
        let err = with_support_depth(|_| table.clone()).expect_err(expected);
        assert!(err.contains(expected), "expected `{expected}` in `{err}`");
    }
}
