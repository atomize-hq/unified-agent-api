use super::*;
use crate::{
    depth_evidence::{payload::Variable, CaseIdentity, CaseOutcome, CaseResult, ProvenanceKind},
    git_inputs::GitInputs,
    runtime_advertising::source::Source,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::PathBuf,
    process::Stdio,
};

struct BuildEnv {
    rustc: PathBuf,
    cargo: PathBuf,
    sysroot: PathBuf,
    cargo_home: PathBuf,
    home: tempfile::TempDir,
    target: tempfile::TempDir,
    root: PathBuf,
}
impl BuildEnv {
    fn new(root: &Path) -> Result<Self, String> {
        let which = |binary: &str| -> Result<PathBuf, String> {
            let bytes =
                output(Command::new("rustup").args(["which", "--toolchain", TOOLCHAIN, binary]))?;
            fs::canonicalize(
                std::str::from_utf8(&bytes)
                    .map_err(|e| e.to_string())?
                    .trim(),
            )
            .map_err(|e| e.to_string())
        };
        let cargo_home = std::env::var_os("CARGO_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cargo")
            });
        for path in [cargo_home.join("config"), cargo_home.join("config.toml")] {
            if fs::symlink_metadata(path).is_ok() {
                return Err("unsupported Cargo-home configuration".into());
            }
        }
        let rustc = which("rustc")?;
        let cargo = which("cargo")?;
        let raw = output(Command::new(&rustc).args(["--print", "sysroot"]))?;
        let sysroot = PathBuf::from(std::str::from_utf8(&raw).map_err(|e| e.to_string())?.trim());
        Ok(Self {
            rustc,
            cargo,
            sysroot,
            cargo_home: fs::canonicalize(cargo_home).map_err(|e| e.to_string())?,
            home: tempfile::tempdir().map_err(|e| e.to_string())?,
            target: tempfile::tempdir().map_err(|e| e.to_string())?,
            root: root.into(),
        })
    }
    fn command(&self, program: &Path) -> Command {
        let mut c = Command::new("/usr/bin/timeout");
        c.args(["--kill-after=5", "180"])
            .arg(program)
            .current_dir(&self.root)
            .env_clear()
            .env("HOME", self.home.path())
            .env("CARGO_HOME", &self.cargo_home)
            .env("CARGO_TARGET_DIR", self.target.path())
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.sysroot.join("bin").display()),
            )
            .env("RUSTC", &self.rustc)
            .env("RUSTUP_TOOLCHAIN", TOOLCHAIN)
            .env(
                "CARGO_ENCODED_RUSTFLAGS",
                format!(
                    "--remap-path-prefix={}=/depth-input\u{1f}--remap-path-prefix={}=/depth-target\u{1f}--remap-path-prefix={}=/depth-cargo\u{1f}-Cdebuginfo=0",
                    self.root.display(), self.target.path().display(), self.cargo_home.display()
                ),
            )
            .env("CARGO_NET_OFFLINE", "true")
            .env("CARGO_INCREMENTAL", "0")
            .env("CARGO_BUILD_JOBS","2")
            .env("SOURCE_DATE_EPOCH", "0")
            .env("LANG", "C")
            .env("LC_ALL", "C")
            .env("TZ", "UTC");
        c
    }
}
fn capture(c: &mut Command, limit: u64) -> Result<(bool, String), String> {
    let mut child = c
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut bytes = vec![];
    child
        .stdout
        .take()
        .unwrap()
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        let _ = child.kill();
        let _ = child.wait();
        return Err("producer output bounds exceeded".into());
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    Ok((
        status.success(),
        String::from_utf8(bytes).map_err(|e| e.to_string())?,
    ))
}
fn required(c: &mut Command, limit: u64) -> Result<String, String> {
    let (ok, raw) = capture(c, limit)?;
    if !ok {
        return Err("isolated producer command failed".into());
    }
    Ok(raw)
}
fn identity(path: &Path) -> Result<String, String> {
    Ok(reference("", &fs::read(path).map_err(|e| e.to_string())?).sha256)
}
fn fingerprint(path: &Path, base: &Path, rows: &mut Vec<(String, String)>) -> Result<(), String> {
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            fingerprint(&entry.path(), base, rows)?
        } else if kind.is_file() {
            rows.push((
                entry
                    .path()
                    .strip_prefix(base)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .into_owned(),
                identity(&entry.path())?,
            ));
        } else {
            return Err("unsupported toolchain/output symlink or file type".into());
        }
        if rows.len() > 4096 {
            return Err("toolchain/output inventory bounds exceeded".into());
        }
    }
    Ok(())
}
fn toolchain(env: &BuildEnv, executable: &Path, scripts: &[Value]) -> Result<String, String> {
    let mut standard = vec![];
    let stdlib = env.sysroot.join("lib/rustlib").join(TARGET).join("lib");
    fingerprint(&stdlib, &stdlib, &mut standard)?;
    standard.sort();
    let version = required(env.command(&env.rustc).arg("-Vv"), 65536)?;
    if !version
        .lines()
        .any(|l| l == format!("release: {TOOLCHAIN}"))
    {
        return Err("unrecognized Rust release".into());
    }
    let mut native = BTreeMap::new();
    for tool in ["cc", "ld"] {
        let path = PathBuf::from("/usr/bin").join(tool);
        native.insert(tool.to_owned(),json!({"sha256":identity(&fs::canonicalize(&path).map_err(|e|e.to_string())?)?,"version":required(env.command(&path).arg("--version"),65536)?}));
    }
    let libraries = required(
        env.command(Path::new("/usr/bin/ldd")).arg(executable),
        65536,
    )?;
    let mut linked = BTreeMap::new();
    for word in libraries.split_whitespace().filter(|s| s.starts_with('/')) {
        let path = Path::new(word);
        linked.insert(
            path.file_name()
                .ok_or("invalid linked library")?
                .to_string_lossy()
                .into_owned(),
            identity(path)?,
        );
    }
    let mut generated = vec![];
    for script in scripts {
        if !script["linked_libs"].as_array().is_some_and(Vec::is_empty)
            || !script["linked_paths"].as_array().is_some_and(Vec::is_empty)
        {
            return Err("native build script linkage is unsupported".into());
        }
        let out = Path::new(
            script["out_dir"]
                .as_str()
                .ok_or("missing build script outputs")?,
        );
        let mut files = vec![];
        fingerprint(out, out, &mut files)?;
        files.sort();
        generated.push(json!({"package_id":script["package_id"],"cfgs":script["cfgs"],"env":script["env"],"files":files}));
    }
    generated.sort_by_key(Value::to_string);
    serde_json::to_string(&json!({"rustc":version,"rustc_sha256":identity(&env.rustc)?,"cargo":required(env.command(&env.cargo).arg("-Vv"),65536)?,"cargo_sha256":identity(&env.cargo)?,"standard_library":standard,"native_tools":native,"runtime_libraries":linked,"build_scripts":generated})).map_err(|e|e.to_string())
}
fn execute(
    env: &BuildEnv,
    executable: &Path,
    names: &[String],
    exact_population: bool,
) -> Result<(Cases, String), String> {
    let listed = required(
        env.command(executable)
            .args(["--list", "--format", "terse"]),
        1048576,
    )?;
    let listed: BTreeSet<_> = listed
        .lines()
        .filter_map(|s| s.strip_suffix(": test"))
        .collect();
    let expected: BTreeSet<_> = names.iter().map(String::as_str).collect();
    if !expected.is_subset(&listed) || (exact_population && listed != expected) {
        return Err("unrecognized test executable population".into());
    }
    let digest = identity(executable)?;
    let requested: Vec<_> = names
        .iter()
        .map(|name| CaseIdentity {
            name: name.clone(),
            executable_sha256: digest.clone(),
        })
        .collect();
    let mut raw = String::new();
    let mut executed = vec![];
    for case in &requested {
        let (success, output) = capture(
            env.command(executable).args([
                "--exact",
                &case.name,
                "--format",
                "pretty",
                "--color",
                "never",
                "--test-threads",
                "1",
            ]),
            1048576,
        )?;
        let outcome = if success
            && output.contains(&format!("test {} ... ok", case.name))
            && output.contains(&format!(
                "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; {} filtered out",
                listed.len() - 1
            )) {
            CaseOutcome::Passed
        } else if !success
            && output.contains(&format!("test {} ... FAILED", case.name))
            && output.contains(&format!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; {} filtered out",
                listed.len() - 1
            ))
        {
            CaseOutcome::Failed
        } else if output.contains("1 ignored;") {
            CaseOutcome::Ignored
        } else if output.contains("running 0 tests") {
            CaseOutcome::Skipped
        } else {
            CaseOutcome::Aborted
        };
        if output.matches("test result:").count() != 1 {
            return Err("unrecognized harness output".into());
        }
        raw.push_str(&format!("=== {} ===\n{}", case.name, output));
        executed.push(CaseResult {
            case: case.clone(),
            outcome,
        });
    }
    let raw_output = reference("", raw.as_bytes());
    Ok((
        Cases {
            requested,
            executed,
            raw_output,
        },
        raw,
    ))
}
fn witness(root: &Path, profile: profile::Profile) -> Result<Witness, String> {
    let (run_id, run_attempt, workflow_commit) = host_header()?;
    let mut view = GitInputs::new(root)?;
    let original = profile.record(
        std::str::from_utf8(&view.required(profile.fixture())?).map_err(|e| e.to_string())?,
    )?;
    if original.execution.is_some() || !original.acceptance_entries.is_empty() {
        return Err("source fixture must contain only frozen P/O".into());
    }
    let commit = String::from_utf8(output(
        Command::new("git")
            .arg("--no-replace-objects")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "HEAD"]),
    )?)
    .map_err(|e| e.to_string())?
    .trim()
    .to_owned();
    let source = Source::tip(root, &commit)?;
    let mut absent = [
        ".cargo/config",
        ".cargo/config.toml",
        "rust-toolchain",
        "rust-toolchain.toml",
    ]
    .map(str::to_owned)
    .to_vec();
    if profile == profile::Profile::Channel {
        absent.push("crates/wrapper_events/build.rs".into());
    } else {
        absent.extend(
            [
                "crates/agent_api/build.rs",
                "crates/codex/build.rs",
                "crates/wrapper_events/build.rs",
                "crates/claude_code/build.rs",
                "crates/aider/build.rs",
                "crates/gemini_cli/build.rs",
                "crates/opencode/build.rs",
            ]
            .map(str::to_owned),
        );
    }
    for path in &absent {
        if view.optional(path)?.is_some() || source.optional_bytes(path)?.is_some() {
            return Err("initial producer does not support additional build configuration".into());
        }
    }
    // Ancestor Cargo config can alter an otherwise clean checkout.
    for parent in root.ancestors().skip(1) {
        for name in [".cargo/config", ".cargo/config.toml"] {
            if fs::symlink_metadata(parent.join(name)).is_ok() {
                return Err("unsupported ancestor Cargo configuration".into());
            }
        }
    }
    let env = BuildEnv::new(root)?;
    let mut metadata: Value = serde_json::from_str(&required(
        env.command(&env.cargo)
            .args([
                "metadata",
                "--format-version",
                "1",
                "--locked",
                "--offline",
                "--no-default-features",
                "--filter-platform",
                TARGET,
            ])
            .args(if profile == profile::Profile::Core {
                vec!["--all-features"]
            } else {
                vec![]
            }),
        16777216,
    )?)
    .map_err(|e| e.to_string())?;
    if profile == profile::Profile::Core {
        let tree = required(
            env.command(&env.cargo).args([
                "tree",
                "--locked",
                "--offline",
                "--no-default-features",
                "--all-features",
                "-p",
                profile.package(),
                "--target",
                TARGET,
                "--prefix",
                "none",
                "--format",
                "{p}",
                "--edges",
                "normal,build,dev",
            ]),
            16777216,
        )?;
        metadata = super::graph::selected(metadata, &tree)?;
    }
    // Check locked source contents/inventories before allowing their build-time code to run.
    super::graph::resolve(
        root,
        &mut view,
        &metadata,
        &super::graph::expected(&metadata, profile)?,
        profile,
    )?;
    let built = required(
        env.command(&env.cargo)
            .args([
                "test",
                "--locked",
                "--offline",
                "-p",
                profile.package(),
                "--no-default-features",
                "--target",
                TARGET,
                "--test",
                profile.test(),
                "--no-run",
                "--message-format=json",
            ])
            .args(if profile == profile::Profile::Core {
                vec!["--all-features"]
            } else {
                vec![]
            }),
        16777216,
    )?;
    let mut compiled = BTreeSet::new();
    let mut features = BTreeMap::new();
    let mut executable = None;
    let mut scripts = vec![];
    for line in built.lines() {
        let m: Value = serde_json::from_str(line).map_err(|_| "unrecognized Cargo output")?;
        if m["reason"] == "compiler-artifact" {
            let id = super::graph::normalize(m["package_id"].as_str().ok_or("missing package id")?);
            let current = m["features"].clone();
            if features
                .insert(id, current.clone())
                .is_some_and(|old| old != current)
            {
                return Err("unsupported multiple feature units".into());
            }
            compiled.insert(
                m["package_id"]
                    .as_str()
                    .ok_or("missing built package identity")?
                    .to_owned(),
            );
            if m["target"]["name"] == profile.test() {
                executable = Some(PathBuf::from(
                    m["executable"].as_str().ok_or("missing executable")?,
                ));
            }
        }
        if m["reason"] == "build-script-executed" {
            scripts.push(m);
        }
    }
    let executable = executable.ok_or("no selected test executable")?;
    let build_profile: Value = serde_json::from_slice(&view.required(&profile.build_profile())?)
        .map_err(|e| e.to_string())?;
    let mut script_ids: Vec<_> = scripts.iter().map(|s| s["package_id"].clone()).collect();
    script_ids.sort_by_key(Value::to_string);
    if serde_json::to_value(&features).map_err(|e| e.to_string())? != build_profile["features"]
        || json!(script_ids) != build_profile["build_scripts"]
    {
        let actual = serde_json::to_value(&features).map_err(|e| e.to_string())?;
        let expected = build_profile["features"]
            .as_object()
            .ok_or("missing reviewed features")?;
        for (id, wanted) in expected {
            if actual[id] != *wanted {
                eprintln!(
                    "profile feature mismatch {id}: wanted {wanted}, observed {}",
                    actual[id]
                );
            }
        }
        eprintln!(
            "profile scripts wanted {}, observed {}",
            build_profile["build_scripts"],
            json!(script_ids)
        );
        return Err("compiled features/build scripts are outside the reviewed profile".into());
    }
    let graph = super::graph::resolve(root, &mut view, &metadata, &compiled, profile)?;
    for file in &graph.inputs {
        if source.optional_bytes(&file.path)?.as_deref()
            != Some(view.required(&file.path)?.as_slice())
            || source.executable(&file.path)? != view.executable(&file.path)?
        {
            return Err("build inputs differ from source commit".into());
        }
    }
    let report = toolchain(&env, &executable, &scripts)?;
    let core_cases = view.optional(&format!("{}/cases.json", profile.directory()))?;
    let names = profile.case_names(
        core_cases
            .as_deref()
            .map(|b| std::str::from_utf8(b))
            .transpose()
            .map_err(|e| e.to_string())?,
    )?;
    let (mut cases, raw) = execute(
        &env,
        &executable,
        &names,
        profile == profile::Profile::Channel,
    )?;
    let prefix = format!("{}/{run_id}-{run_attempt}", profile.evidence());
    cases.raw_output = reference(format!("{prefix}/raw.txt"), raw.as_bytes());
    let environment = Environment {
        schema_version: 1,
        target: TARGET.into(),
        features: profile.features(),
        variables: [
            (
                "CARGO_CONFIG_POLICY",
                "no repository/ancestor/home configuration",
            ),
            ("CARGO_HOME", "verified locked registry cache"),
            ("HOME", "fresh empty directory"),
            ("PATH", "pinned Rust toolchain bin:/usr/bin:/bin"),
            ("RUSTUP_TOOLCHAIN", TOOLCHAIN),
            ("RUSTC", "pinned rustc executable"),
            (
                "CARGO_ENCODED_RUSTFLAGS",
                "remap WORKSPACE=/depth-input,TARGET=/depth-target,CARGO_HOME=/depth-cargo;debuginfo=0",
            ),
            ("CARGO_NET_OFFLINE", "true"),
            ("CARGO_INCREMENTAL", "0"),
            ("CARGO_BUILD_JOBS","2"),
            ("SOURCE_DATE_EPOCH", "0"),
            ("LANG", "C"),
            ("LC_ALL", "C"),
            ("TZ", "UTC"),
        ]
        .into_iter()
        .map(|(name, value)| Variable {
            name: name.into(),
            value: value.into(),
        })
        .collect(),
    };
    view.required(COLLECTOR)?;
    let build = Build {
        source_commit: commit,
        source_tree: source.tree()?,
        host_target: TARGET.into(),
        toolchain: reference(
            format!(
                "{}/toolchains/{}.json",
                profile.evidence(),
                reference("", report.as_bytes()).sha256
            ),
            report.as_bytes(),
        ),
        features: profile.features(),
        environment: reference(
            format!("{prefix}/environment.json"),
            serde_json::to_string(&environment).unwrap().as_bytes(),
        ),
        dependency_graph: reference(
            format!("{prefix}/graph.json"),
            serde_json::to_string(&graph).unwrap().as_bytes(),
        ),
        covered_inputs: graph.inputs.clone(),
        absent_inputs: absent,
    };
    view.verify()?;
    Ok(Witness {
        schema_version: 1,
        run_id,
        run_attempt,
        workflow_commit,
        build,
        graph,
        environment,
        provenance: original
            .obligations
            .unwrap()
            .acquired_inputs
            .into_iter()
            .map(|file| Provenance {
                kind: ProvenanceKind::Fixture,
                origin: format!(
                    "isolated {} fixture provenance; not live upstream execution",
                    profile.version()
                ),
                file,
            })
            .collect(),
        cases,
        toolchain_report: report,
        raw_output: raw,
    })
}
pub(super) fn collect(workspace: &Path) -> Result<(), String> {
    let root = fs::canonicalize(workspace).map_err(|e| e.to_string())?;
    collect_profile(&root, profile::Profile::Channel)
}
pub(super) fn collect_profile(root: &Path, profile: profile::Profile) -> Result<(), String> {
    let result = witness(root, profile);
    let value = match &result {
        Ok(w) => serde_json::to_value(w).map_err(|e| e.to_string())?,
        Err(reason) => json!({"schema_version":1,"unavailable":reason}),
    };
    let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
    if bytes.len() > 1048576 {
        return Err("witness bounds exceeded".into());
    }
    let destination = root.join(if profile == profile::Profile::Channel {
        "target/depth-ci"
    } else {
        "target/depth-core"
    });
    fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
    fs::write(destination.join("witness.json"), bytes).map_err(|e| e.to_string())?;
    if let Err(reason) = result {
        eprintln!("isolated depth witness unavailable: {reason}");
    }
    Ok(())
}

fn host_header() -> Result<(u64, u64, String), String> {
    let positive = |name| {
        std::env::var(name)
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|v| *v > 0)
            .ok_or_else(|| format!("missing positive {name}"))
    };
    let workflow = std::env::var("DEPTH_WORKFLOW_SHA").map_err(|_| "missing workflow SHA")?;
    if workflow.len() != 40
        || !workflow
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid workflow implementation SHA".into());
    }
    Ok((
        positive("GITHUB_RUN_ID")?,
        positive("GITHUB_RUN_ATTEMPT")?,
        workflow,
    ))
}
