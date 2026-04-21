#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
import logging
from pathlib import Path
import shutil
import subprocess

DEFAULT_EXAMPLES: tuple[str, ...] = (
    "nodal-allocation",
    "generator-allocation",
    "price-taker-battery",
    "simple-electricity-market-storage",
    "capacity-expansion",
    "dense-lp",
)
DEFAULT_COMMANDS: tuple[str, ...] = ("validate", "print-model", "inspect", "run")


def find_repo_root(*, start: Path) -> Path:
    for candidate in (start, *start.parents):
        if (candidate / "Cargo.toml").is_file():
            return candidate
    raise ValueError(f"could not find Cargo.toml above {start}")


REPO_ROOT = find_repo_root(start=Path(__file__).resolve().parent)


@dataclass(frozen=True, slots=True)
class CliResult:
    args: tuple[str, ...]
    returncode: int
    stdout: str
    stderr: str


@dataclass(frozen=True, slots=True)
class SmokeSummary:
    total: int
    failures: int


def parse_csv(value: str | None, *, defaults: tuple[str, ...]) -> list[str]:
    if value is None:
        return list(defaults)
    items = [item.strip() for item in value.split(",") if item.strip()]
    if not items:
        raise ValueError("expected at least one comma-separated value")
    return items


def resolve_arco_binary(*, value: str) -> str:
    binary_path = Path(value)
    if binary_path.is_file():
        return str(binary_path)

    resolved = shutil.which(value)
    if resolved is not None:
        return resolved

    raise ValueError(
        f"arco binary not found: {value}. Build with `cargo build -p arco-cli` or pass --arco-binary."
    )


def resolve_example_paths(*, examples: list[str]) -> list[Path]:
    paths: list[Path] = []
    for example in examples:
        if example.endswith(".kdl"):
            candidate = REPO_ROOT / example
        else:
            candidate = REPO_ROOT / "examples" / example / "input.kdl"
        if not candidate.is_file():
            raise ValueError(f"example path not found: {candidate}")
        paths.append(candidate)
    return paths


def run_cli(*, arco_binary: str, command: str, model_path: Path, timeout_seconds: int) -> CliResult:
    args = [arco_binary, command, str(model_path)]
    if command == "run":
        args.append("--compact")

    completed = subprocess.run(
        args,
        check=False,
        capture_output=True,
        text=True,
        timeout=timeout_seconds,
    )
    return CliResult(
        args=tuple(args),
        returncode=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
    )


def build_logger(*, verbose: bool) -> logging.Logger:
    logging.basicConfig(level=logging.DEBUG if verbose else logging.INFO, format="%(message)s")
    return logging.getLogger("example-cli-smoke")


def log_event(
    *,
    logger: logging.Logger,
    level: int,
    event: str,
    **fields: object,
) -> None:
    payload = {"event": event, **fields}
    logger.log(level, json.dumps(payload, ensure_ascii=False, sort_keys=True))


def run_smoke_checks(
    *,
    logger: logging.Logger,
    arco_binary: str,
    example_paths: list[Path],
    commands: list[str],
    timeout_seconds: int,
    fail_fast: bool,
) -> SmokeSummary:
    failures = 0
    total = 0

    for model_path in example_paths:
        relative_model = str(model_path.relative_to(REPO_ROOT))
        for command in commands:
            total += 1
            log_event(
                logger=logger,
                level=logging.INFO,
                event="smoke_command_started",
                command=command,
                model=relative_model,
            )
            result = run_cli(
                arco_binary=arco_binary,
                command=command,
                model_path=model_path,
                timeout_seconds=timeout_seconds,
            )

            if result.returncode == 0:
                log_event(
                    logger=logger,
                    level=logging.INFO,
                    event="smoke_command_passed",
                    command=command,
                    model=relative_model,
                )
                continue

            failures += 1
            log_event(
                logger=logger,
                level=logging.ERROR,
                event="smoke_command_failed",
                command=command,
                model=relative_model,
                exit_code=result.returncode,
                cli_args=list(result.args),
                stdout=result.stdout,
                stderr=result.stderr,
            )
            if fail_fast:
                return SmokeSummary(total=total, failures=failures)

    return SmokeSummary(total=total, failures=failures)


def run_example_formulations_smoke(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Run curated arco CLI example smoke checks with debug-friendly structured logs."
    )
    parser.add_argument(
        "--arco-binary",
        default=str(REPO_ROOT / "target" / "debug" / "arco"),
        help="Path or executable name for arco (default: target/debug/arco)",
    )
    parser.add_argument(
        "--examples",
        help="Comma-separated example names (e.g. dense-lp) or KDL paths under repo root.",
    )
    parser.add_argument(
        "--commands",
        help="Comma-separated commands from: validate,print-model,inspect,run",
    )
    parser.add_argument("--fail-fast", action="store_true", help="Stop on first failure.")
    parser.add_argument("--timeout-seconds", type=int, default=300)
    parser.add_argument("--verbose", action="store_true", help="Enable debug-level logging.")
    args = parser.parse_args(argv)

    commands = parse_csv(args.commands, defaults=DEFAULT_COMMANDS)
    invalid_commands = sorted(set(commands) - set(DEFAULT_COMMANDS))
    if invalid_commands:
        raise ValueError(f"unsupported commands: {', '.join(invalid_commands)}")

    logger = build_logger(verbose=args.verbose)
    example_ids = parse_csv(args.examples, defaults=DEFAULT_EXAMPLES)
    example_paths = resolve_example_paths(examples=example_ids)
    arco_binary = resolve_arco_binary(value=args.arco_binary)

    summary = run_smoke_checks(
        logger=logger,
        arco_binary=arco_binary,
        example_paths=example_paths,
        commands=commands,
        timeout_seconds=args.timeout_seconds,
        fail_fast=args.fail_fast,
    )

    log_event(
        logger=logger,
        level=logging.INFO,
        event="smoke_run_finished",
        total=summary.total,
        failures=summary.failures,
        status="failed" if summary.failures else "passed",
    )

    return 1 if summary.failures else 0
