use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Rules {
    pub union: RulesUnion,
    pub sorting: RulesSorting,
    pub wrapper_coverage: RulesWrapperCoverage,
}

#[derive(Debug, Deserialize)]
pub struct RulesUnion {
    pub expected_targets: Vec<String>,
    pub platform_mapping: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct RulesSorting {
    pub commands: String,
    pub flags: String,
    pub args: String,
    pub expected_targets: String,
}

#[derive(Debug, Deserialize)]
pub struct RulesWrapperCoverage {
    pub scope_semantics: RulesWrapperScopeSemantics,
}

#[derive(Debug, Deserialize)]
pub struct RulesWrapperScopeSemantics {
    pub platforms_expand_to_expected_targets: bool,
    pub platforms_expand_using: String,
}

#[derive(Debug)]
pub enum SharedError {
    RulesUnsupported(String),
    ManifestInvalid(String),
}

pub trait CoverageLevelLike: Copy {
    fn sort_key(self) -> u8;
}

pub trait ScopeLike {
    fn platforms(&self) -> Option<&[String]>;
    fn target_triples(&self) -> Option<&[String]>;
    fn set_platforms(&mut self, platforms: Option<Vec<String>>);
    fn set_target_triples(&mut self, target_triples: Option<Vec<String>>);
}

pub trait FlagLike<S: ScopeLike> {
    type Level: CoverageLevelLike;

    fn key(&self) -> &str;
    fn level(&self) -> Self::Level;
    fn note(&self) -> Option<&str>;
    fn scope(&self) -> Option<&S>;
    fn scope_mut(&mut self) -> &mut Option<S>;
}

pub trait ArgLike<S: ScopeLike> {
    type Level: CoverageLevelLike;

    fn name(&self) -> &str;
    fn level(&self) -> Self::Level;
    fn note(&self) -> Option<&str>;
    fn scope(&self) -> Option<&S>;
    fn scope_mut(&mut self) -> &mut Option<S>;
}

pub trait CommandLike<S: ScopeLike, F: FlagLike<S>, A: ArgLike<S>> {
    type Level: CoverageLevelLike;

    fn path(&self) -> &[String];
    fn level(&self) -> Self::Level;
    fn note(&self) -> Option<&str>;
    fn scope(&self) -> Option<&S>;
    fn scope_mut(&mut self) -> &mut Option<S>;
    fn flags_mut(&mut self) -> &mut Option<Vec<F>>;
    fn args_mut(&mut self) -> &mut Option<Vec<A>>;
}

pub trait ManifestLike<C> {
    fn coverage_mut(&mut self) -> &mut Vec<C>;
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootIntakeLayout {
    root: PathBuf,
}

#[allow(dead_code)]
impl RootIntakeLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.root.join("versions")
    }

    pub fn version_metadata_path(&self, version: &str) -> PathBuf {
        self.versions_dir().join(format!("{version}.json"))
    }

    pub fn latest_supported_pointers_dir(&self) -> PathBuf {
        self.root.join("pointers").join("latest_supported")
    }

    pub fn latest_supported_pointer_path(&self, target_triple: &str) -> PathBuf {
        self.latest_supported_pointers_dir()
            .join(format!("{target_triple}.txt"))
    }

    pub fn latest_validated_pointers_dir(&self) -> PathBuf {
        self.root.join("pointers").join("latest_validated")
    }

    pub fn latest_validated_pointer_path(&self, target_triple: &str) -> PathBuf {
        self.latest_validated_pointers_dir()
            .join(format!("{target_triple}.txt"))
    }

    pub fn current_json_path(&self) -> PathBuf {
        self.root.join("current.json")
    }

    pub fn reports_dir(&self) -> PathBuf {
        self.root.join("reports")
    }

    pub fn reports_version_dir(&self, version: &str) -> PathBuf {
        self.reports_dir().join(version)
    }
}

pub fn assert_supported_rules(rules: &Rules) -> Result<(), SharedError> {
    let mut unsupported = Vec::new();

    if rules.sorting.commands != "lexicographic_path" {
        unsupported.push(format!("sorting.commands={}", rules.sorting.commands));
    }
    if rules.sorting.flags != "by_key_then_long_then_short" {
        unsupported.push(format!("sorting.flags={}", rules.sorting.flags));
    }
    if rules.sorting.args != "by_name" {
        unsupported.push(format!("sorting.args={}", rules.sorting.args));
    }
    if rules.sorting.expected_targets != "rules_expected_targets_order" {
        unsupported.push(format!(
            "sorting.expected_targets={}",
            rules.sorting.expected_targets
        ));
    }

    if !rules
        .wrapper_coverage
        .scope_semantics
        .platforms_expand_to_expected_targets
    {
        unsupported.push(
            "wrapper_coverage.scope_semantics.platforms_expand_to_expected_targets=false"
                .to_string(),
        );
    }
    if rules
        .wrapper_coverage
        .scope_semantics
        .platforms_expand_using
        != "union.platform_mapping"
    {
        unsupported.push(format!(
            "wrapper_coverage.scope_semantics.platforms_expand_using={}",
            rules
                .wrapper_coverage
                .scope_semantics
                .platforms_expand_using
        ));
    }

    if unsupported.is_empty() {
        Ok(())
    } else {
        Err(SharedError::RulesUnsupported(unsupported.join(", ")))
    }
}

pub fn invert_platform_mapping(
    expected_targets: &[String],
    platform_mapping: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Vec<String>>, SharedError> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for target in expected_targets {
        let Some(platform) = platform_mapping.get(target) else {
            return Err(SharedError::RulesUnsupported(format!(
                "union.platform_mapping missing expected target: {target}"
            )));
        };
        out.entry(platform.clone())
            .or_default()
            .push(target.clone());
    }
    Ok(out)
}

pub fn normalize_manifest<M, C, F, A, S>(
    manifest: &mut M,
    expected_targets: &[String],
    platform_to_targets: &BTreeMap<String, Vec<String>>,
) -> Result<(), SharedError>
where
    M: ManifestLike<C>,
    C: CommandLike<S, F, A>,
    F: FlagLike<S>,
    A: ArgLike<S>,
    S: ScopeLike,
{
    let coverage = manifest.coverage_mut();
    for cmd in coverage.iter_mut() {
        normalize_command(cmd, expected_targets, platform_to_targets)?;
    }

    coverage.sort_by(|a, b| {
        command_sort_key(a, expected_targets).cmp(&command_sort_key(b, expected_targets))
    });
    Ok(())
}

fn normalize_command<C, F, A, S>(
    cmd: &mut C,
    expected_targets: &[String],
    platform_to_targets: &BTreeMap<String, Vec<String>>,
) -> Result<(), SharedError>
where
    C: CommandLike<S, F, A>,
    F: FlagLike<S>,
    A: ArgLike<S>,
    S: ScopeLike,
{
    normalize_scope(cmd.scope_mut(), expected_targets, platform_to_targets)?;

    let flags = cmd.flags_mut();
    let mut clear_flags = false;
    if let Some(flags) = flags.as_mut() {
        for flag in flags.iter_mut() {
            normalize_scope(flag.scope_mut(), expected_targets, platform_to_targets)?;
        }
        flags.sort_by(|a, b| {
            flag_sort_key(a, expected_targets).cmp(&flag_sort_key(b, expected_targets))
        });
        clear_flags = flags.is_empty();
    }
    if clear_flags {
        *flags = None;
    }

    let args = cmd.args_mut();
    let mut clear_args = false;
    if let Some(args) = args.as_mut() {
        for arg in args.iter_mut() {
            normalize_scope(arg.scope_mut(), expected_targets, platform_to_targets)?;
        }
        args.sort_by(|a, b| {
            arg_sort_key(a, expected_targets).cmp(&arg_sort_key(b, expected_targets))
        });
        clear_args = args.is_empty();
    }
    if clear_args {
        *args = None;
    }

    Ok(())
}

fn normalize_scope<S: ScopeLike>(
    scope: &mut Option<S>,
    expected_targets: &[String],
    platform_to_targets: &BTreeMap<String, Vec<String>>,
) -> Result<(), SharedError> {
    let Some(scope_value) = scope.as_mut() else {
        return Ok(());
    };

    let expected_set: BTreeSet<&str> = expected_targets
        .iter()
        .map(|target| target.as_str())
        .collect();
    let mut targets: BTreeSet<String> = BTreeSet::new();

    if let Some(existing) = scope_value.target_triples() {
        for target in existing {
            if !expected_set.contains(target.as_str()) {
                return Err(SharedError::ManifestInvalid(format!(
                    "scope target_triples contains non-expected target: {target}"
                )));
            }
        }
        targets.extend(existing.iter().cloned());
    }
    if let Some(platforms) = scope_value.platforms() {
        for platform in platforms {
            let Some(mapped) = platform_to_targets.get(platform) else {
                return Err(SharedError::ManifestInvalid(format!(
                    "scope platforms contains unknown platform label: {platform}"
                )));
            };
            targets.extend(mapped.iter().cloned());
        }
    }

    if targets.is_empty() {
        *scope = None;
        return Ok(());
    }

    let mut target_triples: Vec<String> = targets.into_iter().collect();
    target_triples.sort_by(|a, b| {
        target_order_key(a, expected_targets).cmp(&target_order_key(b, expected_targets))
    });

    scope_value.set_platforms(None);
    scope_value.set_target_triples(Some(target_triples));
    Ok(())
}

fn target_order_key<'a>(target: &'a str, expected_targets: &[String]) -> (usize, &'a str) {
    (
        expected_targets
            .iter()
            .position(|candidate| candidate == target)
            .unwrap_or(usize::MAX),
        target,
    )
}

fn command_sort_key<C, F, A, S>(
    cmd: &C,
    expected_targets: &[String],
) -> (Vec<String>, u8, String, u8, String)
where
    C: CommandLike<S, F, A>,
    F: FlagLike<S>,
    A: ArgLike<S>,
    S: ScopeLike,
{
    let (scope_kind, scope_key) = scope_sort_key(cmd.scope(), expected_targets);
    (
        cmd.path().to_vec(),
        scope_kind,
        scope_key,
        cmd.level().sort_key(),
        cmd.note().unwrap_or_default().to_string(),
    )
}

fn flag_sort_key<F, S>(flag: &F, expected_targets: &[String]) -> (String, u8, String, u8, String)
where
    F: FlagLike<S>,
    S: ScopeLike,
{
    let (scope_kind, scope_key) = scope_sort_key(flag.scope(), expected_targets);
    (
        flag.key().to_string(),
        scope_kind,
        scope_key,
        flag.level().sort_key(),
        flag.note().unwrap_or_default().to_string(),
    )
}

fn arg_sort_key<A, S>(arg: &A, expected_targets: &[String]) -> (String, u8, String, u8, String)
where
    A: ArgLike<S>,
    S: ScopeLike,
{
    let (scope_kind, scope_key) = scope_sort_key(arg.scope(), expected_targets);
    (
        arg.name().to_string(),
        scope_kind,
        scope_key,
        arg.level().sort_key(),
        arg.note().unwrap_or_default().to_string(),
    )
}

fn scope_sort_key<S: ScopeLike>(scope: Option<&S>, expected_targets: &[String]) -> (u8, String) {
    let Some(scope) = scope else {
        return (2, String::new());
    };

    let Some(targets) = scope.target_triples() else {
        return (2, String::new());
    };

    let mut normalized: Vec<&str> = targets.iter().map(|target| target.as_str()).collect();
    normalized.sort_by(|a, b| {
        target_order_key(a, expected_targets).cmp(&target_order_key(b, expected_targets))
    });
    (0, normalized.join(","))
}

#[cfg(test)]
mod tests {
    use super::RootIntakeLayout;
    use std::path::PathBuf;

    #[test]
    fn root_intake_layout_is_shape_driven_for_current_agent_roots() {
        for root in ["cli_manifests/codex", "cli_manifests/claude_code"] {
            let layout = RootIntakeLayout::new(root);
            assert_eq!(layout.root(), PathBuf::from(root).as_path());
            assert_eq!(layout.versions_dir(), PathBuf::from(root).join("versions"));
            assert_eq!(
                layout.version_metadata_path("1.2.3"),
                PathBuf::from(root).join("versions").join("1.2.3.json")
            );
            assert_eq!(
                layout.latest_supported_pointers_dir(),
                PathBuf::from(root)
                    .join("pointers")
                    .join("latest_supported")
            );
            assert_eq!(
                layout.latest_supported_pointer_path("x86_64-unknown-linux-musl"),
                PathBuf::from(root)
                    .join("pointers")
                    .join("latest_supported")
                    .join("x86_64-unknown-linux-musl.txt")
            );
            assert_eq!(
                layout.latest_validated_pointers_dir(),
                PathBuf::from(root)
                    .join("pointers")
                    .join("latest_validated")
            );
            assert_eq!(
                layout.latest_validated_pointer_path("x86_64-unknown-linux-musl"),
                PathBuf::from(root)
                    .join("pointers")
                    .join("latest_validated")
                    .join("x86_64-unknown-linux-musl.txt")
            );
            assert_eq!(
                layout.current_json_path(),
                PathBuf::from(root).join("current.json")
            );
            assert_eq!(layout.reports_dir(), PathBuf::from(root).join("reports"));
            assert_eq!(
                layout.reports_version_dir("1.2.3"),
                PathBuf::from(root).join("reports").join("1.2.3")
            );
        }
    }
}
