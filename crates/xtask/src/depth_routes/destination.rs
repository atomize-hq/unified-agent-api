//! Resolve the effective file destination and bind local delivery to that checked path.
use super::{refuse_proposed, refuse_workspace};
use std::{
    collections::VecDeque,
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

/// Resolve existing aliases with a bounded link count. Missing suffixes are normalized against
/// the resolved ancestor, so `missing/../nested` still observes an existing nested scope.
fn resolved_output(path: &Path) -> Result<PathBuf, String> {
    if directory_syntax(path) {
        return Err("output destination requires a directory".into());
    }
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let mut pending: VecDeque<_> = absolute
        .components()
        .map(|p| p.as_os_str().to_owned())
        .collect();
    let mut resolved = PathBuf::new();
    let mut links = 0;
    while let Some(part) = pending.pop_front() {
        match Path::new(&part)
            .components()
            .next()
            .ok_or("empty output component")?
        {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => resolved.push(part),
            Component::Normal(_) => {
                let next = resolved.join(&part);
                match fs::symlink_metadata(&next) {
                    Ok(metadata) if metadata.file_type().is_symlink() => {
                        links += 1;
                        if links > 40 {
                            return Err("too many output symlinks".into());
                        }
                        let target = fs::read_link(&next).map_err(|e| e.to_string())?;
                        if pending.is_empty() && directory_syntax(&target) {
                            return Err("output destination requires a directory".into());
                        }
                        for component in target.components().rev() {
                            pending.push_front(component.as_os_str().to_owned());
                        }
                        if target.is_absolute() {
                            resolved.clear();
                        }
                    }
                    Ok(metadata) => {
                        if !pending.is_empty() && !metadata.is_dir() {
                            return Err("unsupported output ancestor type".into());
                        }
                        resolved = next;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => resolved = next,
                    Err(error) => return Err(error.to_string()),
                }
            }
        }
    }
    let ancestor = existing_parent(&resolved)?;
    let canonical = ancestor.canonicalize().map_err(|e| e.to_string())?;
    Ok(canonical.join(
        resolved
            .strip_prefix(&ancestor)
            .map_err(|e| e.to_string())?,
    ))
}

// Components omit trailing separators and `/.`, which must retain directory-only semantics.
fn directory_syntax(path: &Path) -> bool {
    let bytes = path.as_os_str().as_encoded_bytes();
    bytes.ends_with(b"/")
        || bytes.ends_with(b"/.")
        || cfg!(windows) && (bytes.ends_with(b"\\") || bytes.ends_with(b"\\."))
}

fn existing_parent(path: &Path) -> Result<PathBuf, String> {
    for ancestor in path.parent().ok_or("output has no parent")?.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.is_dir() => return Ok(ancestor.to_owned()),
            Ok(_) => return Err("unsupported output ancestor type".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("unresolved output destination".into())
}

fn check_resolved(full: &Path, bytes: &[u8], route: &str) -> Result<(), String> {
    // Preserve the enclosing workspace's unsupported-depth refusal before proposed bytes.
    refuse_workspace(&existing_parent(full)?, route)?;
    refuse_proposed(full, bytes, route)
}

pub fn refuse_file_destination(path: &Path, bytes: &[u8], route: &str) -> Result<(), String> {
    check_resolved(&resolved_output(path)?, bytes, route)
}

/// Delivery rechecks the caller's aliases while opening only the validated canonical parent.
/// Directory and scope observations are invocation-local and rechecked before delivery.
pub fn write_file_destination(path: &Path, bytes: &[u8], route: &str) -> Result<(), String> {
    let resolved = resolved_output(path)?;
    let ancestor = existing_parent(&resolved)?;
    let before = fs::metadata(&ancestor).map_err(|e| e.to_string())?;
    check_resolved(&resolved, bytes, route)?;
    #[cfg(test)]
    BEFORE_WRITE.with(|hook| {
        if let Some(hook) = hook.borrow_mut().take() {
            hook();
        }
    });
    let unchanged = || -> Result<(), String> {
        if resolved_output(path)? != resolved
            || !same_directory(
                &before,
                &fs::metadata(&ancestor).map_err(|e| e.to_string())?,
            )
        {
            return Err("output destination changed after validation".into());
        }
        Ok(())
    };
    unchanged()?;
    let parent = resolved.parent().ok_or("output has no parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    if parent.canonicalize().map_err(|e| e.to_string())? != parent {
        return Err("output parent changed after validation".into());
    }
    let expected = fs::metadata(parent).map_err(|e| e.to_string())?;
    #[cfg(test)]
    BEFORE_OPEN.with(|hook| {
        if let Some(hook) = hook.borrow_mut().take() {
            hook();
        }
    });
    let (mut file, directory) = open_output(
        parent,
        resolved.file_name().ok_or("output has no filename")?,
    )?;
    #[cfg(test)]
    AFTER_OPEN.with(|hook| {
        if let Some(hook) = hook.borrow_mut().take() {
            hook();
        }
    });
    // The file opens without truncation. Check both directory identities after opening and
    // before changing bytes, including the original pre-creation ancestor and aliases.
    if !same_directory(&expected, &directory.metadata().map_err(|e| e.to_string())?)
        || !same_directory(&expected, &fs::metadata(parent).map_err(|e| e.to_string())?)
    {
        return Err("output parent changed after opening".into());
    }
    unchanged()?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("unsupported output file type".into());
    }
    file.set_len(0).map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())
}

#[cfg(unix)]
fn same_directory(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    before.dev() == after.dev() && before.ino() == after.ino() && after.is_dir()
}

#[cfg(unix)]
fn open_output(parent: &Path, name: &std::ffi::OsStr) -> Result<(fs::File, fs::File), String> {
    use rustix::fs::{open, openat, Mode, OFlags};
    #[cfg(target_os = "linux")]
    let directory_access = OFlags::PATH;
    #[cfg(not(target_os = "linux"))]
    let directory_access = OFlags::RDONLY;
    let directory = open(
        parent,
        directory_access | OFlags::DIRECTORY | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|e| format!("open checked parent: {e}"))?;
    let file = openat(
        &directory,
        name,
        OFlags::WRONLY | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::NONBLOCK,
        Mode::from_raw_mode(0o666),
    )
    .map_err(|e| format!("open checked output: {e}"))?;
    Ok((file.into(), directory.into()))
}

#[cfg(not(unix))]
fn same_directory(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    after.is_dir()
        && before
            .created()
            .is_ok_and(|created| after.created().is_ok_and(|value| value == created))
}

#[cfg(not(unix))]
fn open_output(parent: &Path, name: &std::ffi::OsStr) -> Result<(fs::File, fs::File), String> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(false);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_FLAG_OPEN_REPARSE_POINT is the Windows final-component no-follow flag.
        options.custom_flags(0x0020_0000);
    }
    let file = options.open(parent.join(name)).map_err(|e| e.to_string())?;
    if file
        .metadata()
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("unsupported output symlink".into());
    }
    let mut directory_options = fs::OpenOptions::new();
    directory_options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_FLAG_BACKUP_SEMANTICS permits opening a directory handle.
        directory_options.custom_flags(0x0200_0000 | 0x0020_0000);
    }
    Ok((
        file,
        directory_options.open(parent).map_err(|e| e.to_string())?,
    ))
}

#[cfg(test)]
thread_local! {
    pub(super) static BEFORE_OPEN: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
    pub(super) static AFTER_OPEN: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
    pub(super) static BEFORE_WRITE: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}
