# Sparse net injection example

Demonstrates a sparse workaround for bus-level net injection when `mw_load` is only defined on `feasible_existing_links`.

Pattern:

- reduce `dispatch_new_gen` from `feasible_links` to bus with projection `b_new`
- reduce `dispatch_existent_gen` from `feasible_existing_links` to bus with projection `b_exist`
- pre-aggregate `mw_load_per_existing_bus` in the `data` block with `index=b reduce=sum` filtered to `existing > 0`
- define `net_injection_existing_bus[b]` only on the existing-bus footprint, avoiding a dense all-bus load object

Run from repo root:

```bash
cargo run -q -p arco-cli -- validate examples/sparse-net-injection/input.kdl
cargo run -q -p arco-cli -- print-model examples/sparse-net-injection/input.kdl
```
