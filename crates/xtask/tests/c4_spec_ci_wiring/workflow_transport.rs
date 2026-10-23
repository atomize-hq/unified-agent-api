use super::*;
use std::path::Path;
use std::process::Command;

use super::acquisition_destination::{git, job, repository, run, shell, step};

const ACQUIRE: &str = ".github/workflows/parity-acquire.yml";
const OPEN: &str = ".github/workflows/agent-maintenance-open-pr.yml";

#[test]
fn exact_transport_runs_real_git_regressions() {
    let output = Command::new("python3")
        .arg("scripts/test_maintenance_transport.py")
        .current_dir(repo_root())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn initial_packet_token_is_required_before_mutation_and_computed_paths_replace_caller_paths() {
    let yml = read_repo_file(OPEN);
    let guard = step(&yml, "Require an automation token for packet delivery");
    assert!(guard.contains("HAS_AUTOMATION_TOKEN: ${{ secrets.AUTOMATION_TOKEN != '' }}"));
    assert_text_order(
        &yml,
        "Require an automation token",
        "uses: actions/checkout@",
        OPEN,
    );
    assert_text_order(
        &yml,
        "Require an automation token",
        "name: Prepare maintenance packet",
        OPEN,
    );
    assert!(!guard.lines().any(|l| l.trim_start().starts_with("if:")));
    for has_token in ["", "false", "true"] {
        let dir = tempfile::tempdir().unwrap();
        let result = run(
            &shell(guard),
            dir.path(),
            &[("HAS_AUTOMATION_TOKEN", has_token)],
        );
        assert_eq!(result.status.success(), has_token == "true", "{result:?}");
    }
    assert!(yml.contains("secrets:\n      AUTOMATION_TOKEN:"));
    assert!(yml.contains("token: ${{ secrets.AUTOMATION_TOKEN }}"));
    assert!(!yml.contains("AUTOMATION_TOKEN || github.token"));
    assert!(!yml.contains("add_paths"));
    assert!(yml.contains("add-paths: ${{ steps.delivery.outputs.paths }}"));
    assert!(step(&yml, "Create PR").contains("steps.delivery.outcome == 'success'"));
    assert_text_order(
        &yml,
        "--emit-json",
        "Stage the computed packet delivery",
        OPEN,
    );
    assert_text_order(&yml, "Verify checked packet bytes", "name: Create PR", OPEN);
}

#[test]
fn acquisition_orders_staging_freeze_publication_and_checked_delivery_before_audit() {
    let yml = read_repo_file(ACQUIRE);
    for (first, second) in [
        (
            "Union → wrapper coverage",
            "Stage exact acquisition inputs before freeze",
        ),
        (
            "Stage exact acquisition inputs before freeze",
            "Re-freeze the maintenance request",
        ),
        (
            "Re-freeze the maintenance request",
            "Publish acquisition support outputs after freeze",
        ),
        (
            "Publish acquisition support outputs after freeze",
            "Stage the complete checked delivery",
        ),
        (
            "Stage the complete checked delivery",
            "name: Maintenance audit gate",
        ),
        (
            "name: Maintenance audit gate",
            "name: Commit the acquired artifacts",
        ),
    ] {
        assert_text_order(&yml, first, second, ACQUIRE);
    }
    assert!(step(
        job(&yml, "union"),
        "Stage exact acquisition inputs before freeze"
    )
    .contains("if: ${{ inputs.commit }}"));
    assert!(
        step(job(&yml, "union"), "Stage the complete checked delivery")
            .contains("ARGS+=(--read-only)")
    );
}

fn write_executable(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
    assert!(Command::new("chmod")
        .arg("+x")
        .arg(path)
        .status()
        .unwrap()
        .success());
}

#[test]
fn workflow_shell_stages_before_freeze_and_delivers_exact_bytes_without_push_retry() {
    for (stand_down, prepare_exit, rejected) in [
        (0, 0, false),
        (3, 0, false),
        (1, 0, false),
        (0, 1, false),
        (0, 0, true),
    ] {
        let yml = read_repo_file(ACQUIRE);
        let union = job(&yml, "union");
        let dir = repository();
        let scratch = tempfile::tempdir().unwrap();
        let remote = tempfile::tempdir().unwrap();
        let branch = "automation/codex-maintenance-1.2.3";
        git(dir.path(), &["branch", "-m", branch]);
        let request = "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml";
        let request_path = dir.path().join(request);
        fs::create_dir_all(request_path.parent().unwrap()).unwrap();
        fs::write(&request_path, "same Event before freeze").unwrap();
        git(dir.path(), &["add", request]);
        git(dir.path(), &["commit", "-m", "packet"]);
        let before = git(dir.path(), &["rev-parse", "HEAD"]);
        git(remote.path(), &["init", "--bare"]);
        git(
            dir.path(),
            &["remote", "add", "origin", remote.path().to_str().unwrap()],
        );
        git(dir.path(), &["push", "origin", "HEAD:refs/heads/staging"]);
        if rejected {
            write_executable(
                &remote.path().join("hooks/pre-receive"),
                "#!/bin/sh\nprintf x >> receiver-calls\nexit 1\n",
            );
        }
        let input = "cli_manifests/codex/snapshots/1.2.3/union.json";
        let input_path = dir.path().join(input);
        fs::create_dir_all(input_path.parent().unwrap()).unwrap();
        fs::write(input_path, "acquired").unwrap();
        let extra = dir
            .path()
            .join("cli_manifests/codex/reports/1.2.3/extra-evidence.json");
        fs::create_dir_all(extra.parent().unwrap()).unwrap();
        fs::write(&extra, "unowned").unwrap();
        let helper = dir.path().join("scripts/maintenance_transport.py");
        fs::create_dir_all(helper.parent().unwrap()).unwrap();
        fs::copy(repo_root().join("scripts/maintenance_transport.py"), helper).unwrap();
        fs::create_dir_all(dir.path().join("_ci_tmp")).unwrap();
        fs::write(
            dir.path().join("_ci_tmp/acquisition-plan.json"),
            serde_json::to_vec(&serde_json::json!({
                "input_paths": [input], "output_paths": [input]
            }))
            .unwrap(),
        )
        .unwrap();
        write_executable(&dir.path().join("bin/cargo"), "#!/bin/sh\nexit 0\n");
        // The external command is stubbed; staging, fetch, commit and push are real Git. The
        // existing Rust suites separately validate freeze semantics and guarded production refusal.
        write_executable(
            &dir.path().join("target/debug/xtask"),
            r#"#!/usr/bin/env python3
import hashlib,json,os,subprocess,sys
from pathlib import Path
if sys.argv[1] == 'maintenance-stand-down-check':
    sys.exit(int(os.environ['STAND_DOWN_EXIT']))
assert sys.argv[1] == 'prepare-agent-maintenance'
if int(os.environ['PREPARE_EXIT']):
    sys.exit(1)
p='cli_manifests/codex/snapshots/1.2.3/union.json'
assert subprocess.check_output(['git','show',':'+p]) == Path(p).read_bytes()
p=sys.argv[sys.argv.index('--from-request')+1]
Path(p).write_text('same Event after freeze')
files=[p,'cli_manifests/support_matrix/current.json','docs/specs/unified-agent-api/support-matrix.md','crates/agent_api/src/runtime_support_data.rs']
for f in files[1:]:
    Path(f).parent.mkdir(parents=True,exist_ok=True)
    Path(f).write_text('prepared publication')
report={'applied':True,'files':[{'path':f,'sha256':hashlib.sha256(Path(f).read_bytes()).hexdigest()} for f in files]}
Path(sys.argv[sys.argv.index('--emit-json')+1]).write_text(json.dumps(report))
"#,
        );
        let path = format!(
            "{}:{}",
            dir.path().join("bin").display(),
            std::env::var("PATH").unwrap()
        );
        let output = scratch.path().join("outputs");
        let marker = stand_down.to_string();
        let failure = prepare_exit.to_string();
        let report = scratch.path().join("prepared.json");
        let env = [
            ("PATH", path.as_str()),
            ("AGENT_ID", "codex"),
            ("VERSION", "1.2.3"),
            ("REF", branch),
            ("TRANSPORT_DIR", scratch.path().to_str().unwrap()),
            ("GITHUB_OUTPUT", output.to_str().unwrap()),
            ("STAND_DOWN_EXIT", marker.as_str()),
            ("PREPARE_EXIT", failure.as_str()),
            ("PREPARATION_REPORT", report.to_str().unwrap()),
            ("COMMIT", "true"),
            ("BRANCH", branch),
            (
                "DELIVERY_REF",
                "refs/heads/automation/codex-maintenance-1.2.3",
            ),
        ];
        let staged = run(
            &shell(step(union, "Stage exact acquisition inputs before freeze")),
            dir.path(),
            &env,
        );
        assert!(staged.status.success(), "{staged:?}");
        let frozen = run(
            &shell(step(
                union,
                "Re-freeze the maintenance request against the acquired artifacts",
            )),
            dir.path(),
            &env,
        );
        if stand_down != 0 || prepare_exit != 0 {
            assert_eq!(frozen.status.success(), stand_down == 3, "{frozen:?}");
            assert_eq!(
                fs::read_to_string(&request_path).unwrap(),
                "same Event before freeze"
            );
            assert!(!report.exists());
            assert_eq!(git(dir.path(), &["rev-parse", "HEAD"]), before);
            continue;
        }
        assert!(frozen.status.success(), "{frozen:?}");
        let publication = run(
            &shell(step(
                union,
                "Publish acquisition support outputs after freeze",
            )),
            dir.path(),
            &env,
        );
        assert!(publication.status.success(), "{publication:?}");
        let delivery = run(
            &shell(step(union, "Stage the complete checked delivery")),
            dir.path(),
            &env,
        );
        assert!(delivery.status.success(), "{delivery:?}");
        let committed = run(
            &shell(step(
                union,
                "Commit the acquired artifacts onto the packet branch",
            )),
            dir.path(),
            &env,
        );
        assert_eq!(committed.status.success(), !rejected, "{committed:?}");
        assert_eq!(
            git(dir.path(), &["rev-parse", "HEAD^"]),
            before,
            "no rebase/retry"
        );
        assert_eq!(fs::read_to_string(&extra).unwrap(), "unowned");
        assert!(!git(dir.path(), &["ls-files"]).contains("extra-evidence"));
        if rejected {
            assert_eq!(
                fs::read_to_string(remote.path().join("receiver-calls")).unwrap(),
                "x"
            );
            assert!(fs::read_to_string(&output)
                .unwrap()
                .contains("committed=false"));
        } else {
            assert_eq!(
                git(remote.path(), &["rev-parse", branch]),
                git(dir.path(), &["rev-parse", "HEAD"])
            );
            assert_eq!(
                git(
                    remote.path(),
                    &[
                        "show",
                        &format!("{branch}:crates/agent_api/src/runtime_support_data.rs")
                    ]
                ),
                "prepared publication"
            );
            assert!(!scratch
                .path()
                .join("bundle/cli_manifests/codex/reports/1.2.3/extra-evidence.json")
                .exists());
        }
    }
}
