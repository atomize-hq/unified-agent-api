use super::{transport::*, *};
use crate::depth_ci::{self, Witness};
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub(super) fn witness(
    api: &impl Api,
    approved_helper: &str,
    run: &Value,
    attempt: u64,
) -> Result<Witness, String> {
    let id = run["id"].as_u64().ok_or("missing run id")?;
    let name = format!("depth-channel-v1-{id}-{attempt}");
    let artifacts = list(
        api,
        &format!("actions/runs/{id}/artifacts"),
        Some("artifacts"),
    )?;
    let matches = artifacts
        .iter()
        .filter(|a| a["name"] == name)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err("missing/ambiguous attempt witness artifact".into());
    }
    let artifact = matches[0];
    let artifact_id = artifact["id"].as_u64().ok_or("missing artifact id")?;
    if artifact["expired"] != false
        || artifact["size_in_bytes"]
            .as_u64()
            .filter(|n| *n > 0 && *n <= 2_097_152)
            .is_none()
        || artifact["workflow_run"]["id"] != id
        || artifact["workflow_run"]["head_sha"] != run["head_sha"]
    {
        return Err("artifact expired or lacks exact host run binding".into());
    }
    let digest = required_string(artifact, "digest")?
        .strip_prefix("sha256:")
        .ok_or("missing host artifact digest")?;
    let jobs = list(
        api,
        &format!("actions/runs/{id}/attempts/{attempt}/jobs"),
        Some("jobs"),
    )?;
    let jobs = jobs
        .iter()
        .filter(|j| j["name"] == "Rust tests (workspace)")
        .collect::<Vec<_>>();
    if jobs.len() != 1 {
        return Err("missing/ambiguous producer job".into());
    }
    let job = jobs[0];
    if job["run_id"] != id || job["run_attempt"] != attempt || job["status"] != "completed" {
        return Err("nonterminal/wrong-attempt producer job".into());
    }
    let steps = job["steps"].as_array().ok_or("missing job steps")?;
    for name in [
        "Collect isolated depth witness",
        "Upload isolated depth witness",
        "Bind isolated depth artifact",
    ] {
        if steps
            .iter()
            .filter(|s| {
                s["name"] == name && s["conclusion"] == "success" && s["status"] == "completed"
            })
            .count()
            != 1
        {
            return Err("producer collection/upload/binding step did not complete".into());
        }
    }
    let binding_step = steps
        .iter()
        .find(|s| s["name"] == "Bind isolated depth artifact")
        .unwrap();
    let started = required_string(binding_step, "started_at")?;
    let completed = required_string(binding_step, "completed_at")?;
    let job_id = job["id"].as_u64().ok_or("missing producer job id")?;
    let logs = api.bytes(
        &endpoint(&format!("actions/jobs/{job_id}/logs")),
        16_777_216,
    )?;
    let logs = std::str::from_utf8(&logs).map_err(|_| "non-UTF8 job logs")?;
    let mut bindings = vec![];
    for line in logs.lines() {
        // Do not accept command echoes, another job's logs, or an un-timestamped receipt claim.
        let Some((timestamp, body)) = line.split_once(' ') else {
            continue;
        };
        let Some(body) = body.strip_prefix("DEPTH_ARTIFACT_BINDING=") else {
            continue;
        };
        let second = timestamp.get(..19).ok_or("invalid binding log time")?;
        if second < started.get(..19).ok_or("invalid binding step start")?
            || second > completed.get(..19).ok_or("invalid binding step end")?
        {
            return Err("artifact binding was not emitted by binding step".into());
        }
        bindings.push(serde_json::from_str::<Value>(body).map_err(|_| "invalid binding log")?);
    }
    if bindings.len() != 1
        || bindings[0]["id"] != artifact_id
        || bindings[0]["name"] != name
        || bindings[0]["digest"] != digest
    {
        return Err("artifact is not bound to the approved job upload".into());
    }
    let zip = api.bytes(
        &endpoint(&format!("actions/artifacts/{artifact_id}/zip")),
        2_097_152,
    )?;
    if zip.len() as u64 != artifact["size_in_bytes"].as_u64().unwrap()
        || depth_ci::reference("", &zip).sha256 != digest
    {
        return Err("downloaded artifact digest/size mismatch".into());
    }
    let mut input = tempfile::tempfile().map_err(|e| e.to_string())?;
    input.write_all(&zip).map_err(|e| e.to_string())?;
    std::io::Seek::rewind(&mut input).map_err(|e| e.to_string())?;
    let result = Command::new("/usr/bin/timeout")
        .args(["30", "python3", "-c", approved_helper, "witness"])
        .stdin(Stdio::from(input))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if !result.status.success() || result.stdout.len() > 1_048_576 {
        return Err("unsupported artifact ZIP/payload form".into());
    }
    crate::depth_evidence::payload::parse(
        std::str::from_utf8(&result.stdout).map_err(|_| "non-UTF8 witness")?,
    )
}
