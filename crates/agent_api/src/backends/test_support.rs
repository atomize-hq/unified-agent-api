use std::{
    env,
    fs::{self, File, OpenOptions},
    io::ErrorKind,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

pub(crate) struct TestEnvLockGuard {
    path: PathBuf,
    _file: File,
}

pub(crate) fn test_env_lock() -> TestEnvLockGuard {
    let path = env::temp_dir().join(format!("agent-api-test-env-{}.lock", std::process::id()));

    loop {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                return TestEnvLockGuard { path, _file: file };
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(err) => panic!("test env lock should be acquired: {err}"),
        }
    }
}

impl Drop for TestEnvLockGuard {
    fn drop(&mut self) {
        if let Err(err) = fs::remove_file(&self.path) {
            assert!(
                err.kind() == ErrorKind::NotFound,
                "test env lock should be released: {err}"
            );
        }
    }
}

pub(crate) struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    pub(crate) fn set(path: &Path) -> Self {
        let previous = env::current_dir().unwrap_or_else(|_| env::temp_dir());
        env::set_current_dir(path).expect("current dir should be set");
        Self { previous }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        if env::set_current_dir(&self.previous).is_err() {
            env::set_current_dir(env::temp_dir()).expect("fallback current dir should be restored");
        }
    }
}

/// Cargo profile directory (`target/<profile>`) holding the uplifted workspace binaries,
/// resolved from the running test executable's position in one of Cargo's two layouts:
/// legacy `<profile>/deps/<test>` or, from Rust 1.100, `<profile>/build/<pkg>/<hash>/out/<test>`.
/// A separately configured `build.build-dir` is not supported (the workspace sets none).
pub(crate) fn cargo_profile_dir() -> PathBuf {
    let current_exe = std::env::current_exe().expect("resolve current test binary path");
    let ancestor = |n: usize| current_exe.ancestors().nth(n);
    let named = |n: usize, name: &str| {
        ancestor(n)
            .and_then(Path::file_name)
            .is_some_and(|file| file == name)
    };
    let profile = if named(1, "deps") {
        ancestor(2)
    } else if named(1, "out") && named(4, "build") {
        ancestor(5)
    } else {
        None
    };
    profile.map(Path::to_path_buf).unwrap_or_else(|| {
        panic!(
            "test binary {current_exe:?} is not in a Cargo deps/ or build/<pkg>/<hash>/out/ layout"
        )
    })
}
