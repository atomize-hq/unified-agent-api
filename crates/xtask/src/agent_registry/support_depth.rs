//! The registry's `support_depth` table, as `docs/specs/agent-registry-contract.md` defines it.
//!
//! This module decides only what that contract says the registry alone decides. Whether a target
//! is one the manifest root lists, and everything else that policy resolution needs a second
//! input for, is not decided here.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::{validate_non_empty_scalar, AgentRegistryError};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportDepth {
    #[serde(default)]
    pub enabled_paths: Option<Vec<LifecyclePath>>,
    #[serde(default)]
    pub enrollments: Option<Vec<DepthEnrollmentDeclaration>>,
}

impl SupportDepth {
    pub(super) fn validate(&self) -> Result<(), AgentRegistryError> {
        if self.enabled_paths.is_none() && self.enrollments.is_none() {
            return Err(invalid(
                "support_depth must be omitted when it holds neither `enabled_paths` nor `enrollments`",
            ));
        }
        if let Some(paths) = &self.enabled_paths {
            if paths.is_empty() {
                return Err(invalid(
                    "support_depth.enabled_paths must be omitted when no path is enabled",
                ));
            }
            let unique: BTreeSet<_> = paths.iter().collect();
            if unique.len() != paths.len() {
                return Err(invalid(
                    "support_depth.enabled_paths must not repeat a path",
                ));
            }
        }
        if let Some(enrollments) = &self.enrollments {
            if enrollments.is_empty() {
                return Err(invalid(
                    "support_depth.enrollments must be omitted when there is no declaration",
                ));
            }
            let mut versions = BTreeSet::new();
            for declaration in enrollments {
                declaration.validate()?;
                if !versions.insert(declaration.version.as_str()) {
                    return Err(invalid(format!(
                        "support_depth.enrollments declares version `{}` more than once",
                        declaration.version
                    )));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecyclePath {
    Maintenance,
    Onboarding,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepthEnrollmentDeclaration {
    pub lifecycle_path: LifecyclePath,
    pub version: String,
    pub targets: Vec<String>,
    pub modes: Vec<ModeSelector>,
    pub operations: Vec<OperationDeclaration>,
    #[serde(default)]
    pub capabilities: Vec<CapabilityMapping>,
}

impl DepthEnrollmentDeclaration {
    fn validate(&self) -> Result<(), AgentRegistryError> {
        let at = format!("support_depth.enrollments[version={}]", self.version);

        let exact = semver::Version::parse(&self.version)
            .map(|parsed| parsed.to_string() == self.version)
            .unwrap_or(false);
        if !exact {
            return Err(invalid(format!(
                "{at}.version must be one exact semantic version"
            )));
        }
        let targets = unique_non_empty(&format!("{at}.targets"), &self.targets)?;

        // Modes: unique names, and one selector for a command path and mode.
        if self.modes.is_empty() {
            return Err(invalid(format!("{at}.modes must list at least one mode")));
        }
        let mut selectors = BTreeSet::new();
        let mut labels = BTreeSet::new();
        let mut used_targets = BTreeSet::new();
        for mode in &self.modes {
            let mode_at = format!("{at}.modes[selector={}]", mode.selector);
            mode.validate(&mode_at)?;
            if !selectors.insert(mode.selector.as_str()) {
                return Err(invalid(format!("{mode_at} is declared more than once")));
            }
            if !labels.insert((mode.command_path.as_str(), mode.mode.as_str())) {
                return Err(invalid(format!(
                    "{mode_at} overlaps another selector with the same command path and mode"
                )));
            }
            match &mode.targets {
                None => used_targets.extend(targets.iter().copied()),
                Some(narrowed) => {
                    for target in unique_non_empty(&format!("{mode_at}.targets"), narrowed)? {
                        if !targets.contains(target) {
                            return Err(invalid(format!(
                                "{mode_at}.targets names `{target}`, which the declaration's \
                                 targets do not list"
                            )));
                        }
                        used_targets.insert(target);
                    }
                }
            }
        }
        if let Some(unused) = targets.difference(&used_targets).next() {
            return Err(invalid(format!(
                "{at}.targets lists `{unused}`, which no mode is selected on"
            )));
        }

        // Operations: each serves declared modes, and an excluded mode has no other operation.
        if self.operations.is_empty() {
            return Err(invalid(format!(
                "{at}.operations must list at least one operation"
            )));
        }
        let mut operations = BTreeMap::new();
        let mut served: BTreeMap<&str, Vec<&OperationDeclaration>> = BTreeMap::new();
        for operation in &self.operations {
            let op_at = format!("{at}.operations[id={}]", operation.id);
            operation.validate(&op_at)?;
            if operations
                .insert(operation.id.as_str(), operation)
                .is_some()
            {
                return Err(invalid(format!("{op_at} is declared more than once")));
            }
            for selector in unique_non_empty(&format!("{op_at}.serves"), &operation.serves)? {
                if !selectors.contains(selector) {
                    return Err(invalid(format!(
                        "{op_at}.serves names undeclared mode `{selector}`"
                    )));
                }
                served.entry(selector).or_default().push(operation);
            }
        }
        for selector in &selectors {
            let serving = served.get(selector).map(Vec::as_slice).unwrap_or_default();
            if serving.is_empty() {
                return Err(invalid(format!(
                    "{at}.modes[selector={selector}] is served by no operation"
                )));
            }
            if serving.len() > 1 && serving.iter().any(|op| op.is_excluded()) {
                return Err(invalid(format!(
                    "{at}.modes[selector={selector}] is excluded and also served by another operation"
                )));
            }
        }

        // Capability mappings name mapped operations, and every mapped operation is named.
        let mut capability_ids = BTreeSet::new();
        let mut named = BTreeSet::new();
        for mapping in &self.capabilities {
            validate_non_empty_scalar(&format!("{at}.capabilities.id"), &mapping.id)?;
            let cap_at = format!("{at}.capabilities[id={}]", mapping.id);
            if !capability_ids.insert(mapping.id.as_str()) {
                return Err(invalid(format!("{cap_at} is declared more than once")));
            }
            for id in unique_non_empty(&format!("{cap_at}.operations"), &mapping.operations)? {
                match operations.get(id) {
                    Some(op) if op.shared_mapping == Some(SharedMapping::Mapped) => {
                        named.insert(id);
                    }
                    Some(_) => {
                        return Err(invalid(format!(
                            "{cap_at}.operations names `{id}`, whose shared_mapping is not `mapped`"
                        )));
                    }
                    None => {
                        return Err(invalid(format!(
                            "{cap_at}.operations names undeclared operation `{id}`"
                        )));
                    }
                }
            }
        }
        for operation in &self.operations {
            if operation.shared_mapping == Some(SharedMapping::Mapped)
                && !named.contains(operation.id.as_str())
            {
                return Err(invalid(format!(
                    "{at}.operations[id={}] is `mapped` but no capability mapping names it",
                    operation.id
                )));
            }
        }
        Ok(())
    }
}

/// A depth enrollment selector. Its `selector` is the name a covered mode keeps.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModeSelector {
    pub selector: String,
    pub command_path: String,
    pub mode: String,
    #[serde(default)]
    pub predicates: Vec<FlagPredicate>,
    #[serde(default)]
    pub targets: Option<Vec<String>>,
}

impl ModeSelector {
    fn validate(&self, at: &str) -> Result<(), AgentRegistryError> {
        validate_name(&format!("{at}.selector"), &self.selector)?;
        validate_name(&format!("{at}.mode"), &self.mode)?;
        if self.command_path.is_empty()
            || self
                .command_path
                .split(' ')
                .any(|token| token.is_empty() || token.contains(char::is_whitespace))
        {
            return Err(invalid(format!(
                "{at}.command_path must be command tokens separated by single spaces"
            )));
        }
        let mut flags = BTreeSet::new();
        for predicate in &self.predicates {
            let flag = predicate.flag.as_str();
            if !flag.starts_with('-') || flag.contains(char::is_whitespace) {
                return Err(invalid(format!(
                    "{at}.predicates flag `{flag}` must start with `-` and hold no whitespace"
                )));
            }
            if predicate.value.as_deref() == Some("") {
                return Err(invalid(format!(
                    "{at}.predicates value for `{flag}` must not be empty"
                )));
            }
            if !flags.insert(flag) {
                return Err(invalid(format!(
                    "{at}.predicates names flag `{flag}` more than once"
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlagPredicate {
    pub flag: String,
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationDeclaration {
    pub id: String,
    pub serves: Vec<String>,
    pub templates: Vec<ObligationTemplate>,
    #[serde(default)]
    pub shared_mapping: Option<SharedMapping>,
    #[serde(default)]
    pub promise: Option<BasePromise>,
    #[serde(default)]
    pub not_applicable_permitted: Vec<ObligationId>,
}

impl OperationDeclaration {
    pub fn is_excluded(&self) -> bool {
        self.templates.contains(&ObligationTemplate::Excluded)
    }

    /// The obligation set the declaration gives this operation, `M` aside: `M` obligations are
    /// the mapped capability's minimum semantics and carry no id of their own here.
    pub fn obligations(&self) -> BTreeSet<ObligationId> {
        let mut set: BTreeSet<_> = self
            .templates
            .iter()
            .flat_map(|template| template.obligations())
            .copied()
            .collect();
        if !self.is_excluded() {
            set.extend(ObligationId::BASE);
        }
        set
    }

    fn validate(&self, at: &str) -> Result<(), AgentRegistryError> {
        validate_name(&format!("{at}.id"), &self.id)?;
        let templates: BTreeSet<_> = self.templates.iter().collect();
        if templates.is_empty() || templates.len() != self.templates.len() {
            return Err(invalid(format!(
                "{at}.templates must list at least one template, without repeats"
            )));
        }

        if self.is_excluded() {
            if self.templates.len() != 1
                || self.shared_mapping.is_some()
                || self.promise.is_some()
                || !self.not_applicable_permitted.is_empty()
            {
                return Err(invalid(format!(
                    "{at} is a mode exclusion: `EXCLUDED` must be its only template and it must \
                     carry no `shared_mapping`, `promise` or `not_applicable_permitted`"
                )));
            }
            return Ok(());
        }

        if self.shared_mapping.is_none() {
            return Err(invalid(format!("{at}.shared_mapping is required")));
        }
        match &self.promise {
            Some(promise) => promise.validate(at)?,
            None => return Err(invalid(format!("{at}.promise is required"))),
        }
        let obligations = self.obligations();
        if !obligations.iter().any(|id| id.is_output())
            || !obligations.iter().any(|id| id.is_lifecycle())
        {
            return Err(invalid(format!(
                "{at}.templates must select an output obligation and a lifecycle obligation"
            )));
        }
        let mut permitted = BTreeSet::new();
        for id in &self.not_applicable_permitted {
            if !obligations.contains(id) || !permitted.insert(id) {
                return Err(invalid(format!(
                    "{at}.not_applicable_permitted must name obligations of this operation, \
                     without repeats (got `{id:?}`)"
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedMapping {
    WrapperOnly,
    Mapped,
    ProposedExtension,
}

/// What the promise states for each BASE dimension of the support-depth contract.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasePromise {
    pub r1: String,
    pub v1: String,
    pub g1: String,
    pub e1: String,
    pub a1: String,
    pub x1: String,
    pub x2: String,
    pub x3: String,
    pub x4: String,
}

impl BasePromise {
    fn validate(&self, at: &str) -> Result<(), AgentRegistryError> {
        for (key, value) in [
            ("r1", &self.r1),
            ("v1", &self.v1),
            ("g1", &self.g1),
            ("e1", &self.e1),
            ("a1", &self.a1),
            ("x1", &self.x1),
            ("x2", &self.x2),
            ("x3", &self.x3),
            ("x4", &self.x4),
        ] {
            validate_non_empty_scalar(&format!("{at}.promise.{key}"), value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityMapping {
    pub id: String,
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ObligationTemplate {
    Core,
    Session,
    Query,
    Admin,
    Diag,
    Opaque,
    Launch,
    Managed,
    Mcp,
    Remote,
    Excluded,
}

impl ObligationTemplate {
    /// The obligations the template adds, as the support-depth contract's template table lists.
    pub fn obligations(self) -> &'static [ObligationId] {
        use ObligationId::*;
        match self {
            Self::Core => &[Out3, Lc2],
            Self::Session => &[S1],
            Self::Query => &[Out2, Lc1],
            Self::Admin => &[Out1, Lc1, Fx1],
            Self::Diag => &[Out1, Lc1, B1],
            Self::Opaque => &[Out1, Lc1],
            Self::Launch => &[Out4, Lc3],
            Self::Managed => &[Out4, Out3, Lc4],
            Self::Mcp => &[Mcp1],
            Self::Remote => &[N1],
            Self::Excluded => &[Ex1],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ObligationId {
    R1,
    V1,
    G1,
    E1,
    A1,
    X1,
    X2,
    X3,
    X4,
    Out1,
    Out2,
    Out3,
    Out4,
    Lc1,
    Lc2,
    Lc3,
    Lc4,
    Fx1,
    S1,
    N1,
    B1,
    Mcp1,
    Ex1,
}

impl ObligationId {
    pub const BASE: [Self; 9] = [
        Self::R1,
        Self::V1,
        Self::G1,
        Self::E1,
        Self::A1,
        Self::X1,
        Self::X2,
        Self::X3,
        Self::X4,
    ];

    fn is_output(self) -> bool {
        matches!(self, Self::Out1 | Self::Out2 | Self::Out3 | Self::Out4)
    }

    fn is_lifecycle(self) -> bool {
        matches!(self, Self::Lc1 | Self::Lc2 | Self::Lc3 | Self::Lc4)
    }
}

fn invalid(message: impl Into<String>) -> AgentRegistryError {
    AgentRegistryError::Validation(message.into())
}

/// Names that a depth record keeps: lowercase ASCII letters, digits, `.`, `_` and `-`, starting
/// with a letter or digit.
fn validate_name(field: &str, value: &str) -> Result<(), AgentRegistryError> {
    let mut bytes = value.bytes();
    let ok = bytes
        .next()
        .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && bytes.all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        });
    if ok {
        Ok(())
    } else {
        Err(invalid(format!(
            "{field} `{value}` must start with a lowercase letter or digit and hold only lowercase \
             letters, digits, `.`, `_` and `-`"
        )))
    }
}

fn unique_non_empty<'a>(
    field: &str,
    values: &'a [String],
) -> Result<BTreeSet<&'a str>, AgentRegistryError> {
    let set: BTreeSet<&str> = values.iter().map(String::as_str).collect();
    if values.is_empty() || set.len() != values.len() || set.contains("") {
        return Err(invalid(format!(
            "{field} must list at least one entry, without repeats or empty entries"
        )));
    }
    Ok(set)
}
