use super::*;

fn baseline() -> (Fixture, String) {
    let mut f = promotions::closed();
    f.record.acceptance_entries.clear();
    f.save();
    let _ = fs::remove_file(f.root.path().join(CLOSEOUT));
    f.producer.current = f.record.clone();
    let request = format!(
        "{}\n[support_depth]\npolicy_identity = \"{}\"\nobligations_identity = \"{}\"\n",
        fs::read_to_string(f.root.path().join(REQUEST)).unwrap(),
        f.record.policy.identity,
        f.record.obligations.as_ref().unwrap().identity
    );
    write(f.root.path(), REQUEST, &request);
    let mut closeout: Value = serde_json::from_slice(&f.closeout).unwrap();
    closeout["request_sha256"] = json!(artifacts::digest(&request));
    f.closeout = serde_json::to_vec(&closeout).unwrap();
    git(f.root.path(), &["add", "."]);
    git(f.root.path(), &["commit", "-qm", "integration baseline"]);
    let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
    git(
        f.root.path(),
        &["update-ref", "refs/remotes/origin/staging", &tip],
    );
    (f, tip)
}
fn check(f: &Fixture, tip: &str) -> Result<(), String> {
    let before = Source::tip(f.root.path(), tip)?;
    let registry = AgentRegistry::parse_unenforced(&before.read(REGISTRY_RELATIVE_PATH)?)
        .map_err(|e| e.to_string())?;
    let old = View::with_registry(before.clone(), &registry, "codex")?;
    let new = View::with_registry(Source::workspace(f.root.path())?, &registry, "codex")?;
    crate::depth_acceptance::integration::admit(
        f.root.path(),
        &old,
        &new,
        &before,
        &f.producer,
        &f.producer,
    )
}
fn deliver(f: &Fixture) {
    let plans = f.plan().unwrap();
    crate::workspace_mutation::apply_admitted_mutations(f.root.path(), &plans).unwrap();
    git(f.root.path(), &["add", "."]);
}
#[test]
fn local_closeout_and_final_admission_agree_without_writes() {
    let (f, tip) = baseline();
    deliver(&f);
    let bytes = fs::read(f.root.path().join(RECORD)).unwrap();
    check(&f, &tip).unwrap();
    assert_eq!(fs::read(f.root.path().join(RECORD)).unwrap(), bytes);
}
#[test]
fn matching_entry_cannot_replace_independent_evidence_at_integration() {
    let (mut f, tip) = baseline();
    deliver(&f);
    f.producer.proved = false;
    assert!(check(&f, &tip)
        .unwrap_err()
        .contains("independently satisfied"));
}
#[test]
fn candidate_only_marker_cannot_authorize_a_closeout() {
    let (f, _) = baseline();
    let plans = f.plan().unwrap();
    fs::remove_file(f.root.path().join(MARKER)).unwrap();
    git(f.root.path(), &["add", "."]);
    git(
        f.root.path(),
        &["commit", "-qm", "unfrozen integration parent"],
    );
    let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
    write(
        f.root.path(),
        MARKER,
        stand_down::render_marker_toml("codex", "1.2.3", STAMP),
    );
    crate::workspace_mutation::apply_admitted_mutations(f.root.path(), &plans).unwrap();
    assert!(check(&f, &tip).is_err());
}
#[test]
fn new_stale_or_unsupported_entries_refuse_even_without_other_effects() {
    for entry in [
        json!({"backs":"promotion","targets":[LINUX],"record_sha256":"a".repeat(64)}),
        json!({"backs":"maintenance_closeout","closeout":{"path":CLOSEOUT,"sha256":"b".repeat(64)},"record_sha256":"a".repeat(64)}),
    ] {
        let (mut f, tip) = baseline();
        f.record
            .acceptance_entries
            .push(serde_json::from_value(entry).unwrap());
        f.save();
        assert!(check(&f, &tip).is_err());
    }
}

fn publish(f: &Fixture, verified: bool) {
    let record = DepthRecord::parse(
        &fs::read_to_string(f.root.path().join(RECORD)).unwrap(),
        "codex",
        "1.2.3",
    )
    .unwrap();
    let path = crate::support_matrix::JSON_OUTPUT_PATH;
    let mut artifact: crate::support_matrix::SupportMatrixArtifact =
        serde_json::from_str(&fs::read_to_string(f.root.path().join(path)).unwrap()).unwrap();
    artifact
        .rows
        .retain(|r| r.agent != "codex" || r.version != "1.2.3" || r.target != LINUX);
    let mut row = artifact
        .rows
        .iter()
        .find(|r| r.agent == "codex")
        .unwrap()
        .clone();
    row.version = "1.2.3".into();
    row.target = LINUX.into();
    row.depth = crate::support_matrix::depth::derive_evaluated_depth_facts(
        Some(&record),
        false,
        LINUX,
        &|p| {
            fs::read_to_string(f.root.path().join(p))
                .ok()
                .map(|raw| artifacts::digest(&raw))
        },
        &crate::depth_evidence::test_results(verified.then_some(&record)),
    )
    .unwrap();
    assert_eq!(row.depth.as_ref().unwrap().promises[0].qualified, verified);
    artifact.rows.push(row);
    let markdown = crate::support_matrix::MARKDOWN_OUTPUT_PATH;
    let bundle = crate::support_matrix::render_publication_artifacts(
        &fs::read_to_string(f.root.path().join(markdown)).unwrap(),
        &artifact.rows,
        String::new(),
    )
    .unwrap();
    write(f.root.path(), path, bundle.json);
    write(f.root.path(), markdown, bundle.markdown);
}
#[test]
fn dependency_only_invalidation_cannot_leave_an_existing_promise_qualified() {
    let (mut f, _) = baseline();
    deliver(&f);
    publish(&f, true);
    git(f.root.path(), &["add", "."]);
    git(
        f.root.path(),
        &["commit", "-qm", "previously admitted qualified promise"],
    );
    let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
    check(&f, &tip).unwrap();
    let before = fs::read(f.root.path().join(RECORD)).unwrap();
    // A fixed independent current-binding producer sees changed acquisition, not a resealed E.
    write(
        f.root.path(),
        "cli_manifests/codex/reports/1.2.3/coverage.dependency.json",
        "new dependency",
    );
    f.producer.current.obligations.as_mut().unwrap().identity = "f".repeat(64);
    assert!(check(&f, &tip).unwrap_err().contains("no longer current"));
    assert_eq!(fs::read(f.root.path().join(RECORD)).unwrap(), before);
}
#[test]
fn full_repository_adapter_accepts_the_same_private_closeout_predicate() {
    let (f, tip) = baseline();
    deliver(&f);
    let bundle = crate::support_matrix::generate_publication_artifacts(f.root.path()).unwrap();
    write(
        f.root.path(),
        crate::support_matrix::JSON_OUTPUT_PATH,
        bundle.json,
    );
    write(
        f.root.path(),
        crate::support_matrix::MARKDOWN_OUTPUT_PATH,
        bundle.markdown,
    );
    write(
        f.root.path(),
        crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
        bundle.runtime_support_data,
    );
    git(f.root.path(), &["add", "."]);

    crate::depth_acceptance::integration::check_with(f.root.path(), &tip, &f.producer, &f.producer)
        .unwrap();
}

#[test]
fn forged_request_hash_in_a_matching_closeout_entry_is_not_authority() {
    let (mut f, tip) = baseline();
    deliver(&f);
    let mut value: Value = serde_json::from_slice(&f.closeout).unwrap();
    value["request_sha256"] = json!("b".repeat(64));
    let raw = value.to_string();
    write(f.root.path(), CLOSEOUT, &raw);
    f.record.acceptance_entries = vec![AcceptanceEntry::MaintenanceCloseout {
        closeout: FileRef {
            path: CLOSEOUT.into(),
            sha256: artifacts::digest(&raw),
        },
        record_sha256: f.record.stated_sha256().unwrap(),
    }];
    f.save();
    assert!(check(&f, &tip).unwrap_err().contains("request"));
}
#[test]
fn invalidated_evidence_can_be_reported_unverified_without_reacceptance() {
    let (mut f, _) = baseline();
    deliver(&f);
    publish(&f, true);
    git(f.root.path(), &["add", "."]);
    git(f.root.path(), &["commit", "-qm", "previous qualification"]);
    let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
    write(f.root.path(), EVIDENCE, "invalidated evidence");
    f.producer.proved = false;
    publish(&f, false);
    check(&f, &tip).unwrap();
}
#[test]
fn wrong_lifecycle_entry_refuses_even_alongside_an_admitted_closeout() {
    let (mut f, tip) = baseline();
    deliver(&f);
    f.record = DepthRecord::parse(
        &fs::read_to_string(f.root.path().join(RECORD)).unwrap(),
        "codex",
        "1.2.3",
    )
    .unwrap();
    f.record
        .acceptance_entries
        .push(AcceptanceEntry::ProvingRunCloseout {
            closeout: FileRef {
                path: "docs/agents/lifecycle/codex/governance/proving-run-closeout.json".into(),
                sha256: "b".repeat(64),
            },
            record_sha256: f.record.stated_sha256().unwrap(),
        });
    f.save();
    assert!(check(&f, &tip).unwrap_err().contains("owning authority"));
}
#[test]
fn moved_integration_parent_during_full_admission_refuses() {
    struct Moving<'a>(&'a Producer);
    impl Host for Moving<'_> {
        fn bindings(
            &self,
            root: &Path,
            record: &DepthRecord,
        ) -> Result<(PolicyPart, ObligationsPart), String> {
            let next = git(
                root,
                &[
                    "commit-tree",
                    "HEAD^{tree}",
                    "-p",
                    "HEAD",
                    "-m",
                    "tip moved",
                ],
            );
            git(
                root,
                &["update-ref", "refs/heads/integration-parent", &next],
            );
            self.0.bindings(root, record)
        }
        fn evidence(&self, root: &Path, record: &DepthRecord) -> Result<EvidenceVerdicts, String> {
            self.0.evidence(root, record)
        }
    }
    let (f, tip) = baseline();
    deliver(&f);
    let bundle = crate::support_matrix::generate_publication_artifacts(f.root.path()).unwrap();
    write(
        f.root.path(),
        crate::support_matrix::JSON_OUTPUT_PATH,
        bundle.json,
    );
    write(
        f.root.path(),
        crate::support_matrix::MARKDOWN_OUTPUT_PATH,
        bundle.markdown,
    );
    write(
        f.root.path(),
        crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
        bundle.runtime_support_data,
    );
    git(f.root.path(), &["add", "."]);
    git(
        f.root.path(),
        &["update-ref", "refs/heads/integration-parent", &tip],
    );
    let error = crate::depth_acceptance::integration::check_with(
        f.root.path(),
        "refs/heads/integration-parent",
        &Moving(&f.producer),
        &f.producer,
    )
    .unwrap_err();
    assert!(error.contains("revision moved"), "{error}");
}

#[test]
fn retained_promotion_and_final_admission_agree_after_working_files_disappear() {
    let f = promotions::closed();
    git(f.root.path(), &["add", "."]);
    git(
        f.root.path(),
        &["commit", "-qm", "admitted retained closeout"],
    );
    let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
    git(
        f.root.path(),
        &["update-ref", "refs/remotes/origin/staging", &tip],
    );
    let _ = fs::remove_file(f.root.path().join(REQUEST));
    let _ = fs::remove_file(f.root.path().join(CLOSEOUT));
    let plans = promotions::promote(&f).unwrap();
    crate::workspace_mutation::apply_admitted_mutations(f.root.path(), &plans).unwrap();
    git(f.root.path(), &["add", "."]);
    check(&f, &tip).unwrap();
    assert!(!f.root.path().join(MARKER).exists());
}
#[test]
fn full_adapter_cannot_omit_another_agents_retained_acceptance_effect() {
    let (f, _) = baseline();
    let mut other = serde_json::to_value(&f.record).unwrap();
    other["policy"]["selection"]["agent_id"] = json!("claude_code");
    other["policy"]["selection"]["enrollment"]["modes"][0]["command_path"] =
        json!("claude_code exec");
    canonical(&mut other, "");
    other["policy"]["identity"] = json!(part_identity(&other["policy"]).unwrap());
    other["obligations"]["policy_identity"] = other["policy"]["identity"].clone();
    other["obligations"]["identity"] = json!(part_identity(&other["obligations"]).unwrap());
    other["execution"]["policy_identity"] = other["policy"]["identity"].clone();
    other["execution"]["obligations_identity"] = other["obligations"]["identity"].clone();
    other["execution"]["identity"] = json!(part_identity(&other["execution"]).unwrap());
    let mut record = DepthRecord::parse(&other.to_string(), "claude_code", "1.2.3").unwrap();
    let path = "cli_manifests/claude_code/reports/1.2.3/depth-record.json";
    write(f.root.path(), path, serde_json::to_vec(&record).unwrap());
    git(f.root.path(), &["add", "."]);
    git(f.root.path(), &["commit", "-qm", "another retained tuple"]);
    let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
    record.acceptance_entries.push(AcceptanceEntry::Promotion {
        targets: vec![LINUX.into()],
        record_sha256: record.stated_sha256().unwrap(),
    });
    write(f.root.path(), path, serde_json::to_vec(&record).unwrap());
    write(
        f.root.path(),
        "cli_manifests/claude_code/pointers/latest_validated/x86_64-unknown-linux-gnu.txt",
        "1.2.3\n",
    );
    let error = crate::depth_acceptance::integration::check_with(
        f.root.path(),
        &tip,
        &f.producer,
        &f.producer,
    )
    .unwrap_err();
    assert!(
        error.contains("claude_code") && error.contains("retained closeout"),
        "{error}"
    );
}

#[test]
fn repository_adapter_refuses_unadopted_retained_roots() {
    let (f, tip) = baseline();
    let path = "cli_manifests/codex/RULES.json";
    let mut rules: Value =
        serde_json::from_slice(&fs::read(f.root.path().join(path)).unwrap()).unwrap();
    rules["storage"]["committed"]["reports"]
        .as_object_mut()
        .unwrap()
        .remove("depth_record");
    write(f.root.path(), path, rules.to_string());
    let error = crate::depth_acceptance::integration::check_with(
        f.root.path(),
        &tip,
        &f.producer,
        &f.producer,
    )
    .unwrap_err();
    assert!(error.contains("does not adopt"), "{error}");
}

#[test]
fn reporting_only_changes_in_another_root_require_their_own_generation() {
    for has_request in [true, false] {
        let (f, _) = baseline();
        let path = "cli_manifests/claude_code/reports/1.2.3/depth-record.json";
        let request = crate::agent_lifecycle::maintenance_request_path("claude_code");
        let mut other = serde_json::to_value(&f.record).unwrap();
        other["policy"]["selection"]["agent_id"] = json!("claude_code");
        other["policy"]["selection"]["enrollment"]["modes"][0]["command_path"] =
            json!("claude_code exec");
        canonical(&mut other, "");
        other["policy"]["identity"] = json!(part_identity(&other["policy"]).unwrap());
        other["obligations"]["policy_identity"] = other["policy"]["identity"].clone();
        other["obligations"]["identity"] = json!(part_identity(&other["obligations"]).unwrap());
        other["execution"]["policy_identity"] = other["policy"]["identity"].clone();
        other["execution"]["obligations_identity"] = other["obligations"]["identity"].clone();
        other["execution"]["identity"] = json!(part_identity(&other["execution"]).unwrap());
        write(f.root.path(), path, other.to_string());
        if has_request {
            let raw = format!("request_commit = \"abc\"\nrequest_recorded_at = \"{STAMP}\"\ntrigger_kind = \"upstream_release_detected\"\nopened_from = \"docs/basis.md\"\n[detected_release]\ntarget_version = \"1.2.3\"\n[support_depth]\npolicy_identity = {}\nobligations_identity = {}\n", other["policy"]["identity"], other["obligations"]["identity"]);
            write(f.root.path(), &request, raw);
        } else {
            let _ = fs::remove_file(f.root.path().join(&request));
        }
        git(f.root.path(), &["add", "."]);
        git(f.root.path(), &["commit", "-qm", "unqualified other root"]);
        let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
        other["event"]["opened_from"] = json!("docs/hand-edited.md");
        write(f.root.path(), path, other.to_string());
        // Keep all generated publication truthful: no acceptance effect or qualification.
        let bundle = crate::support_matrix::generate_publication_artifacts(f.root.path()).unwrap();
        for (path, raw) in [
            (crate::support_matrix::JSON_OUTPUT_PATH, bundle.json),
            (crate::support_matrix::MARKDOWN_OUTPUT_PATH, bundle.markdown),
            (
                crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
                bundle.runtime_support_data,
            ),
        ] {
            write(f.root.path(), path, raw);
        }
        git(f.root.path(), &["add", "."]);
        let error = crate::depth_acceptance::integration::check_with(
            f.root.path(),
            &tip,
            &f.producer,
            &f.producer,
        )
        .unwrap_err();
        assert!(
            error.contains("claude_code")
                && (error.contains("request") || error.contains("freeze")),
            "{error}"
        );
        if has_request {
            // A genuine new frozen Event is still an allowed reporting-only change.
            let raw = fs::read_to_string(f.root.path().join(&request)).unwrap();
            write(
                f.root.path(),
                &request,
                raw.replace("docs/basis.md", "docs/hand-edited.md"),
            );
        } else {
            // An unchanged historical record needs no surviving working request.
            other["event"]["opened_from"] = json!("docs/basis.md");
            write(f.root.path(), path, other.to_string());
        }
        crate::depth_acceptance::integration::check_with(
            f.root.path(),
            &tip,
            &f.producer,
            &f.producer,
        )
        .unwrap();
    }
}
