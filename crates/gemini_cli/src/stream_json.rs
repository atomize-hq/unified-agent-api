use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::GeminiCliError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GeminiStreamJsonRunRequest {
    prompt: String,
    model: Option<String>,
    working_dir: Option<PathBuf>,
}

impl GeminiStreamJsonRunRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            working_dir: None,
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn working_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(path.into());
        self
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn model_name(&self) -> Option<&str> {
        self.model.as_deref()
    }

    pub fn working_directory(&self) -> Option<&Path> {
        self.working_dir.as_deref()
    }

    pub(crate) fn argv(&self) -> Result<Vec<OsString>, GeminiCliError> {
        if self.prompt.trim().is_empty() {
            return Err(GeminiCliError::InvalidRequest(
                "prompt must not be empty".to_string(),
            ));
        }

        let mut argv = vec![
            OsString::from("--prompt"),
            OsString::from(self.prompt.as_str()),
            OsString::from("--output-format"),
            OsString::from("stream-json"),
        ];

        if let Some(model) = normalize_non_empty(self.model.as_deref()) {
            argv.push(OsString::from("--model"));
            argv.push(OsString::from(model));
        }

        Ok(argv)
    }
}

fn normalize_non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeminiToolResultError {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GeminiStreamJsonResultPayload {
    pub status: String,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub stats: Option<Value>,
    pub raw: Value,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GeminiStreamJsonEvent {
    Init {
        session_id: String,
        model: String,
        raw: Value,
    },
    Message {
        role: String,
        content: String,
        delta: bool,
        raw: Value,
    },
    ToolUse {
        tool_name: String,
        tool_id: String,
        parameters: Value,
        raw: Value,
    },
    ToolResult {
        tool_id: String,
        status: String,
        output: Option<String>,
        error: Option<GeminiToolResultError>,
        raw: Value,
    },
    Error {
        severity: String,
        message: String,
        raw: Value,
    },
    Result {
        payload: GeminiStreamJsonResultPayload,
    },
    Unknown {
        event_type: String,
        raw: Value,
    },
}

impl GeminiStreamJsonEvent {
    pub fn raw(&self) -> &Value {
        match self {
            GeminiStreamJsonEvent::Init { raw, .. } => raw,
            GeminiStreamJsonEvent::Message { raw, .. } => raw,
            GeminiStreamJsonEvent::ToolUse { raw, .. } => raw,
            GeminiStreamJsonEvent::ToolResult { raw, .. } => raw,
            GeminiStreamJsonEvent::Error { raw, .. } => raw,
            GeminiStreamJsonEvent::Result { payload } => &payload.raw,
            GeminiStreamJsonEvent::Unknown { raw, .. } => raw,
        }
    }

    pub fn event_type(&self) -> &str {
        match self {
            GeminiStreamJsonEvent::Init { .. } => "init",
            GeminiStreamJsonEvent::Message { .. } => "message",
            GeminiStreamJsonEvent::ToolUse { .. } => "tool_use",
            GeminiStreamJsonEvent::ToolResult { .. } => "tool_result",
            GeminiStreamJsonEvent::Error { .. } => "error",
            GeminiStreamJsonEvent::Result { .. } => "result",
            GeminiStreamJsonEvent::Unknown { event_type, .. } => event_type.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GeminiStreamJsonErrorCode {
    JsonParse,
    TypedParse,
}

#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
#[error("{message}")]
pub struct GeminiStreamJsonError {
    pub code: GeminiStreamJsonErrorCode,
    pub message: String,
    pub details: String,
}

impl GeminiStreamJsonError {
    fn new(code: GeminiStreamJsonErrorCode, message: String) -> Self {
        Self {
            code,
            details: message.clone(),
            message,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GeminiStreamJsonParser;

impl GeminiStreamJsonParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_line(
        &mut self,
        line: &str,
    ) -> Result<Option<GeminiStreamJsonEvent>, GeminiStreamJsonError> {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.chars().all(|ch| ch.is_whitespace()) {
            return Ok(None);
        }

        let value: Value = serde_json::from_str(line).map_err(|err| {
            GeminiStreamJsonError::new(
                GeminiStreamJsonErrorCode::JsonParse,
                format!("invalid JSON: {err}"),
            )
        })?;

        self.parse_json(&value)
    }

    pub fn parse_json(
        &mut self,
        value: &Value,
    ) -> Result<Option<GeminiStreamJsonEvent>, GeminiStreamJsonError> {
        let obj = value.as_object().ok_or_else(|| {
            GeminiStreamJsonError::new(
                GeminiStreamJsonErrorCode::TypedParse,
                "expected JSON object".to_string(),
            )
        })?;

        let event_type = get_required_str(obj, "type").map_err(|message| {
            GeminiStreamJsonError::new(GeminiStreamJsonErrorCode::TypedParse, message)
        })?;

        let event = match event_type.as_str() {
            "init" => GeminiStreamJsonEvent::Init {
                session_id: get_required_str(obj, "session_id").map_err(typed_parse_error)?,
                model: get_required_str(obj, "model").map_err(typed_parse_error)?,
                raw: value.clone(),
            },
            "message" => GeminiStreamJsonEvent::Message {
                role: get_required_str(obj, "role").map_err(typed_parse_error)?,
                content: get_required_str(obj, "content").map_err(typed_parse_error)?,
                delta: obj.get("delta").and_then(Value::as_bool).unwrap_or(false),
                raw: value.clone(),
            },
            "tool_use" => GeminiStreamJsonEvent::ToolUse {
                tool_name: get_required_str(obj, "tool_name").map_err(typed_parse_error)?,
                tool_id: get_required_str(obj, "tool_id").map_err(typed_parse_error)?,
                parameters: obj
                    .get("parameters")
                    .cloned()
                    .ok_or_else(|| typed_parse_error("expected field `parameters`".to_string()))?,
                raw: value.clone(),
            },
            "tool_result" => GeminiStreamJsonEvent::ToolResult {
                tool_id: get_required_str(obj, "tool_id").map_err(typed_parse_error)?,
                status: get_required_str(obj, "status").map_err(typed_parse_error)?,
                output: obj
                    .get("output")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                error: obj
                    .get("error")
                    .cloned()
                    .map(serde_json::from_value)
                    .transpose()
                    .map_err(|_| typed_parse_error("expected object field `error`".to_string()))?,
                raw: value.clone(),
            },
            "error" => GeminiStreamJsonEvent::Error {
                severity: get_required_str(obj, "severity").map_err(typed_parse_error)?,
                message: get_required_str(obj, "message").map_err(typed_parse_error)?,
                raw: value.clone(),
            },
            "result" => GeminiStreamJsonEvent::Result {
                payload: GeminiStreamJsonResultPayload {
                    status: get_required_str(obj, "status").map_err(typed_parse_error)?,
                    error_type: obj
                        .get("error")
                        .and_then(Value::as_object)
                        .and_then(|error| error.get("type"))
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    error_message: obj
                        .get("error")
                        .and_then(Value::as_object)
                        .and_then(|error| error.get("message"))
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    stats: obj.get("stats").cloned(),
                    raw: value.clone(),
                },
            },
            _ => GeminiStreamJsonEvent::Unknown {
                event_type,
                raw: value.clone(),
            },
        };

        Ok(Some(event))
    }
}

fn typed_parse_error(message: String) -> GeminiStreamJsonError {
    GeminiStreamJsonError::new(GeminiStreamJsonErrorCode::TypedParse, message)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GeminiStreamJsonLine {
    pub line_number: usize,
    pub raw: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GeminiStreamJsonLineOutcome {
    Ok {
        line: GeminiStreamJsonLine,
        event: GeminiStreamJsonEvent,
    },
    Err {
        line: GeminiStreamJsonLine,
        error: GeminiStreamJsonError,
    },
}

pub fn parse_stream_json_lines(input: &str) -> Vec<GeminiStreamJsonLineOutcome> {
    let mut parser = GeminiStreamJsonParser::new();
    let mut outcomes = Vec::new();

    for (index, raw) in input.lines().enumerate() {
        let line = GeminiStreamJsonLine {
            line_number: index + 1,
            raw: raw.to_string(),
        };

        match parser.parse_line(raw) {
            Ok(Some(event)) => outcomes.push(GeminiStreamJsonLineOutcome::Ok { line, event }),
            Ok(None) => {}
            Err(error) => outcomes.push(GeminiStreamJsonLineOutcome::Err { line, error }),
        }
    }

    outcomes
}

fn get_required_str(obj: &serde_json::Map<String, Value>, key: &str) -> Result<String, String> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("expected string field `{key}`"))
}
