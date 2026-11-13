//! Review regressions for authenticated ref binding and byte-oriented Git objects.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(unix)]
fn assert_proposal_error_precedes_binding(missing_tracking_ref: bool) {
    for (subject, onboarding) in [
        (&scope::CODEX, false),
        (&scope::SYNTHETIC, false),
        (&scope::SYNTHETIC, true),
    ] {
        for committed in [false, true] {
            let f = Fixture::new_for(subject);
            if onboarding {
                let canonical = include_str!("../../../../data/agent_registry.toml");
                let declared = include_str!("../../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml");
                write(
                    f.root.path(),
                    REGISTRY_RELATIVE_PATH,
                    format!("{canonical}\n{declared}"),
                );
                f.commit();
            }
            let _installed = f.install();
            let tip = head(&f);
            let link = f.root.path().join("docs/link");
            if !committed {
                write(f.root.path(), ".git/info/exclude", "docs/link\n");
            }
            std::os::unix::fs::symlink("input.txt", &link).unwrap();
            let next = candidate(&f, &tip);
            let parent = if missing_tracking_ref {
                git(
                    f.root.path(),
                    &["update-ref", "-d", "refs/remotes/origin/staging"],
                );
                tip
            } else {
                f.host();
                local(&f, &next);
                next
            };
            let before = snapshot(f.root.path());
            assert_eq!(
                check_integration(f.root.path(), &parent).unwrap_err(),
                format!("{}: source symlink is unsupported", link.display()),
                "subject={}, onboarding={onboarding}, committed={committed}",
                subject.agent_id,
            );
            assert_eq!(snapshot(f.root.path()), before);
        }
    }
}

#[cfg(unix)]
#[test]
fn in1_proposal_error_precedes_equal_head_and_parent() {
    assert_proposal_error_precedes_binding(false);
}

#[cfg(unix)]
#[test]
fn in1_proposal_error_precedes_missing_tracking_ref() {
    assert_proposal_error_precedes_binding(true);
}

#[test]
fn in1_local_tip_ignores_shadow_tags_and_branches() {
    for namespace in ["tags", "heads"] {
        let f = Fixture::new_for(&scope::SYNTHETIC);
        let _installed = f.install();
        let tip = head(&f);
        let next = candidate(&f, &tip);
        let shadow = format!("refs/{namespace}/origin/staging");
        // A mismatching shadow must not defeat a matching remote-tracking tip.
        local(&f, &tip);
        git(f.root.path(), &["update-ref", &shadow, &next]);
        check_integration(f.root.path(), &tip).unwrap();
        // A matching shadow must not conceal a mismatching remote-tracking tip.
        local(&f, &next);
        git(f.root.path(), &["update-ref", &shadow, &tip]);
        let err = refused(&f, &tip);
        assert!(err.contains("local integration ref differs"), "{err}");
    }
}

#[test]
fn in1_missing_tracking_ref_is_not_replaced_by_a_nested_shadow() {
    for namespace in ["tags", "heads"] {
        let f = Fixture::new_for(&scope::SYNTHETIC);
        let _installed = f.install();
        let tip = head(&f);
        // The tracking ref is absent; a ref spelled `refs/remotes/origin/staging` under another
        // namespace points at authenticated T and must not stand in for it.
        git(
            f.root.path(),
            &["update-ref", "-d", "refs/remotes/origin/staging"],
        );
        let shadow = format!("refs/{namespace}/refs/remotes/origin/staging");
        git(f.root.path(), &["update-ref", &shadow, &tip]);
        let err = refused(&f, &tip);
        assert!(err.contains("is absent"), "{err}");
    }
}

#[test]
fn in1_admission_and_observation_accept_non_utf8_commit_message() {
    use std::{io::Write, process::Stdio};
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let _installed = f.install();
    let tip = head(&f);
    local(&f, &tip);
    let ordinary = candidate(&f, &tip);
    let mut object = crate::git_inputs::command(f.root.path())
        .args(["cat-file", "commit", &ordinary])
        .output()
        .unwrap()
        .stdout;
    object.extend_from_slice(b"non-UTF-8 message: \xff\n");
    assert!(std::str::from_utf8(&object).is_err());
    let mut child = crate::git_inputs::command(f.root.path())
        .args(["hash-object", "-t", "commit", "-w", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&object).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let next = String::from_utf8(output.stdout).unwrap().trim().to_owned();
    git(f.root.path(), &["reset", "--hard", "--quiet", &next]);
    for ancestor in [&f.baseline, &tip] {
        f.api.put(
            &format!("repos/fixture/isolated/compare/{ancestor}...{next}"),
            json!({"status":"ahead","merge_base_commit":{"sha":ancestor}}),
        );
    }
    f.api.put(
        &format!("repos/fixture/isolated/commits/{next}"),
        json!({"sha":next}),
    );
    let before = snapshot(f.root.path());
    check_integration(f.root.path(), &tip).unwrap();
    f.host();
    local(&f, &next);
    check_integration(f.root.path(), &tip).unwrap();
    assert_eq!(snapshot(f.root.path()), before);
}

/// Move only the caller's parent ref after initial binding, then restore it when closing
/// binding starts. HEAD, index, the authenticated tip and the remote-tracking ref stay fixed.
struct ParentRace {
    api: Arc<Fake>,
    root: PathBuf,
    tip: String,
    head: String,
    reads: AtomicUsize,
}
const PARENT: &str = "refs/heads/in1-parent";
impl Api for ParentRace {
    fn json(&self, endpoint: &str) -> Result<serde_json::Value, String> {
        if endpoint == "repos/fixture/isolated/branches/staging" {
            match self.reads.fetch_add(1, Ordering::SeqCst) + 1 {
                14 => {
                    git(&self.root, &["update-ref", PARENT, &self.head]);
                }
                // H -> H admission performs eighteen more host reads while checking
                // publication. Restore only when the closing binding begins.
                33 => {
                    git(&self.root, &["update-ref", PARENT, &self.tip]);
                }
                _ => {}
            }
        }
        self.api.json(endpoint)
    }
}

#[test]
fn in1_parent_ref_race_cannot_hide_forged_closeout_entry() {
    use crate::{
        agent_maintenance::prepare::{apply_prepare_plan, build_prepare_plan},
        depth_record::{AcceptanceEntry, DepthRecord, FileRef},
    };
    let (f, args) = super::super::prepare::workspace();
    let root = f.root.path();
    {
        let _installed = f.install();
        apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
    }
    f.commit();
    let tip = head(&f);
    local(&f, &tip);
    git(root, &["update-ref", PARENT, &tip]);
    let path = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
    let mut record = DepthRecord::parse(
        &fs::read_to_string(root.join(path)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    record
        .acceptance_entries
        .push(AcceptanceEntry::MaintenanceCloseout {
            closeout: FileRef {
                path: "docs/forged-closeout.json".into(),
                sha256: "0".repeat(64),
            },
            record_sha256: "0".repeat(64),
        });
    write(root, path, serde_json::to_vec_pretty(&record).unwrap());
    let next = candidate(&f, &tip);
    let race = Arc::new(ParentRace {
        api: f.api.clone(),
        root: root.into(),
        tip: tip.clone(),
        head: next,
        reads: AtomicUsize::new(0),
    });
    let baseline = f.baseline.clone();
    let reader = race.clone();
    let _installed = super::super::fixture::install(Box::new(move || {
        (
            super::super::fixture::profile_with(baseline.clone(), &scope::SYNTHETIC),
            reader.clone(),
        )
    }));
    let before = snapshot(root);
    let result = check_integration(root, PARENT);
    // Fixed admission can refuse the forged entry before reaching the closing host read.
    // Complete the ref restoration in that case too, so the endpoint states are identical.
    assert!(race.reads.load(Ordering::SeqCst) >= 14);
    git(root, &["update-ref", PARENT, &tip]);
    assert_eq!(git(root, &["rev-parse", PARENT]), tip);
    assert_eq!(snapshot(root), before);
    let err = result.unwrap_err();
    assert!(
        err.contains("new acceptance entry must describe the current record"),
        "{err}"
    );
}
