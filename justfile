#!/usr/bin/env -S just --justfile

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
set windows-shell := ["C:/Program Files/Git/bin/bash.exe", "-eu", "-o", "pipefail", "-c"]

export UV_CACHE_DIR := justfile_directory() / ".uv-cache"

rust-packages := "--workspace --exclude arco-python"
clippy-packages := "--workspace --exclude arco-python"
arco-debug-bin := justfile_directory() / "target/debug/arco"
arco-release-bin := justfile_directory() / "target/release/arco"
cli-artifact := justfile_directory() / "artifacts/arco-cli-linux.tar.gz"
solver-build-env := justfile_directory() / "scripts/with_solver_build_env.sh"
ruff-version := "0.15.6"

alias t := test
alias rt := rust-test
alias pt := py-test
alias kp := check-pkg
alias tp := test-pkg
alias cp := clippy-pkg

[group: 'onboarding']
default: help

[group: 'onboarding']
help:
    just --list
    printf '\nCommon commands:\n'
    printf '  just setup                 Install git hooks after verifying prek exists\n'
    printf '  just check                 Run Rust, Python, docs, and architecture checks\n'
    printf '  just test                  Run Rust and Python tests\n'
    printf '  just kdl-examples          Run curated KDL CLI acceptance examples\n'
    printf '  just smoke-solver highs    Run one solver smoke check\n'
    printf '  just hawk                  Check unnecessary public Rust visibility\n'
    printf '  just ci                    Run the local CI aggregate\n'

[group: 'onboarding']
setup:
    if ! command -v prek >/dev/null 2>&1; then printf 'prek not found. Install prek before running `just setup`.\n' >&2; exit 1; fi
    prek install

[group: 'dev']
fmt:
    cargo fmt --all
    cd bindings/python && uv run --no-project --with "ruff=={{ ruff-version }}" ruff format --verbose

[group: 'dev']
fmt-check:
    cargo fmt --all -- --check
    cd bindings/python && uv run --no-project --with "ruff=={{ ruff-version }}" ruff format --check

[group: 'dev']
check:
    just rust-check
    just py-check
    just docs-test
    just arch-check

[group: 'dev']
test:
    just rust-test
    just py-test

[group: 'dev']
lint:
    just rust-clippy
    just py-lint-check
    just py-type

[group: 'hygiene']
hooks:
    just pre-commit

[group: 'hygiene']
pre-commit:
    just pre-commit-stage pre-commit
    just pre-commit-stage pre-push

[group: 'hygiene']
pre-commit-stage stage:
    prek run --all-files --hook-stage {{ stage }} --show-diff-on-failure --color always

[group: 'hygiene']
kdl-overlay-check:
    ./scripts/check-kdl-overlay.sh

[group: 'hygiene']
vscode-extension-check:
    bash scripts/ci_vscode_extension_check.sh

[group: 'hygiene']
workflow-quality:
    uvx zizmor --pedantic .github/

[group: 'hygiene']
release-check dist_bin="dist":
    uv run --no-project --python 3.12 --with pytest pytest scripts/test_release_pipeline.py bindings/python/tests/test_rust_boundaries.py -q
    bash scripts/ci_test_dist_pipeline.sh "{{ dist_bin }}"

[group: 'rust']
rust-check:
    ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" cargo check {{ rust-packages }} --tests --benches --examples
    ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" cargo check -p arco-scip --no-default-features --features scip-bundled

[group: 'rust']
rust-clippy:
    ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" cargo clippy {{ clippy-packages }} --benches --tests --examples -- -D warnings
    ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" cargo clippy -p arco-scip --no-default-features --features scip-bundled -- -D warnings

[group: 'rust']
rust-test:
    PYO3_PYTHON=${PYO3_PYTHON:-python3} ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" cargo +${RUST_TOOLCHAIN_VERSION:-1.85.1} test {{ rust-packages }}
    PYO3_PYTHON=${PYO3_PYTHON:-python3} ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" cargo +${RUST_TOOLCHAIN_VERSION:-1.85.1} test -p arco-scip --no-default-features --features scip-bundled

[group: 'rust']
hawk:
    cargo +${RUST_TOOLCHAIN_VERSION:-1.85.1} generate-lockfile
    ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" cargo +${HAWK_TOOLCHAIN_VERSION:-1.97.1} hawk check --target-dir target/hawk -D warnings

[group: 'rust']
scip-feature-guard:
    if cargo check -p arco-scip --no-default-features --features scip-bundled,scip-from-source; then \
        printf 'expected arco-scip to reject scip-bundled + scip-from-source\n' >&2; \
        exit 1; \
    fi

[group: 'rust']
check-pkg package:
    cargo check -p {{ package }} --all-features --tests --benches --examples

[group: 'rust']
test-pkg package:
    cargo test -p {{ package }} --all-features

[group: 'rust']
test-one package test_filter:
    cargo test -p {{ package }} {{ test_filter }} --all-features

[group: 'rust']
clippy-pkg package:
    cargo clippy -p {{ package }} --benches --tests --examples --all-features -- -D warnings

[group: 'rust']
verify-pkg package:
    just check-pkg {{ package }}
    just test-pkg {{ package }}
    just clippy-pkg {{ package }}

[group: 'python']
py-sync:
    cd bindings/python && uv sync

[group: 'python']
py-licenses:
    uv run python scripts/sync_python_licenses.py

[group: 'python']
py-dev: py-licenses py-sync
    CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" bash -lc 'cd bindings/python && ./.venv/bin/maturin develop'

[group: 'python']
py-fmt:
    cd bindings/python && uv run --no-project --with "ruff=={{ ruff-version }}" ruff format --verbose

[group: 'python']
py-fmt-check:
    cd bindings/python && uv run --no-project --with "ruff=={{ ruff-version }}" ruff format --check

[group: 'python']
py-lint:
    cd bindings/python && uv run --no-project --with "ruff=={{ ruff-version }}" ruff check --fix --config=pyproject.toml

[group: 'python']
py-lint-check:
    cd bindings/python && uv run --no-project --with "ruff=={{ ruff-version }}" ruff check --config=pyproject.toml

[group: 'python']
py-type:
    cd bindings/python && uv run --no-project --with ty ty check src/

[group: 'python']
py-check:
    just py-fmt-check
    just py-lint-check
    just py-type

[group: 'python']
py-cli-build:
    ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" cargo build -p arco-cli --no-default-features --bin arco

[group: 'python']
py-test: py-dev py-cli-build
    cli_bin="$PWD/target/debug/arco"; if [[ ! -x "$cli_bin" && -x "$cli_bin.exe" ]]; then cli_bin="$cli_bin.exe"; fi; ARCO_CLI_BIN="$cli_bin" uv run --project bindings/python --with pytest pytest bindings/python/tests -v

[group: 'python']
py-build-wheel: py-licenses
    ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" bash scripts/build_python_wheel.sh

[group: 'python']
py-build-release-wheel: py-licenses
    PYTHON_WHEEL_NO_DEFAULT_FEATURES="${PYTHON_WHEEL_NO_DEFAULT_FEATURES:-1}" PYTHON_WHEEL_FEATURES="${PYTHON_WHEEL_FEATURES:-pyo3/extension-module,xpress,scip-from-source}" ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" bash scripts/build_python_wheel.sh

[group: 'python']
py-build-sdist:
    rm -f dist/*.tar.gz
    uv run --no-project --with maturin maturin sdist --manifest-path bindings/python/Cargo.toml --out dist
    uv run --no-project python scripts/validate_python_sdist.py --artifact dist/*.tar.gz

[group: 'python']
py-build:
    just py-build-wheel
    just py-build-sdist

[group: 'python']
py-smoke-wheel artifact_glob="dist/*.whl":
    uv run --no-project python scripts/python_package_smoke.py --artifact-glob "{{ artifact_glob }}"

[group: 'python']
py-validate-wheel artifact_glob="dist/*.whl":
    just py-build
    just py-smoke-wheel "{{ artifact_glob }}"

[group: 'python']
py-shell: py-dev
    cd bindings/python && uv run --with numpy ipython

[group: 'docs']
docs-test:
    ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" uv run --project bindings/python pytest scripts/test_docs_doctest.py -v

[group: 'docs']
doc:
    cargo doc --workspace --no-deps

[group: 'architecture']
arch-check:
    uv run python scripts/check_architecture.py

[group: 'product']
build-cli:
    "{{ solver-build-env }}" cargo build -p arco-cli --bin arco

[group: 'product']
build-cli-feature features:
    "{{ solver-build-env }}" cargo build -p arco-cli --bin arco --features "{{ features }}"

[group: 'product']
build-cli-release:
    "{{ solver-build-env }}" cargo build --release -p arco-cli --bin arco --no-default-features --features "xpress,scip-from-source"

[group: 'product']
vscode-extension-install:
    npm --prefix tools/vscode-arco-kdl run install:local

[group: 'examples']
kdl-examples args="":
    just _kdl-examples "{{ arco-debug-bin }}" "{{ args }}"

[group: 'examples']
kdl-examples-with-binary arco_binary args="":
    just _kdl-examples "{{ arco_binary }}" "{{ args }}"

[group: 'solver']
smoke-solver solver features="" model="" check_unavailable="":
    if [[ "{{ features }}" == "" ]]; then just build-cli; else just build-cli-feature "{{ features }}"; fi
    uv run python scripts/smoke_solver.py --solver "{{ solver }}" --arco-binary "{{ arco-debug-bin }}" \
        {{ if model != "" { "--model " + model } else { "" } }} \
        {{ if check_unavailable != "" { "--check-unavailable-ipopt" } else { "" } }}

[group: 'solver']
smoke-solver-highs:
    just smoke-solver highs

[group: 'solver']
smoke-solver-scip:
    just smoke-solver scip

[group: 'solver']
smoke-solver-xpress:
    just smoke-solver xpress xpress

[group: 'solver']
smoke-solver-ipopt-unavailable:
    just smoke-solver ipopt "" "" unavailable

[group: 'benchmarks']
benchmarks arco_binary=arco-debug-bin:
    if [[ ! -x "{{ arco_binary }}" ]]; then just _build-cli-for-path "{{ arco_binary }}"; fi
    LD_LIBRARY_PATH="$(dirname "{{ arco_binary }}"):${LD_LIBRARY_PATH:-}" DYLD_LIBRARY_PATH="$(dirname "{{ arco_binary }}"):${DYLD_LIBRARY_PATH:-}" uv run --no-project python scripts/bench.py --arco-binary "{{ arco_binary }}" --workflows validate,run --repetitions 10 --output artifacts/benchmark-results.json

[group: 'benchmarks']
benchmark-guard:
    uv run python scripts/bench_guard.py --current artifacts/benchmark-results.json --baseline cache/benchmark-data.json --ratio-threshold 1.25 --absolute-delta-ms 1.0 --min-baseline-samples 10

[group: 'ci']
ci:
    just ci-rust-fmt
    just ci-rust-clippy
    just ci-rust-test
    just ci-python-check
    just ci-python-test
    just ci-python-wheel
    just ci-docs-test
    just ci-arch-check

[group: 'ci']
ci-rust-fmt:
    cargo fmt --all -- --check

[group: 'ci']
ci-rust-clippy:
    just rust-clippy

[group: 'ci']
ci-rust-test:
    just rust-test

[group: 'ci']
ci-release-cli-check:
    ARCO_HIGHS_ENABLE_APPLE_STATIC=1 "{{ solver-build-env }}" cargo check -p arco-cli --bin arco --no-default-features --features "xpress,scip-from-source"

[group: 'ci']
ci-cli-build:
    just build-cli-release

[group: 'ci']
ci-package-cli-artifact archive=cli-artifact:
    mkdir -p "$(dirname "{{ archive }}")"
    staging_dir="$(mktemp -d)"; \
    trap 'rm -rf "$staging_dir"' EXIT; \
    cp "{{ arco-release-bin }}" "$staging_dir/"; \
    tar -C "$staging_dir" -czf "{{ archive }}" .

[group: 'ci']
ci-unpack-cli-artifact archive=cli-artifact:
    mkdir -p "$(dirname "{{ arco-release-bin }}")"
    tar -C "$(dirname "{{ arco-release-bin }}")" -xzf "{{ archive }}"

[group: 'ci']
ci-solver-smoke solver check_unavailable="":
    uv run --no-project python scripts/smoke_solver.py --solver "{{ solver }}" --arco-binary "{{ arco-release-bin }}" {{ if check_unavailable != "" { "--check-unavailable-ipopt" } else { "" } }}

[group: 'ci']
ci-kdl-examples:
    just kdl-examples-with-binary "{{ arco-release-bin }}"

[group: 'ci']
ci-benchmarks:
    just benchmarks "{{ arco-release-bin }}"

[group: 'ci']
ci-benchmark-guard:
    just benchmark-guard

[group: 'ci']
ci-python-check:
    just py-check

[group: 'ci']
ci-python-test:
    just py-test

[group: 'ci']
ci-python-wheel artifact_glob="dist/*.whl":
    just py-validate-wheel "{{ artifact_glob }}"

[group: 'ci']
ci-python-release-wheel artifact_glob="dist/*.whl":
    just py-build-release-wheel
    just py-smoke-wheel "{{ artifact_glob }}"

[group: 'ci']
ci-docs-test:
    just docs-test

[group: 'ci']
ci-arch-check:
    just arch-check

[group: 'ci']
ci-workflow-quality:
    just workflow-quality

[private]
_kdl-examples arco_binary args="":
    if [[ ! -x "{{ arco_binary }}" ]]; then just _build-cli-for-path "{{ arco_binary }}"; fi
    LD_LIBRARY_PATH="$(dirname "{{ arco_binary }}"):${LD_LIBRARY_PATH:-}" DYLD_LIBRARY_PATH="$(dirname "{{ arco_binary }}"):${DYLD_LIBRARY_PATH:-}" uv run --no-project python -c "from scripts.test_example_formulations import run_example_formulations_smoke; raise SystemExit(run_example_formulations_smoke())" --arco-binary "{{ arco_binary }}" {{ args }}

[private]
_build-cli-for-path arco_binary:
    case "{{ arco_binary }}" in \
        *target/release/arco) just build-cli-release ;; \
        *) just build-cli ;; \
    esac
