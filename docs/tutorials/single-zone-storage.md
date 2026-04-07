# Tutorial: Single-Zone Dispatch (Low-Level KDL)

This tutorial walks through the low-level KDL fixture at
`examples/simple-electricity-market-storage/input.kdl`.

The fixture demonstrates the low-level profile only:

- top-level `data` blocks
- one `model` block
- one `scenario` block
- `set`, `param`, `control`, `expression`, and `constraint` declarations

## Run The Example

From the repository root:

```bash
cargo run -p arco-cli -- run examples/simple-electricity-market-storage/input.kdl
```

## Inspect The Model Structure

```bash
cargo run -p arco-cli -- inspect examples/simple-electricity-market-storage/input.kdl --section sets
cargo run -p arco-cli -- inspect examples/simple-electricity-market-storage/input.kdl --section constraints
```

## Validate Without Solving

```bash
cargo run -p arco-cli -- validate examples/simple-electricity-market-storage/input.kdl
```

## What This Example Shows

- CSV-backed `data` namespaces (`units`, `availability_data`, `load_data`)
- Model-level indexing via `index` child nodes
- Horizon-bound time domain via `set time from=horizon`
- Algebra reductions over named sets (`sum(... for a in asset_id for t in time)`)
- Scenario-level report requests (`report DispatchCost`)
