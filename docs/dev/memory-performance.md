# Measuring memory performance

Measure model construction, solver loading, optimization, and solution
extraction separately when evaluating an allocation change. A lower allocation
count or smaller Rust buffer is useful evidence, but does not establish a lower
whole-process peak or parity with another modeling backend.

## Comparison contract

Record the Arco revision, benchmark revision, input dimensions and seed, build
profile, Rust and Python versions, solver version, operating system, CPU, and
available memory with each result. Build the local revision before timing;
confirm that Python imports that build rather than a released wheel.

Run each repetition in a fresh process on the same machine class. Keep solver
algorithm, crossover, presolve, thread count, tolerances, and solution extraction
settings fixed. Report individual samples and their median; report memory
pressure or swapping that could distort runtime.

Check variable, constraint, and coefficient counts, solve status, objective
value, and primal feasibility before interpreting a performance difference.
Report absolute and relative tolerances. Do not accept lower memory obtained by
omitting constraints, weakening tolerances, or returning an incomplete solution.

For cross-solver comparisons, report both solver configurations explicitly.
Differences in factorization, presolve, or crossover memory cannot be attributed
to the modeling frontend from a total-process peak alone. GAMS comparisons must
include its solver process as well as model generation; a parent-process RSS
measurement alone is insufficient for subprocess-based backends.
Taking the maximum of parent and child high-water marks also does not measure
their concurrent total, while adding their separate peaks can overestimate it.
Use a consistently scoped job/process-tree measurement for the comparison and
retain per-process measurements as diagnostics. Report sampling intervals and
any shared-page accounting limitations.

## Existing construction probe

The [ReEDS-style example](../../examples/reeds-benchmark/README.md) provides
build-stage RSS and matrix diagnostics. Its Python script declares a local
source dependency on `bindings/python` and requires Python 3.12 or newer.
Start with the small case and increase the size only when memory permits.

```bash
uv run examples/reeds-benchmark/formulation.py --size small --build-only --json --profile-build --profile-matrix
```

The `--build-only` result measures construction, not solver loading or solve
memory. Matrix diagnostics can allocate additional working storage: use the
same profiling flags in both revisions and collect final timing samples without
diagnostic work. The example currently solves with HiGHS; it does not measure
Xpress or GAMS performance.

Unavailable memory measurements are `null` in JSON and `n/a` in text. Current
RSS and process high-water RSS are different measurements. Releasing Rust
buffers reduces live allocations but an allocator may retain those pages, and
a previous high-water mark cannot decrease during a run. Record both allocated
buffer bytes and process peak where possible.

## Solver loading

The primitive model retains its coefficient columns while adapters construct
solver input arrays. Count both representations and any solver-owned copy when
estimating the loading peak. Also account for vector capacity rather than only
length, row and column metadata, objective arrays, and integer-variable data.

Input arrays passed through FFI may be released only after the solver has copied
their contents. Scope temporary buffers to the loading operation once that
ownership contract is established. Test LP and MIP paths, inactive variables,
empty columns, bounds, and coefficient ordering when changing this boundary.

### Xpress input buffers

The Xpress adapter counts entries in active columns before allocating its CSC
row-index and value arrays. This adds a read-only pass over the coefficients,
but avoids geometric vector growth and spare matrix capacity. It preserves the
existing treatment of inactive variables and out-of-range rows.

Loading takes ownership of all input buffers and releases them before calling
the optimizer. For a pure LP with `n` columns, `m` rows, and `z` loaded entries,
the buffer payload is `28*n + 17*m + 12*z + 4` bytes with 32-bit Xpress indices.
This excludes vector headers and allocator overhead. Integer-variable buffers
are allocated only for MIP data. Previously, the loading buffers remained live
through optimization, including excess matrix capacity from vector growth.

Xpress imports the supplied arrays into its problem object. Its
[official LP/MIP interface example](https://examples.xpress.fico.com/example.pl?id=myxprs)
separates loading from optimization. The adapter's owned load-data boundary
keeps all pointers valid for the load call and prevents their reuse afterward.
Small native LP and MIP solves exercise that boundary when an Xpress runtime
and license are available; tests that return early without a runtime do not
establish native solver correctness.

```bash
just test-one arco-xpress load_data_
just clippy-pkg arco-xpress
cargo test -p arco-xpress --test integration -- --test-threads=1
```

Set `XPRESSDIR` to the runtime installation and `XPAUTH_PATH` to its license
file for native integration checks. Keep license contents out of results.

### Releasing the source model before Xpress optimization

Rust callers can load a model with `arco_xpress::PreparedXpressModel::prepare`.
The returned object owns the native problem and has no borrow of the source
`ModelView`. After preparation succeeds, callers may drop their source model
before optimization:

```rust,ignore
use arco_xpress::PreparedXpressModel;

let prepared = PreparedXpressModel::prepare(&model, &config)?;
drop(model);
let result = prepared.solve_model_view()?;
```

`solve_model_view` returns the shared result with the fingerprint captured at
preparation and validates vector lengths against captured dimensions. `solve`
returns the Xpress `Solution` type. Both methods consume the prepared problem;
native resources are released on success or error. Dropping an unsolved
prepared problem also releases its native resources. Preparation errors leave
the borrowed source model with the caller. After the caller drops that model,
a subsequent solve error cannot restore it.

Only one Arco Xpress session may be active in a process. Preparing another
problem while one exists returns a busy error immediately. Drop or solve the
first prepared problem before retrying. The session guard also covers ordinary
Xpress solves and prevents process-wide runtime shutdown while another Arco
problem is live. The prepared object must be used on its creating thread.

The preparation API honors `arco.fingerprint=false` and
`arco.extract_solution=false`. The `xpress_prepare_s` metadata includes loading
and any fingerprint calculation; `xpress_run_s` measures native optimization.
Time spent by the caller between preparation and solving is excluded.

Shared model-view validation skips recomputing the model fingerprint when a
result carries the zero fingerprint sentinel. It still validates every result
vector length; nonzero fingerprints continue to be compared with the input
model. This keeps fingerprint-disabled solves from paying for an unused hash
pass.

The existing borrowing solve APIs still retain their source models through
optimization. Python Xpress solves with `arco.consume_model=true` use the
prepared boundary to release the source model before optimization; see
[Manage model memory](../how-to/manage-model-memory.md) for the ownership and
error contract. Releasing source allocations can reduce live memory during
optimization, but allocator retention and the earlier loading peak can limit
process peak RSS savings. This API alone does not establish GAMS memory parity.

Run the lifetime and runtime cleanup regressions explicitly with a configured
Xpress runtime and license:

```bash
cargo test -p arco-xpress --test integration prepared_xpress_ -- --ignored
```

An allocation improvement should ship with a focused correctness regression,
a reproducible measurement, and an explicit statement of what remains
unmeasured. Keep allocator-specific tuning and machine-specific settings out
of the implementation.
