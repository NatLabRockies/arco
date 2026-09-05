# HiGHS objective loading

The direct HiGHS adapter builds one dense objective vector and transfers
ownership of it into its load data. It does not allocate and populate a second
vector for the same coefficients. Sparse objectives still produce zeroes for
variables without objective terms; model objective normalization is unchanged.

This removes one allocation of `8 * num_variables` bytes and one linear copy.
For 3,486,872 variables, that is 27,894,976 bytes (about 26.6 MiB) of live Rust
buffer storage. This is an allocation calculation, not a measured process RSS
reduction. The model, remaining loading arrays, and solver-owned storage still
contribute to the overall peak.

The load operation consumes the vector, so it is released with the other
temporary input arrays when the HiGHS load call returns. Previously, the
separate objective vector remained in the outer solve scope.

The allocation regression checks that the input vector and loaded objective
have the same allocation address. Solve coverage checks sparse, empty, and
duplicate objective terms.

Run the focused checks from the repository root with its pinned Rust toolchain:

```bash
RUSTUP_TOOLCHAIN=1.85.1 scripts/with_solver_build_env.sh just test-one arco-highs direct_load_
RUSTUP_TOOLCHAIN=1.85.1 scripts/with_solver_build_env.sh just clippy-pkg arco-highs
```

Use the [memory measurement contract](memory-performance.md) for whole-process
comparisons. No solver algorithm, tolerance, or public API change is required.
