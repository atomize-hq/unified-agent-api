use super::*;
const POINTER: &str = "cli_manifests/codex/pointers/latest_validated/x86_64-unknown-linux-gnu.txt";
fn closed() -> Fixture {
    let mut f = Fixture::new();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let paths = std::process::Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["ls-files", "-z"])
        .output()
        .unwrap();
    assert!(paths.status.success());
    for path in paths.stdout.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        let path = std::str::from_utf8(path).unwrap();
        if !f.root.path().join(path).exists() || path == "cli_manifests/codex/RULES.json" {
            write(f.root.path(), path, fs::read(repo.join(path)).unwrap());
        }
    }

    f.record
        .acceptance_entries
        .push(AcceptanceEntry::MaintenanceCloseout {
            closeout: FileRef {
                path: CLOSEOUT.into(),
                sha256: artifacts::digest(std::str::from_utf8(&f.closeout).unwrap()),
            },
            record_sha256: f.record.stated_sha256().unwrap(),
        });
    f.save();
    f.producer.current = f.record.clone();
    f
}
fn promote(f: &Fixture) -> Result<Vec<crate::workspace_mutation::PlannedMutation>, String> {
    Proposal::capture(f.root.path())?.plan_changes(
        f.root.path(),
        vec![
            (POINTER.into(), Some(b"1.2.3\n".to_vec())),
            (MARKER.into(), None),
        ],
        |root, before, after, paths| {
            crate::depth_acceptance::promotion::plan_with(root, before, after, paths, &f.producer)
        },
    )
}
#[test]
fn retained_closeout_promotes_after_both_working_files_are_replaced() {
    let f = closed();
    let request=format!("agent_id = \"codex\"\nrequest_commit = \"abc\"\nrequest_recorded_at = \"{STAMP}\"\ntrigger_kind = \"upstream_release_detected\"\nopened_from = \"docs/basis.md\"\n[detected_release]\ntarget_version = \"1.2.4\"\n");
    write(f.root.path(), REQUEST, &request);
    let mut newer: Value = serde_json::from_slice(&f.closeout).unwrap();
    newer["request_sha256"] = json!(artifacts::digest(&request));
    write(f.root.path(), CLOSEOUT, newer.to_string());
    let plans = promote(&f).unwrap();
    crate::workspace_mutation::apply_admitted_mutations(f.root.path(), &plans).unwrap();
    assert_eq!(
        fs::read_to_string(f.root.path().join(POINTER)).unwrap(),
        "1.2.3\n"
    );
    assert!(!f.root.path().join(MARKER).exists());
    let r = DepthRecord::parse(
        &fs::read_to_string(f.root.path().join(RECORD)).unwrap(),
        "codex",
        "1.2.3",
    )
    .unwrap();
    assert_eq!(r.acceptance_entries.len(), 2);
    assert!(
        matches!(&r.acceptance_entries[1],AcceptanceEntry::Promotion{record_sha256,..} if *record_sha256==r.stated_sha256().unwrap())
    );
}
#[test]
fn stale_execution_or_missing_retained_closeout_preserves_every_output_and_marker() {
    for missing_closeout in [true, false] {
        let mut f = closed();
        if missing_closeout {
            f.record.acceptance_entries.clear();
            f.save();
        } else {
            f.producer.proved = false;
        }
        let before = fs::read(f.root.path().join(RECORD)).unwrap();
        let error = promote(&f).unwrap_err();
        assert!(
            error.contains(if missing_closeout {
                "closeout entry"
            } else {
                "independently satisfied"
            }),
            "{error}"
        );
        assert_eq!(fs::read(f.root.path().join(RECORD)).unwrap(), before);
        assert!(f.root.path().join(MARKER).exists());
        assert!(!f.root.path().join(POINTER).exists());
    }
}
#[test]
fn stale_obligations_debt_wrong_lifecycle_and_wrong_marker_refuse_promotion() {
    for case in ["obligations", "debt", "lifecycle", "marker"] {
        let mut f = closed();
        match case {
            "obligations" => {
                f.producer.current.obligations.as_mut().unwrap().identity = "f".repeat(64);
            }
            "debt" => write(
                f.root.path(),
                NON_TUI_SUPPORT_DEBT_PATH,
                "malformed live inventory",
            ),
            "lifecycle" => {
                let mut r = serde_json::to_value(&f.record).unwrap();
                r["policy"]["selection"]["enrollment"]["lifecycle_path"] = json!("onboarding");
                // Event must follow its changed policy; remove maintenance-only fields.
                r["event"] = json!({"lifecycle_path":"onboarding","approval_ref":"docs/approval.toml","approval_sha256":"a".repeat(64)});
                // A malformed or contradictory record itself also must refuse before delivery.
                write(f.root.path(), RECORD, r.to_string());
            }
            "marker" => {
                write(
                    f.root.path(),
                    MARKER,
                    stand_down::render_marker_toml("codex", "1.2.3", "2026-10-23T13:00:00Z"),
                );
                git(f.root.path(), &["add", "."]);
                git(f.root.path(), &["commit", "-qm", "wrong marker"]);
                git(
                    f.root.path(),
                    &["update-ref", "refs/remotes/origin/staging", "HEAD"],
                );
            }
            _ => unreachable!(),
        }
        let result = promote(&f);
        assert!(result.is_err(), "{case}");
        assert!(f.root.path().join(MARKER).exists());
        assert!(!f.root.path().join(POINTER).exists());
    }
}

#[test]
fn another_agents_retained_destination_is_not_hidden_by_the_requested_agent() {
    let f = closed();
    let mut other = serde_json::to_value(&f.record).unwrap();
    other["policy"]["selection"]["agent_id"] = json!("claude_code");
    other["policy"]["selection"]["enrollment"]["modes"][0]["command_path"] =
        json!("claude_code exec");
    other["acceptance_entries"] = json!([]);
    canonical(&mut other, "");
    other["policy"]["identity"] = json!(part_identity(&other["policy"]).unwrap());
    other["obligations"]["policy_identity"] = other["policy"]["identity"].clone();
    other["obligations"]["identity"] = json!(part_identity(&other["obligations"]).unwrap());
    other["execution"]["policy_identity"] = other["policy"]["identity"].clone();
    other["execution"]["obligations_identity"] = other["obligations"]["identity"].clone();
    other["execution"]["identity"] = json!(part_identity(&other["execution"]).unwrap());
    let other = DepthRecord::parse(&other.to_string(), "claude_code", "1.2.3").unwrap();
    write(
        f.root.path(),
        "cli_manifests/claude_code/reports/1.2.3/depth-record.json",
        serde_json::to_vec(&other).unwrap(),
    );
    let path = "cli_manifests/claude_code/pointers/latest_validated/x86_64-unknown-linux-gnu.txt";
    let result = Proposal::capture(f.root.path()).unwrap().plan_changes(
        f.root.path(),
        vec![(path.into(), Some(b"1.2.3\n".to_vec()))],
        |root, before, after, paths| {
            crate::depth_acceptance::promotion::plan_with(root, before, after, paths, &f.producer)
        },
    );
    assert!(result
        .unwrap_err()
        .contains("retained maintenance closeout entry"));
    assert!(!f.root.path().join(path).exists());
    assert!(f.root.path().join(MARKER).exists());
}
