use std::{
    fmt, fs, io,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct PlannedMutation {
    relative_path: PathBuf,
    expected_before: Option<Vec<u8>>,
    desired_after: Vec<u8>,
    remove: bool,
    expected_mode: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MutationState {
    Absent,
    Identical,
    Divergent,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ApplySummary {
    pub total: usize,
    pub written: usize,
    pub identical: usize,
}

#[derive(Debug)]
pub struct WorkspacePathJail {
    root: PathBuf,
    canonical_root: PathBuf,
}

#[derive(Debug)]
pub enum WorkspaceMutationError {
    Validation(String),
    Internal(String),
}

#[derive(Debug)]
struct ClassifiedMutation<'a> {
    mutation: &'a PlannedMutation,
    absolute_path: PathBuf,
    state: MutationState,
}

#[derive(Debug)]
struct PreparedMutation<'a> {
    classified: ClassifiedMutation<'a>,
    temp_path: PathBuf,
}

#[derive(Debug)]
enum AppliedMutation {
    Removed {
        path: PathBuf,
        backup_path: PathBuf,
    },
    Created {
        path: PathBuf,
        written: Vec<u8>,
        permissions: fs::Permissions,
    },
    Replaced {
        path: PathBuf,
        backup_path: PathBuf,
        written: Vec<u8>,
        permissions: fs::Permissions,
    },
}

impl fmt::Display for WorkspaceMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

impl PlannedMutation {
    /// Owning proposals may retire a regular file in the same checked batch as their writes.
    pub(crate) fn remove(relative_path: impl Into<PathBuf>, expected_before: Vec<u8>) -> Self {
        let mut mutation = Self::replace(relative_path, expected_before, Vec::new());
        mutation.remove = true;
        mutation
    }

    pub fn create(relative_path: impl Into<PathBuf>, desired_after: Vec<u8>) -> Self {
        Self {
            relative_path: relative_path.into(),
            expected_before: None,
            desired_after,
            remove: false,
            expected_mode: None,
        }
    }

    pub fn replace(
        relative_path: impl Into<PathBuf>,
        expected_before: Vec<u8>,
        desired_after: Vec<u8>,
    ) -> Self {
        Self {
            relative_path: relative_path.into(),
            expected_before: Some(expected_before),
            desired_after,
            remove: false,
            expected_mode: None,
        }
    }

    /// Optional mode precondition for invocation snapshots; bytes alone cannot detect chmod.
    pub(crate) fn with_expected_mode(mut self, mode: u32) -> Self {
        self.expected_mode = Some(mode);
        self
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }
}

impl WorkspacePathJail {
    pub fn new(root: &Path) -> Result<Self, WorkspaceMutationError> {
        let canonical_root = fs::canonicalize(root).map_err(|err| {
            WorkspaceMutationError::Internal(format!("canonicalize {}: {err}", root.display()))
        })?;
        Ok(Self {
            root: root.to_path_buf(),
            canonical_root,
        })
    }

    pub fn resolve(&self, relative_path: &Path) -> Result<PathBuf, WorkspaceMutationError> {
        let mut components = relative_path.components().peekable();
        if components.peek().is_none() {
            return Err(WorkspaceMutationError::Validation(
                "planned path must not be empty".to_string(),
            ));
        }

        let mut current_lexical = self.root.clone();
        let mut current_canonical = self.canonical_root.clone();

        while let Some(component) = components.next() {
            let Component::Normal(part) = component else {
                return Err(WorkspaceMutationError::Validation(format!(
                    "planned path `{}` must stay repo-relative and must not contain `..` or absolute prefixes",
                    relative_path.display()
                )));
            };

            current_lexical.push(part);
            match fs::symlink_metadata(&current_lexical) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        return Err(WorkspaceMutationError::Validation(format!(
                            "planned path `{}` traverses symlinked component `{}`",
                            relative_path.display(),
                            current_lexical
                                .strip_prefix(&self.root)
                                .unwrap_or(&current_lexical)
                                .display()
                        )));
                    }
                    if components.peek().is_some() && !metadata.is_dir() {
                        return Err(WorkspaceMutationError::Validation(format!(
                            "planned path `{}` traverses non-directory component `{}`",
                            relative_path.display(),
                            current_lexical
                                .strip_prefix(&self.root)
                                .unwrap_or(&current_lexical)
                                .display()
                        )));
                    }
                    let canonical = fs::canonicalize(&current_lexical).map_err(|err| {
                        WorkspaceMutationError::Internal(format!(
                            "canonicalize {}: {err}",
                            current_lexical.display()
                        ))
                    })?;
                    if !canonical.starts_with(&self.canonical_root) {
                        return Err(WorkspaceMutationError::Validation(format!(
                            "planned path `{}` resolves outside workspace root",
                            relative_path.display()
                        )));
                    }
                    current_canonical = canonical;
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    let mut unresolved = PathBuf::from(part);
                    for next in components {
                        let Component::Normal(next_part) = next else {
                            return Err(WorkspaceMutationError::Validation(format!(
                                "planned path `{}` must stay repo-relative and must not contain `..` or absolute prefixes",
                                relative_path.display()
                            )));
                        };
                        unresolved.push(next_part);
                    }
                    return Ok(current_canonical.join(unresolved));
                }
                Err(err) => {
                    return Err(WorkspaceMutationError::Internal(format!(
                        "stat {}: {err}",
                        current_lexical.display()
                    )));
                }
            }
        }

        Ok(current_canonical)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn ensure_parent_dirs(
        &self,
        relative_path: &Path,
        created_dirs: &mut Vec<PathBuf>,
    ) -> Result<(), WorkspaceMutationError> {
        let Some(parent) = relative_path.parent() else {
            return Ok(());
        };

        let mut current = self.canonical_root.clone();
        for component in parent.components() {
            let Component::Normal(part) = component else {
                return Err(WorkspaceMutationError::Validation(format!(
                    "planned path `{}` must stay repo-relative and must not contain `..` or absolute prefixes",
                    relative_path.display()
                )));
            };
            let next = current.join(part);
            match fs::symlink_metadata(&next) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        return Err(WorkspaceMutationError::Validation(format!(
                            "planned path `{}` traverses symlinked component `{}`",
                            relative_path.display(),
                            next.strip_prefix(&self.root).unwrap_or(&next).display()
                        )));
                    }
                    if !metadata.is_dir() {
                        return Err(WorkspaceMutationError::Validation(format!(
                            "planned path `{}` traverses non-directory component `{}`",
                            relative_path.display(),
                            next.strip_prefix(&self.root).unwrap_or(&next).display()
                        )));
                    }
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    fs::create_dir(&next).map_err(|io_err| {
                        WorkspaceMutationError::Internal(format!(
                            "create {}: {io_err}",
                            next.display()
                        ))
                    })?;
                    created_dirs.push(next.clone());
                }
                Err(err) => {
                    return Err(WorkspaceMutationError::Internal(format!(
                        "stat {}: {err}",
                        next.display()
                    )));
                }
            }
            current = next;
        }
        Ok(())
    }
}

#[path = "workspace_mutation/delivery.rs"]
mod delivery;
pub use delivery::apply_mutations;

/// Only owning routes which independently admit their whole proposal may use this boundary.
pub(crate) fn apply_admitted_mutations(
    workspace_root: &Path,
    mutations: &[PlannedMutation],
) -> Result<ApplySummary, WorkspaceMutationError> {
    apply_with_hook(workspace_root, mutations, |_| Ok(()))
}

fn apply_with_hook(
    workspace_root: &Path,
    mutations: &[PlannedMutation],
    mut before_write: impl FnMut(usize) -> Result<(), WorkspaceMutationError>,
) -> Result<ApplySummary, WorkspaceMutationError> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    let mut summary = ApplySummary {
        total: mutations.len(),
        ..ApplySummary::default()
    };

    let mut prepared = Vec::new();
    for mutation in mutations {
        let classified = classify_mutation(&jail, mutation)?;
        match classified.state {
            MutationState::Identical => {
                summary.identical += 1;
            }
            MutationState::Absent => {
                prepared.push(PreparedMutation {
                    temp_path: PathBuf::new(),
                    classified,
                });
            }
            MutationState::Divergent => {
                return Err(WorkspaceMutationError::Validation(format!(
                    "planned write `{}` is divergent; refusing to overwrite unexpected contents",
                    mutation.relative_path().display()
                )));
            }
        }
    }

    if prepared.is_empty() {
        return Ok(summary);
    }

    let temp_root = create_temp_root(&jail.canonical_root)?;
    let mut created_dirs = Vec::new();
    let mut applied = Vec::new();
    let mut rollback_needed = false;

    let result = (|| -> Result<(), WorkspaceMutationError> {
        for (index, mutation) in prepared.iter_mut().enumerate() {
            let temp_path = temp_root.join(format!("{index:04}.tmp"));
            fs::write(&temp_path, &mutation.classified.mutation.desired_after).map_err(|err| {
                WorkspaceMutationError::Internal(format!("write {}: {err}", temp_path.display()))
            })?;
            mutation.temp_path = temp_path;
        }

        for (index, prepared_mutation) in prepared.iter().enumerate() {
            before_write(index)?;
            let refreshed = classify_mutation(&jail, prepared_mutation.classified.mutation)?;
            match refreshed.state {
                MutationState::Identical => {
                    summary.identical += 1;
                }
                MutationState::Divergent => {
                    return Err(WorkspaceMutationError::Validation(format!(
                        "planned write `{}` became divergent before apply",
                        prepared_mutation
                            .classified
                            .mutation
                            .relative_path()
                            .display()
                    )));
                }
                MutationState::Absent => {
                    jail.ensure_parent_dirs(
                        prepared_mutation.classified.mutation.relative_path(),
                        &mut created_dirs,
                    )?;
                    let applied_mutation = apply_one(
                        &refreshed.absolute_path,
                        prepared_mutation.classified.mutation,
                        &prepared_mutation.temp_path,
                        &temp_root.join(format!("{index:04}.bak")),
                    )?;
                    applied.push(applied_mutation);
                    summary.written += 1;
                }
            }
        }

        Ok(())
    })();

    if result.is_err() {
        rollback_needed = true;
    }

    if rollback_needed {
        rollback(&jail, &applied, &created_dirs)?;
    }

    // A failed immediate restoration may leave the original in a backup. Preserve it for
    // recovery instead of deleting it with staging files.
    if let Err(error) = result {
        let has_backup = fs::read_dir(&temp_root)
            .map_err(|e| WorkspaceMutationError::Internal(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| WorkspaceMutationError::Internal(e.to_string()))?
            .iter()
            .any(|entry| entry.path().extension().is_some_and(|ext| ext == "bak"));
        if !has_backup {
            cleanup_temp_root(&temp_root)?;
        }
        return Err(error);
    }
    cleanup_temp_root(&temp_root)?;
    Ok(summary)
}

pub fn plan_create_or_replace(
    jail: &WorkspacePathJail,
    relative_path: impl Into<PathBuf>,
    desired_after: Vec<u8>,
) -> Result<PlannedMutation, WorkspaceMutationError> {
    let relative_path = relative_path.into();
    let absolute_path = jail.resolve(&relative_path)?;
    match fs::symlink_metadata(&absolute_path) {
        Ok(metadata) if metadata.file_type().is_file() && !metadata.file_type().is_symlink() => {
            let current = fs::read(&absolute_path).map_err(|err| {
                WorkspaceMutationError::Internal(format!("read {}: {err}", absolute_path.display()))
            })?;
            Ok(PlannedMutation::replace(
                relative_path,
                current,
                desired_after,
            ))
        }
        Ok(_) => Ok(PlannedMutation::create(relative_path, desired_after)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Ok(PlannedMutation::create(relative_path, desired_after))
        }
        Err(err) => Err(WorkspaceMutationError::Internal(format!(
            "stat {}: {err}",
            absolute_path.display()
        ))),
    }
}

fn create_temp_root(canonical_root: &Path) -> Result<PathBuf, WorkspaceMutationError> {
    let epoch_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| {
            WorkspaceMutationError::Internal(format!("system time before unix epoch: {err}"))
        })?
        .as_nanos();

    for attempt in 0..1024u32 {
        let temp_root = canonical_root.join(format!(
            ".xtask-onboard-agent-{}-{}-{attempt}",
            std::process::id(),
            epoch_nanos
        ));
        match fs::create_dir(&temp_root) {
            Ok(()) => return Ok(temp_root),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(WorkspaceMutationError::Internal(format!(
                    "create {}: {err}",
                    temp_root.display()
                )))
            }
        }
    }

    Err(WorkspaceMutationError::Internal(format!(
        "allocate unique temp root under {} after 1024 attempts",
        canonical_root.display()
    )))
}

fn cleanup_temp_root(temp_root: &Path) -> Result<(), WorkspaceMutationError> {
    if temp_root.exists() {
        fs::remove_dir_all(temp_root).map_err(|err| {
            WorkspaceMutationError::Internal(format!("remove {}: {err}", temp_root.display()))
        })?;
    }
    Ok(())
}

#[cfg(unix)]
fn file_mode(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o7777
}
#[cfg(not(unix))]
fn file_mode(metadata: &fs::Metadata) -> u32 {
    u32::from(metadata.permissions().readonly())
}

fn classify_mutation<'a>(
    jail: &WorkspacePathJail,
    mutation: &'a PlannedMutation,
) -> Result<ClassifiedMutation<'a>, WorkspaceMutationError> {
    let absolute_path = jail.resolve(mutation.relative_path())?;
    let state = match fs::symlink_metadata(&absolute_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || mutation
                    .expected_mode
                    .is_some_and(|m| file_mode(&metadata) != m)
            {
                MutationState::Divergent
            } else {
                let current = fs::read(&absolute_path).map_err(|err| {
                    WorkspaceMutationError::Internal(format!(
                        "read {}: {err}",
                        absolute_path.display()
                    ))
                })?;
                if !mutation.remove && current == mutation.desired_after {
                    MutationState::Identical
                } else if mutation
                    .expected_before
                    .as_ref()
                    .is_some_and(|expected| current == *expected)
                {
                    MutationState::Absent
                } else {
                    MutationState::Divergent
                }
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            if mutation.remove {
                MutationState::Identical
            } else if mutation.expected_before.is_none() {
                MutationState::Absent
            } else {
                MutationState::Divergent
            }
        }
        Err(err) => {
            return Err(WorkspaceMutationError::Internal(format!(
                "stat {}: {err}",
                absolute_path.display()
            )));
        }
    };

    Ok(ClassifiedMutation {
        mutation,
        absolute_path,
        state,
    })
}

fn apply_one(
    absolute_path: &Path,
    mutation: &PlannedMutation,
    temp_path: &Path,
    backup_path: &Path,
) -> Result<AppliedMutation, WorkspaceMutationError> {
    if mutation.remove {
        fs::rename(absolute_path, backup_path)
            .map_err(|e| WorkspaceMutationError::Internal(e.to_string()))?;
        return Ok(AppliedMutation::Removed {
            path: absolute_path.to_owned(),
            backup_path: backup_path.to_owned(),
        });
    }
    if mutation.expected_before.is_some() {
        let permissions = fs::symlink_metadata(absolute_path)
            .map_err(|e| WorkspaceMutationError::Internal(e.to_string()))?
            .permissions();
        fs::set_permissions(temp_path, permissions.clone())
            .map_err(|e| WorkspaceMutationError::Internal(e.to_string()))?;
        fs::rename(absolute_path, backup_path).map_err(|err| {
            WorkspaceMutationError::Internal(format!(
                "rename {} -> {}: {err}",
                absolute_path.display(),
                backup_path.display()
            ))
        })?;
        if let Err(err) = fs::rename(temp_path, absolute_path) {
            if fs::symlink_metadata(absolute_path).is_ok() {
                return Err(WorkspaceMutationError::Internal(format!(
                    "restore conflict at {}; original retained at {}",
                    absolute_path.display(),
                    backup_path.display()
                )));
            }
            fs::rename(backup_path, absolute_path).map_err(|restore| {
                WorkspaceMutationError::Internal(format!(
                    "restore {}: {restore}; original apply error: {err}",
                    absolute_path.display()
                ))
            })?;
            return Err(WorkspaceMutationError::Internal(format!(
                "rename {} -> {}: {err}",
                temp_path.display(),
                absolute_path.display()
            )));
        }
        Ok(AppliedMutation::Replaced {
            path: absolute_path.to_path_buf(),
            backup_path: backup_path.to_path_buf(),
            written: mutation.desired_after.clone(),
            permissions,
        })
    } else {
        let permissions = fs::symlink_metadata(temp_path)
            .map_err(|e| WorkspaceMutationError::Internal(e.to_string()))?
            .permissions();
        fs::rename(temp_path, absolute_path).map_err(|err| {
            WorkspaceMutationError::Internal(format!(
                "rename {} -> {}: {err}",
                temp_path.display(),
                absolute_path.display()
            ))
        })?;
        Ok(AppliedMutation::Created {
            path: absolute_path.to_path_buf(),
            written: mutation.desired_after.clone(),
            permissions,
        })
    }
}

fn rollback(
    jail: &WorkspacePathJail,
    applied: &[AppliedMutation],
    created_dirs: &[PathBuf],
) -> Result<(), WorkspaceMutationError> {
    let mut errors = Vec::new();
    for mutation in applied.iter().rev() {
        let attempt = (|| -> Result<(), WorkspaceMutationError> {
            if let AppliedMutation::Removed { path, backup_path } = mutation {
                jail.resolve(
                    path.strip_prefix(&jail.canonical_root)
                        .map_err(|e| WorkspaceMutationError::Validation(e.to_string()))?,
                )?;
                match fs::symlink_metadata(path) {
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                    _ => {
                        return Err(WorkspaceMutationError::Validation(format!(
                        "rollback conflict at {}; preserving intervening change and recovery files",
                        path.display()
                    )))
                    }
                }
                fs::rename(backup_path, path)
                    .map_err(|e| WorkspaceMutationError::Internal(e.to_string()))?;
                return Ok(());
            }
            let (path, written, permissions) = match mutation {
                AppliedMutation::Removed { .. } => unreachable!(),
                AppliedMutation::Created {
                    path,
                    written,
                    permissions,
                }
                | AppliedMutation::Replaced {
                    path,
                    written,
                    permissions,
                    ..
                } => (path, written, permissions),
            };
            // Best-effort repair only: never overwrite an observed edit/type change made after
            // this invocation wrote the path. This comparison is not cross-route serialization.
            let relative = path
                .strip_prefix(&jail.canonical_root)
                .map_err(|e| WorkspaceMutationError::Validation(e.to_string()))?;
            jail.resolve(relative)?;
            let unchanged = fs::symlink_metadata(path).is_ok_and(|m| {
                m.is_file() && !m.file_type().is_symlink() && &m.permissions() == permissions
            }) && fs::read(path).is_ok_and(|bytes| bytes == *written);
            if !unchanged {
                return Err(WorkspaceMutationError::Validation(format!(
                    "rollback conflict at {}; preserving intervening change and recovery files",
                    path.display()
                )));
            }
            match mutation {
                AppliedMutation::Removed { .. } => unreachable!(),
                AppliedMutation::Created { path, .. } => match fs::remove_file(path) {
                    Ok(()) => {}
                    Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                    Err(err) => {
                        return Err(WorkspaceMutationError::Internal(format!(
                            "rollback remove {}: {err}",
                            path.display()
                        )));
                    }
                },
                AppliedMutation::Replaced {
                    path, backup_path, ..
                } => {
                    match fs::remove_file(path) {
                        Ok(()) => {}
                        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                        Err(err) => {
                            return Err(WorkspaceMutationError::Internal(format!(
                                "rollback remove {}: {err}",
                                path.display()
                            )));
                        }
                    }
                    fs::rename(backup_path, path).map_err(|err| {
                        WorkspaceMutationError::Internal(format!(
                            "rollback rename {} -> {}: {err}",
                            backup_path.display(),
                            path.display()
                        ))
                    })?;
                }
            }
            Ok(())
        })();
        if let Err(e) = attempt {
            errors.push(e.to_string());
        }
    }

    for dir in created_dirs.iter().rev() {
        let attempt = (|| -> Result<(), WorkspaceMutationError> {
            let relative = dir
                .strip_prefix(&jail.canonical_root)
                .map_err(|e| WorkspaceMutationError::Validation(e.to_string()))?;
            jail.resolve(relative)?;
            match fs::remove_dir(dir) {
                Ok(()) => Ok(()),
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::NotFound | io::ErrorKind::DirectoryNotEmpty
                    ) =>
                {
                    Ok(())
                }
                Err(e) => Err(WorkspaceMutationError::Internal(format!(
                    "rollback remove {}: {e}",
                    dir.display()
                ))),
            }
        })();
        if let Err(e) = attempt {
            errors.push(e.to_string());
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(WorkspaceMutationError::Validation(errors.join("\n")))
    }
}

#[cfg(test)]
#[path = "workspace_mutation/tests.rs"]
mod tests;
