#!/usr/bin/env -S just --justfile

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
set windows-shell := ["C:/Program Files/Git/bin/bash.exe", "-eu", "-o", "pipefail", "-c"]

export UV_CACHE_DIR := justfile_directory() / ".uv-cache"

rust-packages := "--workspace --exclude arco-python"
clippy-packages := "--workspace --exclude arco-python"
arco-debug-bin := justfile_directory() / "target/debug/arco"
arco-release-bin := justfile_directory() / "target/release/arco"
cli-artifact := justfile_directory() / "artifacts/arco-cli-linux.tar.gz"

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
    printf '  just ci                    Run the local CI aggregate\n'

[group: 'onboarding']
setup:
    if ! command -v prek >/dev/null 2>&1; then printf 'prek not found. Install prek before running `just setup`.\n' >&2; exit 1; fi
    prek install

[group: 'dev']
fmt:
    cargo fmt --all
    cd bindings/python && uv run --no-project --with ruff ruff format --verbose

[group: 'dev']
fmt-check:
    cargo fmt --all -- --check
    cd bindings/python && uv run --no-project --with ruff ruff format --check

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
workflow-quality:
    uvx zizmor --pedantic .github/

[group: 'rust']
rust-check:
    cargo check {{ rust-packages }} --all-features --tests --benches --examples

[group: 'rust']
rust-clippy:
    cargo clippy {{ clippy-packages }} --benches --tests --examples --all-features -- -D warnings

[group: 'rust']
rust-test:
    PYO3_PYTHON=${PYO3_PYTHON:-python3} cargo +${RUST_TOOLCHAIN_VERSION:-1.85.1} test {{ rust-packages }} --all-features

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
    cd bindings/python && uv run --with maturin maturin develop

[group: 'python']
py-fmt:
    cd bindings/python && uv run --no-project --with ruff ruff format --verbose

[group: 'python']
py-fmt-check:
    cd bindings/python && uv run --no-project --with ruff ruff format --check

[group: 'python']
py-lint:
    cd bindings/python && uv run --no-project --with ruff ruff check --fix --config=pyproject.toml

[group: 'python']
py-lint-check:
    cd bindings/python && uv run --no-project --with ruff ruff check --config=pyproject.toml

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
    cargo build -p arco-cli --no-default-features --bin arco

[group: 'python']
py-test: py-dev py-cli-build
    cli_bin="$PWD/target/debug/arco"; if [[ ! -x "$cli_bin" && -x "$cli_bin.exe" ]]; then cli_bin="$cli_bin.exe"; fi; ARCO_CLI_BIN="$cli_bin" uv run --project bindings/python --with pytest pytest bindings/python/tests -v

[group: 'python']
py-build-wheel: py-licenses
    if [[ -n "${PYTHON_WHEEL_FEATURES:-}" ]]; then uv run --project bindings/python --with maturin maturin build --release --manifest-path bindings/python/Cargo.toml -i ${PYTHON_WHEEL_INTERPRETER:-python3} --compatibility pypi --out dist --features "$PYTHON_WHEEL_FEATURES"; else uv run --project bindings/python --with maturin maturin build --release --manifest-path bindings/python/Cargo.toml -i ${PYTHON_WHEEL_INTERPRETER:-python3} --compatibility pypi --out dist; fi

[group: 'python']
py-build-sdist:
    uv run --project bindings/python --with maturin maturin sdist --manifest-path bindings/python/Cargo.toml --out dist

[group: 'python']
py-build:
    just py-build-wheel
    just py-build-sdist

[group: 'python']
py-smoke-wheel artifact_glob="dist/*.whl":
    uv run --project bindings/python python scripts/python_package_smoke.py --artifact-glob "{{ artifact_glob }}"

[group: 'python']
py-validate-wheel artifact_glob="dist/*.whl":
    just py-build
    just py-smoke-wheel "{{ artifact_glob }}"

[group: 'python']
py-shell: py-dev
    cd bindings/python && uv run --with numpy ipython

[group: 'docs']
docs-test:
    uv run --project bindings/python pytest scripts/test_docs_doctest.py -v

[group: 'docs']
doc:
    cargo doc --workspace --no-deps

[group: 'architecture']
arch-check:
    uv run python scripts/check_architecture.py

[group: 'product']
build-cli:
    cargo build -p arco-cli --bin arco

[group: 'product']
build-cli-feature features:
    cargo build -p arco-cli --bin arco --features "{{ features }}"

[group: 'product']
build-cli-release:
    cargo build --release -p arco-cli --bin arco --all-features

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
    LD_LIBRARY_PATH="$(dirname "{{ arco_binary }}"):${LD_LIBRARY_PATH:-}" DYLD_LIBRARY_PATH="$(dirname "{{ arco_binary }}"):${DYLD_LIBRARY_PATH:-}" uv run python scripts/bench.py --arco-binary "{{ arco_binary }}" --workflows validate,run --repetitions 10 --output artifacts/benchmark-results.json

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
ci-cli-build:
    just build-cli-release

[group: 'ci']
ci-package-cli-artifact archive=cli-artifact:
    mkdir -p "$(dirname "{{ archive }}")"
    staging_dir="$(mktemp -d)"; \
    trap 'rm -rf "$staging_dir"' EXIT; \
    cp "{{ arco-release-bin }}" "$staging_dir/"; \
    find "$(dirname "{{ arco-release-bin }}")/build" \
        \( -path "*/scip_install/lib/*.so*" -o -path "*/scip_install/lib/*.dylib" \) \
        \( -type f -o -type l \) \
        -exec cp -a {} "$staging_dir/" \; ; \
    if ! compgen -G "$staging_dir/libscip.so*" >/dev/null && ! compgen -G "$staging_dir/libscip*.dylib" >/dev/null; then \
        printf 'error: SCIP shared libraries missing from release build output\n' >&2; \
        exit 1; \
    fi; \
    tar -C "$staging_dir" -czf "{{ archive }}" .

[group: 'ci']
ci-unpack-cli-artifact archive=cli-artifact:
    mkdir -p "$(dirname "{{ arco-release-bin }}")"
    tar -C "$(dirname "{{ arco-release-bin }}")" -xzf "{{ archive }}"

[group: 'ci']
ci-solver-smoke solver check_unavailable="":
    LD_LIBRARY_PATH="$(dirname "{{ arco-release-bin }}"):${LD_LIBRARY_PATH:-}" DYLD_LIBRARY_PATH="$(dirname "{{ arco-release-bin }}"):${DYLD_LIBRARY_PATH:-}" uv run python scripts/smoke_solver.py --solver "{{ solver }}" --arco-binary "{{ arco-release-bin }}" {{ if check_unavailable != "" { "--check-unavailable-ipopt" } else { "" } }}

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
    LD_LIBRARY_PATH="$(dirname "{{ arco_binary }}"):${LD_LIBRARY_PATH:-}" DYLD_LIBRARY_PATH="$(dirname "{{ arco_binary }}"):${DYLD_LIBRARY_PATH:-}" uv run python -c "from scripts.test_example_formulations import run_example_formulations_smoke; raise SystemExit(run_example_formulations_smoke())" --arco-binary "{{ arco_binary }}" {{ args }}

[private]
_build-cli-for-path arco_binary:
    case "{{ arco_binary }}" in \
        *target/release/arco) just build-cli-release ;; \
        *) just build-cli ;; \
    esac
