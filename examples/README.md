# Arco examples

This directory contains Arco KDL examples and supporting fixtures for the main CLI workflows, from small dispatch models to larger benchmark-style and power-system formulations.

Most example folders follow the same layout:

- `input.kdl`, the model entrypoint
- `data/`, CSV fixtures loaded by the model
- optional helper scripts such as `formulation.py` for cross-checks or benchmark generation

## Quick start

From the repository root, validate, inspect, print, or solve an example with `arco-cli`:

```bash
cargo run -p arco-cli -- validate examples/nodal-allocation/input.kdl
cargo run -p arco-cli -- inspect examples/nodal-allocation/input.kdl --section constraints
cargo run -p arco-cli -- print-model examples/nodal-allocation/input.kdl
cargo run -p arco-cli -- run examples/nodal-allocation/input.kdl
```

For larger models, `--compact` keeps the solver output readable:

```bash
cargo run -p arco-cli -- run examples/unit-commitment/input.kdl --compact
```

## Example catalog

| Example                                | Path                                                   | Purpose                                                                                                                         | Status                                    |
| -------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| Nodal allocation                       | `examples/nodal-allocation/input.kdl`                  | Tuple-domain tracer bullet for feasible node-generator allocations, explicit subsets, and sparse tuple-membership lowering.     | Ready                                     |
| Generator allocation                   | `examples/generator-allocation/input.kdl`              | Smallest Cartesian indexed dispatch example for validating the legacy core CLI flow.                                            | Ready                                     |
| Price-taker battery                    | `examples/price-taker-battery/input.kdl`               | Battery charge and discharge scheduling against an exogenous price curve.                                                       | Ready                                     |
| Simple electricity market with storage | `examples/simple-electricity-market-storage/input.kdl` | Single-zone dispatch with time-varying availability, load, and storage decisions.                                               | Ready                                     |
| Capacity expansion                     | `examples/capacity-expansion/input.kdl`                | Build versus dispatch tradeoffs, candidate assets, and unmet-demand penalties.                                                  | Ready                                     |
| DCOPF, angle formulation               | `examples/dcopf-angle/input.kdl`                       | Three-bus DC optimal power flow in the voltage-angle form, adapted from PSOPTLIB OF3bus.                                        | Ready                                     |
| DCOPF, PTDF formulation                | `examples/dcopf-ptdf/input.kdl`                        | The same OF3bus case written with PTDF flow equations for formulation comparison.                                               | Ready                                     |
| Unit commitment                        | `examples/unit-commitment/input.kdl`                   | Mixed-integer unit commitment with startup, shutdown, ramping, and piecewise costs, adapted from PSOPTLIB UC.                   | Ready                                     |
| Dense LP benchmark                     | `examples/dense-lp/input.kdl`                          | Synthetic dense LP used to stress model construction and compare against the bundled Python formulation.                        | Ready                                     |
| SDOM                                   | `examples/sdom/input.kdl`                              | Storage deployment optimization with renewables, thermal capacity, storage sizing, and policy-style generation mix constraints. | Ready                                     |
| DEAD + ESS + wind, linearized          | `examples/ded-ess-wind-linearized/input.kdl`           | Linearized dynamic economic dispatch with ramping, storage state of charge, and wind curtailment.                               | Incomplete, `data/storage.csv` is missing |

## Python-backed examples

Two examples also ship a Python formulation so you can compare the KDL model with a direct Python implementation:

```bash
uv run examples/dense-lp/formulation.py --solve --json
uv run examples/sdom/formulation.py --solve --json
```

For interactive exploration of dense-lp (inspect model, then solve from a REPL):

```bash
uv run --with ipython --with-editable ./bindings/python ipython -i examples/dense-lp/formulation.py
```

This leaves `model`, `solve()`, `solution`, and `payload` available in the prompt.

## Suggested walkthrough

If you are new to the repo, this order ramps up nicely:

1. `nodal-allocation`, to see tuple-domain feasible-set semantics and explicit subsets in a small end-to-end flow
2. `generator-allocation`, to compare against a simple Cartesian indexed formulation
3. `price-taker-battery`, to see time coupling and storage dynamics
4. `capacity-expansion`, to see investment-style modeling
5. `dcopf-angle` and `dcopf-ptdf`, to compare equivalent network formulations
6. `unit-commitment` or `sdom`, when you want a heavier mixed-integer case

## Tuple-domain diagnostics contract

The `nodal-allocation` example is the user-facing tracer bullet for tuple-domain
V1 behavior:

- `dispatch[a,i,g,b]` is instantiated only for members of `feasible_links`.
  There is no Cartesian fallback.
- Reduced or filtered scopes are modeled with named subsets such as
  `priority_links`; V1 does not auto-project high-dimensional tuple domains.
- Compile-time diagnostics use scoped identifiers and canonical ordering. For
  example:

  ```text
  index order mismatch for `NodalAllocationDay.NodalAllocation.constraint_1` over tuple domain `feasible_links`
  empty constraint-relevant tuple subset for `NodalAllocationDay.NodalAllocation.capacity_target` at keys: north,solar; north,wind; south,gas; south,solar
  ```

## Current caveats

- All commands are intended to be run from the repository root.
- Each runnable example directory contains its own `input.kdl` and any required CSV fixtures in `data/`.
- `examples/infeasible/` currently holds fixture data only and is not listed as a runnable example yet.
- `examples/ded-ess-wind-linearized/input.kdl` currently references `data/storage.csv`, which is not present in the repository.
