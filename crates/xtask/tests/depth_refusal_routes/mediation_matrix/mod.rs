//! E.8 executes actual commands against onboarding carriers, independently of request scope.
use super::support::{self, write, Workspace, MANIFEST_ROOT, RECORD, REGISTRY};
use std::fs;

mod destinations;
#[path = "../../support/onboarding_guard_record.rs"]
mod guard_record;
mod neutral;
mod onboarding;

#[derive(Clone, Copy, Debug)]
enum Carrier {
    Declared,
    Omitted,
    Disabled,
    Orphaned,
    RemovedRecord,
}

impl Carrier {
    const ALL: [Self; 5] = [
        Self::Declared,
        Self::Omitted,
        Self::Disabled,
        Self::Orphaned,
        Self::RemovedRecord,
    ];

    fn reason(self) -> &'static str {
        match self {
            Self::Omitted | Self::Orphaned => "unsupported depth write (retained record)",
            _ => "unsupported depth write (declared enrollment)",
        }
    }

    fn loader_refuses(self) -> bool {
        matches!(self, Self::Declared | Self::Disabled | Self::RemovedRecord)
    }

    fn manifest_reason(self, root: &str) -> &'static str {
        if root == MANIFEST_ROOT && !matches!(self, Self::RemovedRecord) {
            "unsupported depth write (retained record)"
        } else {
            self.reason()
        }
    }
}

fn workspace(carrier: Carrier) -> Workspace {
    let ws = Workspace::plain();
    let template =
        include_str!("../../fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml");
    let raw = format!("{}\n{template}", support::ordinary_registry());
    let mut doc: toml_edit::DocumentMut = raw.parse().unwrap();
    let entries = doc["agents"].as_array_of_tables_mut().unwrap();
    let position = entries
        .iter()
        .position(|a| a["agent_id"].as_str() == Some("depth_fixture"))
        .unwrap();
    let entry = entries.get_mut(position).unwrap();
    // Neutral rendering uses ordinary rows. The actual connected library leg separately
    // requires and checks the subject's depth-bearing reporting row and all eleven results.
    entry["publication"]["support_matrix_enabled"] = toml_edit::value(false);
    match carrier {
        Carrier::Omitted => {
            entry.remove("support_depth").unwrap();
        }
        Carrier::Disabled => {
            entry["support_depth"]
                .as_table_mut()
                .unwrap()
                .remove("enabled_paths")
                .unwrap();
        }
        Carrier::Orphaned => {
            entries.remove(position);
        }
        _ => {}
    }
    let raw = doc.to_string();
    xtask::agent_registry::AgentRegistry::parse_unenforced(&raw).unwrap();
    write(ws.root(), REGISTRY, raw);
    fs::create_dir_all(ws.root().join(MANIFEST_ROOT)).unwrap();
    for destination in [MANIFEST_ROOT, "nested/inner"] {
        for file in ["RULES.json", "SCHEMA.json", "VERSION_METADATA_SCHEMA.json"] {
            let source = support::repo()
                .join("crates/xtask/tests/fixtures/depth-synthetic/governing/manifest-root")
                .join(file);
            write(
                ws.root(),
                &format!("{destination}/{file}"),
                fs::read(source).unwrap(),
            );
        }
    }
    if !matches!(carrier, Carrier::RemovedRecord) {
        let record = guard_record::record();
        let parsed = xtask::depth_record::DepthRecord::parse(
            std::str::from_utf8(&record).unwrap(),
            "depth_fixture",
            "0.2.0",
        )
        .unwrap();
        assert!(matches!(
            parsed.event,
            xtask::depth_record::Event::Onboarding { .. }
        ));
        write(ws.root(), RECORD, record);
    }
    write(ws.root(), "nested/inner/Cargo.toml", "[workspace]\n");
    write(
        ws.root(),
        "nested/inner/sentinel",
        "preserve mode and bytes\n",
    );
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(
        ws.root().join("nested/inner/sentinel"),
        fs::Permissions::from_mode(0o751),
    )
    .unwrap();
    std::os::unix::fs::symlink("sentinel", ws.root().join("nested/inner/link")).unwrap();
    ws
}
