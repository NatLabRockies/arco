# KDL Low-Level Hard Cut Implementation Plan

> **For agentic workers:** REQUIRED: Use `superpowers:subagent-driven-development` if subagents are available, or `superpowers:executing-plans` otherwise. Steps use checkbox syntax for tracking.

**Goal:** Replace Arco's mixed KDL surface with the low-level profile in `docs/arco-spec.md#11-grammar-low-level-profile`, remove the legacy high-level DSL entirely, and leave code, tests, docs, examples, and tooling aligned to that single language.

**Architecture:** Keep only KDL surface normalization needed for math-block sugar. Remove direct-wiring and all high-level declarations. Rebuild the compiler flow around low-level AST parsing, section 9 semantic validation, and generic set/data lowering. Enforce a temporary repo rule of one `scenario` per file with an explicit diagnostic, not implicit first-scenario behavior.

**Tech Stack:** Rust (`arco-kdl`, `arco-cli`), KDL 2.0, tree-sitter overlay grammar, `cargo`, `just`.

---

## File Map

Core compiler files:

- `crates/arco-kdl/src/source.rs`
- `crates/arco-kdl/src/normalize.rs`
- `crates/arco-kdl/src/pipeline.rs`
- `crates/arco-kdl/src/semantic.rs`
- `crates/arco-kdl/src/lowering.rs`
- `crates/arco-kdl/src/algebra.rs`
- `crates/arco-kdl/src/algebra_diagnostics.rs`
- `crates/arco-kdl/src/lib.rs`

Primary test files:

- `crates/arco-kdl/tests/source_parser.rs`
- `crates/arco-kdl/tests/semantic_validation.rs`
- `crates/arco-kdl/tests/semantic_expression_validation.rs`
- `crates/arco-kdl/tests/algebra_parser.rs`
- `crates/arco-kdl/tests/lowering_suite.rs`
- `crates/arco-kdl/tests/repo_kdl_validity.rs`

Tests to delete or replace:

- `crates/arco-kdl/tests/normalization_suite.rs`

Likely new focused test file:

- `crates/arco-kdl/tests/low_level_data_suite.rs`

Docs and examples:

- `README.md`
- `examples/README.md`
- `examples/generator-allocation/input.kdl`
- `examples/price-taker-battery/input.kdl`
- `examples/simple-electricity-market-storage/input.kdl`
- `examples/capacity-expansion/input.kdl`
- `docs/tutorials/single-zone-storage.md`
- `docs/reference/rfds/rfd-0011-inferred-declarations-and-explicit-control.md`
- `docs/reference/rfds/rfd-0012-semantic-program-construction-contract.md`
- `docs/reference/rfds/rfd-0014-generalized-grammar.md`
- `notes.md`

Tooling:

- `tools/tree-sitter-arco-kdl/grammar.js`
- `tools/tree-sitter-arco-kdl/test/corpus/arco_math.txt`
- `crates/arco-cli/src/debug.rs`

---

## Chunk 1: Lock the Spec With Tests

**Files:**

- Create/Modify: `crates/arco-kdl/tests/source_parser.rs`
- Create/Modify: `crates/arco-kdl/tests/semantic_validation.rs`
- Create/Modify: `crates/arco-kdl/tests/semantic_expression_validation.rs`
- Create/Modify: `crates/arco-kdl/tests/algebra_parser.rs`
- Create/Modify: `crates/arco-kdl/tests/lowering_suite.rs`
- Create: `crates/arco-kdl/tests/low_level_data_suite.rs`
- Delete/Replace: `crates/arco-kdl/tests/normalization_suite.rs`

- [ ] Add parser tests for top-level `data`, top-level `subset`, model-local `expression`, `index_by=`, `index` child nodes, `report dual`, and KDL 2.0 type annotations on nodes and values.
- [ ] Add explicit parser rejection tests for removed declarations: `technology`, `operation`, `asset`, `instances`, `rule`, top-level `expression`, top-level `minimize` or `maximize`, scenario custom sets, and scenario `set` bindings.
- [ ] Add semantic tests for section 9 requirements that are currently missing: duplicate names, field resolution, `subset_of` parent and cycle checks, non-unique indexing without `reduce`, invalid comparators, contradictory bounds, expression cycles, `over in=` unknown set, subset resolution, empty CSV diagnostics, report-to-objective resolution, and scenario `data` name mismatch with model `param`.
- [ ] Add a semantic test that multiple `scenario` blocks fail with an explicit diagnostic.
- [ ] Add algebra parser tests for inline selectors such as `generator_data[class=solar area=north]`.
- [ ] Add lowering tests for top-level `data` namespaces, `subset`, filtered params, reductions, and scenario-level `data` shadowing a top-level `data` name.
- [ ] Delete `crates/arco-kdl/tests/normalization_suite.rs` once the legacy normalization path is removed, or replace it with low-level-only normalization tests for math-block sugar.

**Run during red phase:**

```bash
cargo test -p arco-kdl --test source_parser
cargo test -p arco-kdl --test semantic_validation
cargo test -p arco-kdl --test semantic_expression_validation
cargo test -p arco-kdl --test algebra_parser
cargo test -p arco-kdl --test lowering_suite
```

---

## Chunk 2: Reset the Parser and AST to Low-Level Only

**Files:**

- Modify: `crates/arco-kdl/src/source.rs`
- Modify: `crates/arco-kdl/src/normalize.rs`
- Modify: `crates/arco-kdl/src/pipeline.rs`
- Modify: `crates/arco-kdl/src/lib.rs`

- [ ] Redesign `SourceProgram` around only `data`, `subset`, `model`, and `scenario`.
- [ ] Add AST types for low-level data modeling:
  - `DataDecl`
  - `MapDecl`
  - data-local `SetDecl` with comparators and `subset_of`
  - `IndexDecl`
  - data-local `ParamDecl` with `from`, `index_by`, `index` children, `reduce`, `units`, filter metadata, and type-annotation metadata
- [ ] Add a top-level `SubsetDecl`.
- [ ] Move `ExpressionDecl` and `ObjectiveDecl` into the `model` path only.
- [ ] Remove high-level AST types and parser branches for:
  - `technology`
  - `operation`
  - `asset`
  - `instances`
  - `rule`
  - scenario custom sets
  - scenario `set` bindings
- [ ] Remove top-level `set`, top-level `expression`, and top-level objective parsing.
- [ ] Preserve `normalize_surface_syntax` for math-block support.
- [ ] Remove only the direct-wiring and canonical-model normalization path from `normalize.rs`.
- [ ] Update `pipeline.rs` so validation no longer depends on legacy normalization behavior.

**Run during green phase:**

```bash
cargo test -p arco-kdl --test source_parser
```

---

## Chunk 3: Rewrite Semantics Around the Low-Level Contract

**Files:**

- Modify: `crates/arco-kdl/src/semantic.rs`
- Modify: `crates/arco-kdl/tests/semantic_validation.rs`
- Modify: `crates/arco-kdl/tests/semantic_expression_validation.rs`

- [ ] Replace `first_scenario()` selection with an explicit semantic rule:
  - Zero scenarios is an error
  - More than one scenario is an error for now
  - Exactly one scenario is active
- [ ] Remove asset or instance-driven semantic construction and implicit `assets` or `candidate_assets` assumptions.
- [ ] Make `time` the only built-in set implied by the spec.
- [ ] Build semantic registries for:
  - top-level `data` blocks
  - top-level `subset`s
  - model sets and params
  - scenario bindings
- [ ] Implement section 9 validation as first-class semantic checks, not late lowering errors.
- [ ] Add expression cycle detection instead of the current recursion dedupe.
- [ ] Resolve scalar reports against model-local expressions or the model objective name.
- [ ] Resolve dual reports against known constraints.
- [ ] Validate that scenario `data` binding names match model `param` declarations.
- [ ] Validate `over in=` against known sets before lowering.
- [ ] Validate top-level subset fields against the referenced data block schema.
- [ ] Decide and document how typed annotations participate in column-type inference and failure messages.

**Run:**

```bash
cargo test -p arco-kdl --test semantic_validation
cargo test -p arco-kdl --test semantic_expression_validation
```

---

## Chunk 4: Rebuild Lowering Around Data Namespaces

**Files:**

- Modify: `crates/arco-kdl/src/lowering.rs`
- Modify: `crates/arco-kdl/tests/lowering_suite.rs`

- [ ] Replace the asset or instance-centric input model with top-level CSV-backed `data` namespaces.
- [ ] Implement `map` resolution from logical names to source headers.
- [ ] Implement data-local `set` extraction, `subset_of` hierarchy evaluation, default `index`, and filter comparators.
- [ ] Implement data-local `param` projection, indexing, aggregation reducers, units metadata retention, and validation-sensitive shadowing behavior.
- [ ] Implement top-level `subset` materialization as named filtered domains.
- [ ] Implement scenario-level `data` bindings as model-parameter inputs that can shadow a top-level `data` name when the spec says they should.
- [ ] Remove legacy lowering code that expands `technology`, `operation`, `asset`, `instances`, and `rule`.
- [ ] Make variable and constraint instantiation iterate over declared low-level sets, not legacy built-ins.

**Run:**

```bash
cargo test -p arco-kdl --test lowering_suite
```

---

## Chunk 5: Extend Algebra for Inline Selectors

**Files:**

- Modify: `crates/arco-kdl/src/algebra.rs`
- Modify: `crates/arco-kdl/src/algebra_diagnostics.rs`
- Modify: `crates/arco-kdl/tests/algebra_parser.rs`

- [ ] Teach bracket parsing to distinguish variable indexing from inline selectors by the presence of `=` inside the brackets.
- [ ] Represent inline selectors in the algebra AST without breaking existing indexed-variable rendering.
- [ ] Keep support for ordinary `x[a,t]` and temporal offsets like `x[a,t-1]`.
- [ ] Add diagnostics that make selector parse failures readable.

**Run:**

```bash
cargo test -p arco-kdl --test algebra_parser
```

---

## Chunk 6: Migrate Repo Fixtures, Docs, and Examples in the Same Slice as the Parser Flip

**Files:**

- Modify: `README.md`
- Modify: `examples/README.md`
- Modify/Create: `examples/generator-allocation/input.kdl`
- Delete/Replace: `examples/price-taker-battery/input.kdl`
- Delete/Replace: `examples/simple-electricity-market-storage/input.kdl`
- Delete/Replace: `examples/capacity-expansion/input.kdl`
- Delete/Replace: `docs/tutorials/single-zone-storage.md`
- Modify/Delete: `notes.md`

- [ ] Update the README quickstart to a pure low-level example.
- [ ] Migrate `examples/generator-allocation/input.kdl` to documented low-level syntax, including `index_by=` or `index` child form as appropriate.
- [ ] Rewrite or delete the three legacy high-level example inputs in the same change slice as the parser removal so repo-wide `.kdl` canaries never stay broken.
- [ ] Rewrite or remove `docs/tutorials/single-zone-storage.md`, since it currently teaches the removed DSL.
- [ ] Update `examples/README.md` so only supported examples are advertised.
- [ ] Delete or rewrite `notes.md`, because it contains stale syntax like `index_by=[plant_id,unit_id]` that does not match the reference spec.
- [ ] Do not weaken `docs/arco-spec.md#11-grammar-low-level-profile` to match temporary implementation shortcuts. If the single-scenario limitation needs documentation, put it in README or contributor docs as an implementation note.

**Run:**

```bash
cargo test -p arco-kdl --test repo_kdl_validity
```

---

## Chunk 7: Remove Historical High-Level References and Fix Tooling

**Files:**

- Delete/Archive: `docs/reference/rfds/rfd-0011-inferred-declarations-and-explicit-control.md`
- Delete/Archive: `docs/reference/rfds/rfd-0012-semantic-program-construction-contract.md`
- Delete/Archive: `docs/reference/rfds/rfd-0014-generalized-grammar.md`
- Modify: `tools/tree-sitter-arco-kdl/grammar.js`
- Modify: `tools/tree-sitter-arco-kdl/test/corpus/arco_math.txt`
- Modify: `crates/arco-cli/src/debug.rs`

- [ ] Delete or clearly archive RFDs that are now wrong because they describe the removed high-level layer.
- [ ] Expand the tree-sitter overlay grammar beyond math-only assumptions so the remaining low-level KDL parses cleanly.
- [ ] Add tree-sitter corpus coverage for low-level declarations and updated math bodies.
- [ ] Fix stale path or example assumptions in tests such as `crates/arco-cli/src/debug.rs`.

**Run:**

```bash
just kdl-overlay-check
```

---

## Chunk 8: Final Verification and Finish

**Files:** all touched files

- [ ] Run formatting.
- [ ] Run clippy with warnings as errors.
- [ ] Run the targeted `arco-kdl` test binaries.
- [ ] Run repo-wide `.kdl` parse validity.
- [ ] Run overlay grammar validation.
- [ ] Re-read README and surviving examples to ensure they only teach supported syntax.

**Final commands:**

```bash
cargo fmt --all
cargo clippy --all --benches --tests --examples --all-features -- -D warnings
cargo test -p arco-kdl --test source_parser
cargo test -p arco-kdl --test semantic_validation
cargo test -p arco-kdl --test semantic_expression_validation
cargo test -p arco-kdl --test algebra_parser
cargo test -p arco-kdl --test lowering_suite
cargo test -p arco-kdl --test repo_kdl_validity
just kdl-overlay-check
```

---

## Notes

- The plan explicitly preserves `normalize_surface_syntax` while removing direct-wiring normalization.
- The plan explicitly rejects multiple scenarios for now, rather than silently taking the first.
- The plan treats repo fixture migration as part of the parser flip, not later cleanup, because `repo_kdl_validity` and `kdl-overlay-check` parse every `.kdl` file in the repo.

**Last updated:** 2026-04-07
