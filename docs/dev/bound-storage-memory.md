# Bound storage memory

`arco-model::Model` keeps variable and constraint bounds in fixed-size blocks.
Each block starts as one uniform bound value. If a later value in that block
differs, only that block becomes a dense vector. The block size is 256 entries.

This representation keeps the public model contract unchanged. Variable and
constraint IDs are still positional, `Model::get_variable` still returns a
value, and `Model::get_constraint` still returns a reference. A uniform block
returns a reference to its one stored value for every ID in that block. Bounds
are compared by the bit patterns of both `f64` values, so signed zeroes remain
observable through inspection and fingerprints.

`Model::with_capacities`, `reserve_variables`, and `reserve_constraints` reserve
only block headers. Dense block payloads are allocated when a block actually
contains differing bounds; capacity reservation does not allocate a dense
buffer for unused entries.

The storage saves the bound payload when adjacent entries repeat. It does not
assume that every model has repeated bounds: heterogeneous blocks retain dense
payloads and add only small block metadata. The affected storage should be
measured with the target formulation before attributing a process RSS saving.
