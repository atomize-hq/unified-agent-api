import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

SCRIPTS = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPTS))
spec = importlib.util.spec_from_file_location("observer", SCRIPTS / "observe_codex_pilot_context.py")
observer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(observer)


class ContextObservationTests(unittest.TestCase):
    def test_pinned_schema_config_disables_all_boolean_admitted_features(self):
        report = json.loads((SCRIPTS.parent / "cli_manifests/codex/reports/0.156.1/pilot-protocol-facts.json").read_text())
        config, unbound = observer.expected_config(report)
        self.assertEqual(unbound, ["tool_registry"])
        self.assertEqual(config.count(" = false\n"), 158)  # analytics plus 157 features
        self.assertIn("code_mode = false\n", config)
        self.assertIn("apps_mcp_path_override = false\n", config)
        self.assertNotIn("tool_registry =", config)

    def test_effective_listing_preserves_forced_true_and_rejects_ambiguous_rows(self):
        rows = observer.parse_features("unified_exec stable true\nshell_tool stable false\n")
        self.assertEqual(rows["unified_exec"], {"stage": "stable", "enabled": True})
        self.assertFalse(rows["shell_tool"]["enabled"])
        for text in ["", "a stable maybe", "a stable true\na stable false", "a true"]:
            with self.assertRaises(ValueError):
                observer.parse_features(text)

    def test_unknown_schema_shape_is_not_treated_as_disabled(self):
        with self.assertRaises(ValueError):
            observer.accepts_false({"type": "string"}, {})

    def test_pin_mismatch_refuses_before_any_launch(self):
        with tempfile.TemporaryDirectory() as temp:
            binary = Path(temp) / "codex"
            binary.write_bytes(b"untrusted")
            with patch.object(observer.subprocess, "Popen") as launch:
                with self.assertRaisesRegex(ValueError, "pinned"):
                    observer.observe(binary, SCRIPTS.parent / observer.PROTOCOL, "/usr/bin/bwrap")
                launch.assert_not_called()

    def test_failed_or_interactive_native_exchange_refuses_and_reaps(self):
        for response in [{"id": 1, "error": {"code": -1}},
                         {"id": 90, "method": "requestApproval", "params": {}}]:
            with tempfile.TemporaryDirectory() as temp, tempfile.TemporaryFile() as stderr:
                script = Path(temp) / "fake.py"
                script.write_text("import sys,json\nfor line in sys.stdin:\n print(" + repr(json.dumps(response)) + ",flush=True)\n")
                with self.assertRaises(ValueError):
                    observer.config_reads([sys.executable, str(script)], stderr)


if __name__ == "__main__":
    unittest.main()
