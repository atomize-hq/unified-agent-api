use super::*;
fn fixture() -> (
    Observation,
    Expectations,
    Vec<CaseIdentity>,
    Vec<CaseResult>,
    Vec<FileRef>,
) {
    let case = CaseIdentity {
        name: "native/context-test".into(),
        executable_sha256: "a".repeat(64),
    };
    let source = FileRef {
        path: "approved/context-source.json".into(),
        sha256: "b".repeat(64),
    };
    let bindings = Fact::ALL
        .into_iter()
        .map(|fact| Binding {
            fact,
            source: source.clone(),
            corroboration: case.clone(),
        })
        .collect::<Vec<_>>();
    let launch = Launch {
        route: Route::InitialExec,
        spawned: false,
        binary_matches: true,
        cwd_matches: true,
        home_matches: true,
        environment_known: true,
    };
    let c = CaseContext {
        case: case.clone(),
        route: Route::InitialExec,
        local_before: true,
        local_after: true,
        launches: vec![
            launch.clone(),
            Launch {
                spawned: true,
                ..launch
            },
        ],
        config: None,
        bindings: bindings.clone(),
        missing: vec![],
        derived_defaults: vec![],
        diagnostics_valid: true,
    };
    let expected = Expectations {
        cases: vec![ExpectedCase {
            case: case.clone(),
            route: c.route,
            config: None,
            bindings,
            derived_defaults: vec![],
        }],
    };
    (
        Observation {
            schema_version: 1,
            complete: true,
            cases: vec![c],
        },
        expected,
        vec![case.clone()],
        vec![CaseResult {
            case,
            outcome: CaseOutcome::Passed,
        }],
        vec![source],
    )
}
#[test]
fn context_requires_exact_cases_bound_facts_and_not_a_complete_flag() {
    let (obs, e, cases, results, inputs) = fixture();
    assert!(obs
        .validate(&e, &cases, &results, &inputs, &"f".repeat(64))
        .is_ok());
    for attack in 0..10 {
        let mut c = obs.clone();
        match attack {
            0 => c.cases[0].bindings.pop().map(|_| ()).unwrap(),
            1 => c.cases[0].local_after = false,
            2 => c.cases[0].launches[0].environment_known = false,
            3 => c.cases[0].route = Route::AppServer,
            4 => c.cases.push(c.cases[0].clone()),
            5 => c.cases[0].launches.reverse(),
            6 => c.cases[0].bindings[0].source.sha256 = "0".repeat(64),
            7 => c.cases[0].diagnostics_valid = false,
            8 => c.cases[0].bindings[0].corroboration.name = "other".into(),
            _ => c.schema_version = 2,
        };
        assert!(
            c.validate(&e, &cases, &results, &inputs, &"f".repeat(64))
                .is_err(),
            "attack {attack}"
        );
    }
    assert!(obs
        .validate(&e, &cases, &[], &inputs, &"f".repeat(64))
        .is_err());
    let mut failed = results;
    failed[0].outcome = CaseOutcome::Failed;
    assert!(obs
        .validate(&e, &cases, &failed, &inputs, &"f".repeat(64))
        .is_ok()); // A failed assertion does not erase directly observed context.
}
#[test]
fn named_defaults_require_correspondence_absence_precedence_and_native_case() {
    let (mut obs, mut e, cases, results, inputs) = fixture();
    let d = DerivedDefault {
        id: DefaultId::ToolRegistryErrorOnCollisions,
        value: false,
        source: inputs[0].clone(),
        upstream_commit: "b412ff32c417f855c2b2d1581b77058eed87c84b".into(),
        binary_sha256: "f".repeat(64),
        overrides_absent: true,
        precedence_established: true,
        corroboration: cases[0].clone(),
    };
    e.cases[0].derived_defaults.push(d.clone());
    obs.cases[0].derived_defaults.push(d);
    assert!(obs
        .validate(&e, &cases, &results, &inputs, &"f".repeat(64))
        .is_ok());
    for attack in 0..5 {
        let mut c = obs.clone();
        let mut expected = e.clone();
        let d = &mut c.cases[0].derived_defaults[0];
        match attack {
            0 => d.overrides_absent = false,
            1 => d.precedence_established = false,
            2 => d.binary_sha256 = "e".repeat(64),
            3 => d.upstream_commit = "0".repeat(40),
            _ => d.corroboration.name = "not-executed".into(),
        };
        // Even a source-owned matching expected claim cannot waive these fixed predicates.
        expected.cases[0].derived_defaults = c.cases[0].derived_defaults.clone();
        assert!(c
            .validate(&expected, &cases, &results, &inputs, &"f".repeat(64))
            .is_err());
    }
}
#[test]
fn malformed_context_errors_withhold_secret_values() {
    for raw in [
        br#"{"schema_version":"sk-sentinel"}"#.as_slice(),
        br#"{"schema_version":1,"complete":true,"cases":[],"secret":"sk-sentinel"}"#,
    ] {
        let err = Observation::parse(raw).unwrap_err();
        assert!(!err.contains("sentinel"));
    }
}
#[test]
#[cfg(target_os = "linux")]
fn local_guard_preserves_auth_and_rejects_aliases_loaders_and_nonsecret_drift() {
    use std::{collections::BTreeMap, ffi::OsString, fs};
    let tmp = tempfile::tempdir_in("/tmp").unwrap();
    let home = tmp.path().join("home");
    let work = tmp.path().join("work");
    fs::create_dir(&home).unwrap();
    fs::create_dir(&work).unwrap();
    fs::write(home.join("auth.json"), "sk-never-read-or-hashed").unwrap();
    fs::write(home.join("config.toml"), "model = 'gpt-5'\n").unwrap();
    let env: BTreeMap<OsString, OsString> =
        BTreeMap::from([("HOME".into(), tmp.path().as_os_str().into())]);
    let g = guard::LocalGuard::inspect_with_system(
        &home,
        &work,
        &env,
        &tmp.path().join("absent-system"),
    )
    .unwrap();
    fs::write(home.join("auth.json"), "refreshed-auth-native-owned").unwrap();
    assert!(g.recheck(&env).is_ok());
    fs::write(home.join("config.toml"), "model_provider = 'openai'\n").unwrap();
    assert_eq!(g.recheck(&env), Err(guard::Refusal::InputDrift));
    fs::write(home.join("config.toml"), "model='sk-secret-config'\n").unwrap();
    assert_eq!(
        guard::LocalGuard::inspect_with_system(
            &home,
            &work,
            &env,
            &tmp.path().join("absent-system")
        )
        .unwrap_err(),
        guard::Refusal::Config
    );
    fs::write(home.join("config.toml"), "").unwrap();
    fs::write(work.join("AGENTS.md"), "foreign instruction sk-secret").unwrap();
    assert_eq!(
        guard::LocalGuard::inspect_with_system(
            &home,
            &work,
            &env,
            &tmp.path().join("absent-system")
        )
        .unwrap_err(),
        guard::Refusal::ForeignLoader
    );
    fs::remove_file(work.join("AGENTS.md")).unwrap();
    assert!(guard::role(&work.join(".")).is_err());
    assert!(guard::role(&work.join("../work")).is_err());
    assert!(guard::LocalGuard::inspect(&home, &home, &env).is_err());
    #[cfg(unix)]
    {
        let alias = tmp.path().join("alias");
        std::os::unix::fs::symlink(&work, &alias).unwrap();
        assert!(guard::role(&alias).is_err());
    }
    assert_eq!(
        fs::read_to_string(home.join("auth.json")).unwrap(),
        "refreshed-auth-native-owned"
    );
}
