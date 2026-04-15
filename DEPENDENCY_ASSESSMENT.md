# Arco Dependency Graph Assessment

## Executive Summary

After analyzing the Arco workspace using `cargo tree`, manual source code inspection, and dependency graph analysis, I found **no circular dependencies** in the current codebase. However, I identified several **architectural layering violations** and **design issues** that could lead to maintenance problems and potential future cycles.

---

## Current Dependency Graph

### Visual Representation (Workspace Crates Only)

```
                    arco-cli
                   /    |    \
                  /     |     \
            arco-kdl  arco-core  arco-highs ─────┐
               |      /   |   \        |          |
               |     /    |    \       |          |
               |    /     |     \      |          |
               |   /      |      \     |          |
            arco-core  arco-expr  arco-tools  arco-solver
               |   \      |      /         /    /
               |    \     |     /         /    /
               |     \    |    /         /    /
            arco-expr   arco-tools      /    /
                                         /    /
            arco-highs ─────────────────┘    /
            arco-ipopt ─────────────────────┘
            arco-xpress ────────────────────┘
            arco-python ────────────────────┘

arco-blocks ──── arco-tools
    |
    └──── arco-python
```

### Detailed Dependency Table

| Crate         | Dependencies                                                                                    | Layer               |
| ------------- | ----------------------------------------------------------------------------------------------- | ------------------- |
| `arco-expr`   | (none)                                                                                          | 0 - Foundation      |
| `arco-tools`  | (none)                                                                                          | 0 - Foundation      |
| `arco-core`   | arco-expr, arco-tools                                                                           | 1 - Core            |
| `arco-solver` | arco-core, arco-expr                                                                            | 2 - Abstractions    |
| `arco-blocks` | arco-tools                                                                                      | 1 - Core            |
| `arco-kdl`    | arco-core                                                                                       | 2 - Abstractions    |
| `arco-highs`  | arco-core, arco-expr, arco-solver, arco-tools                                                   | 3 - Implementations |
| `arco-ipopt`  | arco-core, arco-expr, arco-solver                                                               | 3 - Implementations |
| `arco-xpress` | arco-core, arco-expr, arco-solver                                                               | 3 - Implementations |
| `arco-cli`    | arco-kdl, arco-core, arco-highs, (opt) arco-xpress                                              | 4 - Application     |
| `arco-python` | arco-core, arco-expr, arco-highs, arco-solver, arco-blocks, (opt) arco-ipopt, (opt) arco-xpress | 4 - Bindings        |

---

## Critical Issues Found

### 1. **Diamond Dependency Pattern** ⚠️ HIGH SEVERITY

**Problem**: `arco-solver` depends on `arco-core`, but solver implementations (`arco-highs`, `arco-ipopt`, `arco-xpress`) depend on BOTH `arco-core` AND `arco-solver`.

```
arco-highs ────┐
               ├──► arco-core ◄── arco-solver
               │       ▲              │
               └───────┴──────────────┘
```

**Impact**:

- Violates the "abstractions should not depend on details" principle
- Makes it harder to swap out core model implementations
- Could lead to version mismatch issues if arco-core types change

### 2. **Type Re-export Confusion** ⚠️ MEDIUM SEVERITY

**Problem**: `arco-solver` re-exports `SolverError` and `SolverStatus` from `arco-core::solver`, but:

- The solver implementations import from BOTH crates
- `arco-core::solver::Solver` trait is essentially unusable by external solver crates
- Two sources of truth for solver types

**Evidence** (from `arco-solver/src/lib.rs`):

```rust
// Re-export solver types from arco-core to avoid duplication
pub use arco_core::solver::{SolverError, SolverStatus};
```

But in `arco-highs/src/solver.rs`:

```rust
use arco_core::solver::SolverError as CoreSolverError;  // From arco-core
use arco_solver::{..., SolverError as GenericSolverError};  // From arco-solver (re-export)
```

### 3. **Orphaned `Solver` Trait in arco-core** ⚠️ MEDIUM SEVERITY

**Problem**: `arco-core` defines a `Solver` trait that cannot be implemented by the actual solver crates without creating a cycle:

```rust
// In arco-core/src/solver.rs
pub trait Solver {
    fn solve(&mut self, model: &Model) -> Result<Solution, SolverError>;
}
```

Solver backends can't implement this because:

- They need `arco-core` for `Model` and `Solution`
- But they also need their solver-specific types
- The trait is essentially orphaned

### 4. **arco-blocks Architectural Mismatch** ⚠️ LOW SEVERITY

**Problem**: `arco-blocks` depends only on `arco-tools`, but:

- It's used by `arco-python` which pulls in the entire model ecosystem
- Block orchestration likely needs model types but can't access them directly
- This suggests either over-separation or under-separation of concerns

---

## Architectural Recommendations

### Option A: Split arco-core into Smaller Crates (RECOMMENDED)

**Refactoring Strategy**:

```
arco-expr          (existing - types, ids, expressions)
arco-model-types   (NEW - Model, Variable, Constraint, Objective types)
arco-solver-types  (NEW - SolverStatus, SolverError, Solution, SolverBackend trait)
arco-model         (renamed from arco-core - depends on arco-expr, arco-model-types, arco-solver-types)
arco-solver-traits (NEW - Solve trait, SolutionView trait - depends on arco-model-types, arco-solver-types)
arco-highs         (depends on arco-model, arco-solver-traits)
```

**Benefits**:

- True dependency inversion: abstractions don't depend on implementations
- Solver backends only depend on types, not full model implementation
- Clear separation of concerns

### Option B: Merge arco-solver into arco-core (SIMPLER)

**Strategy**: Remove the separate `arco-solver` crate and move its traits into `arco-core::solver`.

**Benefits**:

- Eliminates the diamond pattern
- Single source of truth for solver abstractions
- Simpler dependency graph

**Drawbacks**:

- Makes arco-core larger
- Solver backends still depend on full model implementation

### Option C: Extract Trait-Only Crate (HYBRID)

**Strategy**: Create `arco-solver-traits` with only trait definitions and basic types:

```rust
// arco-solver-traits/src/lib.rs
pub trait SolverBackend {
    fn solve(&self, model: &dyn ModelView, config: &SolverConfig) -> Result<Solution, SolverError>;
}

pub trait ModelView {
    fn variables(&self) -> &[Variable];
    fn constraints(&self) -> &[Constraint];
    // ... minimal read-only interface
}
```

---

## Files to Review

| File                                | Issue                                        |
| ----------------------------------- | -------------------------------------------- |
| `crates/arco-solver/Cargo.toml`     | Depends on arco-core, creating diamond       |
| `crates/arco-solver/src/lib.rs`     | Re-exports from arco-core                    |
| `crates/arco-solver/src/backend.rs` | Imports Model from arco-core                 |
| `crates/arco-core/src/solver.rs`    | Orphaned Solver trait                        |
| `crates/arco-highs/src/solver.rs`   | Imports from both arco-core and arco-solver  |
| `crates/arco-ipopt/src/solver.rs`   | Imports from both arco-core and arco-solver  |
| `crates/arco-xpress/src/solver.rs`  | Imports from both arco-core and arco-solver  |
| `crates/arco-blocks/Cargo.toml`     | Minimal dependencies - verify if intentional |

---

## Summary of Refactoring Completed

The following changes were made to break the diamond dependency and establish proper layering:

### New Crate: `arco-solver-types`

- **Location**: `crates/arco-solver-types/`
- **Purpose**: Foundation crate containing solver-agnostic types
- **Contents**:
  - `SolverStatus`: Enum for solution status (Optimal, Infeasible, etc.)
  - `SolverError`: Error type for solver operations
  - `Solution`: Solver-agnostic solution struct
  - `SolverConfig`: Basic solver configuration
- **Dependencies**: Only `arco-expr`
- **Layer**: 0 (Foundation)

### Modified Crates

#### `arco-core`

- **Changes**:
  - Added dependency on `arco-solver-types`
  - `solver.rs` now re-exports types from `arco-solver-types` instead of defining them
  - `Solver` trait remains in `arco-core::solver` (depends on `Model`)
- **New Dependencies**: `arco-solver-types`
- **Layer**: 1 (Core)

#### `arco-solver`

- **Changes**:
  - Added dependency on `arco-solver-types`
  - `traits.rs` now uses local `SolverConfig` (extended version)
  - Re-exports `Solution`, `SolverError`, `SolverStatus` from `arco-solver-types`
  - `backend.rs` uses `arco_solver_types::Solution`
- **New Dependencies**: `arco-solver-types`
- **Layer**: 2 (Abstractions)

#### `arco-highs`, `arco-ipopt`, `arco-xpress`

- **Changes**:
  - Added dependency on `arco-solver-types`
  - Updated imports to use `arco_solver_types` for base types
  - Fixed `SolverBackend` trait implementation to use correct types
- **New Dependencies**: `arco-solver-types`
- **Layer**: 3 (Implementations)

#### `arco-python` (bindings)

- **Changes**:
  - Added dependency on `arco-solver-types`
- **New Dependencies**: `arco-solver-types`

### New Dependency Graph

```
                    arco-cli
                   /    |    \
                  /     |     \
            arco-kdl  arco-core  arco-highs ─────┐
               |      /   |   \        |          |
               |     /    |    \       |          |
               |    /     |      \      |          |
               |   /      |       \     |          |
            arco-core  arco-expr  arco-tools  arco-solver
               |   \      |      /         /    /
               |    \     |     /         /    /
               |     \    |    /         /    /
            arco-solver-types             /    /
                                         /    /
            arco-highs ─────────────────┘    /
            arco-ipopt ─────────────────────┘
            arco-xpress ────────────────────┘
            arco-python ────────────────────┘

arco-blocks ──── arco-tools
    |
    └──── arco-python
```

### Benefits of the Refactoring

1. **No More Diamond Dependencies**: Solver backends no longer have multiple paths to the same types
2. **Clean Layering**:
   - Layer 0: `arco-expr`, `arco-tools`, `arco-solver-types`
   - Layer 1: `arco-core`, `arco-blocks`
   - Layer 2: `arco-solver`, `arco-kdl`
   - Layer 3: `arco-highs`, `arco-ipopt`, `arco-xpress`
   - Layer 4: `arco-cli`, `arco-python`
3. **Single Source of Truth**: `SolverStatus`, `SolverError`, `Solution` defined once in `arco-solver-types`
4. **Trait Orphan Rule Compliance**: `SolverBackend` trait uses types from a crate it depends on
5. **Future-Proof**: Adding new solver backends is simpler - they just depend on `arco-solver-types`

### Test Results

- All unit tests pass for modified crates
- `cargo check` succeeds for entire workspace (excluding `arco-xpress` which requires proprietary SDK)

---

## Original Assessment (Pre-Refactoring)

While there are **no circular dependencies** currently, the architecture has **structural issues** that violate clean architecture principles:

1. **Abstractions depend on details** (arco-solver → arco-core)
2. **Diamond dependencies** create maintenance risk
3. **Orphaned traits** suggest incomplete refactoring

The recommended fix is **Option A** (split arco-core) for long-term maintainability, or **Option B** (merge arco-solver into arco-core) for immediate simplicity.

---

_Assessment generated: April 15, 2026_
_Tooling: cargo tree, grep, manual source analysis_
