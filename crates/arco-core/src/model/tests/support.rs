use crate::types::{Bounds, Constraint, Variable};

pub(super) fn active_continuous_variable(lower: f64, upper: f64) -> Variable {
    Variable {
        bounds: Bounds::new(lower, upper),
        is_integer: false,
        is_active: true,
    }
}

pub(super) fn bounded_constraint(lower: f64, upper: f64) -> Constraint {
    Constraint {
        bounds: Bounds::new(lower, upper),
    }
}
