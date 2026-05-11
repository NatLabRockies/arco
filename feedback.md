# Desired ReEDS benchmark syntax for Arco implementation

This is the target syntax for the next Arco Python sparse buildout iteration. Keep the public surface small: `IndexSet`, `arco.param`, `add_variables(..., active=...)`, axis-aware array algebra, `expr @ IndexSet` reductions, and NumPy functions with `axis=IndexSet`.

## Core idea

Arco arrays already know their axes from `add_variables(I, R, H, T, ...)`. Raw NumPy arrays do not. Add one low-level data wrapper so numeric parameters also carry axis labels:

```python
cf = arco.param(data.cf, I, R, H)
valcap = arco.param(data.valcap, I, R, T)
pvf = arco.param(data.pvf, T)
```

`arco.param(values, *axes)` should only mean: "this NumPy-like data has these axes." It is not a modeling domain, relation, or orchestration primitive.

## Desired full ReEDS-style formulation sketch

```python
import numpy as np
import arco

m = arco.Model()

I = arco.IndexSet("i", data.techs)
R = arco.IndexSet("r", data.regions)
H = arco.IndexSet("h", data.hours)
T = arco.IndexSet("t", data.years)
H_ramp = H[:-1]

R_from = R.alias("from")
R_to = R.alias("to")

valcap = arco.param(data.valcap, I, R, T)
is_vre = arco.param(data.is_vre, I)
is_storage = arco.param(data.is_storage, I)
is_dispatch = ~is_vre & ~is_storage
storage_active = valcap & is_storage
dispatch_active = valcap & is_dispatch

cf = arco.param(data.cf, I, R, H)
cap_init = arco.param(data.cap_init, I, R)
load = arco.param(data.load, R, H, T)
peak_load = arco.param(data.load.max(axis=1), R, T)
minloadfrac = arco.param(data.minloadfrac, I)
min_cf = arco.param(data.min_cf, I)
emit_rate = arco.param(data.emit_rate, I)
emit_cap = arco.param(data.emit_cap, T)
hours_weight = arco.param(data.hours_weight, H)
pvf = arco.param(data.pvf, T)
cost_inv = arco.param(data.cost_inv, I)
cost_op = arco.param(data.cost_op, I)
startcost = arco.param(data.startcost, I)

route_active = arco.param(data.route_active, R_from, R_to)
transcap = arco.param(data.transcap_matrix, R_from, R_to)

cap = m.add_variables(I, R, T, bounds=arco.NonNegativeFloat, active=valcap, name="CAP")
inv = m.add_variables(I, R, T, bounds=arco.NonNegativeFloat, active=valcap, name="INV")
gen = m.add_variables(I, R, H, T, bounds=arco.NonNegativeFloat, active=valcap, name="GEN")

flow = m.add_variables(
    R_from, R_to, H, T,
    bounds=arco.Bounds(0, transcap),
    active=route_active,
    name="FLOW",
)

rampup = m.add_variables(
    I, R, H_ramp, T,
    bounds=arco.NonNegativeFloat,
    active=dispatch_active,
    name="RAMPUP",
)

charge = m.add_variables(
    I, R, H, T,
    bounds=arco.NonNegativeFloat,
    active=storage_active,
    name="CHARGE",
)
soc = m.add_variables(
    I, R, H, T,
    bounds=arco.NonNegativeFloat,
    active=storage_active,
    name="SOC",
)

m.add_constraints(
    cap == cap_init + np.cumsum(inv, axis=T),
    name="eq_cap_accum",
)

m.add_constraints(
    gen <= cf * cap,
    name="eq_cap_limit",
)

m.add_constraints(
    gen >= minloadfrac * cap,
    active=valcap & (minloadfrac > 0) & ~is_storage,
    name="eq_mingen",
)

imports = ((1.0 - data.tranloss) * flow) @ R_from
exports = flow @ R_to
net_flow = imports - exports

m.add_constraints(
    (gen @ I) + net_flow - (charge @ I) == load,
    name="eq_supply_demand_balance",
)

m.add_constraints(
    (cap @ I) >= (1.0 + data.prm) * peak_load,
    name="eq_reserve_margin",
)

m.add_constraints(
    (emit_rate * hours_weight * gen) @ (I, R, H) <= emit_cap,
    name="eq_emit_cap",
)

m.add_constraints(
    rampup >= np.diff(gen, axis=H),
    active=dispatch_active,
    name="eq_ramping",
)

m.add_constraints(
    (hours_weight * gen) @ H >= min_cf * float(data.hours_weight.sum()) * cap,
    active=valcap & (min_cf > 0),
    name="eq_min_cf",
)

m.add_constraints(
    soc <= data.duration_h * cap,
    active=storage_active,
    name="eq_soc_cap",
)

m.add_constraints(
    charge <= cap,
    active=storage_active,
    name="eq_charge_cap",
)

m.add_constraints(
    np.roll(soc, -1, axis=H) == soc + data.charge_eff * charge - gen,
    active=storage_active,
    name="eq_soc",
)

m.minimize(
    (pvf * cost_inv * inv).sum()
    + (pvf * cost_op * hours_weight * gen).sum()
    + (pvf * startcost * rampup).sum()
)
```

## Implementation guidelines by feature

### 1. Standard Python operators

Support the normal Python operator surface on `Variable`, `Expr`, `VariableArray`, `ExprArray`, and `ParamArray` whenever axes are compatible.

Arithmetic operators:

```python
supply = (gen @ I) + net_flow - (charge @ I)
cap_margin = cf * cap - gen
scaled_generation = (hours_weight * gen) / 1_000.0
negated_flow = -net_flow
```

Comparison operators create constraint expressions or constraint arrays:

```python
m.add_constraints(gen <= cf * cap, name="eq_cap_limit")
m.add_constraints(gen >= minloadfrac * cap, name="eq_mingen")
m.add_constraints(supply == load, name="eq_supply_demand_balance")
```

Mask operators use bitwise syntax, not Python boolean keywords:

```python
dispatch_active = valcap & ~is_vre & ~is_storage
mingen_active = valcap & (minloadfrac > 0)
flexible_active = dispatch_active | storage_active
```

Reduction operators:

```python
gen_by_region_hour_year = gen @ I
annual_emissions = (emit_rate * hours_weight * gen) @ (I, R, H)
gen_by_region_hour_year_alt = gen >> I
```

Canonical reduction style is `expr @ IndexSet` or `expr @ (IndexSet, ...)`. Keep `expr >> IndexSet` as a supported compatibility shorthand for single-axis reductions if it remains in the API.

Indexing and slicing operators should work for declared axes and subsets:

```python
H_ramp = H[:-1]
rampup = m.add_variables(I, R, H_ramp, T, bounds=arco.NonNegativeFloat, active=dispatch_active)
```

### 2. Labeled numeric parameters with `arco.param`

Use this instead of forcing users to write `None` axes everywhere.

```python
cf = arco.param(data.cf, I, R, H)
cap_init = arco.param(data.cap_init, I, R)
pvf = arco.param(data.pvf, T)
```

Required behavior:

- Validate `values.ndim == len(axes)`.
- Validate each dimension length matches the corresponding `IndexSet` length.
- Support numeric operations with `VariableArray`, `ExprArray`, and other labeled params.
- Align by `IndexSet`, not by raw dimension position.

### 3. Structural sparsity with `active=`

Use `active=` for real sparse backing. Inactive entries are not variables and should not produce matrix columns.

```python
valcap = arco.param(data.valcap, I, R, T)

cap = m.add_variables(I, R, T, bounds=arco.NonNegativeFloat, active=valcap, name="CAP")
gen = m.add_variables(I, R, H, T, bounds=arco.NonNegativeFloat, active=valcap, name="GEN")
```

`active=valcap` should broadcast over missing axes, so `valcap(I, R, T)` is valid for `gen(I, R, H, T)`.

Implementation must not build the dense `(I, R, H, T)` cartesian product and then filter it. Follow the KDL sparse lowering approach: walk active mask coordinates and instantiate only active variables.

Prefer this over zeroing inactive variables with constraints.

### 4. Constraint masks with `active=`

Use the same mask idea for sparse constraint families.

```python
m.add_constraints(
    gen >= minloadfrac * cap,
    active=valcap & (minloadfrac > 0) & ~is_storage,
    name="eq_mingen",
)
```

Inactive entries should not create constraint rows. The implementation must not allocate a dense candidate `ConstraintArray` for the full cartesian product before applying the mask.

### 5. Single-axis reduction with `expr @ I`

Use `@ IndexSet` to sum over one named axis.

```python
m.add_constraints(
    (gen @ I) + net_flow - (charge @ I) == load,
    name="eq_supply_demand_balance",
)

gen_by_region_hour_year_alt = gen.sum(over=I)
gen_by_region_hour_year_np = np.sum(gen, axis=I)
```

Equivalent explicit syntax may remain available as `gen.sum(over=I)` and `np.sum(gen, axis=I)`, but `gen @ I` should be canonical in examples.

### 6. Multi-axis reduction with `expr @ (I, H)`

Use tuple reduction for annual or system-wide totals.

```python
m.add_constraints(
    (emit_rate * hours_weight * gen) @ (I, R, H) <= emit_cap,
    name="eq_emit_cap",
)

annual_emissions_alt = (emit_rate * hours_weight * gen).sum(over=(I, R, H))
annual_emissions_np = np.sum(emit_rate * hours_weight * gen, axis=(I, R, H))
```

This leaves the unreduced `T` axis.

### 7. Total reductions with `.sum()` and `sum(...)`

All of these forms should be valid for total reductions.

```python
investment_cost = (pvf * cost_inv * inv).sum()
investment_cost_builtin = sum(pvf * cost_inv * inv)
investment_cost_np = np.sum(pvf * cost_inv * inv)
investment_cost_einsum = np.einsum("t,i,irt->", data.pvf, data.cost_inv, inv)
```

Use `.sum()` as the canonical style because it is explicit, supports Arco fast paths, and can avoid materializing dense cartesian products. Python built-in `sum(...)` and `np.sum(...)` should behave as flat total reductions over active elements only when no axis is provided; they are not the syntax for named-axis reductions.

### 8. Axis-aware broadcasting

Labeled Arco arrays should broadcast by axis identity.

```python
m.add_constraints(
    gen <= cf * cap,
    name="eq_cap_limit",
)
```

Axis meaning:

```text
gen axes = (I, R, H, T)
cf axes  = (I, R, H)     # broadcasts over T
cap axes = (I, R, T)     # broadcasts over H
```

Do not infer semantics from equal dimension sizes alone. Axis labels must drive alignment.

### 9. NumPy functions with `axis=IndexSet`

Support named axes for common time operations.

```python
m.add_constraints(
    cap == cap_init + np.cumsum(inv, axis=T),
    name="eq_cap_accum",
)

m.add_constraints(
    rampup >= np.diff(gen, axis=H),
    active=dispatch_active,
    name="eq_ramping",
)

m.add_constraints(
    np.roll(soc, -1, axis=H) == soc + data.charge_eff * charge - gen,
    active=storage_active,
    name="eq_soc",
)

m.add_constraints(
    np.concatenate((soc[:, :, 1:, :], soc[:, :, :1, :]), axis=H)
    == soc + data.charge_eff * charge - gen,
    active=storage_active,
    name="eq_soc_slice_equivalent",
)
```

Slicing equivalents should preserve axis metadata and must not materialize inactive dense cartesian products.

Minimum required NumPy support:

- `np.sum(..., axis=IndexSet)`
- `np.cumsum(..., axis=IndexSet)`
- `np.diff(..., axis=IndexSet)`
- `np.roll(..., axis=IndexSet)`
- `np.concatenate(..., axis=IndexSet)` for slice-based roll equivalents
- `np.einsum(...)` when one argument is an Arco array or labeled param

### 10. Index aliases for repeated dimensions

Use aliases when the same conceptual set appears twice. This keeps directed-pair modeling generic, not power-specific.

```python
R_from = R.alias("from")
R_to = R.alias("to")

route_active = arco.param(data.route_active, R_from, R_to)
transcap = arco.param(data.transcap_matrix, R_from, R_to)

flow = m.add_variables(
    R_from, R_to, H, T,
    bounds=arco.Bounds(0, transcap),
    active=route_active,
    name="FLOW",
)

imports = ((1.0 - data.tranloss) * flow) @ R_from
exports = flow @ R_to
net_flow = imports - exports
```

No `Transmission`, `Network`, or power-specific helper is needed.

### 11. `np.einsum` as the raw-NumPy escape hatch

Keep `np.einsum` as an alternative when users do not want to wrap raw NumPy data with `arco.param`, or when an expression is clearer in index notation.

```python
m.add_constraints(
    gen <= np.einsum("irh,irt->irht", data.cf, cap),
    name="eq_cap_limit",
)

m.add_constraints(
    np.einsum("i,h,irht->t", data.emit_rate, data.hours_weight, gen)
    <= data.emit_cap,
    name="eq_emit_cap",
)

m.minimize(
    np.einsum("t,i,irt->", data.pvf, data.cost_inv, inv)
    + np.einsum("t,i,h,irht->", data.pvf, data.cost_op, data.hours_weight, gen)
)
```

Use normal NumPy broadcasting only when raw NumPy axes are already suffix-aligned with the Arco array, for example `data.pvf * inv` when `inv` is indexed `(I, R, T)`.

## Equivalent syntax contracts to validate

The implementation should support the canonical style plus these accepted equivalents. Tests should confirm equivalent forms produce the same shape and coefficients without materializing dense inactive cartesian products.

Scope: validate idiomatic operator/NumPy equivalents. Do not optimize for arbitrary user-written Python loops as a first-class style, although scalar loops can still work for small debugging examples.

### Total objective terms

All of these should be valid flat total reductions over active elements:

```python
investment_a = (pvf * cost_inv * inv).sum()
investment_b = sum(pvf * cost_inv * inv)
investment_c = np.sum(pvf * cost_inv * inv)
investment_d = np.sum(pvf * cost_inv * inv, axis=(I, R, T))
investment_e = (pvf * cost_inv * inv) @ (I, R, T)
investment_f = (((pvf * cost_inv * inv) @ I) @ R) @ T
investment_g = np.einsum("t,i,irt->", data.pvf, data.cost_inv, inv)
```

Canonical example style: `.sum()`.

### Single-axis reductions

All of these should reduce the `I` axis and leave `(R, H, T)`:

```python
gen_by_region_hour_year_a = gen @ I
gen_by_region_hour_year_b = gen.sum(over=I)
gen_by_region_hour_year_c = np.sum(gen, axis=I)
gen_by_region_hour_year_d = np.sum(gen, axis=(I,))
gen_by_region_hour_year_e = gen >> I
```

Canonical example style: `gen @ I`.

### Multi-axis reductions

All of these should reduce `(I, R, H)` and leave `T`:

```python
emissions_a = (emit_rate * hours_weight * gen) @ (I, R, H)
emissions_b = (emit_rate * hours_weight * gen).sum(over=(I, R, H))
emissions_c = np.sum(emit_rate * hours_weight * gen, axis=(I, R, H))
emissions_d = (((emit_rate * hours_weight * gen) @ I) @ R) @ H
emissions_e = (emit_rate * hours_weight * gen).sum(over=I).sum(over=R).sum(over=H)
emissions_f = np.einsum("i,h,irht->t", data.emit_rate, data.hours_weight, gen)
```

Canonical example style: `expr @ (I, R, H)`.

### Axis alignment and broadcasting

All of these should express the same capacity-limit coefficients:

```python
cap_limit_a = cf * cap
cap_limit_b = data.cf[:, :, :, None] * cap[:, :, None, :]
cap_limit_c = np.expand_dims(data.cf, axis=-1) * np.expand_dims(cap, axis=2)
cap_limit_d = np.einsum("irh,irt->irht", data.cf, cap)
```

Canonical example style: labeled params, `cf * cap`. The explicit-`None`, `np.expand_dims`, and `einsum` forms are accepted escape hatches for raw NumPy data.

### Constraint expression forms

All of these should produce equivalent row coefficients and bounds:

```python
cap_limit_a = gen <= cf * cap
cap_limit_b = gen - cf * cap <= 0

m.add_constraints(gen <= cf * cap, name="eq_cap_limit")
m.add_constraints(gen - cf * cap <= 0, name="eq_cap_limit")
m.add_constraints(gen - cf * cap, sense="le", rhs=0, name="eq_cap_limit")

supply = (gen @ I) + net_flow - (charge @ I)
m.add_constraints(supply == load, name="eq_supply_demand_balance")
m.add_constraints(supply - load == 0, name="eq_supply_demand_balance")
m.add_constraints(supply - load, sense="eq", rhs=0, name="eq_supply_demand_balance")
```

Canonical example style: comparison expressions, for readability.

### Time-axis operations

Support both NumPy function style and method style when a method exists:

```python
cap_accum_a = cap_init + np.cumsum(inv, axis=T)
cap_accum_b = cap_init + inv.cumsum(over=T)

ramp_delta_a = np.diff(gen, axis=H)
ramp_delta_b = gen.diff(over=H)
ramp_delta_c = gen[:, :, 1:, :] - gen[:, :, :-1, :]

soc_next_a = np.roll(soc, -1, axis=H)
soc_next_b = soc.roll(shift=-1, over=H)
soc_next_c = np.concatenate((soc[:, :, 1:, :], soc[:, :, :1, :]), axis=H)

m.add_constraints(
    soc[:, :, 1:, :] == soc[:, :, :-1, :] + data.charge_eff * charge[:, :, :-1, :] - gen[:, :, :-1, :],
    active=storage_active[:, :, :-1, :],
    name="eq_soc_forward_slice",
)
m.add_constraints(
    soc[:, :, 0, :] == soc[:, :, -1, :] + data.charge_eff * charge[:, :, -1, :] - gen[:, :, -1, :],
    active=storage_active[:, :, -1, :],
    name="eq_soc_wrap_slice",
)
```

Canonical example style: NumPy functions with `axis=IndexSet`. Slicing forms are valid when users want explicit successor relationships. N-dimensional slicing must preserve axis metadata after dropping or narrowing axes.

### Bounds and upper-limit forms

Prefer variable bounds for simple upper limits, but these forms should be equivalent when both are needed for compatibility tests:

```python
flow_a = m.add_variables(
    R_from, R_to, H, T,
    bounds=arco.Bounds(0, transcap),
    active=route_active,
    name="FLOW",
)

flow_b = m.add_variables(
    R_from, R_to, H, T,
    bounds=arco.NonNegativeFloat,
    active=route_active,
    name="FLOW",
)
m.add_constraints(flow_b <= transcap, active=route_active, name="eq_flow_limit")
```

Canonical example style: variable bounds, because they avoid extra constraint rows.

### Active masks

All of these should be valid sparse declarations when their axes are unambiguous:

```python
cap_a = m.add_variables(I, R, T, bounds=arco.NonNegativeFloat, active=valcap)
cap_b = m.add_variables(I, R, T, bounds=arco.NonNegativeFloat, active=data.valcap)

gen_a = m.add_variables(I, R, H, T, bounds=arco.NonNegativeFloat, active=valcap)
gen_b = m.add_variables(
    I, R, H, T,
    bounds=arco.NonNegativeFloat,
    active=data.valcap[:, :, None, :],
)
gen_c = m.add_variables(
    I, R, H, T,
    bounds=arco.NonNegativeFloat,
    active=np.expand_dims(data.valcap, axis=2),
)
```

Canonical example style: labeled params, `active=valcap`. Raw NumPy masks must either exactly match the target shape or use explicit singleton axes via `None` or `np.expand_dims`.

## Python-developer implementation contract

### Acceptance criteria

- ReEDS formulation code uses labeled params and Arco operators instead of manual dense map matrices.
- `arco.param(values, *axes)` is a thin typed data wrapper, not a modeling domain.
- `add_variables(..., active=mask)` creates only structurally active variables.
- `add_constraints(..., active=mask)` creates only structurally active constraint rows.
- Sparse variables and constraints follow the KDL lowering rule: do not materialize or explode the full cartesian product before filtering; iterate active coordinates directly so memory scales with active rows/columns, not dense domain size.
- `expr @ IndexSet` and `expr @ (IndexSet, ...)` reduce by axis identity.
- Total reductions work with `(expr).sum()`, Python built-in `sum(expr)`, `np.sum(expr)`, and equivalent `np.einsum(...)` forms. Prefer `.sum()` in examples because it is explicit and can use Arco fast paths; alternatives should be valid for coverage and user familiarity.
- Arco array operations align by declared `IndexSet`; raw NumPy data never aligns by guessed dimension size.
- The ReEDS small benchmark solves to the same objective as the current formulation within tolerance.

### Public Python API shape

Keep signatures explicit in `arco.pyi` and Python-facing docs.

```python
class ParamArray:
    @property
    def axes(self) -> tuple[IndexSet, ...]: ...

    @property
    def shape(self) -> tuple[int, ...]: ...

    @property
    def values(self) -> object: ...


def param(
    values: object,
    *axes: IndexSet,
    name: str | None = None,
) -> ParamArray: ...
```

`ParamArray` should support arithmetic, comparisons, boolean masks, and NumPy protocol hooks needed by the examples above.

### Error handling

Fail fast with specific Arco exceptions:

- dimension count mismatch: `ArrayDimensionError`
- axis length mismatch: `ArrayShapeMismatchError`
- unsupported operand or mask dtype: `ArrayTypeError`
- ambiguous duplicate axes without aliases: `ArrayDimensionError`

Do not silently align axes by equal lengths.

### Targeted pytest coverage

Add focused tests for each user-visible behavior:

```python
def test_param_validates_axis_lengths() -> None: ...

def test_standard_python_arithmetic_comparison_mask_and_reduction_operators() -> None: ...

def test_param_broadcasts_by_axis_identity() -> None: ...

def test_axis_alignment_equivalent_forms_match_coefficients() -> None: ...

def test_total_reduction_equivalent_forms_match_coefficients() -> None: ...

def test_single_axis_reduction_equivalent_forms_match_shape_and_coefficients() -> None: ...

def test_multi_axis_reduction_equivalent_forms_match_shape_and_coefficients() -> None: ...

def test_active_mask_broadcasts_and_creates_sparse_variables() -> None: ...

def test_active_mask_equivalent_forms_create_same_sparse_variables() -> None: ...

def test_constraint_active_mask_skips_inactive_rows() -> None: ...

def test_matmul_reduces_single_and_multiple_axes() -> None: ...

def test_numpy_cumsum_diff_roll_concatenate_accept_indexset_axis() -> None: ...

def test_time_axis_numpy_method_and_slicing_forms_match() -> None: ...

def test_alias_axes_keep_directed_pair_dimensions_distinct() -> None: ...

def test_einsum_accepts_arco_array_operand() -> None: ...
```

Use function-based pytest tests. Prefer small synthetic arrays over the full ReEDS fixture for unit coverage, then run the ReEDS small benchmark as an integration check.

### Validation commands

Run targeted Python checks first:

```bash
cd bindings/python
uv run pytest tests/test_axis_param.py
uv run ruff check .
uv run ty check .
```

Run the ReEDS benchmark integration from repo root:

```bash
uv run examples/reeds-benchmark/formulation.py --size small --json
uv run examples/reeds-benchmark/formulation.py --size medium --build-only --json
```

If implementation changes Rust/PyO3 code, also run:

```bash
cargo fmt
cargo test -p arco-python
just ci
```

## Non-goals

- Do not add a public `Domain` DSL for this ReEDS benchmark path.
- Do not add a power-specific transmission/network primitive.
- Do not model sparsity by creating dense variables plus `x == 0` constraints.
- Do not rely on matching dimension sizes to infer axis meaning.
