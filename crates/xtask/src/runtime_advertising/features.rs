use super::{projection::AGENTS, source::Source};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct Features {
    lists: BTreeMap<String, Vec<String>>,
}

impl Features {
    pub fn load(source: &Source) -> Result<Self, String> {
        let doc: Value = toml_edit::de::from_str(&source.read("crates/agent_api/Cargo.toml")?)
            .map_err(|e| e.to_string())?;
        if doc.get("target").is_some()
            || doc["package"].get("build").is_some()
            || doc["lib"].get("path").is_some()
            || source.optional("crates/agent_api/build.rs")?.is_some()
        {
            return Err("unsupported runtime Cargo/build selection".into());
        }
        for path in [".cargo/config", ".cargo/config.toml"] {
            if source.optional(path)?.is_some() {
                return Err("unsupported repository Cargo configuration".into());
            }
        }
        let root: Value =
            toml_edit::de::from_str(&source.read("Cargo.toml")?).map_err(|e| e.to_string())?;
        if root.get("patch").is_some() || root.get("replace").is_some() {
            return Err("unsupported workspace dependency replacement".into());
        }
        for table in ["dependencies", "build-dependencies", "dev-dependencies"] {
            if let Some(dependencies) = doc.get(table) {
                let dependencies = dependencies
                    .as_object()
                    .ok_or("invalid runtime dependency table")?;
                for (name, dependency) in dependencies {
                    if dependency["optional"] == true
                        && !AGENTS.contains(&name.as_str())
                        && !["tokio", "futures-util"].contains(&name.as_str())
                    {
                        return Err(format!("unsupported implicit runtime feature: {name}"));
                    }
                }
            }
        }
        let fields = doc["features"]
            .as_object()
            .ok_or("missing runtime feature table")?;
        let expected: BTreeSet<_> = AGENTS.into_iter().chain(["default"]).collect();
        if fields.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected {
            return Err("unsupported runtime feature keys or alias".into());
        }
        let mut lists = BTreeMap::new();
        for (key, value) in fields {
            let list = value
                .as_array()
                .ok_or("feature must be a list")?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or("feature item must be a string")
                })
                .collect::<Result<Vec<_>, _>>()?;
            for value in &list {
                if !AGENTS.contains(&value.as_str())
                    && !["dep:tokio", "dep:futures-util"].contains(&value.as_str())
                    && !AGENTS.iter().any(|a| value == &format!("dep:{a}"))
                {
                    return Err(format!("unsupported feature reference: {value}"));
                }
            }
            lists.insert(key.clone(), list);
        }
        for agent in AGENTS {
            let dep = &doc["dependencies"][agent];
            let package = format!("unified-agent-api-{}", agent.replace('_', "-"));
            if dep["optional"] != true
                || dep["path"] != format!("../{agent}")
                || dep["package"] != package
            {
                return Err(format!("{agent}: unsupported optional wrapper dependency"));
            }
            if !lists[agent].contains(&format!("dep:{agent}")) {
                return Err(format!("{agent}: wrapper dependency is not activated"));
            }
            for dependency in ["tokio", "futures-util"] {
                if !lists[agent].contains(&format!("dep:{dependency}"))
                    || doc["dependencies"][dependency]["optional"] != true
                {
                    return Err(format!(
                        "{agent}: unsupported shared runtime dependency activation"
                    ));
                }
            }
        }
        let this = Self { lists };
        for agent in AGENTS {
            this.walk(agent, &mut BTreeSet::new(), &mut BTreeSet::new())?;
        }
        Ok(this)
    }

    fn walk(
        &self,
        key: &str,
        visiting: &mut BTreeSet<String>,
        enabled: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        if !visiting.insert(key.into()) {
            return Err("cyclic runtime feature definitions".into());
        }
        if key != "default" {
            enabled.insert(key.into());
        }
        for item in &self.lists[key] {
            if !item.starts_with("dep:") {
                self.walk(item, visiting, enabled)?;
            }
        }
        visiting.remove(key);
        Ok(())
    }

    pub fn enabled(&self, requested: u8, defaults: bool) -> Result<BTreeSet<String>, String> {
        if requested >= 32 {
            return Err("unsupported feature request".into());
        }
        let mut enabled = BTreeSet::new();
        if defaults {
            self.walk("default", &mut BTreeSet::new(), &mut enabled)?;
        }
        for (i, agent) in AGENTS.iter().enumerate() {
            if requested & (1 << i) != 0 {
                self.walk(agent, &mut BTreeSet::new(), &mut enabled)?;
            }
        }
        Ok(enabled)
    }
}
