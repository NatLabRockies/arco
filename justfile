#!/usr/bin/env -S just --justfile

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

export UV_CACHE_DIR := justfile_directory() / ".uv-cache"

alias t := test
alias qc := step-quality

# Rust package group (all workspace crates except python and ipopt bindings)
rust-packages := "--workspace --exclude arco-python --exclude arco-ipopt"

# Rust package group for clippy in CI where Xpress SDK is unavailable
clippy-packages := "--workspace --exclude arco-python --exclude arco-ipopt --exclude arco-xpress"

[group: 'bench']
bench-compare baseline candidate:
    cargo run -p arco-bench -- compare \
        --baseline {{ baseline }} \
        --candidate {{ candidate }}

[group: 'bench']
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

[group: 'bench']
bench-report path:
    cargo run -p arco-bench -- report --input {{ path }}

[group: 'bench']
bench-run:
    cargo run -p arco-bench -- run

[group: 'rust']
check:
    cargo check {{ rust-packages }} --all-features --tests --benches --examples

[group: 'rust']
check-dev:
    cargo check --all-features --tests --benches --examples

[group: 'ci']
ci: fmt-check clippy-all test-core docs-test

[group: 'rust']
clippy:
    cargo clippy --benches --tests --examples --all-features -- -D warnings

[group: 'ci']
clippy-all:
    cargo clippy {{ clippy-packages }} --benches --tests --examples -- -D warnings

[group: 'ci']
clippy-core:
    cargo clippy {{ clippy-packages }} --benches --tests --examples -- -D warnings

[group: 'ci']
clippy-solver package:
    cargo clippy -p {{ package }} --benches --tests --examples --all-features -- -D warnings

[group: 'onboarding']
default: check

[group: 'ci']
doc:
    cargo doc --workspace --no-deps

[group: 'ci']
docs-test: py-dev
    uv run --project bindings/python --with pytest --with numpy pytest scripts/test_docs_doctest.py -v

[group: 'rust']
fmt:
    cargo fmt --all

[group: 'rust']
fmt-check:
    cargo fmt --all -- --check

[group: 'hygiene']
pre-commit:
    test -x ./scripts/setup-dev-env.sh
    just pre-commit-stage pre-commit
    just pre-commit-stage pre-push

[group: 'hygiene']
pre-commit-stage stage:
    uvx --from prek==0.3.6 prek run --all-files --hook-stage {{stage}} --show-diff-on-failure --color always

[group: 'hygiene']
kdl-overlay-check:
    ./scripts/check-kdl-overlay.sh

[group: 'python']
py-build-ci:
    uv run --with build python -m build bindings/python --wheel --outdir dist

[group: 'python']
py-check:
    just py-fmt-check
    just py-lint-check
    just py-type

[group: 'python']
py-dev: py-licenses py-sync
    cd bindings/python && uv run --with maturin maturin develop

[group: 'python']
py-doctest-ci:
    uv run --project bindings/python --with pytest --with numpy pytest scripts/test_docs_doctest.py

[group: 'python']
py-fmt:
    cd bindings/python && uv run --no-project --with ruff ruff format --verbose

[group: 'python']
py-fmt-check:
    cd bindings/python && uv run --no-project --with ruff ruff format --check

[group: 'python']
py-licenses:
    uv run python scripts/sync_python_licenses.py

[group: 'python']
py-lint:
    cd bindings/python && uv run --no-project --with ruff ruff check --fix --config=pyproject.toml

[group: 'python']
py-lint-check:
    cd bindings/python && uv run --no-project --with ruff ruff check --config=pyproject.toml

[group: 'python']
py-shell: py-dev
    cd bindings/python && uv run --with numpy ipython

[group: 'python']
py-smoke-wheel artifact_glob="dist/*.whl":
    uv run --project bindings/python python scripts/python_package_smoke.py --artifact-glob "{{artifact_glob}}"

[group: 'python']
py-sync:
    cd bindings/python && uv sync

[group: 'python']
py-test: py-dev
    uv run --project bindings/python --with pytest pytest bindings/python/tests -v

[group: 'python']
py-type:
    cd bindings/python && uv run --no-project --with ty ty check src/

[group: 'python']
py-validate-wheel artifact_glob="dist/*.whl":
    just py-build-ci
    just py-smoke-wheel "{{ artifact_glob }}"

[group: 'onboarding']
setup:
    ./scripts/setup-dev-env.sh

[group: 'steps']
step-fmt:
    just fmt
    just py-fmt

[group: 'steps']
step-lint:
    just clippy
    just py-lint-check
    just py-type

[group: 'steps']
step-quality:
    just step-fmt
    just step-lint
    just step-test
    just check-dev

[group: 'steps']
step-test: test

[group: 'tdd']
tdd-green package test_filter:
    cargo test -p {{ package }} {{ test_filter }} --all-features

[group: 'tdd']
tdd-red package test_filter:
    ! cargo test -p {{ package }} {{ test_filter }} --all-features

[group: 'tdd']
tdd-refactor package:
    cargo fmt --all
    cargo clippy -p {{ package }} --benches --tests --examples --all-features -- -D warnings
    cargo test -p {{ package }} --all-features

[group: 'rust']
test:
    PYO3_PYTHON=${PYO3_PYTHON:-python3} cargo +${RUST_TOOLCHAIN_VERSION:-1.85.1} test {{ rust-packages }} --exclude arco-xpress

[group: 'ci']
test-core:
    PYO3_PYTHON=${PYO3_PYTHON:-python3} cargo +${RUST_TOOLCHAIN_VERSION:-1.85.1} test {{ rust-packages }} --exclude arco-xpress

[group: 'ci']
test-solver package:
    cargo test -p {{ package }} --all-features -- --test-threads=1

[group: 'hygiene']
workflow-quality:
    uvx zizmor .github
