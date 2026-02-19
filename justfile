# Quick reference:
#   just              — type-check workspace (default)
#   just fmt          — format Rust code
#   just test         — run Rust tests
#   just py-test      — build extension + run docs doctests
#   just ci           — full CI pipeline
#   just --list       — show all recipes
#
# Benchmarks:
#   just bench-run                              — run benchmarks
#   just bench-report results.jsonl             — print report
#   just bench-compare base.jsonl new.jsonl     — compare two runs
#   just bench-gate base.jsonl new.jsonl        — CI gate (10% threshold)
#   just bench-gate base.jsonl new.jsonl 5 5    — CI gate (5% threshold)

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

export UV_CACHE_DIR := justfile_directory() / ".uv-cache"

maturin := "uv run --with maturin maturin"

default: check

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

check:
    cargo check --workspace --all-features --tests --benches --examples

check-lib:
    cargo check --workspace --all-features

clippy:
    cargo clippy --all --benches --tests --examples --all-features -- -D warnings

test:
    cargo test --workspace --all-features --exclude arco-python

doc:
    cargo doc --workspace --no-deps

py-sync:
    cd bindings/python && uv sync

py-fmt:
    cd bindings/python && uv run ruff format --verbose

py-lint:
    cd bindings/python && uv run ruff check --fix --config=pyproject.toml

py-type:
    cd bindings/python && uv run ty check src/

py-licenses:
    uv run python scripts/sync_python_licenses.py

py-dev: py-licenses
    cd bindings/python && {{ maturin }} develop

py-build: py-licenses
    cd bindings/python && {{ maturin }} build --release

py-build-ci:
    uv run --with build python -m build bindings/python --wheel --outdir dist

py-smoke-wheel artifact_glob="dist/*.whl":
    uv run python scripts/python_package_smoke.py --artifact-glob "{{ artifact_glob }}"

py-doctest-ci:
    uv run --project bindings/python --with pytest --with numpy pytest scripts/test_docs_doctest.py

py-test: py-dev
    uv run --project bindings/python --with pytest --with numpy pytest scripts/test_docs_doctest.py

docs-test: py-dev
    uv run --project bindings/python --with pytest --with numpy pytest scripts/test_docs_doctest.py -v

py-shell: py-dev
    cd bindings/python && uv run --with numpy ipython


bench-run:
    cargo run -p arco-bench -- run

bench-report path:
    cargo run -p arco-bench -- report --input {{ path }}

bench-compare baseline candidate:
    cargo run -p arco-bench -- compare \
        --baseline {{ baseline }} \
        --candidate {{ candidate }}

bench-gate baseline candidate duration_threshold="10" memory_threshold="10":
    #!/usr/bin/env bash
    set -euo pipefail
    for stage in total variables; do
        echo "Checking stage=${stage} duration<={{ duration_threshold }}% memory<={{ memory_threshold }}%"
        cargo run -p arco-bench -- compare \
            --baseline "{{ baseline }}" \
            --candidate "{{ candidate }}" \
            --stage "${stage}" \
            --duration-threshold-pct "{{ duration_threshold }}" \
            --memory-threshold-pct "{{ memory_threshold }}" \
            --format table
    done

workflow-quality:
    uvx zizmor .github

ci: fmt-check clippy test docs-test
