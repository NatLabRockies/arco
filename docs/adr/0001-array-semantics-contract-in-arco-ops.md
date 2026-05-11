# Shared labeled-array semantics contract lives in arco-ops

We place the binding-agnostic array semantics contract in `arco-ops` and keep binding adapters (starting with Python) thin over that contract. The contract enforces day-one fail-fast semantics, requires equivalent syntax forms to produce identical coefficient sparsity/value results, and keeps raw-NumPy compatibility only for unambiguous suffix-aligned cases. This is chosen because it is hard to reverse, surprising without explicit capture, and reflects a real trade-off between strict semantic correctness and user convenience.
