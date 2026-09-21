use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use super::*;

const SEEDED_REGISTRY: &str = include_str!("../../../data/agent_registry.toml");

const REQUEST: &str = "artifact_version = \"1\"\nrequest_recorded_at = \"2026-09-19T08:03:11Z\"\n";

/// A base-branch checkout: the registry, plus whatever packet files a case needs.
///
/// This models what `agent-maintenance-open-pr.yml` actually has in hand. That workflow checks out
/// `staging` and never the packet branch, which is why markers have to live on base — see the
/// module docs.
fn workspace() -> tempfile::TempDir {
    let root = tempfile::TempDir::new().expect("workspace");
    let registry_path = root
        .path()
        .join(crate::agent_registry::REGISTRY_RELATIVE_PATH);
    fs::create_dir_all(registry_path.parent().expect("registry parent")).expect("registry dir");
    fs::write(&registry_path, SEEDED_REGISTRY).expect("seed registry");
    root
}

fn governance_dir(root: &Path, agent: &str) -> PathBuf {
    let dir = root.join(format!(
        "docs/agents/lifecycle/{agent}-maintenance/governance"
    ));
    fs::create_dir_all(&dir).expect("governance dir");
    dir
}

fn write_marker(root: &Path, agent: &str, file_name: &str, body: &str) {
    let dir = governance_dir(root, agent).join("automation-stand-down");
    fs::create_dir_all(&dir).expect("marker dir");
    fs::write(dir.join(file_name), body).expect("write marker");
}

fn write_request(root: &Path, agent: &str, body: &str) -> Vec<u8> {
    let path = governance_dir(root, agent).join("maintenance-request.toml");
    fs::write(&path, body).expect("write request");
    body.as_bytes().to_vec()
}

/// Built by the renderer the generated `HANDOFF.md` hands to an agent, not written out again here.
///
/// A third hand-written copy of the schema would let the instruction and the parser drift apart
/// while every test stayed green, which is the failure this routine exists to prevent. Going
/// through it means every case below round-trips what an agent is actually told to paste.
fn marker_for(version: &str) -> String {
    render_marker_toml("codex", version, "2026-09-19T08:03:11Z")
}

fn freeze(root: &Path, version: &str) {
    write_marker(
        root,
        "codex",
        &format!("{version}.toml"),
        &marker_for(version),
    );
}

fn check(root: &Path, agent: &str, version: &str) -> Result<StandDownOutcome, Error> {
    run(Args {
        agent: agent.to_string(),
        target_version: version.to_string(),
        from_ref: None,
        workspace_root: Some(root.to_path_buf()),
    })
}

fn expect_validation(result: Result<StandDownOutcome, Error>, case: &str) -> String {
    match result {
        Ok(outcome) => panic!("{case}: expected a validation error, got {outcome:?}"),
        Err(err) => {
            assert_eq!(err.exit_code(), EXIT_VALIDATION, "{case}: exit code");
            err.to_string()
        }
    }
}

#[test]
fn no_marker_leaves_automation_in_authority() {
    let root = workspace();
    governance_dir(root.path(), "codex");

    assert_eq!(
        check(root.path(), "codex", "0.155.0").expect("check"),
        StandDownOutcome::Authorized
    );
}

#[test]
fn a_marker_for_this_generation_stands_automation_down() {
    let root = workspace();
    freeze(root.path(), "0.155.0");

    let outcome = check(root.path(), "codex", "0.155.0").expect("check");
    assert_eq!(outcome, StandDownOutcome::StoodDown);
    assert_eq!(outcome.exit_code(), EXIT_STOOD_DOWN);
}

#[test]
fn a_marker_for_another_generation_does_not_protect_this_one() {
    let root = workspace();
    freeze(root.path(), "0.155.0");

    // Opening 0.156.0 while 0.155.0 is frozen is a legitimate lifecycle step: the new packet gets
    // its own branch and touches neither the frozen branch nor base.
    assert_eq!(
        check(root.path(), "codex", "0.156.0").expect("check"),
        StandDownOutcome::Authorized
    );
}

/// Two same-agent packet PRs can be open at once — supersession exists because of it — so freezing
/// the newer one must not revoke the older one's protection. One file per generation is what makes
/// the second freeze an additive commit instead of an overwrite.
#[test]
fn two_generations_can_be_frozen_at_once() {
    let root = workspace();
    freeze(root.path(), "0.155.0");
    freeze(root.path(), "0.156.0");

    for version in ["0.155.0", "0.156.0"] {
        assert_eq!(
            check(root.path(), "codex", version).expect("check"),
            StandDownOutcome::StoodDown,
            "{version} must stay frozen while the other is"
        );
    }
    assert_eq!(
        check(root.path(), "codex", "0.157.0").expect("check"),
        StandDownOutcome::Authorized
    );
}

/// `uaa-0048` step 4b: a newer run's supersession step must not close a packet a maintainer is
/// mid-closeout on. Supersession asks about the *candidate's* version, so the same command answers
/// it, and the run that legitimately proceeds for 0.156.0 still stands down from closing 0.155.0.
#[test]
fn supersession_asks_about_the_candidate_and_is_refused() {
    let root = workspace();
    freeze(root.path(), "0.155.0");

    assert_eq!(
        check(root.path(), "codex", "0.156.0").expect("run's own version"),
        StandDownOutcome::Authorized
    );
    assert_eq!(
        check(root.path(), "codex", "0.155.0").expect("supersession candidate"),
        StandDownOutcome::StoodDown
    );
}

/// `uaa-0048` step 1c: a merged-but-unpromoted packet has no open PR, no packet branch and no
/// closeout, so every open-PR-shaped signal misses it — while the queue still reads the version as
/// needed, because the validated pointer only advances at promotion. codex 0.155.0 reached exactly
/// that state when #215 merged. Markers are read from base, which is where a merged packet lives,
/// so the case needs no separate mechanism. What makes this more than a duplicate of the plain
/// stand-down test is the request: after a merge the frozen request *is* the one on base, so this
/// is also the only case that exercises the generation binding agreeing.
#[test]
fn a_merged_but_unpromoted_packet_can_still_be_protected() {
    let root = workspace();
    let bytes = write_request(root.path(), "codex", REQUEST);
    write_marker(
        root.path(),
        "codex",
        "0.155.0.toml",
        &format!(
            "schema_version = 1\n\
             agent_id = \"codex\"\n\
             target_version = \"0.155.0\"\n\
             reason = \"closeout in progress\"\n\
             request_recorded_at = \"2026-09-19T08:03:11Z\"\n\
             request_sha256 = \"{}\"\n",
            hex::encode(Sha256::digest(&bytes))
        ),
    );

    assert_eq!(
        check(root.path(), "codex", "0.155.0").expect("check"),
        StandDownOutcome::StoodDown
    );
}

/// The marker records which request generation was frozen. A generation that has since moved is
/// the case where a maintainer most needs automation to keep away, so it must not read as
/// permission to proceed — for either binding.
#[test]
fn a_stale_generation_binding_still_stands_automation_down() {
    for stale in [
        "request_recorded_at = \"1999-01-01T00:00:00Z\"",
        "request_sha256 = \"0000000000000000000000000000000000000000000000000000000000000000\"",
    ] {
        let root = workspace();
        write_request(root.path(), "codex", REQUEST);
        write_marker(
            root.path(),
            "codex",
            "0.155.0.toml",
            &format!(
                "schema_version = 1\n\
                 agent_id = \"codex\"\n\
                 target_version = \"0.155.0\"\n\
                 reason = \"closeout in progress\"\n\
                 {stale}\n"
            ),
        );

        assert_eq!(
            check(root.path(), "codex", "0.155.0").expect("check"),
            StandDownOutcome::StoodDown,
            "stale binding {stale} must not authorize"
        );
    }
}

#[test]
fn an_unusable_marker_is_never_read_as_permission() {
    let cases: [(&str, &str, &str, &str); 8] = [
        (
            "misnamed file",
            "v0.155.0.toml",
            &marker_for("v0.155.0"),
            "not named for a packet generation",
        ),
        (
            "two-part version",
            "0.155.toml",
            &marker_for("0.155"),
            "not named for a packet generation",
        ),
        (
            "prerelease version",
            "0.155.0-rc.1.toml",
            &marker_for("0.155.0-rc.1"),
            "not named for a packet generation",
        ),
        (
            "malformed toml",
            "0.155.0.toml",
            "schema_version = \n",
            "not a usable",
        ),
        (
            "unknown field",
            "0.155.0.toml",
            "schema_version = 1\n\
             agent_id = \"codex\"\n\
             target_version = \"0.155.0\"\n\
             reason = \"r\"\n\
             request_sha256 = \"abc\"\n\
             until = \"forever\"\n",
            "not a usable",
        ),
        (
            "no generation binding",
            "0.155.0.toml",
            "schema_version = 1\n\
             agent_id = \"codex\"\n\
             target_version = \"0.155.0\"\n\
             reason = \"r\"\n",
            "request_recorded_at",
        ),
        (
            "wrong agent",
            "0.155.0.toml",
            "schema_version = 1\n\
             agent_id = \"opencode\"\n\
             target_version = \"0.155.0\"\n\
             reason = \"r\"\n\
             request_sha256 = \"abc\"\n",
            "sits in codex's maintenance root",
        ),
        (
            "name and field disagree",
            "0.155.0.toml",
            &marker_for("0.156.0"),
            "not the generation its file name claims",
        ),
    ];

    for (case, file_name, body, expected_fragment) in cases {
        let root = workspace();
        write_marker(root.path(), "codex", file_name, body);
        let message = expect_validation(check(root.path(), "codex", "0.155.0"), case);
        assert!(
            message.contains(expected_fragment),
            "{case}: expected {expected_fragment:?} in {message:?}"
        );
    }
}

/// An entry this command cannot read means the freeze set is unknown, and an unknown freeze set is
/// not an authorization — even when the unreadable entry names some other generation.
#[test]
fn one_unreadable_entry_stands_down_every_generation() {
    let root = workspace();
    freeze(root.path(), "0.155.0");
    write_marker(
        root.path(),
        "codex",
        "not-a-version.toml",
        "schema_version = 1\n",
    );

    expect_validation(
        check(root.path(), "codex", "0.156.0"),
        "unrelated bad entry",
    );
}

/// Resolving the marker path by string formatting alone would turn a typo into "no marker,
/// proceed" — the one direction this command may never fail in.
#[test]
fn an_unknown_agent_is_a_validation_failure_not_an_authorization() {
    let root = workspace();
    let message = expect_validation(check(root.path(), "codexx", "0.155.0"), "typo'd agent");
    assert!(message.contains("unknown agent"), "{message:?}");
}

#[test]
fn a_missing_registry_is_a_validation_failure_not_an_authorization() {
    let root = tempfile::TempDir::new().expect("workspace");
    let message = expect_validation(check(root.path(), "codex", "0.155.0"), "no registry");
    assert!(message.contains("agent registry"), "{message:?}");
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("git {args:?}: {err}"));
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `--from-ref` is what lets a caller re-read base immediately before mutating, instead of trusting
/// a checkout taken minutes earlier. It must see a marker the working tree does not have.
#[test]
fn from_ref_reads_the_committed_marker_not_the_working_tree() {
    let root = workspace();
    git(root.path(), &["init", "-q", "-b", "base"]);
    git(root.path(), &["config", "user.email", "t@example.com"]);
    git(root.path(), &["config", "user.name", "t"]);
    freeze(root.path(), "0.155.0");
    git(root.path(), &["add", "-A"]);
    git(root.path(), &["commit", "-qm", "freeze"]);

    // The working tree loses the marker; the committed ref still carries it.
    fs::remove_dir_all(
        root.path()
            .join("docs/agents/lifecycle/codex-maintenance/governance/automation-stand-down"),
    )
    .expect("remove marker dir");

    assert_eq!(
        check(root.path(), "codex", "0.155.0").expect("working tree"),
        StandDownOutcome::Authorized
    );

    let from_ref = run(Args {
        agent: "codex".to_string(),
        target_version: "0.155.0".to_string(),
        from_ref: Some("base".to_string()),
        workspace_root: Some(root.path().to_path_buf()),
    })
    .expect("from ref");
    assert_eq!(from_ref, StandDownOutcome::StoodDown);
}

/// A ref that cannot be read is not an empty one. Git failing here must stand automation down, not
/// authorize it.
#[test]
fn an_unreadable_ref_is_an_internal_error_not_an_authorization() {
    let root = workspace();
    git(root.path(), &["init", "-q", "-b", "base"]);

    match run(Args {
        agent: "codex".to_string(),
        target_version: "0.155.0".to_string(),
        from_ref: Some("refs/heads/does-not-exist".to_string()),
        workspace_root: Some(root.path().to_path_buf()),
    }) {
        Ok(outcome) => panic!("expected an internal error, got {outcome:?}"),
        Err(err) => assert_eq!(err.exit_code(), EXIT_INTERNAL, "{err}"),
    }
}

/// `EXPECTED_SHAPE` is the one remaining place the schema is written out by hand: the error text a
/// maintainer reads when their marker was rejected. An example that no longer parses is worse than
/// no example, because it is read at the moment someone is already confused about this file.
#[test]
fn the_shape_the_error_message_teaches_is_one_the_parser_accepts() {
    let example: String = EXPECTED_SHAPE
        .lines()
        .filter_map(|line| line.strip_prefix("    "))
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(
        example.contains("schema_version"),
        "the indented example block moved, so this test is asserting on nothing"
    );

    validate_entry(
        "codex",
        "docs/agents/lifecycle/codex-maintenance/governance/automation-stand-down",
        "0.155.0.toml",
        &example,
    )
    .expect("the error message's own example must be a usable marker");
}
