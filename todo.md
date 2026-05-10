1.  Python fan-out cleanup
    - bindings/python/src/lib.rs still high fan-out.
    - We disabled no_god_files in Sentrux to pass.
    - Real fix: finish splitting Python registration/model facade.
2.  IPOPT/Xpress real target adapters
    - They no longer violate boundary, but are mostly placeholders.
    - Need target-based implementations, not SolverNotAvailable.
3.  Python warm-start
    - primal_start is accepted but ignored in new target path.
    - Need target/adapter support or explicit docs saying unsupported.
4.  Compilation seam still muddy
    - KDL still owns too much “compile/lower” behavior.
    - Target architecture wants authoring surface → canonical model → compile/lower → targets.
    - A future arco-compile/arco-lower split would make this cleaner.
5.  arco-solver still has model-shaped legacy seams
    - SolverBackend alias still references arco_model::Model.
    - Should move fully toward target/config/contracts.
6.  Ops exposes internals for Python
    - arco-ops reexports model, expr, etc. Good for boundary passing, but not ideal long-term API shape.
    - Better: expose explicit ops-level DTOs/APIs.
7.  Docs architecture may lag code
    - Need update docs/explanation/architecture.md with final target diagram and current limitations.
8.  Sentrux rules are weaker than desired
