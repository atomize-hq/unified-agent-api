//! Exercise the production binding and discovery paths over immutable Git heads.
use super::*;
use crate::{depth_ci::profile::GenerationResolver, isolated_maintenance::EvidenceHost};

const P: Profile = Profile::Synthetic;
const REPORT: &str = depth_ci::onboarding::RECORD;

pub(super) fn binding_matrix(root: &Path, base: &str, api: &Fake) {
    let target = P
        .resolve_generation(&source(root, ISOLATED, base).unwrap())
        .unwrap()
        .unwrap()
        .record;
    let request_path = crate::agent_lifecycle::maintenance_request_path(P.agent_id());
    let request = fs::read_to_string(root.join(&request_path)).unwrap();
    let report = fs::read_to_string(root.join(REPORT)).unwrap();
    let mut other = target.clone();
    if let Event::Maintenance { request_commit, .. } = &mut other.event {
        *request_commit = "a".repeat(40);
    }
    let mut other_request: toml_edit::DocumentMut = request.parse().unwrap();
    other_request["request_commit"] = toml_edit::value("a".repeat(40));
    let other_report = serde_json::to_vec_pretty(&other).unwrap();
    let host = EvidenceHost::for_test(ISOLATED, base);
    let query = api
        .0
        .keys()
        .find(|p| p.contains("/runs?created="))
        .unwrap()
        .clone();
    let rows: Value = serde_json::from_slice(&api.0[&query]).unwrap();
    for mutation in 0..13 {
        git(root, &["reset", "--hard", base]);
        let expected = match mutation {
            0 => {
                fs::remove_file(root.join(REPORT)).unwrap();
                "maintenance generation report missing"
            }
            1 => {
                write(root, REPORT, b"{");
                "invalid maintenance generation report"
            }
            2 => {
                write(root, REPORT, &other_report);
                "maintenance generation request/report mismatch"
            }
            3 => {
                fs::remove_file(root.join(&request_path)).unwrap();
                "maintenance generation request missing"
            }
            4 => {
                write(root, &request_path, b"[");
                "invalid maintenance generation request"
            }
            5 => {
                write(root, &request_path, other_request.to_string());
                "maintenance generation request/report mismatch"
            }
            6 => {
                fs::remove_file(root.join(&request_path)).unwrap();
                fs::remove_file(root.join(REPORT)).unwrap();
                "maintenance generation request missing"
            }
            7 => {
                write(root, &request_path, other_request.to_string());
                write(root, REPORT, &other_report);
                "associated head replaced maintenance generation"
            }
            8 => {
                write(
                    root,
                    REPORT,
                    report.replace("\"schema_version\": 1", "\"schema_version\": 99"),
                );
                "invalid maintenance generation report"
            }
            9 => {
                write(
                    root,
                    &request_path,
                    request.replace("artifact_version = \"2\"", "artifact_version = \"99\""),
                );
                "maintenance generation version, agent or enrollment mismatch"
            }
            10 => {
                write(
                    root,
                    &request_path,
                    request.replace("depth_fixture", "other_agent"),
                );
                "maintenance generation version, agent or enrollment mismatch"
            }
            11 => {
                // Two entries claim the same Event. Both records have valid part identities.
                let mut duplicate = target.clone();
                duplicate.policy.selection.enrollment.version = "0.3.0".into();
                duplicate.policy.identity =
                    crate::depth_record::part_identity(&duplicate.policy).unwrap();
                let o = duplicate.obligations.as_mut().unwrap();
                o.policy_identity = duplicate.policy.identity.clone();
                o.identity = crate::depth_record::part_identity(o).unwrap();
                write(
                    root,
                    "cli_manifests/depth_fixture/reports/0.3.0/depth-record.json",
                    serde_json::to_vec_pretty(&duplicate).unwrap(),
                );
                "ambiguous maintenance generation reports"
            }
            _ => {
                write(
                    root,
                    &target.obligations.as_ref().unwrap().acquired_inputs[0].path,
                    b"stale immutable input",
                );
                "maintenance generation has stale acquired input"
            }
        };
        let candidate = commit(root);
        git(root, &["reset", "--hard", base]);
        let mut next = api.clone();
        let mut rows = rows.clone();
        let mut row = rows["workflow_runs"][0].clone();
        row["id"] = json!(44);
        row["head_sha"] = json!(candidate);
        row["pull_requests"] = json!([{"number":9}]);
        rows["workflow_runs"].as_array_mut().unwrap().push(row);
        rows["total_count"] = json!(2);
        next.0
            .insert(query.clone(), serde_json::to_vec(&rows).unwrap());
        let error = resolve_profile_in(&next, root, P, Some(&host)).unwrap_err();
        assert!(error.contains(expected), "mutation {mutation}: {error}");
        next.change(query.strip_prefix(&next.endpoint("")).unwrap(), |value| {
            value["workflow_runs"][1]["pull_requests"] = json!([]);
        });
        if matches!(mutation, 6 | 7) {
            // Neither record associates this head with the target. The auxiliary still matches.
            assert_eq!(
                resolve_profile_in(&next, root, P, Some(&host))
                    .unwrap()
                    .runs
                    .len(),
                11
            );
        } else {
            let error = resolve_profile_in(&next, root, P, Some(&host)).unwrap_err();
            assert!(
                error.contains(expected),
                "unassociated mutation {mutation}: {error}"
            );
        }
    }
    // Fixture bytes never enter the generation projection, including malformed fixture bytes.
    for fixture in [
        None,
        Some(b"{}".as_slice()),
        Some(b"mutated auxiliary".as_slice()),
    ] {
        git(root, &["reset", "--hard", base]);
        match fixture {
            Some(bytes) => {
                write(root, P.fixture(), bytes);
            }
            None => {
                if root.join(P.fixture()).exists() {
                    fs::remove_file(root.join(P.fixture())).unwrap();
                }
            }
        }
        let head = commit(root);
        let producer = P
            .current_generation(&mut crate::git_inputs::GitInputs::new(root).unwrap())
            .unwrap()
            .record;
        let discovery = P
            .resolve_inputs(
                &mut source(root, ISOLATED, &head).unwrap(),
                Some(&target),
                false,
            )
            .unwrap()
            .unwrap()
            .record;
        assert_eq!(producer, target);
        assert_eq!(discovery, producer);
        git(root, &["reset", "--hard", base]);
        let mut next = api.clone();
        let mut population = rows.clone();
        let mut row = population["workflow_runs"][0].clone();
        row["id"] = json!(46);
        row["head_sha"] = json!(head);
        population["workflow_runs"]
            .as_array_mut()
            .unwrap()
            .push(row);
        population["total_count"] = json!(2);
        next.0
            .insert(query.clone(), serde_json::to_vec(&population).unwrap());
        // Changed fixture bytes still belong to the execution closure. Discovery must retain
        // this generation and reach source approval, which refuses that changed closure.
        let error = resolve_profile_in(&next, root, P, Some(&host)).unwrap_err();
        assert!(
            error.contains("producer closure is not approved on protected staging: crates"),
            "{error}"
        );
    }
    git(root, &["reset", "--hard", base]);
    write(root, &request_path, other_request.to_string());
    write(root, REPORT, other_report);
    let unrelated = commit(root);
    git(root, &["reset", "--hard", base]);
    assert!(P
        .resolve_inputs(
            &mut source(root, ISOLATED, &unrelated).unwrap(),
            Some(&target),
            false
        )
        .unwrap()
        .is_none());
    let mut next = api.clone();
    let mut unrelated_row = rows["workflow_runs"][0].clone();
    unrelated_row["id"] = json!(45);
    unrelated_row["head_sha"] = json!(unrelated);
    let mut rows = rows;
    rows["workflow_runs"]
        .as_array_mut()
        .unwrap()
        .push(unrelated_row);
    rows["total_count"] = json!(2);
    next.0
        .insert(query.clone(), serde_json::to_vec(&rows).unwrap());
    let resolved = resolve_profile_in(&next, root, P, Some(&host)).unwrap();
    assert_eq!(resolved.runs.len(), 11);
    let changing = AssociationChanges {
        api: &next,
        query: &query,
        reads: std::cell::Cell::new(0),
        number: 9,
    };
    let error = resolve_profile_in(&changing, root, P, Some(&host)).unwrap_err();
    assert_eq!(
        error,
        "host population or approved anchor changed during read"
    );
    let ignored = AssociationChanges {
        number: 900,
        reads: std::cell::Cell::new(0),
        ..changing
    };
    assert_eq!(
        resolve_profile_in(&ignored, root, P, Some(&host))
            .unwrap()
            .runs
            .len(),
        11
    );
    coherent_other_obligations(root, base, &target, api, &host, &query);
}

fn coherent_other_obligations(
    root: &Path,
    base: &str,
    target: &DepthRecord,
    api: &Fake,
    host: &EvidenceHost,
    query: &str,
) {
    let request_path = crate::agent_lifecycle::maintenance_request_path(P.agent_id());
    let request = fs::read_to_string(root.join(&request_path)).unwrap();
    let mut other = target.clone();
    let o = other.obligations.as_mut().unwrap();
    let Some(input) = o
        .acquired_inputs
        .iter_mut()
        .find(|f| f.path.ends_with("/RULES.json"))
    else {
        return;
    };
    let mut bytes = fs::read(root.join(&input.path)).unwrap();
    bytes.push(b'\n');
    *input = write(root, &input.path, bytes);
    o.identity = crate::depth_record::part_identity(o).unwrap();
    let mut request: toml_edit::DocumentMut = request.parse().unwrap();
    request["support_depth"]["obligations_identity"] = toml_edit::value(o.identity.clone());
    write(root, &request_path, request.to_string());
    write(root, REPORT, serde_json::to_vec_pretty(&other).unwrap());
    let candidate = commit(root);
    let at = source(root, ISOLATED, &candidate).unwrap();
    assert_eq!(P.resolve_generation(&at).unwrap().unwrap().record, other);
    assert!(P
        .resolve_inputs(&mut at.clone(), Some(target), false)
        .unwrap()
        .is_none());
    let error = P
        .resolve_inputs(&mut at.clone(), Some(target), true)
        .err()
        .unwrap();
    assert_eq!(error, "associated head replaced maintenance generation");
    git(root, &["reset", "--hard", base]);
    let mut next = api.clone();
    let mut rows: Value = serde_json::from_slice(&next.0[query]).unwrap();
    let mut row = rows["workflow_runs"][0].clone();
    row["id"] = json!(47);
    row["head_sha"] = json!(candidate);
    rows["workflow_runs"].as_array_mut().unwrap().push(row);
    rows["total_count"] = json!(2);
    next.0
        .insert(query.into(), serde_json::to_vec(&rows).unwrap());
    assert_eq!(
        resolve_profile_in(&next, root, P, Some(host))
            .unwrap()
            .runs
            .len(),
        11
    );
}

struct AssociationChanges<'a> {
    api: &'a Fake,
    query: &'a str,
    reads: std::cell::Cell<usize>,
    number: u64,
}
impl Api for AssociationChanges<'_> {
    fn repository(&self) -> &str {
        self.api.repository()
    }
    fn bytes(&self, path: &str, limit: usize) -> Result<Vec<u8>, String> {
        let bytes = self.api.bytes(path, limit)?;
        if path != self.query {
            return Ok(bytes);
        }
        let count = self.reads.get();
        self.reads.set(count + 1);
        if count == 0 {
            return Ok(bytes);
        }
        let mut value: Value = serde_json::from_slice(&bytes).unwrap();
        value["workflow_runs"][1]["pull_requests"] =
            json!([{"number":self.number,"ignored_title":"edited prose"}]);
        Ok(serde_json::to_vec(&value).unwrap())
    }
}

fn commit(root: &Path) -> String {
    git(root, &["add", "."]);
    git(
        root,
        &["commit", "-qm", "binding mutation", "--allow-empty"],
    );
    git(root, &["rev-parse", "HEAD"])
}

#[test]
fn canonical_maintenance_generation_association_refusals_and_fixture_independence() {
    let f = Fixture::new_for(P);
    binding_matrix(f.root.path(), &f.commit, &f.api);
}

#[test]
fn unavailable_canonical_report_never_becomes_unrelated_absence() {
    use crate::support_depth::acquisition_inputs::Inputs;
    struct Unavailable(crate::runtime_advertising::source::Source);
    impl Inputs for Unavailable {
        fn optional(&mut self, path: &str) -> Result<Option<Vec<u8>>, String> {
            if path == REPORT {
                return Err("authenticated report transport unavailable".into());
            }
            self.0.optional_bytes(path)
        }
        fn list(&mut self, path: &str) -> Result<Vec<String>, String> {
            Inputs::list(&mut self.0, path)
        }
    }
    let f = Fixture::new_for(P);
    let at = source(f.root.path(), ISOLATED, &f.commit).unwrap();
    let target = P.resolve_generation(&at).unwrap().unwrap().record;
    let error = P
        .resolve_inputs(&mut Unavailable(at), Some(&target), false)
        .err()
        .unwrap();
    assert_eq!(error, "authenticated report transport unavailable");
}

#[test]
fn writer_edits_do_not_change_the_committed_maintenance_generation() {
    use crate::{depth_record::AcceptanceEntry, git_inputs::GitInputs};
    let f = Fixture::new_for(P);
    let root = f.root.path();
    let host = EvidenceHost::for_test(ISOLATED, &f.commit);
    let before = resolve_profile_in(&f.api, root, P, Some(&host)).unwrap();
    let frozen = before.original.clone();
    let mut report = frozen.clone();
    report.execution = Some(assembly::assemble(&before).unwrap().execution);
    report
        .acceptance_entries
        .push(AcceptanceEntry::MaintenanceCloseout {
        closeout: depth_ci::reference(
            "docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-closeout.json",
            b"closeout",
        ),
        record_sha256: "a".repeat(64),
    });
    let bytes = serde_json::to_vec_pretty(&report).unwrap();
    DepthRecord::parse(
        std::str::from_utf8(&bytes).unwrap(),
        P.agent_id(),
        P.version(),
    )
    .unwrap();
    let index = fs::read(root.join(".git/index")).unwrap();
    // The writer has updated E/entries but has not staged or committed its proposed report.
    write(root, REPORT, &bytes);
    let mut working = GitInputs::new(root).unwrap();
    assert_eq!(
        working.required(REPORT).unwrap_err(),
        format!("working bytes disagree with index: {REPORT}")
    );
    let after = resolve_profile_in(&f.api, root, P, Some(&host)).unwrap();
    assert_eq!(after.original, frozen);
    assert_eq!(after.runs.len(), before.runs.len());
    assert_eq!(fs::read(root.join(REPORT)).unwrap(), bytes);
    assert_eq!(fs::read(root.join(".git/index")).unwrap(), index);

    // Once MA5's report is committed, the shared resolver still projects exactly Event/P/O.
    let head = commit(root);
    let at = source(root, ISOLATED, &head).unwrap();
    let projected = P.resolve_generation(&at).unwrap().unwrap().record;
    assert_eq!(projected, frozen);
    assert!(projected.execution.is_none());
    assert!(projected.acceptance_entries.is_empty());
    assert_eq!(
        resolve_profile_in(&f.api, root, P, Some(&host))
            .unwrap()
            .original,
        frozen
    );
    // Projection must follow validation: invalid committed E cannot disappear behind it.
    report.execution.as_mut().unwrap().identity = "b".repeat(64);
    write(root, REPORT, serde_json::to_vec_pretty(&report).unwrap());
    let head = commit(root);
    let error = P
        .resolve_generation(&source(root, ISOLATED, &head).unwrap())
        .err()
        .unwrap();
    assert!(
        error.contains("invalid maintenance generation report"),
        "{error}"
    );
}
