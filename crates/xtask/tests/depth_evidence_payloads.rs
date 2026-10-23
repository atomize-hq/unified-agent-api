use serde_json::{json, Value};
use xtask::depth_evidence::payload::{DependencyGraph, Environment};

fn graph() -> Value {
    json!({"schema_version":1, "roots":["local"], "packages":[
        {"id":"local", "origin":{"kind":"local", "directory":"crates/demo"}, "dependencies":["external"]},
        {"id":"external", "origin":{"kind":"registry", "source":"registry+https://example.invalid", "checksum":"a".repeat(64)}, "dependencies":[]}
    ], "inputs":[{"path":"Cargo.lock", "sha256":"a".repeat(64)}], "scopes":["crates/demo"]})
}
fn environment() -> Value {
    json!({"schema_version":1, "target":"x86_64-unknown-linux-gnu", "features":["demo"],
        "variables":[{"name":"RUSTFLAGS", "value":""}]})
}
#[test]
fn payload_forms_preserve_incomplete_claims_without_verifying_them() {
    assert!(DependencyGraph::parse(&graph().to_string()).is_ok());
    assert!(Environment::parse(&environment().to_string()).is_ok());
    let mut empty = graph();
    empty["roots"] = json!([]);
    empty["packages"] = json!([]);
    empty["inputs"] = json!([]);
    assert!(DependencyGraph::parse(&empty.to_string()).is_ok());
}
#[test]
fn payloads_reject_unknown_null_missing_positional_and_duplicate_members() {
    fn objects(v: &Value, path: String, out: &mut Vec<String>) {
        match v {
            Value::Object(map) => {
                out.push(path.clone());
                for (k, v) in map {
                    objects(v, format!("{path}/{k}"), out);
                }
            }
            Value::Array(items) => {
                for (i, v) in items.iter().enumerate() {
                    objects(v, format!("{path}/{i}"), out);
                }
            }
            _ => {}
        }
    }
    for (base, is_graph) in [(graph(), true), (environment(), false)] {
        let parse = |s: &str| {
            if is_graph {
                DependencyGraph::parse(s).map(|_| ())
            } else {
                Environment::parse(s).map(|_| ())
            }
        };
        let mut paths = vec![];
        objects(&base, String::new(), &mut paths);
        for path in paths {
            for mutation in ["unknown", "null", "missing", "positional"] {
                let mut v = base.clone();
                let obj = v.pointer_mut(&path).unwrap();
                let key = obj
                    .as_object()
                    .unwrap()
                    .keys()
                    .find(|k| *k != "kind")
                    .unwrap()
                    .clone();
                match mutation {
                    "unknown" => {
                        obj.as_object_mut()
                            .unwrap()
                            .insert("unexpected".into(), json!(true));
                    }
                    "null" => {
                        obj[&key] = Value::Null;
                    }
                    "missing" => {
                        obj.as_object_mut().unwrap().remove(&key);
                    }
                    _ => {
                        *obj = Value::Array(obj.as_object().unwrap().values().cloned().collect());
                    }
                }
                assert!(parse(&v.to_string()).is_err(), "{path} {mutation}");
            }
        }
        let raw = base.to_string().replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        );
        assert!(parse(&raw).is_err());
        assert!(parse(&format!("{}{}", base, " ".repeat(1_048_576))).is_err());
    }
}
#[test]
fn graph_identity_and_environment_bounds_are_strict() {
    for (pointer, bad) in [
        ("/scopes/0", json!("../escape")),
        ("/packages/0/origin/directory", json!(".git")),
        ("/packages/1/origin/checksum", json!("X".repeat(64))),
        ("/inputs/0/path", json!("a//b")),
        (
            "/packages/1/dependencies",
            json!(["duplicate", "duplicate"]),
        ),
    ] {
        let mut v = graph();
        *v.pointer_mut(pointer).unwrap() = bad;
        assert!(DependencyGraph::parse(&v.to_string()).is_err(), "{pointer}");
    }
    let mut v = graph();
    v["packages"][1]["origin"] =
        json!({"kind":"git", "source":"https://example.invalid/repo", "commit":"a".repeat(40)});
    assert!(DependencyGraph::parse(&v.to_string()).is_ok());
    v["packages"][1]["origin"]["commit"] = json!("a".repeat(39));
    assert!(DependencyGraph::parse(&v.to_string()).is_err());
    let mut v = environment();
    v["variables"][0]["value"] = json!("a\0b");
    assert!(Environment::parse(&v.to_string()).is_err());
    v = environment();
    v["variables"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"RUSTFLAGS","value":"other"}));
    assert!(Environment::parse(&v.to_string()).is_err());
}
