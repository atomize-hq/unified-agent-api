use std::fs;

#[test]
fn publication_validates_registry_before_reading_markdown() {
    let root = tempfile::tempdir().unwrap();
    let registry = root.path().join("crates/xtask/data/agent_registry.toml");
    fs::create_dir_all(registry.parent().unwrap()).unwrap();
    fs::write(registry, "invalid = [").unwrap();

    let expected = xtask::support_matrix::derive_rows(root.path()).unwrap_err();
    assert_eq!(
        xtask::support_matrix::generate_publication_artifacts(root.path()).unwrap_err(),
        expected
    );
}

#[cfg(unix)]
#[test]
fn snapshot_keeps_root_help_when_run_help_fails() {
    use std::os::unix::fs::PermissionsExt;
    use xtask::depth_fixture_snapshot::{run, Args, Error};

    let root = tempfile::tempdir().unwrap();
    let binary = root.path().join("upstream");
    fs::write(
        &binary,
        "#!/bin/sh\ncase \"$*\" in\n  --version) echo 'depth_fixture 0.2.0';;\n  --surface-json) echo '[]';;\n  --help) echo 'root help';;\n  'run --help') exit 1;;\nesac\n",
    )
    .unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    let manifest = root.path().join("manifest");
    let result = run(Args {
        depth_fixture_binary: binary,
        out_file: manifest.join("snapshots/0.2.0/x86_64-unknown-linux-gnu.json"),
        capture_raw_help: true,
        raw_help_target: None,
        supplement: None,
        collected_at: None,
    });

    assert!(matches!(result, Err(Error::CommandFailed(_, _))));
    let help = manifest.join("raw_help/0.2.0/x86_64-unknown-linux-gnu");
    assert_eq!(
        fs::read_to_string(help.join("help.txt")).unwrap(),
        "root help\n"
    );
    assert!(!help.join("run.help.txt").exists());
}
