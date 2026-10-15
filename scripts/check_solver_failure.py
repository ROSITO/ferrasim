"""Check failure behavior of the built CG reference executable, in an empty directory."""
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target/debug/examples" / ("cg_reference.exe" if sys.platform == "win32" else "cg_reference")


def main():
    if not BINARY.is_file():
        raise RuntimeError("Build first: cargo build -p ferrasim-solvers --example cg_reference --locked")
    for scenario, diagnostic in [("singular", "NonPositiveCurvature"), ("limit", "IterationBudget")]:
        with tempfile.TemporaryDirectory(prefix="ferrasim-failed-solve-") as directory:
            result = subprocess.run([str(BINARY), scenario], cwd=directory, capture_output=True, text=True, timeout=30)
            if result.returncode == 0 or result.stdout or diagnostic not in result.stderr:
                raise RuntimeError(f"Incorrect failure contract for {scenario}: {result}")
            if list(Path(directory).iterdir()):
                raise RuntimeError(f"Failed scenario {scenario} wrote result files")
    print("Singular and iteration-limit examples fail without success output or result files.")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, subprocess.SubprocessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
