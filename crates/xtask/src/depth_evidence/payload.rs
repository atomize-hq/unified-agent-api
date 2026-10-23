//! Referenced payload forms; their completeness still requires independent producer resolution.
use super::validate::{file, files, git_identity, path, sha256, strings, text, unique};
use crate::depth_record::FileRef;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyGraph {
    pub schema_version: u64,
    pub roots: Vec<String>,
    pub packages: Vec<Package>,
    pub inputs: Vec<FileRef>,
    pub scopes: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub id: String,
    pub origin: PackageOrigin,
    pub dependencies: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PackageOrigin {
    Local { directory: String },
    Registry { source: String, checksum: String },
    Git { source: String, commit: String },
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub schema_version: u64,
    pub target: String,
    pub features: Vec<String>,
    pub variables: Vec<Variable>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Variable {
    pub name: String,
    pub value: String,
}

pub(super) fn parse<T: DeserializeOwned + Serialize>(raw: &str) -> Result<T, String> {
    if raw.len() > 1_048_576 {
        return Err("evidence payload exceeds 1048576 bytes".into());
    }
    let typed = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    super::check_shape(
        &serde_json::from_str(raw).map_err(|e| e.to_string())?,
        &serde_json::to_value(&typed).map_err(|e| e.to_string())?,
    )?;
    Ok(typed)
}
impl DependencyGraph {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let graph: Self = parse(raw)?;
        if graph.schema_version != 1 {
            return Err("unsupported dependency graph schema_version".into());
        }
        strings(&graph.roots)?;
        files(&graph.inputs)?;
        strings(&graph.scopes)?;
        for scope in &graph.scopes {
            path(scope)?;
        }
        unique(&graph.packages, |p| &p.id)?;
        for p in &graph.packages {
            text(&p.id)?;
            strings(&p.dependencies)?;
            match &p.origin {
                PackageOrigin::Local { directory } => path(directory)?,
                PackageOrigin::Registry { source, checksum } => {
                    text(source)?;
                    sha256("package checksum", checksum)?;
                }
                PackageOrigin::Git { source, commit } => {
                    text(source)?;
                    git_identity(commit)?;
                }
            }
        }
        Ok(graph)
    }

    /// Form-valid empty or disconnected claims have no complete affected build closure.
    pub(super) fn closure(&self) -> Result<(), String> {
        let packages: BTreeMap<_, _> = self.packages.iter().map(|p| (&p.id, p)).collect();
        if self.roots.is_empty() || self.inputs.is_empty() {
            return Err("missing affected build closure".into());
        }
        let mut todo: Vec<_> = self.roots.iter().collect();
        let mut seen = BTreeSet::new();
        while let Some(id) = todo.pop() {
            let package = packages.get(id).ok_or("unresolved package dependency")?;
            if seen.insert(id) {
                todo.extend(&package.dependencies);
            }
        }
        if seen.len() != packages.len() {
            return Err("dependency graph has disconnected packages".into());
        }
        for package in &self.packages {
            if let PackageOrigin::Local { directory } = &package.origin {
                if !self.scopes.contains(directory)
                    || !self
                        .inputs
                        .iter()
                        .any(|f| f.path == format!("{directory}/Cargo.toml"))
                {
                    return Err("local package lacks complete input scope/manifest".into());
                }
            }
        }
        for required in ["Cargo.toml", "Cargo.lock"] {
            if !self.inputs.iter().any(|f| f.path == required) {
                return Err(format!("build graph lacks {required}"));
            }
        }
        // Conflicting identities are rejected by files(); a single file can have many roles.
        for input in &self.inputs {
            file(input)?;
        }
        Ok(())
    }
}
impl Environment {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let environment: Self = parse(raw)?;
        if environment.schema_version != 1 {
            return Err("unsupported environment schema_version".into());
        }
        text(&environment.target)?;
        strings(&environment.features)?;
        unique(&environment.variables, |v| &v.name)?;
        for variable in &environment.variables {
            text(&variable.name)?;
            // An explicitly set empty environment value is meaningful.
            if variable.value.len() > 4096 || variable.value.contains('\0') {
                return Err("invalid environment variable value".into());
            }
        }
        Ok(environment)
    }
}
