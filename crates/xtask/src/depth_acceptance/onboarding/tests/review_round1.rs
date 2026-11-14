use super::*;

// Both sources are independently well formed; codex is registered but has no depth record.
pub(super) fn foreign_additions(files: &mut BTreeMap<String, String>, runtime: bool) {
    let path = if runtime {
        "crates/agent_api/Cargo.toml"
    } else {
        crate::capability_matrix::DEFAULT_OUT_PATH
    };
    let raw = files
        .get(path)
        .cloned()
        .unwrap_or_else(|| std::fs::read_to_string(repo().join(path)).unwrap());
    let changed = if runtime {
        assert!(raw.contains("default = []"));
        raw.replace("default = []", "default = [\"codex\"]")
    } else if !raw
        .lines()
        .any(|line| line.starts_with("| capability id |"))
    {
        // The connected wrapper-only ON3 fixture has an empty capability inventory.
        format!("{raw}\n## `agent_api.core`\n\n| capability id | `codex` |\n|---|---|\n| `agent_api.run` | ✅ |\n")
    } else {
        let header = raw
            .lines()
            .find(|line| line.starts_with("| capability id |"))
            .unwrap();
        let columns: Vec<_> = header
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .skip(1)
            .collect();
        assert!(columns.contains(&"`codex`"));
        let column = columns
            .iter()
            .position(|column| *column == "`codex`")
            .unwrap()
            + 1;
        let row = raw
            .lines()
            .find(|line| line.starts_with("| `backend.claude_code.print_stream_json` |"))
            .unwrap();
        let mut cells: Vec<_> = row.trim_matches('|').split('|').map(str::trim).collect();
        assert_eq!(cells[column], "—");
        cells[column] = "✅";
        raw.replace(row, &format!("| {} |", cells.join(" | ")))
    };
    files.insert(path.into(), changed);
}

#[test]
fn foreign_advertising_without_records_refuses_raw_additions() {
    for case in ["capability", "runtime", "both", "empty_inventory"] {
        let mut f = Fixture::new(Kind::Publication);
        if case == "empty_inventory" {
            let path = crate::capability_matrix::DEFAULT_OUT_PATH;
            let raw = std::fs::read_to_string(repo().join(path)).unwrap();
            let empty = raw.split_once("\n## `").unwrap().0.to_owned() + "\n\n";
            f.before.insert(path.into(), empty.clone());
            f.after.insert(path.into(), empty);
        }
        let registry = f.view().registry;
        assert!(View::with_registry(source(&f.after), &registry, "codex")
            .unwrap()
            .records
            .is_empty());
        if case != "runtime" {
            foreign_additions(&mut f.after, false);
        }
        if case == "runtime" || case == "both" {
            foreign_additions(&mut f.after, true);
        }
        // Maintenance deliberately retains record-scoped output, even for these raw additions.
        let mut maintenance_effects = std::collections::BTreeSet::new();
        super::super::super::publication::detect(
            &View::with_registry(source(&f.before), &registry, "codex").unwrap(),
            &View::with_registry(source(&f.after), &registry, "codex").unwrap(),
            &mut maintenance_effects,
        )
        .unwrap();
        assert!(maintenance_effects.is_empty(), "{case}");
        assert_eq!(
            effects::check(
                &source(&f.before),
                &source(&f.after),
                &f.view(),
                &f.record,
                f.kind
            )
            .unwrap_err(),
            "onboarding batch has acceptance outside its owned generation",
            "{case}"
        );
    }
}

#[cfg(target_os = "linux")]
pub(super) fn connected_refusals(
    root: &Path,
    proposal: &crate::agent_maintenance::closeout_proposal::Proposal,
    captured: &crate::isolated_maintenance::CapturedRegistry,
    view: &crate::agent_maintenance::closeout_proposal::IsolatedView<'_>,
    scratch: &Path,
    record: &DepthRecord,
) {
    // A real, already accepted post-entry ON3 proposal is the control for every mutation.
    currentness_refusals(scratch, record);
    let refuse = |expected: &str| {
        let rendered = std::cell::Cell::new(false);
        let error = evaluate_proposed(root, proposal, captured, view, Kind::Publication, |_| {
            rendered.set(true);
            Ok(())
        })
        .unwrap_err();
        assert_eq!(error, expected);
        assert!(!rendered.get(), "forbidden proposal reached rendering");
    };
    let metadata_path = format!("{ROOT}/versions/{VERSION}.json");
    let mut metadata: Value =
        serde_json::from_slice(&std::fs::read(scratch.join(&metadata_path)).unwrap()).unwrap();
    metadata["status"] = json!("supported");
    for (path, bytes, expected) in [
        (
            format!("{ROOT}/pointers/latest_validated/x86_64-unknown-linux-gnu.txt"),
            "0.2.0\n".into(),
            "onboarding Publication forbids acceptance effect latest_validated_by_target",
        ),
        (
            metadata_path,
            encode(&metadata),
            "onboarding Publication forbids acceptance effect status",
        ),
        (
            "cli_manifests/codex/versions/99.0.0.json".into(),
            encode(
                &json!({"schema_version":1,"semantic_version":"99.0.0","status":"supported","updated_at":STAMP,"validation":{"passed_targets":[]}}),
            ),
            "onboarding batch has acceptance outside its owned generation",
        ),
    ] {
        eprintln!("review round 1 proposal refusal: {path}");
        if let Some((manifest_root, _)) = path.split_once("/versions/") {
            let mut schema: Value = serde_json::from_slice(
                &std::fs::read(
                    scratch
                        .join(manifest_root)
                        .join("VERSION_METADATA_SCHEMA.json"),
                )
                .unwrap(),
            )
            .unwrap();
            // These self-contained schemas use repository-relative IDs in committed bytes.
            schema.as_object_mut().unwrap().remove("$id");
            assert!(
                jsonschema::JSONSchema::options()
                    .with_draft(jsonschema::Draft::Draft202012)
                    .compile(&schema)
                    .unwrap()
                    .is_valid(&serde_json::from_str::<Value>(&bytes).unwrap()),
                "invalid metadata fixture: {path}"
            );
        }
        let full = scratch.join(&path);
        let prior = std::fs::read(&full).ok();
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, &bytes).unwrap();
        refuse(expected);
        assert_eq!(std::fs::read_to_string(&full).unwrap(), bytes);
        if let Some(prior) = prior {
            std::fs::write(full, prior).unwrap();
        } else {
            std::fs::remove_file(full).unwrap();
        }
    }
    let path = crate::capability_matrix::DEFAULT_OUT_PATH;
    let original = std::fs::read_to_string(scratch.join(path)).unwrap();
    eprintln!("review round 1 advertising refusal: {path}");
    let mut changes = BTreeMap::from([(path.into(), original.clone())]);
    foreign_additions(&mut changes, false);
    std::fs::write(scratch.join(path), &changes[path]).unwrap();
    refuse("onboarding batch has acceptance outside its owned generation");
    std::fs::write(scratch.join(path), original).unwrap();
    evaluate_proposed(
        root,
        proposal,
        captured,
        view,
        Kind::Publication,
        |_| Ok(()),
    )
    .unwrap();
}

// Give the real derivation a valid Git view of changed inputs, rather than tripping the
// immutable proposal's working-file guard. The frozen record remains the actual ON3 output.
#[cfg(target_os = "linux")]
fn currentness_refusals(scratch: &Path, record: &DepthRecord) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let mut paths: Vec<_> = record
        .obligations
        .as_ref()
        .unwrap()
        .acquired_inputs
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    paths.extend([
        REGISTRY_RELATIVE_PATH,
        APPROVAL,
        RECORD,
        crate::agent_maintenance::support_audit::NON_TUI_SUPPORT_DEBT_PATH,
    ]);
    for path in paths {
        let full = root.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::copy(scratch.join(path), full).unwrap();
    }
    let git = |args: &[&str]| {
        let output = crate::git_inputs::command(root)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    };
    git(&["init", "-q"]);
    git(&["add", "."]);
    let current = || {
        let mut inputs = crate::git_inputs::GitInputs::new(root).unwrap();
        adequacy::current(record, "onboarding", || {
            crate::support_depth::current_onboarding_bindings(
                &mut inputs,
                &Source::workspace(root).unwrap(),
                record,
            )
        })
    };
    current().unwrap();
    for path in [
        format!("{ROOT}/snapshots/{VERSION}/union.json"),
        format!("{ROOT}/snapshots/{VERSION}/x86_64-unknown-linux-gnu.json"),
        format!("{ROOT}/wrapper_coverage.json"),
        format!("{ROOT}/reports/{VERSION}/coverage.any.json"),
    ] {
        eprintln!("review round 1 derived currentness refusal: {path}");
        assert!(record
            .obligations
            .as_ref()
            .unwrap()
            .acquired_inputs
            .iter()
            .any(|file| file.path == path));
        let original = std::fs::read(root.join(&path)).unwrap();
        let mut changed = original.clone();
        changed.push(b'\n');
        std::fs::write(root.join(&path), changed).unwrap();
        git(&["add", &path]);
        assert_eq!(
            current().unwrap_err(),
            "onboarding frozen O is no longer current",
            "{path}"
        );
        std::fs::write(root.join(&path), original).unwrap();
        git(&["add", &path]);
    }
    let path = REGISTRY_RELATIVE_PATH;
    let original = std::fs::read_to_string(root.join(path)).unwrap();
    let changed = original.replace("Synthetic machinery-proof wrapper promise; see depth-synthetic/contract.md; qualifies no real agent.", "Changed reviewed policy promise");
    assert_ne!(changed, original);
    std::fs::write(root.join(path), changed).unwrap();
    git(&["add", path]);
    assert_eq!(
        current().unwrap_err(),
        "frozen policy is no longer current; a new generation is required"
    );
    std::fs::write(root.join(path), original).unwrap();
    git(&["add", path]);
    current().unwrap();
}
