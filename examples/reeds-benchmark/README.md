# ReEDS benchmark

Small ReEDS-style KDL benchmark for capacity expansion and dispatch.

The KDL files are committed, but CSV inputs are generated into a temporary
working directory so the repository does not carry generated data files.

Run from the repository root:

```bash
REEDS_KDL_INPUT=$(uv run examples/reeds-benchmark/prepare_kdl_contract.py)
cargo run -p arco-cli -- validate "$REEDS_KDL_INPUT"
cargo run -p arco-cli -- run "$REEDS_KDL_INPUT" --compact
```

For comparison, the Python benchmark remains available:

```bash
uv run examples/reeds-benchmark/formulation.py --size small --json
```

To capture build-stage RSS and sparse matrix diagnostics for memory regression
work:

```bash
uv run examples/reeds-benchmark/formulation.py --size small --json --profile-build --profile-matrix
```

Memory fields report `null` in JSON, or `n/a` in text output, when a platform
does not expose the measurement. They do not use `0.0` as an unavailable-memory
sentinel.
