set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

export UV_CACHE_DIR := justfile_directory() / ".uv-cache"

maturin := "uv run --with maturin maturin"
prek := "uvx --from prek==0.3.6 prek"
core-packages := "-p arco-core -p arco-expr -p arco-solver -p arco-tools -p arco-blocks -p arco-highs -p arco-bench"
workspace-packages := "-p arco-core -p arco-expr -p arco-solver -p arco-tools -p arco-blocks -p arco-highs -p arco-bench -p arco-kdl -p arco-cli -p arco-xpress"
test-packages := "-p arco-core -p arco-expr -p arco-solver -p arco-tools -p arco-blocks -p arco-highs -p arco-bench"

bench-compare baseline candidate:
    cargo run -p arco-bench -- compare \
        --baseline {{ baseline }} \
        --candidate {{ candidate }}

bench-gate baseline candidate duration_threshold="10" memory_threshold="10":
    #!/usr/bin/env bash
    set -euo pipefail
    for stage in total export_csc export_crs export_coo; do
        echo "Checking stage=${stage} duration<={{ duration_threshold }}% memory<={{ memory_threshold }}%"
        cargo run -p arco-bench -- compare \
            --baseline "{{ baseline }}" \
            --candidate "{{ candidate }}" \
            --stage "${stage}" \
            --duration-threshold-pct "{{ duration_threshold }}" \
            --memory-threshold-pct "{{ memory_threshold }}" \
            --format table
    done

bench-report path:
    cargo run -p arco-bench -- report --input {{ path }}

bench-run:
    cargo run -p arco-bench -- run

check:
    cargo check {{ workspace-packages }} --all-features --tests --benches --examples

check-lib:
    cargo check {{ workspace-packages }} --all-features

ci: fmt-check clippy test docs-test

clippy:
    cargo clippy {{ workspace-packages }} --benches --tests --examples --all-features -- -D warnings

clippy-core:
    cargo clippy {{ core-packages }} --benches --tests --examples --all-features -- -D warnings

clippy-solver package:
    cargo clippy -p {{ package }} --benches --tests --examples --all-features -- -D warnings

default: check

doc:
    cargo doc --workspace --no-deps

docs-test: py-dev
    uv run --project bindings/python --with pytest --with numpy pytest scripts/test_docs_doctest.py -v

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

pre-commit:
    test -x ./scripts/setup-dev-env.sh
    {{prek}} run --all-files

py-build: py-licenses
    cd bindings/python && {{ maturin }} build --release

py-build-ci:
    uv run --with build python -m build bindings/python --wheel --outdir dist

py-dev: py-licenses
    cd bindings/python && {{ maturin }} develop

py-doctest-ci:
    uv run --project bindings/python --with pytest --with numpy pytest scripts/test_docs_doctest.py

py-fmt:
    cd bindings/python && uv run ruff format --verbose

py-licenses:
    uv run python scripts/sync_python_licenses.py

py-lint:
    cd bindings/python && uv run ruff check --fix --config=pyproject.toml

py-shell: py-dev
    cd bindings/python && uv run --with numpy ipython

py-smoke-wheel artifact_glob="dist/*.whl":
    uv run python scripts/python_package_smoke.py --artifact-glob "{{ artifact_glob }}"

py-sync:
    cd bindings/python && uv sync

py-test: py-dev
    uv run --project bindings/python --with pytest pytest bindings/python/tests -v

py-type:
    cd bindings/python && uv run ty check src/

setup:
    ./scripts/setup-dev-env.sh

test:
    PYO3_PYTHON=${PYO3_PYTHON:-python3} cargo test {{ test-packages }} --all-features

test-core:
    PYO3_PYTHON=${PYO3_PYTHON:-python3} cargo test {{ core-packages }} --all-features

test-solver package:
    cargo test -p {{ package }} --all-features -- --test-threads=1

workflow-quality:
    uvx zizmor .github
