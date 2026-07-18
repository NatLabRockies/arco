// Shared Python classes for Arco's Python extension.

use pyo3::prelude::*;

pub type PyObject = Py<PyAny>;

macro_rules! wrap_pyfunction {
    ($function:path, $py_or_module:expr) => {{
        use $function as wrapped_pyfunction;
        pyo3::impl_::pyfunction::WrapPyFunctionArg::wrap_pyfunction(
            $py_or_module,
            &wrapped_pyfunction::_PYO3_DEF,
        )
    }};
}

#[path = "arrays.rs"]
pub mod arrays;
#[path = "bounds.rs"]
pub mod bounds;
#[path = "constraint.rs"]
pub mod constraint;
#[path = "enums.rs"]
pub mod enums;
#[path = "errors.rs"]
pub mod errors;
#[path = "expr.rs"]
pub mod expr;
#[path = "index_set.rs"]
pub mod index_set;
#[cfg(feature = "ipopt")]
#[path = "nonlinear.rs"]
pub mod nonlinear;
#[cfg(feature = "ipopt")]
#[path = "nonlinear_state.rs"]
pub mod nonlinear_state;
#[path = "serde_bridge.rs"]
pub mod serde_bridge;
#[path = "snapshot.rs"]
pub mod snapshot;
#[path = "solver.rs"]
pub mod solver;
#[path = "variable.rs"]
pub mod variable;
#[path = "views.rs"]
pub mod views;

pub mod py_modules {
    pub use crate::arrays;
    pub use crate::bounds;
    pub use crate::constraint;
    pub use crate::enums;
    pub use crate::errors;
    pub use crate::expr;
    pub use crate::index_set;
    #[cfg(feature = "ipopt")]
    pub use crate::nonlinear;
    #[cfg(feature = "ipopt")]
    pub use crate::nonlinear_state;
    pub use crate::serde_bridge;
    pub use crate::snapshot;
    pub use crate::solver;
    pub use crate::variable;
    pub use crate::views;
}
