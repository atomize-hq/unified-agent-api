//! NE1–NE8: neutral writers refuse a depth workspace or manifest before any write.

use super::support::{Depth, Workspace, MANIFEST_ROOT};

/// The registry loader's own refusal of a declared table (Annex A "Public refusal"). Routes whose
/// generation loads the registry before reaching their writer guard stop here when depth is
/// declared; a retained record with no declaration still reaches the writer guard.
const LOADER: &str = "carries `support_depth`, which no command enforces yet";

/// `Depth::Retained` is left out: with the subject still registered for support publication, the
/// generator first fails reading the synthetic root's absent `current.json`, which is not a depth
/// refusal. The orphaned record reaches the writer guard.
#[test]
fn ne1_support_matrix_write_refuses_depth_workspace() {
    for depth in [Depth::Declared, Depth::Orphaned] {
        let reason = match depth {
            Depth::Declared => LOADER.to_owned(),
            _ => format!("support-matrix writer: {}", depth.guard_reason()),
        };
        Workspace::with(depth).assert_refuses(&["support-matrix"], &reason);
    }
}

#[test]
fn ne2_capability_matrix_write_refuses_depth_workspace() {
    for depth in Depth::ALL {
        let ws = Workspace::with(depth);
        let reason = format!("capability-matrix: {}", depth.guard_reason());
        ws.assert_refuses(&["capability-matrix"], &reason);
        ws.assert_refuses(
            &["capability-matrix", "--out", "elsewhere/matrix.md"],
            &reason,
        );
    }
}

#[test]
fn ne3_version_metadata_and_its_alias_refuse_depth_manifest() {
    for depth in Depth::ALL {
        let ws = Workspace::with(depth);
        let reason = format!("manifest-version-metadata: {}", depth.guard_reason());
        let metadata = ["--version", "0.2.0", "--status", "validated"];
        ws.assert_refuses(
            &[
                &["manifest-version-metadata", "--root", MANIFEST_ROOT][..],
                &metadata,
            ]
            .concat(),
            &reason,
        );
        // The historical alias defaults to another agent's root; the enclosing workspace's depth
        // still refuses it.
        ws.assert_refuses(
            &[&["codex-version-metadata"][..], &metadata].concat(),
            &reason,
        );
    }
}

#[test]
fn ne4_manifest_validate_fix_mode_and_its_alias_refuse_depth_manifest() {
    for depth in Depth::ALL {
        let ws = Workspace::with(depth);
        let reason = format!("manifest-validate fix: {}", depth.guard_reason());
        // The synthetic root lacks pointer files that fix mode would otherwise create.
        ws.assert_refuses(
            &[
                "manifest-validate",
                "--root",
                MANIFEST_ROOT,
                "--mode",
                "fix",
            ],
            &reason,
        );
        ws.assert_refuses(&["codex-validate", "--mode", "fix"], &reason);
    }
}

#[test]
fn ne5_retention_apply_and_its_alias_refuse_depth_manifest() {
    for depth in Depth::ALL {
        let ws = Workspace::with(depth);
        let reason = format!("manifest-retain: {}", depth.guard_reason());
        ws.assert_refuses(
            &["manifest-retain", "--root", MANIFEST_ROOT, "--apply"],
            &reason,
        );
        ws.assert_refuses(&["codex-retain", "--apply"], &reason);
    }
}

#[test]
fn ne6_legacy_snapshot_out_dir_refuses_depth_destination() {
    for depth in Depth::ALL {
        let ws = Workspace::with(depth);
        let reason = format!("legacy snapshot: {}", depth.guard_reason());
        // The binaries do not exist: the destination refusal precedes any upstream execution.
        for (command, binary_flag, out_dir) in [
            ("codex-snapshot", "--codex-binary", "cli_manifests/codex"),
            (
                "claude-snapshot",
                "--claude-binary",
                "cli_manifests/claude_code",
            ),
            (
                "opencode-snapshot",
                "--opencode-binary",
                "cli_manifests/opencode",
            ),
            ("codex-snapshot", "--codex-binary", MANIFEST_ROOT),
            // A destination that does not exist yet resolves through its nearest ancestor.
            (
                "opencode-snapshot",
                "--opencode-binary",
                "cli_manifests/depth_fixture/not-yet/created",
            ),
        ] {
            ws.assert_refuses(
                &[command, binary_flag, "absent-binary", "--out-dir", out_dir],
                &reason,
            );
        }
    }
}

#[test]
fn ne7_historical_lifecycle_backfill_refuses_depth_workspace() {
    for depth in Depth::ALL {
        let reason = match depth {
            Depth::Declared => LOADER.to_owned(),
            _ => format!("historical-lifecycle-backfill: {}", depth.guard_reason()),
        };
        let ws = Workspace::with(depth);
        ws.assert_refuses(&["historical-lifecycle-backfill", "--all"], &reason);
        ws.assert_refuses(
            &["historical-lifecycle-backfill", "--agent", "opencode"],
            &reason,
        );
    }
}

#[test]
fn ne8_runtime_advertising_model_write_refuses_depth_workspace() {
    for depth in Depth::ALL {
        let reason = format!("runtime-advertising-model: {}", depth.guard_reason());
        Workspace::with(depth).assert_refuses(&["runtime-advertising-model"], &reason);
    }
}
