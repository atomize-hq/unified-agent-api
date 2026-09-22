use super::*;

#[test]
fn c0_validate_reports_wrapper_overlap_errors_with_required_fields_and_is_deterministic() {
    let temp = make_temp_dir("ccm-c0-validate-overlap");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    let wrapper_coverage = json!({
        "schema_version": 1,
        "generated_at": TS,
        "wrapper_version": "0.0.0-test",
        "coverage": [
            {
                "path": ["exec"],
                "level": "explicit",
                "scope": { "platforms": ["linux"] }
            },
            {
                "path": ["exec"],
                "level": "explicit",
                "scope": { "target_triples": [REQUIRED_TARGET] }
            }
        ]
    });
    write_json(&codex_dir.join("wrapper_coverage.json"), &wrapper_coverage);

    let a = run_xtask_validate(&codex_dir);
    let b = run_xtask_validate(&codex_dir);
    assert!(
        !a.status.success(),
        "expected overlap failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        a.status,
        String::from_utf8_lossy(&a.stdout),
        String::from_utf8_lossy(&a.stderr)
    );
    assert_eq!(
        a.stderr, b.stderr,
        "validator output must be deterministic for identical inputs"
    );

    let stderr = String::from_utf8_lossy(&a.stderr);
    assert!(
        stderr.contains("wrapper_coverage.json"),
        "expected wrapper_coverage.json path in errors, got:\n{stderr}"
    );
    assert!(
        stderr.contains(REQUIRED_TARGET),
        "expected target triple in overlap errors, got:\n{stderr}"
    );
    assert!(
        stderr.contains("exec"),
        "expected unit key (command path) in overlap errors, got:\n{stderr}"
    );
    assert!(
        stderr.contains("0") && stderr.contains("1"),
        "expected matching entry indexes mentioned in overlap errors, got:\n{stderr}"
    );
}

#[test]
fn c0_validate_rejects_pointer_files_without_trailing_newline() {
    let temp = make_temp_dir("ccm-c0-validate-pointer-newline");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    write_text(&codex_dir.join("latest_validated.txt"), VERSION);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected pointer format failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("latest_validated.txt"),
        "expected latest_validated.txt referenced in errors, got:\n{stderr}"
    );
}

/// Rewrite `RULES.json` in place, then run the validator and return its stderr.
fn validate_stderr_for_mutated_rules(codex_dir: &Path, mutate: impl FnOnce(&mut Value)) -> String {
    let rules_path = codex_dir.join("RULES.json");
    let mut rules: Value =
        serde_json::from_str(&fs::read_to_string(&rules_path).expect("read RULES.json"))
            .expect("parse RULES.json");
    mutate(&mut rules);
    write_json(&rules_path, &rules);

    let output = run_xtask_validate(codex_dir);
    assert!(
        !output.status.success(),
        "expected manifest-validate to reject the descriptor:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn c0_validate_rejects_a_descriptor_without_the_global_flags_model() {
    // A union-model agent that omits `globals` gets the merger's permissive default silently and
    // publishes one coverage row per repeated global flag. `uaa-0045` made that omission fatal
    // here rather than invisible there.
    let temp = make_temp_dir("ccm-c0-validate-globals-missing");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    let stderr = validate_stderr_for_mutated_rules(&codex_dir, |rules| {
        rules
            .as_object_mut()
            .expect("RULES.json is an object")
            .remove("globals")
            .expect("fixture descriptor declares globals");
    });

    assert!(
        stderr.contains("globals") && stderr.contains("RULES.json"),
        "stderr should name the missing key and the file; got:\n{stderr}"
    );
}

#[test]
fn c0_validate_rejects_a_dedupe_key_the_merger_would_refuse() {
    // One validation contract: a descriptor that passes `manifest-validate` must be one
    // `manifest-union` can actually run.
    let temp = make_temp_dir("ccm-c0-validate-dedupe-key");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    let stderr = validate_stderr_for_mutated_rules(&codex_dir, |rules| {
        rules["globals"]["effective_flags_model"]["union_normalization"]["dedupe_key"] =
            json!("long_name");
    });

    assert!(
        stderr.contains("dedupe_key=long_name"),
        "stderr should name the unsupported dedupe_key; got:\n{stderr}"
    );
}

#[test]
fn c0_validate_rejects_the_flags_model_when_the_root_command_is_parity_excluded() {
    // `uaa-0052`: the two policies cancel. The model relocates a global flag's coverage delta to
    // the root path; a root-command parity exclusion removes it there; and the union has already
    // dropped the subcommand copies. opencode shipped that way and 345 surfaces were reported
    // under no delta list at all, so the descriptor must not be able to hold both.
    let temp = make_temp_dir("ccm-c0-validate-root-excluded-flags-model");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    let stderr = validate_stderr_for_mutated_rules(&codex_dir, |rules| {
        rules["globals"]["effective_flags_model"]["enabled"] = json!(true);
        rules["parity_exclusions"]["units"]
            .as_array_mut()
            .expect("parity_exclusions.units is an array")
            .push(json!({
                "unit": "command",
                "path": [],
                "category": "interactive",
                "note": "Root command is the interactive TUI."
            }));
    });

    assert!(
        stderr.contains("root command parity-excluded"),
        "stderr should name the conflicting pair; got:\n{stderr}"
    );
}
