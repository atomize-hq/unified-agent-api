use super::*;

const LOADER: &str = "carries `support_depth`, which no command enforces yet";

#[test]
fn every_ne_command_and_alias_refuses_onboarding_carriers() {
    for carrier in Carrier::ALL {
        let ws = workspace(carrier);
        let reason = carrier.reason();
        ws.assert_refuses(&["support-matrix"], &if carrier.loader_refuses() {
            LOADER.to_owned()
        } else if matches!(carrier, Carrier::Omitted) {
            "holds a depth record, and the support matrix does not publish agent `depth_fixture`".to_owned()
        } else { format!("support-matrix writer: {reason}") });
        ws.assert_refuses(
            &["capability-matrix"],
            &format!("capability-matrix: {reason}"),
        );
        ws.assert_refuses(
            &["capability-matrix", "--out", "nested/inner/new/matrix.md"],
            &format!("capability-matrix: {reason}"),
        );
        for command in ["manifest-version-metadata", "codex-version-metadata"] {
            for root in [None, Some(MANIFEST_ROOT), Some("nested/inner")] {
                if root.is_none() && command == "manifest-version-metadata" {
                    continue;
                }
                let mut args = vec![
                    command,
                    "--version",
                    "0.3.0",
                    "--status",
                    "validated",
                    "--passed-target",
                    "x86_64-unknown-linux-gnu",
                ];
                if let Some(root) = root {
                    args.extend(["--root", root]);
                }
                ws.assert_refuses(
                    &args,
                    &format!(
                        "manifest-version-metadata: {}",
                        carrier.manifest_reason(root.unwrap_or("cli_manifests/codex"))
                    ),
                );
            }
        }
        for command in ["manifest-validate", "codex-validate"] {
            for root in [None, Some(MANIFEST_ROOT), Some("nested/inner")] {
                if root.is_none() && command == "manifest-validate" {
                    continue;
                }
                let mut args = vec![command, "--mode", "fix"];
                if let Some(root) = root {
                    args.extend(["--root", root]);
                }
                ws.assert_refuses(
                    &args,
                    &format!(
                        "manifest-validate fix: {}",
                        carrier.manifest_reason(root.unwrap_or("cli_manifests/codex"))
                    ),
                );
            }
        }
        for command in ["manifest-retain", "codex-retain"] {
            for root in [None, Some(MANIFEST_ROOT), Some("nested/inner")] {
                if root.is_none() && command == "manifest-retain" {
                    continue;
                }
                let mut args = vec![command, "--apply"];
                if let Some(root) = root {
                    args.extend(["--root", root]);
                }
                ws.assert_refuses(
                    &args,
                    &format!(
                        "manifest-retain: {}",
                        carrier.manifest_reason(root.unwrap_or("cli_manifests/codex"))
                    ),
                );
            }
        }
        for (command, binary) in [
            ("codex-snapshot", "--codex-binary"),
            ("claude-snapshot", "--claude-binary"),
            ("opencode-snapshot", "--opencode-binary"),
        ] {
            for destination in [MANIFEST_ROOT, "nested/inner/not-yet/created"] {
                ws.assert_refuses(
                    &[command, binary, "absent-binary", "--out-dir", destination],
                    &format!("legacy snapshot: {}", carrier.manifest_reason(destination)),
                );
            }
        }
        for args in [
            vec!["historical-lifecycle-backfill", "--all"],
            vec!["historical-lifecycle-backfill", "--agent", "opencode"],
        ] {
            ws.assert_refuses(
                &args,
                &if carrier.loader_refuses() {
                    LOADER.to_owned()
                } else {
                    format!("historical-lifecycle-backfill: {reason}")
                },
            );
        }
        ws.assert_refuses(
            &["runtime-advertising-model"],
            &format!("runtime-advertising-model: {reason}"),
        );
    }
}
