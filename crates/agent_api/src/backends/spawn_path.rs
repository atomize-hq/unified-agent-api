use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

pub(super) fn resolve_binary_path_for_spawn(
    binary_path: PathBuf,
    effective_path_env: Option<&str>,
    ambient_path_env: Option<OsString>,
    current_dir: Option<&Path>,
) -> Option<PathBuf> {
    if is_path_qualified(&binary_path) {
        return Some(resolve_path_qualified_binary(binary_path, current_dir));
    }

    if let Some(path_env) = effective_path_env {
        return find_binary_on_path(&binary_path, Some(OsString::from(path_env)));
    }

    find_binary_on_path(&binary_path, ambient_path_env)
}

fn resolve_path_qualified_binary(binary_path: PathBuf, current_dir: Option<&Path>) -> PathBuf {
    if binary_path.is_absolute() {
        return binary_path;
    }

    let joined = current_dir
        .map(Path::to_path_buf)
        .or_else(|| env::current_dir().ok())
        .map(|cwd| cwd.join(&binary_path))
        .unwrap_or(binary_path);

    fs::canonicalize(&joined).unwrap_or(joined)
}

fn is_path_qualified(path: &Path) -> bool {
    path.is_absolute()
        || path
            .parent()
            .is_some_and(|parent| !parent.as_os_str().is_empty())
}

fn find_binary_on_path(binary_name: &Path, path_env: Option<OsString>) -> Option<PathBuf> {
    let path_env = path_env?;
    env::split_paths(&path_env)
        .find_map(|directory| candidate_binary_path(&directory, binary_name))
        .map(|candidate| fs::canonicalize(&candidate).unwrap_or(candidate))
}

fn candidate_binary_path(directory: &Path, binary_name: &Path) -> Option<PathBuf> {
    let candidate = directory.join(binary_name);
    if candidate.is_file() {
        return Some(candidate);
    }

    #[cfg(windows)]
    {
        if candidate.extension().is_some() {
            return None;
        }

        let pathext =
            env::var_os("PATHEXT").unwrap_or_else(|| OsString::from(".COM;.EXE;.BAT;.CMD"));
        for extension in pathext.to_string_lossy().split(';') {
            let extension = extension.trim();
            if extension.is_empty() {
                continue;
            }

            let suffixed = candidate.with_extension(extension.trim_start_matches('.'));
            if suffixed.is_file() {
                return Some(suffixed);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use tempfile::TempDir;

    use super::resolve_binary_path_for_spawn;

    #[test]
    fn resolves_relative_path_qualified_binary_against_supplied_current_dir() {
        let temp_dir = TempDir::new().expect("temp dir");
        let binary_dir = temp_dir.path().join("bin");
        let binary_path = binary_dir.join("codex");
        std::fs::create_dir_all(&binary_dir).expect("create binary dir");
        std::fs::write(&binary_path, "binary").expect("write fake binary");

        let resolved = resolve_binary_path_for_spawn(
            PathBuf::from("bin/codex"),
            None,
            None,
            Some(temp_dir.path()),
        )
        .expect("relative qualified path should resolve");

        assert_eq!(
            resolved,
            std::fs::canonicalize(&binary_path).expect("canonicalize binary")
        );
        assert!(resolved.is_absolute());
    }

    #[test]
    fn resolves_bare_binary_from_path_before_current_dir_joining() {
        let path_dir = TempDir::new().expect("path dir");
        let cwd_dir = TempDir::new().expect("cwd dir");
        let binary_path = path_dir.path().join("codex");
        std::fs::write(&binary_path, "binary").expect("write fake binary");

        let resolved = resolve_binary_path_for_spawn(
            PathBuf::from("codex"),
            Some(path_dir.path().to_string_lossy().as_ref()),
            env::var_os("PATH"),
            Some(cwd_dir.path()),
        )
        .expect("PATH lookup should resolve");

        assert_eq!(
            resolved,
            std::fs::canonicalize(&binary_path).expect("canonicalize binary")
        );
    }
}
