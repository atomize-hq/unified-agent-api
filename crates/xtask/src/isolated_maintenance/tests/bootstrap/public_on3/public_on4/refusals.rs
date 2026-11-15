//! Refusals traverse the real controller or IN1 and retain the complete caller tree.
use super::*;
use crate::candidate_execution::tree::{self, Node, Tree};
use std::cell::RefCell;
thread_local! {
    static CALLER: RefCell<Option<std::path::PathBuf>> = const { RefCell::new(None) };
}

fn foreign_capability(root: &Path, _: &[String]) -> Result<(), String> {
    let path = root.join(crate::capability_matrix::DEFAULT_OUT_PATH);
    let mut raw = fs::read_to_string(&path).unwrap();
    assert!(!raw.contains("| capability id |"));
    raw.push_str("\n## `agent_api.core`\n\n| capability id | `codex` |\n|---|---|\n| `agent_api.run` | ✅ |\n");
    fs::write(path, raw).unwrap();
    Ok(())
}
fn foreign_pointer(root: &Path, _: &[String]) -> Result<(), String> {
    write(root, "cli_manifests/codex/min_supported.txt", "0.3.0\n");
    Ok(())
}
fn pre_entry(root: &Path, _: &[String]) -> Result<(), String> {
    let mut record: Value = serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
    record["acceptance_entries"] = json!([]);
    write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
    Ok(())
}
fn concurrent(_: &Path, _: &[String]) -> Result<(), String> {
    CALLER.with(|slot| {
        let slot = slot.borrow();
        let caller = slot.as_ref().unwrap();
        let mut bytes = fs::read(caller.join(STATE)).unwrap();
        bytes.push(b'\n');
        fs::write(caller.join(STATE), bytes).unwrap();
    });
    Err("injected ON4 concurrent gate failure".into())
}

pub(super) fn preparation(root: &Path, before: &Tree) {
    type Runner = fn(&Path, &[String]) -> Result<(), String>;
    for (runner, expected) in [
        (
            foreign_capability as Runner,
            "onboarding batch has acceptance outside its owned generation",
        ),
        (
            foreign_pointer as Runner,
            "onboarding cannot change pointer file cli_manifests/codex/min_supported.txt",
        ),
        (
            pre_entry as Runner,
            "onboarding proposal lacks its exact current owning entry",
        ),
    ] {
        eprintln!("ON4 preparation refusal expects {expected}");
        publication_refresh::set_test_gate_runner(Some(runner));
        let error = refresh(root).unwrap_err().to_string();
        assert_eq!(error, expected);
        assert_eq!(&tree::capture(root, &Default::default()).unwrap(), before);
    }
    publication_refresh::set_test_gate_runner(Some(green));
    let error = publication_refresh::routed::with_delivery_failure(1, || refresh(root))
        .unwrap_err()
        .to_string();
    assert_eq!(error, "injected repair delivery failure");
    assert_eq!(&tree::capture(root, &Default::default()).unwrap(), before);

    CALLER.with(|slot| slot.replace(Some(root.into())));
    publication_refresh::set_test_gate_runner(Some(concurrent));
    let original = fs::read(root.join(STATE)).unwrap();
    let error = refresh(root).unwrap_err().to_string();
    assert_eq!(
        error,
        "isolated caller has uncommitted or untracked local changes"
    );
    let mut concurrent_tree = before.clone();
    let Node::File(bytes, _) = concurrent_tree.get_mut(STATE).unwrap() else {
        panic!("missing lifecycle file")
    };
    bytes.push(b'\n');
    assert_eq!(
        tree::capture(root, &Default::default()).unwrap(),
        concurrent_tree
    );
    // The test owns this intentional edit and restores it before the successful sibling.
    write(root, STATE, original);
    CALLER.with(|slot| slot.take());
    publication_refresh::set_test_gate_runner(None);
}

pub(super) fn integration(
    f: &Fixture,
    root: &Path,
    api: &crate::isolated_maintenance::tests::fixture::Fake,
    parent: &str,
    candidate: &str,
) {
    for case in ["packet", "entry", "pointer", "foreign_pointer"] {
        match case {
            "packet" => {
                let mut packet: Value =
                    serde_json::from_slice(&fs::read(root.join(PACKET)).unwrap()).unwrap();
                packet["blocking_issues"] = json!(["forged packet"]);
                let bytes = serde_json::to_vec_pretty(&packet).unwrap();
                let digest = hex::encode(Sha256::digest(&bytes));
                write(root, PACKET, bytes);
                let mut state: life::LifecycleState =
                    serde_json::from_slice(&fs::read(root.join(STATE)).unwrap()).unwrap();
                state.publication_packet_sha256 = Some(digest.clone());
                write(root, STATE, serde_json::to_vec_pretty(&state).unwrap());
                let mut record: Value =
                    serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
                record["acceptance_entries"][0]["packet"]["sha256"] = json!(digest);
                write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
            }
            "entry" => {
                let mut record: Value =
                    serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
                record["acceptance_entries"][0]["packet"]["sha256"] = json!("a".repeat(64));
                write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
            }
            "pointer" => write(root, &format!("{ROOT}/min_supported.txt"), "0.2.0\n"),
            "foreign_pointer" => write(root, "cli_manifests/codex/min_supported.txt", "0.3.0\n"),
            _ => unreachable!(),
        }
        eprintln!("ON4 IN1 refusal case {case}");
        git(root, &["reset", "--soft", parent]);
        commit(root, api, &f.baseline, parent);
        let before = tree::capture(root, &Default::default()).unwrap();
        let error = check_integration(root, parent).unwrap_err();
        let expected = match case {
            "packet" => "onboarding publication packet differs from committed authority",
            "entry" => "integration publication lacks its exact current owning entry",
            "pointer" => "onboarding cannot change pointer file cli_manifests/depth_fixture/min_supported.txt",
            "foreign_pointer" => "onboarding cannot change pointer file cli_manifests/codex/min_supported.txt",
            _ => unreachable!(),
        };
        assert!(error.contains(expected), "{case}: {error}");
        assert_eq!(tree::capture(root, &Default::default()).unwrap(), before);
        git(root, &["reset", "--hard", candidate]);
    }
}

pub(super) fn unchanged(
    f: &Fixture,
    root: &Path,
    host: &Arc<MemoryHost>,
    parent: &str,
    evidence: &SyntheticHost,
) {
    let (api, _route) = install_route(f, root, host);
    commit(root, &api, &f.baseline, parent);
    check_integration(root, parent).unwrap();
    let before = tree::capture(root, &Default::default()).unwrap();
    let revoked = evidence.fork();
    revoked.revoke_adoption();
    let _reader = revoked.install();
    let error = check_integration(root, parent).unwrap_err();
    assert!(
        error.contains("missing exact authenticated adequacy adoption on PR #9"),
        "{error}"
    );
    assert_eq!(tree::capture(root, &Default::default()).unwrap(), before);
}
