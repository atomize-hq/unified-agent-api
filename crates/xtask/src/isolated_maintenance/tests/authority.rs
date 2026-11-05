//! CP-P1 host part (B.5a): the owning predicates' isolated host borrows a resolved context for
//! bindings, evidence and the integration tip. Production keeps its host, unchanged.
use super::*;
use crate::{
    agent_maintenance::closeout_proposal::Proposal,
    depth_acceptance::{Authority, IsolatedHost},
};

const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";

fn local(root: &Path) -> Result<String, String> {
    let out = crate::git_inputs::command(root)
        .args(["rev-parse", "--verify", "origin/staging^{commit}"])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("no local integration ref".into());
    }
    Ok(String::from_utf8(out.stdout).unwrap().trim().into())
}
fn set_local(root: &Path, commit: &str) {
    git(root, &["update-ref", "refs/remotes/origin/staging", commit]);
}

#[test]
fn proposals_forward_their_own_authority_only() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    // Without a route: the production host, and no route to take a proposal from.
    assert!(route(root).unwrap().is_none());
    let production = Proposal::capture(root).unwrap();
    assert!(matches!(production.authority(), Authority::Production));
    // An installed route's proposal admits under its own borrowed context.
    let _installed = f.install();
    let proposal = route(root).unwrap().unwrap().proposal(root).unwrap();
    assert!(matches!(proposal.authority(), Authority::Isolated(_)));
    // Installing a route never changes a proposal captured without it.
    assert!(matches!(
        Proposal::capture(root).unwrap().authority(),
        Authority::Production
    ));
    // A route resolved for one workspace cannot take a proposal of another.
    let other = Fixture::new_for(&scope::SYNTHETIC);
    let error = route(root)
        .unwrap()
        .unwrap()
        .proposal(other.root.path())
        .err()
        .unwrap();
    assert!(error.contains("changed"), "{error}");
}

#[test]
fn isolated_tip_is_the_authenticated_protected_tip() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    write(root, "docs/input.txt", "second\n");
    f.commit();
    let head = git(root, &["rev-parse", "HEAD"]);
    let context = f.resolve().unwrap();
    for host in [
        IsolatedHost::closeout(&context),
        IsolatedHost::promotion(&context),
    ] {
        // No local integration ref: the local read refuses before any binding.
        assert!(host.test_tip(root, local).is_err());
        set_local(root, &f.baseline);
        let error = host.test_tip(root, local).unwrap_err();
        assert!(error.contains("authenticated protected tip"), "{error}");
        set_local(root, &head);
        assert_eq!(host.test_tip(root, local).unwrap(), head);
        git(root, &["update-ref", "-d", "refs/remotes/origin/staging"]);
    }
    // A moved host tip refuses even when the local ref follows it.
    f.api.put(
        "repos/fixture/isolated/branches/staging",
        json!({"name":"staging","protected":true,"commit":{"sha":f.baseline}}),
    );
    set_local(root, &f.baseline);
    assert!(IsolatedHost::closeout(&context)
        .test_tip(root, local)
        .is_err());
}

#[test]
fn isolated_bindings_and_evidence_refuse_a_foreign_or_moved_caller() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    write(root, RECORD, f.retained());
    f.commit();
    let record =
        crate::depth_record::DepthRecord::parse(&f.retained(), "depth_fixture", "0.2.0").unwrap();
    let context = f.resolve().unwrap();
    let other = Fixture::new_for(&scope::SYNTHETIC);
    for host in [
        IsolatedHost::closeout(&context),
        IsolatedHost::promotion(&context),
    ] {
        // Another workspace is never read through this context.
        assert!(host.test_bindings(other.root.path(), &record).is_err());
        assert!(host.test_evidence(other.root.path(), &record).is_err());
    }
    // A registry edit, uncommitted or committed, moves the caller: refused, never derived.
    f.registry(|doc| doc["agents"][0]["display_name"] = toml_edit::value("moved"));
    assert!(IsolatedHost::closeout(&context)
        .test_bindings(root, &record)
        .is_err());
    f.commit();
    let error = IsolatedHost::promotion(&context)
        .test_bindings(root, &record)
        .unwrap_err();
    assert!(error.contains("changed"), "{error}");
}

#[test]
fn borrowed_evidence_host_evaluates_only_its_compiled_profile() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    let codex = crate::depth_ci::profile::Profile::Core
        .record(include_str!(
            "../../../tests/fixtures/depth-core/record.json"
        ))
        .unwrap();
    let host = crate::isolated_maintenance::EvidenceHost::for_test("fixture/isolated", "0");
    let error =
        crate::depth_evidence::EvidenceVerdicts::authenticate(root, &codex, &host).unwrap_err();
    assert!(error.contains("compiled profile"), "{error}");
}

#[test]
fn production_ordinary_closeout_bytes_are_the_baseline() {
    // Independent oracle: with no route, an ordinary (non-depth) closeout proposal admits under
    // the production host and delivers exactly the proposed bytes, with no acceptance output.
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    assert!(route(root).unwrap().is_none());
    let path = "docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-closeout.json";
    let bytes = b"{\"ordinary\": true}\n".to_vec();
    let proposal = Proposal::capture(root).unwrap();
    assert!(matches!(proposal.authority(), Authority::Production));
    let planned = proposal
        .plan(root, vec![(PathBuf::from(path), bytes.clone())])
        .unwrap();
    assert_eq!(
        format!("{planned:?}"),
        format!(
            "{:?}",
            vec![crate::workspace_mutation::PlannedMutation::create(
                path,
                bytes.clone()
            )]
        )
    );
    proposal
        .apply(root, vec![(PathBuf::from(path), bytes.clone())])
        .unwrap();
    assert_eq!(fs::read(root.join(path)).unwrap(), bytes);
    assert_eq!(
        git(root, &["status", "--porcelain=v1", "--untracked-files=all"]),
        format!("?? {path}")
    );
    // The production evidence entry still reads nothing without a route.
    let record =
        crate::depth_record::DepthRecord::parse(&f.retained(), "depth_fixture", "0.2.0").unwrap();
    let results = crate::depth_evidence::evaluate_isolated_record(root, &record).unwrap();
    assert_eq!(
        format!("{results:?}"),
        format!("{:?}", crate::depth_evidence::EvidenceVerdicts::default())
    );
}

/// The connected evidence chain needs the Linux channel peer anchor (see `ma4_evidence`).
#[cfg(target_os = "linux")]
mod evidence {
    use super::super::{
        ma4_route::run_as,
        prepare::{acquire, workspace},
    };
    use super::*;
    use crate::agent_maintenance::{
        execute,
        prepare::{apply_prepare_plan, build_prepare_plan},
    };
    use crate::depth_ci::profile::Profile;
    use crate::depth_record::{AcceptanceEntry, DepthRecord, DepthResult};

    const ENTRY: &str = "isolated_maintenance::tests::ma4_evidence::executor_entry";

    /// A stand-in executor that answers the preflight and otherwise runs the offline command as
    /// its own, separately located binary.
    fn executor(dir: &Path) -> String {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("executor.sh");
        let exe = dir.join("assembler");
        std::fs::copy(std::env::current_exe().unwrap(), &exe).unwrap();
        std::fs::write(
            &path,
            format!(
                "#!/bin/sh\nprompt=$(cat)\ncase \"$prompt\" in\n*'Repository preflight'*) echo {} ;;\n*) \"{}\" --exact {ENTRY} --ignored --quiet || exit 1 ;;\nesac\n",
                execute::PREFLIGHT_SENTINEL,
                exe.display(),
            ),
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path.to_string_lossy().into_owned()
    }

    /// MA1, acquisition, MA2's re-freeze and MA4 with authenticated reporting E, committed to the
    /// fake protected tip.
    fn delivered(classified: bool) -> (Fixture, crate::depth_evidence::SyntheticHost) {
        let (f, mut args) = workspace();
        let root = f.root.path();
        args.request_commit = "0b1e1f0a478af0cd06d6ba6998ab594f09fbe592".into();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.gitignore"),
            root.join(".gitignore"),
        )
        .unwrap();
        f.commit();
        {
            let _installed = f.install();
            apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
            f.commit();
            acquire(root, "0.2.0");
            f.commit();
            let frozen = build_prepare_plan(root, &args).unwrap();
            apply_prepare_plan(root, &frozen).unwrap();
            f.commit();
            if classified {
                // The approved synthetic CORE classification (B.3) enters as a new generation.
                let [(_, basis)] = frozen.classification_bases.as_slice() else {
                    panic!("one basis: {:?}", frozen.classification_bases)
                };
                let record = DepthRecord::parse(
                    &std::fs::read_to_string(root.join(RECORD)).unwrap(),
                    "depth_fixture",
                    "0.2.0",
                )
                .unwrap();
                let surfaces: Vec<_> = record
                    .obligations
                    .unwrap()
                    .classification_required
                    .into_iter()
                    .map(|c| c.surface)
                    .collect();
                classify(&f, &surfaces, basis);
                f.commit();
                args.request_recorded_at = "2026-11-04T00:00:00Z".into();
                args.request_commit = "0c2e2f1b589bf1de17e7c7c7ab695f10cef6fa03".into();
                apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
                f.commit();
            }
        }
        write(
            root,
            Profile::Synthetic.fixture(),
            std::fs::read(root.join(RECORD)).unwrap(),
        );
        let host = crate::depth_evidence::synthetic_host(root);
        f.host();
        let bin = tempfile::tempdir().unwrap();
        let binary = executor(bin.path());
        {
            let _installed = f.install();
            let _reader = host.install();
            let _gates = execute::gate_stub::stub();
            run_as(root, &binary, "authority").unwrap();
        }
        f.commit();
        host.staging(&git(root, &["rev-parse", "HEAD"]));
        (f, host)
    }

    fn results(
        verdicts: &crate::depth_evidence::EvidenceVerdicts,
        record: &DepthRecord,
    ) -> Vec<DepthResult> {
        Profile::Synthetic
            .obligations(record)
            .iter()
            .map(|o| verdicts.result(record, o))
            .collect()
    }

    #[test]
    fn isolated_host_borrows_the_callers_context_and_reauthenticates_each_record() {
        let (f, host) = delivered(false);
        let root = f.root.path();
        let record = DepthRecord::parse(
            &std::fs::read_to_string(root.join(RECORD)).unwrap(),
            "depth_fixture",
            "0.2.0",
        )
        .unwrap();
        assert!(record.execution.is_some());
        let unverified = vec![DepthResult::Unverified; 11];
        let _reader = host.install();
        // No route is installed: the root-resolving entry finds no binding and reads nothing.
        let unrouted = crate::depth_evidence::evaluate_isolated_record(root, &record).unwrap();
        assert_eq!(results(&unrouted, &record), unverified);
        // The isolated host borrows the caller's resolved context instead.
        let context = f.resolve().unwrap();
        for owner in [
            IsolatedHost::closeout(&context),
            IsolatedHost::promotion(&context),
        ] {
            let verdicts = owner.test_evidence(root, &record).unwrap();
            assert_eq!(results(&verdicts, &record), vec![DepthResult::Verified; 11]);
            // Sealed to the evaluated record: an appended entry is never resealed.
            let mut entered = record.clone();
            entered.acceptance_entries.push(AcceptanceEntry::Promotion {
                targets: vec![],
                record_sha256: record.stated_sha256().unwrap(),
            });
            assert_eq!(results(&verdicts, &entered), unverified);
        }
        // Bindings are the production derivation, read through the scoped registry.
        assert_eq!(
            format!(
                "{:?}",
                IsolatedHost::closeout(&context).test_bindings(root, &record)
            ),
            format!(
                "{:?}",
                crate::support_depth::current_closeout_bindings(root, &record)
            )
        );
        assert_eq!(
            format!(
                "{:?}",
                IsolatedHost::promotion(&context).test_bindings(root, &record)
            ),
            format!(
                "{:?}",
                crate::support_depth::current_promotion_bindings(root, &record)
            )
        );
        // A forged host tip never authenticates, whatever E claims.
        let forged =
            crate::isolated_maintenance::EvidenceHost::for_test("fixture/isolated", &f.baseline);
        assert!(
            crate::depth_evidence::EvidenceVerdicts::authenticate(root, &record, &forged).is_err()
        );
        // A changed population is re-authenticated on the next call, not reused.
        let changed = host.fork();
        changed.revoke_adoption();
        let _changed = changed.install();
        let after = IsolatedHost::closeout(&context).test_evidence(root, &record);
        assert!(
            after.map_or(true, |v| results(&v, &record) == unverified),
            "revoked adoption still verified"
        );
    }

    /// The approved synthetic classification (B.3, as in `prepare.rs`): every owned `run` unit
    /// selects only CORE.
    fn classify(
        f: &Fixture,
        surfaces: &[crate::agent_maintenance::support_audit::SurfaceIdentity],
        basis: &str,
    ) {
        f.registry(|doc| {
            let entry = doc["agents"]
                .as_array_of_tables_mut()
                .unwrap()
                .iter_mut()
                .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
                .unwrap();
            let operation = &mut entry["support_depth"]["enrollments"][0]["operations"][0];
            let mut rows = toml_edit::Array::new();
            for surface in surfaces {
                let mut identity = toml_edit::InlineTable::new();
                identity.insert("surface_kind", surface.surface_kind.as_str().into());
                identity.insert("command_path", surface.command_path.as_str().into());
                identity.insert("surface_id", surface.surface_id.as_str().into());
                let mut row = toml_edit::InlineTable::new();
                row.insert("surface", identity.into());
                row.insert(
                    "templates",
                    ["CORE"].into_iter().collect::<toml_edit::Array>().into(),
                );
                row.insert(
                    "rationale",
                    "CORE covers this whole `run --json` unit.".into(),
                );
                rows.push(row);
            }
            operation["classification_basis"] = toml_edit::value(basis);
            operation["classifications"] = toml_edit::value(rows);
        });
    }
    fn digest(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        hex::encode(Sha256::digest(bytes))
    }
    fn tracked(root: &Path) -> (String, String, Vec<u8>) {
        (
            git(root, &["rev-parse", "HEAD"]),
            git(root, &["status", "--porcelain=v1", "--untracked-files=all"]),
            std::fs::read(root.join(RECORD)).unwrap(),
        )
    }
    /// Commit the caller and move the fake protected tip, the evidence host and the local
    /// integration ref to it.
    fn advance(f: &Fixture, host: &crate::depth_evidence::SyntheticHost) -> String {
        f.commit();
        let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
        host.staging(&tip);
        set_local(f.root.path(), &tip);
        tip
    }
    fn entries(planned: &[crate::workspace_mutation::PlannedMutation]) -> String {
        format!("{planned:?}")
    }

    /// The private owning predicates themselves, reached through an isolated proposal: MA5
    /// closeout and then MA6 promotion admit only under the proposal's forwarded authority.
    #[test]
    fn isolated_proposal_admits_closeout_and_promotion_through_the_owning_predicates() {
        use crate::agent_maintenance::stand_down;
        let (f, host) = delivered(true);
        let root = f.root.path();
        let _reader = host.install();
        let record = DepthRecord::parse(
            &std::fs::read_to_string(root.join(RECORD)).unwrap(),
            "depth_fixture",
            "0.2.0",
        )
        .unwrap();
        let crate::depth_record::Event::Maintenance {
            request_recorded_at,
            ..
        } = &record.event
        else {
            panic!("not a maintenance generation");
        };
        // The exact-generation marker on the protected tip.
        let marker = stand_down::marker_relative_path(
            "docs/agents/lifecycle/depth_fixture-maintenance",
            "0.2.0",
        );
        write(
            root,
            &marker,
            stand_down::render_marker_toml("depth_fixture", "0.2.0", request_recorded_at),
        );
        let tip = advance(&f, &host);
        let request = crate::agent_lifecycle::maintenance_request_path("depth_fixture");
        let closeout_path =
            "docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-closeout.json";
        let closeout = serde_json::to_vec_pretty(&json!({
            "request_ref": request,
            "request_sha256": digest(&std::fs::read(root.join(&request)).unwrap()),
            "resolved_findings": [{"category_id":"governance_doc_drift","summary":"fixture","surfaces":["docs/input.txt"]}],
            "explicit_none_reason": "fixture",
            "preflight_passed": true,
            "recorded_at": request_recorded_at,
            "commit": tip,
        }))
        .unwrap();
        let files = vec![(PathBuf::from(closeout_path), closeout.clone())];
        let before = tracked(root);

        // Production on the same, otherwise valid, depth proposal: its host has no verdicts.
        let error = Proposal::capture(root)
            .unwrap()
            .plan(root, files.clone())
            .unwrap_err();
        assert!(error.contains("not independently satisfied"), "{error}");
        assert_eq!(tracked(root), before);

        // A local integration ref other than the authenticated tip refuses.
        let parent = git(root, &["rev-parse", "HEAD~1"]);
        set_local(root, &parent);
        let error = f
            .resolve()
            .unwrap()
            .proposal(root)
            .unwrap()
            .plan(root, files.clone())
            .unwrap_err();
        assert!(error.contains("authenticated protected tip"), "{error}");
        set_local(root, &tip);
        assert_eq!(tracked(root), before);

        // A context that moved after capture refuses (here: the protected tip moved on the host).
        let proposal = f.resolve().unwrap().proposal(root).unwrap();
        f.api.put(
            "repos/fixture/isolated/branches/staging",
            json!({"name":"staging","protected":true,"commit":{"sha":parent}}),
        );
        assert!(proposal.plan(root, files.clone()).is_err());
        assert!(proposal.apply(root, files.clone()).is_err());
        f.host();
        assert_eq!(tracked(root), before);

        // Success: one closeout and exactly one current closeout entry, applied as one batch.
        let proposal = f.resolve().unwrap().proposal(root).unwrap();
        assert!(matches!(proposal.authority(), Authority::Isolated(_)));
        let planned = proposal.plan(root, files.clone()).unwrap();
        assert_eq!(planned.len(), 2, "{}", entries(&planned));
        proposal.apply(root, files.clone()).unwrap();
        assert_eq!(std::fs::read(root.join(closeout_path)).unwrap(), closeout);
        let closed = DepthRecord::parse(
            &std::fs::read_to_string(root.join(RECORD)).unwrap(),
            "depth_fixture",
            "0.2.0",
        )
        .unwrap();
        assert_eq!(
            closed.acceptance_entries,
            vec![AcceptanceEntry::MaintenanceCloseout {
                closeout: crate::depth_record::FileRef {
                    path: closeout_path.into(),
                    sha256: digest(&closeout),
                },
                record_sha256: record.stated_sha256().unwrap(),
            }]
        );
        advance(&f, &host);

        // MA6: retire the marker; the promotion predicate appends its own entry.
        let promote = |authority: Option<()>| {
            let proposal = match authority {
                Some(()) => f.resolve().unwrap().proposal(root).unwrap(),
                None => Proposal::capture(root).unwrap(),
            };
            proposal.plan_changes(root, vec![(PathBuf::from(&marker), None)], |r, b, a, w| {
                crate::depth_acceptance::plan_promotion_as(proposal.authority(), r, b, a, w)
            })
        };
        let before = tracked(root);
        let error = promote(None).unwrap_err();
        assert!(error.contains("not independently satisfied"), "{error}");
        let tip = git(root, &["rev-parse", "HEAD"]);
        set_local(root, &git(root, &["rev-parse", "HEAD~1"]));
        let error = promote(Some(())).unwrap_err();
        assert!(error.contains("authenticated protected tip"), "{error}");
        set_local(root, &tip);
        assert_eq!(tracked(root), before);
        let planned = promote(Some(())).unwrap();
        assert_eq!(planned.len(), 2, "{}", entries(&planned));
        crate::workspace_mutation::apply_admitted_mutations(root, &planned).unwrap();
        assert!(!root.join(&marker).exists());
        let promoted = DepthRecord::parse(
            &std::fs::read_to_string(root.join(RECORD)).unwrap(),
            "depth_fixture",
            "0.2.0",
        )
        .unwrap();
        assert_eq!(
            promoted.acceptance_entries[..1],
            closed.acceptance_entries[..]
        );
        assert_eq!(
            promoted.acceptance_entries[1..],
            [AcceptanceEntry::Promotion {
                targets: vec!["x86_64-unknown-linux-gnu".into()],
                record_sha256: record.stated_sha256().unwrap(),
            }]
        );
    }
}
