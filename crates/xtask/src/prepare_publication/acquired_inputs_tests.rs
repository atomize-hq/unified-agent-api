//! ON3–ON5 exercise the same entrypoint body after the unchanged production depth guard.
use super::*;
use crate::{prepare_publication as prepare, publication_refresh as refresh};

#[allow(dead_code)]
#[path = "../close_proving_run.rs"]
mod close;
#[path = "../../tests/support/lifecycle_depth.rs"]
mod lifecycle_depth;

#[derive(Clone, Copy, Debug)]
enum Entrypoint {
    Prepare,
    Refresh,
    Close,
}
const ENTRYPOINTS: [Entrypoint; 3] = [Entrypoint::Prepare, Entrypoint::Refresh, Entrypoint::Close];

impl Entrypoint {
    fn run(self, root: &Path, public: bool) -> String {
        let mut output = Vec::new();
        match self {
            Self::Prepare => {
                let args = prepare::Args {
                    approval: APPROVAL.into(),
                    check: false,
                    write: true,
                };
                if public {
                    prepare::run_in_workspace(root, args, &mut output)
                } else {
                    prepare::isolated::run(root, args, &mut output)
                }
                .unwrap_err()
                .to_string()
            }
            Self::Refresh => {
                let args = refresh::Args {
                    approval: APPROVAL.into(),
                    check: false,
                    write: true,
                };
                if public {
                    refresh::run_in_workspace(root, args, &mut output)
                } else {
                    refresh::isolated::run(root, args, &mut output)
                }
                .unwrap_err()
                .to_string()
            }
            Self::Close => {
                let args = close::Args {
                    approval: APPROVAL.into(),
                    closeout: format!(
                        "docs/agents/lifecycle/{PACK}/governance/proving-run-closeout.json"
                    )
                    .into(),
                };
                if public {
                    close::run_in_workspace(root, args, &mut output)
                } else {
                    close::isolated::run(root, args, &mut output)
                }
                .unwrap_err()
                .to_string()
            }
        }
    }
}

fn unchanged(f: &Fixture, entry: Entrypoint, expected: &str, paths: &[String]) {
    let before = lifecycle_depth::snapshot(f.root.path());
    let error = entry.run(f.root.path(), false);
    assert!(
        error.contains(expected),
        "{entry:?}: expected {expected}: {error}"
    );
    for path in paths {
        assert!(error.contains(path), "{entry:?}: missing {path}: {error}");
    }
    assert_eq!(
        before,
        lifecycle_depth::snapshot(f.root.path()),
        "{entry:?} wrote on refusal"
    );
}

#[test]
fn publication_entrypoints_refuse_each_changed_frozen_acquired_input_without_writes() {
    let f = setup(true);
    let _installed = f.install();
    freeze(&f).unwrap();
    let inputs = json_file(&f, RECORD)["obligations"]["acquired_inputs"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(
        inputs.len(),
        8,
        "the shared single-target O includes all OF-P6 inputs"
    );
    for input in inputs {
        let path = input["path"].as_str().unwrap().to_string();
        let bytes = fs::read(f.root.path().join(&path)).unwrap();
        write(f.root.path(), &path, [bytes.as_slice(), b"\n"].concat());
        f.commit();
        for entry in ENTRYPOINTS {
            unchanged(
                &f,
                entry,
                "frozen acquired inputs",
                std::slice::from_ref(&path),
            );
        }
        write(f.root.path(), &path, bytes);
    }
}

#[test]
fn publication_entrypoints_name_all_changed_and_deleted_acquired_paths() {
    let f = setup(true);
    let _installed = f.install();
    freeze(&f).unwrap();
    let paths = vec![
        format!("{ROOT}/wrapper_coverage.json"),
        format!("{ROOT}/SCHEMA.json"),
    ];
    write(f.root.path(), &paths[0], "changed");
    fs::remove_file(f.root.path().join(&paths[1])).unwrap();
    f.commit();
    for entry in ENTRYPOINTS {
        unchanged(&f, entry, "frozen acquired inputs", &paths);
    }
}

#[test]
fn publication_entrypoints_unchanged_freeze_reaches_the_existing_approval_refusal() {
    let f = setup(true);
    let _installed = f.install();
    freeze(&f).unwrap();
    f.commit();
    for entry in ENTRYPOINTS {
        unchanged(&f, entry, "`backend_module`", &[]);
        let before = lifecycle_depth::snapshot(f.root.path());
        assert!(entry
            .run(f.root.path(), true)
            .contains("unsupported depth write"));
        assert_eq!(before, lifecycle_depth::snapshot(f.root.path()));
    }
}

#[cfg(unix)]
#[test]
fn publication_entrypoints_with_depth_bindings_keep_strict_source_reads() {
    let f = setup(true);
    let _installed = f.install();
    freeze(&f).unwrap();
    let path = f.root.path().join(APPROVAL);
    fs::rename(&path, path.with_file_name("approval-target.toml")).unwrap();
    std::os::unix::fs::symlink("approval-target.toml", &path).unwrap();
    f.commit();
    for entry in ENTRYPOINTS {
        unchanged(
            &f,
            entry,
            "source symlink is unsupported",
            &[APPROVAL.into()],
        );
    }
}

#[test]
fn publication_entrypoints_without_depth_bindings_keep_the_existing_behavior() {
    let f = setup(true);
    edit_approval(&f, |approval| {
        approval.as_table_mut().remove("support_depth");
    });
    f.commit();
    let _installed = f.install();
    let before: Vec<_> = ENTRYPOINTS
        .map(|entry| entry.run(f.root.path(), false))
        .into();
    write(
        f.root.path(),
        &format!("{ROOT}/wrapper_coverage.json"),
        "changed",
    );
    f.commit();
    for (entry, prior) in ENTRYPOINTS.into_iter().zip(before) {
        unchanged(&f, entry, "`backend_module`", &[]);
        assert_eq!(prior, entry.run(f.root.path(), false));
    }
}

#[test]
fn publication_entrypoints_with_p_only_reach_the_existing_refusal() {
    let f = setup(false);
    let _installed = f.install();
    freeze(&f).unwrap();
    f.commit();
    for entry in ENTRYPOINTS {
        unchanged(&f, entry, "`backend_module`", &[]);
    }
}

#[test]
fn publication_entrypoints_do_not_treat_p_preconditions_or_run_files_as_acquired() {
    let f = setup(true);
    let _installed = f.install();
    freeze(&f).unwrap();
    for path in [
        format!("{ROOT}/versions/0.2.0.json"),
        format!("{ROOT}/VALIDATOR_SPEC.md"),
        format!("{RUN}/input-contract.json"),
    ] {
        write(f.root.path(), &path, "changed");
    }
    f.commit();
    for entry in ENTRYPOINTS {
        unchanged(&f, entry, "`backend_module`", &[]);
    }
}

#[test]
fn publication_entrypoints_refuse_deleted_lifecycle_or_record_bindings() {
    for missing_object in [true, false] {
        let f = setup(true);
        let _installed = f.install();
        freeze(&f).unwrap();
        if missing_object {
            edit_json(&f, STATE, |state| {
                state.as_object_mut().unwrap().remove("support_depth");
            });
        } else {
            fs::remove_file(f.root.path().join(RECORD)).unwrap();
        }
        f.commit();
        for entry in ENTRYPOINTS {
            unchanged(&f, entry, "DEPTH_LIFECYCLE_BINDING_", &[]);
        }
    }
}
