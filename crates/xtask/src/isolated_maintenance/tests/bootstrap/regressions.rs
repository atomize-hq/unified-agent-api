use super::*;

/// Exercise ON1's public dry-run and extract its ordinary renderer's exact bytes. No copy of
/// the bootstrap JSON shape lives in this fixture, and no production visibility is widened.
/// Runs on a fresh thread: the cfg(test) route is thread-local, so a caller's installed fixture
/// route must not leak into this ordinary (route-absent) render, whatever the test order.
pub(super) fn ordinary_current() -> &'static str {
    static CURRENT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    CURRENT.get_or_init(|| std::thread::spawn(ordinary_render).join().unwrap())
}

fn ordinary_render() -> String {
    {
        use clap::Parser;
        let root = tempfile::tempdir().unwrap();
        write(
            root.path(),
            REGISTRY_RELATIVE_PATH,
            include_str!("../../../../data/agent_registry.toml"),
        );
        write(root.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
        write(root.path(), "docs/crates-io-release.md", "# Releases\n");
        let args = crate::onboard_agent::Args::parse_from([
            "onboard-agent",
            "--dry-run",
            "--agent-id",
            "depth_fixture",
            "--display-name",
            "Depth Fixture",
            "--crate-path",
            "crates/depth_fixture",
            "--backend-module",
            "crates/agent_api/src/backends/depth_fixture",
            "--manifest-root",
            ROOT,
            "--package-name",
            "unified-agent-api-depth-fixture",
            "--canonical-target",
            "x86_64-unknown-linux-gnu",
            "--wrapper-coverage-binding-kind",
            "generated_from_wrapper_crate",
            "--wrapper-coverage-source-path",
            "crates/depth_fixture",
            "--support-matrix-enabled",
            "false",
            "--capability-matrix-enabled",
            "false",
            "--docs-release-track",
            "crates-io",
            "--onboarding-pack-prefix",
            PACK,
        ]);
        let mut output = Vec::new();
        crate::onboard_agent::run_in_workspace(root.path(), args, &mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        output
            .split_once(&format!("Path: {ROOT}/current.json\n```json\n"))
            .unwrap()
            .1
            .split_once("```\n")
            .unwrap()
            .0
            .to_owned()
    }
}

#[test]
fn bootstrap_ordinary_renderer_pointer_survives_creation_and_preparation_unchanged() {
    let f = setup(false);
    let _installed = f.install();
    let parent = head(&f);
    create(&f);
    integrate(&f, &parent);
    let parent = head(&f);
    adopt(&f);
    integrate(&f, &parent);
    assert_eq!(
        fs::read_to_string(f.root.path().join(format!("{ROOT}/current.json"))).unwrap(),
        ordinary_current()
    );
    let parent = head(&f);
    let path = format!("{ROOT}/current.json");
    write(f.root.path(), &path, format!("{}\n", ordinary_current()));
    candidate(&f, &parent);
    assert!(
        check_integration(f.root.path(), &parent).is_err(),
        "preparation must preserve bootstrap bytes"
    );
}

#[test]
fn bootstrap_current_rejects_populated_and_foreign_input_identities() {
    for identity in [
        json!({"agent_id":"depth_fixture", "binary":{"semantic_version":"0.2.0"}, "target_triple":"x86_64-unknown-linux-gnu"}),
        json!({"agent_id":"foreign", "binary":{"semantic_version":"0.2.0"}, "target_triple":"x86_64-unknown-linux-gnu"}),
        json!({"agent_id":"depth_fixture", "binary":{"semantic_version":"0.3.0"}, "target_triple":"x86_64-unknown-linux-gnu"}),
        json!({"agent_id":"depth_fixture", "binary":{"semantic_version":"0.2.0"}, "target_triple":"foreign-target"}),
    ] {
        // Begin with the actual renderer, including its exact ordinary empty-input scaffold.
        let mut current: Value = serde_json::from_str(ordinary_current()).unwrap();
        current["inputs"] = json!([identity]);
        for union_shaped in [false, true] {
            let f = setup(true);
            let parent = head(&f);
            let mut value = current.clone();
            if union_shaped {
                value["snapshot_schema_version"] = json!(2);
                value["mode"] = json!("union");
            }
            write(
                f.root.path(),
                &format!("{ROOT}/current.json"),
                serde_json::to_vec(&value).unwrap(),
            );
            candidate(&f, &parent);
            let _installed = f.install();
            assert!(
                check_integration(f.root.path(), &parent).is_err(),
                "accepted {value}"
            );
        }
    }
}

#[test]
fn bootstrap_preparation_rejects_orphan_report_without_union() {
    let f = setup(true);
    adopt(&f);
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    producer::acquire(f.root.path());
    let path = format!("{ROOT}/reports/0.2.0/coverage.any.json");
    let report = fs::read(f.root.path().join(&path)).unwrap();
    git(f.root.path(), &["reset", "--hard", &parent]);
    git(f.root.path(), &["clean", "-fd"]);
    write(f.root.path(), &path, report);
    candidate(&f, &parent);
    let _installed = f.install();
    let error = check_integration(f.root.path(), &parent).unwrap_err();
    assert!(
        error.contains("report requires its acquired union"),
        "{error}"
    );
}

#[test]
fn bootstrap_preparation_rejects_malformed_report_with_union() {
    let f = setup(true);
    adopt(&f);
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    producer::acquire(f.root.path());
    write(
        f.root.path(),
        &format!("{ROOT}/reports/0.2.0/coverage.any.json"),
        "not-json",
    );
    candidate(&f, &parent);
    let _installed = f.install();
    assert!(check_integration(f.root.path(), &parent).is_err());
}

#[test]
fn bootstrap_preparation_rejects_malformed_partial_snapshot() {
    let f = setup(true);
    adopt(&f);
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    write(f.root.path(), &format!("{ROOT}/snapshots/0.2.0/x86_64-unknown-linux-gnu.json"),
        json!({"snapshot_schema_version":1,"binary":{"semantic_version":"0.2.0","target_triple":"x86_64-unknown-linux-gnu"}}).to_string());
    candidate(&f, &parent);
    let _installed = f.install();
    assert!(check_integration(f.root.path(), &parent).is_err());
}

#[test]
fn bootstrap_current_refuses_even_the_exact_acquired_union_after_empty_renderer_pointer() {
    let f = setup(true);
    let _installed = f.install();
    let parent = head(&f);
    adopt(&f);
    integrate(&f, &parent);
    let parent = head(&f);
    producer::acquire(f.root.path());
    candidate(&f, &parent);
    check_integration(f.root.path(), &parent).unwrap();
    assert_eq!(
        fs::read_to_string(f.root.path().join(format!("{ROOT}/current.json"))).unwrap(),
        ordinary_current()
    );
    let union = fs::read(
        f.root
            .path()
            .join(format!("{ROOT}/snapshots/0.2.0/union.json")),
    )
    .unwrap();
    let path = format!("{ROOT}/current.json");
    write(f.root.path(), &path, &union);
    let rewritten = candidate(&f, &parent);
    assert!(check_integration(f.root.path(), &parent).is_err());
    for edit in 0..4 {
        git(f.root.path(), &["reset", "--hard", &rewritten]);
        let mut current: Value = serde_json::from_slice(&union).unwrap();
        match edit {
            0 => current["tool"] = json!("foreign-cli"),
            1 => current["inputs"][0]["binary"]["semantic_version"] = json!("0.3.0"),
            2 => current["inputs"][0]["target_triple"] = json!("foreign-target"),
            _ => current["agent_id"] = json!("foreign"),
        }
        write(f.root.path(), &path, serde_json::to_vec(&current).unwrap());
        candidate(&f, &parent);
        assert!(
            check_integration(f.root.path(), &parent).is_err(),
            "pointer edit {edit}"
        );
    }
}

#[test]
fn bootstrap_checkpoint_refuses_masked_acquisition_bytes_and_modes() {
    let f = setup(true);
    let parent = head(&f);
    adopt(&f);
    candidate(&f, &parent);
    let checkpoint = f
        .resolve()
        .unwrap()
        .bootstrap_checkpoint(f.root.path(), &parent)
        .unwrap()
        .unwrap();
    let path = format!("{ROOT}/RULES.json");
    let original = fs::read(f.root.path().join(&path)).unwrap();
    git(
        f.root.path(),
        &["update-index", "--assume-unchanged", "--", &path],
    );
    write(f.root.path(), &path, "forged worktree input");
    assert!(git(f.root.path(), &["status", "--porcelain=v1"]).is_empty());
    assert!(checkpoint.with_registry(f.root.path(), || Ok(())).is_err());
    write(f.root.path(), &path, original);
    checkpoint.with_registry(f.root.path(), || Ok(())).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        git(f.root.path(), &["config", "core.filemode", "false"]);
        fs::set_permissions(f.root.path().join(&path), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(git(f.root.path(), &["status", "--porcelain=v1"]).is_empty());
        assert!(checkpoint.with_registry(f.root.path(), || Ok(())).is_err());
    }
}

#[test]
fn bootstrap_checkpoint_rejects_uncommitted_additions_in_its_bound_view() {
    let f = setup(true);
    let parent = head(&f);
    adopt(&f);
    candidate(&f, &parent);
    let checkpoint = f
        .resolve()
        .unwrap()
        .bootstrap_checkpoint(f.root.path(), &parent)
        .unwrap()
        .unwrap();
    assert!(checkpoint
        .with_view(f.root.path(), |view| {
            write(view, &format!("{ROOT}/snapshots/0.2.0/foreign.json"), "{}");
            Ok(())
        })
        .is_err());
}

#[test]
fn bootstrap_preparation_rejects_wrong_artifact_kinds_and_foreign_partial_identities() {
    let f = setup(true);
    adopt(&f);
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    producer::acquire(f.root.path());
    let shard_path = format!("{ROOT}/snapshots/0.2.0/x86_64-unknown-linux-gnu.json");
    let shard: Value =
        serde_json::from_slice(&fs::read(f.root.path().join(&shard_path)).unwrap()).unwrap();
    let wrapper_path = format!("{ROOT}/wrapper_coverage.json");
    let wrapper = fs::read(f.root.path().join(&wrapper_path)).unwrap();
    let _installed = f.install();
    for edit in 0..5 {
        git(f.root.path(), &["reset", "--hard", &parent]);
        git(f.root.path(), &["clean", "-fd"]);
        let mut value = shard.clone();
        match edit {
            0 => write(f.root.path(), &shard_path, &wrapper),
            1 => write(
                f.root.path(),
                &wrapper_path,
                serde_json::to_vec(&shard).unwrap(),
            ),
            _ => {
                match edit {
                    2 => value["tool"] = json!("foreign-cli"),
                    3 => value["binary"]["semantic_version"] = json!("0.3.0"),
                    _ => value["binary"]["target_triple"] = json!("foreign-target"),
                }
                write(
                    f.root.path(),
                    &shard_path,
                    serde_json::to_vec(&value).unwrap(),
                );
            }
        }
        candidate(&f, &parent);
        assert!(
            check_integration(f.root.path(), &parent).is_err(),
            "artifact edit {edit}"
        );
    }
}

#[test]
fn bootstrap_preparation_accepts_well_formed_partial_snapshot() {
    let f = setup(true);
    adopt(&f);
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    producer::acquire(f.root.path());
    let path = format!("{ROOT}/snapshots/0.2.0/x86_64-unknown-linux-gnu.json");
    let shard = fs::read(f.root.path().join(&path)).unwrap();
    git(f.root.path(), &["reset", "--hard", &parent]);
    git(f.root.path(), &["clean", "-fd"]);
    write(f.root.path(), &path, shard);
    let _installed = f.install();
    integrate(&f, &parent);
    assert_eq!(
        fs::read_to_string(f.root.path().join(format!("{ROOT}/current.json"))).unwrap(),
        ordinary_current()
    );
}

fn reject_shard_semantics(with_union: bool, edit: impl FnOnce(&mut Value), expected: &str) {
    let f = setup(true);
    adopt(&f);
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    producer::acquire(f.root.path());
    let shard_path = format!("{ROOT}/snapshots/0.2.0/x86_64-unknown-linux-gnu.json");
    let union_path = format!("{ROOT}/snapshots/0.2.0/union.json");
    let mut shard: Value =
        serde_json::from_slice(&fs::read(f.root.path().join(&shard_path)).unwrap()).unwrap();
    let union = fs::read(f.root.path().join(&union_path)).unwrap();
    git(f.root.path(), &["reset", "--hard", &parent]);
    git(f.root.path(), &["clean", "-fd"]);
    write(f.root.path(), &shard_path, producer::encode_shard(&shard));
    if with_union {
        write(f.root.path(), &union_path, union);
    }
    let _installed = f.install();
    candidate(&f, &parent);
    check_integration(f.root.path(), &parent).unwrap();
    assert!(!f.root.path().join(format!("{ROOT}/reports/0.2.0")).exists());
    assert_eq!(f.root.path().join(&union_path).exists(), with_union);
    edit(&mut shard);
    write(f.root.path(), &shard_path, producer::encode_shard(&shard));
    candidate(&f, &parent);
    let error = check_integration(f.root.path(), &parent).unwrap_err();
    assert!(error.contains(expected), "{error}");
}

#[test]
fn bootstrap_shard_semantics_rejects_duplicate_command_without_union_or_reports() {
    reject_shard_semantics(
        false,
        |shard| {
            let duplicate = shard["commands"][1].clone();
            shard["commands"].as_array_mut().unwrap().push(duplicate);
        },
        "duplicate snapshot command path",
    );
}

fn conflicting_json_flags(shard: &mut Value) {
    shard["commands"][1]["flags"] = json!([
        {"long":"--json","takes_value":false},
        {"long":"--json","takes_value":true,"value_name":"FORMAT"}
    ]);
}

#[test]
fn bootstrap_shard_semantics_rejects_conflicting_flag_arity_without_union_or_reports() {
    reject_shard_semantics(false, conflicting_json_flags, "duplicate snapshot flag");
}

#[test]
fn bootstrap_shard_semantics_rejects_duplicate_argument_without_union_or_reports() {
    reject_shard_semantics(
        false,
        |shard| {
            let duplicate = shard["commands"][1]["args"][0].clone();
            shard["commands"][1]["args"]
                .as_array_mut()
                .unwrap()
                .push(duplicate);
        },
        "duplicate snapshot argument",
    );
}

#[test]
fn bootstrap_shard_semantics_rejects_ambiguous_alias_without_union_or_reports() {
    reject_shard_semantics(
        false,
        |shard| {
            shard["commands"][1]["flags"] = json!([
                {"long":"--json","short":"-j","takes_value":false},
                {"long":"--other","short":"-j","takes_value":false}
            ]);
        },
        "ambiguous snapshot flag alias",
    );
}

#[test]
fn bootstrap_shard_semantics_rejects_conflicting_flag_arity_with_union_without_reports() {
    reject_shard_semantics(true, conflicting_json_flags, "duplicate snapshot flag");
}

#[test]
fn bootstrap_creation_admits_complete_ordinary_on1_scaffold() {
    let f = setup(false);
    let parent = head(&f);
    create(&f);
    let _installed = f.install();
    integrate(&f, &parent);
}

#[test]
fn bootstrap_creation_refuses_extra_or_changed_ordinary_on1_file() {
    let f = setup(false);
    let parent = head(&f);
    create(&f);
    let valid = candidate(&f, &parent);
    let _installed = f.install();
    check_integration(f.root.path(), &parent).unwrap();
    let inventory = format!("docs/agents/lifecycle/{PACK}/governance/written-paths.json");
    let mut paths: Vec<String> =
        serde_json::from_slice(&fs::read(f.root.path().join(inventory)).unwrap()).unwrap();
    paths.push(format!("{ROOT}/versions/extra.txt"));
    for path in paths {
        git(f.root.path(), &["reset", "--hard", &valid]);
        git(f.root.path(), &["clean", "-fd"]);
        write(f.root.path(), &path, "changed scaffold");
        candidate(&f, &parent);
        assert!(check_integration(f.root.path(), &parent).is_err(), "{path}");
    }
}

fn foreign_wrapper_without_reports(with_union: bool) {
    let f = setup(true);
    adopt(&f);
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    producer::acquire(f.root.path());
    let shard_path = format!("{ROOT}/snapshots/0.2.0/x86_64-unknown-linux-gnu.json");
    let union_path = format!("{ROOT}/snapshots/0.2.0/union.json");
    let shard = fs::read(f.root.path().join(&shard_path)).unwrap();
    let union = fs::read(f.root.path().join(&union_path)).unwrap();
    git(f.root.path(), &["reset", "--hard", &parent]);
    git(f.root.path(), &["clean", "-fd"]);
    if with_union {
        write(f.root.path(), &shard_path, shard);
        write(f.root.path(), &union_path, union);
    }
    write(
        f.root.path(),
        &format!("{ROOT}/wrapper_coverage.json"),
        json!({"schema_version":1,"coverage":[{"path":["run"],"level":"explicit",
            "scope":{"platforms":["foreign-os"]}}]})
        .to_string(),
    );
    candidate(&f, &parent);
    let _installed = f.install();
    let error = check_integration(f.root.path(), &parent).unwrap_err();
    assert!(
        error.contains("wrapper scope names an unknown target/platform"),
        "{error}"
    );
}

#[test]
fn bootstrap_wrapper_only_refuses_foreign_platform_without_union_or_reports() {
    foreign_wrapper_without_reports(false);
}

#[test]
fn bootstrap_wrapper_refuses_foreign_platform_with_union_without_reports() {
    foreign_wrapper_without_reports(true);
}

#[test]
fn bootstrap_wrapper_only_checks_claims_scopes_and_subject_without_reports() {
    let f = setup(true);
    adopt(&f);
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    let _installed = f.install();
    let valid = json!({"schema_version":1,"coverage":[{"path":["run"],"level":"explicit",
        "scope":{"platforms":["linux"]},"flags":[{"key":"--json","level":"explicit"}],
        "args":[{"name":"PROMPT","level":"explicit"}]}]});
    for edit in 0..8 {
        git(f.root.path(), &["reset", "--hard", &parent]);
        let mut wrapper = valid.clone();
        match edit {
            0 => {}
            1 => wrapper["coverage"][0]["scope"] = json!({"platforms":["foreign-os"]}),
            2 => wrapper["coverage"][0]["flags"][0]["scope"] = json!({"platforms":["foreign-os"]}),
            3 => wrapper["coverage"][0]["args"][0]["scope"] = json!({"platforms":["foreign-os"]}),
            4 => wrapper["coverage"][0]["scope"] = json!({"target_triples":["foreign-target"]}),
            5 => wrapper["agent_id"] = json!("foreign"),
            6 => {
                wrapper["coverage"][0]["level"] = json!("passthrough");
            }
            _ => {
                let duplicate = wrapper["coverage"][0].clone();
                wrapper["coverage"].as_array_mut().unwrap().push(duplicate);
            }
        }
        write(
            f.root.path(),
            &format!("{ROOT}/wrapper_coverage.json"),
            producer::encode_wrapper(&wrapper),
        );
        candidate(&f, &parent);
        let result = check_integration(f.root.path(), &parent);
        if edit == 0 {
            result.unwrap();
        } else {
            assert!(result.is_err(), "wrapper edit {edit}");
        }
    }
}
