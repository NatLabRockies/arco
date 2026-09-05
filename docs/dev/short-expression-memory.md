# Short expression normalization memory

`Expr::normalized_terms()` uses a small linear merge for expressions with at
most two linear terms. It skips exact-zero coefficients before merging,
accumulates duplicate IDs from a zero starting value, and removes coefficients
that cancel to exact zero. Larger expressions keep the existing `HashMap`
implementation. Normalized term order is unspecified.

The short path keeps the owned output `Vec`, so its main benefit is avoiding the
temporary hash table and its allocation traffic. That can reduce frontend
working memory and allocator pressure, but it does not by itself establish a
lower process RSS or a lower whole-solve peak; allocator retention and solver
memory remain separate measurements.

`crates/arco-model/tests/short_normalize.rs` checks behavior for zero,
duplicate, cancellation, nonfinite, and larger expressions. Its allocation
budget uses a test-only `GlobalAlloc` wrapper with a thread-local counter and
checks an upper bound of one allocation for one- and two-term normalization.
The wrapper is never part of the application allocator, and the test does not
interpret an allocation count as a minimum or exact implementation contract.
The counter follows the standard library's
[allocator safety requirements](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html#safety),
including the prohibition on unwinding and the possibility of optimized-away
allocations.
