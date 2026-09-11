"""Consumer-facing typing fixture for the labeled array formulation API."""

from typing import assert_type

import arco


i = arco.IndexSet(name="i", members=["gas", "wind"])
r = arco.IndexSet(name="r", members=["north", "south"])
h = arco.IndexSet(name="h", members=[0, 1, 2])
t = arco.IndexSet(name="t", members=[2030, 2035])
r_from = r.alias("from")
r_to = r.alias("to")

model = arco.Model()
cap = model.add_variables(i, r, t, bounds=arco.NonNegativeFloat, name="CAP")
gen = model.add_variables(i, r, h, t, bounds=arco.NonNegativeFloat, name="GEN")
flow = model.add_variables(
    r_from, r_to, h, t, bounds=arco.NonNegativeFloat, name="FLOW"
)
cf = arco.param([[[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]], [[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]], i, r, h)
emit_rate = arco.param([1.0, 1.0], i)
hours_weight = arco.param([1.0, 1.0, 1.0], h)
emit_cap = arco.param([1.0, 1.0], t)
peak_load = arco.param([[1.0, 1.0], [1.0, 1.0]], r, t)
minloadfrac = arco.param([1.0, 1.0], i)
valcap = arco.param([[[True, True], [True, True]], [[True, True], [True, True]]], i, r, t)

assert_type(cf * gen, arco.ExprArray)
assert_type(gen * cf, arco.ExprArray)
assert_type(2.0 * emit_rate, arco.ParamArray | float)
assert_type(minloadfrac > 0, arco.ParamArray | float)
assert_type(valcap & (minloadfrac > 0), arco.ParamArray | float)

assert_type(gen @ i, arco.Expr | arco.ExprArray)
assert_type(gen @ (i, r, h), arco.Expr | arco.ExprArray)
assert_type(gen >> i, arco.Expr | arco.ExprArray)
assert_type(gen.sum(over=i), arco.Expr | arco.ExprArray)
assert_type(gen.sum(), arco.Expr)
assert_type(gen.diff(over=h), arco.ExprArray)
assert_type(cap.cumsum(over=t), arco.ExprArray)
assert_type(gen.roll(shift=-1, over=h), arco.ExprArray)

imports_by_region = flow @ r_from
assert isinstance(imports_by_region, arco.ExprArray)
assert_type(imports_by_region.relabel_axis(r_to, r), arco.ExprArray)

emissions_by_year = (emit_rate * hours_weight * gen) @ (i, r, h)
assert isinstance(emissions_by_year, arco.ExprArray)
assert_type(emissions_by_year <= emit_cap, arco.ConstraintArray)

capacity_by_region = cap @ i
assert isinstance(capacity_by_region, arco.ExprArray)
assert_type(capacity_by_region >= 1.15 * peak_load, arco.ConstraintArray)

assert_type(gen <= cf * cap, arco.ConstraintArray)
assert_type(gen.diff(over=h) >= 0.0, arco.ConstraintArray)
assert_type((emit_rate * cap).sum(), arco.Expr)

# The bound form used by the ReEDS transmission variables is part of the public
# type contract even though this fixture uses a nonnegative bound above.
model.add_variables(
    r_from,
    r_to,
    bounds=arco.Bounds(lower=0.0, upper=arco.param([[1.0, 1.0], [1.0, 1.0]], r_from, r_to)),
    active=arco.param([[True, True], [True, True]], r_from, r_to),
    name="FLOW_BOUNDED",
)
