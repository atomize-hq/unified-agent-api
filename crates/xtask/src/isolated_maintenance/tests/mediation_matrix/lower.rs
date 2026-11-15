use super::*;
use crate::agent_lifecycle::{
    write_lifecycle_state, write_publication_ready_packet, LifecycleState, PublicationReadyPacket,
};

pub(super) fn carriers(record: &[u8], state: &LifecycleState, packet: &PublicationReadyPacket) {
    for carrier in [
        "declared",
        "omitted",
        "disabled",
        "orphaned",
        "removed_record",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut doc: toml_edit::DocumentMut = DECLARED.parse().unwrap();
        let entry = doc["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .get_mut(0)
            .unwrap();
        match carrier {
            "omitted" => {
                entry.remove("support_depth").unwrap();
            }
            "disabled" => {
                entry["support_depth"]
                    .as_table_mut()
                    .unwrap()
                    .remove("enabled_paths")
                    .unwrap();
            }
            "orphaned" => {
                doc = super::super::registry_fixture::undeclared_registry()
                    .parse()
                    .unwrap();
            }
            _ => {}
        }
        write(root, REGISTRY_RELATIVE_PATH, doc.to_string());
        if carrier != "removed_record" {
            write(root, RECORD, record);
        }
        write(root, "nested/inner/sentinel", b"preserve bytes\n");
        std::os::unix::fs::symlink("sentinel", root.join("nested/inner/link")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            root.join("nested/inner/sentinel"),
            fs::Permissions::from_mode(0o751),
        )
        .unwrap();
        let reason = match carrier {
            "omitted" | "orphaned" => "unsupported depth write (retained record)",
            _ => "unsupported depth write (declared enrollment)",
        };
        for caller in [root.to_owned(), root.join("nested/inner")] {
            unchanged(root, &format!("neutral mutation writer: {reason}"), || {
                apply_mutations(
                    &caller,
                    &[PlannedMutation::create(
                        "ordinary/other-agent/0.3.0/output",
                        b"accepting output".to_vec(),
                    )],
                )
                .unwrap_err()
                .to_string()
            });
            unchanged(root, &format!("lifecycle writer: {reason}"), || {
                write_lifecycle_state(&caller, STATE, state)
                    .unwrap_err()
                    .to_string()
            });
            unchanged(
                root,
                &format!("publication packet writer: {reason}"),
                || {
                    write_publication_ready_packet(&caller, PACKET, packet)
                        .unwrap_err()
                        .to_string()
                },
            );
        }
    }
}

pub(super) fn proposed(state: &LifecycleState, packet: &PublicationReadyPacket, record: &[u8]) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        REGISTRY_RELATIVE_PATH,
        super::super::registry_fixture::undeclared_registry(),
    );
    write(root, "nested/sentinel", "ordinary caller\n");
    let nested_record = "nested/cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
    for path in [RECORD, nested_record] {
        unchanged(
            root,
            "neutral mutation writer: unsupported depth write (proposed depth record)",
            || {
                apply_mutations(
                    root,
                    &[
                        PlannedMutation::create("ordinary-first", b"must remain absent".to_vec()),
                        PlannedMutation::create(path, record.to_vec()),
                    ],
                )
                .unwrap_err()
                .to_string()
            },
        );
        unchanged(
            root,
            "lifecycle writer: unsupported depth write (proposed depth record)",
            || {
                write_lifecycle_state(root, path, state)
                    .unwrap_err()
                    .to_string()
            },
        );
        unchanged(
            root,
            "publication packet writer: unsupported depth write (proposed depth record)",
            || {
                write_publication_ready_packet(root, path, packet)
                    .unwrap_err()
                    .to_string()
            },
        );
    }
    for path in [
        REGISTRY_RELATIVE_PATH.to_owned(),
        format!("nested/{REGISTRY_RELATIVE_PATH}"),
    ] {
        unchanged(
            root,
            "neutral mutation writer: unsupported depth write (declared enrollment)",
            || {
                let mutation = if path == REGISTRY_RELATIVE_PATH {
                    PlannedMutation::replace(
                        &path,
                        fs::read(root.join(&path)).unwrap(),
                        DECLARED.as_bytes().to_vec(),
                    )
                } else {
                    PlannedMutation::create(&path, DECLARED.as_bytes().to_vec())
                };
                apply_mutations(root, &[mutation]).unwrap_err().to_string()
            },
        );
    }
    // An ordinary workspace and ordinary output remain writable through all three boundaries.
    apply_mutations(root, &[PlannedMutation::create("ordinary", b"ok".to_vec())]).unwrap();
    write_lifecycle_state(root, STATE, state).unwrap();
    write_publication_ready_packet(root, PACKET, packet).unwrap();
    assert_eq!(fs::read(root.join("ordinary")).unwrap(), b"ok");
}
