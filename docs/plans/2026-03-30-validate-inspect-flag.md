# Validate Inspect Flag Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `--inspect <category>` and `--name <element>` flags to `arco validate` so users can inspect resolved semantic elements (sets, constraints, variables, parameters, expressions, objective, reports, chronology) from a KDL file without running a full solve.

**Architecture:** The `validate` pipeline already produces a full `SemanticProgram` with all resolved data. This plan adds CLI flags that select which portion to render, and formatting functions that produce human-readable output for each category. Without `--inspect`, validate behavior is unchanged. With `--inspect`, the output shifts to category-specific detail. With `--name`, it drills into a single named element.

**Tech Stack:** Rust workspace, `clap` (ValueEnum), `arco-kdl` (SemanticProgram), `arco-cli` (driver, main).

---

## CLI syntax

```bash
# Normal validation (unchanged)
arco validate input.kdl

# Inspect a category — lists elements with counts
arco validate input.kdl --inspect sets
arco validate input.kdl --inspect constraints
arco validate input.kdl --inspect variables
arco validate input.kdl --inspect parameters
arco validate input.kdl --inspect expressions
arco validate input.kdl --inspect objective
arco validate input.kdl --inspect reports
arco validate input.kdl --inspect chronology

# Drill into a specific element by name
arco validate input.kdl --inspect sets --name nodes
arco validate input.kdl --inspect constraints --name meet_demand
arco validate input.kdl --inspect variables --name dispatch
```

## Category output specification

| `--inspect` value | Without `--name`                                                                                       | With `--name <x>`                                                                      |
| ----------------- | ------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------- |
| `sets`            | List all set names + member counts (built-in: assets, candidate_assets, time; user: from set_registry) | Print full member list of set `<x>`                                                    |
| `constraints`     | List constraint names + source kind                                                                    | Print constraint `<x>`: source_kind, source_name, expression_text, generation_bindings |
| `variables`       | List variable family signatures (e.g. `dispatch[a,t]`)                                                 | Print variable `<x>`: signature, index_domains, overrides (kind, bounds)               |
| `parameters`      | List all parameters grouped by type (series, indexed, asset)                                           | Print parameter `<x>`: type + name                                                     |
| `expressions`     | List named expression names                                                                            | Print expression `<x>`: formula_text                                                   |
| `objective`       | Print objective name, sense, expression_text                                                           | (`--name` ignored)                                                                     |
| `reports`         | List report names                                                                                      | Print report `<x>`: formula_text                                                       |
| `chronology`      | Print boundary conditions                                                                              | (`--name` ignored)                                                                     |

If `--name` doesn't match any element, print an error listing available names.
Note that naming is optional for all the assets.

---

## Chunk 1: CLI flag plumbing

### Task 1: Add `InspectCategory` enum and CLI flags

**Files:**

- Modify: `crates/arco-cli/src/main.rs`

- [ ] **Step 1: Add `InspectCategory` enum**
- Derive `Clone`, `Copy`, `Debug`, `ValueEnum` from clap.
- Variants: `Sets`, `Constraints`, `Variables`, `Parameters`, `Expressions`, `Objective`, `Reports`, `Chronology`.

- [ ] **Step 2: Add flags to `Validate` variant**
- Add `--inspect` as `Option<InspectCategory>` and `--name` as `Option<String>`:

```rust
Validate {
    path: PathBuf,
    /// Inspect a specific semantic category
    #[arg(long)]
    inspect: Option<InspectCategory>,
    /// Filter to a specific element by name within the inspected category
    #[arg(long)]
    name: Option<String>,
},
```

- [ ] **Step 3: Pass flags to `validate_file_report()`**
- Update the `Command::Validate` match arm to forward `inspect` and `name` to the driver.

- [ ] **Step 4: Run build check**
- Run: `just check`
- Expected: PASS (driver signature change happens in Task 2)

- [ ] **Step 5: Commit**

```bash
git add crates/arco-cli/src/main.rs
git commit -m "feat: add --inspect and --name flags to arco validate"
```

---

## Chunk 2: Driver formatting logic

### Task 2: Update `validate_file_report()` with inspect rendering

**Files:**

- Modify: `crates/arco-cli/src/driver.rs`

- [ ] **Step 1: Update `validate_file_report()` signature**

```rust
pub fn validate_file_report(
    path: &Path,
    inspect: Option<InspectCategory>,
    name: Option<&str>,
) -> Result<String, DriverError>
```

- Import `InspectCategory` from `main.rs` or define it in a shared location accessible to both files.

- [ ] **Step 2: Implement the dispatch logic**
- When `inspect` is `None`: keep current behavior (validation summary with counts).
- When `inspect` is `Some(category)`: call a category-specific formatting function.

- [ ] **Step 3: Implement `format_inspect_sets()`**
- Without `--name`: list all sets with counts:
  - Built-in: `assets` (from `sets.assets.len()`), `candidate_assets` (from `sets.candidate_assets.len()`), `time` (steps + resolution).
  - User-declared: iterate `set_registry`, print name + `values.len()`.
- With `--name`: look up in built-in sets first (match "assets", "candidate_assets", "time"), then `set_registry`. Print full member list. Error with available names if not found.

- [ ] **Step 4: Implement `format_inspect_constraints()`**
- Without `--name`: list constraint names + source_kind from `active_constraints`.
- With `--name`: find by name, print `source_kind`, `source_name`, `expression_text`, and `generation_bindings` details.

- [ ] **Step 5: Implement `format_inspect_variables()`**
- Without `--name`: list variable family signatures via `render()` from `variable_families`.
- With `--name`: find by target name, print signature, `index_domains`, and any overrides from `variable_overrides`.

- [ ] **Step 6: Implement `format_inspect_parameters()`**
- Without `--name`: list grouped by type (series, indexed, asset) from `parameters`.
- With `--name`: find which group contains it, print type + name.

- [ ] **Step 7: Implement `format_inspect_expressions()`**
- Without `--name`: list expression names from `active_expressions`.
- With `--name`: find by name, print `formula_text`.

- [ ] **Step 8: Implement `format_inspect_objective()`**
- Print `name`, `sense`, `expression_text` from `active_objective`. Ignore `--name`.

- [ ] **Step 9: Implement `format_inspect_reports()`**
- Without `--name`: list report names from `active_reports`.
- With `--name`: find by name, print `formula_text`.

- [ ] **Step 10: Implement `format_inspect_chronology()`**
- Print `initial_boundary`, `terminal_boundary`, `initial_commitment_boundary` from `chronology`. Ignore `--name`.

- [ ] **Step 11: Run build and lint**
- Run: `just check && just clippy`
- Expected: PASS

- [ ] **Step 12: Commit**

```bash
git add crates/arco-cli/src/driver.rs
git commit -m "feat: implement inspect rendering for all semantic categories"
```

---

## Chunk 3: Tests

### Task 3: Add CLI tests for inspect functionality

**Files:**

- Modify: `crates/arco-cli/tests/cli_run.rs` (or appropriate test file)

- [ ] **Step 1: Add test for `--inspect sets` (list mode)**
- Use existing fixture `examples/price-taker-battery/input.kdl`.
- Assert output contains `assets: 1` and `time: 24 steps`.

- [ ] **Step 2: Add test for `--inspect sets --name assets` (detail mode)**
- Assert output contains `Battery1`.

- [ ] **Step 3: Add test for `--inspect constraints` (list mode)**
- Assert output lists constraint names from the fixture.

- [ ] **Step 4: Add test for `--inspect variables` (list mode)**
- Assert output contains variable signatures like `charge[a,t]`.

- [ ] **Step 5: Add test for `--inspect sets --name nonexistent` (error case)**
- Assert output contains an error message listing available set names.

- [ ] **Step 6: Add test for `--inspect objective`**
- Assert output contains objective name and sense.

- [ ] **Step 7: Update any existing validate tests**
- If existing tests call `validate_file_report` with the old signature, update them to pass `None, None`.

- [ ] **Step 8: Run full test suite**
- Run: `just tdd-refactor arco-cli`
- Expected: PASS

- [ ] **Step 9: Commit**

```bash
git add crates/arco-cli/tests/
git commit -m "test: add cli tests for validate --inspect flag"
```

---

## Cross-cutting design rules

- **No new crates or modules** — this is a small feature contained in `main.rs` and `driver.rs`.
- **No changes to SemanticProgram** — all data is already available, we only add rendering.
- **Graceful --name miss** — always list available names when a lookup fails, so the user can discover what's available.
- **--name without --inspect is a no-op** — silently ignored or warn. Don't error on it.

---

## Verification checklist

After all chunks are complete, run the full quality gate:

```bash
# Format + lint + test (single command)
just step-quality

# Or individually:
just fmt                              # format rust
just clippy                           # lint rust
just test                             # run all rust tests

# Targeted validation:
just tdd-green arco-cli inspect       # run inspect-related tests
just tdd-refactor arco-cli            # fmt + clippy + test for arco-cli

# Manual smoke tests:
cargo run -p arco-cli -- validate examples/price-taker-battery/input.kdl
cargo run -p arco-cli -- validate examples/price-taker-battery/input.kdl --inspect sets
cargo run -p arco-cli -- validate examples/price-taker-battery/input.kdl --inspect sets --name assets
cargo run -p arco-cli -- validate examples/price-taker-battery/input.kdl --inspect constraints
cargo run -p arco-cli -- validate examples/price-taker-battery/input.kdl --inspect variables
cargo run -p arco-cli -- validate examples/price-taker-battery/input.kdl --inspect objective
```

## Expected output examples

### `arco validate input.kdl --inspect sets`

```
sets:
  assets: 1
  candidate_assets: 0
  time: 24 steps @ PT1H
```

### `arco validate input.kdl --inspect sets --name assets`

```
set "assets": ["Battery1"]
```

### `arco validate input.kdl --inspect constraints`

```
constraints:
  soc_balance (constraint)
  charge_capacity (constraint)
  discharge_capacity (constraint)
```

### `arco validate input.kdl --inspect variables`

```
variables:
  charge[a,t]
  discharge[a,t]
  soc[a,t]
```

### `arco validate input.kdl --inspect objective`

```
objective:
  name: total_profit
  sense: maximize
  expr: sum(prices[t] * discharge[a,t] - prices[t] * charge[a,t] for a in assets for t in time)
```
