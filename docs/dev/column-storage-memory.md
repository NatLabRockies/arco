# Column storage memory

`Model` stores one private `ColumnVec` per variable. The public
`Model::columns()` and `ModelView::column()` APIs expose each column as a
borrowed `&[(ConstraintId, f64)]` slice, so the representation must keep each
column contiguous and must preserve the existing mutation and CSC import paths.

The workspace enables [`smallvec`'s `union` feature](https://docs.rs/smallvec/1.16.0/smallvec/#feature-flags)
for this storage.
With `smallvec 1.16`, `SmallVec<[(ConstraintId, f64); 2]>` stores its capacity
word and its inline or spilled data without an enum tag. The column header is
therefore 40 bytes on the supported 64-bit target instead of 48 bytes. Empty,
one-entry, and two-entry columns remain inline; the existing heap growth starts
at three entries.

The saving is 8 bytes per logical column. At 3,486,872 columns this is about
26.6 MiB of column headers. The actual process reduction depends on vector
spare capacity, allocator behavior, optional names and metadata, temporary
construction arrays, and solver-owned memory. Header accounting does not prove
a lower whole-process peak.

For the measurement protocol and fair process-level comparisons, see
[Measuring memory performance](memory-performance.md). Do not compare a cached
backend probe or a source-buffer estimate directly with a solver RSS result.
