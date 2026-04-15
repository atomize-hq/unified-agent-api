use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use semver::Version;
use toml_edit::{value, DocumentMut, Item, Table, Value};

const DEP_SECTION_KEYS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

#[derive(Debug, Parser)]
pub struct Args {
    /// New workspace release version (semver).
    pub version: String,
    /// Workspace root. Defaults to the repository root.
    #[arg(long)]
    pub root: Option<PathBuf>,
}

pub fn run(args: Args) -> Result<(), String> {
    let version = Version::parse(&args.version)
        .map_err(|err| format!("invalid semver version `{}`: {err}", args.version))?;
    let new_version = version.to_string();
    let root = resolve_workspace_root(args.root.as_deref())?;
    let mut workspace = WorkspaceState::load(&root)?;
    workspace.apply(&new_version)?;
    workspace.validate(&new_version)?;
    workspace.write()?;
    println!(
        "Updated VERSION, workspace version, and release pins to {new_version} under {}.",
        root.display()
    );
    Ok(())
}

struct WorkspaceState {
    root: PathBuf,
    root_doc: DocumentMut,
    members: Vec<MemberManifest>,
}

struct MemberManifest {
    path: PathBuf,
    doc: DocumentMut,
    package_name: String,
    publishable: bool,
}

impl WorkspaceState {
    fn load(root: &Path) -> Result<Self, String> {
        let root_manifest_path = root.join("Cargo.toml");
        let root_doc = read_toml(&root_manifest_path)?;
        let member_paths = workspace_member_manifest_paths(root, &root_doc)?;
        let mut members = Vec::with_capacity(member_paths.len());

        for path in member_paths {
            let doc = read_toml(&path)?;
            let package_name = package_name(&doc)
                .ok_or_else(|| format!("missing [package].name in {}", path.display()))?;
            let publishable = is_publishable(&doc);
            members.push(MemberManifest {
                path,
                doc,
                package_name,
                publishable,
            });
        }

        Ok(Self {
            root: root.to_path_buf(),
            root_doc,
            members,
        })
    }

    fn apply(&mut self, new_version: &str) -> Result<(), String> {
        self.root_doc["workspace"]["package"]["version"] = value(new_version);
        let publishable_names: BTreeSet<String> = self
            .members
            .iter()
            .filter(|member| member.publishable)
            .map(|member| member.package_name.clone())
            .collect();

        for member in &mut self.members {
            bump_package_version(&mut member.doc, new_version)?;
            update_dependency_versions(
                member.doc.as_item_mut(),
                false,
                &publishable_names,
                new_version,
            )?;
        }

        Ok(())
    }

    fn validate(&self, new_version: &str) -> Result<(), String> {
        let root_version = self.root_doc["workspace"]["package"]["version"]
            .as_str()
            .ok_or_else(|| "workspace.package.version must remain a string".to_string())?;
        if root_version != new_version {
            return Err(format!(
                "workspace.package.version drifted to {root_version}, expected {new_version}"
            ));
        }

        let publishable_names: BTreeSet<String> = self
            .members
            .iter()
            .filter(|member| member.publishable)
            .map(|member| member.package_name.clone())
            .collect();

        for member in &self.members {
            let package_version = current_package_version(&member.doc, root_version)
                .ok_or_else(|| format!("missing package version in {}", member.path.display()))?;
            if package_version != new_version {
                return Err(format!(
                    "{} has package version {}, expected {}",
                    member.path.display(),
                    package_version,
                    new_version
                ));
            }

            validate_dependency_versions(
                member.doc.as_item(),
                false,
                &publishable_names,
                new_version,
                &member.path,
                member.publishable,
            )?;
        }

        Ok(())
    }

    fn write(&self) -> Result<(), String> {
        let root_manifest_path = self.root.join("Cargo.toml");
        let root_version = self.root_doc["workspace"]["package"]["version"]
            .as_str()
            .ok_or_else(|| "workspace.package.version must remain a string".to_string())?;
        write_toml(&root_manifest_path, &self.root_doc)?;
        fs::write(self.root.join("VERSION"), format!("{root_version}\n"))
            .map_err(|err| format!("write {}: {err}", self.root.join("VERSION").display()))?;

        for member in &self.members {
            write_toml(&member.path, &member.doc)?;
        }

        Ok(())
    }
}

fn resolve_workspace_root(root: Option<&Path>) -> Result<PathBuf, String> {
    let path = match root {
        Some(path) => path.to_path_buf(),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .ok_or_else(|| "failed to derive workspace root from xtask manifest dir".to_string())?
            .to_path_buf(),
    };
    Ok(path)
}

fn workspace_member_manifest_paths(
    root: &Path,
    root_doc: &DocumentMut,
) -> Result<Vec<PathBuf>, String> {
    let members = root_doc["workspace"]["members"]
        .as_array()
        .ok_or_else(|| "workspace.members must be an array".to_string())?;
    let mut paths = Vec::with_capacity(members.len());
    for member in members.iter() {
        let member = member
            .as_str()
            .ok_or_else(|| "workspace.members entries must be strings".to_string())?;
        paths.push(root.join(member).join("Cargo.toml"));
    }
    Ok(paths)
}

fn read_toml(path: &Path) -> Result<DocumentMut, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    text.parse::<DocumentMut>()
        .map_err(|err| format!("parse {}: {err}", path.display()))
}

fn write_toml(path: &Path, doc: &DocumentMut) -> Result<(), String> {
    let mut rendered = doc.to_string();
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    fs::write(path, rendered).map_err(|err| format!("write {}: {err}", path.display()))
}

fn package_name(doc: &DocumentMut) -> Option<String> {
    doc.get("package")
        .and_then(Item::as_table_like)
        .and_then(|package| package.get("name"))
        .and_then(Item::as_str)
        .map(ToOwned::to_owned)
}

fn is_publishable(doc: &DocumentMut) -> bool {
    let Some(publish) = doc
        .get("package")
        .and_then(Item::as_table_like)
        .and_then(|package| package.get("publish"))
    else {
        return true;
    };

    if publish.is_none() {
        return true;
    }
    if publish.as_bool() == Some(false) {
        return false;
    }
    publish
        .as_array()
        .map(|items| !items.is_empty())
        .unwrap_or(true)
}

fn bump_package_version(doc: &mut DocumentMut, new_version: &str) -> Result<(), String> {
    let package = doc["package"]
        .as_table_mut()
        .ok_or_else(|| "missing [package] table".to_string())?;
    match package.get("version") {
        Some(item) if item.as_str().is_some() => {
            package["version"] = value(new_version);
        }
        Some(item)
            if item
                .as_table_like()
                .and_then(|version| version.get("workspace"))
                .and_then(Item::as_bool)
                == Some(true) => {}
        Some(_) => {
            return Err("package.version must be a string or workspace reference".to_string())
        }
        None => {}
    }

    Ok(())
}

fn current_package_version(doc: &DocumentMut, root_version: &str) -> Option<String> {
    if doc
        .get("package")
        .and_then(Item::as_table_like)
        .and_then(|package| package.get("version"))
        .and_then(Item::as_table_like)
        .and_then(|version| version.get("workspace"))
        .and_then(Item::as_bool)
        == Some(true)
    {
        return Some(root_version.to_string());
    }

    doc.get("package")
        .and_then(Item::as_table_like)
        .and_then(|package| package.get("version"))
        .and_then(Item::as_str)
        .map(ToOwned::to_owned)
}

fn update_dependency_versions(
    item: &mut Item,
    in_dep_section: bool,
    publishable_names: &BTreeSet<String>,
    new_version: &str,
) -> Result<(), String> {
    match item {
        Item::Table(table) => update_dependency_versions_in_table(
            table,
            in_dep_section,
            publishable_names,
            new_version,
        ),
        Item::ArrayOfTables(array) => {
            for table in array.iter_mut() {
                update_dependency_versions_in_table(
                    table,
                    in_dep_section,
                    publishable_names,
                    new_version,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn update_dependency_versions_in_table(
    table: &mut Table,
    in_dep_section: bool,
    publishable_names: &BTreeSet<String>,
    new_version: &str,
) -> Result<(), String> {
    if in_dep_section {
        let keys: Vec<String> = table.iter().map(|(key, _)| key.to_string()).collect();
        for key in keys {
            if let Some(item) = table.get_mut(&key) {
                maybe_update_dependency_spec(&key, item, publishable_names, new_version)?;
            }
        }
        return Ok(());
    }

    let keys: Vec<String> = table.iter().map(|(key, _)| key.to_string()).collect();
    for key in keys {
        let next_in_dep_section = DEP_SECTION_KEYS.contains(&key.as_str());
        if let Some(item) = table.get_mut(&key) {
            update_dependency_versions(item, next_in_dep_section, publishable_names, new_version)?;
        }
    }

    Ok(())
}

fn maybe_update_dependency_spec(
    dep_key: &str,
    item: &mut Item,
    publishable_names: &BTreeSet<String>,
    new_version: &str,
) -> Result<(), String> {
    match item {
        Item::Value(Value::InlineTable(inline)) => {
            let package_name = inline
                .get("package")
                .and_then(Value::as_str)
                .unwrap_or(dep_key);
            if inline.contains_key("path")
                && inline.contains_key("version")
                && publishable_names.contains(package_name)
            {
                inline.insert("version", Value::from(format!("={new_version}")));
            }
            Ok(())
        }
        Item::Table(table) => {
            let package_name = table
                .get("package")
                .and_then(Item::as_str)
                .unwrap_or(dep_key);
            if table.contains_key("path")
                && table.contains_key("version")
                && publishable_names.contains(package_name)
            {
                table["version"] = value(format!("={new_version}"));
            }
            Ok(())
        }
        Item::None | Item::Value(_) => Ok(()),
        Item::ArrayOfTables(_) => Err(format!(
            "dependency entry `{dep_key}` must not be an array of tables"
        )),
    }
}

fn validate_dependency_versions(
    item: &Item,
    in_dep_section: bool,
    publishable_names: &BTreeSet<String>,
    new_version: &str,
    manifest_path: &Path,
    require_exact_pins: bool,
) -> Result<(), String> {
    match item {
        Item::Table(table) => validate_dependency_versions_in_table(
            table,
            in_dep_section,
            publishable_names,
            new_version,
            manifest_path,
            require_exact_pins,
        ),
        Item::ArrayOfTables(array) => {
            for table in array.iter() {
                validate_dependency_versions_in_table(
                    table,
                    in_dep_section,
                    publishable_names,
                    new_version,
                    manifest_path,
                    require_exact_pins,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_dependency_versions_in_table(
    table: &Table,
    in_dep_section: bool,
    publishable_names: &BTreeSet<String>,
    new_version: &str,
    manifest_path: &Path,
    require_exact_pins: bool,
) -> Result<(), String> {
    if in_dep_section {
        for (key, item) in table.iter() {
            validate_dependency_spec(
                key,
                item,
                publishable_names,
                new_version,
                manifest_path,
                require_exact_pins,
            )?;
        }
        return Ok(());
    }

    for (key, item) in table.iter() {
        validate_dependency_versions(
            item,
            DEP_SECTION_KEYS.contains(&key),
            publishable_names,
            new_version,
            manifest_path,
            require_exact_pins,
        )?;
    }

    Ok(())
}

fn validate_dependency_spec(
    dep_key: &str,
    item: &Item,
    publishable_names: &BTreeSet<String>,
    new_version: &str,
    manifest_path: &Path,
    require_exact_pins: bool,
) -> Result<(), String> {
    let expected = format!("={new_version}");
    match item {
        Item::Value(Value::InlineTable(inline)) => {
            let package_name = inline
                .get("package")
                .and_then(Value::as_str)
                .unwrap_or(dep_key);
            if inline.contains_key("path") && publishable_names.contains(package_name) {
                let Some(req) = inline.get("version").and_then(Value::as_str) else {
                    if require_exact_pins {
                        return Err(format!(
                            "{} -> {} is missing an exact version pin",
                            manifest_path.display(),
                            package_name
                        ));
                    }
                    return Ok(());
                };
                if req != expected {
                    return Err(format!(
                        "{} -> {} uses {}, expected {}",
                        manifest_path.display(),
                        package_name,
                        req,
                        expected
                    ));
                }
            }
            Ok(())
        }
        Item::Table(table) => {
            let package_name = table
                .get("package")
                .and_then(Item::as_str)
                .unwrap_or(dep_key);
            if table.contains_key("path") && publishable_names.contains(package_name) {
                let Some(req) = table.get("version").and_then(Item::as_str) else {
                    if require_exact_pins {
                        return Err(format!(
                            "{} -> {} is missing an exact version pin",
                            manifest_path.display(),
                            package_name
                        ));
                    }
                    return Ok(());
                };
                if req != expected {
                    return Err(format!(
                        "{} -> {} uses {}, expected {}",
                        manifest_path.display(),
                        package_name,
                        req,
                        expected
                    ));
                }
            }
            Ok(())
        }
        Item::None | Item::Value(_) => Ok(()),
        Item::ArrayOfTables(_) => Err(format!(
            "{} has invalid dependency entry {}",
            manifest_path.display(),
            dep_key
        )),
    }
}
