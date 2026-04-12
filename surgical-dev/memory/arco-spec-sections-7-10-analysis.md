# Arco Specification Sections 7-10: Surgical Analysis Report

**Agent:** surgical-dev
**Target:** docs/arco-spec.md (lines 1310-2053)
**Focus Areas:** Execution Semantics, Validation Rules Clarity, Specification Gaps
**Date:** 2026-04-12

---

## Executive Summary

The surgical-dev analysis identified **3 critical gaps**, **5 clarity issues**, and **2 structural inconsistencies** in sections 7-10 of the Arco specification. The specification is generally well-structured but has several areas where implementation guidance is ambiguous or missing.

---

## Section 7: `scenario` Declaration Analysis

### 7.1 Execution Semantics Clarity

**Status:** ⚠️ PARTIALLY CLEAR - Gaps Identified

**Findings:**

1. **Multi-scenario execution order (GAP)**
   The spec states: "When multiple `scenario` declarations exist in a document, the execution order is implementation-defined."
   - **Issue:** This creates non-deterministic behavior for users
   - **Risk:** Scenarios with data dependencies across scenarios will fail unpredictably
   - **Recommendation:** Add explicit ordering mechanism or declare cross-scenario dependencies unsupported

2. **Parallel execution state isolation (CLARITY ISSUE)**
   "Each scenario is independent and MUST NOT share mutable state with other scenarios."
   - **Question:** Are immutable data (top-level `data` blocks) shared across parallel scenarios?
   - **Ambiguity:** Unclear if top-level sets/params are copied or referenced
   - **Recommendation:** Explicitly state that top-level data is read-only shared across all scenarios

3. **Report output file naming (GAP)**
   The spec RECOMMENDS CSV format but does not specify:
   - Output file naming convention
   - How multiple reports in one scenario are organized
   - Whether directory structure is preserved

### 7.2 Data Binding Rules (Section 7.2, 7.4)

**Status:** ✅ CLEAR - Well Specified

**Strengths:**

- Column-to-index matching rules are explicit (5-step process)
- Override semantics for scenario-level data are clearly defined
- Name collision rules across data blocks are comprehensive

**Minor Issue:**

- **Rule 29 reference consistency:** The spec mentions "scenario `data` bindings that do not match any model param or top-level data param MUST fail validation (see [§10](#10-validation-requirements), rule 29)"
- Rule 29 text says: "Scenario `data` binding names MUST match model `param` declarations"
- **Inconsistency:** Rule 29 doesn't mention the "top-level data param" fallback that section 7.2 describes

### 7.3 Report Semantics (Section 7.3)

**Status:** ⚠️ PARTIALLY CLEAR - Output Specification Gaps

**Gaps Identified:**

1. **Solver status reporting format (GAP)**
   "Implementations MUST report at minimum the solver status (optimal, infeasible, unbounded, time limit)"
   - No format specified (stdout? File? Structured?)
   - No schema for the status report

2. **Expression report free variable detection (CLARITY ISSUE)**
   "If the reported expression has free variables... the output is indexed by those free variables"
   - How are free variables determined when expressions reference other expressions?
   - No algorithm specified for computing the free variable set

3. **Dual report value column naming (INCONSISTENCY)**
   "CSV MUST contain... a value column: `dual` for dual reports, or the expression name for scalar reports"
   - This creates inconsistent column naming - some reports have named value columns, others use generic `dual`
   - **Recommendation:** Standardize on `value` for all report types, add `report_type` column if needed

---

## Section 8: KDL 2.0 Type Annotations

**Status:** ✅ CLEAR - Minimal but Sufficient

**Assessment:**

- Section is appropriately brief (30 lines)
- Type annotations are correctly marked as optional
- Reference to validation rules 21-22 is correct
- Examples cover node annotations, typed literals, and metadata

**No issues found.**

---

## Section 9: Filter Predicate Semantics

**Status:** ⚠️ PARTIALLY CLEAR - Semantic Gaps

**Findings:**

1. **Bare identifier RHS resolution (CLARITY ISSUE)**
   "A bare identifier (e.g., `thermal`) is treated as a categorical string value matched against column contents. Bare identifiers on the RHS are never interpreted as column references"
   - **Question:** How are bare identifiers distinguished from numeric literals?
   - **Example:** In `filter { status == active }`, is `active` a string or an identifier?
   - **Gap:** No specification for how bare identifiers are tokenized vs. numeric literals

2. **String literal quoting requirements (GAP)**
   "a quoted string (e.g., `"thermal"`) is a string value"
   - When MUST strings be quoted vs. used as bare identifiers?
   - Are there escaping rules for strings containing special characters?
   - **Example:** What if a categorical value contains spaces or special characters?

3. **Filter predicate on aliased columns (UNDERSPECIFIED)**
   - Can filter predicates reference aliased set names?
   - If a column is mapped with `map new_name from=old_name`, which name is used in filters?
   - **Recommendation:** Add explicit rule that filters use the original CSV column names (before map resolution)

4. **Short-circuit evaluation (GAP)**
   - No specification of `and`/`or` evaluation order
   - For predicates like `filter { x > 0 and y/x < 1 }`, is short-circuiting guaranteed?
   - **Recommendation:** Specify left-to-right short-circuit evaluation

---

## Section 10: Validation Requirements

**Status:** ⚠️ ISSUES IDENTIFIED - Rules Need Clarification

### 10.1 Rule Numbering Inconsistency

**CRITICAL STRUCTURAL ISSUE:**
Rules 49, 50, and 63 are **missing or misnumbered**:

- Rules 49-50: Marked as "(removed)" with references to rules 60, 68
- Rule 63: Present in the detailed text but **missing from the quick-reference table**

**Evidence:**

- Table shows rule 62 (Binary bounds) followed by rule 64 (Param namespace)
- Detailed text has rule 63 (Param resolution) between them
- This is a documentation bug that needs fixing

### 10.2 Rule Clarity Issues

| Rule | Issue                                                                                                                                             | Severity |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| 20   | "SHOULD" detect contradictory predicates - weak requirement                                                                                       | Minor    |
| 38   | Nonlinear diagnostic is "SHOULD" not "MUST" - inconsistent with other solver compatibility rules                                                  | Minor    |
| 67   | Empty-domain aggregation produces "solve-time error" - contradicts section 10.1 which says validation errors SHOULD be collected before execution | Moderate |

### 10.3 Overlapping or Conflicting Rules

**Rule 39, 69 overlap:**

- Rule 39: Auto-generated slack names must not collide with controls
- Rule 69: `<constraint>_slack_lo`/`_slack_hi` names must not collide with controls
- **Issue:** Rule 69 is redundant if rule 39 is interpreted broadly enough
- **Recommendation:** Consolidate or clarify the distinction

**Rule 60 and 68 relationship:**

- Rule 60: Literal and formula bounds on same direction conflict
- Rule 68: `value=` with `lower`/`upper` MUST fail
- **Clarity Issue:** Rule 68 is a specific case of the general rule 60, but this relationship isn't stated

### 10.4 Validation Rule Gaps

**Missing Validation Rules Identified:**

1. **Scenario-level data CSV existence (GAP)**
   - No rule requires the CSV file referenced by scenario `data from="path"` to exist at validation time
   - This is a runtime error rather than validation error
   - **Recommendation:** Add rule requiring path existence validation

2. **Report name uniqueness within scenario (GAP)**
   - Can a scenario have multiple `report <name>` with the same name?
   - No rule prohibits duplicate report declarations
   - **Recommendation:** Add rule for report name uniqueness per scenario

3. **Temporal offset boundary condition (UNDERSPECIFIED)**
   - Rule 34: Temporal offsets without boundary `if` guard MUST fail
   - **Gap:** What constitutes a "boundary `if` guard"?
   - Is `if { t > first(time) }` sufficient? What about `if { t != first(time) }`?
   - **Recommendation:** Define the pattern that satisfies temporal boundary validation

4. **Filter predicate empty parentheses (GAP)**
   - What is the behavior of `filter { }` (empty predicate)?
   - No validation rule addresses this
   - **Recommendation:** Add rule 75: Empty filter predicates MUST fail validation

### 10.5 Error Reporting Strategy (Section 10.1)

**Status:** ✅ WELL SPECIFIED

**Strengths:**

- Clear distinction between parse errors (MAY abort) and validation errors (SHOULD collect)
- Source location requirements specified
- Severity categorization (error vs warning) defined

**Minor Gap:**

- No specification for maximum number of errors to report
- No guidance on error ordering (source order vs severity)

---

## Summary of Findings

### Critical Issues (Require Immediate Fix)

1. **Rule numbering inconsistency** - Rules 49-50 marked removed, rule 63 missing from table
2. **Multi-scenario execution non-determinism** - No ordering mechanism specified
3. **Missing validation rules** - CSV existence, report uniqueness, empty filters

### Moderate Issues (Should Be Addressed)

1. **Filter predicate string/identifier ambiguity** - Quoting requirements unclear
2. **Rule 67 contradiction** - Solve-time error vs validation-time error
3. **Report output column naming inconsistency** - `dual` vs expression name as column header

### Minor Issues (Nice to Have)

1. Rule 39/69 overlap
2. Rule 60/68 relationship clarification
3. Short-circuit evaluation specification

---

## Recommendations

1. **Fix rule numbering** - Add rule 63 to the quick-reference table, clarify 49-50 status
2. **Add scenario ordering** - Either add explicit ordering syntax or declare cross-scenario dependencies unsupported
3. **Complete filter predicate spec** - Define string quoting, aliased column references, empty filter handling
4. **Standardize report output** - Use consistent `value` column naming
5. **Add missing validation rules** - CSV existence, report uniqueness, temporal boundary patterns

---

## Validation Checklist

- [x] Section 7 execution semantics reviewed
- [x] Section 7 data binding rules reviewed
- [x] Section 8 type annotations reviewed
- [x] Section 9 filter predicates reviewed
- [x] Section 10 validation rules (1-74) reviewed
- [x] Rule numbering consistency checked
- [x] Cross-references validated
- [x] Error reporting strategy assessed

**Analysis Complete** - 3 critical, 5 moderate, 3 minor issues identified.
