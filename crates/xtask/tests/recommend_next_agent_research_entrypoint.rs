use std::fs;

use clap::{CommandFactory, Parser};
use serde_json::Value;

#[path = "support/onboard_agent_harness.rs"]
mod harness;
#[path = "support/recommend_next_agent_research_harness.rs"]
mod recommendation_harness;

use recommendation_harness::{
    fake_codex_binary, force_freeze_discovery_failure, packet_dir, pass2_args,
    prepare_recommendation_fixture, read_json, recommend_args, seed_prior_insufficiency_run,
    snapshot_without_packet_runs, write_fake_codex_scenario, Cli, PASS1_RUN_ID, PASS2_RUN_ID,
};

#[test]
fn recommend_next_agent_research_help_text_includes_required_surface() {
    let top_help = Cli::command().render_help().to_string();
    assert!(top_help.contains("recommend-next-agent-research"));

    let err = Cli::try_parse_from(["xtask", "recommend-next-agent-research", "--help"])
        .expect_err("subcommand help should short-circuit parsing");
    assert_eq!(err.exit_code(), 0);
    let help_text = err.to_string();
    assert!(help_text.contains("--dry-run"));
    assert!(help_text.contains("--write"));
    assert!(help_text.contains("--pass"));
    assert!(help_text.contains("--run-id"));
    assert!(help_text.contains("--prior-run-dir"));
    assert!(help_text.contains("--codex-binary"));
}

#[test]
fn recommend_next_agent_research_pass2_requires_prior_run_dir() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-pass2");
    let output = recommendation_harness::run_cli(
        [
            "xtask",
            "recommend-next-agent-research",
            "--dry-run",
            "--pass",
            "pass2",
            "--run-id",
            PASS2_RUN_ID,
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("--prior-run-dir"));
}

#[test]
fn recommend_next_agent_research_write_requires_matching_dry_run_packet() {
    let fixture =
        prepare_recommendation_fixture("recommend-next-agent-research-write-precondition");
    let output = recommendation_harness::run_cli(
        recommend_args(
            "--write",
            "pass1",
            PASS1_RUN_ID,
            &fake_codex_binary(&fixture),
        ),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("matching dry-run packet"));
}

#[test]
fn recommend_next_agent_research_dry_run_writes_complete_packet_for_pass1() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-dry-run-pass1");
    let codex_binary = fake_codex_binary(&fixture);
    let before = snapshot_without_packet_runs(&fixture);
    let output = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    let after = snapshot_without_packet_runs(&fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert_eq!(before, after, "dry-run must only mutate the packet root");

    let run_dir = packet_dir(&fixture, PASS1_RUN_ID);
    for name in [
        "input-contract.json",
        "discovery-prompt.md",
        "research-prompt.md",
        "codex-execution.discovery.json",
        "codex-execution.research.json",
        "codex-stdout.discovery.log",
        "codex-stderr.discovery.log",
        "codex-stdout.research.log",
        "codex-stderr.research.log",
        "written-paths.discovery.json",
        "written-paths.research.json",
        "validation-report.json",
        "run-status.json",
        "run-summary.md",
    ] {
        assert!(run_dir.join(name).is_file(), "missing {name}");
    }

    let prompt = fs::read_to_string(run_dir.join("discovery-prompt.md")).expect("read prompt");
    assert!(prompt.contains("best AI coding CLI"));
    assert!(prompt.contains("AI agent CLI tools"));
    assert!(prompt.contains("developer agent command line"));
    assert!(prompt.contains("Nominate at least 3 candidate ids"));
    assert!(prompt.contains("exact `display_name` string"));
    assert!(prompt.contains("docs/agents/.uaa-temp/recommend-next-agent/discovery/rna-pass1"));
}

#[test]
fn recommend_next_agent_research_dry_run_writes_complete_packet_for_pass2() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-dry-run-pass2");
    let prior_run_dir = seed_prior_insufficiency_run(&fixture, "prior-pass1", false);
    let output = recommendation_harness::run_cli(
        pass2_args("--dry-run", PASS2_RUN_ID, &prior_run_dir, None),
        &fixture,
    );

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let prompt = fs::read_to_string(packet_dir(&fixture, PASS2_RUN_ID).join("discovery-prompt.md"))
        .expect("read pass2 prompt");
    assert!(prompt.contains("alternatives to <top surviving candidate>"));
    assert!(prompt.contains("Excluded candidate ids: `alpha, beta, gamma`"));
    let contract = read_json(&packet_dir(&fixture, PASS2_RUN_ID).join("input-contract.json"));
    assert_eq!(
        contract.get("prior_run_dir").and_then(Value::as_str),
        Some(prior_run_dir.as_str())
    );
}

#[test]
fn recommend_next_agent_research_write_rejects_out_of_bounds_paths() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-boundary");
    let codex_binary = fake_codex_binary(&fixture);
    let dry_run = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "out_of_bounds");

    let output = recommendation_harness::run_cli(
        recommend_args("--write", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("write boundary violation"));
    assert!(output.stderr.contains("docs/unowned.md"));
}

#[test]
fn recommend_next_agent_research_write_invokes_freeze_discovery_with_expected_args() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-freeze");
    let codex_binary = fake_codex_binary(&fixture);
    let dry_run = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "success");

    let output = recommendation_harness::run_cli(
        recommend_args("--write", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let report = read_json(&packet_dir(&fixture, PASS1_RUN_ID).join("validation-report.json"));
    let freeze = report.get("freeze_discovery").expect("freeze evidence");
    let argv = freeze
        .get("argv")
        .and_then(Value::as_array)
        .expect("freeze argv");
    assert_eq!(
        argv.iter().map(Value::as_str).collect::<Option<Vec<_>>>(),
        Some(vec![
            "scripts/recommend_next_agent.py",
            "freeze-discovery",
            "--discovery-dir",
            "docs/agents/.uaa-temp/recommend-next-agent/discovery/rna-pass1",
            "--research-dir",
            "docs/agents/.uaa-temp/recommend-next-agent/research/rna-pass1",
        ])
    );
    let research_prompt =
        fs::read_to_string(packet_dir(&fixture, PASS1_RUN_ID).join("research-prompt.md"))
            .expect("read refreshed research prompt");
    assert!(research_prompt.contains("dossiers/alpha.json"));
    assert!(!research_prompt.contains("Seed snapshot sha256: ``"));
}

#[test]
fn recommend_next_agent_research_write_fails_closed_on_freeze_discovery_error() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-freeze-fail");
    let codex_binary = fake_codex_binary(&fixture);
    let dry_run = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "success");
    force_freeze_discovery_failure(&fixture);

    let output = recommendation_harness::run_cli(
        recommend_args("--write", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("freeze-discovery failed"));
}

#[test]
fn recommend_next_agent_research_write_rejects_discovery_sources_lock_contract() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-sources-lock");
    let codex_binary = fake_codex_binary(&fixture);
    let dry_run = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "invalid_sources_lock_keys");

    let output = recommendation_harness::run_cli(
        recommend_args("--write", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("sources lock"));
    let report = read_json(&packet_dir(&fixture, PASS1_RUN_ID).join("validation-report.json"));
    let checks = report
        .get("checks")
        .and_then(Value::as_array)
        .expect("validation checks");
    assert!(checks.iter().any(|check| {
        check.get("name").and_then(Value::as_str) == Some("discovery_sources_lock_contract")
            && check.get("ok").and_then(Value::as_bool) == Some(false)
    }));
}

#[test]
fn recommend_next_agent_research_write_normalizes_discovery_sources_lock_sha256() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-sources-sha");
    let codex_binary = fake_codex_binary(&fixture);
    let dry_run = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "freeze_fail");

    let output = recommendation_harness::run_cli(
        recommend_args("--write", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let sources = read_json(
        &fixture
            .join("docs/agents/.uaa-temp/recommend-next-agent/discovery")
            .join(PASS1_RUN_ID)
            .join("sources.lock.json"),
    );
    let first_sha = sources
        .get("sources")
        .and_then(Value::as_array)
        .and_then(|sources| sources.first())
        .and_then(|entry| entry.get("sha256"))
        .and_then(Value::as_str)
        .expect("first source sha256");
    assert_ne!(
        first_sha,
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
}

#[test]
fn recommend_next_agent_research_write_rejects_discovery_seed_with_too_few_candidates() {
    let fixture =
        prepare_recommendation_fixture("recommend-next-agent-research-too-few-candidates");
    let codex_binary = fake_codex_binary(&fixture);
    let dry_run = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "too_few_candidates");

    let output = recommendation_harness::run_cli(
        recommend_args("--write", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("at least 3 candidates"));
    let report = read_json(&packet_dir(&fixture, PASS1_RUN_ID).join("validation-report.json"));
    let checks = report
        .get("checks")
        .and_then(Value::as_array)
        .expect("validation checks");
    assert!(checks.iter().any(|check| {
        check.get("name").and_then(Value::as_str) == Some("discovery_candidate_minimum")
            && check.get("ok").and_then(Value::as_bool) == Some(false)
    }));
}

#[test]
fn recommend_next_agent_research_write_rejects_discovery_summary_missing_display_name() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-summary-contract");
    let codex_binary = fake_codex_binary(&fixture);
    let dry_run = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "summary_missing_display_name");

    let output = recommendation_harness::run_cli(
        recommend_args("--write", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("display name"));
    let report = read_json(&packet_dir(&fixture, PASS1_RUN_ID).join("validation-report.json"));
    let checks = report
        .get("checks")
        .and_then(Value::as_array)
        .expect("validation checks");
    assert!(checks.iter().any(|check| {
        check.get("name").and_then(Value::as_str) == Some("discovery_summary_contract")
            && check.get("ok").and_then(Value::as_bool) == Some(false)
    }));
}

#[test]
fn recommend_next_agent_research_write_rejects_invalid_research_schema() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-schema-contract");
    let codex_binary = fake_codex_binary(&fixture);
    let dry_run = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "invalid_research_schema");

    let output = recommendation_harness::run_cli(
        recommend_args("--write", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("research schema validation failed"));
    let report = read_json(&packet_dir(&fixture, PASS1_RUN_ID).join("validation-report.json"));
    let checks = report
        .get("checks")
        .and_then(Value::as_array)
        .expect("validation checks");
    assert!(checks.iter().any(|check| {
        check.get("name").and_then(Value::as_str) == Some("research_schema_contract")
            && check.get("ok").and_then(Value::as_bool) == Some(false)
    }));
}

#[test]
fn recommend_next_agent_research_write_enforces_research_identity() {
    let fixture = prepare_recommendation_fixture("recommend-next-agent-research-identity");
    let codex_binary = fake_codex_binary(&fixture);
    let dry_run = recommendation_harness::run_cli(
        recommend_args("--dry-run", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "identity_mismatch");

    let output = recommendation_harness::run_cli(
        recommend_args("--write", "pass1", PASS1_RUN_ID, &codex_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("seed_snapshot_sha256 does not match the frozen seed"));
}
