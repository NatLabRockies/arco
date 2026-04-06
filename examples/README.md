# Workspace KDL Examples

These are canonical KDL examples at the workspace root.

## Available examples

- `examples/generator-allocation/input.kdl`
- `examples/price-taker-battery/input.kdl`
- `examples/simple-electricity-market-storage/input.kdl`
- `examples/capacity-expansion/input.kdl`

Each folder contains the model `input.kdl`, and any required CSV fixture data in `data/`.

## Run an example

```bash
cargo run -p arco-cli -- run examples/generator-allocation/input.kdl
cargo run -p arco-cli -- run examples/price-taker-battery/input.kdl
```
