# Architecture

Arco separates interaction surfaces, text authoring formats, compilation, solver
contracts, and concrete solver adapters. The goal is that every user-facing path
shares the same semantic lowering and solver orchestration.

## Crate roles

```text
Interaction surfaces
  arco-cli       command-line executable
  arco-python    Python bindings
  arco-julia     planned Julia bindings

Text authoring formats
  arco-kdl       KDL text parser and source AST
  arco-json      planned JSON text/document loader
  arco-yaml      planned YAML text/document loader

Compilation
  arco-compile   semantic validation and lowering from parsed authoring ASTs
                 to solver-facing targets

Canonical model
  arco-model     in-memory model representation
  arco-expr      expression IDs and algebraic expressions
  arco-blocks    block composition support

Operations facade
  arco-ops       shared facade used by CLI and language bindings

Solver platform
  arco-solver    selection, profiles, and preflight
  arco-contracts shared solver config/status/result contracts
  arco-targets   solver-facing algebraic target structs

Solver adapters
  arco-highs
  arco-scip
  arco-ipopt
  arco-xpress

Exchange/export
  arco-export
  arco-exchange
```

## Dependency shape

```text
arco-cli ───────┐
arco-python ────┼──▶ arco-ops
arco-julia ─────┘       │
                        ├──▶ arco-kdl/json/yaml  parse text into source ASTs
                        ├──▶ arco-model          canonical in-memory model APIs
                        ├──▶ arco-compile        validate and lower parsed sources
                        ├──▶ arco-solver         selection and preflight
                        ├──▶ arco-targets        solver-facing algebra
                        ├──▶ solver adapters     execute solves
                        └──▶ export/exchange     write exchange formats
```

Text authoring crates parse files and source text. They do not own solver
selection, target lowering, runtime execution, or export behavior. `arco-compile`
is the semantic bridge that validates parsed authoring ASTs and lowers them into
solver-facing targets.

Language bindings and the CLI are interaction surfaces. They stay thin and route
shared behavior through `arco-ops` rather than assembling parser, compiler,
solver, and export crates directly.

Solver adapters consume solver targets and shared solver contracts. They must not
compile directly from canonical model internals.

`arco-core` has been retired; canonical model code lives in `arco-model`.
