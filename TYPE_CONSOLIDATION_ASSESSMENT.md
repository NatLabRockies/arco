# Type Fragmentation Assessment: Arco Rust Workspace

## Executive Summary

After analyzing type definitions across the arco workspace (11 crates, ~150+ type definitions), I've identified several patterns of type duplication and fragmentation. The highest-confidence consolidations are type aliases in arco-blocks and ObjectiveSense enums. SolverStatus/SolverError have deliberate parallel hierarchies with conversion traits that should be preserved.

## Type Inventory by Crate

### arco-core (Foundation Crate)

- **Core types**: `Sense`, `SimplifyLevel`, `Bounds`, `Variable`, `Constraint`, `Objective`
- **Solver types**: `SolverStatus`, `SolverError`, `Solution`, `Solver` (trait)
- **Model types**: `Model`, `ModelError`, `VariableView`, `ConstraintView`, etc.
- **Slack types**: `SlackBound`, `SlackVariables`, `SlackHandle`, `ElasticHandle`

### arco-solver (Solver Abstraction)

- **Parallel solver types**: `SolverStatus`, `SolverError` (with From/Into conversions to core)
- **Config**: `SolverConfig` (detailed solver parameters)
- **Backend trait**: `SolverBackend`, `Solve`, `SolutionView`

### arco-kdl (KDL Language)

- **AST types**: 20+ types (SourceProgram, ModelDecl, DataDecl, etc.)
- **Algebra types**: `Expr`, `ComparisonOp`, `BinaryOp`, `UnaryOp`, `ReductionOp`
- **Compiled types**: `CompiledProblem`, `CompiledVariable`, `CompiledConstraint`, etc.
- **Semantic types**: `SemanticProgram`, `ResolvedSets`, `ResolvedConstraint`, etc.
- **Enum**: `ObjectiveSense` (duplicates core::Sense)
- **Enum**: `ConstraintSense` (similar to arco_expr::ComparisonSense)

### arco-expr (Expression System)

- **Expr types**: `Expr`, `ConstraintExpr`
- **Enum**: `ComparisonSense` (similar to arco_kdl::ConstraintSense)
- **IDs**: `VariableId`, `ConstraintId`

### arco-highs, arco-xpress, arco-ipopt (Solver Backends)

- Each has: `Solution`, `Solver` struct, `SolverError` (type alias to core)
- arco-highs: `ObjectiveSense` (duplicates core::Sense)
- arco-highs: `HighsStatus`, `HighsModelError` (backend-specific)

### arco-blocks (Python Integration)

- **8 duplicate type aliases**: `type PyObject = Py<PyAny>` across 7 files
- Block types: `Block`, `BlockContext`, `BlockPort`, `BlockLink`, etc.

### arco-cli (CLI Application)

- CLI types: ~40 types for execution, inspection, reporting
- `SolverConfig` (simple backend selection) - different from arco-solver's
- `SolverBackend` enum (Highs/Xpress)

## Critical Duplications

### 1. PyObject Type Alias (HIGH CONFIDENCE)

**Files**: 7 files in arco-blocks define:

```rust
type PyObject = Py<PyAny>;
```

- `lib.rs`, `decorator.rs`, `spec.rs`, `util.rs`, `resolve.rs`, `transform.rs`, `schema.rs`

**Consolidation**: Move to shared location in arco-blocks.

### 2. ObjectiveSense / Sense (HIGH CONFIDENCE)

**Duplicated across**:

- `arco_core::types::Sense` (Minimize, Maximize)
- `arco_kdl::ObjectiveSense` (Minimize, Maximize) + serde
- `arco_highs::ffi::ObjectiveSense` (Minimize, Maximize)

**Consolidation**: Use core::Sense as canonical, add conversions/serde where needed.

### 3. SolverStatus / SolverError (INTENTIONAL - DO NOT CONSOLIDATE)

**Structure**:

- `arco_core::solver::{SolverStatus, SolverError}` - Core foundation
- `arco_solver::{SolverStatus, SolverError}` - Solver abstraction layer with bidirectional conversions

**Rationale**: This is a deliberate layered architecture. The solver crate provides a stable interface while core provides the foundation. The conversion traits allow clean interop.

### 4. ComparisonSense vs ConstraintSense (MEDIUM CONFIDENCE)

**Types**:

- `arco_expr::ComparisonSense`: LessEqual, GreaterEqual, Equal
- `arco_kdl::ConstraintSense`: GreaterEqual, LessEqual, Equal (order differs)

**Consolidation**: Could unify but different use cases (generic expressions vs compiled constraints).

### 5. SolverConfig (DIFFERENT PURPOSES - DO NOT CONSOLIDATE)

- `arco_solver::SolverConfig`: 8 solver parameters (time_limit, mip_gap, etc.)
- `arco_cli::SolverConfig`: Just backend selection (Highs/Xpress)

These serve different abstraction levels and should remain separate.

## Recommendations

### Immediate Actions (High Confidence)

1. **Consolidate PyObject alias** in arco-blocks
2. **Consolidate ObjectiveSense** using core::Sense as canonical

### Do Not Consolidate

1. **SolverStatus/SolverError** - intentional layered architecture
2. **SolverConfig** - different abstraction levels
3. **Backend-specific types** (HighsStatus, XpressGuard) - solver internals

### Future Considerations

1. Consider workspace-hack crate for truly shared types
2. Evaluate if arco-kdl's algebra types should use arco-expr types

## Implementation Notes

All changes must:

- Preserve existing public APIs (backward compatibility)
- Maintain serde compatibility where present
- Keep conversion traits for inter-crate interop
- Follow the "no breadcrumbs" rule (no "moved to" comments)
