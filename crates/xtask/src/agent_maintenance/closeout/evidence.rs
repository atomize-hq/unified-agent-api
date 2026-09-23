//! T4 — closeout evidence resolution.
//!
//! A closeout records `preflight_passed` as a fact about a specific commit. Nothing downstream
//! re-derives it, so whatever this module concludes is what the governance artifact asserts. That
//! makes the *selection* rule the load-bearing part, not the lookup: a resolver that picks among
//! candidate runs can be made to pick a favourable one, and a resolver that guesses when evidence
//! is absent produces an artifact that reads as verified.
//!
//! The policy is therefore declared here rather than left to whatever the API happened to return.
//!
//! - **Canonical repository, not the local remote.** Governance evidence comes from
//!   [`EVIDENCE_REPO`], never from `git remote get-url origin`. A checkout pointed at a fork would
//!   otherwise resolve a fork's CI into this repository's governance record.
//! - **Selection.** Workflow runs whose `head_sha` equals the requested commit exactly, narrowed to
//!   [`PREFLIGHT_WORKFLOW_PATH`]. The query filters server-side; both fields are re-checked here,
//!   because a filter that silently stops filtering is indistinguishable from one that matched.
//!   Every page is read.
//! - **Terminality.** A matching run that is not `completed` refuses. Evidence that is still moving
//!   is not evidence, and waiting for it is the caller's decision, not this module's.
//! - **Agreement, not ranking.** All matching runs must agree. They are never sorted by recency and
//!   never reduced to a "latest" — a rule that picks among disagreeing runs is a rule that can be
//!   steered by re-running until the preferred answer exists. Disagreement refuses and names every
//!   candidate. Re-run attempts of *one* run id collapse to that run's highest attempt, which is
//!   that run's current conclusion; distinct run ids stay distinct candidates.
//! - **Known failure is not failure to resolve.** Unanimous `failure` resolves to
//!   `preflight_passed = false` and is recorded. No runs, a non-terminal run, disagreement, or a
//!   terminal conclusion that establishes neither outcome (`cancelled`, `skipped`, `neutral`,
//!   `stale`, `action_required`) all refuse. T4 reports a failure; it does not gate on one.
//! - **Merge commits refuse before any network call.** `ci.yml` triggers on `pull_request`, so
//!   check runs attach to the packet-branch head and the merge commit carries none (measured on
//!   #215: head `f51e0b62` 17 runs, merge `404ff19d` 0). Detected by parent count so the refusal
//!   names the real cause instead of presenting as "no evidence found".
//!
//! **Association is not provenance.** A resolved conclusion says CI *associated* that result with
//! that commit. It does not say the commit is the tree that was built: `pull_request` workflows
//! check out the merge ref by default. Callers must not upgrade this into a claim about what was
//! compiled.

use std::{collections::BTreeMap, path::Path, process::Command};

use serde::Deserialize;

use super::types::MaintenanceCloseoutError;

/// Governance evidence is read from this repository regardless of the checkout's remotes.
const EVIDENCE_REPO: &str = "atomize-hq/unified-agent-api";

/// The workflow whose conclusion `preflight_passed` records.
const PREFLIGHT_WORKFLOW_PATH: &str = ".github/workflows/ci.yml";

/// GitHub's maximum; a short page is the last page.
const PAGE_SIZE: usize = 100;

/// One candidate run, at its highest observed attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRun {
    pub run_id: u64,
    pub run_attempt: u64,
    pub conclusion: String,
}

/// A resolved conclusion and the runs it was resolved from.
///
/// `runs` is kept so a caller can render what was consulted. It is evidence of the derivation, not
/// a set to re-adjudicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPreflight {
    pub commit: String,
    pub passed: bool,
    pub runs: Vec<EvidenceRun>,
}

#[derive(Deserialize)]
struct RunsPage {
    #[serde(default)]
    workflow_runs: Vec<RawRun>,
}

#[derive(Deserialize)]
struct RawRun {
    id: u64,
    path: Option<String>,
    head_sha: Option<String>,
    status: Option<String>,
    conclusion: Option<String>,
    /// Absent on older runs, which predate re-run attempts and therefore are attempt 1.
    #[serde(default = "first_attempt")]
    run_attempt: u64,
}

fn first_attempt() -> u64 {
    1
}

/// Resolve the preflight conclusion for `commit`, or refuse.
///
/// `workspace_root` is used only for the merge-commit check, which is local.
pub fn resolve_preflight(
    workspace_root: &Path,
    commit: &str,
) -> Result<ResolvedPreflight, MaintenanceCloseoutError> {
    reject_merge_commit(workspace_root, commit)?;
    resolve_preflight_with_fetcher(commit, |url| {
        super::super::watch::fetch_text(url).map_err(map_fetch_error)
    })
}

/// The network-free half, so the policy above is testable against frozen API responses.
pub fn resolve_preflight_with_fetcher<F>(
    commit: &str,
    mut fetch: F,
) -> Result<ResolvedPreflight, MaintenanceCloseoutError>
where
    F: FnMut(&str) -> Result<String, MaintenanceCloseoutError>,
{
    let commit = normalized_commit(commit)?;

    // Keyed by run id so re-run attempts of one run collapse, while distinct runs stay distinct.
    let mut candidates: BTreeMap<u64, EvidenceRun> = BTreeMap::new();
    let mut page = 1usize;
    loop {
        let url = format!(
            "https://api.github.com/repos/{EVIDENCE_REPO}/actions/runs\
             ?head_sha={commit}&per_page={PAGE_SIZE}&page={page}"
        );
        let body = fetch(&url)?;
        let parsed: RunsPage = serde_json::from_str(&body).map_err(|err| {
            MaintenanceCloseoutError::Validation(format!(
                "parse workflow runs for `{commit}` from {url}: {err}"
            ))
        })?;
        let returned = parsed.workflow_runs.len();

        for run in parsed.workflow_runs {
            if run.head_sha.as_deref() != Some(commit.as_str()) {
                continue;
            }
            if run.path.as_deref() != Some(PREFLIGHT_WORKFLOW_PATH) {
                continue;
            }
            let status = run.status.as_deref().unwrap_or("");
            if status != "completed" {
                return Err(MaintenanceCloseoutError::Validation(format!(
                    "closeout evidence for `{commit}` is not final: run {} is `{status}`, not \
                     `completed`. Evidence that is still moving is not evidence — re-run this \
                     once {PREFLIGHT_WORKFLOW_PATH} has finished for that commit.",
                    run.id
                )));
            }
            let Some(conclusion) = run.conclusion else {
                return Err(MaintenanceCloseoutError::Validation(format!(
                    "closeout evidence for `{commit}` is unusable: run {} reports `completed` \
                     with no conclusion.",
                    run.id
                )));
            };
            let entry = candidates.entry(run.id).or_insert_with(|| EvidenceRun {
                run_id: run.id,
                run_attempt: 0,
                conclusion: String::new(),
            });
            if run.run_attempt >= entry.run_attempt {
                entry.run_attempt = run.run_attempt;
                entry.conclusion = conclusion;
            }
        }

        if returned < PAGE_SIZE {
            break;
        }
        page += 1;
    }

    if candidates.is_empty() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "no {PREFLIGHT_WORKFLOW_PATH} run is associated with `{commit}` in {EVIDENCE_REPO}. \
             Check runs attach to the packet-branch head, so this is what a merge commit, a \
             force-pushed-away head, or a commit CI never observed all look like."
        )));
    }

    let runs: Vec<EvidenceRun> = candidates.into_values().collect();
    let first = runs[0].conclusion.as_str();
    if runs.iter().any(|run| run.conclusion != first) {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "closeout evidence for `{commit}` disagrees: {}. This is not resolved by preferring \
             one of them — establish which run describes the packet and remove the other.",
            render_candidates(&runs)
        )));
    }

    let passed = match first {
        "success" => true,
        "failure" | "timed_out" | "startup_failure" => false,
        other => {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "closeout evidence for `{commit}` is inconclusive: `{other}` establishes neither a \
                 pass nor a failure ({}).",
                render_candidates(&runs)
            )));
        }
    };

    Ok(ResolvedPreflight {
        commit,
        passed,
        runs,
    })
}

fn render_candidates(runs: &[EvidenceRun]) -> String {
    runs.iter()
        .map(|run| {
            format!(
                "run {} attempt {} concluded `{}`",
                run.run_id, run.run_attempt, run.conclusion
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn normalized_commit(commit: &str) -> Result<String, MaintenanceCloseoutError> {
    let trimmed = commit.trim();
    let valid = (7..=40).contains(&trimmed.len())
        && trimmed
            .chars()
            .all(|ch| matches!(ch, '0'..='9' | 'a'..='f'));
    if valid {
        Ok(trimmed.to_string())
    } else {
        Err(MaintenanceCloseoutError::Validation(format!(
            "closeout evidence commit `{commit}` must be 7-40 lowercase hex characters"
        )))
    }
}

/// Refuse a merge commit by parent count, before any network call.
///
/// Without this the refusal would be "no run is associated with this commit", which is true and
/// useless: it describes a commit CI never saw and a commit CI cannot see identically.
pub fn reject_merge_commit(
    workspace_root: &Path,
    commit: &str,
) -> Result<(), MaintenanceCloseoutError> {
    let line = git(
        workspace_root,
        &["rev-list", "--parents", "-n", "1", commit],
    )?;
    let parents = line.split_whitespace().count().saturating_sub(1);
    if parents > 1 {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "`{commit}` is a merge commit ({parents} parents), so no {PREFLIGHT_WORKFLOW_PATH} \
             run is associated with it: the workflow triggers on `pull_request`, and its check \
             runs attach to the packet-branch head. Record the packet-branch head instead."
        )));
    }
    Ok(())
}

fn map_fetch_error(err: super::super::watch::Error) -> MaintenanceCloseoutError {
    match err {
        super::super::watch::Error::Validation(message) => {
            MaintenanceCloseoutError::Validation(message)
        }
        super::super::watch::Error::Internal(message) => {
            MaintenanceCloseoutError::Internal(message)
        }
    }
}

fn git(workspace_root: &Path, args: &[&str]) -> Result<String, MaintenanceCloseoutError> {
    let output = Command::new("git")
        .current_dir(workspace_root)
        .args(args)
        .output()
        .map_err(|err| {
            MaintenanceCloseoutError::Internal(format!(
                "could not run `git {}`: {err}",
                args.join(" ")
            ))
        })?;
    if !output.status.success() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "`git {}` failed ({}): {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout).map_err(|err| {
        MaintenanceCloseoutError::Internal(format!(
            "`git {}` emitted non-UTF-8: {err}",
            args.join(" ")
        ))
    })
}
