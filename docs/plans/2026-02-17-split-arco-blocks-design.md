# Split arco-blocks lib.rs Design

## Goal

Split `crates/arco-blocks/src/lib.rs` (2072 lines) into focused flat modules under
`crates/arco-blocks/src/` while preserving behavior and public API.

## Constraints

- Keep a flat file layout (no nested folders).
- Preserve all Python-facing exports registered by `add_blocks_submodule`.
- Preserve external Rust usage from `bindings/python`, which imports only:
  - `BlockPort`
  - `add_blocks_submodule`
- Keep existing behavior unchanged.

## Module Layout

Final layout:

```text
crates/arco-blocks/src/
  dag.rs
  error.rs
  once_map.rs
  lib.rs
  transform.rs
  schema.rs
  util.rs
  spec.rs
  decorator.rs
  resolve.rs
```

### `transform.rs`

Move transform primitives and helpers:

- `TransformStep`
- `Transform`
- `clone_steps`
- `apply_step`, `apply_binary`, `apply_shift`, `apply_clip`, `apply_select`
- `is_sequence`

### `schema.rs`

Move schema validation/coercion helpers:

- `is_pydantic_schema`, `is_dataclass_schema`, `dataclass_fields`
- `compare_fields`, `validate_data`, `outputs_schema_dict`
- `coerce_inputs`, `coerce_outputs`, `coerce_schema`

### `util.rs`

Move shared helpers:

- `log_block_error`
- `rss_bytes`, `log_block_phase`
- `model_type`, `create_model`

### `spec.rs`

Move spec-related types and helpers:

- `BlockSpec`
- `SpecBuilder`, `SpecExtractor`
- `validate_spec`, `get_spec_attr`

### `decorator.rs`

Move function decorator machinery:

- `ARCO_BLOCK_*` marker constants
- `FunctionBlockDecorator`
- `block`, `decorate_block_function`, `typed_block_meta_from_function`

### `resolve.rs`

Move link resolution/output extraction and standalone pyfunctions:

- `resolve_links`, `extract_outputs`
- `block_spec`, `build_model_from_spec`, `inspect_model`
- `schemas_compatible`, `specs_are_swappable`

## What Stays in `lib.rs`

Keep tightly coupled core orchestration types and registration:

- `DropPolicy`
- `BlockContext`
- `BlockPort`, `BlockLink`, `BlockDiagnostics`, `BlockRun`
- `BuildResult`
- `Block`
- `BlockModel`
- `add_blocks_submodule`
- tests

## Dependency Direction

- `lib.rs` depends on all extracted modules.
- Extracted modules must avoid cycles and expose only `pub(crate)` helpers unless
  Python class/function registration requires wider visibility.

## Verification

After each extraction chunk:

1. `cargo check -p arco-blocks`
2. `cargo test -p arco-blocks`
3. `cargo clippy -p arco-blocks --all-features -- -D warnings`

Final validation:

1. `cargo fmt --all`
2. `cargo clippy --all --benches --tests --examples --all-features -- -D warnings`
3. `cargo test --workspace --all-features`
