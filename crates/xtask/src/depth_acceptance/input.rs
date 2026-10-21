use crate::{
    agent_registry::{AgentRegistry, AgentRegistryEntry, REGISTRY_RELATIVE_PATH},
    depth_record::DepthRecord,
    runtime_advertising::source::Source,
};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::Value;
use std::collections::BTreeMap;

/// Unlike Value's default reader, this rejects duplicate JSON object members before projection.
pub fn json(raw: &str) -> Result<Value, String> {
    struct Unique(Value);
    impl<'de> Deserialize<'de> for Unique {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            struct V;
            impl<'de> Visitor<'de> for V {
                type Value = Unique;
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("JSON without duplicate members")
                }
                fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<Unique, M::Error> {
                    let mut values = serde_json::Map::new();
                    while let Some((k, v)) = m.next_entry::<String, Unique>()? {
                        if values.insert(k.clone(), v.0).is_some() {
                            return Err(de::Error::custom(format!("duplicate JSON member {k}")));
                        }
                    }
                    Ok(Unique(Value::Object(values)))
                }
                fn visit_seq<M: SeqAccess<'de>>(self, mut m: M) -> Result<Unique, M::Error> {
                    let mut a = Vec::new();
                    while let Some(v) = m.next_element::<Unique>()? {
                        a.push(v.0);
                    }
                    Ok(Unique(Value::Array(a)))
                }
                fn visit_bool<E: de::Error>(self, v: bool) -> Result<Unique, E> {
                    Ok(Unique(v.into()))
                }
                fn visit_str<E: de::Error>(self, v: &str) -> Result<Unique, E> {
                    Ok(Unique(v.into()))
                }
                fn visit_i64<E: de::Error>(self, v: i64) -> Result<Unique, E> {
                    Ok(Unique(v.into()))
                }
                fn visit_u64<E: de::Error>(self, v: u64) -> Result<Unique, E> {
                    Ok(Unique(v.into()))
                }
                fn visit_f64<E: de::Error>(self, v: f64) -> Result<Unique, E> {
                    serde_json::Number::from_f64(v)
                        .map(|n| Unique(Value::Number(n)))
                        .ok_or_else(|| E::custom("invalid number"))
                }
                fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
                    Ok(Unique(Value::Null))
                }
            }
            d.deserialize_any(V)
        }
    }
    serde_json::from_str::<Unique>(raw)
        .map(|v| v.0)
        .map_err(|e| e.to_string())
}
pub fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("missing/invalid {key}"))
}
pub fn strings(value: &Value) -> Result<Vec<String>, String> {
    let values = value.as_array().ok_or("expected string array")?;
    let mut result = Vec::new();
    for v in values {
        let s = v.as_str().ok_or("expected string")?.to_owned();
        if result.contains(&s) {
            return Err(format!("duplicate value {s}"));
        }
        result.push(s);
    }
    Ok(result)
}

pub struct View {
    pub source: Source,
    pub registry: AgentRegistry,
    pub entry: AgentRegistryEntry,
    pub records: BTreeMap<String, DepthRecord>,
}
impl View {
    pub fn load(source: Source, agent: &str) -> Result<Self, String> {
        let registry = AgentRegistry::parse(&source.read(REGISTRY_RELATIVE_PATH)?)
            .map_err(|e| e.to_string())?;
        let entry = registry
            .find(agent)
            .ok_or("record lost its registry anchor")?
            .clone();
        let prefix = format!("{}/reports/", entry.manifest_root);
        let mut records = BTreeMap::new();
        for path in source.paths(prefix.trim_end_matches('/'))? {
            let Some(relative) = path.strip_prefix(&prefix) else {
                continue;
            };
            let parts: Vec<_> = relative.split('/').collect();
            if parts.len() == 2 && parts[1] == "depth-record.json" {
                let raw = source.read(&path)?;
                json(&raw)?;
                records.insert(parts[0].into(), DepthRecord::parse(&raw, agent, parts[0])?);
            }
        }
        Ok(Self {
            source,
            registry,
            entry,
            records,
        })
    }
    pub fn optional_json(&self, path: &str) -> Result<Option<Value>, String> {
        self.source
            .optional(path)?
            .map(|raw| json(&raw).map_err(|e| format!("{path}: {e}")))
            .transpose()
    }
    pub fn json(&self, path: &str) -> Result<Value, String> {
        json(&self.source.read(path)?).map_err(|e| format!("{path}: {e}"))
    }
    pub fn covered(&self, version: &str, target: Option<&str>) -> bool {
        self.records.get(version).is_some_and(|r| {
            !crate::support_matrix::depth::covered_targets(r).is_empty()
                && target.map_or(true, |t| {
                    crate::support_matrix::depth::covered_targets(r).contains(t)
                })
        })
    }
}
