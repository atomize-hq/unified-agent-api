use super::*;

pub(super) fn packet_file_names() -> Vec<String> {
    [
        INPUT_CONTRACT_FILE_NAME,
        DISCOVERY_PROMPT_FILE_NAME,
        RESEARCH_PROMPT_FILE_NAME,
        DISCOVERY_EXECUTION_FILE_NAME,
        RESEARCH_EXECUTION_FILE_NAME,
        DISCOVERY_STDOUT_FILE_NAME,
        DISCOVERY_STDERR_FILE_NAME,
        RESEARCH_STDOUT_FILE_NAME,
        RESEARCH_STDERR_FILE_NAME,
        DISCOVERY_WRITTEN_PATHS_FILE_NAME,
        RESEARCH_WRITTEN_PATHS_FILE_NAME,
        VALIDATION_REPORT_FILE_NAME,
        RUN_STATUS_FILE_NAME,
        RUN_SUMMARY_FILE_NAME,
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

pub(super) fn build_context(workspace_root: &Path, args: &Args) -> Result<Context, Error> {
    validate_args(workspace_root, args)?;

    let run_id = args.run_id.clone().unwrap_or_else(generate_run_id);
    let packet_dir = packet_root(workspace_root, &run_id);
    let discovery_dir = discovery_root(workspace_root, &run_id);
    let research_dir = research_root(workspace_root, &run_id);
    let packet_dir_rel = packet_root_rel(&run_id);
    let discovery_dir_rel = discovery_root_rel(&run_id);
    let research_dir_rel = research_root_rel(&run_id);
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Validation(format!("load agent registry: {err}")))?;
    let mut onboarded_agent_ids = registry
        .agents
        .iter()
        .map(|entry| entry.agent_id.clone())
        .collect::<Vec<_>>();
    onboarded_agent_ids.sort();

    let pass2_state = if matches!(args.pass, Pass::Pass2) {
        Some(load_pass2_state(
            workspace_root,
            args.prior_run_dir
                .as_deref()
                .expect("pass2 prior_run_dir validated"),
            args.run_id.as_deref(),
        )?)
    } else {
        None
    };

    let discovery_hints_path = workspace_root
        .join(DISCOVERY_HINTS_PATH)
        .is_file()
        .then(|| DISCOVERY_HINTS_PATH.to_string());
    let query_family = match args.pass {
        Pass::Pass1 => PASS1_QUERY_FAMILY
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        Pass::Pass2
            if pass2_state
                .as_ref()
                .is_some_and(|state| state.zero_survivors) =>
        {
            PASS2_QUERY_FAMILY_ZERO_SURVIVOR
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        }
        Pass::Pass2 => PASS2_QUERY_FAMILY_WITH_SURVIVOR
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    };
    let input_contract = InputContract {
        workflow_version: WORKFLOW_VERSION.to_string(),
        run_id: run_id.clone(),
        pass: args.pass.as_str().to_string(),
        prior_run_dir: args.prior_run_dir.clone(),
        packet_root: packet_dir_rel.clone(),
        discovery_root: discovery_dir_rel.clone(),
        research_root: research_dir_rel.clone(),
        python_runs_root: PYTHON_RUNS_ROOT.to_string(),
        discovery_hints_path,
        live_seed_path: LIVE_SEED_PATH.to_string(),
        dossier_contract_path: DOSSIER_CONTRACT_PATH.to_string(),
        required_packet_files: packet_file_names(),
        discovery_required_files: DISCOVERY_REQUIRED_FILES
            .iter()
            .map(ToString::to_string)
            .collect(),
        research_required_files: RESEARCH_REQUIRED_FILES
            .iter()
            .map(ToString::to_string)
            .collect(),
        query_family,
        excluded_candidate_ids: pass2_state
            .as_ref()
            .map(|state| state.excluded_candidate_ids.clone())
            .unwrap_or_default(),
        top_surviving_candidate: pass2_state
            .as_ref()
            .and_then(|state| state.top_surviving_candidate.clone()),
        onboarded_agent_ids,
        proving_flow_order: PROVING_FLOW_ORDER.iter().map(ToString::to_string).collect(),
    };

    Ok(Context {
        run_id,
        pass: args.pass,
        prior_run_dir: args.prior_run_dir.clone(),
        codex_binary: resolve_codex_binary(args),
        packet_dir,
        packet_dir_rel,
        discovery_dir,
        discovery_dir_rel,
        research_dir,
        research_dir_rel,
        input_contract,
    })
}

fn validate_args(workspace_root: &Path, args: &Args) -> Result<(), Error> {
    if args.write && args.run_id.is_none() {
        return Err(Error::Validation(
            "--run-id is required with --write so the command can validate against a prepared dry-run packet"
                .to_string(),
        ));
    }

    match args.pass {
        Pass::Pass1 => {
            if args.prior_run_dir.is_some() {
                return Err(Error::Validation(
                    "--prior-run-dir is only valid with --pass pass2".to_string(),
                ));
            }
        }
        Pass::Pass2 => {
            let prior_run_dir = args.prior_run_dir.as_deref().ok_or_else(|| {
                Error::Validation(
                    "--prior-run-dir is required with --pass pass2 because pass2 must consume prior insufficiency output"
                        .to_string(),
                )
            })?;
            let prior_run_path = workspace_root.join(prior_run_dir);
            if !prior_run_path.is_dir() {
                return Err(Error::Validation(format!(
                    "prior run directory `{prior_run_dir}` does not exist"
                )));
            }
            let prior_basename = prior_run_path
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| {
                    Error::Validation(format!(
                        "prior run directory `{prior_run_dir}` must end with a run id basename"
                    ))
                })?;
            if args.run_id.as_deref() == Some(prior_basename) {
                return Err(Error::Validation(
                    "pass2 must use a fresh run_id instead of reusing the prior insufficiency run id"
                        .to_string(),
                ));
            }
            validate_prior_run_for_pass2(&prior_run_path)?;
        }
    }

    if args.write {
        let run_id = args.run_id.as_deref().expect("write run_id checked above");
        let input_contract = packet_root(workspace_root, run_id).join(INPUT_CONTRACT_FILE_NAME);
        if !input_contract.is_file() {
            return Err(Error::Validation(format!(
                "--write requires a matching dry-run packet for run_id `{run_id}`; missing `{}`",
                input_contract.display()
            )));
        }
    }

    Ok(())
}

pub(super) fn resolve_codex_binary(args: &Args) -> String {
    args.codex_binary
        .clone()
        .or_else(|| std::env::var(CODEX_BINARY_ENV).ok())
        .unwrap_or_else(|| "codex".to_string())
}

fn packet_root(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(RESEARCH_PACKET_ROOT).join(run_id)
}

fn discovery_root(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(DISCOVERY_ROOT).join(run_id)
}

fn research_root(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(RESEARCH_ROOT).join(run_id)
}

fn packet_root_rel(run_id: &str) -> String {
    format!("{RESEARCH_PACKET_ROOT}/{run_id}")
}

fn discovery_root_rel(run_id: &str) -> String {
    format!("{DISCOVERY_ROOT}/{run_id}")
}

fn research_root_rel(run_id: &str) -> String {
    format!("{RESEARCH_ROOT}/{run_id}")
}

fn generate_run_id() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
        .replace(['-', ':'], "")
        .replace("+00", "Z")
}
