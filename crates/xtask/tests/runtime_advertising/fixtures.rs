use super::*;

pub fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    for path in [
        "Cargo.toml",
        "crates/agent_api/Cargo.toml",
        "crates/xtask/data/agent_registry.toml",
    ] {
        let dest = temp.path().join(path);
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        std::fs::copy(workspace().join(path), dest).unwrap();
    }
    fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &to.join(entry.file_name()));
            } else {
                std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
            }
        }
    }
    copy_tree(
        &workspace().join("crates/agent_api/src"),
        &temp.path().join("crates/agent_api/src"),
    );
    Revision::workspace(temp.path()).expect("valid starting fixture");
    temp
}

pub fn edit(root: &std::path::Path, path: &str, old: &str, new: &str) {
    let path = root.join(path);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(old), "missing fixture discriminator {old}");
    std::fs::write(path, text.replacen(old, new, 1)).unwrap();
}

pub fn regenerate(root: &std::path::Path) {
    let registry = xtask::agent_registry::AgentRegistry::load(root).unwrap();
    let text = xtask::runtime_advertising::projection::render(
        &xtask::runtime_advertising::projection::project(&registry).unwrap(),
    );
    std::fs::write(root.join(xtask::runtime_advertising::DATA), text).unwrap();
}

pub fn git(root: &std::path::Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout).unwrap().trim().into()
}

pub fn commit(root: &std::path::Path) -> String {
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "fixture"]);
    git(root, &["rev-parse", "HEAD"])
}
