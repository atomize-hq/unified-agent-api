//! Ordinary wrapper note/exclusion rules shared by manifest validation and read-only intake.
use super::{models::WrapperCoverageV1, report, rules::RulesParityExclusions};
use serde_json::Value;

#[derive(Debug)]
pub struct ClaimViolation {
    pub code: &'static str,
    pub json_pointer: String,
    pub message: String,
    pub unit: &'static str,
    pub command_path: String,
    pub key_or_name: Option<String>,
    pub field: &'static str,
}

/// Schema validation is the caller's boundary; this applies the ordinary semantic note and
/// exclusion constraints without filesystem reads, writes or report generation.
pub fn validate(
    wrapper: &Value,
    exclusions: Option<&Value>,
) -> Result<Vec<ClaimViolation>, serde_json::Error> {
    let wrapper: WrapperCoverageV1 = serde_json::from_value(wrapper.clone())?;
    let exclusions = exclusions
        .map(|value| serde_json::from_value::<RulesParityExclusions>(value.clone()))
        .transpose()?;
    let exclusions = exclusions
        .as_ref()
        .filter(|e| e.schema_version == 1)
        .map(report::build_parity_exclusions_index);
    let mut out = Vec::new();
    for (ci, command) in wrapper.coverage.iter().enumerate() {
        let path = if command.path.is_empty() {
            "[]".to_owned()
        } else {
            command.path.join("/")
        };
        let mut claims = vec![(
            "command",
            None,
            command.level.as_str(),
            command.note.as_deref(),
            format!("/coverage/{ci}"),
        )];
        claims.extend(command.flags.iter().enumerate().map(|(i, f)| {
            (
                "flag",
                Some(f.key.as_str()),
                f.level.as_str(),
                f.note.as_deref(),
                format!("/coverage/{ci}/flags/{i}"),
            )
        }));
        claims.extend(command.args.iter().enumerate().map(|(i, a)| {
            (
                "arg",
                Some(a.name.as_str()),
                a.level.as_str(),
                a.note.as_deref(),
                format!("/coverage/{ci}/args/{i}"),
            )
        }));
        for (unit, leaf, level, note, pointer) in claims {
            let (wrapper_unit, field, context) = match unit {
                "flag" => (
                    "wrapper_flag",
                    "flags",
                    format!("command_path={path} key={}", leaf.unwrap_or_default()),
                ),
                "arg" => (
                    "wrapper_arg",
                    "args",
                    format!("command_path={path} name={}", leaf.unwrap_or_default()),
                ),
                _ => (
                    "wrapper_command",
                    "coverage",
                    format!("command_path={path}"),
                ),
            };
            if matches!(level, "passthrough" | "intentionally_unsupported")
                && note.unwrap_or("").trim().is_empty()
            {
                out.push(ClaimViolation {
                    code: if level == "passthrough" {
                        "PASSTHROUGH_NOTE_MISSING"
                    } else {
                        "IU_NOTE_MISSING"
                    },
                    json_pointer: format!("{pointer}/note"),
                    message: format!("{level} requires non-empty note (unit={unit} {context})"),
                    unit: wrapper_unit,
                    command_path: path.clone(),
                    key_or_name: leaf.map(str::to_owned),
                    field: "note",
                });
            }
            let excluded = exclusions.as_ref().and_then(|e| match unit {
                "flag" => e
                    .flags
                    .get(&(command.path.clone(), leaf.unwrap_or_default().into())),
                "arg" => e
                    .args
                    .get(&(command.path.clone(), leaf.unwrap_or_default().into())),
                _ => e.commands.get(&command.path),
            });
            if let Some(ex) = excluded {
                let message = if unit == "command" {
                    format!(
                        "wrapper_coverage includes excluded command_path={path} (note={})",
                        ex.note
                    )
                } else {
                    format!(
                        "wrapper_coverage includes excluded {unit} ({context} note={})",
                        ex.note
                    )
                };
                out.push(ClaimViolation {
                    code: "WRAPPER_COVERAGE_INCLUDES_EXCLUDED",
                    json_pointer: pointer,
                    message,
                    unit: wrapper_unit,
                    command_path: path.clone(),
                    key_or_name: leaf.map(str::to_owned),
                    field,
                });
            }
        }
    }
    Ok(out)
}
