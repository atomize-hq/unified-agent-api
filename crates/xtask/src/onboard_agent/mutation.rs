use std::{
    fs, io,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::Error;

#[derive(Debug, Clone)]
pub(super) struct PlannedMutation {
    relative_path: PathBuf,
    expected_before: Option<Vec<u8>>,
    desired_after: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MutationState {
    Absent,
    Identical,
    Divergent,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct ApplySummary {
    pub(super) total: usize,
    pub(super) written: usize,
    pub(super) identical: usize,
}

#[derive(Debug)]
pub(super) struct WorkspacePathJail {
    root: PathBuf,
    canonical_root: PathBuf,
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
    CreatedFile(PathBuf),
    ReplacedFile { path: PathBuf, backup_path: PathBuf },
}

impl PlannedMutation {
    pub(super) fn create(relative_path: impl Into<PathBuf>, desired_after: Vec<u8>) -> Self {
        Self {
            relative_path: relative_path.into(),
            expected_before: None,
            desired_after,
        }
    }

    pub(super) fn replace(
        relative_path: impl Into<PathBuf>,
        expected_before: Vec<u8>,
        desired_after: Vec<u8>,
    ) -> Self {
        Self {
            relative_path: relative_path.into(),
            expected_before: Some(expected_before),
            desired_after,
        }
    }

    pub(super) fn relative_path(&self) -> &Path {
        &self.relative_path
    }
}

impl WorkspacePathJail {
    pub(super) fn new(root: &Path) -> Result<Self, Error> {
        let canonical_root = fs::canonicalize(root)
            .map_err(|err| Error::Internal(format!("canonicalize {}: {err}", root.display())))?;
        Ok(Self {
            root: root.to_path_buf(),
            canonical_root,
        })
    }

    pub(super) fn resolve(&self, relative_path: &Path) -> Result<PathBuf, Error> {
        let mut components = relative_path.components().peekable();
        if components.peek().is_none() {
            return Err(Error::Validation(
                "planned path must not be empty".to_string(),
            ));
        }

        let mut current_lexical = self.root.clone();
        let mut current_canonical = self.canonical_root.clone();

        while let Some(component) = components.next() {
            let Component::Normal(part) = component else {
                return Err(Error::Validation(format!(
                    "planned path `{}` must stay repo-relative and must not contain `..` or absolute prefixes",
                    relative_path.display()
                )));
            };

            current_lexical.push(part);
            match fs::symlink_metadata(&current_lexical) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        return Err(Error::Validation(format!(
                            "planned path `{}` traverses symlinked component `{}`",
                            relative_path.display(),
                            current_lexical
                                .strip_prefix(&self.root)
                                .unwrap_or(&current_lexical)
                                .display()
                        )));
                    }
                    if components.peek().is_some() && !metadata.is_dir() {
                        return Err(Error::Validation(format!(
                            "planned path `{}` traverses non-directory component `{}`",
                            relative_path.display(),
                            current_lexical
                                .strip_prefix(&self.root)
                                .unwrap_or(&current_lexical)
                                .display()
                        )));
                    }
                    let canonical = fs::canonicalize(&current_lexical).map_err(|err| {
                        Error::Internal(format!(
                            "canonicalize {}: {err}",
                            current_lexical.display()
                        ))
                    })?;
                    if !canonical.starts_with(&self.canonical_root) {
                        return Err(Error::Validation(format!(
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
                            return Err(Error::Validation(format!(
                                "planned path `{}` must stay repo-relative and must not contain `..` or absolute prefixes",
                                relative_path.display()
                            )));
                        };
                        unresolved.push(next_part);
                    }
                    return Ok(current_canonical.join(unresolved));
                }
                Err(err) => {
                    return Err(Error::Internal(format!(
                        "stat {}: {err}",
                        current_lexical.display()
                    )));
                }
            }
        }

        Ok(current_canonical)
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    fn ensure_parent_dirs(
        &self,
        relative_path: &Path,
        created_dirs: &mut Vec<PathBuf>,
    ) -> Result<(), Error> {
        let Some(parent) = relative_path.parent() else {
            return Ok(());
        };

        let mut current = self.canonical_root.clone();
        for component in parent.components() {
            let Component::Normal(part) = component else {
                return Err(Error::Validation(format!(
                    "planned path `{}` must stay repo-relative and must not contain `..` or absolute prefixes",
                    relative_path.display()
                )));
            };
            let next = current.join(part);
            match fs::symlink_metadata(&next) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        return Err(Error::Validation(format!(
                            "planned path `{}` traverses symlinked component `{}`",
                            relative_path.display(),
                            next.strip_prefix(&self.root).unwrap_or(&next).display()
                        )));
                    }
                    if !metadata.is_dir() {
                        return Err(Error::Validation(format!(
                            "planned path `{}` traverses non-directory component `{}`",
                            relative_path.display(),
                            next.strip_prefix(&self.root).unwrap_or(&next).display()
                        )));
                    }
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    fs::create_dir(&next).map_err(|io_err| {
                        Error::Internal(format!("create {}: {io_err}", next.display()))
                    })?;
                    created_dirs.push(next.clone());
                }
                Err(err) => {
                    return Err(Error::Internal(format!("stat {}: {err}", next.display())));
                }
            }
            current = next;
        }
        Ok(())
    }
}

pub(super) fn apply_mutations(
    workspace_root: &Path,
    mutations: &[PlannedMutation],
) -> Result<ApplySummary, Error> {
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
                return Err(Error::Validation(format!(
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

    let result = (|| -> Result<(), Error> {
        for (index, mutation) in prepared.iter_mut().enumerate() {
            let temp_path = temp_root.join(format!("{index:04}.tmp"));
            fs::write(&temp_path, &mutation.classified.mutation.desired_after)
                .map_err(|err| Error::Internal(format!("write {}: {err}", temp_path.display())))?;
            mutation.temp_path = temp_path;
        }

        for (index, prepared_mutation) in prepared.iter().enumerate() {
            let refreshed = classify_mutation(&jail, prepared_mutation.classified.mutation)?;
            match refreshed.state {
                MutationState::Identical => {
                    summary.identical += 1;
                }
                MutationState::Divergent => {
                    return Err(Error::Validation(format!(
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
        rollback(&applied, &created_dirs)?;
    }

    cleanup_temp_root(&temp_root)?;
    result?;
    Ok(summary)
}

fn create_temp_root(canonical_root: &Path) -> Result<PathBuf, Error> {
    let unique = format!(
        ".xtask-onboard-agent-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| Error::Internal(format!("system time before unix epoch: {err}")))?
            .as_nanos()
    );
    let temp_root = canonical_root.join(unique);
    fs::create_dir(&temp_root)
        .map_err(|err| Error::Internal(format!("create {}: {err}", temp_root.display())))?;
    Ok(temp_root)
}

fn cleanup_temp_root(temp_root: &Path) -> Result<(), Error> {
    if temp_root.exists() {
        fs::remove_dir_all(temp_root)
            .map_err(|err| Error::Internal(format!("remove {}: {err}", temp_root.display())))?;
    }
    Ok(())
}

fn classify_mutation<'a>(
    jail: &WorkspacePathJail,
    mutation: &'a PlannedMutation,
) -> Result<ClassifiedMutation<'a>, Error> {
    let absolute_path = jail.resolve(mutation.relative_path())?;
    let state = match fs::symlink_metadata(&absolute_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                MutationState::Divergent
            } else {
                let current = fs::read(&absolute_path).map_err(|err| {
                    Error::Internal(format!("read {}: {err}", absolute_path.display()))
                })?;
                if current == mutation.desired_after {
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
            if mutation.expected_before.is_none() {
                MutationState::Absent
            } else {
                MutationState::Divergent
            }
        }
        Err(err) => {
            return Err(Error::Internal(format!(
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
) -> Result<AppliedMutation, Error> {
    if mutation.expected_before.is_some() {
        fs::rename(absolute_path, backup_path).map_err(|err| {
            Error::Internal(format!(
                "rename {} -> {}: {err}",
                absolute_path.display(),
                backup_path.display()
            ))
        })?;
        if let Err(err) = fs::rename(temp_path, absolute_path) {
            let _ = fs::rename(backup_path, absolute_path);
            return Err(Error::Internal(format!(
                "rename {} -> {}: {err}",
                temp_path.display(),
                absolute_path.display()
            )));
        }
        Ok(AppliedMutation::ReplacedFile {
            path: absolute_path.to_path_buf(),
            backup_path: backup_path.to_path_buf(),
        })
    } else {
        fs::rename(temp_path, absolute_path).map_err(|err| {
            Error::Internal(format!(
                "rename {} -> {}: {err}",
                temp_path.display(),
                absolute_path.display()
            ))
        })?;
        Ok(AppliedMutation::CreatedFile(absolute_path.to_path_buf()))
    }
}

fn rollback(applied: &[AppliedMutation], created_dirs: &[PathBuf]) -> Result<(), Error> {
    for mutation in applied.iter().rev() {
        match mutation {
            AppliedMutation::CreatedFile(path) => match fs::remove_file(path) {
                Ok(()) => {}
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(Error::Internal(format!(
                        "rollback remove {}: {err}",
                        path.display()
                    )));
                }
            },
            AppliedMutation::ReplacedFile { path, backup_path } => {
                match fs::remove_file(path) {
                    Ok(()) => {}
                    Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                    Err(err) => {
                        return Err(Error::Internal(format!(
                            "rollback remove {}: {err}",
                            path.display()
                        )));
                    }
                }
                fs::rename(backup_path, path).map_err(|err| {
                    Error::Internal(format!(
                        "rollback rename {} -> {}: {err}",
                        backup_path.display(),
                        path.display()
                    ))
                })?;
            }
        }
    }

    for dir in created_dirs.iter().rev() {
        match fs::remove_dir(dir) {
            Ok(()) => {}
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::DirectoryNotEmpty
                ) => {}
            Err(err) => {
                return Err(Error::Internal(format!(
                    "rollback remove {}: {err}",
                    dir.display()
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_rejects_parent_dir_escape() {
        let root = temp_root();
        let jail = WorkspacePathJail::new(&root).expect("build jail");
        let err = jail
            .resolve(Path::new("../escape.txt"))
            .expect_err("parent-dir escape should fail");
        assert!(err.to_string().contains("must stay repo-relative"));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = temp_root();
        let outside = temp_root();
        symlink(&outside, root.join("linked")).expect("create symlink");

        let jail = WorkspacePathJail::new(&root).expect("build jail");
        let err = jail
            .resolve(Path::new("linked/file.txt"))
            .expect_err("symlink escape should fail");
        assert!(err.to_string().contains("symlinked component"));
    }

    fn temp_root() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "xtask-onboard-agent-mutation-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time after unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create temp root");
        root
    }
}
