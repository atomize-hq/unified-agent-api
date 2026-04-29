use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AiderCliError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AiderStreamJsonRunRequest {
    prompt: String,
    model: Option<String>,
    working_dir: Option<PathBuf>,
}

impl AiderStreamJsonRunRequest {
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

    pub(crate) fn argv(&self) -> Result<Vec<OsString>, AiderCliError> {
        if self.prompt.trim().is_empty() {
            return Err(AiderCliError::InvalidRequest(
                "prompt must not be empty".to_string(),
            ));
        }

        let mut argv = vec![
            OsString::from("--prompt"),
            OsString::from(self.prompt.as_str()),
            OsString::from("--message-format"),
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
pub struct AiderToolResultError {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AiderStreamJsonResultPayload {
    pub status: String,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub stats: Option<Value>,
    pub raw: Value,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AiderStreamJsonEvent {
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
        error: Option<AiderToolResultError>,
        raw: Value,
    },
    Error {
        severity: String,
        message: String,
        raw: Value,
    },
    Result {
        payload: AiderStreamJsonResultPayload,
    },
    Unknown {
        event_type: String,
        raw: Value,
    },
}

impl AiderStreamJsonEvent {
    pub fn raw(&self) -> &Value {
        match self {
            AiderStreamJsonEvent::Init { raw, .. } => raw,
            AiderStreamJsonEvent::Message { raw, .. } => raw,
            AiderStreamJsonEvent::ToolUse { raw, .. } => raw,
            AiderStreamJsonEvent::ToolResult { raw, .. } => raw,
            AiderStreamJsonEvent::Error { raw, .. } => raw,
            AiderStreamJsonEvent::Result { payload } => &payload.raw,
            AiderStreamJsonEvent::Unknown { raw, .. } => raw,
        }
    }

    pub fn event_type(&self) -> &str {
        match self {
            AiderStreamJsonEvent::Init { .. } => "init",
            AiderStreamJsonEvent::Message { .. } => "message",
            AiderStreamJsonEvent::ToolUse { .. } => "tool_use",
            AiderStreamJsonEvent::ToolResult { .. } => "tool_result",
            AiderStreamJsonEvent::Error { .. } => "error",
            AiderStreamJsonEvent::Result { .. } => "result",
            AiderStreamJsonEvent::Unknown { event_type, .. } => event_type.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AiderStreamJsonErrorCode {
    JsonParse,
    TypedParse,
}

#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
#[error("{message}")]
pub struct AiderStreamJsonError {
    pub code: AiderStreamJsonErrorCode,
    pub message: String,
    pub details: String,
}

impl AiderStreamJsonError {
    fn new(code: AiderStreamJsonErrorCode, message: String) -> Self {
        Self {
            code,
            details: message.clone(),
            message,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AiderStreamJsonParser;

impl AiderStreamJsonParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_line(
        &mut self,
        line: &str,
    ) -> Result<Option<AiderStreamJsonEvent>, AiderStreamJsonError> {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.chars().all(|ch| ch.is_whitespace()) {
            return Ok(None);
        }

        let value: Value = serde_json::from_str(line).map_err(|err| {
            AiderStreamJsonError::new(
                AiderStreamJsonErrorCode::JsonParse,
                format!("invalid JSON: {err}"),
            )
        })?;

        self.parse_json(&value)
    }

    pub fn parse_json(
        &mut self,
        value: &Value,
    ) -> Result<Option<AiderStreamJsonEvent>, AiderStreamJsonError> {
        let obj = value.as_object().ok_or_else(|| {
            AiderStreamJsonError::new(
                AiderStreamJsonErrorCode::TypedParse,
                "expected JSON object".to_string(),
            )
        })?;

        let event_type = get_required_str(obj, "type").map_err(|message| {
            AiderStreamJsonError::new(AiderStreamJsonErrorCode::TypedParse, message)
        })?;

        let event = match event_type.as_str() {
            "init" => AiderStreamJsonEvent::Init {
                session_id: get_required_str(obj, "session_id").map_err(typed_parse_error)?,
                model: get_required_str(obj, "model").map_err(typed_parse_error)?,
                raw: value.clone(),
            },
            "message" => AiderStreamJsonEvent::Message {
                role: get_required_str(obj, "role").map_err(typed_parse_error)?,
                content: get_required_str(obj, "content").map_err(typed_parse_error)?,
                delta: obj.get("delta").and_then(Value::as_bool).unwrap_or(false),
                raw: value.clone(),
            },
            "tool_use" => AiderStreamJsonEvent::ToolUse {
                tool_name: get_required_str(obj, "tool_name").map_err(typed_parse_error)?,
                tool_id: get_required_str(obj, "tool_id").map_err(typed_parse_error)?,
                parameters: obj
                    .get("parameters")
                    .cloned()
                    .ok_or_else(|| typed_parse_error("expected field `parameters`".to_string()))?,
                raw: value.clone(),
            },
            "tool_result" => AiderStreamJsonEvent::ToolResult {
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
            "error" => AiderStreamJsonEvent::Error {
                severity: get_required_str(obj, "severity").map_err(typed_parse_error)?,
                message: get_required_str(obj, "message").map_err(typed_parse_error)?,
                raw: value.clone(),
            },
            "result" => AiderStreamJsonEvent::Result {
                payload: AiderStreamJsonResultPayload {
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
            _ => AiderStreamJsonEvent::Unknown {
                event_type,
                raw: value.clone(),
            },
        };

        Ok(Some(event))
    }
}

fn typed_parse_error(message: String) -> AiderStreamJsonError {
    AiderStreamJsonError::new(AiderStreamJsonErrorCode::TypedParse, message)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AiderStreamJsonLine {
    pub line_number: usize,
    pub raw: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AiderStreamJsonLineOutcome {
    Ok {
        line: AiderStreamJsonLine,
        event: AiderStreamJsonEvent,
    },
    Err {
        line: AiderStreamJsonLine,
        error: AiderStreamJsonError,
    },
}

pub fn parse_stream_json_lines(input: &str) -> Vec<AiderStreamJsonLineOutcome> {
    let mut parser = AiderStreamJsonParser::new();
    let mut outcomes = Vec::new();

    for (index, raw) in input.lines().enumerate() {
        let line = AiderStreamJsonLine {
            line_number: index + 1,
            raw: raw.to_string(),
        };

        match parser.parse_line(raw) {
            Ok(Some(event)) => outcomes.push(AiderStreamJsonLineOutcome::Ok { line, event }),
            Ok(None) => {}
            Err(error) => outcomes.push(AiderStreamJsonLineOutcome::Err { line, error }),
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
