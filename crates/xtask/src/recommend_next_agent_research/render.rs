use super::*;

pub(super) fn render_discovery_prompt(context: &Context) -> String {
    let hints = context
        .input_contract
        .discovery_hints_path
        .as_deref()
        .unwrap_or("none");
    let excluded = if context.input_contract.excluded_candidate_ids.is_empty() {
        "none".to_string()
    } else {
        context.input_contract.excluded_candidate_ids.join(", ")
    };
    let onboarded = if context.input_contract.onboarded_agent_ids.is_empty() {
        "none".to_string()
    } else {
        context.input_contract.onboarded_agent_ids.join(", ")
    };
    let top_survivor = context
        .input_contract
        .top_surviving_candidate
        .as_deref()
        .unwrap_or("none");
    format!(
        concat!(
            "# Recommendation Research Discovery Prompt\n\n",
            "Run id: `{run_id}`\n",
            "Pass: `{pass}`\n",
            "Live seed path: `{live_seed}`\n",
            "Discovery hints path: `{hints}`\n",
            "Allowed output root: `{discovery_root}`\n",
            "Execution packet root: `{packet_root}`\n",
            "Currently onboarded agent ids: `{onboarded}`\n",
            "Excluded candidate ids: `{excluded}`\n",
            "Top surviving candidate: `{top_survivor}`\n\n",
            "Read only these repo files before researching:\n",
            "- `{live_seed}`\n",
            "- `{hints}`\n",
            "- `crates/xtask/data/agent_registry.toml`\n\n",
            "Required discovery files:\n",
            "- `candidate-seed.generated.toml`\n",
            "- `discovery-summary.md`\n",
            "- `sources.lock.json`\n\n",
            "`discovery-summary.md` must mention the run id and, for each nominated candidate, both the candidate id and the exact `display_name` string from the generated seed.\n\n",
            "`sources.lock.json` must be a JSON object with only `run_id` and `sources`. Each `sources[]` entry must use `candidate_id`, `source_kind`, `url`, `title`, `captured_at`, `sha256`, and `role`; `web_search_result` entries must also include `query` and `rank`. Allowed `source_kind` values: `web_search_result`, `official_doc`, `github`, `package_registry`. Allowed `role` values: `frontier_signal`, `discovery_seed`, `install_surface`, `docs_surface`. `captured_at` must be UTC RFC3339 with a trailing `Z`, and `sha256` must hash the canonical per-entry object.\n\n",
            "Fixed query family:\n{queries}\n\n",
            "Requirements:\n",
            "- Use only public discovery evidence relevant to the fixed query family.\n",
            "- Respect discovery hints when present.\n",
            "- Exclude already onboarded agents and every pass1 candidate listed in `Excluded candidate ids`.\n",
            "- Nominate at least 3 candidate ids in `candidate-seed.generated.toml` before stopping.\n",
            "- Write exactly the three required files and nothing else.\n",
            "- Do not write `seed.snapshot.toml`; the repo owns `freeze-discovery`.\n",
            "- Do not inspect unrelated repo files or run repo-wide searches.\n",
            "- Do not write outside `{discovery_root}`.\n",
            "- `sources.lock.json` must use the frozen contract fields and stable sha256 entries.\n"
        ),
        run_id = context.run_id,
        pass = context.pass.as_str(),
        live_seed = context.input_contract.live_seed_path,
        hints = hints,
        discovery_root = context.discovery_dir_rel,
        packet_root = context.packet_dir_rel,
        onboarded = onboarded,
        excluded = excluded,
        top_survivor = top_survivor,
        queries = render_bullets(&context.input_contract.query_family),
    )
}

pub(super) fn render_research_prompt(context: &Context) -> String {
    let seed_ids =
        extract_candidate_ids_from_seed_file(&context.research_dir.join("seed.snapshot.toml"))
            .unwrap_or_default();
    let required_dossiers = if seed_ids.is_empty() {
        "none".to_string()
    } else {
        seed_ids
            .iter()
            .map(|agent_id| format!("- `dossiers/{agent_id}.json`"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        concat!(
            "# Recommendation Research Dossier Prompt\n\n",
            "Run id: `{run_id}`\n",
            "Pass: `{pass}`\n",
            "Frozen seed snapshot: `{seed_snapshot}`\n",
            "Seed snapshot sha256: `{seed_sha}`\n",
            "Dossier contract path: `{contract_path}`\n",
            "Allowed output root: `{research_root}`\n",
            "Execution packet root: `{packet_root}`\n\n",
            "Read only these repo files before researching:\n",
            "- `{seed_snapshot}`\n",
            "- `{research_root}/discovery-input/candidate-seed.generated.toml`\n",
            "- `{research_root}/discovery-input/discovery-summary.md`\n",
            "- `{research_root}/discovery-input/sources.lock.json`\n",
            "- `{contract_path}`\n\n",
            "Required research files:\n",
            "- `research-summary.md`\n",
            "- `research-metadata.json`\n",
            "{required_dossiers}\n\n",
            "Each dossier MUST be one JSON object with exactly these top-level fields: `schema_version`, `agent_id`, `display_name`, `generated_at`, `seed_snapshot_sha256`, `official_links`, `install_channels`, `auth_prerequisites`, `claims`, `probe_requests`, `blocked_steps`, `normalized_caveats`, and `evidence`.\n",
            "`official_links`, `install_channels`, `auth_prerequisites`, `blocked_steps`, and `normalized_caveats` must each be arrays of strings.\n",
            "`claims` must contain exactly `non_interactive_execution`, `offline_strategy`, `observable_cli_surface`, `redaction_fit`, `crate_first_fit`, `reproducibility`, and `future_leverage`. Each claim must include `state`, `summary`, and `evidence_ids`, with optional `blocked_by` and `notes`.\n",
            "`probe_requests` must be an array. Each probe request object must contain exactly `probe_kind`, `binary`, and `required_for_gate`, and `probe_kind` is limited to `help` or `version`.\n",
            "`evidence` must be an array of objects containing `evidence_id`, `kind`, `url`, `title`, `captured_at`, `sha256`, and `excerpt`. `kind` is limited to `official_doc`, `github`, `package_registry`, `ancillary`, or `probe_output`.\n\n",
            "Requirements:\n",
            "- Read only the frozen seed snapshot under `{research_root}`.\n",
            "- Produce exactly one dossier file per seeded candidate id.\n",
            "- Each dossier `agent_id` must match its filename stem.\n",
            "- Each dossier `seed_snapshot_sha256` must equal `{seed_sha}`.\n",
            "- `probe_requests` are structured metadata, not shell instructions.\n",
            "- Prefer the official/install/documentation URLs already present in `discovery-input/sources.lock.json` before widening to other sources.\n",
            "- Do not inspect unrelated repo files or run repo-wide searches.\n",
            "- Do not modify discovery artifacts.\n",
            "- Do not write outside `{research_root}`.\n"
        ),
        run_id = context.run_id,
        pass = context.pass.as_str(),
        seed_snapshot = format!("{}/seed.snapshot.toml", context.research_dir_rel),
        seed_sha = sha256_hex(&context.research_dir.join("seed.snapshot.toml")).unwrap_or_default(),
        contract_path = context.input_contract.dossier_contract_path,
        research_root = context.research_dir_rel,
        packet_root = context.packet_dir_rel,
        required_dossiers = required_dossiers,
    )
}

pub(super) fn render_dry_run_summary(context: &Context) -> String {
    format!(
        concat!(
            "# Recommendation Research Dry Run\n\n",
            "- run_id: `{}`\n",
            "- pass: `{}`\n",
            "- packet root: `{}`\n",
            "- discovery root: `{}`\n",
            "- research root: `{}`\n",
            "- discovery prompt: `{}`\n",
            "- research prompt: `{}`\n",
            "- query family size: `{}`\n"
        ),
        context.run_id,
        context.pass.as_str(),
        context.packet_dir_rel,
        context.discovery_dir_rel,
        context.research_dir_rel,
        DISCOVERY_PROMPT_FILE_NAME,
        RESEARCH_PROMPT_FILE_NAME,
        context.input_contract.query_family.len(),
    )
}

pub(super) fn render_write_summary(
    context: &Context,
    report: &ValidationReport,
    discovery_written_paths: &[String],
    research_written_paths: &[String],
    discovery_execution: Option<&CodexExecutionEvidence>,
    research_execution: Option<&CodexExecutionEvidence>,
) -> String {
    let mut text = format!(
        concat!(
            "# Recommendation Research Validation\n\n",
            "- run_id: `{}`\n",
            "- pass: `{}`\n",
            "- status: `{}`\n",
            "- packet root: `{}`\n",
            "- discovery root: `{}`\n",
            "- research root: `{}`\n"
        ),
        context.run_id,
        context.pass.as_str(),
        report.status,
        context.packet_dir_rel,
        context.discovery_dir_rel,
        context.research_dir_rel,
    );
    if let Some(execution) = discovery_execution {
        text.push_str("\n## Discovery Execution\n");
        text.push_str(&format!("- binary: `{}`\n", execution.binary));
        text.push_str(&format!("- exit_code: `{}`\n", execution.exit_code));
        text.push_str(&format!("- prompt: `{}`\n", execution.prompt_path));
    }
    if let Some(freeze) = &report.freeze_discovery {
        text.push_str("\n## Freeze Discovery\n");
        text.push_str(&format!("- binary: `{}`\n", freeze.binary));
        text.push_str(&format!("- exit_code: `{}`\n", freeze.exit_code));
        text.push_str("- argv:\n");
        for arg in &freeze.argv {
            text.push_str(&format!("  - `{arg}`\n"));
        }
    }
    if let Some(execution) = research_execution {
        text.push_str("\n## Research Execution\n");
        text.push_str(&format!("- binary: `{}`\n", execution.binary));
        text.push_str(&format!("- exit_code: `{}`\n", execution.exit_code));
        text.push_str(&format!("- prompt: `{}`\n", execution.prompt_path));
    }
    text.push_str("\n## Checks\n");
    for check in &report.checks {
        text.push_str(&format!(
            "- {}: {} ({})\n",
            check.name,
            if check.ok { "pass" } else { "fail" },
            check.message
        ));
    }
    text.push_str("\n## Discovery Written Paths\n");
    if discovery_written_paths.is_empty() {
        text.push_str("- none detected\n");
    } else {
        for path in discovery_written_paths {
            text.push_str(&format!("- `{path}`\n"));
        }
    }
    text.push_str("\n## Research Written Paths\n");
    if research_written_paths.is_empty() {
        text.push_str("- none detected\n");
    } else {
        for path in research_written_paths {
            text.push_str(&format!("- `{path}`\n"));
        }
    }
    if !report.errors.is_empty() {
        text.push_str("\n## Errors\n");
        for error in &report.errors {
            text.push_str(&format!("- {error}\n"));
        }
    }
    text
}

fn render_bullets(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("- `{value}`"))
        .collect::<Vec<_>>()
        .join("\n")
}
