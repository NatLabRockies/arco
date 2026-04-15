# Legacy Code Assessment and Removal for Arco

## Date: 2026-04-15

## Summary

Successfully identified and removed legacy/fallback code from the arco Rust codebase. The main legacy pattern was backward compatibility support for renamed KDL properties.

## Legacy Code Found and Removed

### 1. Legacy KDL Property: `from=` in `data` declarations

**Location**: `crates/arco-kdl/src/source/parser.rs` (lines 179-180, 407-408)

**What was removed**:

- The parser no longer accepts both `source=` (new) and `from=` (legacy) for data file paths
- Changed `property_string_alternatives(node, &["source", "from"], ...)` to `property_string(node, "source", ...)` in two places
- Removed the `property_string_alternatives()` helper function from `parser_helpers.rs`

**Impact**: Cleaner, singular code path - users must now use the `source=` property name

**Files modified**:

- `crates/arco-kdl/src/source/parser.rs` - Updated imports and two call sites
- `crates/arco-kdl/src/source/parser_helpers.rs` - Removed unused helper function
- `crates/arco-kdl/tests/source_parser.rs` - Updated tests to use `source=` instead of `from=`
- `crates/arco-kdl/tests/compile_suite.rs` - Updated tests to use `source=` instead of `from=`
- `crates/arco-kdl/tests/semantic_validation.rs` - Updated tests to use `source=` instead of `from=`
- `examples/dcopf-angle/input.kdl` - Updated 5 `data` declarations
- `examples/dcopf-ptdf/input.kdl` - Updated 5 `data` declarations
- `examples/ded-ess-wind-linearized/input.kdl` - Updated 3 `data` declarations

### 2. Legacy Test for `lowering_rules` JSON field

**Location**: `crates/arco-cli/src/benchmark.rs` (lines 327-345)

**What was removed**:

- Test `semantic_expectation_accepts_legacy_lowering_rules_json_field` that validated acceptance of an obsolete `lowering_rules` field

**Files modified**:

- `crates/arco-cli/src/benchmark.rs` - Removed legacy test

## Legacy Code Kept (Intentionally)

### Error handling for `index_by` property

**Location**: `crates/arco-kdl/src/source/parser.rs` (lines 192-198)

**Why kept**: This is an error message, not silent fallback code. It actively rejects the obsolete `index_by` property and directs users to use `index` instead. This provides clear migration guidance rather than silently accepting outdated syntax.

## Feature Flags Status (All Active)

- `async` and `parallel` in `arco-highs`: ACTIVELY USED
- `ipopt` and `xpress` in `arco-python` and `arco-cli`: ACTIVELY USED (optional solver backends)
- `serde` in `arco-solver-types`: ACTIVELY USED

## No Deprecated Attributes Found

- No `#[deprecated]` attributes in the codebase
- No `#[allow(deprecated)]` overrides
- Workspace `Cargo.toml` explicitly denies `todo!()` and `unimplemented!()` macros at compile time

## Test Results

All tests pass after the removals:

```
cargo test -p arco-kdl      # 23 tests passed
cargo test -p arco-cli       # 19 tests passed
```

## Commands to Verify

```bash
cargo fmt --all
cargo check -p arco-kdl -p arco-cli -p arco-core -p arco-highs -p arco-solver -p arco-solver-types -p arco-blocks -p arco-tools -p arco-expr
cargo test -p arco-kdl -p arco-cli
```
