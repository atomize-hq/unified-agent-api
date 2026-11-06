use super::*;

#[test]
fn runtime_follow_on_replace_ref_cannot_admit_a_branch_only_statement() {
    let f = setup(true);
    let raw = fs::read(f.root.path().join(STATEMENT)).unwrap();
    fs::remove_file(f.root.path().join(STATEMENT)).unwrap();
    f.commit();
    let protected = git(f.root.path(), &["rev-parse", "HEAD"]);
    write(f.root.path(), STATEMENT, raw);
    branch_commit(&f, &protected);
    let branch = git(f.root.path(), &["rev-parse", "HEAD"]);
    git(f.root.path(), &["replace", &protected, &branch]);
    let _installed = f.install();
    refuses_unchanged(&f, "acquisition statement missing at integration ref");
}

#[test]
fn runtime_follow_on_replaced_statement_blob_cannot_supply_provenance() {
    let f = setup(true);
    let raw = fs::read_to_string(f.root.path().join(STATEMENT)).unwrap();
    write(
        f.root.path(),
        STATEMENT,
        "not a valid acquisition statement",
    );
    f.commit();
    let protected = git(f.root.path(), &["rev-parse", "HEAD"]);
    let genuine = git(f.root.path(), &["rev-parse", &format!("HEAD:{STATEMENT}")]);
    write(f.root.path(), STATEMENT, raw);
    branch_commit(&f, &protected);
    let substitute = git(f.root.path(), &["rev-parse", &format!("HEAD:{STATEMENT}")]);
    git(f.root.path(), &["replace", &genuine, &substitute]);
    let _installed = f.install();
    refuses_unchanged(&f, "acquisition statement");
}

#[test]
fn runtime_follow_on_replaced_bound_blob_cannot_fake_integration_equality() {
    let f = setup(true);
    let path = format!("{ROOT}/wrapper_coverage.json");
    let original = fs::read(f.root.path().join(&path)).unwrap();
    let changed = [original.as_slice(), b"\n"].concat();
    write(f.root.path(), &path, &changed);
    statement(&f);
    write(f.root.path(), &path, &original);
    f.commit(); // The integrated statement describes bytes the protected input does not hold.
    let protected = git(f.root.path(), &["rev-parse", "HEAD"]);
    let genuine = git(f.root.path(), &["rev-parse", &format!("HEAD:{path}")]);
    write(f.root.path(), &path, changed);
    branch_commit(&f, &protected);
    let substitute = git(f.root.path(), &["rev-parse", &format!("HEAD:{path}")]);
    git(f.root.path(), &["replace", &genuine, &substitute]);
    let _installed = f.install();
    refuses_unchanged(&f, "acquired inputs differ from integration ref");
    assert!(freeze(&f).unwrap_err().contains(&path));
}

#[test]
fn runtime_follow_on_grafts_and_alternates_cannot_supply_integration_authority() {
    for graft in [true, false] {
        let f = setup(true);
        let raw = fs::read(f.root.path().join(STATEMENT)).unwrap();
        fs::remove_file(f.root.path().join(STATEMENT)).unwrap();
        f.commit();
        let protected = git(f.root.path(), &["rev-parse", "HEAD"]);
        write(f.root.path(), STATEMENT, raw);
        branch_commit(&f, &protected);
        let branch = git(f.root.path(), &["rev-parse", "HEAD"]);
        if graft {
            // Forge ancestry from the protected tip to the branch carrying the statement.
            write(
                f.root.path(),
                ".git/info/grafts",
                format!("{protected} {branch}\n"),
            );
        } else {
            write(
                f.root.path(),
                ".git/objects/info/alternates",
                "/nonexistent/objects\n",
            );
        }
        let _installed = f.install();
        refuses_unchanged(
            &f,
            "shared, grafted or shallow isolated history is unsupported",
        );
    }
}

#[test]
fn runtime_follow_on_inherited_git_redirects_cannot_change_statement_authority() {
    const CHILD: &str = "XTASK_RUNTIME_FOLLOW_ON_GIT_ENV_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let f = setup(true);
        let _installed = f.install();
        freeze(&f).unwrap();
        fs::remove_file(f.root.path().join(STATEMENT)).unwrap();
        f.commit();
        refuses_unchanged(&f, "acquisition statement missing at integration ref");
        drop(_installed);
        runtime_follow_on_replace_ref_cannot_admit_a_branch_only_statement();
        return;
    }
    // Separate process: hostile Git environment must not race other test threads.
    let foreign = tempfile::tempdir().unwrap();
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "isolated_maintenance::tests::fixture::runtime_follow_on::authority::runtime_follow_on_inherited_git_redirects_cannot_change_statement_authority",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env("GIT_DIR", foreign.path())
        .env("GIT_COMMON_DIR", foreign.path())
        .env("GIT_WORK_TREE", foreign.path())
        .env("GIT_INDEX_FILE", foreign.path().join("forged-index"))
        .env("GIT_OBJECT_DIRECTORY", foreign.path())
        .env("GIT_ALTERNATE_OBJECT_DIRECTORIES", foreign.path())
        .env("GIT_REPLACE_REF_BASE", "refs/forged/")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.bare")
        .env("GIT_CONFIG_VALUE_0", "true")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("1 passed"),
        "child test did not execute: {stdout}"
    );
}

#[test]
fn runtime_follow_on_rule_four_accepts_integrated_retained_target_authority() {
    let f = setup(false);
    let _installed = f.install();
    freeze(&f).unwrap();
    let frozen = json_file(&f, RECORD);
    f.commit(); // Integrate the P-only record and lifecycle object.
    edit_json(&f, &format!("{ROOT}/RULES.json"), |r| {
        r["union"]["expected_targets"] = json!(["other-target"]);
    });
    f.commit();
    freeze(&f).unwrap();
    assert_eq!(json_file(&f, RECORD), frozen);
    assert_eq!(
        json_file(&f, STATE)["support_depth"]["policy_identity"],
        frozen["policy"]["identity"]
    );
    assert!(frozen["obligations"].is_null());
}

#[test]
fn runtime_follow_on_rule_four_branch_local_record_does_not_authorize_targets() {
    let f = setup(false);
    let _installed = f.install();
    freeze(&f).unwrap();
    let protected = git(f.root.path(), &["rev-parse", "HEAD"]);
    edit_json(&f, &format!("{ROOT}/RULES.json"), |r| {
        r["union"]["expected_targets"] = json!(["other-target"]);
    });
    branch_commit(&f, &protected);
    refuses_unchanged(&f, "rule 4");
    let branch = git(f.root.path(), &["rev-parse", "HEAD"]);
    git(f.root.path(), &["replace", &protected, &branch]);
    refuses_unchanged(
        &f,
        "neither RULES.json union.expected_targets nor depth-record coverage",
    );
}
