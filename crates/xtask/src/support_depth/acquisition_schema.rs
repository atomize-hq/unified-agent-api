//! The acquisition planner's offline schema compiler and validator, shared by checkpoints.
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;

pub(crate) fn compile(bytes: &[u8]) -> Result<JSONSchema, String> {
    let mut schema: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    local_schema(&schema, true)?;
    schema
        .as_object_mut()
        .ok_or("schema must be an object")?
        .remove("$id");
    JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .map_err(|e| e.to_string())
}

pub(crate) fn validate(schema: &JSONSchema, path: &str, value: &Value) -> Result<(), String> {
    if let Err(errors) = schema.validate(value) {
        return Err(format!(
            "{path}: invalid acquisition schema: {}",
            errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
        ));
    }
    Ok(())
}

// Compilation is offline and consumes only this bound schema. External policy/schema inputs
// cannot be fetched invisibly; supporting them later requires binding each referenced file.
fn local_schema(value: &Value, root: bool) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "$ref" && !value.as_str().is_some_and(|r| r.starts_with("#/$defs/")) {
                    return Err("external schema references are unsupported".into());
                }
                if (key == "$id" && !root)
                    || matches!(key.as_str(), "$dynamicRef" | "$recursiveRef")
                {
                    return Err(
                        "nested schema identities/dynamic references are unsupported".into(),
                    );
                }
                local_schema(value, false)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                local_schema(item, false)?;
            }
        }
        _ => {}
    }
    Ok(())
}
