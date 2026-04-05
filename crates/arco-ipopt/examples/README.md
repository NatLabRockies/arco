# arco-ipopt Examples

This folder contains executable examples demonstrating how to use the `arco-ipopt` crate.

## Available Examples

*To be added from integration tests:*
- `simple_lp` - Basic linear programming example
- `maximize_lp` - Maximization problem example
- Various nonlinear programming examples

## Running Examples

```bash
# List available examples
cargo run --example

# Run a specific example
cargo run --example <example_name>

# Run with release optimizations
cargo run --release --example <example_name>
```

## Integration Test Migration

The integration tests in `../tests/integration.rs` can be converted to runnable examples:
- Tests demonstrate various IPOPT solver capabilities
- Nonlinear optimization problems
- Constraint handling

To convert:
1. Copy test functions from `../tests/integration.rs`
2. Change `#[test]` to `fn main()` with proper error handling
3. Add to this folder as `<name>.rs`
