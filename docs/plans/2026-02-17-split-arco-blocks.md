# Split `arco-blocks/src/lib.rs` Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `crates/arco-blocks/src/lib.rs` into focused modules without changing behavior or public API.

**Architecture:** Extract independent domains first (`transform`, `schema`, `util`) and then higher-coupling domains (`spec`, `decorator`, `resolve`), keeping core block orchestration in `lib.rs`.

**Tech Stack:** Rust, PyO3, tracing, cargo

---

### Task 1: Extract Transform Module

**Files:**

- Create: `crates/arco-blocks/src/transform.rs`
- Modify: `crates/arco-blocks/src/lib.rs`

**Step 1: Write the failing test**

Add/extend one unit test in `crates/arco-blocks/src/lib.rs` that still references `Transform::identity()` so compile fails until imports/module wiring is correct.

**Step 2: Run test to verify it fails**

Run: `cargo test -p arco-blocks test_drop_policy_enum_values`

Expected: fail to compile due to unresolved moved symbol while extraction is in-progress.

**Step 3: Write minimal implementation**

Create `transform.rs` with:

- `TransformStep`
- `Transform`
- `clone_steps`
- `apply_step`, `apply_binary`, `apply_shift`, `apply_clip`, `apply_select`
- `is_sequence`

Wire in `lib.rs` with `mod transform;` and imports.

**Step 4: Run test to verify it passes**

Run: `cargo test -p arco-blocks test_drop_policy_enum_values`

Expected: PASS.

**Step 5: Commit**

`git commit -m "refactor(arco-blocks): extract transform module"`

### Task 2: Extract Schema Module

**Files:**

- Create: `crates/arco-blocks/src/schema.rs`
- Modify: `crates/arco-blocks/src/lib.rs`

**Step 1: Write the failing test**

Add/adjust a unit test touching `schemas_compatible` call path so move breaks compilation until module wiring is completed.

**Step 2: Run test to verify it fails**

Run: `cargo test -p arco-blocks test_block_port_clone`

Expected: compile failure while extraction is in-progress.

**Step 3: Write minimal implementation**

Move schema/coercion helpers into `schema.rs`:

- `is_pydantic_schema`, `is_dataclass_schema`, `dataclass_fields`
- `compare_fields`, `validate_data`, `outputs_schema_dict`
- `coerce_inputs`, `coerce_outputs`, `coerce_schema`

Wire imports and call sites in `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p arco-blocks test_block_port_clone`

Expected: PASS.

**Step 5: Commit**

`git commit -m "refactor(arco-blocks): extract schema and coercion module"`

### Task 3: Extract Utility Module

**Files:**

- Create: `crates/arco-blocks/src/util.rs`
- Modify: `crates/arco-blocks/src/lib.rs`

**Step 1: Write the failing test**

Add/adjust a unit test that compiles through `BlockModel::solve` references so moving utility functions causes compile failure until wiring is done.

**Step 2: Run test to verify it fails**

Run: `cargo test -p arco-blocks test_block_diagnostics_creation`

Expected: compile failure while extraction is in-progress.

**Step 3: Write minimal implementation**

Move into `util.rs`:

- `log_block_error`
- `rss_bytes`, `log_block_phase`
- `model_type`, `create_model`

Wire imports in `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p arco-blocks test_block_diagnostics_creation`

Expected: PASS.

**Step 5: Commit**

`git commit -m "refactor(arco-blocks): extract utility helper module"`

### Task 4: Extract Spec Module

**Files:**

- Create: `crates/arco-blocks/src/spec.rs`
- Modify: `crates/arco-blocks/src/lib.rs`

**Step 1: Write the failing test**

Add/adjust a unit test touching `Block::from_spec` compile path so move fails until module is wired.

**Step 2: Run test to verify it fails**

Run: `cargo test -p arco-blocks`

Expected: compile failure while extraction is in-progress.

**Step 3: Write minimal implementation**

Move into `spec.rs`:

- `BlockSpec`
- `SpecBuilder`, `SpecExtractor`
- `validate_spec`, `get_spec_attr`

Wire references in `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p arco-blocks`

Expected: PASS.

**Step 5: Commit**

`git commit -m "refactor(arco-blocks): extract block spec module"`

### Task 5: Extract Decorator Module

**Files:**

- Create: `crates/arco-blocks/src/decorator.rs`
- Modify: `crates/arco-blocks/src/lib.rs`

**Step 1: Write the failing test**

Add/adjust a compile path test for `block` decorator registration.

**Step 2: Run test to verify it fails**

Run: `cargo test -p arco-blocks`

Expected: compile failure while extraction is in-progress.

**Step 3: Write minimal implementation**

Move into `decorator.rs`:

- `ARCO_BLOCK_*` constants
- `FunctionBlockDecorator`
- `block`, `decorate_block_function`, `typed_block_meta_from_function`

Wire `add_blocks_submodule` registration references in `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p arco-blocks`

Expected: PASS.

**Step 5: Commit**

`git commit -m "refactor(arco-blocks): extract function block decorator module"`

### Task 6: Extract Resolve Module

**Files:**

- Create: `crates/arco-blocks/src/resolve.rs`
- Modify: `crates/arco-blocks/src/lib.rs`

**Step 1: Write the failing test**

Add/adjust a compile path test for solve/link resolution to fail until wiring is complete.

**Step 2: Run test to verify it fails**

Run: `cargo test -p arco-blocks`

Expected: compile failure while extraction is in-progress.

**Step 3: Write minimal implementation**

Move into `resolve.rs`:

- `resolve_links`, `extract_outputs`
- `block_spec`, `build_model_from_spec`, `inspect_model`
- `schemas_compatible`, `specs_are_swappable`

Wire imports/exports in `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p arco-blocks`

Expected: PASS.

**Step 5: Commit**

`git commit -m "refactor(arco-blocks): extract resolve and pyfunction module"`

### Task 7: Final Validation and Chore Update

**Files:**

- Modify: `chores.md`

**Step 1: Format**

Run: `cargo fmt --all`

Expected: formatting clean.

**Step 2: Lint**

Run: `cargo clippy --all --benches --tests --examples --all-features -- -D warnings`

Expected: PASS.

**Step 3: Test**

Run: `cargo test --workspace --all-features`

Expected: PASS.

**Step 4: Update chores**

Remove chore #1 from `chores.md` once implementation is complete.
