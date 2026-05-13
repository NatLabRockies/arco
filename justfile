#!/usr/bin/env -S just --justfile

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

export UV_CACHE_DIR := justfile_directory() / ".uv-cache"

alias t := test
alias tp := test-pkg
alias to := test-one
alias cp := clippy-pkg
alias kp := check-pkg

# Rust package group (all workspace crates except python and ipopt bindings)
rust-packages := "--workspace --exclude arco-python --exclude arco-ipopt"

# Rust package group for clippy in CI where Xpress SDK is unavailable
clippy-packages := "--workspace --exclude arco-python --exclude arco-ipopt --exclude arco-xpress"

[group: 'rust']
check:
    cargo check {{ rust-packages }} --all-features --tests --benches --examples

[group: 'ci']
ci: fmt-check clippy-all test docs-test arch-check

[group: 'rust']
clippy:
    cargo clippy --benches --tests --examples --all-features -- -D warnings

[group: 'ci']
clippy-all:
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
arch-check:
    uv run python scripts/check_architecture.py

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
py-build-ci: py-licenses
    uv run --project bindings/python --with maturin maturin build --release --manifest-path bindings/python/Cargo.toml -i ${PYTHON_WHEEL_INTERPRETER:-python3} --compatibility pypi --out dist
    uv run --project bindings/python --with maturin maturin sdist --manifest-path bindings/python/Cargo.toml --out dist

[group: 'python']
py-check:
    just py-fmt-check
    just py-lint-check
    just py-type

[group: 'python']
py-dev: py-licenses py-sync
    cd bindings/python && uv run --with maturin maturin develop

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

[group: 'ci']
lint:
    just fmt-check
    just clippy-all
    just py-lint-check
    just py-type

[group: 'ci']
lint-fix:
    just fmt
    just py-fmt
    just py-lint

[group: 'rust']
test-pkg package:
    cargo test -p {{ package }} --all-features

[group: 'rust']
test-one package test_filter:
    cargo test -p {{ package }} {{ test_filter }} --all-features

[group: 'rust']
check-pkg package:
    cargo check -p {{ package }} --all-features --tests --benches --examples

[group: 'rust']
clippy-pkg package:
    cargo clippy -p {{ package }} --benches --tests --examples --all-features -- -D warnings

[group: 'rust']
verify-pkg package:
    just check-pkg {{ package }}
    just test-pkg {{ package }}
    just clippy-pkg {{ package }}

[group: 'rust']
test:
    PYO3_PYTHON=${PYO3_PYTHON:-python3} cargo +${RUST_TOOLCHAIN_VERSION:-1.85.1} test {{ rust-packages }} --exclude arco-xpress

[group: 'rust']
test-example-formulations args="":
    cargo build -p arco-cli
    uv run python -c "from scripts.test_example_formulations import run_example_formulations_smoke; raise SystemExit(run_example_formulations_smoke())" {{ args }}
[group: 'ci']
test-solver package:
    cargo test -p {{ package }} --all-features -- --test-threads=1

[group: 'hygiene']
workflow-quality:
    workflows=(); while IFS= read -r file; do workflows+=("$file"); done < <(find .github/workflows -type f \( -name '*.yml' -o -name '*.yaml' \) ! -name 'release.yml' -print); uvx zizmor .github/actions .github/dependabot.yml "${workflows[@]}"
