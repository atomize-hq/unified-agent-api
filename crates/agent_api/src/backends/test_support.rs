use std::{
    env,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

pub(crate) fn test_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    pub(crate) fn set(path: &Path) -> Self {
        let previous = env::current_dir().expect("current dir should be readable");
        env::set_current_dir(path).expect("current dir should be set");
        Self { previous }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        env::set_current_dir(&self.previous).expect("current dir should be restored");
    }
}
