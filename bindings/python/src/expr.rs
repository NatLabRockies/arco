//! Python wrappers for expressions.

use arco_model::VariableId;
use arco_model::expr::{ComparisonSense, ConstraintExpr, Expr};
use pyo3::Borrowed;
use pyo3::prelude::*;

use crate::PyObject;
use crate::py_modules::errors::{
    ExprCoefficientError, ExprConstantOffsetError, ExprDivisionByZeroError,
    ExprNotSingleVariableError, ExprTypeError,
};
use crate::py_modules::variable::PyVariable;

/// A Python object coercible to an expression.
///
/// Tries in order: PyVariable -> PyExpr -> f64 scalar.
pub struct ExprLike(pub PyExpr);

impl<'a, 'py> FromPyObject<'a, 'py> for ExprLike {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let ob = ob.to_owned();
        if let Ok(var) = ob.extract::<PyRef<'_, PyVariable>>() {
            return Ok(ExprLike(var.to_expr()));
        }
        if let Ok(expr) = ob.extract::<PyExpr>() {
            return Ok(ExprLike(expr));
        }
        if let Ok(val) = ob.extract::<f64>() {
            return Ok(ExprLike(PyExpr::from_expr(Expr::from_constant(val))));
        }
        Err(ExprTypeError::new_err(
            "expected an Expr, Variable, or numeric constant",
        ))
    }
}

/// Composable expression for objectives and constraints.
#[pyo3_macros::pyclass(from_py_object, name = "Expr")]
#[derive(Debug, Clone, Default)]
pub struct PyExpr {
    inner: Expr,
}

impl PyExpr {
    pub fn from_expr(inner: Expr) -> Self {
        Self { inner }
    }

    pub fn from_term(var_id: u32, coeff: f64) -> Self {
        Self {
            inner: Expr::term(VariableId::new(var_id), coeff),
        }
    }

    pub fn into_inner(self) -> Expr {
        self.inner
    }

    /// Return (Expr-without-constant, constant) for callers that need to
    /// adjust bounds by the constant offset.
    pub fn into_parts(self) -> (Expr, f64) {
        let constant = self.inner.constant();
        (self.inner.without_constant(), constant)
    }

    pub fn inner(&self) -> &Expr {
        &self.inner
    }

    pub fn constant(&self) -> f64 {
        self.inner.constant()
    }

    pub fn without_constant(&self) -> Self {
        Self::from_expr(self.inner.without_constant())
    }

    pub(crate) fn scale(&self, by: f64) -> Self {
        Self::from_expr(self.inner.scale(by))
    }

    pub(crate) fn add(&self, other: PyExpr) -> Self {
        Self::from_expr(self.inner.add(&other.inner))
    }

    pub(crate) fn add_assign(&mut self, other: &PyExpr) {
        self.inner.add_assign(&other.inner);
    }

    pub(crate) fn add_assign_owned(&mut self, other: PyExpr) {
        self.inner.add_assign_owned(other.inner);
    }

    pub(crate) fn add_constant(&self, value: f64) -> Self {
        Self::from_expr(self.inner.add_constant(value))
    }

    pub(crate) fn compare_py(
        &self,
        rhs: &Bound<'_, PyAny>,
        sense: ComparisonSense,
    ) -> PyResult<PyConstraintExpr> {
        let ExprLike(rhs) = rhs.extract()?;
        Ok(PyConstraintExpr::new(
            self.inner.compare_expr(&rhs.inner, sense),
        ))
    }

    pub(crate) fn add_any(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let ExprLike(other) = other.extract()?;
        Ok(self.add(other))
    }

    pub(crate) fn sub_any(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let ExprLike(other) = other.extract()?;
        Ok(self.add(other.scale(-1.0)))
    }

    pub(crate) fn rsub_any(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let ExprLike(other) = other.extract()?;
        Ok(other.add(self.scale(-1.0)))
    }
}

#[pyo3_macros::pymethods]
impl PyExpr {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    /// Scale the expression by a constant factor.
    #[pyo3(name = "scale", signature = (*, by))]
    fn py_scale(&self, by: f64) -> Self {
        self.scale(by)
    }

    /// Add another expression to this one, preserving duplicate terms.
    #[pyo3(name = "add", signature = (*, other))]
    fn py_add(&self, other: PyExpr) -> Self {
        self.add(other)
    }

    fn __add__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        warn_if_large(py, self.inner.num_terms())?;
        #[cfg(feature = "ipopt")]
        {
            use crate::py_modules::nonlinear::{PyNonlinearExpr, coerce_to_nl, linear_expr_to_nl};
            use arco_ops::nlp::{BinaryOp, NonlinearExpr as NlExpr};
            if let Ok(nl_other) = other.extract::<PyRef<'_, PyNonlinearExpr>>() {
                let nl = NlExpr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(linear_expr_to_nl(&self.inner)),
                    right: Box::new(nl_other.nl().clone()),
                };
                let _ = coerce_to_nl;
                return Ok(PyNonlinearExpr::from_nl(nl)
                    .into_pyobject(py)?
                    .unbind()
                    .into());
            }
        }
        Ok(self.add_any(other)?.into_pyobject(py)?.unbind().into())
    }

    fn __radd__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        warn_if_large(py, self.inner.num_terms())?;
        #[cfg(feature = "ipopt")]
        {
            use crate::py_modules::nonlinear::{PyNonlinearExpr, linear_expr_to_nl};
            use arco_ops::nlp::{BinaryOp, NonlinearExpr as NlExpr};
            if let Ok(nl_other) = other.extract::<PyRef<'_, PyNonlinearExpr>>() {
                let nl = NlExpr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(nl_other.nl().clone()),
                    right: Box::new(linear_expr_to_nl(&self.inner)),
                };
                return Ok(PyNonlinearExpr::from_nl(nl)
                    .into_pyobject(py)?
                    .unbind()
                    .into());
            }
        }
        Ok(self.add_any(other)?.into_pyobject(py)?.unbind().into())
    }

    fn __iadd__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        let ExprLike(other) = other.extract()?;
        self.add_assign_owned(other);
        Ok(())
    }

    fn __sub__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        #[cfg(feature = "ipopt")]
        {
            use crate::py_modules::nonlinear::{PyNonlinearExpr, linear_expr_to_nl};
            use arco_ops::nlp::{BinaryOp, NonlinearExpr as NlExpr};
            if let Ok(nl_other) = other.extract::<PyRef<'_, PyNonlinearExpr>>() {
                let nl = NlExpr::Binary {
                    op: BinaryOp::Subtract,
                    left: Box::new(linear_expr_to_nl(&self.inner)),
                    right: Box::new(nl_other.nl().clone()),
                };
                return Ok(PyNonlinearExpr::from_nl(nl)
                    .into_pyobject(py)?
                    .unbind()
                    .into());
            }
        }
        Ok(self.sub_any(other)?.into_pyobject(py)?.unbind().into())
    }

    fn __rsub__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        #[cfg(feature = "ipopt")]
        {
            use crate::py_modules::nonlinear::{PyNonlinearExpr, linear_expr_to_nl};
            use arco_ops::nlp::{BinaryOp, NonlinearExpr as NlExpr};
            if let Ok(nl_other) = other.extract::<PyRef<'_, PyNonlinearExpr>>() {
                let nl = NlExpr::Binary {
                    op: BinaryOp::Subtract,
                    left: Box::new(nl_other.nl().clone()),
                    right: Box::new(linear_expr_to_nl(&self.inner)),
                };
                return Ok(PyNonlinearExpr::from_nl(nl)
                    .into_pyobject(py)?
                    .unbind()
                    .into());
            }
        }
        Ok(self.rsub_any(other)?.into_pyobject(py)?.unbind().into())
    }

    fn __mul__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        #[cfg(feature = "ipopt")]
        {
            use crate::py_modules::nonlinear::{PyNonlinearExpr, coerce_to_nl, linear_expr_to_nl};
            use crate::py_modules::variable::PyVariable;
            use arco_ops::nlp::{BinaryOp, NonlinearExpr as NlExpr};
            if other.extract::<PyRef<'_, PyNonlinearExpr>>().is_ok()
                || other.extract::<PyRef<'_, PyVariable>>().is_ok()
                || other.extract::<PyExpr>().is_ok()
            {
                let rhs = coerce_to_nl(other)?;
                let nl = NlExpr::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(linear_expr_to_nl(&self.inner)),
                    right: Box::new(rhs),
                };
                return Ok(PyNonlinearExpr::from_nl(nl)
                    .into_pyobject(py)?
                    .unbind()
                    .into());
            }
        }
        if let Ok(scalar) = other.extract::<f64>() {
            return Ok(self.scale(scalar).into_pyobject(py)?.unbind().into());
        }
        Err(ExprTypeError::new_err(
            "expected a numeric scalar, Variable, Expr, or NonlinearExpr",
        ))
    }

    fn __rmul__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        #[cfg(feature = "ipopt")]
        {
            use crate::py_modules::nonlinear::{PyNonlinearExpr, coerce_to_nl, linear_expr_to_nl};
            use crate::py_modules::variable::PyVariable;
            use arco_ops::nlp::{BinaryOp, NonlinearExpr as NlExpr};
            if other.extract::<PyRef<'_, PyNonlinearExpr>>().is_ok()
                || other.extract::<PyRef<'_, PyVariable>>().is_ok()
                || other.extract::<PyExpr>().is_ok()
            {
                let lhs = coerce_to_nl(other)?;
                let nl = NlExpr::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(lhs),
                    right: Box::new(linear_expr_to_nl(&self.inner)),
                };
                return Ok(PyNonlinearExpr::from_nl(nl)
                    .into_pyobject(py)?
                    .unbind()
                    .into());
            }
        }
        if let Ok(scalar) = other.extract::<f64>() {
            return Ok(self.scale(scalar).into_pyobject(py)?.unbind().into());
        }
        Err(ExprTypeError::new_err(
            "expected a numeric scalar, Variable, Expr, or NonlinearExpr",
        ))
    }

    fn __neg__(&self) -> Self {
        self.scale(-1.0)
    }

    fn __truediv__(&self, other: f64) -> PyResult<Self> {
        if other == 0.0 {
            return Err(ExprDivisionByZeroError::new_err("division by zero"));
        }
        Ok(self.scale(1.0 / other))
    }

    fn __ge__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        nl_or_linear_compare(py, self, rhs, ComparisonSense::GreaterEqual)
    }

    fn __le__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        nl_or_linear_compare(py, self, rhs, ComparisonSense::LessEqual)
    }

    fn __eq__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        nl_or_linear_compare(py, self, rhs, ComparisonSense::Equal)
    }

    #[allow(clippy::float_cmp)]
    fn __int__(&self) -> PyResult<u32> {
        if self.inner.constant() != 0.0 {
            return Err(ExprConstantOffsetError::new_err(
                "expression has constant offset",
            ));
        }
        let terms = self.inner.linear_terms();
        if terms.len() != 1 {
            return Err(ExprNotSingleVariableError::new_err(
                "expression does not represent a single variable",
            ));
        }
        let (var_id, coeff) = terms[0];
        // Exact comparison is intentional - we only allow coefficient of exactly 1.0
        if coeff != 1.0 {
            return Err(ExprCoefficientError::new_err(
                "expression coefficient must be 1.0",
            ));
        }
        Ok(var_id.inner())
    }

    fn __index__(&self) -> PyResult<u32> {
        self.__int__()
    }
}

/// A constraint expression (linear expression with comparison and RHS).
#[pyo3_macros::pyclass(from_py_object, name = "ConstraintExpr")]
#[derive(Clone)]
pub struct PyConstraintExpr {
    inner: ConstraintExpr,
}

impl PyConstraintExpr {
    pub(crate) fn new(inner: ConstraintExpr) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &ConstraintExpr {
        &self.inner
    }
}

#[pyo3_macros::pymethods]
impl PyConstraintExpr {
    #[getter]
    fn expr(&self) -> PyExpr {
        PyExpr::from_expr(self.inner.expr().clone())
    }

    #[getter]
    fn sense(&self) -> String {
        self.inner.sense().as_str().to_string()
    }

    #[getter]
    fn rhs(&self) -> f64 {
        self.inner.rhs()
    }

    fn __repr__(&self) -> String {
        format!(
            "ConstraintExpr(sense='{}', rhs={})",
            self.inner.sense().as_str(),
            self.inner.rhs()
        )
    }
}

const LARGE_EXPR_THRESHOLD: usize = 10_000;

/// Emit a UserWarning when an Expr being copied via `+` is large.
fn warn_if_large(py: Python<'_>, num_terms: usize) -> PyResult<()> {
    if num_terms >= LARGE_EXPR_THRESHOLD {
        let warnings = py.import("warnings")?;
        warnings.call_method1(
            "warn",
            (
                format!(
                    "Adding to an Expr with {num_terms} terms using `+` copies the entire \
                     expression. Use `+=` for in-place accumulation or `.sum()` on arrays."
                ),
                py.get_type::<pyo3::exceptions::PyUserWarning>(),
                2_i32, // stacklevel: point at the caller's code
            ),
        )?;
    }
    Ok(())
}

/// Register expression classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyExpr>()?;
    m.add_class::<PyConstraintExpr>()?;
    Ok(())
}

/// Compare a linear `PyExpr` against `rhs`. If `rhs` is a `NonlinearExpr` (and
/// the `ipopt` feature is enabled), returns a `PyNonlinearConstraintExpr`;
/// otherwise returns a `PyConstraintExpr`.
pub(crate) fn nl_or_linear_compare(
    py: Python<'_>,
    lhs: &PyExpr,
    rhs: &Bound<'_, PyAny>,
    sense: ComparisonSense,
) -> PyResult<PyObject> {
    #[cfg(feature = "ipopt")]
    {
        use crate::py_modules::nonlinear::{
            NlSense, PyNonlinearConstraintExpr, PyNonlinearExpr, linear_expr_to_nl,
        };
        use arco_ops::nlp::{BinaryOp, NonlinearExpr as NlExpr};
        if let Ok(nl_rhs) = rhs.extract::<PyRef<'_, PyNonlinearExpr>>() {
            let lhs_nl = linear_expr_to_nl(&lhs.inner);
            let diff = NlExpr::Binary {
                op: BinaryOp::Subtract,
                left: Box::new(lhs_nl),
                right: Box::new(nl_rhs.nl().clone()),
            };
            let nl_sense = match sense {
                ComparisonSense::GreaterEqual => NlSense::Ge,
                ComparisonSense::LessEqual => NlSense::Le,
                ComparisonSense::Equal => NlSense::Eq,
            };
            let con = PyNonlinearConstraintExpr::new(diff, nl_sense);
            let _ = PyNonlinearExpr::from_nl;
            return Ok(con.into_pyobject(py)?.unbind().into());
        }
    }
    Ok(lhs
        .compare_py(rhs, sense)?
        .into_pyobject(py)?
        .unbind()
        .into())
}
