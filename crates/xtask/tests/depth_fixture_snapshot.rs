//! `depth-fixture-snapshot`: the synthetic subject's per-target snapshot (SA-P1).

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SURFACE: &str = r#"[{"path":["run"],"about":"Run one prompt","flags":[{"long":"--json","takes_value":false}]},{"path":[],"about":"root"}]"#;

fn fake_binary(dir: &Path, version_line: &str) -> PathBuf {
    let path = dir.join("depth_fixture");
    let script = format!(
        "#!/bin/sh\nPATH=/usr/bin:/bin\ncase \"$1\" in\n  --version) echo '{version_line}' ;;\n  --surface-json) echo '{SURFACE}' ;;\n  --help) echo 'root help' ;;\n  run) echo 'run help' ;;\n  *) exit 2 ;;\nesac\n"
    );
    fs::write(&path, script).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn snapshot(binary: &Path, out_file: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("depth-fixture-snapshot")
        .arg("--depth-fixture-binary")
        .arg(binary)
        .arg("--out-file")
        .arg(out_file)
        .args(extra)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn writes_a_sorted_per_target_snapshot_and_raw_help() {
    let dir = tempfile::tempdir().unwrap();
    let binary = fake_binary(dir.path(), "depth_fixture 0.2.0");
    let root = dir.path().join("cli_manifests/depth_fixture");
    let out = root.join("snapshots/0.2.0/linux-x64.json");
    let output = snapshot(
        &binary,
        &out,
        &[
            "--capture-raw-help",
            "--raw-help-target",
            "linux-x64",
            "--collected-at",
            "2026-11-02T00:00:00Z",
        ],
    );
    assert!(output.status.success(), "{}", stderr(&output));

    let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(json["tool"], "depth-fixture-cli");
    assert_eq!(json["collected_at"], "2026-11-02T00:00:00Z");
    assert_eq!(json["binary"]["target_triple"], "linux-x64");
    assert_eq!(json["binary"]["semantic_version"], "0.2.0");
    assert_eq!(json["binary"]["version_output"], "depth_fixture 0.2.0");
    let bytes = fs::read(&binary).unwrap();
    assert_eq!(json["binary"]["size_bytes"], bytes.len() as u64);
    let paths: Vec<_> = json["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["path"].clone())
        .collect();
    assert_eq!(
        paths,
        vec![serde_json::json!([]), serde_json::json!(["run"])]
    );

    let raw = root.join("raw_help/0.2.0/linux-x64");
    assert_eq!(
        fs::read_to_string(raw.join("help.txt")).unwrap(),
        "root help\n"
    );
    assert_eq!(
        fs::read_to_string(raw.join("run.help.txt")).unwrap(),
        "run help\n"
    );
}

#[test]
fn refuses_inputs_outside_its_contract() {
    let dir = tempfile::tempdir().unwrap();
    let binary = fake_binary(dir.path(), "depth_fixture 0.1.0");
    let good = dir.path().join("root/snapshots/0.1.0/linux-x64.json");
    let supplement = dir.path().join("commands.json");
    let cases: Vec<(PathBuf, Vec<&str>, &str)> = vec![
        (
            dir.path().join("root/snapshots/0.2.0/linux-x64.json"),
            vec![],
            "expected <root>/snapshots/0.1.0/<target>.json",
        ),
        (
            dir.path().join("root/other/0.1.0/linux-x64.json"),
            vec![],
            "expected <root>/snapshots/0.1.0/<target>.json",
        ),
        (
            good.clone(),
            vec!["--raw-help-target", "darwin-arm64"],
            "differs from the out-file target",
        ),
        (
            good.clone(),
            vec!["--supplement", supplement.to_str().unwrap()],
            "has no supplement",
        ),
        (
            good.clone(),
            vec!["--collected-at", "yesterday"],
            "invalid collected_at",
        ),
    ];
    for (out, extra, expected) in cases {
        let output = snapshot(&binary, &out, &extra);
        assert!(!output.status.success());
        assert!(stderr(&output).contains(expected), "{}", stderr(&output));
        assert!(!out.exists());
    }

    let wrong = dir.path().join("wrong");
    fs::create_dir(&wrong).unwrap();
    let output = snapshot(&fake_binary(&wrong, "other 0.1.0"), &good, &[]);
    assert!(
        stderr(&output).contains("unexpected version output"),
        "{}",
        stderr(&output)
    );
    let output = snapshot(Path::new("/nonexistent/depth_fixture"), &good, &[]);
    assert!(stderr(&output).contains("invalid depth_fixture binary path"));
    assert!(!good.exists());
}
