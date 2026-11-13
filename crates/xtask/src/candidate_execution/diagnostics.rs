//! Successful host diagnostics refer only to paths retained by delivery. The original
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
    mappings.sort_by_key(|(source, _)| std::cmp::Reverse(source.len()));
    let holder = candidate
        .root
        .parent()
        .expect("candidate holder")
        .to_string_lossy();
    let json_holder = serde_json::to_string(holder.as_ref())
        .map_err(|e| Error::Internal(format!("candidate diagnostic path: {e}")))?;
    for path in diagnostic_paths {
        let Some(Node::File(data, _)) = delivery.get_mut(path) else {
            continue;
        };
        let text = String::from_utf8_lossy(data);
        if !text.contains(holder.as_ref()) && !text.contains(&json_holder[1..json_holder.len() - 1])
        {
            continue;
        }
        if path.ends_with(".json") {
            let mut value: serde_json::Value = serde_json::from_slice(data)
                .map_err(|e| Error::Internal(format!("host diagnostic {path}: {e}")))?;
            rebase_value(&mut value, &holder, &mappings);
            let trailing_newline = data.ends_with(b"\n");
            *data = serde_json::to_vec_pretty(&value)
                .map_err(|e| Error::Internal(format!("host diagnostic {path}: {e}")))?;
            if trailing_newline {
                data.push(b'\n');
            }
        } else {
            *data = rebase_text(&text, &holder, &mappings).into_bytes();
        }
    }
    Ok(())
}

fn rebase_value(value: &mut serde_json::Value, holder: &str, mappings: &[(String, String)]) {
    match value {
        serde_json::Value::String(text) => {
            if text.starts_with(holder) && !text.chars().any(char::is_control) {
                *text = mappings
                    .iter()
                    .find(|(source, _)| source == text)
                    .map(|(_, target)| target.clone())
                    .unwrap_or_else(|| "[not retained]".into());
            } else {
                *text = rebase_text(text, holder, mappings);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                rebase_value(value, holder, mappings);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                rebase_value(value, holder, mappings);
            }
        }
        _ => {}
    }
}

fn path_boundary(character: char) -> bool {
    character.is_whitespace() || "\"'`<>()[]{},;:".contains(character)
}

fn rebase_text(text: &str, holder: &str, mappings: &[(String, String)]) -> String {
    let mut remaining = text;
    let mut output = String::new();
    while let Some(start) = remaining.find(holder) {
        output.push_str(&remaining[..start]);
        remaining = &remaining[start..];
        if let Some((source, target)) = mappings.iter().find(|(source, _)| {
            remaining
                .strip_prefix(source)
                .is_some_and(|tail| tail.chars().next().map_or(true, path_boundary))
        }) {
            output.push_str(target);
            remaining = &remaining[source.len()..];
        } else {
            output.push_str("[not retained]");
            let end = remaining[holder.len()..]
                .find(path_boundary)
                .map_or(remaining.len(), |end| holder.len() + end);
            remaining = &remaining[end..];
        }
    }
    output.push_str(remaining);
    output
}

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
