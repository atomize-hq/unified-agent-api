//! Fixed private session chain; successful SDK outcomes are not evidence admission.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, io::Read, path::Path};
const SELECTORS: [&str; 5] = [
    "initial-json",
    "resume-last-json",
    "resume-id-json",
    "fork-last-stdio",
    "fork-id-stdio",
];
#[derive(Default)]
struct ChainState {
    next: usize,
    seed: Option<String>,
    seen: BTreeSet<String>,
    after: Option<String>,
}
pub(super) fn valid_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit() && !c.is_ascii_uppercase()
            }
        })
        && id != "11111111-1111-4111-8111-111111111111"
}
impl ChainState {
    fn check_before(&self, selector: &str, before: &str) -> Result<(), String> {
        if SELECTORS.get(self.next) != Some(&selector)
            || self.after.as_deref().is_some_and(|old| old != before)
        {
            return Err("wrong order or changed private state before invocation".into());
        }
        Ok(())
    }

    fn accept(
        &mut self,
        selector: &str,
        success: bool,
        id: Option<&str>,
        before: &str,
        after: &str,
    ) -> Result<(), String> {
        self.check_before(selector, before)?;
        if !success {
            return Err("failed seed/stage, wrong order or changed private state".into());
        }
        let id = id
            .filter(|id| valid_id(id))
            .ok_or("missing or invalid native session identity")?;
        if (self.next == 1 || self.next == 2) && self.seed.as_deref() != Some(id) {
            return Err("resume changed the seed identity".into());
        }
        if self.next >= 3 && self.seen.contains(id) {
            return Err("fork did not create a distinct native identity".into());
        }
        if self.next == 0 {
            self.seed = Some(id.into());
        }
        self.seen.insert(id.into());
        self.next += 1;
        self.after = Some(after.into());
        Ok(())
    }
}

fn manifest(root: &Path) -> Result<Value, String> {
    fn visit(
        root: &Path,
        dir: &Path,
        rows: &mut Vec<Value>,
        total: &mut u64,
    ) -> Result<(), String> {
        let mut paths = fs::read_dir(dir)
            .map_err(|e| e.to_string())?
            .map(|r| r.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        paths.sort();
        for path in paths {
            let name = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("non-UTF8 fixture path")?
                .to_string();
            if name == "trace" {
                continue;
            }
            let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            let mode = {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o7777
            };
            #[cfg(not(unix))]
            let mode = 0;
            let mut row = json!({"path":name,"mode":mode});
            if metadata.is_symlink() {
                row["kind"] = json!("symlink");
                row["target"] = json!(fs::read_link(&path)
                    .map_err(|e| e.to_string())?
                    .to_str()
                    .ok_or("non-UTF8 symlink")?);
            } else if metadata.is_dir() {
                row["kind"] = json!("directory");
            } else if metadata.is_file() {
                if metadata.len() > 64 * 1024 * 1024 - *total {
                    return Err("fixture artifact size limit".into());
                }
                let mut file = fs::File::open(&path).map_err(|e| e.to_string())?;
                let mut hash = Sha256::new();
                let mut chunk = [0; 65536];
                let mut read = 0;
                loop {
                    let n = file.read(&mut chunk).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    *total += n as u64;
                    read += n as u64;
                    if *total > 64 * 1024 * 1024 {
                        return Err("fixture artifact grew beyond size limit".into());
                    }
                    hash.update(&chunk[..n]);
                }
                if read != metadata.len()
                    || file.metadata().map_err(|e| e.to_string())?.len() != read
                {
                    return Err("fixture artifact changed while hashing".into());
                }
                row["kind"] = json!("file");
                row["sha256"] = json!(format!("{:x}", hash.finalize()));
            } else {
                return Err("unsupported fixture artifact type".into());
            }
            if rows.len() >= 4096 {
                return Err("fixture artifact count limit".into());
            }
            rows.push(row);
            if metadata.is_dir() {
                visit(root, &path, rows, total)?;
            }
        }
        Ok(())
    }
    let mut rows = Vec::new();
    visit(root, root, &mut rows, &mut 0)?;
    Ok(json!(rows))
}
fn identity(value: &Value) -> Result<String, String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).map_err(|e| e.to_string())?)
    ))
}

fn expected_config() -> Result<String, String> {
    let baseline: Value = serde_json::from_str(include_str!(
        "../../../../cli_manifests/codex/reports/0.156.1/pilot-runtime-context.json"
    ))
    .map_err(|e| e.to_string())?;
    baseline["expected_config_toml"]
        .as_str()
        .map(String::from)
        .ok_or("missing config pin".into())
}
fn context(root: &Path, entries: &Value) -> Result<(), String> {
    let config = root.join("home/config.toml");
    if !fs::symlink_metadata(&config)
        .map_err(|e| e.to_string())?
        .is_file()
    {
        return Err("config is not an owned file".into());
    }
    let before = expected_config()?;
    let after = fs::read(&config).map_err(|e| e.to_string())?;
    let trusted = format!("{before}\n[projects.\"/pilot/work\"]\ntrust_level = \"trusted\"\n");
    if after != before.as_bytes() && after != trusted.as_bytes() {
        return Err("unapproved config drift".into());
    }
    for row in entries.as_array().ok_or("missing private manifest")? {
        let name = row["path"].as_str().ok_or("missing artifact path")?;
        let project_loader = name.starts_with("work/")
            && name
                .split('/')
                .any(|p| matches!(p, ".codex" | "AGENTS.md" | "AGENTS.override.md"));
        let home_loader = [
            "AGENTS.md",
            "AGENTS.override.md",
            ".codex",
            "home/.codex",
            "home/AGENTS.md",
            "home/AGENTS.override.md",
            "home/hooks",
            "home/plugins",
            "home/mcp",
            "home/config.d",
            "home/auth.json",
        ]
        .iter()
        .any(|p| name == *p || name.starts_with(&format!("{p}/")));
        let foreign_skill = name.starts_with("home/skills/")
            && name != "home/skills/.system"
            && !name.starts_with("home/skills/.system/");
        if project_loader || home_loader || foreign_skill {
            return Err("foreign private loader or credential input".into());
        }
    }
    Ok(())
}

pub(super) async fn run(variant: &str) -> Result<Value, String> {
    // Validate before any native request, including for unknown safety variants.
    super::extensions(SELECTORS[0], variant, "unused")?;
    let mut state = ChainState::default();
    let mut stages = Vec::new();
    let mut stopped = None;
    for selector in SELECTORS {
        if let Some(reason) = &stopped {
            stages.push(json!({"selector":selector,"executed":false,"reason":reason}));
            continue;
        }
        let before = manifest(Path::new("/pilot"))?;
        let before_sha = identity(&before)?;
        if let Err(reason) = context(Path::new("/pilot"), &before)
            .and_then(|()| state.check_before(selector, &before_sha))
        {
            stages.push(json!({"selector":selector,"executed":false,"reason":reason}));
            stopped = Some(reason);
            continue;
        }
        let source = state.seed.clone();
        let result = super::observe(
            selector,
            variant,
            source.as_deref().unwrap_or("unused"),
            false,
            std::time::Duration::from_secs(30),
        )
        .await?;
        let after = manifest(Path::new("/pilot"))?;
        let after_sha = identity(&after)?;
        let accepted = state.accept(
            selector,
            result["status_success"] == true
                && result["session_identity_consistent"] == true
                && result["expected_reply"] == true,
            result["session_id"].as_str(),
            &before_sha,
            &after_sha,
        );
        if let Err(reason) = accepted {
            stopped = Some(reason);
        }
        stages.push(
            json!({"selector":selector,"executed":true,"source_session_id":source,
            "before":before,"after":after,"before_sha256":before_sha,"after_sha256":after_sha,
            "adapter":result,"state_refusal":stopped}),
        );
    }
    Ok(
        json!({"selector":"session-chain","variant":variant,"stages":stages,
        "chain_completed":stopped.is_none(),"execution_verified":false,"live_execution_authorized":false}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    const SEED: &str = "aaaaaaaa-1111-4111-8111-111111111111";
    const FORK1: &str = "bbbbbbbb-1111-4111-8111-111111111111";
    const FORK2: &str = "cccccccc-1111-4111-8111-111111111111";
    #[test]
    fn private_manifest_and_context_refuse_loader_or_config_drift_before_execution() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("home")).unwrap();
        fs::create_dir_all(root.join("work")).unwrap();
        let config = expected_config().unwrap();
        fs::write(root.join("home/config.toml"), &config).unwrap();
        let before = manifest(root).unwrap();
        context(root, &before).unwrap();
        let mut state = ChainState::default();
        state
            .accept(
                SELECTORS[0],
                true,
                Some(SEED),
                "initial",
                &identity(&before).unwrap(),
            )
            .unwrap();
        fs::write(root.join("work/AGENTS.md"), "foreign instructions").unwrap();
        let changed = manifest(root).unwrap();
        assert!(context(root, &changed).is_err());
        assert!(state
            .check_before(SELECTORS[1], &identity(&changed).unwrap())
            .is_err());
        fs::remove_file(root.join("work/AGENTS.md")).unwrap();
        fs::write(
            root.join("home/config.toml"),
            format!("{config}\nextra = true\n"),
        )
        .unwrap();
        assert!(context(root, &manifest(root).unwrap()).is_err());
    }
    #[test]
    fn content_changes_in_session_artifacts_invalidate_continuation() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::write(root.join("session.jsonl"), "original").unwrap();
        let original = identity(&manifest(root).unwrap()).unwrap();
        fs::write(root.join("session.jsonl"), "modified").unwrap();
        assert_ne!(original, identity(&manifest(root).unwrap()).unwrap());
        assert!(!valid_id("11111111-1111-4111-8111-111111111111"));
    }
    #[test]
    fn seed_and_continuations_preserve_identity_and_exact_state_chain() {
        let mut state = ChainState::default();
        for (i, selector) in SELECTORS.into_iter().enumerate() {
            let id = if i < 3 {
                SEED
            } else if i == 3 {
                FORK1
            } else {
                FORK2
            };
            state
                .accept(
                    selector,
                    true,
                    Some(id),
                    &format!("state-{i}"),
                    &format!("state-{}", i + 1),
                )
                .unwrap();
        }
    }
    #[test]
    fn failed_or_missing_seed_never_admits_continuations() {
        for (success, id) in [(false, Some(SEED)), (true, None), (true, Some("foreign"))] {
            assert!(ChainState::default()
                .accept(SELECTORS[0], success, id, "before", "after")
                .is_err());
        }
        assert!(ChainState::default()
            .accept(SELECTORS[1], true, Some(SEED), "before", "after")
            .is_err());
    }
    #[test]
    fn changed_state_or_resume_identity_and_reused_fork_identity_refuse() {
        let mut state = ChainState::default();
        state
            .accept(SELECTORS[0], true, Some(SEED), "a", "b")
            .unwrap();
        assert!(state
            .accept(SELECTORS[1], true, Some(SEED), "changed", "c")
            .is_err());
        assert!(state
            .accept(SELECTORS[1], true, Some(FORK1), "b", "c")
            .is_err());
        state
            .accept(SELECTORS[1], true, Some(SEED), "b", "c")
            .unwrap();
        state
            .accept(SELECTORS[2], true, Some(SEED), "c", "d")
            .unwrap();
        assert!(state
            .accept(SELECTORS[3], true, Some(SEED), "d", "e")
            .is_err());
        state
            .accept(SELECTORS[3], true, Some(FORK1), "d", "e")
            .unwrap();
        assert!(state
            .accept(SELECTORS[4], true, Some(FORK1), "e", "f")
            .is_err());
    }
}
