//! Structural checks for exactly five pinned native contracts (no policy evaluation).
use serde_json::Value;
use std::sync::OnceLock;

const SCHEMAS: [(&str, &str, &str); 5] = [
    (
        "item/commandExecution/requestApproval",
        include_str!("interaction_schemas/CommandExecutionRequestApprovalParams.json"),
        include_str!("interaction_schemas/CommandExecutionRequestApprovalResponse.json"),
    ),
    (
        "item/fileChange/requestApproval",
        include_str!("interaction_schemas/FileChangeRequestApprovalParams.json"),
        include_str!("interaction_schemas/FileChangeRequestApprovalResponse.json"),
    ),
    (
        "item/permissions/requestApproval",
        include_str!("interaction_schemas/PermissionsRequestApprovalParams.json"),
        include_str!("interaction_schemas/PermissionsRequestApprovalResponse.json"),
    ),
    (
        "item/tool/requestUserInput",
        include_str!("interaction_schemas/ToolRequestUserInputParams.json"),
        include_str!("interaction_schemas/ToolRequestUserInputResponse.json"),
    ),
    (
        "mcpServer/elicitation/request",
        include_str!("interaction_schemas/McpServerElicitationRequestParams.json"),
        include_str!("interaction_schemas/McpServerElicitationRequestResponse.json"),
    ),
];

pub(super) fn valid(method: &str, value: &Value, response: bool) -> bool {
    static PARSED: OnceLock<Vec<(Value, Value)>> = OnceLock::new();
    let Some(index) = SCHEMAS.iter().position(|(m, _, _)| *m == method) else {
        return false;
    };
    let schemas = PARSED.get_or_init(|| {
        SCHEMAS
            .iter()
            .map(|(_, p, r)| {
                (
                    serde_json::from_str(p).expect("pinned schema"),
                    serde_json::from_str(r).expect("pinned schema"),
                )
            })
            .collect()
    });
    let root = if response {
        &schemas[index].1
    } else {
        &schemas[index].0
    };
    matches_schema(root, value, root, 0)
}

fn matches_schema(schema: &Value, value: &Value, root: &Value, depth: usize) -> bool {
    if depth > 64 {
        return false;
    }
    if let Some(allowed) = schema.as_bool() {
        return allowed;
    }
    let Some(object) = schema.as_object() else {
        return false;
    };
    // Unknown validation keywords fail closed. Annotation keywords do not change validation.
    if object.keys().any(|k| {
        !matches!(
            k.as_str(),
            "$schema"
                | "definitions"
                | "$ref"
                | "type"
                | "properties"
                | "required"
                | "additionalProperties"
                | "items"
                | "enum"
                | "anyOf"
                | "oneOf"
                | "allOf"
                | "minimum"
                | "maximum"
                | "format"
                | "description"
                | "title"
                | "default"
        )
    }) {
        return false;
    }
    let check = |s: &Value, v: &Value| matches_schema(s, v, root, depth + 1);
    if let Some(reference) = schema.get("$ref") {
        let Some(pointer) = reference.as_str().and_then(|r| r.strip_prefix('#')) else {
            return false;
        };
        if !root.pointer(pointer).is_some_and(|s| check(s, value)) {
            return false;
        }
    }
    for (keyword, mode) in [("anyOf", 0), ("oneOf", 1), ("allOf", 2)] {
        if let Some(branches) = schema.get(keyword).and_then(Value::as_array) {
            let count = branches.iter().filter(|s| check(s, value)).count();
            if (mode == 0 && count == 0)
                || (mode == 1 && count != 1)
                || (mode == 2 && count != branches.len())
            {
                return false;
            }
        }
    }
    if let Some(kind) = schema.get("type") {
        let is_type = |t: &str| match t {
            "null" => value.is_null(),
            "string" => value.is_string(),
            "boolean" => value.is_boolean(),
            "object" => value.is_object(),
            "array" => value.is_array(),
            "number" => value.is_number(),
            "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
            _ => false,
        };
        if !kind.as_str().map(is_type).unwrap_or_else(|| {
            kind.as_array()
                .is_some_and(|a| a.iter().any(|t| t.as_str().is_some_and(is_type)))
        }) {
            return false;
        }
    }
    if schema
        .get("enum")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.contains(value))
    {
        return false;
    }
    if let Some(n) = value.as_f64() {
        if schema
            .get("minimum")
            .and_then(Value::as_f64)
            .is_some_and(|m| n < m)
            || schema
                .get("maximum")
                .and_then(Value::as_f64)
                .is_some_and(|m| n > m)
        {
            return false;
        }
        if let Some(format) = schema.get("format").and_then(Value::as_str) {
            if !match format {
                "int64" => value.as_i64().is_some(),
                "uint32" => value.as_u64().is_some_and(|n| n <= u32::MAX.into()),
                "uint" | "uint64" => value.as_u64().is_some(),
                "double" => true,
                _ => false,
            } {
                return false;
            }
        }
    }
    if let Some(fields) = value.as_object() {
        if schema
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|r| {
                r.iter()
                    .any(|k| !k.as_str().is_some_and(|k| fields.contains_key(k)))
            })
        {
            return false;
        }
        for (name, v) in fields {
            if let Some(s) = schema.get("properties").and_then(|p| p.get(name)) {
                if !check(s, v) {
                    return false;
                }
            } else if let Some(s) = schema.get("additionalProperties") {
                if !check(s, v) {
                    return false;
                }
            }
        }
    }
    if let Some(items) = value.as_array() {
        if let Some(s) = schema.get("items") {
            if items.iter().any(|v| !check(s, v)) {
                return false;
            }
        }
    }
    true
}
