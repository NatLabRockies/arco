# Plan: Simplify Arco-KDL to Low-Level Only

## Goal
Remove all high-level declarative abstractions from arco-kdl, leaving only direct low-level optimization constructs (variables, constraints, objectives with explicit indexing).

## High-Level Constructs to REMOVE

### 1. Technology System (Full Removal)
- `TechnologyDecl` - Technology templates
- `AssetDecl` - Asset instances referencing technologies
- `InstancesDecl` - CSV-based instance generation
- Auto-wiring between technologies/assets/instances
- Technology-scoped variable declarations (investments, controls, states)

**Impact**: Eliminates the entire technology abstraction layer

### 2. Operation System (Full Removal)
- `OperationDecl` - Named operation blocks
- Operation-scoped constraints
- Operation references in assets

**Impact**: Removes operation-level grouping

### 3. Explicit Generation Syntax (Remove if not lowering)
- `GenerationBinding` - `over` clauses
- `generation_filter` - `when` clauses
- Constraint generation with implicit indexing

**Current State**: Parses but doesn't lower (already broken)
**Action**: Remove or implement proper lowering

### 4. Rules System (Full Removal)
- `RuleDecl` - Named reusable constraint groups
- Rule application/instantiation

**Impact**: Removes rule-based abstraction

### 5. Expression Declarations (Evaluate)
- `ExpressionDecl` - Named reusable expressions

**Decision**: Keep (low-level macro) or Remove (inline everything)

### 6. Semantic Layer (Full Removal)
- `semantic.rs` - Semantic validation pass
- Set existence checking
- Technology/asset reference validation
- Cross-reference validation

**Impact**: Eliminates semantic validation layer

### 7. Normalization Pass (Remove High-Level Transforms)
- Surface syntax transformation
- High-level to high-level conversions

**Keep**: Low-level normalizations only

## Low-Level Constructs to KEEP

### Core Model Direct (Keep)
- `ModelDecl` - Container (simplify: remove sub-declarations)
- `SetDecl` - Index sets (essential for indexing)
- `ParamDecl` - Parameters with explicit indices
- `ControlDecl` - Variables with explicit bounds and indices
- `ConstraintDecl` - Direct constraints with explicit formulas
- `ObjectiveDecl` - Objective with explicit formula
- `ScenarioDecl` - Scenario binding (simplify)

### Algebra (Keep)
- `Expr` - Expression AST
- Formula parsing
- Index notation (`x[a,t]`, `sum(a, expr)`)

### Source (Keep Simplified)
- KDL parsing
- Direct-to-lowering pipeline
- Error reporting

## Proposed New Minimal Structure

```rust
// Simplified SourceProgram
pub struct SourceProgram {
    pub sets: Vec<SetDecl>,           // Keep: essential for indexing
    pub parameters: Vec<ParamDecl>,   // Keep: data
    pub variables: Vec<VariableDecl>, // NEW: direct variable declarations
    pub constraints: Vec<ConstraintDecl>, // Keep: but simplified
    pub objectives: Vec<ObjectiveDecl>, // Keep
    pub scenarios: Vec<ScenarioDecl>,   // Simplified
}

// New direct variable declaration (replaces technology/operation scoping)
pub struct VariableDecl {
    pub name: String,           // Base name
    pub indices: Vec<String>,   // Index dimensions (a, t, etc.)
    pub lower: Option<BoundExpr>,
    pub upper: Option<BoundExpr>,
    pub kind: VariableKind,
}

// Simplified ConstraintDecl - remove generation bindings
pub struct ConstraintDecl {
    pub name: String,
    pub expression: String,     // Direct formula: "sum(a, gen[a,t]) <= capacity[a]"
    pub parsed_expression: ConstraintBody,
}

// Simplified ScenarioDecl - remove technology/asset/instance refs
pub struct ScenarioDecl {
    pub name: String,
    pub horizon: HorizonDecl,
    pub data: Vec<DataBindingDecl>,
    pub set_bindings: Vec<SetBindingDecl>,
}
```

## Migration Path

### Phase 1: Deprecation (1-2 releases)
1. Add deprecation warnings for technology/asset/instance declarations
2. Document migration examples
3. Provide automatic migration tool (transform high-level to low-level)

### Phase 2: Feature Flag (1 release)
1. Add `low-level-only` feature flag
2. Compile out high-level code paths
3. Test low-level-only mode

### Phase 3: Removal (1 release)
1. Remove high-level modules:
   - `semantic.rs` (or drastically simplify)
   - `normalize.rs` (high-level transforms)
   - Technology/asset/instance parsing
   - Operation blocks
2. Update `source.rs` to new minimal structure
3. Update `lowering.rs` to direct lowering

### Phase 4: Cleanup (ongoing)
1. Remove dead code
2. Update documentation
3. Performance optimization

## Files to Modify/Remove

### Remove Entirely
- `crates/arco-kdl/src/semantic.rs` - Or reduce to basic validation
- Technology/asset/instance parsing in `source.rs` (~500 lines)
- Operation block parsing in `source.rs` (~200 lines)
- Rule declaration parsing

### Major Changes
- `source.rs` - Remove ~50% of declaration types
- `lowering.rs` - Simplify lowering pipeline
- `pipeline.rs` - Remove semantic pass
- `lib.rs` - Update exports

### Keep Mostly As-Is
- `algebra.rs` - Core expression system
- `algebra_diagnostics.rs` - Error reporting
- Basic KDL parsing infrastructure

## Example: Before and After

### BEFORE (High-Level with Technology)
```kdl
model "dispatch" {
    set "time" "t" {}

    technology "Generator" {
        invest name="capacity" lower=0 {}
        control name="gen" indices=["t"] lower=0 {}
    }

    asset "Gen1" technology="Generator" {
        param "capacity" value=100.0
    }

    scenario "base" {
        assets=["Gen1"]
        horizon { years 2024..2030 }
    }
}
```

### AFTER (Low-Level Direct)
```kdl
model "dispatch" {
    set "assets" "a" {}
    set "time" "t" {}

    // Direct variable declarations (no technology abstraction)
    variable "capacity" indices=["a"] lower=0
    variable "gen" indices=["a", "t"] lower=0

    // Direct constraint (no generation bindings)
    constraint "balance" expr="sum(a, gen[a,t]) >= demand[t]"

    scenario "base" {
        horizon { years 2024..2030 }
        set "assets" csv="assets.csv"
    }
}
```

## Benefits

1. **Simpler Mental Model**: Users write optimization directly
2. **Less Code**: ~40% reduction in kdl crate size
3. **Faster Compilation**: Fewer passes, no semantic analysis
4. **Easier Debugging**: Direct mapping from KDL to solver
5. **More Flexible**: No imposed structure from abstractions

## Costs

1. **More Verbose**: Users must write explicit indices
2. **Less Guidance**: No technology templates
3. **Migration Effort**: Existing models need conversion
4. **Feature Loss**: Auto-wiring, technology libraries

## Decision Needed

**Question**: Should we also remove or simplify:
1. Sets (keep as essential indexing)?
2. Scenarios (simplify to just data binding)?
3. Expression declarations (inline macros)?
4. CSV data loading (keep for practicality)?

## Next Steps

1. Review this plan with stakeholders
2. Create proof-of-concept branch
3. Migrate 1-2 example models
4. Measure code reduction and performance
5. Decide on deprecation timeline
