# Workspace KDL Examples

These are canonical KDL examples at the workspace root.

## Available examples

- `examples/generator-allocation/input.kdl`
- `examples/price-taker-battery/input.kdl`
- `examples/simple-electricity-market-storage/input.kdl`
- `examples/capacity-expansion/input.kdl`
- `examples/ded-ess-wind-linearized/input.kdl` (linearized version of Soroudi DEAD + ESS + wind case)
- `examples/tep-sparse-mip/input.kdl` (TEP-style linear MIP with sparse corridors and candidate subset)
- `examples/unit-commitment-mip/input.kdl` (UC-style linear MIP with binaries, startup, and ramping)
- `examples/transmission-expansion/input.kdl` (near 1:1 PSOPTLIB TEP port for GAMS parity)
- `examples/unit-commitment/input.kdl` (near 1:1 PSOPTLIB UC port for GAMS parity)
- `examples/dcopf-angle/input.kdl` (DCOPF angle formulation adapted from PSOPTLIB OF3bus)
- `examples/dcopf-ptdf/input.kdl` (equivalent PTDF formulation for the same DCOPF case)
- `examples/sparse-distance-lookup/input.kdl` (intentionally sparse data, used to reproduce a lowering failure)

Each folder contains the model `input.kdl`, and any required CSV fixture data in `data/`.

## Run an example

```bash
cargo run -p arco-cli -- run examples/generator-allocation/input.kdl
cargo run -p arco-cli -- run examples/price-taker-battery/input.kdl
cargo run -p arco-cli -- run examples/ded-ess-wind-linearized/input.kdl
cargo run -p arco-cli -- run examples/tep-sparse-mip/input.kdl
cargo run -p arco-cli -- run examples/unit-commitment-mip/input.kdl
cargo run -p arco-cli -- run examples/transmission-expansion/input.kdl --compact
cargo run -p arco-cli -- run examples/unit-commitment/input.kdl --compact
cargo run -p arco-cli -- run examples/dcopf-angle/input.kdl --compact
cargo run -p arco-cli -- run examples/dcopf-ptdf/input.kdl --compact

# intentionally fails during lowering (sparse distance table)
cargo run -p arco-cli -- print-model examples/sparse-distance-lookup/input.kdl
```

## Try all CLI flows on one example

```bash
cargo run -p arco-cli -- validate examples/generator-allocation/input.kdl
cargo run -p arco-cli -- print-model examples/generator-allocation/input.kdl
cargo run -p arco-cli -- inspect examples/generator-allocation/input.kdl --section constraints
cargo run -p arco-cli -- run examples/generator-allocation/input.kdl --compact
```
