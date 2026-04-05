# Arco-KDL UX Review Summary

## Review Scope
Reviewed the arco-kdl crate for KDL syntax ergonomics, error messages, developer experience, and overall usability when writing optimization models.

## Files Reviewed
- `crates/arco-kdl/src/source.rs` - Source parsing and error reporting
- `crates/arco-kdl/src/semantic.rs` - Semantic validation
- `crates/arco-kdl/src/lowering.rs` - Lowering to core model
- `crates/arco-kdl/src/algebra.rs` - Expression syntax parser
- `crates/arco-kdl/src/normalize.rs` - Surface syntax normalization
- `crates/arco-kdl/src/pipeline.rs` - Compilation pipeline
- `crates/arco-kdl/tests/e2e/*/input.kdl` - Example KDL files
- `docs/reference/kdl-syntax-summary.md` - Documentation

---

## 1. KDL Syntax Ergonomics

### Positive Findings

**1.1 Intuitive Block-Based Structure**
The KDL syntax uses a clean hierarchical structure that maps well to optimization concepts:
```kdl
technology Generator {
  control dispatch
}

operation Dispatch {
  constraint capacity_limit {
    dispatch[g,t] <= capacity_mw[g]
  }
}
```

**1.2 Flexible Naming Syntax**
Supports both positional and property-based naming:
```kdl
technology "Generator"    // positional
technology name="Generator" // property
```

**1.3 Natural Math Expression Syntax**
Algebra expressions in blocks feel natural:
```kdl
constraint balance {
  sum(dispatch[a,t] for a in assets) = load[t]
}
```

**1.4 Auto-Wiring Reduces Boilerplate**
The SDOM example demonstrates auto-wiring where technologies, operations, and rules are inferred from instances/assets, reducing repetition.

**1.5 Technology `as=` Property**
The `as=` alias for naming asset sets makes constraints more explicit and greppable:
```kdl
technology PV as=pv_plants {
  control pv_dispatch
}
// Later: sum(pv_dispatch[a,t] for a in pv_plants)
```

### Issues Found

**1.6 Inconsistent Constraint Syntax Forms**
There are multiple ways to write constraints, which can confuse users:

- Simple form: `constraint "name" { expr }`
- Property form: `constraint name="..." if="..." { expr }`
- Explicit generation form: `constraint { over "n" in="..." expr { ... } }`

The documentation states: "The `over`/`when`/`expr` form parses correctly but lowering to solver form is not yet implemented." This is a significant UX gap - users can write valid KDL that won't work.

**1.7 Hidden Index Convention**
The implicit `a` (asset) and `t` (time) indices are not obvious to new users. The system assumes these conventions without clear documentation in the KDL files themselves.

**1.8 Math Block vs Expression Property Inconsistency**
Some declarations use math blocks while others use expression properties:
```kdl
// In constraint:
constraint "name" { dispatch[a,t] <= capacity[a] }

// In minimize (after normalization becomes property)
minimize "Cost" expression="..."
```

The surface syntax normalization hides this from users, but it creates cognitive overhead.

---

## 2. Error Messages

### Positive Findings

**2.1 Miette Integration for Rich Diagnostics**
The crate uses `miette` for structured error reporting with diagnostic codes:
```rust
#[error("missing required node `{name}` in {path}")]
#[diagnostic(code(arco::source::missing_node), help("add a `{name}` child declaration"))]
```

**2.2 Source Span Tracking**
Errors include source spans for pointing to specific locations in the KDL file.

**2.3 Hierarchical Error Types**
Well-structured error taxonomy:
- `SourceError` - Parse-time errors (KDL structure, missing nodes/properties)
- `SemanticError` - Validation errors (missing declarations, data mismatches)
- `LoweringError` - Compilation errors (missing parameters, invalid formulations)
- `NormalizeError` - Normalization errors

**2.4 Helpful Error Messages with Suggestions**
Many errors include actionable help text:
```rust
#[diagnostic(
    code(arco::semantic::missing_declaration),
    help("add the missing declaration or update the reference to an existing one")
)]
```

### Issues Found

**2.5 Algebra Parse Errors Lack Context**
In `algebra.rs`, parse errors only include position (byte offset) but not line/column:
```rust
pub fn position(&self) -> usize {
    self.position  // Just a byte offset
}
```

Users must manually map byte offsets to file locations. The error display format is also basic:
```
<message> at byte <position>
```

**2.6 KDL Parse Errors Don't Chain Well**
When KDL parsing fails, the error doesn't always clearly indicate which node caused the issue. The `SourceError::Kdl` variant wraps `kdl::KdlError` but the integration could provide more context.

**2.7 Missing Suggestions for Common Mistakes**
Some errors don't suggest fixes:
- Missing scenario: suggests "add a `scenario` declaration" ✓
- Missing declaration: suggests adding/updating reference ✓
- Invalid algebra: generic "fix the algebra syntax" message ✗

**2.8 No Error Recovery**
The parser stops at the first error. For large model files, users must fix errors one at a time.

---

## 3. Error Recovery

### Issues Found

**3.1 No Partial Parsing**
The parser doesn't support partial/incremental parsing. Any syntax error causes complete failure.

**3.2 Strict Mode No Flexibility**
No "lenient mode" for exploratory modeling where users could see what the parser understood even with errors.

**3.3 No Suggestion Engine for Typos**
When referencing non-existent declarations, the error doesn't suggest similar names:
```kdl
maximize "SystemCost"  // typo: maximize "SystemCost"
// Error: missing declaration, but doesn't suggest "SystemCost"
```

---

## 4. Documentation

### Positive Findings

**4.1 Comprehensive Syntax Documentation**
The `kdl-syntax-summary.md` is thorough with:
- Syntax examples for all declaration types
- Table of properties for each node type
- Clear separation of high-level vs low-level layers

**4.2 Inline Comments in Examples**
The SDOM example has helpful comments explaining auto-wiring and syntax features.

### Issues Found

**4.2 Documentation Gap for Constraint Forms**
The documentation states explicit generation form is "not yet implemented" but doesn't prominently warn users not to use it.

**4.3 Missing Troubleshooting Section**
No section on common errors and how to fix them.

**4.4 No Visual Guide**
Documentation is text-heavy. A visual diagram showing the relationship between technologies, operations, rules, assets, and scenarios would help new users.

**4.5 Index Convention Not Documented**
The implicit `a` and `t` index variables are not prominently documented in the KDL syntax guide.

---

## 5. Semantic Validation

### Positive Findings

**5.1 Comprehensive Validation Checks**
- Duplicate asset detection
- Missing declaration validation
- Data binding validation (CSV column existence)
- Time series length validation
- Indexed data dimensions validation
- Chronology boundary validation

**5.2 Auto-Derived Variable Families**
The semantic analysis correctly derives implicit variable families (e.g., `unserved_energy`, `build` for candidates).

### Issues Found

**5.3 Limited Set Validation**
Custom sets defined in scenarios aren't fully validated against their usage in constraints.

**5.4 No Unused Declaration Warnings**
Declarations not referenced in the active scenario don't produce warnings. This could help users catch typos or incomplete wiring.

**5.5 Data Type Validation Missing**
No validation that CSV data types match expected parameter types (e.g., numeric vs string).

---

## 6. Common Patterns & User Struggles

### Patterns Observed in Tests

**6.1 Asset vs Instances Duality**
Users must choose between individual assets and bulk instances. The distinction is clear but adds cognitive load.

**6.2 Technology Set Aliasing**
The `as=` property is used inconsistently in examples. Some use it, others don't.

**6.3 Expression References**
Named expressions can reference other expressions, creating a dependency graph. The semantic validation checks for missing dependencies.

**6.4 Formula Bounds vs Literal Bounds**
Control bounds can be either literals or formulas, creating two mental models:
```kdl
// Literal bound
control dispatch lower=0

// Formula bound (via child node)
control output {
  upper { max_cap[a] }
}
```

---

## 7. Lowering/Diagnostics

### Positive Findings

**7.1 Traceability Records**
The lowering phase maintains traceability from DSL names to lowered names.

**7.2 Pipeline Timing**
The compilation pipeline tracks timing for parse/validate/lower phases, useful for performance analysis.

### Issues Found

**7.3 Constraint Filter Limitations**
Constraint filters in lowering have strict limitations but the error messages don't clearly explain what's supported:
```rust
Expr::FunctionCall { .. } => Err(invalid_constraint_filter(
    constraint,
    path,
    "function calls are not supported in constraint filters",
))
```

**7.4 No Lowering Explanation**
When lowering fails, users don't get an explanation of what was being attempted (e.g., "generating constraint instances for rule X...").

**7.5 Missing Debug Mode**
No way to see the intermediate representations (normalized, semantic, lowered) for debugging model issues.

---

## Summary of Key Issues

| Category | Severity | Issue |
|----------|----------|-------|
| Error Messages | High | Algebra parse errors use byte offsets, not line/column |
| Syntax | High | Explicit generation form parses but doesn't lower |
| Documentation | Medium | Missing troubleshooting and visual guides |
| Error Recovery | Medium | No partial parsing or typo suggestions |
| Validation | Medium | No unused declaration warnings |
| Lowering | Medium | No intermediate representation debug output |

---

## Recommendations

1. **Improve Algebra Error Reporting**: Add line/column information to algebra parse errors
2. **Document Constraint Form Status**: Prominently mark unimplemented features in docs
3. **Add Unused Warning**: Warn about declarations not used in active scenario
4. **Implement Error Recovery**: Allow parsing to continue past errors where possible
5. **Add Typos Suggestions**: Suggest similar names when declarations are not found
6. **Add Debug Mode**: Option to output intermediate representations
7. **Visual Documentation**: Add diagrams showing the relationship between concepts
8. **Explicit Index Documentation**: Document the `a` and `t` conventions prominently
