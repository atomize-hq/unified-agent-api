// Shared by the integration targets with `include!` (not `mod`): depth_ci_core loads the other
// targets as modules, and a file loaded as several modules trips clippy::duplicate_mod.

/// Cargo profile directory (`target/<profile>`) holding the uplifted workspace binaries,
/// resolved from the running test executable's position in one of Cargo's two layouts:
/// legacy `<profile>/deps/<test>` or, from Rust 1.100, `<profile>/build/<pkg>/<hash>/out/<test>`.
/// A separately configured `build.build-dir` is not supported (the workspace sets none).
fn cargo_profile_dir() -> std::path::PathBuf {
    let current_exe = std::env::current_exe().expect("resolve current test binary path");
    let ancestor = |n: usize| current_exe.ancestors().nth(n);
    let named = |n: usize, name: &str| {
        ancestor(n)
            .and_then(std::path::Path::file_name)
            .is_some_and(|file| file == name)
    };
    let profile = if named(1, "deps") {
        ancestor(2)
    } else if named(1, "out") && named(4, "build") {
        ancestor(5)
    } else {
        None
    };
    profile.map(std::path::Path::to_path_buf).unwrap_or_else(|| {
        panic!(
            "test binary {current_exe:?} is not in a Cargo deps/ or build/<pkg>/<hash>/out/ layout"
        )
    })
}
