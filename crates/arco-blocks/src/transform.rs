use crate::PyObject;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyFloat, PyList, PySequence, PySequenceMethods, PyString};

enum TransformStep {
    Custom(Py<PyAny>),
    Scale(Py<PyAny>),
    Offset(Py<PyAny>),
    Shift(i64),
    Clip { lower: f64, upper: f64 },
    Select(Vec<usize>),
}

#[pyclass(name = "Transform")]
pub struct Transform {
    steps: Vec<TransformStep>,
}

#[pymethods]
impl Transform {
    #[new]
    #[pyo3(signature = (steps=None))]
    fn new(steps: Option<Vec<PyObject>>) -> Self {
        let steps = steps
            .unwrap_or_default()
            .into_iter()
            .map(TransformStep::Custom)
            .collect();
        Self { steps }
    }

    fn __or__(&self, py: Python<'_>, other: &Transform) -> Transform {
        let mut steps = clone_steps(py, &self.steps);
        steps.extend(clone_steps(py, &other.steps));
        Transform { steps }
    }

    fn apply(&self, py: Python<'_>, values: PyObject) -> PyResult<PyObject> {
        self.apply_internal(py, values)
    }

    #[staticmethod]
    fn identity() -> Self {
        Self::identity_internal()
    }

    #[staticmethod]
    fn scale(factor: PyObject) -> Self {
        Self {
            steps: vec![TransformStep::Scale(factor)],
        }
    }

    #[staticmethod]
    fn offset(delta: PyObject) -> Self {
        Self {
            steps: vec![TransformStep::Offset(delta)],
        }
    }

    #[staticmethod]
    fn shift(periods: i64) -> Self {
        Self {
            steps: vec![TransformStep::Shift(periods)],
        }
    }

    #[staticmethod]
    fn clip(lower: f64, upper: f64) -> Self {
        Self {
            steps: vec![TransformStep::Clip { lower, upper }],
        }
    }

    #[staticmethod]
    fn select(indices: Vec<usize>) -> Self {
        Self {
            steps: vec![TransformStep::Select(indices)],
        }
    }

    fn clone_with_py(&self, py: Python<'_>) -> Transform {
        self.clone_with_py_internal(py)
    }
}

impl Transform {
    pub(crate) fn apply_internal(&self, py: Python<'_>, values: PyObject) -> PyResult<PyObject> {
        let mut current = values;
        for step in &self.steps {
            current = apply_step(py, step, current)?;
        }
        Ok(current)
    }

    pub(crate) fn identity_internal() -> Self {
        Self { steps: Vec::new() }
    }

    pub(crate) fn clone_with_py_internal(&self, py: Python<'_>) -> Transform {
        Transform {
            steps: clone_steps(py, &self.steps),
        }
    }
}

fn clone_steps(py: Python<'_>, steps: &[TransformStep]) -> Vec<TransformStep> {
    steps
        .iter()
        .map(|step| match step {
            TransformStep::Custom(func) => TransformStep::Custom(func.clone_ref(py)),
            TransformStep::Scale(factor) => TransformStep::Scale(factor.clone_ref(py)),
            TransformStep::Offset(delta) => TransformStep::Offset(delta.clone_ref(py)),
            TransformStep::Shift(periods) => TransformStep::Shift(*periods),
            TransformStep::Clip { lower, upper } => TransformStep::Clip {
                lower: *lower,
                upper: *upper,
            },
            TransformStep::Select(indices) => TransformStep::Select(indices.clone()),
        })
        .collect()
}

fn apply_step(py: Python<'_>, step: &TransformStep, values: PyObject) -> PyResult<PyObject> {
    let value_any = values.bind(py);
    match step {
        TransformStep::Custom(func) => Ok(func.bind(py).call1((values,))?.unbind()),
        TransformStep::Scale(factor) => apply_binary(py, value_any, factor.bind(py), "__mul__"),
        TransformStep::Offset(delta) => apply_binary(py, value_any, delta.bind(py), "__add__"),
        TransformStep::Shift(periods) => apply_shift(py, value_any, *periods),
        TransformStep::Clip { lower, upper } => apply_clip(py, value_any, *lower, *upper),
        TransformStep::Select(indices) => apply_select(py, value_any, indices),
    }
}

fn apply_binary(
    py: Python<'_>,
    values: &Bound<'_, PyAny>,
    rhs: &Bound<'_, PyAny>,
    op: &str,
) -> PyResult<PyObject> {
    if is_sequence(values) {
        let seq = values.cast::<PySequence>()?;
        let rhs_seq = if is_sequence(rhs) {
            Some(rhs.cast::<PySequence>()?)
        } else {
            None
        };
        let len = seq.len()?;
        let mut results = Vec::new();
        if let Some(rhs_seq) = rhs_seq {
            let rhs_len = rhs_seq.len()?;
            let count = len.min(rhs_len);
            for idx in 0..count {
                let left = seq.get_item(idx)?;
                let right = rhs_seq.get_item(idx)?;
                let value = left.call_method1(op, (right,))?;
                results.push(value.unbind());
            }
        } else {
            for idx in 0..len {
                let left = seq.get_item(idx)?;
                let value = left.call_method1(op, (rhs,))?;
                results.push(value.unbind());
            }
        }
        return Ok(PyList::new(py, results)?.into_any().unbind());
    }
    Ok(values.call_method1(op, (rhs,))?.unbind())
}

fn apply_shift(py: Python<'_>, values: &Bound<'_, PyAny>, periods: i64) -> PyResult<PyObject> {
    if !is_sequence(values) {
        return Ok(values.clone().unbind());
    }
    let seq = values.cast::<PySequence>()?;
    let len = seq.len()?;
    let mut items = Vec::with_capacity(len);
    for idx in 0..len {
        items.push(seq.get_item(idx)?.unbind());
    }
    if periods == 0 {
        return Ok(PyList::new(py, items)?.into_any().unbind());
    }
    let fill = PyFloat::new(py, 0.0).into_any().unbind();
    if periods > 0 {
        let shift = periods as usize;
        let mut out = Vec::with_capacity(len + shift);
        for _ in 0..shift {
            out.push(fill.clone_ref(py));
        }
        let keep = len.saturating_sub(shift);
        out.extend(items.into_iter().take(keep));
        return Ok(PyList::new(py, out)?.into_any().unbind());
    }
    let shift = (-periods) as usize;
    let mut out: Vec<PyObject> = items.into_iter().skip(shift).collect();
    for _ in 0..shift {
        out.push(fill.clone_ref(py));
    }
    Ok(PyList::new(py, out)?.into_any().unbind())
}

fn apply_clip(
    py: Python<'_>,
    values: &Bound<'_, PyAny>,
    lower: f64,
    upper: f64,
) -> PyResult<PyObject> {
    if is_sequence(values) {
        let seq = values.cast::<PySequence>()?;
        let len = seq.len()?;
        let mut out = Vec::with_capacity(len);
        for idx in 0..len {
            let value = seq.get_item(idx)?;
            let number = value.extract::<f64>()?;
            let clipped = number.max(lower).min(upper);
            out.push(PyFloat::new(py, clipped).into_any().unbind());
        }
        return Ok(PyList::new(py, out)?.into_any().unbind());
    }
    let number = values.extract::<f64>()?;
    Ok(PyFloat::new(py, number.max(lower).min(upper))
        .into_any()
        .unbind())
}

fn apply_select(
    py: Python<'_>,
    values: &Bound<'_, PyAny>,
    indices: &[usize],
) -> PyResult<PyObject> {
    if !is_sequence(values) {
        return Ok(values.clone().unbind());
    }
    let seq = values.cast::<PySequence>()?;
    let mut out = Vec::with_capacity(indices.len());
    for idx in indices {
        out.push(seq.get_item(*idx)?.unbind());
    }
    Ok(PyList::new(py, out)?.into_any().unbind())
}

fn is_sequence(value: &Bound<'_, PyAny>) -> bool {
    if value.is_instance_of::<PyString>() {
        return false;
    }
    if value.is_instance_of::<PyBytes>() {
        return false;
    }
    value.cast::<PySequence>().is_ok()
}
