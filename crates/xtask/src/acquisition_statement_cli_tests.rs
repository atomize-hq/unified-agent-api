//! The synthetic onboarding acquisition statement template (OF-P4 B) names, for each input O binds,
//! the exact command that produced it. Each `xtask` command is parsed with this binary's own CLI
//! definitions and must write the input it is listed for, so the template cannot drift from the CLI.
use super::{Cli, Command};
use clap::Parser;
use std::path::{Path, PathBuf};

const STATEMENT: &str = include_str!(
    "../tests/fixtures/depth-synthetic/onboarding/governance/depth-acquisition/0.2.0.toml"
);
const BINARY_PLACEHOLDER: &str = "{{DEPTH_FIXTURE_BINARY}}";
const ROOT: &str = "cli_manifests/depth_fixture";
const VERSION: &str = "0.2.0";
const TARGET: &str = "x86_64-unknown-linux-gnu";

/// What one statement command is, after checking that it writes `path`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Step {
    Install,
    Snapshot,
    Union,
    WrapperCoverage,
    Report,
}

fn some_root(root: &Option<PathBuf>) -> bool {
    root.as_deref() == Some(Path::new(ROOT))
}

fn step(path: &str, command: &str) -> Step {
    let words: Vec<&str> = command.split_whitespace().collect();
    if words.first() == Some(&"cp") {
        // A governing template copied into place: the source exists and the target is the input.
        assert_eq!(words.len(), 3, "{command}");
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(repo.join(words[1]).is_file(), "missing source: {command}");
        assert_eq!(words[2], path, "{command}");
        return Step::Install;
    }
    assert_eq!(words.first(), Some(&"xtask"), "{command}");
    let binary = "/acquired/depth_fixture";
    let words = words.iter().map(|w| w.replace(BINARY_PLACEHOLDER, binary));
    let cli = Cli::try_parse_from(words).unwrap_or_else(|e| panic!("{command}: {e}"));
    match cli.command {
        Command::DepthFixtureSnapshot(args) => {
            assert!(command.contains(BINARY_PLACEHOLDER), "{command}");
            assert_eq!(args.depth_fixture_binary, Path::new(binary), "{command}");
            assert_eq!(args.out_file, Path::new(path), "{command}");
            assert_eq!(path, format!("{ROOT}/snapshots/{VERSION}/{TARGET}.json"));
            assert!(args.capture_raw_help, "{command}");
            assert_eq!(args.raw_help_target.as_deref(), Some(TARGET), "{command}");
            assert!(
                args.supplement.is_none() && args.collected_at.is_none(),
                "{command}"
            );
            Step::Snapshot
        }
        Command::ManifestUnion(args) => {
            assert!(some_root(&args.root) && args.rules.is_none(), "{command}");
            assert_eq!(args.version, VERSION, "{command}");
            assert_eq!(path, format!("{ROOT}/snapshots/{VERSION}/union.json"));
            Step::Union
        }
        Command::DepthFixtureWrapperCoverage(args) => {
            assert_eq!(args.out, Path::new(path), "{command}");
            assert_eq!(path, format!("{ROOT}/wrapper_coverage.json"));
            assert_eq!(
                args.rules,
                Path::new(&format!("{ROOT}/RULES.json")),
                "{command}"
            );
            Step::WrapperCoverage
        }
        Command::ManifestReport(args) => {
            assert!(some_root(&args.root) && args.rules.is_none(), "{command}");
            assert_eq!(args.version, VERSION, "{command}");
            let reports = format!("{ROOT}/reports/{VERSION}/");
            let file = path
                .strip_prefix(&reports)
                .unwrap_or_else(|| panic!("{path}"));
            let known = ["coverage.any.json", "coverage.all.json"];
            assert!(
                known.contains(&file) || file == format!("coverage.{TARGET}.json"),
                "{path}"
            );
            Step::Report
        }
        other => panic!("{path} names no producing command: {other:?}"),
    }
}

#[test]
fn statement_commands_are_runnable_producers_of_their_inputs() {
    let statement: toml_edit::DocumentMut = STATEMENT.parse().unwrap();
    let inputs = statement["inputs"].as_array_of_tables().unwrap();
    let steps: Vec<Step> = inputs
        .iter()
        .map(|input| {
            let path = input["path"].as_str().unwrap();
            step(path, input["command"].as_str().unwrap())
        })
        .collect();
    // Listed in run order: the inputs a command reads come from earlier commands.
    let first = |s: Step| steps.iter().position(|x| *x == s).unwrap();
    let last = |s: Step| steps.iter().rposition(|x| *x == s).unwrap();
    assert!(last(Step::Install) < first(Step::Snapshot));
    assert!(last(Step::Snapshot) < first(Step::Union));
    assert!(last(Step::Union) < first(Step::WrapperCoverage));
    assert!(last(Step::WrapperCoverage) < first(Step::Report));
    // Each command that writes one file is listed once; the report command writes three.
    for once in [Step::Snapshot, Step::Union, Step::WrapperCoverage] {
        assert_eq!(steps.iter().filter(|s| **s == once).count(), 1, "{once:?}");
    }
    assert_eq!(steps.iter().filter(|s| **s == Step::Report).count(), 3);
    assert_eq!(steps.iter().filter(|s| **s == Step::Install).count(), 2);
}
