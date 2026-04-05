# Arco Codebase UX Review Report

**Reviewer:** Code Review Agent
**Date:** April 2, 2026
**Scope:** API ergonomics, error handling, documentation, CLI experience, Python bindings

---

## Executive Summary

Arco is a well-architected optimization library with strong foundations in Rust and Python bindings via PyO3. The codebase demonstrates good separation of concerns, comprehensive error handling, and thoughtful API design. However, several UX improvements could significantly enhance the developer experience.

**Overall Grade:** B+ (Good with room for improvement)

---

## 1. API Ergonomics

### ✅ Strengths

1. **Intuitive Operator Overloading**
   - Natural Python syntax: `x + y >= 5.0`, `3.0 * x + 2.0 * y`
   - Good use of `__add__`, `__mul__`, `__ge__`, etc.
   - Seamless integration between `Variable`, `Expr`, and scalars

2. **Flexible Variable Creation**
   - `add_variable()` for scalars
   - `add_variables()` for arrays with IndexSets
   - Support for per-element bounds via numpy arrays

3. **Solver Configuration Pattern**
   - `solver.copy(update={...})` is elegant for derived configs
   - Direct kwargs to `solve()` for one-off settings
   - Sensible defaults with clear override paths

### ⚠️ Issues & Recommendations

| Issue | Severity | Recommendation |
|-------|----------|----------------|
| `solution.get_primal(index=x)` API is confusing | Medium | `solution.get_value(x)` or `solution[x]` would be more intuitive |
| No built-in pretty-printing for model overview | Low | Add `model.summary()` or `print(model)` showing vars/constraints/obj |
| `Bounds(lower=..., upper=...)` is verbose | Low | Provide shortcuts: `arco.nonnegative()`, `arco.binary()`, `arco.range(0, 10)` |
| No method to easily inspect constraint activity | Medium | Add `solution.get_constraint_activity(constraint)` |

### Code Example - Current vs Proposed

```python
# Current - somewhat clunky
value = solution.get_primal(index=x)  # Why 'index' for a Variable?

# Proposed - more intuitive
value = solution.get_value(x)         # or
value = solution[x]                   # __getitem__ support
```

---

## 2. Error Messages & Handling

### ✅ Strengths

1. **Excellent Exception Hierarchy**
   - 40+ specific exception types in `errors.rs`
   - All inherit from `ArcoError` base class
   - Both module-level and class-level access: `arco.ModelEmptyError` and `arco.ArcoError.MODEL_EMPTY`

2. **Clear Error Messages**
   ```python
   "Binary variables must use bounds=[0,1]"
   "lower bound exceeds upper bound"
   "Model has no variables"
   ```

3. **Proper Separation of Concerns**
   - Model-building errors raise exceptions immediately
   - Solver outcomes (infeasible, unbounded) returned as status codes

### ⚠️ Issues & Recommendations

| Issue | Severity | Recommendation |
|-------|----------|----------------|
| No error context (variable/constraint names) | Medium | Include context: `BoundsInvalidError: lower=10 > upper=0 for variable 'x'` |
| `SolverInternalError` is too generic | Medium | Distinguish between "solver crashed" vs "numerical issues" vs "licensing" |
| No suggestions in error messages | Low | Add "Did you mean..." hints for common mistakes |

### Example Improved Error

```python
# Current
arco.BoundsInvalidError: Bounds are invalid: lower > upper

# Improved
arco.BoundsInvalidError: Variable 'production_rate': lower bound (100) exceeds upper bound (50).
Did you mean to swap them or use Bounds(lower=50, upper=100)?
```

---

## 3. Documentation Gaps

### ✅ Strengths

1. **Comprehensive Tutorial Structure** (Diátaxis pattern)
   - Tutorials: your-first-model, integer-programming, indexed-models
   - How-to guides: error handling, solver config, numpy integration
   - Reference: KDL syntax summary

2. **Good Code Examples**
   - Doctest-validated examples throughout
   - API comparison table (Arco vs Pyomo)

### ⚠️ Documentation Gaps

| Gap | Severity | Notes |
|-----|----------|-------|
| No API reference documentation (Python docstrings) | **High** | Critical gap - users can't explore API in IDE |
| Missing troubleshooting guide | Medium | Common issues and solutions |
| No performance best practices | Medium | Memory optimization tips, sparse patterns |
| Block composition tutorial incomplete | Medium | DAG orchestration is complex, needs more examples |
| Migration guide from Pyomo/JuMP missing | Low | Would help adoption |
| No CLI cookbook | Low | Common CLI workflows and one-liners |

### Critical: Missing Python Docstrings

**This is the most significant documentation gap.** The Python bindings lack docstrings, making IDE exploration impossible:

```python
# Current - no docstring
>>> help(arco.Model.add_variable)
Help on method add_variable in module arco:

add_variable(...)  # No information!

# Needed - comprehensive docstring
>>> help(arco.Model.add_variable)
add_variable(bounds, *, is_integer=False, is_binary=False, name=None)
    Add a decision variable to the model.

    Parameters
    ----------
    bounds : Bounds
        Lower and upper bounds for the variable.
    is_integer : bool, optional
        Whether the variable is restricted to integer values.
    ...
```

---

## 4. CLI Experience

### ✅ Strengths

1. **Clear Command Structure**
   ```
   arco run       - Compile and solve
   arco validate  - Check KDL without solving
   arco inspect   - Examine model structure
   arco export    - LP/MPS export
   arco debug     - IPython shell
   ```

2. **Good Verbosity Control**
   - `-v`, `-vv` flags with tracing integration

### ⚠️ Issues & Recommendations

| Issue | Severity | Recommendation |
|-------|----------|----------------|
| No `--version` flag | Medium | `arco --version` should show version |
| `arco solver set` lacks confirmation | Low | Show "Solver backend changed: highs → xpress" |
| No shell completion scripts | Low | Generate bash/zsh/fish completions |
| `arco run` output format not configurable | Medium | `--format` flag for table/json/csv output |
| Missing `arco init` command | Low | Scaffold a new KDL file |

### CLI Enhancement Suggestions

```bash
# Version flag
$ arco --version
arco-cli 0.2.8 (arco-core 0.2.8, highs 1.7.0)

# Better output formatting
$ arco run model.kdl --format table
Variable    Value    Reduced Cost
x           5.0      0.0
y          10.0      0.0

# Init command
$ arco init --template simple my_model.kdl
Created my_model.kdl with boilerplate
```

---

## 5. Python Bindings Design

### ✅ Strengths

1. **Clean Module Structure**
   - `arco.Model`, `arco.Bounds`, `arco.Variable`
   - Logical organization matching Rust crates

2. **NumPy Integration**
   - `__array_ufunc__` and `__array_function__` protocols
   - Element-wise operations with VariableArrays

3. **Type Safety**
   - Proper use of PyO3's type system
   - `ExprLike` for flexible argument handling

### ⚠️ Issues & Recommendations

| Issue | Severity | Recommendation |
|-------|----------|----------------|
| Inconsistent naming: `get_primal` vs `get_value` | Medium | Standardize on `get_value` |
| `SolveResult` has legacy `primal_values` list | Low | Deprecate in favor of `get_value()` |
| No context manager for models | Low | `with arco.Model() as m:` auto-cleanup |
| Missing `__len__`, `__iter__` on VariableArray | Medium | Enable `len(vars)` and `for v in vars` |
| `block` decorator lacks documentation | Medium | Explain when/why to use blocks |

### API Consistency Issue

```python
# Inconsistent naming across API
solution.get_primal(index=x)      # uses 'index'
solution.get_value(variable=x)  # uses 'variable'
solution.get_dual(constraint=c)   # uses 'constraint'

# Should standardize parameter names
solution.get_value(x)             # unified
solution.get_dual(c)              # unified
```

---

## 6. Common Gotchas

### Observed Patterns That Could Trip Up Users

1. **Bounds Construction**
   ```python
   # Easy to get wrong
   arco.Bounds(lower=0, upper=1)  # ints work but...
   arco.Bounds(lower=0.0, upper=float('inf'))  # need explicit float for inf
   ```
   **Recommendation:** Accept `int` and convert, or provide clearer error.

2. **Binary Variable Bounds**
   ```python
   # This raises an error
   x = model.add_variable(bounds=arco.Bounds(lower=0, upper=1), is_binary=True)
   # Error: Binary variables must use bounds=[0,1]

   # But this is redundant
   x = model.add_variable(is_binary=True)  # bounds should be optional for binary
   ```
   **Recommendation:** `is_binary=True` should imply bounds.

3. **Solver Log Verbosity**
   ```python
   solution = model.solve(log_to_console=False)  # needed every time
   ```
   **Recommendation:** Make `False` the default, or respect env var.

4. **IndexSet vs list confusion**
   ```python
   # Users might try:
   vars = model.add_variables(['a', 'b', 'c'])  # Won't work

   # Must use:
   set = arco.IndexSet(name="items", members=['a', 'b', 'c'])
   vars = model.add_variables(set)
   ```
   **Recommendation:** Accept sequences and auto-convert to IndexSet.

---

## 7. Developer Experience Improvements

### Priority Recommendations

#### High Priority
1. **Add comprehensive Python docstrings** - Critical for IDE integration
2. **Standardize solution accessor API** - `get_value()` consistently
3. **Add API reference documentation** - Auto-generated from docstrings

#### Medium Priority
4. **Add error context** - Variable/constraint names in errors
5. **Add `model.summary()`** - Pretty-print model overview
6. **Add CLI `--version`** and `--format` flags
7. **Implement `__len__`, `__iter__` on arrays**

#### Low Priority
8. **Add convenience bounds constructors** - `arco.nonnegative()`, `arco.binary()`
9. **Shell completion scripts**
10. **Troubleshooting guide in docs**

---

## 8. Positive Findings

Notable design decisions that enhance UX:

1. **Solver backend abstraction** - Clean `HiGHS`/`Xpress`/`Solver` interface
2. **Block composition system** - Powerful DAG orchestration
3. **Memory diagnostics** - Built-in for the library's core value proposition
4. **KDL format** - Human-readable domain-specific language
5. **Exception hierarchy** - Well-structured with 40+ specific errors
6. **NumPy integration** - Seamless array operations
7. **Solver outcome design** - Exceptions for build errors, status codes for solve outcomes

---

## Appendix: File-Level Observations

### Core Rust (`crates/arco-core/src/`)
- Clean module organization
- Good use of SmallVec for memory efficiency
- Packed bit flags for integer/inactive state

### Python Bindings (`bindings/python/src/`)
- Well-structured PyO3 code
- Good error conversion in `errors.rs`
- `ExprLike` pattern is elegant for argument handling

### CLI (`crates/arco-cli/src/`)
- Clap usage is idiomatic
- Good separation: driver, execution, config
- Debug shell feature is excellent

### Documentation (`docs/`)
- Diátaxis structure followed well
- KDL syntax summary is comprehensive
- Missing API reference is the big gap

---

*End of Review Report*
