use super::*;
#[test]
fn unknown_and_nonunicode_environment_channels_refuse_without_retaining_values() {
    let mut env = BTreeMap::from([
        ("HOME".into(), "/private/home".into()),
        ("OPENAI_API_KEY".into(), "sk-sentinel".into()),
    ]);
    assert!(environment_known(&env));
    env.insert("HTTP_PROXY".into(), "secret-proxy".into());
    assert!(!environment_known(&env));
    let p = Launch {
        route: Route::InitialExec,
        spawned: false,
        binary_matches: false,
        cwd_matches: false,
        home_matches: false,
        environment_known: environment_known(&env),
    };
    let raw = serde_json::to_string(&p).unwrap();
    assert!(!raw.contains("sentinel") && !raw.contains("proxy") && !raw.contains("private"));
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        env.remove(std::ffi::OsStr::new("HTTP_PROXY"));
        env.insert(OsString::from_vec(vec![0xff]), "sentinel".into());
        assert!(!environment_known(&env));
    }
}
#[test]
fn effective_environment_respects_removal_and_last_overlay() {
    let mut cmd = Command::new("not-executed");
    cmd.env("HOME", "/final/home");
    cmd.env("HTTP_PROXY", "secret");
    cmd.env_remove("HTTP_PROXY");
    let env = effective_environment(&cmd);
    assert_eq!(
        env.get(std::ffi::OsStr::new("HOME")),
        Some(&OsString::from("/final/home"))
    );
    assert!(!env.contains_key(std::ffi::OsStr::new("HTTP_PROXY")));
}
