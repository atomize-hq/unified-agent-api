use super::*;

pub(super) fn validate_matching_contract(
    context: &Context,
    persisted: &InputContract,
) -> Result<(), Error> {
    if persisted.workflow_version != WORKFLOW_VERSION {
        return Err(Error::Validation(format!(
            "dry-run packet workflow version `{}` does not match `{WORKFLOW_VERSION}`",
            persisted.workflow_version
        )));
    }
    if persisted.run_id != context.run_id {
        return Err(Error::Validation(format!(
            "dry-run packet run_id `{}` does not match write run_id `{}`",
            persisted.run_id, context.run_id
        )));
    }
    if persisted.pass != context.pass.as_str() {
        return Err(Error::Validation(format!(
            "dry-run packet pass `{}` does not match write pass `{}`",
            persisted.pass,
            context.pass.as_str()
        )));
    }
    if persisted.prior_run_dir != context.prior_run_dir {
        return Err(Error::Validation(
            "dry-run packet prior run context does not match the write invocation".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn validate_prior_run_for_pass2(prior_run_path: &Path) -> Result<(), Error> {
    let run_status = load_json::<Value>(&prior_run_path.join("run-status.json"))?;
    let status = run_status
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            Error::Validation(format!(
                "prior run `{}` is missing `status` in run-status.json",
                prior_run_path.display()
            ))
        })?;
    if status != "insufficient_eligible_candidates" {
        return Err(Error::Validation(format!(
            "prior run `{}` must have status `insufficient_eligible_candidates` for pass2",
            prior_run_path.display()
        )));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(super) struct Pass2State {
    pub(super) excluded_candidate_ids: Vec<String>,
    pub(super) top_surviving_candidate: Option<String>,
    pub(super) zero_survivors: bool,
}

pub(super) fn load_pass2_state(
    workspace_root: &Path,
    prior_run_dir: &str,
    requested_run_id: Option<&str>,
) -> Result<Pass2State, Error> {
    let prior_run_path = workspace_root.join(prior_run_dir);
    validate_prior_run_for_pass2(&prior_run_path)?;
    let candidate_pool = load_json::<Value>(&prior_run_path.join("candidate-pool.json"))?;
    let candidates = candidate_pool
        .get("candidates")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            Error::Validation(format!(
                "prior run `{}` is missing candidate-pool.json candidates array",
                prior_run_dir
            ))
        })?;
    let mut excluded = candidates
        .iter()
        .filter_map(|candidate| candidate.get("agent_id").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    excluded.sort();
    excluded.dedup();

    let run_status = load_json::<Value>(&prior_run_path.join("run-status.json"))?;
    let recommended = run_status
        .get("recommended_agent_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let eligible = candidates
        .iter()
        .filter(|candidate| candidate.get("status").and_then(Value::as_str) == Some("eligible"))
        .filter_map(|candidate| candidate.get("agent_id").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let top_surviving_candidate = recommended.or_else(|| eligible.first().cloned());

    if let Some(run_id) = requested_run_id {
        let prior_basename = prior_run_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if prior_basename == run_id {
            return Err(Error::Validation(
                "pass2 must use a fresh run_id instead of reusing the prior insufficiency run id"
                    .to_string(),
            ));
        }
    }

    Ok(Pass2State {
        excluded_candidate_ids: excluded,
        top_surviving_candidate,
        zero_survivors: eligible.is_empty(),
    })
}

pub(super) fn execute_research_contract_validation(
    workspace_root: &Path,
    context: &Context,
) -> Result<(), String> {
    const PYTHON_VALIDATION_SNIPPET: &str = r#"
import importlib.util
import pathlib
import sys

repo = pathlib.Path(sys.argv[1])
research_dir = repo / sys.argv[2]
run_id = sys.argv[3]
run_dir = repo / sys.argv[4]
module_path = repo / 'scripts' / 'recommend_next_agent.py'
spec = importlib.util.spec_from_file_location('recommend_next_agent', module_path)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = module
spec.loader.exec_module(module)

try:
    seed_path = research_dir / 'seed.snapshot.toml'
    seed = module.parse_seed_file(seed_path)
    snapshot_sha = module.sha256_file(seed_path)
    module.validate_research_metadata(
        research_metadata_path=research_dir / 'research-metadata.json',
        expected_run_id=run_id,
        research_dir=research_dir,
        run_dir=run_dir,
    )
    seeded_ids = {candidate.agent_id for candidate in seed.candidates}
    for candidate in seed.candidates:
        dossier_path = research_dir / 'dossiers' / f'{candidate.agent_id}.json'
        dossier = module.load_dossier_payload(dossier_path)
        module.validate_dossier_top_level(
            dossier,
            agent_id=candidate.agent_id,
            snapshot_sha=snapshot_sha,
            seeded_ids=seeded_ids,
        )
        module.validate_claim_evidence_links(dossier, agent_id=candidate.agent_id)
except Exception as exc:
    print(exc)
    raise SystemExit(1)
"#;

    let run_dir_rel = format!("{}/{}", PYTHON_RUNS_ROOT, context.run_id);
    let argv = vec![
        "-c".to_string(),
        PYTHON_VALIDATION_SNIPPET.to_string(),
        workspace_root.display().to_string(),
        context.research_dir_rel.clone(),
        context.run_id.clone(),
        run_dir_rel,
    ];
    let output = Command::new("python3")
        .current_dir(workspace_root)
        .args(&argv)
        .output()
        .map_err(|err| format!("spawn python research validation: {err}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let detail = if !stdout.is_empty() {
        stdout
    } else if !stderr.is_empty() {
        stderr
    } else {
        "unknown python validation failure".to_string()
    };
    Err(format!(
        "research schema validation failed against scripts/recommend_next_agent.py: {detail}"
    ))
}

pub(super) fn validate_discovery_artifacts(discovery_dir: &Path) -> Result<(), String> {
    if !discovery_dir.is_dir() {
        return Err(format!(
            "required discovery artifact directory `{}` is missing",
            discovery_dir.display()
        ));
    }
    let actual = fs::read_dir(discovery_dir)
        .map_err(|err| format!("read discovery dir {}: {err}", discovery_dir.display()))?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<BTreeSet<_>>();
    let expected = DISCOVERY_REQUIRED_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!(
            "discovery artifact set does not match the frozen contract: expected {:?}, got {:?}",
            expected, actual
        ));
    }
    Ok(())
}

pub(super) fn validate_discovery_candidate_count(discovery_dir: &Path) -> Result<(), String> {
    let seed_path = discovery_dir.join("candidate-seed.generated.toml");
    let candidate_ids = extract_candidate_ids_from_seed_file(&seed_path)
        .map_err(|err| format!("validate discovery seed {}: {err}", seed_path.display()))?;
    if candidate_ids.len() < MIN_DISCOVERY_CANDIDATES {
        return Err(format!(
            "discovery seed must define at least {MIN_DISCOVERY_CANDIDATES} candidates before freeze-discovery; got {}",
            candidate_ids.len()
        ));
    }
    Ok(())
}

pub(super) fn validate_discovery_summary_contract(
    discovery_dir: &Path,
    run_id: &str,
) -> Result<(), String> {
    let seed_path = discovery_dir.join("candidate-seed.generated.toml");
    let candidates = extract_seed_candidates_from_seed_file(&seed_path)?;
    let summary_path = discovery_dir.join("discovery-summary.md");
    let summary = fs::read_to_string(&summary_path)
        .map_err(|err| format!("read {}: {err}", summary_path.display()))?;
    if summary.trim().is_empty() {
        return Err("discovery summary must not be empty".to_string());
    }
    if !summary.contains(run_id) {
        return Err("discovery summary must mention the discovery run id".to_string());
    }
    for (agent_id, display_name) in candidates {
        if !summary.contains(&agent_id) {
            return Err(format!(
                "discovery summary must mention candidate id `{agent_id}`"
            ));
        }
        if !summary.contains(&display_name) {
            return Err(format!(
                "discovery summary must mention display name `{display_name}`"
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_discovery_sources_lock_contract(
    discovery_dir: &Path,
    run_id: &str,
) -> Result<(), String> {
    let seeded_ids =
        extract_candidate_ids_from_seed_file(&discovery_dir.join("candidate-seed.generated.toml"))
            .map_err(|err| format!("validate discovery seed {}: {err}", discovery_dir.display()))?
            .into_iter()
            .collect::<BTreeSet<_>>();
    let path = discovery_dir.join("sources.lock.json");
    let bytes = fs::read(&path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let mut data: Value =
        serde_json::from_slice(&bytes).map_err(|err| format!("parse {}: {err}", path.display()))?;
    let object = data
        .as_object_mut()
        .ok_or_else(|| "discovery sources lock must be a JSON object".to_string())?;
    let top_keys = object.keys().cloned().collect::<BTreeSet<_>>();
    let expected_top_keys = ["run_id".to_string(), "sources".to_string()]
        .into_iter()
        .collect::<BTreeSet<_>>();
    if top_keys != expected_top_keys {
        return Err("discovery sources lock keys do not match the frozen contract".to_string());
    }
    let actual_run_id = object
        .get("run_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "discovery sources lock run_id must be a non-empty string".to_string())?;
    if actual_run_id != run_id {
        return Err("discovery sources lock run_id must match the discovery run id".to_string());
    }
    let sources = object
        .get_mut("sources")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "discovery sources lock sources must be an array".to_string())?;
    let mut normalized = false;
    for (index, entry) in sources.iter_mut().enumerate() {
        let object = entry
            .as_object_mut()
            .ok_or_else(|| format!("discovery sources lock sources[{index}] must be an object"))?;
        let source_kind = entry_string(
            object.get("source_kind"),
            &format!("discovery sources lock sources[{index}].source_kind"),
        )?;
        if !DISCOVERY_SOURCE_KINDS.contains(&source_kind.as_str()) {
            return Err(format!(
                "discovery sources lock sources[{index}] has unsupported source_kind `{source_kind}`"
            ));
        }
        let expected_keys = discovery_source_expected_keys(&source_kind);
        let actual_keys = object.keys().cloned().collect::<BTreeSet<_>>();
        if actual_keys != expected_keys {
            return Err(format!(
                "discovery sources lock sources[{index}] keys do not match the frozen contract"
            ));
        }
        let candidate_id = entry_string(
            object.get("candidate_id"),
            &format!("discovery sources lock sources[{index}].candidate_id"),
        )?;
        if !seeded_ids.contains(&candidate_id) {
            return Err(format!(
                "discovery sources lock sources[{index}] references unknown candidate_id `{candidate_id}`"
            ));
        }
        entry_string(
            object.get("url"),
            &format!("discovery sources lock sources[{index}].url"),
        )?;
        entry_string(
            object.get("title"),
            &format!("discovery sources lock sources[{index}].title"),
        )?;
        let captured_at = entry_string(
            object.get("captured_at"),
            &format!("discovery sources lock sources[{index}].captured_at"),
        )?;
        validate_utc_timestamp(
            &captured_at,
            &format!("discovery sources lock sources[{index}].captured_at"),
        )?;
        let role = entry_string(
            object.get("role"),
            &format!("discovery sources lock sources[{index}].role"),
        )?;
        if !DISCOVERY_SOURCE_ROLES.contains(&role.as_str()) {
            return Err(format!(
                "discovery sources lock sources[{index}] has unsupported role `{role}`"
            ));
        }
        if source_kind == "web_search_result" {
            entry_string(
                object.get("query"),
                &format!("discovery sources lock sources[{index}].query"),
            )?;
            let rank = object.get("rank").and_then(Value::as_i64).ok_or_else(|| {
                format!("discovery sources lock sources[{index}].rank must be an integer")
            })?;
            if rank <= 0 {
                return Err(format!(
                    "discovery sources lock sources[{index}].rank must be greater than zero"
                ));
            }
        }
        let actual_sha = entry_hex64(
            object.get("sha256"),
            &format!("discovery sources lock sources[{index}].sha256"),
        )?;
        let expected_sha = canonical_discovery_source_sha(object, &source_kind)?;
        if actual_sha != expected_sha {
            object.insert("sha256".to_string(), Value::String(expected_sha));
            normalized = true;
        }
    }
    if normalized {
        let mut bytes = serde_json::to_vec_pretty(&data)
            .map_err(|err| format!("serialize {}: {err}", path.display()))?;
        bytes.push(b'\n');
        fs::write(&path, bytes).map_err(|err| format!("write {}: {err}", path.display()))?;
    }
    Ok(())
}

pub(super) fn validate_frozen_seed_boundary(research_dir: &Path) -> Result<(), String> {
    let seed_snapshot = research_dir.join("seed.snapshot.toml");
    if !seed_snapshot.is_file() {
        return Err(format!(
            "freeze-discovery did not produce `{}`",
            seed_snapshot.display()
        ));
    }
    let discovery_input = research_dir.join("discovery-input");
    for filename in DISCOVERY_REQUIRED_FILES {
        let path = discovery_input.join(filename);
        if !path.is_file() {
            return Err(format!(
                "freeze-discovery did not copy required discovery input `{}`",
                path.display()
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_research_tree(research_dir: &Path) -> Result<(), String> {
    for filename in RESEARCH_REQUIRED_FILES {
        let path = research_dir.join(filename);
        if !path.is_file() {
            return Err(format!(
                "required research artifact `{}` is missing",
                path.display()
            ));
        }
    }
    validate_frozen_seed_boundary(research_dir)?;
    let seed_snapshot = research_dir.join("seed.snapshot.toml");
    let seeded_ids = extract_candidate_ids_from_seed_file(&seed_snapshot)
        .map_err(|err| format!("parse {}: {err}", seed_snapshot.display()))?;
    if seeded_ids.is_empty() {
        return Err("seed.snapshot.toml does not define any candidate ids".to_string());
    }
    let seed_sha = sha256_hex(&seed_snapshot)
        .map_err(|err| format!("hash {}: {err}", seed_snapshot.display()))?;
    let dossier_dir = research_dir.join("dossiers");
    if !dossier_dir.is_dir() {
        return Err(format!(
            "required dossier directory `{}` is missing",
            dossier_dir.display()
        ));
    }
    let mut dossier_ids = fs::read_dir(&dossier_dir)
        .map_err(|err| format!("read dossier dir {}: {err}", dossier_dir.display()))?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
        .map(|entry| {
            let path = entry.path();
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| format!("invalid dossier filename `{}`", path.display()))?
                .to_string();
            let dossier = load_json::<Value>(&path)
                .map_err(|err| format!("parse dossier {}: {err}", path.display()))?;
            let agent_id = dossier
                .get("agent_id")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!("dossier `{}` is missing string agent_id", path.display())
                })?;
            if agent_id != stem {
                return Err(format!(
                    "dossier `{}` agent_id `{agent_id}` does not match filename stem `{stem}`",
                    path.display()
                ));
            }
            let dossier_seed = dossier
                .get("seed_snapshot_sha256")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!(
                        "dossier `{}` is missing string seed_snapshot_sha256",
                        path.display()
                    )
                })?;
            if dossier_seed != seed_sha {
                return Err(format!(
                    "dossier `{}` seed_snapshot_sha256 does not match the frozen seed",
                    path.display()
                ));
            }
            Ok(stem)
        })
        .collect::<Result<Vec<_>, _>>()?;
    dossier_ids.sort();
    let mut expected = seeded_ids.clone();
    expected.sort();
    if dossier_ids != expected {
        return Err(format!(
            "dossier set does not match frozen seed candidate ids: expected {:?}, got {:?}",
            expected, dossier_ids
        ));
    }
    Ok(())
}

pub(super) fn validate_written_paths(
    written_paths: &[String],
    allowed_root: &str,
    phase: &str,
) -> Result<(), String> {
    let violations = written_paths
        .iter()
        .filter(|path| !path.starts_with(&(allowed_root.to_string() + "/")))
        .cloned()
        .collect::<Vec<_>>();
    if !violations.is_empty() {
        return Err(format!(
            "{phase} write boundary violation: {}",
            violations.join(", ")
        ));
    }
    Ok(())
}

pub(super) fn push_passed_check(report: &mut ValidationReport, name: &str, message: String) {
    report.checks.push(ValidationCheck {
        name: name.to_string(),
        ok: true,
        message,
    });
}

pub(super) fn push_failed_check(report: &mut ValidationReport, name: &str, message: String) {
    report.checks.push(ValidationCheck {
        name: name.to_string(),
        ok: false,
        message,
    });
}

pub(super) fn extract_candidate_ids_from_seed_file(path: &Path) -> Result<Vec<String>, Error> {
    extract_candidate_ids_from_seed_text(&read_string(path)?).map_err(Error::Validation)
}

fn extract_seed_candidates_from_seed_file(path: &Path) -> Result<Vec<(String, String)>, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let regex = Regex::new(
        r#"(?ms)^\[candidate\.([A-Za-z0-9_-]+)\]\s*$.*?^display_name\s*=\s*"([^"\n]+)"\s*$"#,
    )
    .map_err(|err| format!("compile candidate display name regex: {err}"))?;
    let candidates = regex
        .captures_iter(&text)
        .filter_map(|captures| {
            let agent_id = captures.get(1)?.as_str().to_string();
            let display_name = captures.get(2)?.as_str().to_string();
            Some((agent_id, display_name))
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Err(format!(
            "seed file `{}` does not define any candidate display names",
            path.display()
        ));
    }
    Ok(candidates)
}

fn extract_candidate_ids_from_seed_text(text: &str) -> Result<Vec<String>, String> {
    let regex = Regex::new(r"(?m)^\[candidate\.([A-Za-z0-9_-]+)\]\s*$")
        .map_err(|err| format!("compile candidate id regex: {err}"))?;
    let mut ids = regex
        .captures_iter(text)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
        .collect::<Vec<_>>();
    let unique = ids.iter().cloned().collect::<BTreeSet<_>>();
    if unique.len() != ids.len() {
        return Err("seed snapshot contains duplicate candidate ids".to_string());
    }
    ids.sort();
    Ok(ids)
}

fn discovery_source_expected_keys(source_kind: &str) -> BTreeSet<String> {
    let mut keys = [
        "candidate_id".to_string(),
        "source_kind".to_string(),
        "url".to_string(),
        "title".to_string(),
        "captured_at".to_string(),
        "sha256".to_string(),
        "role".to_string(),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    if source_kind == "web_search_result" {
        keys.insert("query".to_string());
        keys.insert("rank".to_string());
    }
    keys
}

fn canonical_discovery_source_sha(
    object: &serde_json::Map<String, Value>,
    source_kind: &str,
) -> Result<String, String> {
    let mut canonical = BTreeMap::<String, Value>::new();
    for key in [
        "candidate_id",
        "captured_at",
        "role",
        "source_kind",
        "title",
        "url",
    ] {
        canonical.insert(
            key.to_string(),
            object
                .get(key)
                .cloned()
                .ok_or_else(|| format!("canonical discovery source is missing `{key}`"))?,
        );
    }
    if source_kind == "web_search_result" {
        for key in ["query", "rank"] {
            canonical.insert(
                key.to_string(),
                object
                    .get(key)
                    .cloned()
                    .ok_or_else(|| format!("canonical discovery source is missing `{key}`"))?,
            );
        }
    }
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| format!("serialize canonical discovery source: {err}"))?;
    Ok(sha256_bytes(&bytes))
}

fn entry_string(value: Option<&Value>, label: &str) -> Result<String, String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| format!("{label} must be a non-empty string"))
}

fn entry_hex64(value: Option<&Value>, label: &str) -> Result<String, String> {
    let value = entry_string(value, label)?;
    let regex =
        Regex::new(r"^[0-9a-f]{64}$").map_err(|err| format!("compile sha256 regex: {err}"))?;
    if !regex.is_match(&value) {
        return Err(format!(
            "{label} must be a lowercase 64-char SHA-256 hex string"
        ));
    }
    Ok(value)
}

fn validate_utc_timestamp(value: &str, label: &str) -> Result<(), String> {
    let regex = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
        .map_err(|err| format!("compile timestamp regex: {err}"))?;
    if !regex.is_match(value) {
        return Err(format!("{label} must be UTC RFC3339 with trailing Z"));
    }
    Ok(())
}
