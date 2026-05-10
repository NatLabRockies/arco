//! Python bindings for nonlinear expressions.
//!
//! Exposes a `NonlinearExpr` type, a `NonlinearConstraintExpr`, and module-level
//! functions (`cos`, `sin`, `sqrt`, `atan`, `exp`, `ln`, `abs`, `pow`) that
//! produce nonlinear expressions consumable by the IPOPT solve path.

use arco_ops::expression::Expr as LinearExpr;
use arco_ops::nlp::{BinaryOp, NonlinearExpr as NlExpr, UnaryOp};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::py_modules::expr::{PyConstraintExpr, PyExpr};
use crate::py_modules::variable::PyVariable;

/// Synthetic name used inside `NonlinearExpr` trees to reference a model
/// variable by its internal `VariableId`.
#[inline]
pub(crate) fn nl_var_name(var_id: u32) -> String {
    format!("__v{var_id}")
}

/// Translate a linear polynomial `Expr` (linear/quadratic/cubic + constant)
/// into the equivalent `NonlinearExpr` tree.
pub(crate) fn linear_expr_to_nl(expr: &LinearExpr) -> NlExpr {
    let mut acc = NlExpr::Constant(expr.constant());

    let push_add = |acc: &mut NlExpr, term: NlExpr| {
        let prev = std::mem::replace(acc, NlExpr::Constant(0.0));
        *acc = match prev {
            NlExpr::Constant(c) if c == 0.0 => term,
            _ => NlExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(prev),
                right: Box::new(term),
            },
        };
    };

    for &(var_id, coeff) in expr.linear_terms() {
        let var = NlExpr::Variable(nl_var_name(var_id.inner()));
        let term = if coeff == 1.0 {
            var
        } else {
            NlExpr::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(NlExpr::Constant(coeff)),
                right: Box::new(var),
            }
        };
        push_add(&mut acc, term);
    }
    for &(a, b, coeff) in expr.quadratic_terms() {
        let prod = NlExpr::Binary {
            op: BinaryOp::Multiply,
            left: Box::new(NlExpr::Variable(nl_var_name(a.inner()))),
            right: Box::new(NlExpr::Variable(nl_var_name(b.inner()))),
        };
        let term = if coeff == 1.0 {
            prod
        } else {
            NlExpr::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(NlExpr::Constant(coeff)),
                right: Box::new(prod),
            }
        };
        push_add(&mut acc, term);
    }
    for &(a, b, c, coeff) in expr.cubic_terms() {
        let prod = NlExpr::Binary {
            op: BinaryOp::Multiply,
            left: Box::new(NlExpr::Variable(nl_var_name(a.inner()))),
            right: Box::new(NlExpr::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(NlExpr::Variable(nl_var_name(b.inner()))),
                right: Box::new(NlExpr::Variable(nl_var_name(c.inner()))),
            }),
        };
        let term = if coeff == 1.0 {
            prod
        } else {
            NlExpr::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(NlExpr::Constant(coeff)),
                right: Box::new(prod),
            }
        };
        push_add(&mut acc, term);
    }
    acc
}

/// Coerce a Python object into a `NonlinearExpr` AST.
pub(crate) fn coerce_to_nl(ob: &Bound<'_, PyAny>) -> PyResult<NlExpr> {
    if let Ok(nl) = ob.extract::<PyRef<'_, PyNonlinearExpr>>() {
        return Ok((*nl.inner).clone());
    }
    if let Ok(var) = ob.extract::<PyRef<'_, PyVariable>>() {
        return Ok(NlExpr::Variable(nl_var_name(var.var_id)));
    }
    if let Ok(expr) = ob.extract::<PyExpr>() {
        return Ok(linear_expr_to_nl(expr.inner()));
    }
    if let Ok(val) = ob.extract::<f64>() {
        return Ok(NlExpr::Constant(val));
    }
    Err(PyTypeError::new_err(
        "expected NonlinearExpr, Expr, Variable, or numeric constant",
    ))
}

/// Comparison sense for a nonlinear constraint expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NlSense {
    Ge,
    Le,
    Eq,
}

impl NlSense {
    pub fn as_str(self) -> &'static str {
        match self {
            NlSense::Ge => "ge",
            NlSense::Le => "le",
            NlSense::Eq => "eq",
        }
    }
}

/// Nonlinear expression (Python-visible).
///
/// Created via operator arithmetic over `Variable`/`Expr`/`NonlinearExpr`
/// operands, or via the module-level `cos`, `sin`, `sqrt`, `atan`, `exp`,
/// `ln`, `abs`, `pow` functions.
#[pyclass(name = "NonlinearExpr", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyNonlinearExpr {
    pub(crate) inner: Box<NlExpr>,
}

impl PyNonlinearExpr {
    pub fn from_nl(inner: NlExpr) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    pub fn nl(&self) -> &NlExpr {
        &self.inner
    }

    pub fn into_inner(self) -> NlExpr {
        *self.inner
    }
}

fn nl_binary(op: BinaryOp, left: NlExpr, right: NlExpr) -> PyNonlinearExpr {
    PyNonlinearExpr::from_nl(NlExpr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn nl_func(name: &str, args: Vec<NlExpr>) -> PyNonlinearExpr {
    PyNonlinearExpr::from_nl(NlExpr::FunctionCall {
        name: name.to_string(),
        args,
    })
}

#[pymethods]
impl PyNonlinearExpr {
    fn __repr__(&self) -> String {
        format!("NonlinearExpr({:?})", self.inner)
    }

    fn __add__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let rhs = coerce_to_nl(other)?;
        Ok(nl_binary(BinaryOp::Add, (*self.inner).clone(), rhs))
    }

    fn __radd__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let lhs = coerce_to_nl(other)?;
        Ok(nl_binary(BinaryOp::Add, lhs, (*self.inner).clone()))
    }

    fn __sub__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let rhs = coerce_to_nl(other)?;
        Ok(nl_binary(BinaryOp::Subtract, (*self.inner).clone(), rhs))
    }

    fn __rsub__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let lhs = coerce_to_nl(other)?;
        Ok(nl_binary(BinaryOp::Subtract, lhs, (*self.inner).clone()))
    }

    fn __mul__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let rhs = coerce_to_nl(other)?;
        Ok(nl_binary(BinaryOp::Multiply, (*self.inner).clone(), rhs))
    }

    fn __rmul__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let lhs = coerce_to_nl(other)?;
        Ok(nl_binary(BinaryOp::Multiply, lhs, (*self.inner).clone()))
    }

    fn __truediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let rhs = coerce_to_nl(other)?;
        Ok(nl_binary(BinaryOp::Divide, (*self.inner).clone(), rhs))
    }

    fn __rtruediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let lhs = coerce_to_nl(other)?;
        Ok(nl_binary(BinaryOp::Divide, lhs, (*self.inner).clone()))
    }

    fn __neg__(&self) -> Self {
        PyNonlinearExpr::from_nl(NlExpr::Unary {
            op: UnaryOp::Negate,
            expr: Box::new((*self.inner).clone()),
        })
    }

    fn __pow__(
        &self,
        exponent: &Bound<'_, PyAny>,
        _modulo: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let exp = coerce_to_nl(exponent)?;
        Ok(nl_func("pow", vec![(*self.inner).clone(), exp]))
    }

    fn __ge__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyNonlinearConstraintExpr> {
        let rhs_nl = coerce_to_nl(rhs)?;
        Ok(PyNonlinearConstraintExpr::new(
            NlExpr::Binary {
                op: BinaryOp::Subtract,
                left: Box::new((*self.inner).clone()),
                right: Box::new(rhs_nl),
            },
            NlSense::Ge,
        ))
    }

    fn __le__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyNonlinearConstraintExpr> {
        let rhs_nl = coerce_to_nl(rhs)?;
        Ok(PyNonlinearConstraintExpr::new(
            NlExpr::Binary {
                op: BinaryOp::Subtract,
                left: Box::new((*self.inner).clone()),
                right: Box::new(rhs_nl),
            },
            NlSense::Le,
        ))
    }

    fn __eq__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyNonlinearConstraintExpr> {
        let rhs_nl = coerce_to_nl(rhs)?;
        Ok(PyNonlinearConstraintExpr::new(
            NlExpr::Binary {
                op: BinaryOp::Subtract,
                left: Box::new((*self.inner).clone()),
                right: Box::new(rhs_nl),
            },
            NlSense::Eq,
        ))
    }
}

/// Constraint expression in nonlinear form. The stored `expr` represents
/// `lhs - rhs`; the constraint reads `expr <sense> 0`.
#[pyclass(name = "NonlinearConstraintExpr", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyNonlinearConstraintExpr {
    pub(crate) expr: PyNonlinearExpr,
    pub(crate) sense: NlSense,
}

impl PyNonlinearConstraintExpr {
    pub fn new(expr: NlExpr, sense: NlSense) -> Self {
        Self {
            expr: PyNonlinearExpr::from_nl(expr),
            sense,
        }
    }

    pub fn nl(&self) -> &NlExpr {
        self.expr.nl()
    }

    pub fn nl_sense(&self) -> NlSense {
        self.sense
    }
}

#[pymethods]
impl PyNonlinearConstraintExpr {
    #[getter]
    fn sense(&self) -> &'static str {
        self.sense.as_str()
    }

    fn __repr__(&self) -> String {
        format!(
            "NonlinearConstraintExpr(sense='{}', expr={:?})",
            self.sense.as_str(),
            self.expr.inner
        )
    }
}

// ───── Module-level math functions ─────────────────────────────────────────

#[pyfunction]
pub(crate) fn cos(arg: &Bound<'_, PyAny>) -> PyResult<PyNonlinearExpr> {
    Ok(nl_func("cos", vec![coerce_to_nl(arg)?]))
}

#[pyfunction]
pub(crate) fn sin(arg: &Bound<'_, PyAny>) -> PyResult<PyNonlinearExpr> {
    Ok(nl_func("sin", vec![coerce_to_nl(arg)?]))
}

#[pyfunction]
pub(crate) fn atan(arg: &Bound<'_, PyAny>) -> PyResult<PyNonlinearExpr> {
    Ok(nl_func("atan", vec![coerce_to_nl(arg)?]))
}

#[pyfunction]
pub(crate) fn sqrt(arg: &Bound<'_, PyAny>) -> PyResult<PyNonlinearExpr> {
    Ok(nl_func("sqrt", vec![coerce_to_nl(arg)?]))
}

#[pyfunction]
pub(crate) fn exp(arg: &Bound<'_, PyAny>) -> PyResult<PyNonlinearExpr> {
    Ok(nl_func("exp", vec![coerce_to_nl(arg)?]))
}

#[pyfunction]
pub(crate) fn ln(arg: &Bound<'_, PyAny>) -> PyResult<PyNonlinearExpr> {
    Ok(nl_func("ln", vec![coerce_to_nl(arg)?]))
}

#[pyfunction(name = "abs_")]
pub(crate) fn abs_nl(arg: &Bound<'_, PyAny>) -> PyResult<PyNonlinearExpr> {
    Ok(nl_func("abs", vec![coerce_to_nl(arg)?]))
}

#[pyfunction]
pub(crate) fn pow(
    base: &Bound<'_, PyAny>,
    exponent: &Bound<'_, PyAny>,
) -> PyResult<PyNonlinearExpr> {
    Ok(nl_func(
        "pow",
        vec![coerce_to_nl(base)?, coerce_to_nl(exponent)?],
    ))
}

/// Promote a `PyConstraintExpr` (linear `lhs sense rhs`) to a nonlinear
/// constraint expression. Used when callers mix linear and nonlinear pieces.
#[allow(dead_code)]
pub(crate) fn linear_constraint_to_nl(expr: &PyConstraintExpr) -> PyNonlinearConstraintExpr {
    let inner = expr.inner();
    let lhs_nl = linear_expr_to_nl(inner.expr());
    let rhs_nl = NlExpr::Constant(inner.rhs());
    let sense = match inner.sense() {
        arco_ops::expression::ComparisonSense::GreaterEqual => NlSense::Ge,
        arco_ops::expression::ComparisonSense::LessEqual => NlSense::Le,
        arco_ops::expression::ComparisonSense::Equal => NlSense::Eq,
    };
    PyNonlinearConstraintExpr::new(
        NlExpr::Binary {
            op: BinaryOp::Subtract,
            left: Box::new(lhs_nl),
            right: Box::new(rhs_nl),
        },
        sense,
    )
}

/// Coerce a Python object into a `PyNonlinearConstraintExpr`. Accepts:
///  - `NonlinearConstraintExpr` directly,
///  - linear `ConstraintExpr` (promoted).
#[allow(dead_code)]
pub(crate) fn coerce_to_nl_constraint(
    ob: &Bound<'_, PyAny>,
) -> PyResult<PyNonlinearConstraintExpr> {
    if let Ok(c) = ob.extract::<PyRef<'_, PyNonlinearConstraintExpr>>() {
        return Ok((*c).clone());
    }
    if let Ok(c) = ob.extract::<PyConstraintExpr>() {
        return Ok(linear_constraint_to_nl(&c));
    }
    Err(PyValueError::new_err(
        "expected a NonlinearConstraintExpr or linear ConstraintExpr",
    ))
}

/// Register classes and functions with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNonlinearExpr>()?;
    m.add_class::<PyNonlinearConstraintExpr>()?;
    m.add_function(wrap_pyfunction!(cos, m)?)?;
    m.add_function(wrap_pyfunction!(sin, m)?)?;
    m.add_function(wrap_pyfunction!(atan, m)?)?;
    m.add_function(wrap_pyfunction!(sqrt, m)?)?;
    m.add_function(wrap_pyfunction!(exp, m)?)?;
    m.add_function(wrap_pyfunction!(ln, m)?)?;
    m.add_function(wrap_pyfunction!(abs_nl, m)?)?;
    m.add_function(wrap_pyfunction!(pow, m)?)?;
    Ok(())
}
