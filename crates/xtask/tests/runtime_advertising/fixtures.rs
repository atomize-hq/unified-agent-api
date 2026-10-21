use super::*;

pub fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let mut paths = vec![
        "Cargo.toml".to_string(),
        "crates/agent_api/Cargo.toml".into(),
        "crates/xtask/data/agent_registry.toml".into(),
        "crates/agent_api/src/lib.rs".into(),
        "crates/agent_api/src/backends/mod.rs".into(),
    ];
    for name in ["mod.rs", "evaluator.rs", "data.rs"] {
        paths.push(format!("crates/agent_api/src/runtime_advertising/{name}"));
    }
    for agent in AGENTS {
        for name in ["mod.rs", "backend.rs"] {
            paths.push(format!("crates/agent_api/src/backends/{agent}/{name}"));
        }
    }
    for path in paths {
        let dest = temp.path().join(&path);
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        std::fs::copy(workspace().join(&path), dest).unwrap();
    }
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
