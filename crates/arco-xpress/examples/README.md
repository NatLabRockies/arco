# arco-xpress Examples

This folder contains executable examples demonstrating how to use the `arco-xpress` crate.

## Available Examples

*To be added from integration tests:*
- `simple_lp` - Basic linear programming example
- `mip_optimization` - Mixed integer programming examples
- `solution_metadata` - Accessing solution information

## Running Examples

```bash
# List available examples
cargo run --example

# Run a specific example
cargo run --example <example_name>

# Run with release optimizations
cargo run --release --example <example_name>
```

## Prerequisites

The FICO Xpress solver must be installed and licensed on your system.

## Integration Test Migration

The integration tests in `../tests/integration.rs` can be converted to runnable examples:
- Tests demonstrate various Xpress solver capabilities
- LP and MIP problem solving
- Solution analysis and metadata access

To convert:
1. Copy test functions from `../tests/integration.rs`
2. Change `#[test]` to `fn main()` with proper error handling
3. Add to this folder as `<name>.rs`
