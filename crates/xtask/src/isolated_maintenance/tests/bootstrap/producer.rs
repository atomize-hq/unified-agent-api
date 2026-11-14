//! Use the actual built upstream and acquisition writers, including raw help.
use super::*;

fn binary() -> &'static Path {
    static BUILD: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    BUILD.get_or_init(|| {
        let directory = tempfile::tempdir().unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../depth_fixture/src/bin");
        write(directory.path(), "src/main.rs", fs::read(source.join("depth_fixture.rs")).unwrap());
        write(directory.path(), "src/depth_fixture/help.rs", fs::read(source.join("depth_fixture/help.rs")).unwrap());
        write(directory.path(), "Cargo.toml", "[package]\nname = \"bootstrap-upstream\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[dependencies]\nserde_json = \"1\"\n");
        let output = std::process::Command::new(env!("CARGO"))
            .args(["build", "--offline", "--quiet"])
            .arg("--target-dir")
            .arg(directory.path().join("target"))
            .current_dir(directory.path())
            .env("DEPTH_FIXTURE_UPSTREAM_VERSION", "0.2.0")
            .output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        directory
    }).path()
}

pub(super) fn acquire(root: &Path) {
    let base = root.join(ROOT);
    crate::depth_fixture_snapshot::run(crate::depth_fixture_snapshot::Args {
        depth_fixture_binary: binary().join("target/debug/bootstrap-upstream"),
        out_file: base.join("snapshots/0.2.0/x86_64-unknown-linux-gnu.json"),
        capture_raw_help: true,
        raw_help_target: Some("x86_64-unknown-linux-gnu".into()),
        supplement: None,
        collected_at: Some("2026-11-03T00:00:00Z".into()),
    })
    .unwrap();
    crate::depth_fixture_wrapper_coverage::run(crate::depth_fixture_wrapper_coverage::CliArgs {
        out: base.join("wrapper_coverage.json"),
        rules: base.join("RULES.json"),
    })
    .unwrap();
    crate::manifest_union::run_at(
        crate::manifest_union::Args {
            root: Some(base.clone()),
            rules: None,
            version: "0.2.0".into(),
        },
        "2026-11-03T00:00:00Z",
    )
    .unwrap();
    crate::manifest_report::run_at(
        crate::manifest_report::Args {
            root: Some(base),
            rules: None,
            version: "0.2.0".into(),
        },
        "2026-11-03T00:00:00Z",
    )
    .unwrap();
}

#[test]
fn bootstrap_preparation_admits_actual_producer_and_refuses_raw_help_drift() {
    let f = setup(true);
    let _installed = f.install();
    let parent = head(&f);
    adopt(&f);
    integrate(&f, &parent);
    let parent = head(&f);
    acquire(f.root.path());
    integrate(&f, &parent);
    let valid = head(&f);
    for (path, bytes) in [
        (
            "raw_help/0.2.0/x86_64-unknown-linux-gnu/extra.txt",
            "foreign",
        ),
        ("raw_help/0.2.0/foreign/help.txt", "foreign"),
        (
            "raw_help/0.3.0/x86_64-unknown-linux-gnu/help.txt",
            "foreign",
        ),
        (
            "raw_help/0.2.0/x86_64-unknown-linux-gnu/help.txt",
            "changed\n",
        ),
        (
            "raw_help/0.2.0/x86_64-unknown-linux-gnu/run.help.txt",
            "changed\n",
        ),
    ] {
        git(f.root.path(), &["reset", "--hard", &valid]);
        git(f.root.path(), &["clean", "-fd"]);
        write(f.root.path(), &format!("{ROOT}/{path}"), bytes);
        candidate(&f, &valid);
        assert!(check_integration(f.root.path(), &valid).is_err(), "{path}");
    }
}

pub(super) fn encode_shard(value: &Value) -> String {
    let value: crate::manifest_snapshot_schema::SnapshotV1 =
        serde_json::from_value(value.clone()).unwrap();
    format!("{}\n", serde_json::to_string_pretty(&value).unwrap())
}
pub(super) fn encode_wrapper(value: &Value) -> String {
    let typed: depth_fixture::wrapper_coverage_manifest::WrapperCoverageManifestV1 =
        serde_json::from_value(value.clone()).unwrap();
    if serde_json::to_value(&typed).unwrap() != *value {
        return value.to_string(); // Preserve deliberately unsupported fields in refusal fixtures.
    }
    format!("{}\n", serde_json::to_string_pretty(&typed).unwrap())
}
