use super::*;
use serde_json::json;
fn scope() -> Scope {
    crate::depth_evidence::EvidenceReceipt::parse(include_str!(
        "../../../../../tests/fixtures/depth-evidence-v1.json"
    ))
    .unwrap()
    .scope
}
fn repo() -> form::Repository {
    form::Repository {
        name: "owner/proof".into(),
        id: 1,
        owner_id: 2,
    }
}
fn comment<T: serde::Serialize>(id: u64, kind: &str, v: &T) -> Comment {
    Comment {
        id,
        body: format!("```{kind}\n{}\n```", serde_json::to_string(v).unwrap()),
        author_id: 7,
        login: "maintainer".into(),
        authorized: true,
    }
}
fn start(id: u64) -> Comment {
    comment(
        id,
        "depth-native-start-v1",
        &form::Start {
            schema_version: 1,
            profile: "isolated-codex-native-v1".into(),
            repository: repo(),
            pull_request: 8,
            packet_head: "a".repeat(40),
            scope: scope(),
            plan: FileRef {
                path: "reports/plan.json".into(),
                sha256: "c".repeat(64),
            },
        },
    )
}
fn finish(start: &Comment, id: u64, ids: Vec<u64>) -> Comment {
    comment(
        id,
        "depth-native-finish-v1",
        &form::Finish {
            schema_version: 1,
            registration: start.reference(),
            evidence_commit: "b".repeat(40),
            bundle: FileRef {
                path: "reports/bundle.json".into(),
                sha256: "d".repeat(64),
            },
            executor_identity: "execution-one".into(),
            population_through_id: *ids.iter().max().unwrap(),
            registrations: ids,
            execution_attested: true,
            outcomes_attested: true,
            registration_order_attested: true,
            completeness_attested: true,
        },
    )
}
fn discover_for(cs: &[Comment], bound: &[u64]) -> Result<Vec<Attempt>, String> {
    discover(cs, &scope(), &repo(), 8, &bound.iter().copied().collect())
}
#[test]
fn snapshot_a_does_not_erase_later_unfinished_attempt_b() {
    let a = start(10);
    let fa = finish(&a, 20, vec![10]);
    let b = start(30);
    let fb = finish(&b, 40, vec![10, 30]);
    assert_eq!(
        discover_for(&[a.clone(), fa.clone()], &[]).unwrap().len(),
        1
    );
    assert!(discover_for(&[a.clone(), fa.clone(), b.clone()], &[]).is_err());
    assert_eq!(discover_for(&[a, fa, b, fb], &[]).unwrap().len(), 2);
}
#[test]
fn copied_unauthorized_records_cannot_poison_but_bound_revoked_records_refuse() {
    let a = start(10);
    let fa = finish(&a, 20, vec![10]);
    let mut rogue = start(30);
    rogue.authorized = false;
    assert_eq!(
        discover_for(&[a.clone(), fa.clone(), rogue.clone()], &[])
            .unwrap()
            .len(),
        1
    );
    assert!(discover_for(&[a.clone(), fa.clone(), rogue], &[30]).is_err());
    let mut duplicate = fa.clone();
    duplicate.id = 40;
    duplicate.authorized = false;
    assert_eq!(
        discover_for(&[a.clone(), fa.clone(), duplicate.clone()], &[])
            .unwrap()
            .len(),
        1
    );
    duplicate.authorized = true;
    assert!(discover_for(&[a, fa, duplicate], &[]).is_err());
}
#[test]
fn edited_registration_or_favorable_prefix_omission_refuses() {
    let a = start(10);
    let mut fa = finish(&a, 20, vec![10]);
    let mut v: serde_json::Value = serde_json::from_str(
        fa.body
            .split_once('\n')
            .unwrap()
            .1
            .split_once("\n```")
            .unwrap()
            .0,
    )
    .unwrap();
    v["registration"]["body_sha256"] = json!("0".repeat(64));
    fa = comment(20, "depth-native-finish-v1", &v);
    assert!(discover_for(&[a.clone(), fa], &[]).is_err());
    let b = start(30);
    let fb = finish(&b, 40, vec![30]);
    assert!(discover_for(&[a, b, fb], &[]).is_err());
}
#[test]
fn surviving_authoritative_finish_cannot_hide_its_deleted_start() {
    let a = start(10);
    let fa = finish(&a, 20, vec![10]);
    let b = start(30);
    let fb = finish(&b, 40, vec![10, 30]);
    assert_eq!(
        discover_for(&[a.clone(), fa.clone(), b, fb.clone()], &[])
            .unwrap()
            .len(),
        2
    );
    assert!(
        discover_for(&[a, fa, fb], &[]).is_err(),
        "surviving finish binds the missing later registration"
    );
}
#[test]
fn another_generation_is_irrelevant_only_after_its_start_is_resolved() {
    let a = start(10);
    let fa = finish(&a, 20, vec![10]);
    let mut v: serde_json::Value = serde_json::from_str(
        block(&start(30).body, "depth-native-start-v1")
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    v["scope"]["policy_identity"] = json!("0".repeat(64));
    let b = comment(30, "depth-native-start-v1", &v);
    let fb = finish(&b, 40, vec![30]);
    assert_eq!(
        discover_for(&[a.clone(), fa.clone(), b, fb.clone()], &[])
            .unwrap()
            .len(),
        1
    );
    assert!(discover_for(&[a, fa, fb], &[]).is_err());
}
