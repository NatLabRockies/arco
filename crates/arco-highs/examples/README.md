# arco-highs Examples

This folder contains executable examples demonstrating how to use the `arco-highs` crate.

## Available Examples

- **`profile_crs_build`** - Performance profiling tool for CRS (Compressed Row Storage) matrix building
  - Supports baseline, dense, and async modes
  - Run with: `cargo run --example profile_crs_build -- --help`

## Running Examples

```bash
# List available examples
cargo run --example

# Run a specific example
cargo run --example profile_crs_build

# Run with release optimizations
cargo run --release --example profile_crs_build
```

## Integration Test Migration

The integration tests in `../tests/` may be converted to runnable examples:
- `integration.rs` - Various LP/MILP solver tests
- `ffi_smoke.rs` - FFI binding smoke tests

To convert a test to an example:
1. Copy the test file to this folder
2. Change `#[test]` functions to `fn main()` or standalone functions
3. Add example to [[example]] section in Cargo.toml if needed
