use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use regex::Regex;
use toml_edit::{DocumentMut, Item, Value};

const AGENT_API_MANIFEST_PATH: &str = "crates/agent_api/Cargo.toml";
const AGENT_API_SRC_PATH: &str = "crates/agent_api/src";

#[derive(Debug, Parser)]
pub struct Args {}

pub fn run(_args: Args) -> Result<(), String> {
    let workspace_root = resolve_workspace_root()?;
    run_for_root(&workspace_root)
}

fn run_for_root(workspace_root: &Path) -> Result<(), String> {
    let manifest_path = workspace_root.join(AGENT_API_MANIFEST_PATH);
    let manifest = read_toml(&manifest_path)?;
    let backend_ids = derive_backend_ids(&manifest)?;
    if backend_ids.is_empty() {
        return Err(format!(
            "{} does not declare any optional local backend dependencies",
            manifest_path.display()
        ));
    }

    let source_root = workspace_root.join(AGENT_API_SRC_PATH);
    let signature_pattern = build_signature_regex(&backend_ids)?;
    let reexport_pattern = build_reexport_regex(&backend_ids)?;

    let mut violations = Vec::new();
    scan_dir(
        &source_root,
        &signature_pattern,
        &reexport_pattern,
        &mut violations,
    )?;

    if violations.is_empty() {
        return Ok(());
    }

    Err(render_report(&backend_ids, &violations))
}

fn resolve_workspace_root() -> Result<PathBuf, String> {
    let current_dir = std::env::current_dir().map_err(|err| format!("current_dir: {err}"))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    ))
}

fn read_toml(path: &Path) -> Result<DocumentMut, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    text.parse::<DocumentMut>()
        .map_err(|err| format!("parse {}: {err}", path.display()))
}

fn derive_backend_ids(doc: &DocumentMut) -> Result<Vec<String>, String> {
    let feature_backends = declared_backend_features(doc)?;
    let dependencies = doc
        .get("dependencies")
        .and_then(Item::as_table_like)
        .ok_or_else(|| "agent_api manifest is missing [dependencies]".to_string())?;

    let mut backend_ids = Vec::new();
    for (dep_key, item) in dependencies.iter() {
        if !feature_backends.contains(dep_key) {
            continue;
        }
        if dependency_is_optional_local_backend(item)? {
            backend_ids.push(dep_key.to_string());
        }
    }

    backend_ids.sort();
    backend_ids.dedup();
    Ok(backend_ids)
}

fn declared_backend_features(doc: &DocumentMut) -> Result<BTreeSet<String>, String> {
    let features = doc
        .get("features")
        .and_then(Item::as_table_like)
        .ok_or_else(|| "agent_api manifest is missing [features]".to_string())?;
    let mut backend_features = BTreeSet::new();

    for (feature_name, item) in features.iter() {
        let Some(entries) = item.as_array() else {
            continue;
        };
        if entries
            .iter()
            .filter_map(Value::as_str)
            .any(|entry| entry == format!("dep:{feature_name}"))
        {
            backend_features.insert(feature_name.to_string());
        }
    }

    Ok(backend_features)
}

fn dependency_is_optional_local_backend(item: &Item) -> Result<bool, String> {
    match item {
        Item::Value(Value::InlineTable(inline)) => {
            let is_optional = inline.get("optional").and_then(Value::as_bool) == Some(true);
            let is_local = inline
                .get("path")
                .and_then(Value::as_str)
                .map(is_sibling_backend_path)
                .unwrap_or(false);
            Ok(is_optional && is_local)
        }
        Item::Table(table) => {
            let is_optional = table.get("optional").and_then(Item::as_bool) == Some(true);
            let is_local = table
                .get("path")
                .and_then(Item::as_str)
                .map(is_sibling_backend_path)
                .unwrap_or(false);
            Ok(is_optional && is_local)
        }
        Item::None | Item::Value(_) => Ok(false),
        Item::ArrayOfTables(_) => {
            Err("backend dependency entries must be table or inline table values".to_string())
        }
    }
}

fn is_sibling_backend_path(path: &str) -> bool {
    path.starts_with("../") && !path[3..].contains("../")
}

fn build_signature_regex(backend_ids: &[String]) -> Result<Regex, String> {
    let alternation = backend_alternation(backend_ids);
    Regex::new(&format!(
        r"(?m)pub\s+(?:fn|struct|enum|trait|type|const|static)\b[^\{{;]*\b(?:{alternation})::"
    ))
    .map_err(|err| format!("build pub signature regex: {err}"))
}

fn build_reexport_regex(backend_ids: &[String]) -> Result<Regex, String> {
    let alternation = backend_alternation(backend_ids);
    Regex::new(&format!(r"(?m)pub\s+use\b[^;]*\b(?:{alternation})::"))
        .map_err(|err| format!("build pub re-export regex: {err}"))
}

fn backend_alternation(backend_ids: &[String]) -> String {
    backend_ids
        .iter()
        .map(|id| regex::escape(id))
        .collect::<Vec<_>>()
        .join("|")
}

fn scan_dir(
    dir: &Path,
    signature_pattern: &Regex,
    reexport_pattern: &Regex,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|err| format!("read_dir {}: {err}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("read_dir entry {}: {err}", dir.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| format!("read file type {}: {err}", path.display()))?;
        if file_type.is_dir() {
            scan_dir(&path, signature_pattern, reexport_pattern, violations)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let contents =
            fs::read_to_string(&path).map_err(|err| format!("read {}: {err}", path.display()))?;
        violations.extend(scan_file(
            &path,
            &contents,
            "pub signature",
            signature_pattern,
        ));
        violations.extend(scan_file(
            &path,
            &contents,
            "pub re-export",
            reexport_pattern,
        ));
    }

    Ok(())
}

fn scan_file(path: &Path, contents: &str, rule: &'static str, pattern: &Regex) -> Vec<Violation> {
    pattern
        .find_iter(contents)
        .flat_map(|matched| lines_for_match(path, contents, matched.start(), matched.end(), rule))
        .collect()
}

fn lines_for_match(
    path: &Path,
    contents: &str,
    start: usize,
    end: usize,
    rule: &'static str,
) -> Vec<Violation> {
    let match_start_line = line_number_at_offset(contents, start);
    let match_end_line = line_number_at_offset(contents, end.saturating_sub(1));
    let mut violations = Vec::new();

    for (line_idx, line) in contents.lines().enumerate() {
        let line_number = line_idx + 1;
        if line_number < match_start_line || line_number > match_end_line {
            continue;
        }
        if !line.contains("::") {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        violations.push(Violation {
            rule,
            path: path.to_path_buf(),
            line_number,
            line: trimmed.to_string(),
        });
    }

    if violations.is_empty() {
        let line_number = match_start_line;
        let line = contents[start..]
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        violations.push(Violation {
            rule,
            path: path.to_path_buf(),
            line_number,
            line,
        });
    }

    violations
}

fn line_number_at_offset(contents: &str, offset: usize) -> usize {
    1 + contents[..offset.min(contents.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
}

fn render_report(backend_ids: &[String], violations: &[Violation]) -> String {
    let mut out = String::new();
    out.push_str("agent-api-backend-type-leak-guard failed\n");
    out.push_str(&format!(
        "derived backend ids: [{}]\n",
        backend_ids.join(", ")
    ));
    for violation in violations {
        out.push_str(&format!(
            "- {}: {}:{}: {}\n",
            violation.rule,
            violation.path.display(),
            violation.line_number,
            violation.line
        ));
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    rule: &'static str,
    path: PathBuf,
    line_number: usize,
    line: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_manifest(contents: &str) -> DocumentMut {
        contents.parse::<DocumentMut>().expect("valid manifest")
    }

    #[test]
    fn derives_backend_ids_from_optional_local_dependencies_with_matching_features() {
        let manifest = parse_manifest(
            r#"[features]
default = []
opencode = ["dep:opencode"]
codex = ["dep:codex"]
claude_code = ["dep:claude_code"]

[dependencies]
serde = "1"
opencode = { path = "../opencode", optional = true }
codex = { path = "../codex", optional = true }
claude_code = { path = "../claude_code", optional = true }
"#,
        );

        let backend_ids = derive_backend_ids(&manifest).expect("derive backend ids");
        assert_eq!(backend_ids, vec!["claude_code", "codex", "opencode"]);
    }

    #[test]
    fn excludes_non_optional_non_local_and_featureless_dependencies() {
        let manifest = parse_manifest(
            r#"[features]
default = []
codex = ["dep:codex"]
helper = ["dep:helper"]

[dependencies]
codex = { path = "../codex", optional = true }
helper = { path = "../helper", optional = false }
remote = { version = "1.0", optional = true }
missing_feature = { path = "../missing_feature", optional = true }
nested = { path = "../../not-a-sibling", optional = true }
"#,
        );

        let backend_ids = derive_backend_ids(&manifest).expect("derive backend ids");
        assert_eq!(backend_ids, vec!["codex"]);
    }

    #[test]
    fn detects_pub_signature_and_reexport_leaks() {
        let backend_ids = vec!["codex".to_string(), "opencode".to_string()];
        let signature_pattern = build_signature_regex(&backend_ids).expect("signature regex");
        let reexport_pattern = build_reexport_regex(&backend_ids).expect("reexport regex");
        let contents = r#"
pub fn leaked() -> opencode::Client {
    todo!()
}

pub use codex::Thing;
"#;

        let signature_matches = scan_file(
            Path::new("crates/agent_api/src/lib.rs"),
            contents,
            "pub signature",
            &signature_pattern,
        );
        let reexport_matches = scan_file(
            Path::new("crates/agent_api/src/lib.rs"),
            contents,
            "pub re-export",
            &reexport_pattern,
        );

        assert_eq!(signature_matches.len(), 1);
        assert_eq!(signature_matches[0].line_number, 2);
        assert!(signature_matches[0].line.contains("opencode::Client"));

        assert_eq!(reexport_matches.len(), 1);
        assert_eq!(reexport_matches[0].line_number, 6);
        assert!(reexport_matches[0].line.contains("codex::Thing"));
    }

    #[test]
    fn ignores_private_backend_references() {
        let backend_ids = vec!["codex".to_string(), "opencode".to_string()];
        let signature_pattern = build_signature_regex(&backend_ids).expect("signature regex");
        let reexport_pattern = build_reexport_regex(&backend_ids).expect("reexport regex");
        let contents = r#"
fn private_helper() -> opencode::Client {
    todo!()
}

pub(crate) use codex::Thing;
"#;

        assert!(scan_file(
            Path::new("crates/agent_api/src/lib.rs"),
            contents,
            "pub signature",
            &signature_pattern,
        )
        .is_empty());
        assert!(scan_file(
            Path::new("crates/agent_api/src/lib.rs"),
            contents,
            "pub re-export",
            &reexport_pattern,
        )
        .is_empty());
    }
}
