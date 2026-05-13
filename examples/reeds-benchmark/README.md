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
