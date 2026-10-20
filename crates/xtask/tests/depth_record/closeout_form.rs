//! The closeouts the repository holds are in form for check 10 of `manifest-validate`, which reads them
//! through the raw structs their loaders use (child module of `depth_record.rs`). A change to a closeout's
//! schema that a committed closeout does not meet fails here, before it turns the closeout of an agent
//! with a depth record into a violation.

use std::{fs, path::PathBuf};

use xtask::{agent_maintenance::closeout, proving_run_closeout};

type Form = fn(&str) -> Result<(), String>;

#[test]
fn every_closeout_the_repository_holds_is_in_form() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/agents/lifecycle");
    let mut read = Vec::new();
    for entry in fs::read_dir(&root).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let (file, form) = if name.ends_with("-maintenance") {
            (
                "maintenance-closeout.json",
                closeout::check_closeout_form as Form,
            )
        } else {
            (
                "proving-run-closeout.json",
                proving_run_closeout::check_closeout_form as Form,
            )
        };
        let path = entry.path().join("governance").join(file);
        if let Ok(text) = fs::read_to_string(&path) {
            form(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
            read.push(name);
        }
    }
    // Three maintenance closeouts and five proving-run closeouts are committed; reading none would
    // pass for the wrong reason.
    assert!(read.len() >= 8, "the closeouts this test read: {read:?}");
}
