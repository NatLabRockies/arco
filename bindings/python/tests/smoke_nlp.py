"""Smoke test: solve Rosenbrock via IPOPT bindings."""

import arco


def main() -> None:
    m = arco.Model()
    x = m.add_variable(bounds=arco.Bounds(lower=-5.0, upper=5.0), name="x")
    y = m.add_variable(bounds=arco.Bounds(lower=-5.0, upper=5.0), name="y")

    # f(x,y) = (1 - x)^2 + 100 * (y - x^2)^2
    one_minus_x = (
        1.0 - x
    )  # PyNonlinearExpr via __rsub__? Falls back to linear; that's fine.
    term1 = arco.pow(one_minus_x, 2)
    inner = y - x * x  # x*x produces NonlinearExpr
    term2 = 100.0 * arco.pow(inner, 2)
    obj = term1 + term2

    m.minimize(obj)
    # Add a trivial constraint so IPOPT sees at least one row (avoids
    # zero-constraint edge case in the underlying binding).
    m.add_constraint(x + y >= -10.0)
    res = m.solve(solver=arco.Ipopt(log_to_console=False))
    print("status:", res.status)
    print("objective:", res.objective_value)
    print("x:", res.value(x))
    print("y:", res.value(y))


if __name__ == "__main__":
    main()
