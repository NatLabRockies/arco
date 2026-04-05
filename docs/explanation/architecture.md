Architecture

Arco is structured as a Rust workspace with multiple crates and Python bindings on top. This split is not arbitrary. It reflects a separation between the stable, performance-critical core and the user-facing API that prioritizes ergonomics over raw speed.

Crate overview

The workspace contains eleven crates organized into three layers. At the bottom are the foundational crates that define the data structures and algebraic building blocks. arco-core provides the basic abstractions: variables, constraints, expressions, and the solver-agnostic model representation. arco-algebra defines the arithmetic traits that allow composing expressions without dynamic dispatch. arco-index handles the index set machinery that powers multi-dimensional variable arrays.

The middle layer connects these abstractions to actual solvers. arco-solver defines the interface that any backend must implement. Currently there are two production backends: arco-highs, which wraps the embedded HiGHS solver, and arco-ipopt, which provides nonlinear programming capabilities. There is also arco-xpress for users with access to the FICO Xpress SDK, though this requires a commercial license and separate installation.

The top layer provides specialized functionality. arco-blocks defines the composition system for multi-stage optimization workflows. arco-reporters handles solution output and diagnostic formatting. arco-python contains the PyO3-based bindings that expose everything to Python.

Python binding layer

The Python API lives in bindings/python and is built with PyO3 and maturin. The binding layer is intentionally thin. It does not reimplement logic in Python; it wraps the Rust types directly and handles the translation between Python objects and Rust data structures.

This thinness has consequences. Error messages come from Rust and retain their precision. Type stubs are provided so static analysis tools can catch mistakes before runtime. The performance characteristics are essentially those of the underlying Rust code, minus some unavoidable overhead from crossing the language boundary.

The binding layer also handles NumPy integration. Variable arrays can be constructed from NumPy arrays, and solution values can be extracted back into NumPy format without copying data through Python lists. This matters for large models where the difference between a view and a copy is the difference between fitting in memory and not.

Solver abstraction

The solver interface in arco-solver is designed around a simple contract. A solver takes a model description in a normalized form, translates it to whatever representation the underlying solver requires, executes the solve, and returns results in a standard format. The normalization step is where much of the complexity lives.

When you build a model in Arco, you are constructing an expression graph. Variables reference constraints. Constraints reference expressions. Expressions reference other expressions. Before solving, this graph must be flattened into a sparse matrix format that solvers understand. This flattening involves topological sorting to resolve dependencies, constant folding where possible, and the actual construction of the CSR matrix structure.

The abstraction intentionally leaks slightly. Solver-specific options are exposed through configuration objects that accept raw key-value pairs. This allows users to tune solver behavior without Arco needing to maintain a mapping of every possible option for every backend. If you know HiGHS supports a particular setting, you can pass it through. If you pass something invalid, the solver will complain and Arco will surface that error.

Data flow

A typical workflow looks like this. The user constructs a Model in Python, adding variables and constraints through the bound API. This builds up the expression graph in Rust memory. When solve() is called, the model is frozen and normalized. The normalization produces a compact representation that the solver backend consumes. The backend translates this to its own format, calls the actual solver, and returns raw solution data. Arco wraps this in a SolveResult object that provides convenient accessors for variable values and constraint satisfaction status.

Throughout this process, allocations are minimized. The expression graph is built incrementally but stored compactly. Normalization reuses buffers where possible. Solution data is returned as views rather than copies when the caller requests it. The goal is that solving a model twice should not allocate twice the memory.

The workspace structure

The Cargo workspace is defined at the repository root. All crates share a common version number and lint configuration. This keeps the dependency graph consistent and ensures that a checkout at any particular commit builds all crates against compatible versions of each other.

The Python package is a separate concern. It is built with maturin, which handles the Rust compilation and produces a wheel that contains the compiled extension module. The resulting package embeds the HiGHS solver, so users who pip install arco get a working optimization tool without needing to install anything else.

Version management is automated through release-please, which bumps the version in all Cargo.toml files and pyproject.toml simultaneously. This prevents the drift that often happens when a workspace contains multiple crates that are supposed to be released together.
