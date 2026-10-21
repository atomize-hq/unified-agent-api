use std::{path::PathBuf, process::Command};

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

#[test]
fn checked_in_runtime_advertising_model_is_conformant() {
    let out = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["runtime-advertising-model", "--check"])
        .current_dir(workspace())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

use xtask::runtime_advertising::{
    projection::AGENTS,
    revision::{Context, Revision},
};
#[path = "runtime_advertising/behavior.rs"]
mod behavior;
#[path = "runtime_advertising/fixtures.rs"]
mod fixtures;
#[path = "runtime_advertising/git_views.rs"]
mod git_views;
#[path = "runtime_advertising/guards.rs"]
mod guards;

#[path = "runtime_advertising/wiring.rs"]
mod wiring;
