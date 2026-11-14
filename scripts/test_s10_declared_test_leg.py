"""Regression coverage for the declared checkout runner's filesystem/process boundary."""

import importlib.util
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest import mock


SCRIPT = Path(__file__).with_name("s10_declared_test_leg.py")
SPEC = importlib.util.spec_from_file_location("s10_declared_test_leg", SCRIPT)
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)
REPOSITORY = SCRIPT.resolve().parents[1]


class DeclaredTestLegTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.source = Path(temporary.name) / "source"
        self.source.mkdir()
        for relative in [
            "crates/xtask/tests/fixtures/depth-synthetic",
            "crates/xtask/data/agent_registry.toml",
            "docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json",
            "Makefile",
        ]:
            source = REPOSITORY / relative
            destination = self.source / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            if source.is_dir():
                shutil.copytree(source, destination)
            else:
                shutil.copy2(source, destination)
        self.files = [str(path.relative_to(self.source)) for path in self.source.rglob("*")
                      if path.is_file()]
        self.cache = Path(temporary.name) / "dependency-cache"
        (self.cache / "registry/cache").mkdir(parents=True)
        (self.cache / "registry/cache/dependency.crate").write_text("cached dependency")
        (self.cache / "config.toml").write_text("untrusted home config")
        (self.cache / "credentials.toml").write_text("secret")
        self.calls = []

    def subprocess(self, arguments, **kwargs):
        self.calls.append((list(arguments), kwargs))
        if "ls-files" in arguments:
            return "\0".join(self.files) + "\0"
        if arguments[:2] == ["git", "init"]:
            (Path(kwargs["cwd"]) / ".git/info").mkdir(parents=True)
        if "--message-format=json" in arguments:
            return json.dumps({"reason": "compiler-artifact", "executable": "fixture-reader",
                               "target": {"kind": ["lib"]}})
        return ""

    def invoke(self, destination="target/declared", lifecycle="onboarding", extra=()):
        with mock.patch.object(runner, "__file__", str(self.source / "scripts/runner.py")), \
                mock.patch("sys.argv", [str(SCRIPT), "--destination", destination,
                                       "--lifecycle-path", lifecycle, *extra]), \
                mock.patch.object(runner.subprocess, "check_output", side_effect=self.subprocess), \
                mock.patch.object(runner.subprocess, "run", side_effect=self.subprocess), \
                mock.patch("builtins.print"):
            runner.main()
        return self.source / destination

    def test_escaping_registry_symlinks_refuse_before_destination_mutation(self):
        registry = self.source / "crates/xtask/data/agent_registry.toml"
        original = registry.read_bytes()
        outside = self.source.parent / "external-registry"
        outside.write_bytes(original)
        for target in [str(outside), "../../../../external-registry", "missing-external"]:
            with self.subTest(target=target):
                registry.unlink()
                registry.symlink_to(target)
                self.calls.clear()
                with self.assertRaisesRegex(ValueError, "symlink"):
                    self.invoke()
                self.assertFalse((self.source / "target/declared").exists())
                self.assertEqual(outside.read_bytes(), original)
                self.assertEqual(len(self.calls), 1, "only source enumeration may run")

    def test_symlinked_source_parent_refuses_before_mutation(self):
        directory = self.source / "crates/xtask/data"
        external = self.source.parent / "external-data"
        directory.rename(external)
        directory.symlink_to(external, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.invoke()
        self.assertFalse((self.source / "target/declared").exists())

    def test_every_subprocess_receives_an_allowlisted_environment(self):
        poison = {"GIT_DIR": "/caller/.git", "GIT_WORK_TREE": "/caller",
                  "GIT_CONFIG_COUNT": "1", "GH_TOKEN": "secret", "AWS_SECRET_ACCESS_KEY": "secret",
                  "CARGO_REGISTRIES_CRATES_IO_TOKEN": "secret", "SSH_AUTH_SOCK": "/agent",
                  "RUSTC_WRAPPER": "remote-cache", "RUSTC_WORKSPACE_WRAPPER": "remote-cache",
                  "RUSTFLAGS": "caller flags", "LD_PRELOAD": "caller library",
                  "HOME": str(self.source.parent), "CARGO_HOME": str(self.cache)}
        with mock.patch.dict(os.environ, poison):
            root = self.invoke()
        for arguments, kwargs in self.calls:
            with self.subTest(arguments=arguments):
                environment = kwargs.get("env")
                self.assertIsNotNone(environment)
                self.assertFalse((set(poison) - {"HOME", "CARGO_HOME"}) & set(environment))
                self.assertFalse(any(key.startswith("GIT_") for key in environment))
                for key in ["HOME", "CARGO_HOME", "CARGO_TARGET_DIR"]:
                    self.assertTrue(Path(environment[key]).is_relative_to(root))
                self.assertEqual(environment["CARGO_NET_OFFLINE"], "true")
        cargo_home = Path(self.calls[-1][1]["env"]["CARGO_HOME"])
        self.assertEqual((cargo_home / "registry/cache/dependency.crate").read_text(),
                         "cached dependency")
        self.assertFalse((cargo_home / "config.toml").exists())
        self.assertFalse((cargo_home / "credentials.toml").exists())
        self.assertEqual((self.cache / "registry/cache/dependency.crate").read_text(),
                         "cached dependency")

    def test_dependency_cache_symlinks_refuse_before_destination_mutation(self):
        archive = self.cache / "registry/cache/dependency.crate"
        archive.unlink()
        archive.symlink_to(self.cache / "credentials.toml")
        with mock.patch.dict(os.environ, {"CARGO_HOME": str(self.cache)}):
            with self.assertRaisesRegex(ValueError, "symlink"):
                self.invoke()
        self.assertFalse((self.source / "target/declared").exists())
        self.assertEqual((self.cache / "credentials.toml").read_text(), "secret")

    def test_explicit_dependency_cache_overrides_caller_home(self):
        with mock.patch.dict(os.environ, {"CARGO_HOME": str(self.source.parent / "absent-cache")}):
            self.invoke(extra=("--dependency-cache", str(self.cache)))
        cargo_home = Path(self.calls[-1][1]["env"]["CARGO_HOME"])
        self.assertTrue((cargo_home / "registry/cache/dependency.crate").is_file())
        self.assertFalse((cargo_home / "credentials.toml").exists())

    def test_build_outputs_are_per_destination(self):
        environments = []
        with mock.patch.dict(os.environ, {"CARGO_HOME": str(self.cache)}):
            for destination in ["target/onboarding", "target/maintenance"]:
                self.calls.clear()
                root = self.invoke(destination)
                environment = self.calls[-1][1]["env"]
                self.assertTrue(Path(environment["CARGO_TARGET_DIR"]).is_relative_to(root))
                environments.append(environment)
        for key in ["CARGO_TARGET_DIR", "CARGO_HOME", "HOME"]:
            self.assertNotEqual(environments[0][key], environments[1][key])

    def test_default_command_matches_the_makefile_test_leg(self):
        with mock.patch.dict(os.environ, {"CARGO_HOME": str(self.cache)}):
            self.invoke()
        command = next(arguments for arguments, _ in reversed(self.calls)
                       if arguments[:2] == ["cargo", "test"])
        makefile = (REPOSITORY / "Makefile").read_text()
        authoritative = makefile.split("\ntest:\n\t", 1)[1].splitlines()[0].split()
        self.assertEqual([argument for argument in command if argument != "--offline"], authoritative)
        self.assertIn("--offline", command)

    def test_both_lifecycle_declarations_and_focused_arguments(self):
        with mock.patch.dict(os.environ, {"CARGO_HOME": str(self.cache)}):
            for lifecycle in ["onboarding", "maintenance"]:
                with self.subTest(lifecycle=lifecycle):
                    self.calls.clear()
                    root = self.invoke(f"target/{lifecycle}", lifecycle,
                                       ("--", "-p", "xtask", "--test", "agent_registry"))
                    registry = (root / "crates/xtask/data/agent_registry.toml").read_text()
                    self.assertIn(f'enabled_paths = ["{lifecycle}"]', registry)
                    self.assertIn(f'lifecycle_path = "{lifecycle}"', registry)
                    self.assertNotIn("{{ISOLATED_", registry)
                    self.assertTrue((root / "producer/source.rs").is_file())
                    command = next(arguments for arguments, _ in reversed(self.calls)
                                   if arguments[:2] == ["cargo", "test"])
                    self.assertEqual(command, ["cargo", "test", "--offline", "-p", "xtask",
                                               "--test", "agent_registry"])


if __name__ == "__main__":
    unittest.main()
