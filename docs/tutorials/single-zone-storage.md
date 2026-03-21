# Tutorial: Re-implement the PyPSA Single-Zone Storage Model

This tutorial is a re-implementation in `arco-dsl` of the PyPSA model
[`Single bidding zone with fixed load and storage, several periods`](https://docs.pypsa.org/latest/examples/simple-electricity-market-examples/#single-bidding-zone-with-fixed-load-and-storage-several-periods).

The goal is simple:

1. run the Arco version of the example
2. verify the total system cost
3. inspect the solved variable values

By the end, you will have run the exact fixture at
[`tests/e2e/simple-electricity-market-storage/input.kdl`](../../tests/e2e/simple-electricity-market-storage/input.kdl)
and compared its results against the published PyPSA example.

## Before You Start

You need a working Rust toolchain and the private Arco crates configured for
this repository.

From the repository root, the fixture lives at:

- [`tests/e2e/simple-electricity-market-storage/input.kdl`](../../tests/e2e/simple-electricity-market-storage/input.kdl)
- [`tests/e2e/simple-electricity-market-storage/data/load.csv`](../../tests/e2e/simple-electricity-market-storage/data/load.csv)
- [`tests/e2e/simple-electricity-market-storage/data/availability.csv`](../../tests/e2e/simple-electricity-market-storage/data/availability.csv)

## Step 1: Read The KDL Model

The fixture separates generators and storage into two technologies but uses one
shared `dispatch[a,t]` family so the system balance reads naturally.

Generators are modeled as non-negative dispatch with an availability cap:

```kdl
technology "Generator" {
  control "dispatch"
}

operation "GeneratorDispatch" {
  constraint "generator_limit" {
    dispatch[a,t] <= capacity_mw[a] * availability[a,t]
  }
}
```

Storage uses the same `dispatch[a,t]` family, but it is signed:

- positive `dispatch` means injection into the market
- negative `dispatch` means charging

Its state transition is:

```kdl
technology "Storage" {
  control "dispatch"
  state "soc"
}

operation "StorageDispatch" {
  constraint "soc_balance" {
    soc[a,t] = soc[a,t-1] - dispatch[a,t]
  }
}
```

The system clears one bidding zone against a fixed load:

```kdl
rule "SingleZoneBalance" {
  constraint "balance" {
    sum(dispatch[a,t] for a in assets) = load[t]
  }
}
```

## Step 2: Run The Example

Run the fixture through the current pipeline:

```bash
cargo run -- run tests/e2e/simple-electricity-market-storage/input.kdl
```

The output is JSON. The most important fields are:

```json
{
  "mode": "simple_electricity_market",
  "active_scenario": "SouthAfricaSingleZoneWithStorage",
  "objective": {
    "name": "TotalSystemCost",
    "sense": "minimize",
    "value": 6046000.0
  }
}
```

PyPSA prints the objective as `6.05e+06`. Arco reports the full scalar value
`6046000.0`, which is the same result to the displayed precision on the PyPSA
page.

## Step 3: Inspect The Solved Variables

The `variables` section in the JSON output contains the full time series for
each variable family.

For `dispatch[a,t]`, Arco returns:

| Variable                  |   Value |
| ------------------------- | ------: |
| `dispatch[Coal,1]`        | `35000` |
| `dispatch[Coal,2]`        | `35000` |
| `dispatch[Coal,3]`        | `35000` |
| `dispatch[Coal,4]`        | `35000` |
| `dispatch[Wind,1]`        |   `900` |
| `dispatch[Wind,2]`        |  `1800` |
| `dispatch[Wind,3]`        |  `1200` |
| `dispatch[Wind,4]`        |  `1500` |
| `dispatch[Gas,1]`         |  `7100` |
| `dispatch[Gas,2]`         |  `7000` |
| `dispatch[Gas,3]`         |  `8000` |
| `dispatch[Gas,4]`         |  `8000` |
| `dispatch[Oil,1]`         |     `0` |
| `dispatch[Oil,2]`         |     `0` |
| `dispatch[Oil,3]`         |     `0` |
| `dispatch[Oil,4]`         |   `500` |
| `dispatch[PumpedHydro,1]` | `-1000` |
| `dispatch[PumpedHydro,2]` |  `-800` |
| `dispatch[PumpedHydro,3]` |   `800` |
| `dispatch[PumpedHydro,4]` |  `1000` |

For `soc[a,t]`, Arco returns:

| Variable             |  Value |
| -------------------- | -----: |
| `soc[PumpedHydro,1]` | `1000` |
| `soc[PumpedHydro,2]` | `1800` |
| `soc[PumpedHydro,3]` | `1000` |
| `soc[PumpedHydro,4]` |    `0` |

## Step 4: Compare Against PyPSA

The PyPSA page shows the same economic outcome:

- coal fully loaded in every hour
- wind dispatched to availability
- gas filling the remaining demand before oil
- storage charging in the first two hours and discharging in the last two
- oil only needed in hour 4

There is one important detail: the first two charging hours are degenerate.
PyPSA publishes storage dispatch `[-800, -1000, 800, 1000]`, while Arco with
HiGHS returns `[-1000, -800, 800, 1000]`.

Both solutions are optimal because:

- hours 1 and 2 have the same marginal charging cost
- the storage power limit is `1000 MW`
- the model only needs `1800 MWh` of stored energy before hour 3

So the cost-optimal solution is not unique in those first two hours. The
repository test accepts both the published PyPSA schedule and the equivalent
Arco schedule.

## Step 5: Verify It With Tests

The exact regression test is:

- [`tests/execution_suite.rs`](../../tests/execution_suite.rs)

Run just that check with:

```bash
cargo test --test execution_suite executes_simple_electricity_market_storage_fixture_with_expected_dispatch
```

That test verifies:

1. the objective value is `6046000.0`
2. the dispatch trajectory matches the PyPSA optimum or the equivalent
   degenerate optimum
3. the storage state trajectory matches the corresponding dispatch schedule

## What You Learned

You now have a working Arco tutorial for a multi-period single-zone market with:

- fixed demand
- generator availability limits
- storage charging and discharging
- full solved variable extraction from the CLI output

If you want the underlying fixture, use
[`tests/e2e/simple-electricity-market-storage/input.kdl`](../../tests/e2e/simple-electricity-market-storage/input.kdl)
as the starting point.
