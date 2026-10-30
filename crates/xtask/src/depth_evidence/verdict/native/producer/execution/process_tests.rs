use super::*;
#[test]
fn fake_collector_uses_foreground_child_and_retains_only_finite_journal_fields() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let prefix = "cli_manifests/codex/reports/0.156.1/evidence/native-v1/attempt";
    fs::create_dir_all(root.join(prefix)).unwrap();
    let project = tempfile::tempdir().unwrap();
    let executable = root.join("fake-closed-collector");
    // This fake only produces a finite failed case. Neither it nor the test invokes Codex.
    let sha = "b".repeat(64);
    let step = form::NativeStep {
        case: requested(&sha)[0].clone(),
        observation: form::NativeObservation::default(),
        outcome: CaseOutcome::Failed,
        elapsed_ms: 1,
        failure: Some(form::NativeFailure::RequestRefused),
    };
    // Harness identity is checked before spawn; self-reference is avoided by substituting the
    // actual measured identity into the journal at runtime using the sha256sum system utility.
    let step_json = String::from_utf8(canonical(&step).unwrap()).unwrap();
    let script=format!("#!/bin/sh\nwhile [ $# -gt 0 ]; do\n case \"$1\" in --workspace) shift; workspace=$1;; --output-dir) shift; output=$1;; esac\n shift\ndone\nmkdir \"$workspace/$output\"\ndigest=$(/usr/bin/sha256sum \"$0\"); digest=${{digest%% *}}\ncat <<'FINITE' | /usr/bin/sed \"s/{sha}/$digest/g\" > \"$workspace/$output/case-0.json\"\n{step_json}\nFINITE\necho sk-secret-native-stdout\necho sk-secret-native-error >&2\nexit 1\n");
    fs::write(&executable, script).unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let digest = fingerprint(&executable).unwrap();
    let binary = root.join("native-not-executed");
    fs::write(&binary, b"native bytes only").unwrap();
    let mut record = crate::depth_ci::profile::Profile::Core
        .record(include_str!(
            "../../../../../../tests/fixtures/depth-core/record.json"
        ))
        .unwrap();
    let obligation = crate::depth_record::due(&record)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let scope = Scope {
        agent_id: "codex".into(),
        version: "0.156.1".into(),
        event: record.event.clone(),
        enrollment_origin: EnrollmentOrigin::Isolated,
        obligation,
        policy_identity: record.policy.identity.clone(),
        obligations_identity: record.obligations.take().unwrap().identity,
        rules_revision: "1".into(),
        trust_revision: "maintainer-native-v1".into(),
        review_revision: "maintainer-mapping-v1".into(),
    };
    let file = crate::depth_ci::reference("source", b"source");
    let build = Build {
        source_commit: "a".repeat(40),
        source_tree: "b".repeat(40),
        host_target: crate::depth_ci::TARGET.into(),
        toolchain: file.clone(),
        features: vec![],
        environment: file.clone(),
        dependency_graph: file.clone(),
        covered_inputs: vec![file.clone()],
        absent_inputs: vec![],
    };
    let plan = form::Plan {
        schema_version: 1,
        scope,
        packet_head: "c".repeat(40),
        build: build.clone(),
        native: form::Runtime {
            version: "0.156.1".into(),
            target: "x86_64-unknown-linux-musl".into(),
            executable_sha256: fingerprint(&binary).unwrap(),
            acquisition: file.clone(),
        },
        context: crate::depth_ci::reference(format!("{prefix}/intent.json"), b"intent"),
        collector: file,
        requested: requested(&digest),
    };
    let built = crate::depth_ci::native::Built {
        _target: tempfile::tempdir().unwrap(),
        executable,
        executable_sha256: digest,
        build,
        payloads: vec![],
    };
    let args = CollectArgs {
        live: true,
        register: true,
        binary: binary.clone(),
        working_dir: project.path().into(),
        codex_home: None,
        attempt: "attempt".into(),
    };
    let result = run(
        root,
        &args,
        &binary,
        &built,
        &plan,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    assert!(!result.complete);
    assert!(!result.context.complete);
    assert_eq!(result.cases.requested.len(), 28);
    assert_eq!(result.cases.executed.len(), 1);
    assert_eq!(result.cases.executed[0].outcome, CaseOutcome::Failed);
    let retained = serde_json::to_string(&result.observations).unwrap();
    assert!(!retained.contains("sk-secret"));
    let alias = root.join("project-alias");
    std::os::unix::fs::symlink(project.path(), &alias).unwrap();
    let mut args = args;
    for path in [alias, project.path().join(".")] {
        args.working_dir = path;
        let error = match run(
            root,
            &args,
            &binary,
            &built,
            &plan,
            &std::sync::atomic::AtomicBool::new(false),
        ) {
            Err(error) => error,
            Ok(_) => panic!("aliased project must refuse before spawn"),
        };
        assert_eq!(error, "native working directory role refused");
    }
}
