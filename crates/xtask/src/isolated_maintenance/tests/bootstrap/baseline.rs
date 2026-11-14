//! Explicit subject-absent inputs, independent of the build workspace's declaration.
use super::*;

const INPUTS: &[&str] = &[
    REGISTRY_RELATIVE_PATH,
    APPROVAL,
    "Cargo.toml",
    "docs/crates-io-release.md",
];

/// Readers share one immutable baseline; each test copies only the inputs it needs.
fn workspace() -> &'static Path {
    static BASELINE: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    BASELINE
        .get_or_init(|| construct(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")))
        .path()
}

fn construct(source: &Path) -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let ordinary = registry_fixture::undeclared_registry();
    write(root, REGISTRY_RELATIVE_PATH, ordinary);
    write(
        root,
        APPROVAL,
        APPROVAL_TEMPLATE
            .replace("{{APPROVAL_COMMIT}}", "abc1234")
            .replace("{{APPROVAL_RECORDED_AT}}", "2026-11-05T00:00:00Z"),
    );
    write(root, "Cargo.toml", "[workspace]\nmembers = []\n");
    write(root, "docs/crates-io-release.md", "# Releases\n");
    // Copy only roots named by the committed ordinary registry. In particular, omit the
    // synthetic root, including all current/pointer, acquisition and retained depth files.
    // Lifecycle inputs are constructed above; no build-workspace lifecycle state is copied.
    for entry in AgentRegistry::parse(ordinary).unwrap().agents {
        let path = entry.manifest_root;
        declaration::copy(&source.join(&path), &root.join(path));
    }
    declaration::copy(
        &source.join("cli_manifests/support_matrix"),
        &root.join("cli_manifests/support_matrix"),
    );
    directory
}

pub(super) fn install(root: &Path) {
    for path in INPUTS {
        declaration::copy(&workspace().join(path), &root.join(path));
    }
}

/// Merge ordinary manifests without overwriting this test's actual ON1 outputs.
pub(super) fn manifests(root: &Path) {
    declaration::copy(
        &workspace().join("cli_manifests"),
        &root.join("cli_manifests"),
    );
}

#[test]
fn bootstrap_baseline_omits_declared_subject_and_preserves_source() {
    let source = tempfile::tempdir().unwrap();
    declaration::copy(workspace(), source.path());
    write(
        source.path(),
        REGISTRY_RELATIVE_PATH,
        format!("{}\n{DECLARED}", registry_fixture::undeclared_registry()),
    );
    let artifacts = [
        format!("{ROOT}/current.json"),
        format!("{ROOT}/latest_validated.txt"),
        format!("{ROOT}/pointers/latest_supported/x86_64-unknown-linux-gnu.txt"),
        RECORD.into(),
        STATE.into(),
        format!("docs/agents/lifecycle/{PACK}/governance/depth-acquisition/0.2.0.toml"),
        format!("docs/agents/lifecycle/{PACK}/README.md"),
    ];
    for path in &artifacts {
        write(source.path(), path, "preexisting declared subject\n");
    }
    let baseline = construct(source.path());
    assert_eq!(
        fs::read_to_string(baseline.path().join(REGISTRY_RELATIVE_PATH)).unwrap(),
        registry_fixture::undeclared_registry()
    );
    assert!(
        fs::read_to_string(source.path().join(REGISTRY_RELATIVE_PATH))
            .unwrap()
            .contains(DECLARED)
    );
    assert!(!baseline.path().join(ROOT).exists());
    for path in &artifacts {
        assert!(!baseline.path().join(path).exists(), "copied {path}");
        assert_eq!(
            fs::read_to_string(source.path().join(path)).unwrap(),
            "preexisting declared subject\n"
        );
    }
    for entry in AgentRegistry::parse(registry_fixture::undeclared_registry())
        .unwrap()
        .agents
    {
        let path = format!("{}/current.json", entry.manifest_root);
        assert_eq!(
            fs::read(baseline.path().join(&path)).unwrap(),
            fs::read(source.path().join(path)).unwrap()
        );
    }
}

#[test]
fn bootstrap_baseline_merge_preserves_on1_and_preexisting_root_refusal() {
    let f = setup(true);
    let current = format!("{ROOT}/current.json");
    let before = fs::read(f.root.path().join(&current)).unwrap();
    manifests(f.root.path());
    assert_eq!(fs::read(f.root.path().join(current)).unwrap(), before);
    assert_eq!(before, regressions::ordinary_current().as_bytes());

    let f = setup(false);
    manifests(f.root.path());
    assert!(!f.root.path().join(ROOT).exists());
    write(
        f.root.path(),
        &format!("{ROOT}/unexpected"),
        "preexisting root",
    );
    f.commit();
    let error = f.resolve().err().unwrap();
    assert!(
        error.contains("absent bootstrap subject has a preexisting manifest root"),
        "{error}"
    );
}
