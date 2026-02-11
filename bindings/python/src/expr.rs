//! Python wrappers for expressions.

use arco_expr::{ComparisonSense, ConstraintExpr, Expr, VariableId};
use pyo3::Borrowed;
use pyo3::prelude::*;

use crate::errors::{
    ExprCoefficientError, ExprConstantOffsetError, ExprDivisionByZeroError,
    ExprNotSingleVariableError,
};
use crate::variable::PyVariable;

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
        Err(pyo3::exceptions::PyTypeError::new_err(
            "expected an Expr, Variable, or numeric constant",
        ))
    }
}

/// Composable expression for objectives and constraints.
#[pyclass(from_py_object, name = "Expr")]
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

    pub fn scale(&self, by: f64) -> Self {
        Self::from_expr(self.inner.scale(by))
    }

    pub fn add(&self, other: PyExpr) -> Self {
        Self::from_expr(self.inner.add(&other.inner))
    }

    pub fn add_constant(&self, value: f64) -> Self {
        Self::from_expr(self.inner.add_constant(value))
    }

    pub fn compare_py(
        &self,
        rhs: &Bound<'_, PyAny>,
        sense: ComparisonSense,
    ) -> PyResult<PyConstraintExpr> {
        let ExprLike(rhs) = rhs.extract()?;
        Ok(PyConstraintExpr::new(
            self.inner.compare_expr(&rhs.inner, sense),
        ))
    }

    pub fn add_any(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let ExprLike(other) = other.extract()?;
        Ok(self.add(other))
    }

    pub fn sub_any(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let ExprLike(other) = other.extract()?;
        Ok(self.add(other.scale(-1.0)))
    }

    pub fn rsub_any(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let ExprLike(other) = other.extract()?;
        Ok(other.add(self.scale(-1.0)))
    }
}

#[pymethods]
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

    fn __add__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.add_any(other)
    }

    fn __radd__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.add_any(other)
    }

    fn __sub__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.sub_any(other)
    }

    fn __rsub__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.rsub_any(other)
    }

    fn __mul__(&self, other: f64) -> Self {
        self.scale(other)
    }

    fn __rmul__(&self, other: f64) -> Self {
        self.scale(other)
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

    fn __ge__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintExpr> {
        self.compare_py(rhs, ComparisonSense::GreaterEqual)
    }

    fn __le__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintExpr> {
        self.compare_py(rhs, ComparisonSense::LessEqual)
    }

    fn __eq__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintExpr> {
        self.compare_py(rhs, ComparisonSense::Equal)
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
#[pyclass(from_py_object, name = "ConstraintExpr")]
#[derive(Clone)]
pub struct PyConstraintExpr {
    inner: ConstraintExpr,
}

impl PyConstraintExpr {
    pub fn new(inner: ConstraintExpr) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &ConstraintExpr {
        &self.inner
    }
}

#[pymethods]
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

/// Register expression classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyExpr>()?;
    m.add_class::<PyConstraintExpr>()?;
    Ok(())
}
