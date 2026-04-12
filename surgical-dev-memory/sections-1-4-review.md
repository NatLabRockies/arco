# Surgical-Dev Review: Arco Spec Sections 1-4 (Lines 1-400)

**Review Date:** 2026-04-12
**Focus:** Specification completeness, ambiguities, RFC 2119 keyword usage accuracy, technical correctness

## Summary

| Category              | Count | Status          |
| --------------------- | ----- | --------------- |
| Technical Correctness | 6     | Issues found    |
| RFC 2119 Usage        | 3     | Issues found    |
| Ambiguities           | 7     | Issues found    |
| Completeness          | 6     | Gaps identified |

---

## Detailed Line-Specific Findings

### Technical Correctness Issues

| Line    | Finding                                                                                                                                                                                                                                |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 35-39   | **KDL Conformance Contradiction:** "Arco KDL files MUST conform to KDL 2.0" conflicts with §1.1's statement that Arco is a **superset** of KDL 2.0. Document simultaneously claims files are both valid KDL 2.0 and not valid KDL 2.0. |
| 48-50   | **Comment Support Overstatement:** "KDL comments … are fully supported" overstates - `/-` slashdash is KDL structural, not clearly valid in algebra blocks. Needs distinction between KDL-context and algebra-context comments.        |
| 82-84   | **Incompatible Statements:** Claims children of `set` use normal KDL rules, but §4 defines `set` bodies as bare member lists like `{ 1; 2; 3 }` which are NOT standard KDL child nodes.                                                |
| 160-162 | **Unreachable Resolution Rule:** Canonical-name-preferred resolution is unreachable because prior rule forbids alias/name collisions. No valid case where both could match.                                                            |
| 275-276 | **Incorrect Positional Argument Description:** "Inline literal value as its first positional argument" is wrong - in `param voll 9000`, name is first, value is second.                                                                |
| 304-309 | **KDL Syntax Violation:** `set <name> { <member1>; <member2>; ... }` is NOT standard KDL child-block syntax - contradicts line 82-84 claim.                                                                                            |

### RFC 2119 Keyword Usage Issues

| Line    | Finding                                                                                                                                                                                    |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 37-39   | **Mixed Normative/Descriptive:** Conformance list mixes normative and descriptive items. "File extension: `.kdl`" is not written as requirement but nested under "MUST conform."           |
| 232-238 | **Weak Normative Wording:** "MAY contain these top-level declarations" is weak given line 52's rejection of unknown nodes. This defines the complete allowed set, not optional permission. |
| 353     | **Inappropriate SHOULD:** BOM handling as "SHOULD accept" permits incompatible implementations without stating valid reasons to reject BOM-bearing files.                                  |

### Ambiguities

| Line    | Finding                                                                                                                                                                                             |
| ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 52-55   | **Unknown Node Definition:** "unknown child node types MUST also fail validation" - "unknown" is undefined. Not clear if "not in spec anywhere" or "not allowed under this parent."                 |
| 97-98   | **Imprecise Terminology:** "Positional arguments carry names and values" is imprecise - in KDL, positional arguments are values. Blurs host-syntax with Arco semantics.                             |
| 143-146 | **Forward Reference:** Alias uniqueness rule references "model-level" set declarations before they are introduced in sections 1-4.                                                                  |
| 176     | **Inaccurate Syntax Description:** "Body uses algebra-block syntax" inaccurate for reduction guards `if cond` (no block). Conflates `if { ... }` constraint filters with inline algebra `if`.       |
| 308-310 | **Undefined Token Class:** "Members are KDL arguments (strings or numbers)" doesn't define whether identifiers like `g1` are strings, identifiers, or another token class.                          |
| 334-337 | **Namespace Confusion:** Calling `data` a "namespace" conflicts with "globally visible" contents. Unclear what the namespace actually namespaces.                                                   |
| 376-378 | **Underspecified Mapping:** "Unmapped columns remain available" - unclear if mapped header is still available under original name, or how conflicts resolved if logical name equals another header. |
| 383-386 | **Map Resolution Ambiguity:** Column resolution "after `map` resolution" depends on unresolved semantics about original vs mapped name availability.                                                |

### Completeness Gaps

| Line    | Finding                                                                                                                                                                                                                                                                              |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 123-136 | **Name Form Interaction:** Allows `name=` for declarations but per-declaration syntaxes only show positional-name forms. No definition of whether both forms may appear together, invalidity, or conflict handling.                                                                  |
| 179     | **Missing Param Form:** `param` defined as "data-backed or model-declared" but §3.1 allows top-level inline scalar `param` - terminology table omits one allowed form.                                                                                                               |
| 189     | **Undefined Child-Node Form:** `reduce` defined as allowing `reduce sum` child-node form but sections 1-4 don't define this syntax anywhere.                                                                                                                                         |
| 287-292 | **Missing Model-Local Syntax:** Inline scalars permitted "inside `model` blocks" but no model-local `param` syntax example or rules in sections 1-4.                                                                                                                                 |
| 324-328 | **Incomplete Namespace Rules:** Only forbids top-level/data-level collisions. Doesn't address: two top-level sets sharing name, two data blocks with same set name, or interaction with other global declarations.                                                                   |
| 348-356 | **Undefined Column Typing:** Rules for "numeric column" vs "string/categorical column" depend on undefined column typing model. No definition of type inference.                                                                                                                     |
| 390-400 | **Incomplete Set Semantics:** Syntax for `set` inside `data` introduced but semantics incomplete at section boundary. Missing: `in <parent_set>` requirement, `filter` without `in`, subset membership computation, ordering preservation, validation when parent set missing/empty. |

---

## Highest-Priority Structural Issues

1. **KDL Conformance Contradiction (Lines 35-39, 82-84, 304-309)**
   - The specification simultaneously claims Arco files conform to KDL 2.0 while defining non-KDL constructs
   - Top-level `set` member syntax is NOT valid KDL
   - **Recommendation:** Define Arco as "KDL-based with explicit non-KDL subgrammars" rather than claiming full KDL conformance

2. **Alias/Name Resolution Unreachable Code (Lines 143-146, 160-162)**
   - Alias uniqueness + no-collision rules make canonical-name-preference unreachable
   - **Recommendation:** Remove unreachable resolution rule OR relax collision prohibition

3. **Map Semantics Undefined (Lines 376-386)**
   - Critical for `set` and `param` inside `data` blocks
   - **Recommendation:** Define whether logical names alias or replace physical headers

---

## Validation Outcome

- **Review completed on sections 1-4 only**
- **No code or docs changed during review**
- **22 distinct findings across 4 categories**

**Largest Risk:** Unresolved contradiction between KDL conformance claims and non-KDL algebra/set constructs.
