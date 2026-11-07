//! Raw help is bounded to the pinned fixture's complete producer output, never candidate text.
use super::*;

mod fixture {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../depth_fixture/src/bin/depth_fixture/help.rs"
    ));
}

pub(super) fn paths(adapter: &Adapter) -> Vec<String> {
    adapter
        .scaffold
        .canonical_targets
        .iter()
        .flat_map(|target| {
            ["help.txt", "run.help.txt"].map(|name| {
                format!(
                    "{}/raw_help/{}/{target}/{name}",
                    adapter.scaffold.manifest_root, adapter.version
                )
            })
        })
        .collect()
}

pub(super) fn validate(source: &Source, adapter: &Adapter) -> Result<(), String> {
    let prefix = format!("{}/raw_help", adapter.scaffold.manifest_root);
    let present: Vec<_> = source
        .paths(&prefix)?
        .into_iter()
        .filter(|p| p.starts_with(&format!("{prefix}/")))
        .collect();
    let allowed = paths(adapter);
    for path in &present {
        if !allowed.contains(path) && !allowed.iter().any(|p| p.starts_with(&format!("{path}/"))) {
            return Err(format!("foreign bootstrap raw help: {path}"));
        }
    }
    let temporary = tempfile::tempdir().map_err(|e| e.to_string())?;
    for target in &adapter.scaffold.canonical_targets {
        let dir = format!("{prefix}/{}/{target}", adapter.version);
        if !present.iter().any(|p| p.starts_with(&format!("{dir}/"))) {
            continue;
        }
        // Reuse the actual producer's writer with the same help constants compiled into the
        // upstream fixture. No candidate binary, pathname, or text supplies expected output.
        crate::depth_fixture_snapshot::write_raw_help(
            &temporary.path().join(&dir),
            fixture::ROOT_HELP,
            fixture::RUN_HELP,
        )
        .map_err(|e| e.to_string())?;
        for name in ["help.txt", "run.help.txt"] {
            let path = format!("{dir}/{name}");
            let expected = fs::read(temporary.path().join(&path)).map_err(|e| e.to_string())?;
            if source.optional_bytes(&path)?.as_ref() != Some(&expected) {
                return Err(format!(
                    "bootstrap raw help differs from snapshot producer: {path}"
                ));
            }
        }
        let shard = format!(
            "{}/snapshots/{}/{target}.json",
            adapter.scaffold.manifest_root, adapter.version
        );
        if source.optional_bytes(&shard)?.is_none() {
            return Err("bootstrap raw help requires its producer snapshot".into());
        }
    }
    Ok(())
}
