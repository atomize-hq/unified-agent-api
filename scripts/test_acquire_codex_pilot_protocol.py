import unittest

from acquire_codex_pilot_protocol import select_methods


class SelectedProtocolTests(unittest.TestCase):
    def test_selected_methods_keep_transitive_definitions_without_foreign_methods(self):
        schema = {
            "oneOf": [
                {"properties": {"method": {"enum": ["thread/start"]},
                                "params": {"$ref": "#/definitions/A"}}},
                {"properties": {"method": {"enum": ["foreign"]}}},
            ],
            "definitions": {"A": {"$ref": "#/definitions/B"}, "B": {"type": "string"},
                            "Unused": {"type": "number"}},
        }
        selected = select_methods(schema, ["thread/start"])
        self.assertEqual(len(selected["oneOf"]), 1)
        self.assertEqual(set(selected["definitions"]), {"A", "B"})

    def test_missing_duplicate_and_external_references_refuse(self):
        row = {"properties": {"method": {"enum": ["wanted"]}}}
        for schema in [
            {"oneOf": []},
            {"oneOf": [row, row]},
            {"oneOf": [dict(row, bad={"$ref": "https://example.com"})]},
            {"oneOf": [dict(row, bad={"$ref": "#/definitions/Missing"})]},
        ]:
            with self.assertRaises(ValueError):
                select_methods(schema, ["wanted"])


if __name__ == "__main__":
    unittest.main()
