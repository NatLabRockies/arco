# ReEDS benchmark

Python-only Arco benchmark adapted from the ReEDS framework comparison. It builds a simplified single-vintage capacity-expansion LP with sparse active tech/region/year domains, transmission, ramping, emissions, and storage constraints.

Run from repo root:

```bash
uv run examples/reeds-benchmark/formulation.py --size small --json
uv run examples/reeds-benchmark/formulation.py --size medium --build-only --json
```

Sizes: `small`, `medium`, `large`, `xlarge`.

Use this example as the baseline for improving the Arco Python dense-array UX and reducing formulation LOC.
