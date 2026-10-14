"""Check S00 planning artifacts; this does not run scientific validations."""

import json
import math
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def main():
    registry = tomllib.loads((ROOT / "validation/cases.toml").read_text(encoding="utf-8"))
    require(registry["registry_version"] == 1, "Unknown validation registry")
    cases = registry["cases"]
    expected = {
        "tet4-symmetry", "tet4-rigid-modes", "affine-patch",
        "uniaxial-traction", "prescribed-nonzero", "surface-traction",
        "global-equilibrium", "cantilever-convergence", "external-solver",
        "invalid-models",
    }
    ids = [case["id"] for case in cases]
    require(len(ids) == len(set(ids)), "Duplicate validation cases")
    require(set(ids) == expected, "MVP validation coverage changed")
    for case in cases:
        require(case["status"] == "planned", "S00 has no scientific results")
        require(bool(case["reference"]) and bool(case["metric"]), "Missing protocol")
        if "tolerance" in case:
            require(math.isfinite(case["tolerance"]) and case["tolerance"] > 0,
                    "Invalid validation tolerance")

    model_path = ROOT / "examples/model-draft/model.json"
    model = json.loads(model_path.read_text(encoding="utf-8"))
    require(model["schema_version"] == "0.1-draft", "Example must remain a draft")
    require(model["units"] == "SI", "Example reference units must be SI")
    require((model_path.parent / model["mesh"]["path"]).is_file(), "Missing fixture")

    files = [ROOT / "README.md", ROOT / "CONTRIBUTING.md"]
    for directory in ["docs", "validation", "examples"]:
        files.extend((ROOT / directory).rglob("*.md"))
    for path in files:
        text = path.read_text(encoding="utf-8")
        require(text.count("```") % 2 == 0, f"Unbalanced code fences: {path}")
        for link in re.findall(r"\]\(([^)]+)\)", text):
            if link.startswith(("https://", "http://", "#")):
                continue
            target = link.split("#", 1)[0]
            require((path.parent / target).exists(), f"Broken link in {path}: {link}")

    print(f"Planning checks passed: {len(cases)} planned cases and {len(files)} documents.")
    print("No FEM computation or scientific qualification was performed.")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError) as error:
        print(f"Project check failed: {error}", file=sys.stderr)
        sys.exit(1)
