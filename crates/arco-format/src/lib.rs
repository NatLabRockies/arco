//! Exchange seam for portable Arco IR and primitive model views.

use std::io::Write;

pub use arco_export::ExportError;
use arco_ir::{
    PortableConstraintSense, PortableLinearConstraint, PortableLinearObjective, PortableLinearTerm,
    PortableObjectiveSense, PortableProblem, PortableVariableInstance, PortableVariableKind,
};
use arco_model::{ConstraintId, ModelView, Sense, VariableId};

pub fn write_lp(problem: &PortableProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    arco_export::write_portable_lp(problem, writer)
}

pub fn write_mps(problem: &PortableProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    arco_export::write_portable_mps(problem, writer)
}

/// Format request over a primitive model view.
pub struct ModelViewFormatRequest<'a, V: ModelView + ?Sized> {
    pub model: &'a V,
    pub policy: RenderPolicy,
}

/// Rendering policy for model-view exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderPolicy {
    pub include_generated_names: bool,
}

impl Default for RenderPolicy {
    fn default() -> Self {
        Self {
            include_generated_names: true,
        }
    }
}

/// Format result bytes and basic metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatResult {
    pub bytes: Vec<u8>,
    pub format: &'static str,
}

/// Render a primitive model view as LP text.
pub fn write_model_view_lp(
    model: &impl ModelView,
    writer: &mut dyn Write,
) -> Result<(), ExportError> {
    let portable = portable_problem_from_model_view(model);
    write_lp(&portable, writer)
}

/// Render a primitive model view as LP bytes.
pub fn export_model_view_lp(model: &impl ModelView) -> Result<FormatResult, ExportError> {
    let mut bytes = Vec::new();
    write_model_view_lp(model, &mut bytes)?;
    Ok(FormatResult {
        bytes,
        format: "lp",
    })
}

fn portable_problem_from_model_view(model: &impl ModelView) -> PortableProblem {
    let variable_instances = (0..model.num_variables())
        .filter_map(|idx| {
            let variable_id = VariableId::new(idx as u32);
            let variable = model.variable(variable_id)?;
            let is_binary_bounds = variable.bounds.lower.to_bits() == 0.0f64.to_bits()
                && variable.bounds.upper.to_bits() == 1.0f64.to_bits();
            let kind = if variable.is_integer && is_binary_bounds {
                PortableVariableKind::Binary
            } else if variable.is_integer {
                PortableVariableKind::Integer
            } else {
                PortableVariableKind::Continuous
            };
            Some(PortableVariableInstance {
                name: variable_name(idx),
                family: variable_name(idx),
                lower: variable.bounds.lower,
                upper: variable
                    .bounds
                    .upper
                    .is_finite()
                    .then_some(variable.bounds.upper),
                kind,
            })
        })
        .collect::<Vec<_>>();

    let mut terms_by_constraint = vec![Vec::new(); model.num_constraints()];
    for var_idx in 0..model.num_variables() {
        let variable_id = VariableId::new(var_idx as u32);
        let Some(column) = model.column(variable_id) else {
            continue;
        };
        for (constraint_id, coefficient) in column {
            let row_idx = constraint_id.inner() as usize;
            if let Some(terms) = terms_by_constraint.get_mut(row_idx) {
                terms.push(PortableLinearTerm {
                    variable_name: variable_name(var_idx),
                    coefficient: *coefficient,
                });
            }
        }
    }

    let constraints = (0..model.num_constraints())
        .filter_map(|idx| {
            let constraint = model.constraint(ConstraintId::new(idx as u32))?;
            Some(
                match (
                    constraint.bounds.lower.is_finite(),
                    constraint.bounds.upper.is_finite(),
                ) {
                    (true, true)
                        if constraint.bounds.lower.to_bits()
                            == constraint.bounds.upper.to_bits() =>
                    {
                        PortableLinearConstraint {
                            name: constraint_name(idx),
                            sense: PortableConstraintSense::Equal,
                            rhs: constraint.bounds.lower,
                            terms: terms_by_constraint[idx].clone(),
                        }
                    }
                    (_, true) => PortableLinearConstraint {
                        name: constraint_name(idx),
                        sense: PortableConstraintSense::LessEqual,
                        rhs: constraint.bounds.upper,
                        terms: terms_by_constraint[idx].clone(),
                    },
                    (true, false) => PortableLinearConstraint {
                        name: constraint_name(idx),
                        sense: PortableConstraintSense::GreaterEqual,
                        rhs: constraint.bounds.lower,
                        terms: terms_by_constraint[idx].clone(),
                    },
                    (false, false) => PortableLinearConstraint {
                        name: constraint_name(idx),
                        sense: PortableConstraintSense::Equal,
                        rhs: 0.0,
                        terms: Vec::new(),
                    },
                },
            )
        })
        .collect();

    let objective = model.objective();
    PortableProblem {
        variable_instances,
        constraints,
        objective: PortableLinearObjective {
            name: "obj".to_string(),
            sense: match objective.sense.unwrap_or(Sense::Minimize) {
                Sense::Minimize => PortableObjectiveSense::Minimize,
                Sense::Maximize => PortableObjectiveSense::Maximize,
            },
            constant: 0.0,
            terms: objective
                .terms
                .iter()
                .map(|(variable_id, coefficient)| PortableLinearTerm {
                    variable_name: variable_name(variable_id.inner() as usize),
                    coefficient: *coefficient,
                })
                .collect(),
        },
        reports: Vec::new(),
    }
}

fn variable_name(index: usize) -> String {
    format!("x{index}")
}

fn constraint_name(index: usize) -> String {
    format!("c{index}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use arco_ir::{PortableLinearObjective, PortableObjectiveSense};
    use arco_model::{Bounds, Constraint, Model, Objective, Sense, Variable};

    #[test]
    fn write_lp_accepts_portable_problem() {
        let problem = PortableProblem {
            variable_instances: Vec::new(),
            constraints: Vec::new(),
            objective: PortableLinearObjective {
                name: "obj".to_string(),
                sense: PortableObjectiveSense::Minimize,
                constant: 0.0,
                terms: Vec::new(),
            },
            reports: Vec::new(),
        };
        let mut output = Vec::new();

        write_lp(&problem, &mut output).expect("portable LP export should succeed");

        assert!(
            String::from_utf8(output)
                .expect("valid utf8")
                .contains("Minimize")
        );
    }

    #[test]
    fn write_lp_accepts_model_view() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("variable");
        let demand = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("constraint");
        model.set_coefficient(x, demand, 1.0).expect("coefficient");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 2.0)],
            })
            .expect("objective");

        let result = export_model_view_lp(&model).expect("model-view LP export");
        let rendered = String::from_utf8(result.bytes).expect("valid utf8");
        assert_eq!(result.format, "lp");
        assert!(rendered.contains("Minimize"));
        assert!(rendered.contains("c0:"));
        assert!(rendered.contains("x0"));
    }
}
