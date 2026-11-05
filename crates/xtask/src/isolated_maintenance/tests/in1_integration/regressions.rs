//! Review regressions for authenticated ref binding and byte-oriented Git objects.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

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
