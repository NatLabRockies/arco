//! Binding-agnostic labeled array utilities.

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AxisSpec {
    name: String,
    len: usize,
}

impl AxisSpec {
    #[must_use]
    pub fn new(name: impl Into<String>, len: usize) -> Self {
        Self {
            name: name.into(),
            len,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabeledShape {
    axes: Vec<AxisSpec>,
}

impl LabeledShape {
    /// Create a labeled shape and reject ambiguous duplicate axes.
    pub fn new(axes: Vec<AxisSpec>) -> Result<Self, ShapeError> {
        let mut seen = BTreeSet::new();
        for axis in &axes {
            let key = (axis.name.clone(), axis.len);
            if !seen.insert(key) {
                return Err(ShapeError::DuplicateAxis {
                    axis: axis.name.clone(),
                });
            }
        }
        Ok(Self { axes })
    }

    #[must_use]
    pub fn axes(&self) -> &[AxisSpec] {
        &self.axes
    }

    #[must_use]
    pub fn shape(&self) -> Vec<usize> {
        self.axes.iter().map(AxisSpec::len).collect()
    }

    #[must_use]
    pub fn total_len(&self) -> usize {
        self.axes.iter().map(AxisSpec::len).product()
    }

    /// Find an axis by exact label match.
    #[must_use]
    pub fn axis_index(&self, axis: &AxisSpec) -> Option<usize> {
        self.axes.iter().position(|candidate| candidate == axis)
    }

    /// Return the axis positions for a reduction selection.
    pub fn axis_indices<'a>(
        &self,
        selected: impl IntoIterator<Item = &'a AxisSpec>,
    ) -> Result<Vec<usize>, ShapeError> {
        let mut indices = Vec::new();
        for axis in selected {
            let idx = self
                .axis_index(axis)
                .ok_or_else(|| ShapeError::MissingAxis {
                    axis: axis.name.clone(),
                })?;
            if indices.contains(&idx) {
                return Err(ShapeError::DuplicateAxis {
                    axis: axis.name.clone(),
                });
            }
            indices.push(idx);
        }
        Ok(indices)
    }

    #[must_use]
    pub fn reduced(&self, reduced_axes: &[usize]) -> Self {
        let axes = self
            .axes
            .iter()
            .enumerate()
            .filter(|(idx, _)| !reduced_axes.contains(idx))
            .map(|(_, axis)| axis.clone())
            .collect();
        Self { axes }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeError {
    DuplicateAxis {
        axis: String,
    },
    MissingAxis {
        axis: String,
    },
    AxisLengthMismatch {
        axis: String,
        source: usize,
        target: usize,
    },
    ValueCountMismatch {
        expected: usize,
        actual: usize,
    },
}

impl std::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateAxis { axis } => {
                write!(f, "duplicate axis '{axis}' requires an explicit alias")
            }
            Self::MissingAxis { axis } => write!(f, "axis '{axis}' is not present in the target"),
            Self::AxisLengthMismatch {
                axis,
                source,
                target,
            } => write!(
                f,
                "axis '{axis}' has mismatched lengths ({source} vs {target})"
            ),
            Self::ValueCountMismatch { expected, actual } => write!(
                f,
                "flat value count {actual} does not match expected size {expected}"
            ),
        }
    }
}

impl std::error::Error for ShapeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastPlan {
    source: LabeledShape,
    target: LabeledShape,
    target_to_source: Vec<Option<usize>>,
    source_strides: Vec<usize>,
    target_strides: Vec<usize>,
}

impl BroadcastPlan {
    pub fn new(source: LabeledShape, target: LabeledShape) -> Result<Self, ShapeError> {
        let mut target_to_source = Vec::with_capacity(target.axes.len());
        for target_axis in target.axes() {
            if let Some(source_idx) = source.axis_index(target_axis) {
                let source_axis = &source.axes()[source_idx];
                if source_axis.len() != target_axis.len() {
                    return Err(ShapeError::AxisLengthMismatch {
                        axis: target_axis.name().to_string(),
                        source: source_axis.len(),
                        target: target_axis.len(),
                    });
                }
                target_to_source.push(Some(source_idx));
            } else {
                target_to_source.push(None);
            }
        }

        for source_axis in source.axes() {
            if target.axis_index(source_axis).is_none() {
                return Err(ShapeError::MissingAxis {
                    axis: source_axis.name().to_string(),
                });
            }
        }

        Ok(Self {
            source_strides: row_major_strides(&source.shape()),
            target_strides: row_major_strides(&target.shape()),
            source,
            target,
            target_to_source,
        })
    }

    #[must_use]
    pub fn target_shape(&self) -> &[AxisSpec] {
        self.target.axes()
    }

    #[must_use]
    pub fn target_total_len(&self) -> usize {
        self.target.total_len()
    }

    /// Broadcast a flat dense source array into the target order and shape.
    pub fn broadcast_dense<T: Clone>(&self, values: &[T]) -> Result<Vec<T>, ShapeError> {
        let expected = self.source.total_len();
        if values.len() != expected {
            return Err(ShapeError::ValueCountMismatch {
                expected,
                actual: values.len(),
            });
        }

        let mut out = Vec::with_capacity(self.target_total_len());
        for target_flat in 0..self.target_total_len() {
            let source_flat = self.source_offset_for_target_flat(target_flat);
            out.push(values[source_flat].clone());
        }
        Ok(out)
    }

    /// Return the target-flat coordinates where the broadcasted predicate is true.
    pub fn active_target_indices<T>(
        &self,
        values: &[T],
        predicate: impl Fn(&T) -> bool,
    ) -> Result<Vec<usize>, ShapeError> {
        let expected = self.source.total_len();
        if values.len() != expected {
            return Err(ShapeError::ValueCountMismatch {
                expected,
                actual: values.len(),
            });
        }

        let mut indices = Vec::new();
        for target_flat in 0..self.target_total_len() {
            let source_flat = self.source_offset_for_target_flat(target_flat);
            if predicate(&values[source_flat]) {
                indices.push(target_flat);
            }
        }
        Ok(indices)
    }

    fn source_offset_for_target_flat(&self, target_flat: usize) -> usize {
        let mut remainder = target_flat;
        let mut source_flat = 0;

        for (target_axis, target_stride) in self.target_strides.iter().enumerate() {
            let coordinate = remainder / target_stride;
            remainder %= target_stride;
            if let Some(source_axis) = self.target_to_source[target_axis] {
                source_flat += coordinate * self.source_strides[source_axis];
            }
        }

        source_flat
    }
}

#[must_use]
pub fn row_major_strides(shape: &[usize]) -> Vec<usize> {
    let mut strides = vec![1; shape.len()];
    let mut stride = 1;
    for (idx, size) in shape.iter().enumerate().rev() {
        strides[idx] = stride;
        stride *= *size;
    }
    strides
}

#[cfg(test)]
mod tests {
    use super::{AxisSpec, BroadcastPlan, LabeledShape, ShapeError};

    #[test]
    fn rejects_duplicate_axes_without_alias() {
        let result = LabeledShape::new(vec![AxisSpec::new("r", 2), AxisSpec::new("r", 2)]);
        assert!(matches!(result, Err(ShapeError::DuplicateAxis { .. })));
    }

    #[test]
    fn broadcasts_labeled_values_into_missing_axis() {
        let source = LabeledShape::new(vec![AxisSpec::new("i", 2), AxisSpec::new("t", 3)])
            .expect("source shape");
        let target = LabeledShape::new(vec![
            AxisSpec::new("i", 2),
            AxisSpec::new("h", 2),
            AxisSpec::new("t", 3),
        ])
        .expect("target shape");

        let plan = BroadcastPlan::new(source, target).expect("broadcast plan");
        let values = vec![1, 2, 3, 4, 5, 6];
        let broadcast = plan.broadcast_dense(&values).expect("broadcast");

        assert_eq!(broadcast, vec![1, 2, 3, 1, 2, 3, 4, 5, 6, 4, 5, 6]);
    }

    #[test]
    fn computes_active_indices_from_broadcast_mask() {
        let source = LabeledShape::new(vec![AxisSpec::new("r", 2), AxisSpec::new("t", 2)])
            .expect("source shape");
        let target = LabeledShape::new(vec![
            AxisSpec::new("r", 2),
            AxisSpec::new("h", 3),
            AxisSpec::new("t", 2),
        ])
        .expect("target shape");
        let plan = BroadcastPlan::new(source, target).expect("broadcast plan");

        let active = plan
            .active_target_indices(&[true, false, false, true], |value| *value)
            .expect("active indices");

        assert_eq!(active, vec![0, 2, 4, 7, 9, 11]);
    }
}
