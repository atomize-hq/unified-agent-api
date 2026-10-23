#[test]
fn bounded_archive_provenance_and_tamper_cases() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let result = std::process::Command::new("python3")
        .args([
            "-m",
            "unittest",
            "discover",
            "-s",
            "scripts",
            "-p",
            "test_depth_ci_archive.py",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
