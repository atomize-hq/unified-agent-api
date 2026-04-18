use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::ExitStatus,
};

use serde_json::{Map, Value};

use crate::OpencodeError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OpencodeRunRequest {
    prompt: String,
    model: Option<String>,
    session: Option<String>,
    continue_session: bool,
    fork: bool,
    working_dir: Option<PathBuf>,
}

impl OpencodeRunRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            session: None,
            continue_session: false,
            fork: false,
            working_dir: None,
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn session(mut self, session_id: impl Into<String>) -> Self {
        self.session = Some(session_id.into());
        self
    }

    pub fn continue_session(mut self, value: bool) -> Self {
        self.continue_session = value;
        self
    }

    pub fn fork(mut self, value: bool) -> Self {
        self.fork = value;
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

    pub fn session_id(&self) -> Option<&str> {
        self.session.as_deref()
    }

    pub fn continue_requested(&self) -> bool {
        self.continue_session
    }

    pub fn fork_requested(&self) -> bool {
        self.fork
    }

    pub fn working_directory(&self) -> Option<&Path> {
        self.working_dir.as_deref()
    }

    pub(crate) fn argv(&self) -> Result<Vec<OsString>, OpencodeError> {
        if self.prompt.trim().is_empty() {
            return Err(OpencodeError::InvalidRequest(
                "prompt must not be empty".to_string(),
            ));
        }

        let mut argv = vec![
            OsString::from("run"),
            OsString::from("--format"),
            OsString::from("json"),
        ];

        if let Some(model) = normalize_non_empty(self.model.as_deref()) {
            argv.push(OsString::from("--model"));
            argv.push(OsString::from(model));
        }

        if let Some(session) = normalize_non_empty(self.session.as_deref()) {
            argv.push(OsString::from("--session"));
            argv.push(OsString::from(session));
        }

        if self.continue_session {
            argv.push(OsString::from("--continue"));
        }

        if self.fork {
            argv.push(OsString::from("--fork"));
        }

        if let Some(path) = &self.working_dir {
            argv.push(OsString::from("--dir"));
            argv.push(path.as_os_str().to_os_string());
        }

        argv.push(OsString::from(self.prompt.as_str()));
        Ok(argv)
    }
}

fn normalize_non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OpencodeRunCompletion {
    pub status: ExitStatus,
    pub final_text: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OpencodeRunJsonErrorCode {
    JsonParse,
    TypedParse,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct OpencodeRunJsonParseError {
    pub code: OpencodeRunJsonErrorCode,
    pub message: String,
    pub details: String,
}

impl OpencodeRunJsonParseError {
    fn new(code: OpencodeRunJsonErrorCode, message: String) -> Self {
        Self {
            code,
            details: message.clone(),
            message,
        }
    }
}

#[derive(Debug, Clone)]
pub enum OpencodeRunJsonEvent {
    StepStart {
        session_id: Option<String>,
        raw: Value,
    },
    Text {
        session_id: Option<String>,
        text: String,
        raw: Value,
    },
    StepFinish {
        session_id: Option<String>,
        raw: Value,
    },
    Unknown {
        event_type: String,
        session_id: Option<String>,
        raw: Value,
    },
}

impl OpencodeRunJsonEvent {
    pub fn raw(&self) -> &Value {
        match self {
            OpencodeRunJsonEvent::StepStart { raw, .. } => raw,
            OpencodeRunJsonEvent::Text { raw, .. } => raw,
            OpencodeRunJsonEvent::StepFinish { raw, .. } => raw,
            OpencodeRunJsonEvent::Unknown { raw, .. } => raw,
        }
    }

    pub fn event_type(&self) -> &str {
        match self {
            OpencodeRunJsonEvent::StepStart { .. } => "step_start",
            OpencodeRunJsonEvent::Text { .. } => "text",
            OpencodeRunJsonEvent::StepFinish { .. } => "step_finish",
            OpencodeRunJsonEvent::Unknown { event_type, .. } => event_type.as_str(),
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        match self {
            OpencodeRunJsonEvent::StepStart { session_id, .. } => session_id.as_deref(),
            OpencodeRunJsonEvent::Text { session_id, .. } => session_id.as_deref(),
            OpencodeRunJsonEvent::StepFinish { session_id, .. } => session_id.as_deref(),
            OpencodeRunJsonEvent::Unknown { session_id, .. } => session_id.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OpencodeRunJsonParser {
    last_session_id: Option<String>,
}

impl OpencodeRunJsonParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.last_session_id = None;
    }

    pub fn parse_line(
        &mut self,
        line: &str,
    ) -> Result<Option<OpencodeRunJsonEvent>, OpencodeRunJsonParseError> {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.chars().all(|ch| ch.is_whitespace()) {
            return Ok(None);
        }

        let value: Value = serde_json::from_str(line).map_err(|err| {
            OpencodeRunJsonParseError::new(
                OpencodeRunJsonErrorCode::JsonParse,
                format!("invalid JSON: {err}"),
            )
        })?;

        self.parse_json(&value)
    }

    pub fn parse_json(
        &mut self,
        value: &Value,
    ) -> Result<Option<OpencodeRunJsonEvent>, OpencodeRunJsonParseError> {
        let obj = value.as_object().ok_or_else(|| {
            OpencodeRunJsonParseError::new(
                OpencodeRunJsonErrorCode::TypedParse,
                "expected JSON object".to_string(),
            )
        })?;

        let event_type = get_required_str(obj, "type").map_err(|message| {
            OpencodeRunJsonParseError::new(OpencodeRunJsonErrorCode::TypedParse, message)
        })?;

        let session_id = get_optional_session_id(obj).or_else(|| self.last_session_id.clone());

        let event = match event_type.as_str() {
            "step_start" => OpencodeRunJsonEvent::StepStart {
                session_id,
                raw: value.clone(),
            },
            "text" => {
                let text = get_required_str(obj, "text").map_err(|message| {
                    OpencodeRunJsonParseError::new(OpencodeRunJsonErrorCode::TypedParse, message)
                })?;
                OpencodeRunJsonEvent::Text {
                    session_id,
                    text,
                    raw: value.clone(),
                }
            }
            "step_finish" => OpencodeRunJsonEvent::StepFinish {
                session_id,
                raw: value.clone(),
            },
            _ => OpencodeRunJsonEvent::Unknown {
                event_type,
                session_id,
                raw: value.clone(),
            },
        };

        if let Some(session_id) = event.session_id() {
            self.last_session_id = Some(session_id.to_string());
        }

        Ok(Some(event))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OpencodeRunJsonLine {
    pub line_number: usize,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub enum OpencodeRunJsonLineOutcome {
    Ok {
        line: OpencodeRunJsonLine,
        event: OpencodeRunJsonEvent,
    },
    Err {
        line: OpencodeRunJsonLine,
        error: OpencodeRunJsonParseError,
    },
}

pub fn parse_run_json_lines(input: &str) -> Vec<OpencodeRunJsonLineOutcome> {
    let mut parser = OpencodeRunJsonParser::new();
    let mut outcomes = Vec::new();

    for (index, raw) in input.lines().enumerate() {
        let line = OpencodeRunJsonLine {
            line_number: index + 1,
            raw: raw.to_string(),
        };

        match parser.parse_line(raw) {
            Ok(Some(event)) => outcomes.push(OpencodeRunJsonLineOutcome::Ok { line, event }),
            Ok(None) => {}
            Err(error) => outcomes.push(OpencodeRunJsonLineOutcome::Err { line, error }),
        }
    }

    outcomes
}

fn get_required_str(obj: &Map<String, Value>, key: &str) -> Result<String, String> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("expected string field `{key}`"))
}

fn get_optional_session_id(obj: &Map<String, Value>) -> Option<String> {
    ["session_id", "sessionId"]
        .into_iter()
        .find_map(|key| obj.get(key).and_then(Value::as_str))
        .map(ToOwned::to_owned)
}
