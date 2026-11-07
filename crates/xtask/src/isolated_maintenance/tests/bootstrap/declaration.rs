use super::*;
use crate::{
    agent_registry::support_depth::SurfaceClassification, depth_record::acquisition,
    support_depth::FreezePlan, support_matrix,
};

fn copy(src: &Path, dst: &Path) {
    if src.is_dir() {
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            copy(&entry.path(), &dst.join(entry.file_name()));
        }
    } else {
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(src, dst).unwrap();
    }
}
fn prepared() -> Fixture {
    let f = setup(true);
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in [
        "Cargo.toml",
        "crates/agent_api/Cargo.toml",
        "crates/agent_api/src",
        "cli_manifests",
        "docs/specs/unified-agent-api",
    ] {
        copy(&repo.join(path), &f.root.path().join(path));
    }
    f.commit();
    local(&f, &head(&f));
    let _installed = f.install();
    let parent = head(&f);
    adopt(&f);
    integrate(&f, &parent);
    let parent = head(&f);
    producer::acquire(f.root.path());
    integrate(&f, &parent);
    let parent = head(&f);
    metadata(&f);
    integrate(&f, &parent);
    f
}
fn declare(f: &Fixture) {
    let root = f.root.path();
    let entry = AgentRegistry::parse_unenforced(DECLARED)
        .unwrap()
        .agents
        .remove(0);
    let table = toml_edit::ser::to_document(&entry.support_depth.unwrap()).unwrap();
    f.registry(|d| {
        d["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap()
            .insert(
                "support_depth",
                toml_edit::Item::Table(table.as_table().clone()),
            );
    });
    f.stage();
    let raw = fs::read(root.join(APPROVAL)).unwrap();
    let adapter = crate::isolated_maintenance::onboarding::approval::read(
        &raw,
        APPROVAL,
        &AgentRegistry::parse_unenforced(SCAFFOLD).unwrap().agents[0],
    )
    .unwrap();
    let plan = FreezePlan::onboarding(&mut GitInputs::new(root).unwrap(), &adapter, None).unwrap();
    let basis = &plan.classification_bases[0];
    let classifications: Vec<_> = plan
        .record
        .obligations
        .as_ref()
        .unwrap()
        .classification_required
        .iter()
        .map(|c| SurfaceClassification {
            surface: c.surface.clone(),
            templates: vec![crate::agent_registry::support_depth::ObligationTemplate::Core],
            rationale: "Reviewed synthetic wrapper CORE promise".into(),
        })
        .collect();
    let mut entry = AgentRegistry::parse_unenforced(DECLARED)
        .unwrap()
        .agents
        .remove(0);
    let operation = &mut entry
        .support_depth
        .as_mut()
        .unwrap()
        .enrollments
        .as_mut()
        .unwrap()[0]
        .operations[0];
    operation.classification_basis = Some(basis.sha256.clone());
    operation.classifications = Some(classifications);
    let table = toml_edit::ser::to_document(&entry.support_depth.unwrap()).unwrap();
    f.registry(|d| {
        let entry = d["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap();
        entry.insert(
            "support_depth",
            toml_edit::Item::Table(table.as_table().clone()),
        );
    });
    let template = include_str!("../../../../tests/fixtures/depth-synthetic/onboarding/governance/depth-acquisition/0.2.0.toml");
    let mut statement: toml_edit::DocumentMut = template.parse().unwrap();
    for input in statement["inputs"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
    {
        let path = input["path"].as_str().unwrap();
        let hash = hex::encode(Sha256::digest(fs::read(root.join(path)).unwrap()));
        input["sha256"] = toml_edit::value(hash);
        let command = input["command"].as_str().unwrap().replace(
            "{{DEPTH_FIXTURE_BINARY}}",
            "/offline/acquired/depth_fixture",
        );
        input["command"] = toml_edit::value(command);
    }
    write(
        root,
        &acquisition::statement_path(PACK, "0.2.0"),
        statement.to_string(),
    );
}
fn publish(f: &Fixture, parent: &str) -> String {
    candidate(f, parent);
    let context = f.resolve().unwrap();
    let checkpoint = context
        .bootstrap_checkpoint(f.root.path(), parent)
        .unwrap()
        .unwrap();
    let (bundle, capability) = checkpoint
        .with_view(f.root.path(), |view| {
            Ok((
                support_matrix::generate_publication_artifacts(view)?,
                crate::capability_matrix::generate_markdown_for_workspace(view)?,
            ))
        })
        .unwrap();
    for (path, raw) in [
        (support_matrix::JSON_OUTPUT_PATH, bundle.json),
        (support_matrix::MARKDOWN_OUTPUT_PATH, bundle.markdown),
        (
            support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            bundle.runtime_support_data,
        ),
        (crate::capability_matrix::DEFAULT_OUT_PATH, capability),
    ] {
        write(f.root.path(), path, raw);
    }
    git(f.root.path(), &["reset", "--soft", parent]);
    candidate(f, parent)
}
#[test]
fn bootstrap_declaration_refuses_even_the_exact_acquired_union_as_current() {
    let f = prepared();
    let _installed = f.install();
    let parent = head(&f);
    declare(&f);
    let valid = publish(&f, &parent);
    check_integration(f.root.path(), &parent).unwrap();
    let path = format!("{ROOT}/current.json");
    assert_eq!(
        fs::read_to_string(f.root.path().join(&path)).unwrap(),
        regressions::ordinary_current()
    );
    let union = fs::read(
        f.root
            .path()
            .join(format!("{ROOT}/snapshots/0.2.0/union.json")),
    )
    .unwrap();
    for bytes in [
        union,
        format!("{}\n", regressions::ordinary_current()).into_bytes(),
    ] {
        git(f.root.path(), &["reset", "--hard", &valid]);
        write(f.root.path(), &path, bytes);
        candidate(&f, &parent);
        assert!(check_integration(f.root.path(), &parent).is_err());
    }
}

#[test]
fn first_declaration_is_admitted_before_tip_and_only_captured_after_tip_readback() {
    let f = prepared();
    let _installed = f.install();
    let parent = head(&f);
    declare(&f);
    let next = publish(&f, &parent);
    let route = route(f.root.path()).unwrap().unwrap();
    assert!(route
        .registry(f.root.path())
        .unwrap()
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_none());
    check_integration(f.root.path(), &parent).unwrap();
    f.host();
    local(&f, &next);
    check_integration(f.root.path(), &parent).unwrap();
    assert!(
        route.recheck(f.root.path()).is_err(),
        "old captured route must expire"
    );
    assert!(crate::isolated_maintenance::route(f.root.path())
        .unwrap()
        .unwrap()
        .registry(f.root.path())
        .unwrap()
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_some());
    assert!(AgentRegistry::parse(
        &fs::read_to_string(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap()
    )
    .is_err());
    assert!(!f.root.path().join(RECORD).exists());
}
#[test]
fn first_declaration_observation_keeps_bootstrap_effect_bounds() {
    for parent_has_entry in [false, true] {
        let f = prepared();
        let _installed = f.install();
        let registry = fs::read(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
        if !parent_has_entry {
            f.registry(|doc| {
                let entries = doc["agents"].as_array_of_tables_mut().unwrap();
                let index = entries
                    .iter()
                    .position(|entry| entry["agent_id"].as_str() == Some("depth_fixture"))
                    .unwrap();
                entries.remove(index);
            });
            f.commit();
        }
        let parent = head(&f);
        write(f.root.path(), REGISTRY_RELATIVE_PATH, registry);
        declare(&f);
        write(
            f.root.path(),
            "docs/foreign.txt",
            "outside the bootstrap checkpoint\n",
        );
        let next = candidate(&f, &parent);
        f.host();
        local(&f, &next);
        assert_eq!(
            check_integration(f.root.path(), &parent).unwrap_err(),
            if parent_has_entry {
                "bootstrap checkpoint has an unauthorized effect: docs/foreign.txt"
            } else {
                "bootstrap requires the exact absent-to-scaffold registry delta"
            }
        );
    }
}

#[test]
fn first_declaration_refuses_extra_tables_records_entries_acceptance_and_selection_changes() {
    let f = prepared();
    let _installed = f.install();
    let parent = head(&f);
    declare(&f);
    let valid = publish(&f, &parent);
    check_integration(f.root.path(), &parent).unwrap();
    for edit in 0..14 {
        git(f.root.path(), &["reset", "--hard", &valid]);
        // Remove untracked files left by a refused candidate; this disposable test owns them.
        git(f.root.path(), &["clean", "-fd"]);
        match edit {
            0 => write(f.root.path(), RECORD, "{}"),
            1 => write(f.root.path(),"reports/foreign/depth-record.json","{}"),
            2 => write(f.root.path(), &format!("{ROOT}/latest_validated.txt"),"0.2.0\n"),
            3 => write(f.root.path(), "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/proving-run-closeout.json", r#"{"state":"closed"}"#),
            4 => f.registry(|d| { let e=d["agents"].as_array_of_tables_mut().unwrap().iter_mut().find(|e|e["agent_id"].as_str()==Some("depth_fixture")).unwrap(); e["support_depth"]["enrollments"][0]["operations"][0]["not_applicable_permitted"]=toml_edit::value(["X4"].into_iter().collect::<toml_edit::Array>()); }),
            5 => f.registry(|d| { d["agents"].as_array_of_tables_mut().unwrap().iter_mut().find(|e|e["agent_id"].as_str()==Some("codex")).unwrap()["display_name"]=toml_edit::value("Unrelated CORE edit"); }),
            6 => f.registry(|d| { let e=d["agents"].as_array_of_tables_mut().unwrap().iter_mut().find(|e|e["agent_id"].as_str()==Some("depth_fixture")).unwrap(); e["support_depth"]["enrollments"][0]["version"]=toml_edit::value("0.3.0"); }),
            7 => { let table:crate::agent_registry::support_depth::SupportDepth=serde_json::from_value(fixture::declaration()).unwrap(); let table=toml_edit::ser::to_document(&table).unwrap(); f.registry(|d| { d["agents"].as_array_of_tables_mut().unwrap().iter_mut().find(|e|e["agent_id"].as_str()==Some("codex")).unwrap().insert("support_depth",toml_edit::Item::Table(table.as_table().clone())); }); },
            8 => { let path=acquisition::statement_path(PACK,"0.2.0"); let raw=fs::read_to_string(f.root.path().join(&path)).unwrap(); write(f.root.path(),&path,raw.replace("schema_version = 1","schema_version = 2")); },
            9 => { let path=support_matrix::JSON_OUTPUT_PATH; let mut value:Value=serde_json::from_slice(&fs::read(f.root.path().join(path)).unwrap()).unwrap(); value["rows"][0]["version"]=json!("99.0.0"); write(f.root.path(),path,serde_json::to_vec(&value).unwrap()); },
            10 => f.registry(|d| { let e=d["agents"].as_array_of_tables_mut().unwrap().iter_mut().find(|e|e["agent_id"].as_str()==Some("depth_fixture")).unwrap(); e["support_depth"]["enrollments"][0]["operations"][0]["classifications"][0]["surface"]["command_path"]=toml_edit::value("foreign run"); }),
            13 => {
                let raw = fs::read_to_string(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
                write(f.root.path(), REGISTRY_RELATIVE_PATH, format!("{raw}\n# unrelated registry edit\n"));
            },
            12 => {
                let path=acquisition::statement_path(PACK,"0.2.0");
                let mut doc:toml_edit::DocumentMut=fs::read_to_string(f.root.path().join(&path)).unwrap().parse().unwrap();
                doc["inputs"][0]["sha256"]=toml_edit::value("b".repeat(64));
                write(f.root.path(),&path,doc.to_string());
            },
            _ => write(f.root.path(),"cli_manifests/codex/versions/0.156.1.json",r#"{"status":"validated"}"#),
        }
        candidate(&f, &parent);
        let before = git(f.root.path(), &["status", "--porcelain=v1"]);
        assert!(
            check_integration(f.root.path(), &parent).is_err(),
            "edit {edit}"
        );
        assert_eq!(before, git(f.root.path(), &["status", "--porcelain=v1"]));
    }
}

#[test]
fn bootstrap_publication_rejects_forged_commit_masked_by_assume_unchanged_worktree() {
    let f = prepared();
    let _installed = f.install();
    let parent = head(&f);
    declare(&f);
    publish(&f, &parent);
    let path = support_matrix::JSON_OUTPUT_PATH;
    let correct = fs::read(f.root.path().join(path)).unwrap();
    let mut forged: Value = serde_json::from_slice(&correct).unwrap();
    forged["rows"][0]["version"] = json!("99.0.0");
    write(f.root.path(), path, serde_json::to_vec(&forged).unwrap());
    git(f.root.path(), &["reset", "--soft", &parent]);
    let forged_head = candidate(&f, &parent);
    git(
        f.root.path(),
        &["update-index", "--assume-unchanged", "--", path],
    );
    write(f.root.path(), path, &correct);
    assert!(git(f.root.path(), &["status", "--porcelain=v1"]).is_empty());
    let index = git(f.root.path(), &["ls-files", "--stage"]);
    assert!(check_integration(f.root.path(), &parent).is_err());
    assert_eq!(head(&f), forged_head);
    assert_eq!(git(f.root.path(), &["ls-files", "--stage"]), index);
    assert_eq!(fs::read(f.root.path().join(path)).unwrap(), correct);
    assert!(git(f.root.path(), &["status", "--porcelain=v1"]).is_empty());
}

#[test]
fn bootstrap_publication_checks_complete_files_for_preparation_and_declaration() {
    for declaration in [false, true] {
        let f = prepared();
        let _installed = f.install();
        let parent = head(&f);
        if declaration {
            declare(&f);
        }
        let valid = publish(&f, &parent);
        check_integration(f.root.path(), &parent).unwrap();
        for path in [
            support_matrix::MARKDOWN_OUTPUT_PATH,
            support_matrix::JSON_OUTPUT_PATH,
            support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            crate::capability_matrix::DEFAULT_OUT_PATH,
        ] {
            git(f.root.path(), &["reset", "--hard", &valid]);
            let raw = fs::read_to_string(f.root.path().join(path)).unwrap();
            let edited = if path == support_matrix::MARKDOWN_OUTPUT_PATH {
                let edited = raw.replace(
                    "manifest evidence alone MUST NOT be promoted into backend support",
                    "manifest evidence alone MAY be promoted into backend support",
                );
                assert_ne!(raw, edited);
                edited
            } else {
                format!("{raw}\n")
            };
            write(f.root.path(), path, edited);
            candidate(&f, &parent);
            let error = check_integration(f.root.path(), &parent).unwrap_err();
            assert!(
                error.contains("publication is incoherent"),
                "{path}: {error}"
            );
        }
    }
}
