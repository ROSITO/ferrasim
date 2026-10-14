"""Independently check the editor schema and example with JSON Schema 2020-12."""

import copy
import json
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
schema = json.loads((ROOT / "schemas/model-0.1.schema.json").read_text(encoding="utf-8"))
model = json.loads((ROOT / "examples/minimal/model.json").read_text(encoding="utf-8"))
Draft202012Validator.check_schema(schema)
validator = Draft202012Validator(schema)
validator.validate(model)

for field, value in [("schema_version", "future"), ("unexpected", True)]:
    invalid = copy.deepcopy(model)
    invalid[field] = value
    if validator.is_valid(invalid):
        raise ValueError(f"Schema did not reject {field}")

invalid = copy.deepcopy(model)
invalid["solver_settings"]["rtol"] = 0
invalid["solver_settings"]["atol"] = 0
if validator.is_valid(invalid):
    raise ValueError("Schema accepted two zero tolerances")

invalid = copy.deepcopy(model)
invalid["boundary_conditions"][0]["components"] = {"x": None}
if validator.is_valid(invalid):
    raise ValueError("Schema accepted a prescription without a numeric component")

print("Editor schema and reference example passed independent structural checks.")
