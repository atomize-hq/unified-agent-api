//! Successful host path fields refer only to paths retained by delivery. The original
//! candidate evidence remains available for every failure, including a refused delivery.
use super::{candidate::Candidate, delivery_index, tree::Node, tree::Tree, Error};
use std::{collections::BTreeSet, path::Path};

pub(super) fn rebase(
    candidate: &Candidate,
    caller: &Path,
    delivery: &mut Tree,
    diagnostic_paths: &[String],
) -> Result<(), Error> {
    let bound = delivery_index::evidence_paths(&candidate.before, delivery);
    let mut retained = BTreeSet::new();
    for (path, node) in candidate.before.iter().chain(delivery.iter()) {
        if !matches!(node, Node::File(..)) {
            continue;
        }
        let diagnostic = path.starts_with(&format!("{}/", candidate.run_dir_rel));
        let protected = candidate.protects_delivery_path(path);
        if !candidate.before.contains_key(path) && (protected || diagnostic && bound.contains(path))
        {
            continue;
        }
        // Parent directories are retained with their files, including the caller root.
        retained.extend(Path::new(path).ancestors().map(Path::to_path_buf));
    }
    let mut mappings = retained
        .into_iter()
        .filter(|path| !path.as_os_str().is_empty())
        .map(|path| {
            (
                candidate.root.join(&path).to_string_lossy().into_owned(),
                caller.join(&path).to_string_lossy().into_owned(),
            )
        })
        .collect::<Vec<_>>();
    mappings.push((
        candidate.root.to_string_lossy().into_owned(),
        caller.to_string_lossy().into_owned(),
    ));
    let holder = candidate
        .root
        .parent()
        .expect("candidate holder")
        .to_string_lossy();
    for path in diagnostic_paths {
        // These host schemas are the complete set of candidate-absolute path fields.
        // Written paths, captured streams, prompts and executor-authored files keep their bytes.
        let name = Path::new(path).file_name().and_then(|name| name.to_str());
        if !matches!(
            name,
            Some("codex-execution.json" | "validation-report.json" | "run-status.json")
        ) {
            continue;
        }
        let Some(Node::File(data, _)) = delivery.get_mut(path) else {
            continue;
        };
        let mut value: serde_json::Value = serde_json::from_slice(data)
            .map_err(|e| Error::Internal(format!("host diagnostic {path}: {e}")))?;
        let original = value.clone();
        match name {
            Some("codex-execution.json") => rebase_execution(&mut value, &holder, &mappings),
            Some("validation-report.json") => {
                if let Some(execution) = value.get_mut("codex_execution") {
                    rebase_execution(execution, &holder, &mappings);
                }
                if let Some(preflight) = value.get_mut("preflight") {
                    rebase_execution(preflight, &holder, &mappings);
                }
            }
            Some("run-status.json") => {
                if let Some(path) = value.get_mut("run_dir") {
                    rebase_path(path, &holder, &mappings);
                }
            }
            _ => unreachable!(),
        }
        if value != original {
            let trailing_newline = data.ends_with(b"\n");
            *data = serde_json::to_vec_pretty(&value)
                .map_err(|e| Error::Internal(format!("host diagnostic {path}: {e}")))?;
            if trailing_newline {
                data.push(b'\n');
            }
        }
    }
    Ok(())
}

fn rebase_execution(value: &mut serde_json::Value, holder: &str, mappings: &[(String, String)]) {
    for field in ["prompt_path", "stdout_path", "stderr_path", "binary"] {
        if let Some(path) = value.get_mut(field) {
            rebase_path(path, holder, mappings);
        }
    }
    // Only --cd's operand is a host-generated path. Other arguments are opaque.
    if let Some(argv) = value
        .get_mut("argv")
        .and_then(serde_json::Value::as_array_mut)
    {
        for index in 1..argv.len() {
            if argv[index - 1] == "--cd" {
                rebase_path(&mut argv[index], holder, mappings);
            }
        }
    }
}

fn rebase_path(value: &mut serde_json::Value, holder: &str, mappings: &[(String, String)]) {
    let serde_json::Value::String(path) = value else {
        return;
    };
    // Compare exact host-produced strings. Path component iteration would normalize them.
    if path != holder
        && !path
            .strip_prefix(holder)
            .is_some_and(|tail| tail.starts_with(std::path::MAIN_SEPARATOR))
    {
        return;
    }
    *path = mappings
        .iter()
        .find(|(source, _)| source == path)
        .map(|(_, target)| target.clone())
        .unwrap_or_else(|| "[not retained]".into());
}

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
