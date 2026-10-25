use super::*;
use crate::agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH};
use serde_json::json;
use std::{collections::BTreeMap, fs};

mod fixture;
use fixture::{declaration, git, write, Fake, Fixture};

#[test]
fn matching_authenticated_private_scope_resolves_without_admission() {
    let f = Fixture::new();
    let context = f.resolve().unwrap();
    context.recheck(f.root.path()).unwrap();
}
#[test]
fn absent_profile_never_reads_host() {
    let root = tempfile::tempdir().unwrap();
    let api = Arc::new(Fake::default());
    assert!(Context::resolve(root.path(), None, api.clone())
        .err()
        .unwrap()
        .contains("not installed"));
    assert_eq!(api.2.load(std::sync::atomic::Ordering::SeqCst), 0);
}
#[test]
fn host_identity_owner_name_and_protection_are_independent_requirements() {
    for (endpoint, replacement) in [
        (
            "repos/fixture/isolated",
            json!({"id":11,"owner":{"id":20},"full_name":"fixture/isolated"}),
        ),
        (
            "repos/fixture/isolated",
            json!({"id":10,"owner":{"id":21},"full_name":"fixture/isolated"}),
        ),
        (
            "repos/fixture/isolated",
            json!({"id":10,"owner":{"id":20},"full_name":"fixture/renamed"}),
        ),
        (
            "repos/fixture/canonical",
            json!({"id":31,"owner":{"id":40},"full_name":"fixture/canonical"}),
        ),
        (
            "repos/fixture/isolated/branches/staging",
            json!({"name":"staging","protected":false,"commit":{"sha":"0".repeat(40)}}),
        ),
    ] {
        let f = Fixture::new();
        f.api.put(endpoint, replacement);
        assert!(f.resolve().is_err(), "accepted {endpoint}");
    }
}
#[test]
fn bound_proposal_preserves_local_concurrent_inputs() {
    let f = Fixture::new();
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    write(f.root.path(), "docs/input.txt", "external edit\n");
    assert!(proposal
        .plan_with(
            f.root.path(),
            vec![("docs/output.txt".into(), b"candidate".to_vec())],
            |_, _, _, _| Ok(BTreeMap::new())
        )
        .is_err());
    assert_eq!(
        fs::read(f.root.path().join("docs/input.txt")).unwrap(),
        b"external edit\n"
    );
    assert!(!f.root.path().join("docs/output.txt").exists());
}

#[test]
fn caller_local_changes_source_inventory_and_baseline_drift_refuse() {
    let f = Fixture::new();
    write(f.root.path(), "untracked-root-input", "unapproved");
    assert!(
        f.resolve().is_err(),
        "untracked caller-local changes accepted"
    );
    fs::remove_file(f.root.path().join("untracked-root-input")).unwrap();
    write(f.root.path(), "docs/input.txt", "dirty tracked input");
    assert!(f.resolve().is_err(), "dirty caller-local changes accepted");
    f.stage();
    assert!(f.resolve().is_err(), "staged caller-local changes accepted");
    git(f.root.path(), &["reset", "--hard", "HEAD"]);
    write(
        f.root.path(),
        "producer/new.rs",
        "additional executed source",
    );
    f.commit();
    assert!(
        f.resolve().is_err(),
        "extra executed producer source accepted"
    );
    let mut profile = f.profile();
    profile.baseline = "0".repeat(40);
    assert!(Context::resolve(f.root.path(), Some(profile), f.api.clone()).is_err());
}
#[test]
fn original_baseline_can_precede_current_independently_approved_source() {
    let f = Fixture::new();
    write(
        f.root.path(),
        "docs/input.txt",
        "later protected installation",
    );
    f.commit();
    assert_ne!(f.baseline, git(f.root.path(), &["rev-parse", "HEAD"]));
    f.resolve().unwrap();
}
#[test]
fn canonical_ordinary_registry_and_isolated_protected_declaration_are_distinct() {
    let f = Fixture::new();
    let head = git(f.root.path(), &["rev-parse", "HEAD"]);
    let key = format!("repos/fixture/canonical/git/trees/{head}?recursive=1");
    let mut tree = f.api.json(&key).unwrap();
    tree["tree"].as_array_mut().unwrap().push(
        json!({"path":REGISTRY_RELATIVE_PATH,"type":"blob","mode":"100644","sha":"1".repeat(40)}),
    );
    f.api.put(&key, tree);
    let context = f.resolve().unwrap();
    assert!(context
        .registry(f.root.path())
        .unwrap()
        .find("codex")
        .unwrap()
        .support_depth
        .is_some());
    let ordinary = include_str!("../../data/agent_registry.toml");
    assert!(AgentRegistry::parse(ordinary)
        .unwrap()
        .find("codex")
        .unwrap()
        .support_depth
        .is_none());
    f.registry(|doc| {
        doc["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("codex"))
            .unwrap()["support_depth"]["enrollments"][0]["operations"][0]["promise"]["r1"] =
            toml_edit::value("changed");
    });
    f.stage();
    assert!(f.resolve().is_err(), "unprotected registry edit accepted");
}
#[test]
fn finite_selection_rejects_scope_expansion_even_when_protected() {
    for edit in 0..10 {
        let f = Fixture::new();
        f.registry(|doc| {
            let agent = doc["agents"]
                .as_array_of_tables_mut()
                .unwrap()
                .iter_mut()
                .find(|a| a["agent_id"].as_str() == Some("codex"))
                .unwrap();
            let declaration = &mut agent["support_depth"]["enrollments"][0];
            match edit {
                0 => declaration["version"] = toml_edit::value("0.156.2"),
                1 => declaration["lifecycle_path"] = toml_edit::value("onboarding"),
                2 => declaration["targets"][0] = toml_edit::value("x86_64-unknown-linux-gnu"),
                3 => declaration["modes"][0]["mode"] = toml_edit::value("interactive"),
                4 => declaration["modes"][0]["predicates"][0]["flag"] = toml_edit::value("--other"),
                5 => declaration["operations"][0]["forwarding"] = toml_edit::value(true),
                6 => {
                    declaration["operations"][0]["not_applicable_permitted"] =
                        toml_edit::value(["X1"].into_iter().collect::<toml_edit::Array>())
                }
                7 => declaration["capabilities"][0]["id"] = toml_edit::value("agent_api.other"),
                8 => declaration["operations"][0]["templates"][0] = toml_edit::value("SESSION"),
                _ => declaration["modes"][0]["predicates"][0]["value"] = toml_edit::value("true"),
            }
        });
        f.commit();
        assert!(f.resolve().is_err(), "accepted expansion {edit}");
    }
}
#[test]
fn every_retained_root_and_removed_declaration_is_mediated() {
    let f = Fixture::new();
    write(
        f.root.path(),
        "cli_manifests/codex/reports/0.156.1/depth-record.json",
        f.retained(),
    );
    f.commit();
    f.resolve().unwrap();
    for path in [
        "cli_manifests/claude_code/reports/0.156.1/depth-record.json",
        "cli_manifests/orphan/reports/0.156.1/depth-record.json",
        "reports/0.156.1/depth-record.json",
        "cli_manifests/codex/reports/0.156.2/depth-record.json",
    ] {
        write(f.root.path(), path, f.retained());
        f.commit();
        assert!(f.resolve().is_err(), "accepted {path}");
        fs::remove_file(f.root.path().join(path)).unwrap();
        f.commit();
    }
    f.registry(|doc| {
        doc["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("codex"))
            .unwrap()
            .remove("support_depth");
    });
    f.commit();
    assert!(f.resolve().is_err());
}
#[test]
fn retained_record_and_inventory_drift_invalidate_the_existing_context() {
    let f = Fixture::new();
    let path = "cli_manifests/codex/reports/0.156.1/depth-record.json";
    write(f.root.path(), path, f.retained());
    f.commit();
    let context = f.resolve().unwrap();
    write(f.root.path(), path, "external record edit");
    assert!(context.recheck(f.root.path()).is_err());
    assert_eq!(
        fs::read_to_string(f.root.path().join(path)).unwrap(),
        "external record edit"
    );
    git(f.root.path(), &["reset", "--hard", "HEAD"]);
    write(
        f.root.path(),
        "cli_manifests/codex/reports/new/depth-record.json",
        "external addition",
    );
    assert!(context.recheck(f.root.path()).is_err());
}
#[test]
fn foreign_git_shared_metadata_copied_scratch_and_forged_remote_refuse() {
    let f = Fixture::new();
    let scratch = tempfile::tempdir().unwrap();
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    assert!(view
        .source()
        .unwrap()
        .read(REGISTRY_RELATIVE_PATH)
        .unwrap()
        .contains("support_depth"));
    assert!(Context::resolve(scratch.path(), Some(f.profile()), f.api.clone()).is_err());
    assert!(proposal
        .plan_with(scratch.path(), vec![], |_, _, _, _| Ok(BTreeMap::new()))
        .is_err());
    git(
        f.root.path(),
        &[
            "remote",
            "set-url",
            "origin",
            "https://github.com/forged/repo.git",
        ],
    );
    assert!(f.resolve().is_err());
    assert!(view.source().is_err());
    git(
        f.root.path(),
        &[
            "remote",
            "set-url",
            "origin",
            "https://github.com/fixture/isolated.git",
        ],
    );
    let linked = tempfile::tempdir().unwrap();
    git(
        f.root.path(),
        &[
            "worktree",
            "add",
            "--detach",
            linked.path().to_str().unwrap(),
            "HEAD",
        ],
    );
    assert!(Context::resolve(linked.path(), Some(f.profile()), f.api.clone()).is_err());
    let foreign = tempfile::tempdir().unwrap();
    git(foreign.path(), &["init"]);
    assert!(Context::resolve(foreign.path(), Some(f.profile()), f.api.clone()).is_err());
    fs::create_dir(scratch.path().join(".git")).unwrap();
    assert!(view.source().is_err());
}
#[test]
fn host_source_tip_read_failures_and_recreated_repository_invalidate_plans() {
    for change in 0..5 {
        let f = Fixture::new();
        let context = f.resolve().unwrap();
        let head = git(f.root.path(), &["rev-parse", "HEAD"]);
        match change {
            0 => f.api.put(
                "repos/fixture/isolated",
                json!({"id":999,"owner":{"id":20},"full_name":"fixture/isolated"}),
            ),
            1 => f.api.put(
                "repos/fixture/isolated/branches/staging",
                json!({"name":"staging","protected":true,"commit":{"sha":"2".repeat(40)}}),
            ),
            2 => f.api.put(
                "repos/fixture/canonical/branches/staging",
                json!({"name":"staging","protected":true,"commit":{"sha":"3".repeat(40)}}),
            ),
            3 => {
                f.api.0.lock().unwrap().remove("repos/fixture/isolated");
            }
            _ => {
                let endpoint = format!("repos/fixture/canonical/git/trees/{head}?recursive=1");
                let mut tree = f.api.json(&endpoint).unwrap();
                tree["tree"][1]["sha"] = json!("4".repeat(40));
                f.api.put(&endpoint, tree);
            }
        }
        assert!(
            context.recheck(f.root.path()).is_err(),
            "accepted changed host/source {change}"
        );
    }
}
#[test]
fn apply_rechecks_host_and_preserves_concurrent_changes_before_any_write() {
    let f = Fixture::new();
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let result = proposal.apply_with(
        f.root.path(),
        vec![("docs/output.txt".into(), b"planned".to_vec())],
        |_, _, _, _| {
            write(
                f.root.path(),
                "docs/input.txt",
                "external edit during admission",
            );
            f.api.put("repos/fixture/isolated", json!({"id":999}));
            Ok(BTreeMap::new())
        },
    );
    assert!(result.is_err());
    assert!(!f.root.path().join("docs/output.txt").exists());
    assert_eq!(
        fs::read_to_string(f.root.path().join("docs/input.txt")).unwrap(),
        "external edit during admission"
    );
}
#[test]
fn private_bound_apply_and_ordinary_callers_keep_existing_expected_before_semantics() {
    let f = Fixture::new();
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    // Fixture admission is deliberately empty: this tests binding/delivery mechanics, no depth verdict.
    proposal
        .apply_with(
            f.root.path(),
            vec![("docs/output.txt".into(), b"planned".to_vec())],
            |_, _, _, _| Ok(BTreeMap::new()),
        )
        .unwrap();
    assert_eq!(
        fs::read(f.root.path().join("docs/output.txt")).unwrap(),
        b"planned"
    );
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "docs/input.txt", "ordinary");
    let ordinary = Proposal::capture(root.path()).unwrap();
    assert!(ordinary
        .isolated_view(root.path(), tempfile::tempdir().unwrap().path())
        .is_err());
    ordinary
        .apply_with(
            root.path(),
            vec![("docs/output.txt".into(), b"ordinary output".to_vec())],
            |_, _, _, _| Ok(BTreeMap::new()),
        )
        .unwrap();
    assert_eq!(
        fs::read(root.path().join("docs/output.txt")).unwrap(),
        b"ordinary output"
    );
}

#[test]
fn root_and_git_replacement_do_not_inherit_a_prior_binding() {
    for replace_git in [false, true] {
        let f = Fixture::new();
        let context = f.resolve().unwrap();
        let saved = tempfile::tempdir().unwrap();
        let subject = if replace_git {
            f.root.path().join(".git")
        } else {
            f.root.path().to_owned()
        };
        fs::rename(&subject, saved.path().join("original")).unwrap();
        fs::create_dir(&subject).unwrap();
        assert!(context.recheck(f.root.path()).is_err());
    }
}
#[test]
fn copied_or_retargeted_derived_view_never_reseals_itself() {
    let f = Fixture::new();
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    let source = view.source().unwrap();
    write(scratch.path(), REGISTRY_RELATIVE_PATH, "forged registry");
    assert!(source
        .read(REGISTRY_RELATIVE_PATH)
        .unwrap()
        .contains("support_depth"));
    assert!(view
        .source()
        .unwrap()
        .read(REGISTRY_RELATIVE_PATH)
        .unwrap()
        .contains("support_depth"));
    let saved = tempfile::tempdir().unwrap();
    fs::rename(scratch.path(), saved.path().join("original")).unwrap();
    fs::create_dir(scratch.path()).unwrap();
    assert!(view.source().is_err());
}
#[test]
fn missing_protected_git_objects_refuse_without_fetching() {
    let f = Fixture::new();
    let absent = "9".repeat(40);
    f.api.put(
        "repos/fixture/isolated/branches/staging",
        json!({"name":"staging","protected":true,"commit":{"sha":absent}}),
    );
    f.api.put(
        &format!("repos/fixture/isolated/compare/{}...{absent}", f.baseline),
        json!({"status":"ahead","merge_base_commit":{"sha":f.baseline}}),
    );
    let head = git(f.root.path(), &["rev-parse", "HEAD"]);
    f.api.put(
        &format!("repos/fixture/isolated/compare/{absent}...{head}"),
        json!({"status":"ahead","merge_base_commit":{"sha":absent}}),
    );
    let registry = git(
        f.root.path(),
        &["rev-parse", &format!("HEAD:{REGISTRY_RELATIVE_PATH}")],
    );
    f.api.put(
        &format!("repos/fixture/isolated/contents/{REGISTRY_RELATIVE_PATH}?ref={absent}"),
        json!({"path":REGISTRY_RELATIVE_PATH,"type":"file","sha":registry}),
    );
    assert!(f.resolve().err().unwrap().contains("git rev-parse"));
    let out = crate::git_inputs::command(f.root.path())
        .args(["cat-file", "-e", &absent])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(!f.root.path().join(".git/FETCH_HEAD").exists());
}
#[test]
fn canonical_tree_mode_inventory_truncation_and_baseline_ancestry_refuse() {
    for change in 0..5 {
        let f = Fixture::new();
        let head = git(f.root.path(), &["rev-parse", "HEAD"]);
        let endpoint = format!("repos/fixture/canonical/git/trees/{head}?recursive=1");
        let mut tree = f.api.json(&endpoint).unwrap();
        match change {
            0 => tree["truncated"] = json!(true),
            1 => tree["tree"][1]["mode"] = json!("100755"),
            2 => { let duplicate = tree["tree"][1].clone(); tree["tree"].as_array_mut().unwrap().push(duplicate); },
            3 => tree["tree"].as_array_mut().unwrap().push(json!({"path":"producer/extra.rs","type":"blob","mode":"100644","sha":"8".repeat(40)})),
            _ => f.api.put(&format!("repos/fixture/isolated/compare/{}...{head}",f.baseline),json!({"status":"diverged","merge_base_commit":{"sha":"0".repeat(40)}})),
        }
        f.api.put(&endpoint, tree);
        assert!(
            f.resolve().is_err(),
            "accepted source/ancestry change {change}"
        );
    }
}
#[test]
fn inherited_git_redirects_cannot_replace_the_actual_caller() {
    const CHILD: &str = "XTASK_PRIVATE_BINDING_ENV_TEST";
    if std::env::var_os(CHILD).is_some() {
        let f = Fixture::new();
        f.resolve().unwrap();
        return;
    }
    let foreign = tempfile::tempdir().unwrap();
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "isolated_maintenance::tests::inherited_git_redirects_cannot_replace_the_actual_caller",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env("GIT_DIR", foreign.path())
        .env("GIT_WORK_TREE", foreign.path())
        .env("GIT_INDEX_FILE", foreign.path().join("forged-index"))
        .env("GIT_OBJECT_DIRECTORY", foreign.path())
        .env("GH_REPO", "forged/repository")
        .env("GH_HOST", "forged.invalid")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn moving_host_reads_are_rejected_inside_one_resolution() {
    let f = Fixture::new();
    let endpoint = "repos/fixture/isolated/branches/staging";
    let good = f.api.json(endpoint).unwrap();
    let mut moved = good.clone();
    moved["commit"]["sha"] = json!("7".repeat(40));
    f.api
        .1
        .lock()
        .unwrap()
        .insert(endpoint.into(), [good, moved].into());
    assert!(f
        .resolve()
        .err()
        .unwrap()
        .contains("moved during observation"));
}
#[test]
fn canonical_and_local_source_cannot_self_approve_executed_changes() {
    for mode in [false, true] {
        let f = Fixture::new();
        if mode {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    f.root.path().join("producer/source.rs"),
                    fs::Permissions::from_mode(0o755),
                )
                .unwrap();
            }
        } else {
            write(
                f.root.path(),
                "producer/source.rs",
                "self-approved replacement",
            );
        }
        f.commit();
        assert!(
            f.resolve().is_err(),
            "modified executed source/mode accepted"
        );
    }
}
#[test]
fn extra_registry_agents_enrollments_targets_and_modes_refuse() {
    for change in 0..4 {
        let f = Fixture::new();
        let mut depth = declaration();
        match change {
            1 => {
                let mut extra = depth["enrollments"][0].clone();
                extra["version"] = json!("0.156.2");
                depth["enrollments"].as_array_mut().unwrap().push(extra);
            }
            2 => depth["enrollments"][0]["targets"]
                .as_array_mut()
                .unwrap()
                .push(json!("x86_64-unknown-linux-gnu")),
            3 => {
                let mut mode = depth["enrollments"][0]["modes"][0].clone();
                mode["selector"] = json!("another");
                mode["mode"] = json!("another-mode");
                depth["enrollments"][0]["modes"]
                    .as_array_mut()
                    .unwrap()
                    .push(mode);
                depth["enrollments"][0]["operations"][0]["serves"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("another"));
            }
            _ => depth = json!({"enabled_paths":["maintenance"]}),
        }
        let table: crate::agent_registry::support_depth::SupportDepth =
            serde_json::from_value(depth).unwrap();
        let depth = toml_edit::ser::to_document(&table).unwrap();
        f.registry(|doc| {
            let agent = doc["agents"]
                .as_array_of_tables_mut()
                .unwrap()
                .iter_mut()
                .find(|a| {
                    a["agent_id"].as_str()
                        == Some(if change == 0 { "claude_code" } else { "codex" })
                })
                .unwrap();
            agent.insert(
                "support_depth",
                toml_edit::Item::Table(depth.as_table().clone()),
            );
        });
        f.commit();
        AgentRegistry::parse_unenforced(
            &fs::read_to_string(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap(),
        )
        .unwrap();
        assert!(f.resolve().is_err(), "extra scope accepted: {change}");
    }
}

#[test]
fn apply_host_only_drift_refuses_with_unchanged_local_inputs() {
    let f = Fixture::new();
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let result = proposal.apply_with(
        f.root.path(),
        vec![("docs/output.txt".into(), b"planned".to_vec())],
        |_, _, _, _| {
            f.api.put(
                "repos/fixture/isolated/branches/staging",
                json!({"name":"staging","protected":true,"commit":{"sha":"6".repeat(40)}}),
            );
            Ok(BTreeMap::new())
        },
    );
    assert!(result.is_err());
    assert!(!f.root.path().join("docs/output.txt").exists());
    assert_eq!(
        fs::read_to_string(f.root.path().join("docs/input.txt")).unwrap(),
        "initial\n"
    );
}
#[test]
fn apply_keeps_external_changes_to_its_exact_destination() {
    let f = Fixture::new();
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let result = proposal.apply_with(
        f.root.path(),
        vec![("docs/input.txt".into(), b"planned replacement".to_vec())],
        |_, _, _, _| {
            write(f.root.path(), "docs/input.txt", "external replacement");
            Ok(BTreeMap::new())
        },
    );
    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(f.root.path().join("docs/input.txt")).unwrap(),
        "external replacement"
    );
}

#[test]
fn authority_gap_governing_blob_requires_independent_host_identity() {
    let f = Fixture::new();
    let head = git(f.root.path(), &["rev-parse", "HEAD"]);
    f.api.put(
        &format!("repos/fixture/isolated/contents/{REGISTRY_RELATIVE_PATH}?ref={head}"),
        json!({"path":REGISTRY_RELATIVE_PATH,"type":"file","sha":"0".repeat(40)}),
    );
    assert!(
        f.resolve().is_err(),
        "local protected objects alone authorized governing data"
    );
}
#[cfg(unix)]
#[test]
fn authority_gap_shared_object_metadata_requires_owned_directory() {
    let f = Fixture::new();
    let external = tempfile::tempdir().unwrap();
    fs::rename(
        f.root.path().join(".git/objects"),
        external.path().join("objects"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        external.path().join("objects"),
        f.root.path().join(".git/objects"),
    )
    .unwrap();
    assert!(
        f.resolve().is_err(),
        "shared symlinked object directory accepted"
    );
}
