use super::*;
use serde_json::json;
use std::sync::Arc;

#[test]
fn reporting_dispatcher_retains_captured_inputs_on_upload_changes() {
    let (scratch, publication, host) = crate::isolated_maintenance::admission_fixture();
    let root = scratch.path();
    let path = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
    std::fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
    let bytes = include_str!("../../../tests/fixtures/depth-synthetic/record.json");
    std::fs::write(root.join(path), bytes).unwrap();
    crate::depth_ci::output(crate::git_inputs::command(root).args(["add", path])).unwrap();
    crate::depth_ci::output(crate::git_inputs::command(root).args([
        "commit",
        "--quiet",
        "-m",
        "reporting generation",
    ]))
    .unwrap();
    let record = crate::depth_record::DepthRecord::parse(bytes, "depth_fixture", "0.2.0").unwrap();
    publication_failures(root, &publication, &host, &record, |_, _, _, _| {});
}

/// Called by both genuine-writer proofs with their real retained history and population API.
pub(crate) fn publication_failures(
    root: &Path,
    publication: &Publication,
    host: &std::sync::Arc<admission::fake::MemoryHost>,
    record: &crate::depth_record::DepthRecord,
    check_population: impl Fn(&Publication, &Arc<admission::fake::MemoryHost>, &ContextRecord, &str),
) {
    use std::sync::Mutex;
    let candidate = String::from_utf8(
        crate::depth_ci::output(crate::git_inputs::command(root).args(["rev-parse", "HEAD"]))
            .unwrap(),
    )
    .unwrap()
    .trim()
    .to_owned();
    for mutation in 0..9 {
        let host = Arc::new(host.fork());
        let publication = Publication {
            authority: publication.authority.clone(),
            host: host.clone(),
        };
        let candidate_object = admission::emitter::object(root, &candidate).unwrap();
        let parent = String::from_utf8(
            crate::depth_ci::output(crate::git_inputs::command(root).args(["rev-parse", "HEAD^"]))
                .unwrap(),
        )
        .unwrap()
        .trim()
        .to_owned();
        let claims = host
            .begin(&candidate, &authority(&publication).policy)
            .unwrap();
        let invocation = Invocation {
            claims: claims.clone(),
            checkout: candidate_object,
            checkout_parents: vec![parent],
            dispatch_inputs: [("depth_mode".into(), KIND.into())].into(),
            configuration: [("profile".into(), "synthetic-wrapper-v1".into())].into(),
            substantive_invoked: false,
        };
        let candidate_path = format!("reporting/candidates/{candidate}");
        let attempt_path = format!("reporting/invocations/{}/1", claims.run);
        host.put(&candidate_path, serde_json::to_value(&invocation).unwrap());
        host.put("reporting/current", json!(candidate));
        host.put(&attempt_path, serde_json::to_value(&invocation).unwrap());
        let captured = Arc::new(Mutex::new(None));
        let before = captured.clone();
        let directory = root.to_owned();
        let changed_candidate = candidate_path.clone();
        let execution_path = format!("reporting/execution/{}/1", claims.run);
        let changed_execution = execution_path.clone();
        let payload_path = format!("reporting-payload/{}-1", claims.run);
        let uploaded_payload = payload_path.clone();
        *host.after_reporting_upload.lock().unwrap() = Some(Box::new(move |host| {
            let bytes = host.bytes(&uploaded_payload, RECORD_LIMIT).unwrap();
            let context = ContextRecord::parse(&bytes).unwrap();
            *before.lock().unwrap() = Some(context.clone());
            let git = |args: &[&str]| {
                crate::depth_ci::output(crate::git_inputs::command(&directory).args(args)).unwrap();
            };
            match mutation {
                0 => {
                    std::fs::write(
                        directory.join("docs/upload-added.txt"),
                        b"staged during upload",
                    )
                    .unwrap();
                    git(&["add", "docs/upload-added.txt"]);
                }
                1 => git(&["rm", "--quiet", "docs/input.txt"]),
                2 => git(&["update-index", "--chmod=+x", "docs/input.txt"]),
                3 => {
                    std::fs::write(directory.join("docs/input.txt"), b"changed during upload")
                        .unwrap();
                    git(&["add", "docs/input.txt"]);
                }
                4 => {
                    let changed =
                        crate::depth_ci::output(crate::git_inputs::command(&directory).args([
                            "commit-tree",
                            &context.candidate.tree,
                            "-p",
                            &context.candidate.commit,
                            "-m",
                            "same-tree checkout change during upload",
                        ]))
                        .unwrap();
                    let changed = std::str::from_utf8(&changed).unwrap().trim();
                    git(&["reset", "--hard", changed]);
                }
                5 => host.change(&changed_candidate, |v| {
                    v["configuration"]["profile"] = json!("changed")
                }),
                6 => host.put(&changed_execution, json!(true)),
                7 => host
                    .fail_reporting_execution_once
                    .store(true, std::sync::atomic::Ordering::SeqCst),
                _ => host.put("reporting/current", json!("f".repeat(40))),
            }
        }));
        let error = dispatch(root, &publication, &candidate, record).unwrap_err();
        assert_eq!(
            error, "reporting collection/publication incomplete",
            "mutation {mutation}"
        );
        let retained =
            ContextRecord::parse(&host.bytes(&payload_path, RECORD_LIMIT).unwrap()).unwrap();
        let mut expected = captured.lock().unwrap().take().unwrap();
        assert!(expected.successful());
        expected.collection_complete = false;
        expected.reporting_outcome = "failure".into();
        assert_eq!(
            retained, expected,
            "captured identities changed for mutation {mutation}"
        );
        assert_eq!(
            authenticate(root, &publication, claims.run, 1).unwrap(),
            retained
        );
        crate::depth_ci::output(
            crate::git_inputs::command(root).args(["reset", "--hard", &candidate]),
        )
        .unwrap();
        assert!(!host
            .fail_reporting_execution_once
            .load(std::sync::atomic::Ordering::SeqCst));
        let refusal = capture(
            root,
            &publication,
            &retained.candidate,
            &retained.predecessor,
            &retained.generation,
            &retained.predecessor_admissions,
        )
        .unwrap_err();
        assert_eq!(
            refusal,
            if mutation == 6 {
                "reporting context has substantive execution observation"
            } else {
                "reporting context candidate/predecessor/generation/dispatch mismatch"
            }
        );
        check_population(
            &publication,
            &host,
            &retained,
            if mutation == 6 {
                "missing/ambiguous attempt witness artifact"
            } else {
                "failed reporting producer or incomplete collection"
            },
        );
    }
}

#[test]
fn reporting_context_literal_canonical_bytes_digest_version_and_bounds() {
    let bytes = include_bytes!("context-v1.json");
    let record = ContextRecord::parse(bytes).unwrap();
    assert_eq!(record.canonical().unwrap(), bytes);
    assert_eq!(
        record.identity().unwrap(),
        "63e791e20e3d0828b64e1e258fe565868afa6d39fe673379da12207c5e4e4c6a"
    );
    assert_ne!(record.identity().unwrap(), digest(admission::DOMAIN, bytes));
    let mut spaced = bytes.to_vec();
    spaced.push(b'\n');
    assert_eq!(
        ContextRecord::parse(&spaced).unwrap_err(),
        "noncanonical reporting context"
    );
    for field in ["schema_version", "unknown", "selected_mode"] {
        let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        value[field] = json!(2);
        assert!(ContextRecord::parse(&canonical(&value).unwrap()).is_err());
    }
    assert_eq!(
        ContextRecord::parse(&vec![b' '; RECORD_LIMIT + 1]).unwrap_err(),
        "reporting context exceeds 64 KiB"
    );
    let mut large = record.clone();
    large.repository.name = "x".repeat(RECORD_LIMIT);
    assert_eq!(
        large.canonical().unwrap_err(),
        "reporting context exceeds 64 KiB"
    );
    for mutation in 0..5 {
        let mut context = record.clone();
        match mutation {
            0 => context.invocation.substantive_invoked = true,
            1 => context.enrolled_substantive_path_not_invoked = false,
            2 => context.reporting_outcome = "failure".into(),
            3 => context.reporting_outcome = "aborted".into(),
            _ => context.collection_complete = false,
        }
        assert!(!context.successful());
    }
}
