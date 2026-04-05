# Arco Test Structure Analysis and Conversion Plan

## Executive Summary

This document analyzes the test structure in `crates/arco-highs/tests/` and `crates/arco-core/src/model/tests/` to identify which tests should be converted from integration tests to pure Rust unit tests. The goal is to improve test organization by moving tests that exercise internal functionality closer to the code they test.

---

## Current Test Structure Overview

### 1. arco-highs Integration Tests (`crates/arco-highs/tests/`)

| File | Lines | Test Count | Description |
|------|-------|------------|-------------|
| `ffi_smoke.rs` | 83 | 3 tests | Low-level HiGHS FFI wrapper tests |
| `integration.rs` | 379 | 10 tests | High-level solver integration tests |

### 2. arco-core Model Tests (`crates/arco-core/src/model/tests/`)

| File | Lines | Test Count | Description |
|------|-------|------------|-------------|
| `support.rs` | 15 | N/A | Test helper functions |
| `metadata_inspect.rs` | 188 | 6 tests | Metadata and inspection functionality |
| `slack_csc.rs` | 260 | 10 tests | Slack variables and CSC import |
| `sparse_export.rs` | 91 | 4 tests | Sparse matrix export formats |
| `mod.rs` (inline) | ~300 | 25+ tests | Core model operations |

---

## Detailed Analysis: arco-highs/tests/ffi_smoke.rs

### Current Tests

1. **`test_minimize_simple`** (lines 4-50)
   - Tests basic minimization via `HighsModel` FFI wrapper
   - Creates model, adds variable with bounds, sets objective sense
   - Tests primal start setting
   - Verifies solution status and objective value
   - **Classification**: TRUE INTEGRATION TEST - requires HiGHS library
   - **Recommendation**: KEEP as integration test (tests actual FFI solver)

2. **`test_integer_variable_is_enforced`** (lines 53-75)
   - Tests integer variable enforcement via `HighsModel`
   - Adds integer column, constraint, maximizes
   - Verifies integer solution is respected
   - **Classification**: TRUE INTEGRATION TEST - requires HiGHS MIP solver
   - **Recommendation**: KEEP as integration test (tests solver's MIP capability)

3. **`test_primal_start_length_mismatch`** (lines 78-83)
   - Tests validation error for wrong primal start length
   - Only tests error handling, doesn't solve
   - **Classification**: UNIT-TESTABLE - tests `HighsModel` validation logic
   - **Recommendation**: CONVERT to unit test in `src/ffi.rs` using `#[cfg(test)]` module

---

## Detailed Analysis: arco-highs/tests/integration.rs

### Current Tests

1. **`test_simple_lp`** (lines 9-60)
   - Builds model using `arco_core::Model` API
   - Creates solver via `arco_highs::Solver`, solves, verifies solution
   - **Classification**: TRUE INTEGRATION TEST - end-to-end solve
   - **Recommendation**: KEEP as integration test OR convert to example

2. **`test_integer_variable_solution`** (lines 63-105)
   - Tests MIP solving via `arco_highs::Solver`
   - **Classification**: TRUE INTEGRATION TEST
   - **Recommendation**: KEEP as integration test

3. **`test_primal_start_storage`** (lines 145-153)
   - Tests `Solver::set_primal_start()` and `get_primal_start()`
   - Tests internal state management of solver, doesn't actually solve
   - **Classification**: UNIT-TESTABLE - tests solver configuration state
   - **Recommendation**: CONVERT to unit test in `src/solver.rs`

4. **`test_primal_start_validation`** (lines 156-162)
   - Tests validation of invalid variable IDs for primal start
   - Tests error handling logic
   - **Classification**: UNIT-TESTABLE - tests validation logic
   - **Recommendation**: CONVERT to unit test in `src/solver.rs`

5. **`test_primal_start_clear`** (lines 165-174)
   - Tests `clear_primal_start()` method
   - Tests state management
   - **Classification**: UNIT-TESTABLE - tests internal state
   - **Recommendation**: CONVERT to unit test in `src/solver.rs`

6. **`test_primal_start_solve`** (lines 177-190)
   - Tests that primal start is used during solve
   - **Classification**: TRUE INTEGRATION TEST - requires actual solve
   - **Recommendation**: KEEP as integration test

7. **`test_dual_values_exposed`** (lines 193-215)
   - Tests dual value accessors on solution
   - **Classification**: TRUE INTEGRATION TEST - requires solve
   - **Recommendation**: KEEP as integration test

8. **`test_solution_metadata_accessors`** (lines 218-267)
   - Tests timing, iteration counts, tolerance accessors
   - **Classification**: MIXED - some could be unit tested, but need solve for real data
   - **Recommendation**: KEEP as integration test (tests real solver metadata)

9. **`test_solution_status_methods`** (lines 270-312)
   - Tests solution status methods (is_optimal, is_feasible, etc.)
   - **Classification**: TRUE INTEGRATION TEST - requires solve
   - **Recommendation**: KEEP as integration test

10. **`test_solution_accessor_edge_cases`** (lines 315-379)
    - Tests infeasible and unbounded problem handling
    - **Classification**: TRUE INTEGRATION TEST - requires solve
    - **Recommendation**: KEEP as integration test

---

## Detailed Analysis: arco-core/src/model/tests/

### Current Tests (Already Unit Tests)

The tests in `arco-core/src/model/tests/` are **already correctly organized** as unit tests within the `#[cfg(test)]` module in `mod.rs`. These are all testing internal model functionality and do NOT require external solvers.

#### metadata_inspect.rs (6 tests)
- `test_variable_name_lifecycle` - variable naming
- `test_variable_metadata` - JSON metadata storage
- `test_constraint_name_lifecycle` - constraint naming
- `test_name_lookup_helpers` - name→ID resolution
- `test_constraint_metadata` - constraint JSON metadata
- `inspect_includes_coefficients_and_slacks` - model inspection
- `inspect_respects_filters` - filtering in inspection

**Status**: ✅ Already proper unit tests - NO ACTION NEEDED

#### slack_csc.rs (10 tests)
- `test_add_slack_upper_adds_variable_and_objective`
- `test_add_slack_both_sets_names_and_coefficients`
- `test_add_slack_penalty_flips_on_maximize`
- `test_add_slack_requires_objective`
- `test_make_elastic_respects_optional_bounds`
- `test_from_csc_builds_model`
- `test_from_csc_rejects_bad_col_ptrs`
- `test_from_csc_only_stores_non_empty_columns`
- `test_from_csc_rejects_non_finite_values`

**Status**: ✅ Already proper unit tests - NO ACTION NEEDED

#### sparse_export.rs (4 tests)
- `sparse_export_csc_has_expected_shape_and_arrays`
- `sparse_export_crs_groups_values_by_row_and_keeps_monotonic_ptrs`
- `sparse_export_coo_emits_triplets_in_column_scan_order`
- `sparse_export_empty_model_returns_empty_payloads`

**Status**: ✅ Already proper unit tests - NO ACTION NEEDED

---

## Conversion Recommendations Summary

### arco-highs: Tests to Convert to Unit Tests

| Test | Source File | Destination | Rationale |
|------|-------------|-------------|-----------|
| `test_primal_start_length_mismatch` | `tests/ffi_smoke.rs` | `src/ffi.rs` (new `#[cfg(test)]` module) | Tests validation logic only |
| `test_primal_start_storage` | `tests/integration.rs` | `src/solver.rs` (add to existing tests) | Tests internal state management |
| `test_primal_start_validation` | `tests/integration.rs` | `src/solver.rs` (add to existing tests) | Tests validation logic |
| `test_primal_start_clear` | `tests/integration.rs` | `src/solver.rs` (add to existing tests) | Tests internal state management |

### arco-highs: Tests to Keep as Integration Tests

| Test | Rationale |
|------|-----------|
| `test_minimize_simple` | Requires actual HiGHS solve |
| `test_integer_variable_is_enforced` | Requires MIP solver |
| `test_simple_lp` | End-to-end solve test |
| `test_integer_variable_solution` | MIP solve test |
| `test_primal_start_solve` | Tests warm-start in actual solve |
| `test_dual_values_exposed` | Requires solve for dual values |
| `test_solution_metadata_accessors` | Tests real solver metadata |
| `test_solution_status_methods` | Requires solve for status |
| `test_solution_accessor_edge_cases` | Tests solver behavior |

---

## Proposed Unit Test Additions

### 1. Add to `crates/arco-highs/src/ffi.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_start_length_mismatch() {
        let mut model = HighsModel::new();
        model.add_col(0.0, 1.0, 1.0);
        assert!(model.set_primal_start(vec![0.0, 1.0]).is_err());
    }

    // Additional FFI-level unit tests:
    #[test]
    fn test_new_model_has_zero_columns() {
        let model = HighsModel::new();
        assert_eq!(model.columns(), 0);
    }

    #[test]
    fn test_add_col_increases_column_count() {
        let mut model = HighsModel::new();
        model.add_col(0.0, 1.0, 1.0);
        assert_eq!(model.columns(), 1);
    }

    #[test]
    fn test_add_integer_col_marks_as_integer() {
        let mut model = HighsModel::new();
        let idx = model.add_integer_col(0.0, 10.0, 1.0);
        // Verify via column properties if exposed
    }
}
```

### 2. Add to `crates/arco-highs/src/solver.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use arco_core::{Bounds, Constraint, Model, Objective, Sense, Variable};

    fn build_simple_model() -> Model {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        let y = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        let constraint = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 5.0),
            })
            .unwrap();
        model.set_coefficient(x, constraint, 1.0).unwrap();
        model.set_coefficient(y, constraint, 1.0).unwrap();
        let objective = Objective {
            sense: Some(Sense::Minimize),
            terms: vec![(x, 1.0), (y, 1.0)],
        };
        model.set_objective(objective).unwrap();
        model
    }

    #[test]
    fn test_primal_start_storage() {
        let model = build_simple_model();
        let mut solver = Solver::new(model).unwrap();
        let hints = vec![(VariableId::new(0), 0.5), (VariableId::new(1), 1.0)];
        assert!(solver.set_primal_start(&hints).is_ok());
        assert_eq!(solver.get_primal_start(), Some(hints.as_slice()));
    }

    #[test]
    fn test_primal_start_validation() {
        let model = build_simple_model();
        let mut solver = Solver::new(model).unwrap();
        let invalid_hints = vec![(VariableId::new(9999), 0.5)];
        assert!(solver.set_primal_start(&invalid_hints).is_err());
    }

    #[test]
    fn test_primal_start_clear() {
        let model = build_simple_model();
        let mut solver = Solver::new(model).unwrap();
        let hints = vec![(VariableId::new(0), 0.5)];
        solver.set_primal_start(&hints).unwrap();
        assert!(solver.get_primal_start().is_some());
        solver.clear_primal_start();
        assert!(solver.get_primal_start().is_none());
    }

    #[test]
    fn test_solver_config_defaults() {
        let model = build_simple_model();
        let solver = Solver::new(model).unwrap();
        assert!(!solver.config().log_to_console);
    }

    #[test]
    fn test_solver_config_setters() {
        let model = build_simple_model();
        let mut solver = Solver::new(model).unwrap();
        solver.set_log_to_console(true);
        solver.set_time_limit(60.0);
        solver.set_mip_gap(0.01);
        assert!(solver.config().log_to_console);
        assert_eq!(solver.config().time_limit, Some(60.0));
        assert_eq!(solver.config().mip_gap, Some(0.01));
    }
}
```

---

## Implementation Plan

### Phase 1: Add Unit Tests to arco-highs/src/ffi.rs
1. Create `#[cfg(test)]` module at end of `src/ffi.rs`
2. Move `test_primal_start_length_mismatch` from `tests/ffi_smoke.rs`
3. Add additional FFI-level unit tests for basic operations

### Phase 2: Add Unit Tests to arco-highs/src/solver.rs
1. Add `#[cfg(test)]` module with test helper `build_simple_model()`
2. Move `test_primal_start_storage`, `test_primal_start_validation`, `test_primal_start_clear` from `tests/integration.rs`
3. Add additional unit tests for solver configuration

### Phase 3: Clean Up Integration Tests
1. Remove the moved tests from integration test files
2. Ensure remaining integration tests are truly integration-level
3. Consider converting some end-to-end tests to examples in `examples/` directory

### Phase 4: Verify arco-core Tests
1. Confirm all arco-core tests are properly organized (they are ✅)
2. No changes needed for arco-core

---

## Expected Benefits

1. **Faster Test Runs**: Unit tests run without linking/running HiGHS
2. **Better Organization**: Tests for internal logic are co-located with code
3. **Clearer Test Intent**: Integration tests focus on actual solver behavior
4. **Easier Debugging**: Unit test failures point directly to logic issues
5. **CI Efficiency**: Unit tests can run in parallel without solver dependencies

---

## Files to Modify

| File | Action |
|------|--------|
| `crates/arco-highs/src/ffi.rs` | Add `#[cfg(test)]` module with unit tests |
| `crates/arco-highs/src/solver.rs` | Add `#[cfg(test)]` module with unit tests |
| `crates/arco-highs/tests/ffi_smoke.rs` | Remove `test_primal_start_length_mismatch` after move |
| `crates/arco-highs/tests/integration.rs` | Remove 3 primal_start tests after move |

---

## Files Not to Modify

| File | Reason |
|------|--------|
| `crates/arco-core/src/model/tests/*.rs` | Already proper unit tests |
| `crates/arco-core/src/model/mod.rs` | Test structure is correct |

---

## Note on arco-core Test Organization

The arco-core crate already follows best practices:
- Tests are in `#[cfg(test)]` modules
- Submodules are organized by functionality (metadata_inspect, slack_csc, sparse_export)
- Common test utilities are in `support.rs`
- Tests don't require external solvers

No changes needed for arco-core tests.
