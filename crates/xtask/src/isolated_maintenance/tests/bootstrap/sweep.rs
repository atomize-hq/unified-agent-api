//! Checkpoint admission must preserve the ordinary writers' preconditions and byte contracts.
use super::*;

#[test]
fn bootstrap_creation_replays_all_parent_filesystem_conflicts() {
    for path in [
        "crates/depth_fixture",
        "crates/agent_api/src/backends/depth_fixture",
    ] {
        let f = setup(false);
        write(f.root.path(), path, "protected regular file\n");
        f.commit();
        let parent = head(&f);
        local(&f, &parent);
        let ordinary = crate::onboard_agent::run_in_workspace(
            f.root.path(),
            renderer::args(),
            &mut Vec::new(),
        )
        .unwrap_err()
        .to_string();
        assert!(ordinary.contains("not a directory"), "{ordinary}");
        // Deliberately render without the retained conflict, then restore it. The private
        // replay must detect the wrapper conflict using the whole pinned parent.
        fs::remove_file(f.root.path().join(path)).unwrap();
        create(&f);
        write(f.root.path(), path, "protected regular file\n");
        candidate(&f, &parent);
        let _installed = f.install();
        if path == "crates/depth_fixture" {
            let error = check_integration(f.root.path(), &parent).unwrap_err();
            assert!(error.contains("not a directory"), "{error}");
        } else {
            // This subject has no backend. Ordinary raw ON1 above still checks its
            // declared backend; finite replay must never invent one to impose a conflict.
            check_integration(f.root.path(), &parent).unwrap();
        }
        assert_eq!(
            fs::read_to_string(f.root.path().join(path)).unwrap(),
            "protected regular file\n"
        );
    }
}

#[test]
fn bootstrap_preparation_requires_exact_pointer_bytes() {
    for pointer in [
        "latest_validated.txt",
        "min_supported.txt",
        "pointers/latest_validated/x86_64-unknown-linux-gnu.txt",
        "pointers/latest_supported/x86_64-unknown-linux-gnu.txt",
    ] {
        let f = setup(true);
        adopt(&f);
        f.commit();
        let parent = head(&f);
        local(&f, &parent);
        let _installed = f.install();
        for bytes in ["none", " none\n", "none \n", "none\n\n", " none\n\n"] {
            git(f.root.path(), &["reset", "--hard", &parent]);
            write(f.root.path(), &format!("{ROOT}/{pointer}"), bytes);
            candidate(&f, &parent);
            assert!(
                check_integration(f.root.path(), &parent).is_err(),
                "{pointer}: {bytes:?}"
            );
        }
    }
}

#[test]
fn bootstrap_preparation_requires_exact_reviewed_governing_bytes() {
    for name in ["RULES.json", "SCHEMA.json", "VERSION_METADATA_SCHEMA.json"] {
        let f = setup(true);
        let parent = head(&f);
        adopt(&f);
        let path = format!("{ROOT}/{name}");
        let raw = fs::read_to_string(f.root.path().join(&path)).unwrap();
        write(f.root.path(), &path, format!("{raw}\n"));
        candidate(&f, &parent);
        let _installed = f.install();
        assert!(check_integration(f.root.path(), &parent).is_err(), "{name}");
    }
}

#[test]
fn bootstrap_preparation_rechecks_inherited_bootstrap_current_bytes() {
    let f = setup(true);
    write(
        f.root.path(),
        &format!("{ROOT}/current.json"),
        format!("{}\n", regressions::ordinary_current()),
    );
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    adopt(&f);
    candidate(&f, &parent);
    let _installed = f.install();
    assert!(check_integration(f.root.path(), &parent).is_err());
}

#[test]
fn bootstrap_metadata_cannot_introduce_notes_or_hide_duplicate_members() {
    let f = setup(true);
    adopt(&f);
    producer::acquire(f.root.path());
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    metadata(&f);
    let path = format!("{ROOT}/versions/0.2.0.json");
    let valid = fs::read_to_string(f.root.path().join(&path)).unwrap();
    let _installed = f.install();
    candidate(&f, &parent);
    check_integration(f.root.path(), &parent).unwrap();
    for edit in [
        valid.replacen('{', "{\n  \"notes\": \"unrelated normative claim\",", 1),
        valid.replacen('{', "{\n  \"status\": \"supported\",", 1),
        format!("{valid}\n"),
    ] {
        write(f.root.path(), &path, edit);
        candidate(&f, &parent);
        let error = check_integration(f.root.path(), &parent).unwrap_err();
        assert!(error.contains("metadata"), "{error}");
    }
}

#[test]
fn bootstrap_acquisition_requires_complete_writer_encoding() {
    let f = setup(true);
    let _installed = f.install();
    let parent = head(&f);
    adopt(&f);
    integrate(&f, &parent);
    let parent = head(&f);
    producer::acquire(f.root.path());
    let valid = candidate(&f, &parent);
    check_integration(f.root.path(), &parent).unwrap();
    for suffix in [
        "snapshots/0.2.0/x86_64-unknown-linux-gnu.json",
        "snapshots/0.2.0/union.json",
        "wrapper_coverage.json",
        "reports/0.2.0/coverage.any.json",
        "reports/0.2.0/coverage.all.json",
        "reports/0.2.0/coverage.x86_64-unknown-linux-gnu.json",
    ] {
        git(f.root.path(), &["reset", "--hard", &valid]);
        let path = format!("{ROOT}/{suffix}");
        let raw = fs::read_to_string(f.root.path().join(&path)).unwrap();
        write(f.root.path(), &path, format!("{raw}\n"));
        candidate(&f, &parent);
        assert!(check_integration(f.root.path(), &parent).is_err(), "{path}");
    }
}
