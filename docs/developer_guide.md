# Developer Guide

This guide is the practical local workflow for Arco contributors.

## Prerequisites

- Rust toolchain (repo targets Rust 1.85.x)
- `just` (task runner)
- `uv` (Python package/env runner)
- Python 3.12+ recommended for local bindings workflows

Install `just` if needed:

```bash
cargo install just --locked --version 1.43.0
```

## First-time setup

From repo root (recommended before running anything else):

```bash
just setup
just py-sync
```

## Build and run locally

### Install CLI into your cargo bin

```bash
cargo install --path ~/dev/arco/crates/arco-cli --force --locked
```

### Run CLI without installing

```bash
cargo run -p arco-cli -- --help
cargo run -p arco-cli -- run examples/dense-lp/input.kdl --compact
```

### Build Python extension in editable mode

```bash
just py-dev
```

## Day-to-day development loops

### Fast Rust loop (single crate)

```bash
just check-pkg arco-ops
just test-pkg arco-ops
just clippy-pkg arco-ops
```

### Fast Python loop

```bash
just py-fmt-check
just py-lint-check
just py-type
just py-test
```

### Hawk visibility analysis

Hawk checks the public Rust surface reachable from the shipped `arco` CLI.
It uses Rust 1.97.1 because Hawk is coupled to the compiler version it was
built against.

Install the pinned release and run the local check with:

```bash
rustup toolchain install 1.97.1
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/astral-sh/hawk/releases/download/0.1.9/cargo-hawk-installer.sh | sh
just hawk
```

The configuration lives in `hawk.toml`. The default feature profile is
intentional: Arco's SCIP backends are mutually exclusive, so Hawk's default
`--all-features` profile cannot be used for this workspace. CI enforces Hawk
with `-D warnings`; `dead_code` is allowed separately because Hawk is the
workspace's public-visibility check and restricted test/optional-feature APIs
must not remain public solely to suppress Rust's dead-code lint.

### Docs and examples loop

```bash
just docs-test
just kdl-examples
```

## Benchmarking

For allocation changes and backend comparisons, follow the
[memory measurement contract](dev/memory-performance.md).

Run curated CLI benchmarks with:

```bash
just benchmarks
```

The benchmark harness isolates Arco solver configuration by default so local
`~/.config/arco/solver.toml` or project `.arco/solver.toml` files cannot change
the measured backend. This keeps benchmark runs comparable across developer
machines and CI.

Each case/workflow records median duration from the configured timing
repetitions and peak RSS from one additional monitored probe. That keeps the
same scenario coverage while avoiding long memory-sampling loops on every
timing repetition.

## Architecture policy

Arco uses a repo-local architecture contract:

- `architecture-layers.toml`
- `scripts/check_architecture.py`

Run policy checks with:

```bash
just arch-check
```

Rules are strict:

- every workspace crate must be classified in `architecture-layers.toml`
- unknown/unclassified crates fail the check
- disallowed crate-to-crate layer edges fail the check

When adding a new crate, update `architecture-layers.toml` in the same change.

## GitHub workflow tips

- Open draft PRs early for visibility and CI signal.
- Keep PRs scoped; split unrelated changes before review.
- Reference issues in PR body (`Closes #123`) only when fully resolved.
- Prefer force-push only on your feature branch; avoid rewriting shared history.
- Re-run checks after resolving merge conflicts; don’t trust stale CI.
- If CI fails, post a short root-cause + fix note in the PR for reviewers.

Suggested PR checklist:

- [ ] `just arch-check` passes
- [ ] `just ci` passes
- [ ] docs updated for behavior/API changes
- [ ] migration/dependency impacts called out

## Full local CI-equivalent gate

Before pushing substantial changes:

```bash
just ci
```

This is the canonical pre-push validation path.

## Recommended pre-push checklist

1. Format/lint/tests pass for touched scope.
2. `just arch-check` passes.
3. `just ci` passes for broader changes.
4. Docs updated for any user-visible behavior/API changes.

## Building with optional solver features

Default builds exclude native IPOPT and Xpress runtimes. The shipped solver
selection command (`arco solver set ipopt`) is available in the default product
and returns a clear unavailable diagnostic unless the binary was compiled with
native IPOPT support.

To build the CLI with Xpress SDK support:

```bash
just build-cli-feature xpress
```

To build with both Xpress and IPOPT:

```bash
cargo build -p arco-cli --bin arco --features ipopt,xpress
```

Workspace-wide Rust `just` targets run through the solver build environment
wrapper, which reuses prebuilt SCIP and HiGHS artifacts when the current target
has a supported archive. When no safe HiGHS archive is available, the wrapper
builds a native HiGHS cache under `~/.cache/arco-highs` and exposes it through
`pkg-config`, so fresh Cargo targets do not rebuild HiGHS. Set
`ARCO_HIGHS_ENABLE_SOURCE_CACHE=0` to force the old `highs-sys` source-build
fallback.

Check and test recipes opt in to the official Apple Silicon HiGHS static archive
for faster local validation. Product build recipes and direct calls to
`scripts/setup_highs_binary_env.sh` only use that archive when
`ARCO_HIGHS_ENABLE_APPLE_STATIC=1` is set; release jobs should opt in only after
confirming the archive's macOS deployment target is acceptable for that product.
The source-built macOS cache defaults to `MACOSX_DEPLOYMENT_TARGET=11.0` unless
`ARCO_HIGHS_MACOS_DEPLOYMENT_TARGET` or `MACOSX_DEPLOYMENT_TARGET` overrides it.
Linux HiGHS static archives require glibc 2.38 or newer; older Linux images use
the source-built cache when native build tools are present. Windows release
builds also use the source-built cache by default because the official archive
can require a newer MSVC STL than a release runner provides; set
`ARCO_HIGHS_ENABLE_WINDOWS_STATIC=1` only after validating the target toolchain.
SCIP-enabled Linux product builds also need the GNU Fortran runtime
(`libgfortran.so.5`; on Debian/Ubuntu install `libgfortran5` or `gfortran`).

> [!NOTE]
> IPOPT is intentionally outside the normal `--all-features` workspace path.
> The `arco-ipopt` crate compiles without native IPOPT libraries; this
> repository ships the selection surface and unavailable diagnostics, while
> native solve execution is provided by an external adapter build.

## Troubleshooting

### `just ci` fails in optional solver environments

Some solver backends may require external SDK/runtime setup depending on target.
Use package-scoped commands (`just test-pkg`, `just clippy-pkg`) while iterating,
then run full `just ci` in a fully provisioned environment.

### Python binding import/runtime issues

Rebuild editable extension:

```bash
just py-dev
```

Then re-run tests:

```bash
just py-test
```

### Architecture check fails after crate changes

Update `architecture-layers.toml` for:

- new crate classification
- intentional dependency overrides (only when justified)

## PR body template tip

Use concise sections in GitHub PR descriptions:

1. **Summary**: what changed
2. **Why**: problem/risk addressed
3. **Validation**: exact commands run
4. **Follow-ups**: explicit non-goals or deferred work
