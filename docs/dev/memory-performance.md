# Measuring memory performance

For the canonical model's per-column storage budget, see
[Column storage memory](column-storage-memory.md).

That page also describes the private sparse display-index mapping used by the
Python binding and its payload accounting limits.

For short-expression normalization allocation behavior, see
[Short expression memory](short-expression-memory.md).

For releasing the source model before HiGHS optimization, see
[HiGHS model ownership](highs-model-ownership.md).

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

## Compact constraint rows

`Model::add_constraints_compact` derives consecutive source row positions from
the bounds length and streams them directly into the same insertion loop used
by `add_constraints_compact_indexed`. It does not materialize a temporary
`usize` index vector. The indexed API still borrows the caller's row positions
and retains its existing zip truncation and partial-error behavior.

The removed buffer is sized by one compact insertion batch. Its upper bound is
the largest batch's row count; it must not be estimated from the formulation's
aggregate reported constraint count. Eliminating this temporary reduces
construction allocations without changing stored columns, coefficient counts,
or later model mutation. It does not by itself establish a lower process RSS;
measure the same compact batches in fresh processes when evaluating the effect.

## Sparse differences

Sparse `np.diff(array, axis=...)` maps each active source slot to at most two
output slots: the positive source at the preceding axis coordinate and the
negative source at the current coordinate. Sparse active indices are maintained
in strictly increasing row-major order by the internal constructors and
producer paths. The implementation merges those two monotonic streams, so it
does not allocate the previous `O(active_slots)` pair buffer or sort cloned
expressions. Variable arrays pass borrowed variable IDs into the merge and
materialize expressions only for output rows.

Sparse row membership remains observable through `memory_estimate()` and model
inspection. A row with two nonzero contributions stays active even when the
contributions cancel; a row whose available contributions are both exactly zero
is omitted. Tests should cover inactive holes, each axis boundary, singleton
and empty axes, zero-valued expressions, and broadcast expressions that reuse a
variable across the differenced axis.

The merge reduces temporary allocation during expression construction. It does
not by itself establish a lower process RSS: use a fresh-process build probe
and record dimensions, active-slot counts, coefficient counts, and RSS as
specified above. The focused behavioral checks are:

```bash
bindings/python/.venv/bin/pytest \
  bindings/python/tests/test_active_masks.py \
  bindings/python/tests/test_axis_param.py \
  bindings/python/tests/test_param_api.py -q
```

## Canonical coefficient count

`Model::num_coefficients()` is an O(1) read backed by one cached `usize` per
model. The cache counts physical entries in the column storage, including
duplicate row IDs and explicit zero values. It is incremented only when a
matrix entry is actually inserted; updating an existing `(variable,
constraint)` entry does not change the count. Expression normalization still
controls which terms reach storage, so this cache does not change duplicate,
zero, cancellation, sparse export, fingerprint, or partial-error semantics.

The compact and streaming batch insertion paths update the count after each
successful push. CSC import updates it after each fully validated column is
installed, so malformed input still cannot make a returned model report a
different count from its stored entries. Cloning a model copies the cached
value with the rest of the model. The trade-off is one `usize` per canonical
model and one increment on each new stored entry; callers that need to inspect
the columns should still use `Model::columns()`.

For a bounded black-box query check, construct the same one-million-variable
model in fresh release processes, use a bounded mix of empty and one-entry
columns, and repeat `num_coefficients()` through `std::hint::black_box`. Report
individual query and construction samples, process peak RSS, and the exact Rust
revision/profile. This isolates the count-query cost while checking insertion
overhead; it does not establish a lower construction peak or end-to-end solver
memory use.

## Reusing sparse constraint arrays

When a sparse `ConstraintArray` is inserted into a model, normalization
borrows each stored expression and returns an owned normalized coefficient
vector. The same constraint array can still be inserted more than once, with or without an
active mask, without cloning each expression first. Normalization still merges
duplicate variable terms and retains rows whose terms cancel to zero so row
selection and model inspection remain unchanged.

The focused regression is in
`bindings/python/tests/test_active_masks.py`:

```bash
bindings/python/.venv/bin/pytest \
  bindings/python/tests/test_active_masks.py \
  -k sparse_constraint_array_reuse_normalizes_terms -q
```

Sparse array-to-array comparisons defer row merging until insertion. The
comparison retains references to the immutable sparse source arrays and applies
the active mask while streaming the merged rows, so inactive union rows do not
first become `PyExpr` and RHS buffers. This applies to sparse expression and
sparse variable operands with matching shapes; axis broadcasting continues to
use the materialized comparison path. Calling the public `rhs` or indexing
accessors still computes the same visible values on demand, and the source
arrays remain reusable for later comparisons and insertions.

The focused checks are:

```bash
PYTHONPATH=bindings/python bindings/python/.venv/bin/pytest \
  bindings/python/tests/test_active_masks.py \
  -k 'sparse_comparison_applies_active_mask or sparse_comparison_can_be_reused' -q
```

## Reusing owned materialized rows

The materialized Python constraint paths receive expressions that they already
own. When an expression contains finite, nonzero linear terms with strictly
increasing unique variable IDs, insertion consumes that expression and reuses
its linear-term buffer. Duplicate, unsorted, zero, NaN, and infinite terms use
the existing normalization path, preserving its duplicate accumulation and
zero filtering. For expressions longer than two terms, this also avoids the
temporary `HashMap` used by the general normalizer. The borrowed sparse
insertion path is unchanged.

The comparison path reads an expression's constant before consuming its owned
linear terms, so constants continue to become constraint bounds. Reusing the
same materialized or comparison array remains supported. This removes a
temporary coefficient vector for eligible rows; it does not by itself
establish a lower process RSS or change the stored matrix.

The focused checks are:

```bash
RUSTUP_TOOLCHAIN=1.85.1 CARGO_BUILD_JOBS=1 \
  scripts/with_solver_build_env.sh cargo test -p arco-python-core --lib \
  py_modules::model_edit::tests

PYTHONPATH=bindings/python bindings/python/.venv/bin/pytest \
  bindings/python/tests/test_owned_normalize.py -q
```

## Streaming full materialized rows

Full `ConstraintArray` insertion accepts an `ExactSizeIterator` of expressions.
The array-backed callers borrow the source expression slice and clone one
expression as the insertion iterator advances, so they do not first allocate a
second vector containing the complete batch. The row count is read from the
iterator before active-mask resolution; mask and shape errors therefore leave
the source unconsumed and the model unchanged. Once insertion starts, the
existing row order, normalization, and partial-error behavior remain in force:
the model can contain rows inserted before a later invalid coefficient is
reported.

This removes a full-batch expression-vector allocation from the materialized
path. It does not remove the expression storage held by a source array or
establish a lower whole-process RSS. Reusing a source `ConstraintArray` remains
supported, including repeated insertion with different active masks.

## Solver loading

The primitive model retains its coefficient columns while adapters construct
solver input arrays. Count both representations and any solver-owned copy when
estimating the loading peak. Also account for vector capacity rather than only
length, row and column metadata, objective arrays, and integer-variable data.

Input arrays passed through FFI may be released only after the solver has copied
their contents. Scope temporary buffers to the loading operation once that
ownership contract is established. Test LP and MIP paths, inactive variables,
empty columns, bounds, and coefficient ordering when changing this boundary.

### HiGHS input buffers

HiGHS copies the arrays passed through `Highs_passLp` and `Highs_passMip` into
its native model during the load call. The HiGHS 1.15.0 implementation uses
`std::vector::assign` for those fields; see the [official source](https://github.com/ERGO-Code/HiGHS/blob/v1.15.0/highs/lp_data/Highs.cpp#L533-L624).
The adapter therefore scopes its objective, bounds, integrality, and CSC
buffers to preparation. Those temporary arrays are released before
`Highs_run`; the native model and its solver-owned factorization remain live
through optimization.

When callers use the [HiGHS model ownership](highs-model-ownership.md) API and
drop the source `ModelView` after loading, only native solver state,
dimensions, fingerprint choice, extraction choice, and timing metadata remain.
The existing borrowing solve path remains available for callers that need the
source model throughout the solve.

Run the ownership regressions with the pinned toolchain:

```bash
scripts/with_solver_build_env.sh rustup run 1.85.1 cargo test -p arco-highs --lib prepared_model_
```

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
