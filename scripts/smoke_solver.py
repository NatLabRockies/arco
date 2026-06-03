#!/usr/bin/env python3
"""Solver smoke test: run arco with a small model and verify the result.

Usage:
    uv run python scripts/smoke_solver.py --solver highs
    uv run python scripts/smoke_solver.py --solver xpress --arco-binary target/release/arco
    uv run python scripts/smoke_solver.py --solver ipopt --check-unavailable-ipopt

Exit code 0 on success, 1 on failure.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Solver smoke test for arco CLI")
    parser.add_argument(
        "--solver",
        required=True,
        help="Solver family to test (highs, scip, xpress, ipopt)",
    )
    parser.add_argument(
        "--arco-binary",
        default="target/debug/arco",
        help="Path to the arco CLI binary (default: target/debug/arco)",
    )
    parser.add_argument(
        "--model",
        default="examples/dense-lp/input.kdl",
        help="Path to a small KDL model (default: examples/dense-lp/input.kdl)",
    )
    parser.add_argument(
        "--expected-backend",
        default=None,
        help="Expected solver backend name in output (e.g. arco-rust-highs)",
    )
    parser.add_argument(
        "--expected-status",
        default="optimal",
        help="Expected solve status (default: optimal)",
    )
    parser.add_argument(
        "--check-unavailable-ipopt",
        action="store_true",
        help="Instead of solving, verify that IPOPT returns a clear unavailable diagnostic",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit structured JSON log on failure (for CI parsing)",
    )
    return parser.parse_args(argv)


def resolve_arco_binary(path: str) -> Path:
    """Resolve the arco binary path, appending .exe on Windows if needed."""
    p = Path(path)
    if p.is_file():
        return p.resolve()
    if sys.platform == "win32" or sys.platform == "cygwin":
        p = p.with_suffix(".exe")
        if p.is_file():
            return p.resolve()
    raise FileNotFoundError(f"arco binary not found at {path} (or with .exe)")


def run_arco(
    binary: Path,
    args: list[str],
    config_dir: str,
) -> subprocess.CompletedProcess:
    """Run arco with an isolated config dir and env passthrough."""
    env = os.environ.copy()
    env["ARCO_CONFIG_DIR"] = config_dir
    env["NO_COLOR"] = "1"
    return subprocess.run(
        [str(binary)] + args,
        capture_output=True,
        text=True,
        env=env,
    )


def check_solve(
    binary: Path,
    model: str,
    solver: str,
    expected_backend: str | None,
    expected_status: str,
    config_dir: str,
) -> str | None:
    """Run a solve and return None on success or an error message on failure."""
    # Set solver selection
    result = run_arco(binary, ["solver", "set", solver], config_dir)
    if result.returncode != 0:
        return (
            f"arco solver set {solver} failed:\n"
            f"  stdout: {result.stdout.strip()}\n"
            f"  stderr: {result.stderr.strip()}"
        )

    # Run solve (output JSON to stdout by default)
    result = run_arco(
        binary,
        ["run", model],
        config_dir,
    )

    if result.returncode != 0:
        return (
            f"arco run --json for solver={solver} failed (exit={result.returncode}):\n"
            f"  stdout: {result.stdout.strip()}\n"
            f"  stderr: {result.stderr.strip()}"
        )

    # Parse JSON output
    try:
        summary = json.loads(result.stdout)
    except json.JSONDecodeError as err:
        return f"Failed to parse JSON output for solver={solver}: {err}"

    # Check status
    actual_status = summary.get("solve_status", summary.get("status", "unknown"))
    if actual_status != expected_status:
        return (
            f"solver={solver}: expected status '{expected_status}', "
            f"got '{actual_status}'"
        )

    # Check backend name if given
    if expected_backend is not None:
        actual_backend = summary.get("backend", "")
        if expected_backend != actual_backend:
            return (
                f"solver={solver}: expected backend '{expected_backend}', "
                f"got '{actual_backend}'"
            )

    return None


def check_unavailable_ipopt(binary: Path, model: str, config_dir: str) -> str | None:
    """Verify that IPOPT emits a clear unavailable diagnostic in the default build."""
    # Set solver selection to ipopt
    result = run_arco(binary, ["solver", "set", "ipopt"], config_dir)
    if result.returncode != 0:
        return (
            f"arco solver set ipopt failed:\n"
            f"  stdout: {result.stdout.strip()}\n"
            f"  stderr: {result.stderr.strip()}"
        )

    # Try solving a small model
    result = run_arco(binary, ["run", model], config_dir)

    # Should fail with a clear diagnostic
    if result.returncode == 0:
        return (
            "IPOPT solve should have failed in default build but succeeded.\n"
            f"  stdout: {result.stdout.strip()}"
        )

    combined = (result.stdout + "\n" + result.stderr).lower()
    diagnostic_keywords = [
        "not available",
        "not shipped",
        "native ipopt adapter",
        "unavailable",
    ]

    if not any(kw in combined for kw in diagnostic_keywords):
        return (
            "IPOPT diagnostic does not mention unavailability clearly.\n"
            f"  stdout: {result.stdout.strip()}\n"
            f"  stderr: {result.stderr.strip()}"
        )

    return None


def main() -> int:
    args = parse_args()

    try:
        binary = resolve_arco_binary(args.arco_binary)
    except FileNotFoundError as err:
        print(f"ERROR: {err}", file=sys.stderr)
        if args.json:
            print(json.dumps({"ok": False, "error": str(err)}))
        return 1

    with tempfile.TemporaryDirectory(prefix="arco-smoke-") as config_dir:
        # Compute expected backend name if not explicitly given
        expected_backend = args.expected_backend
        if expected_backend is None and not args.check_unavailable_ipopt:
            backend_map = {
                "highs": "arco-rust-highs",
                "scip": "arco-rust-scip",
                "xpress": "arco-rust-xpress",
                "ipopt": "arco-rust-ipopt",
            }
            expected_backend = backend_map.get(args.solver)

        if args.check_unavailable_ipopt:
            error = check_unavailable_ipopt(binary, args.model, config_dir)
        else:
            error = check_solve(
                binary=binary,
                model=args.model,
                solver=args.solver,
                expected_backend=expected_backend,
                expected_status=args.expected_status,
                config_dir=config_dir,
            )

    if error is not None:
        print(f"FAIL ({args.solver} smoke):", file=sys.stderr)
        print(error, file=sys.stderr)
        if args.json:
            print(json.dumps({"ok": False, "solver": args.solver, "error": error}))
        return 1

    print(f"PASS ({args.solver} smoke)")
    if args.json:
        print(json.dumps({"ok": True, "solver": args.solver}))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
