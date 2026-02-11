//! Human-readable ASCII model formatting.

use std::collections::HashSet;
use std::fmt::Write as _;

use arco_expr::ids::{ConstraintId, VariableId};

use crate::model::Model;
use crate::types::{Bounds, Sense};

const FLOAT_EQ_EPSILON: f64 = 1e-12;
const PREVIEW_CONSTRAINTS: usize = 20;
const PREVIEW_TERMS: usize = 30;
const PREVIEW_DOMAIN_ITEMS: usize = 20;

/// Formatting controls for pretty-print output.
#[derive(Debug, Clone, Copy)]
pub struct PrettyPrintOptions {
    /// Maximum number of constraints to render.
    pub constraints: Option<usize>,
    /// Maximum number of terms to render per linear expression.
    pub terms: Option<usize>,
    /// Maximum number of domain or bounds items to show in grouped sections.
    pub domain_items: Option<usize>,
}

impl PrettyPrintOptions {
    /// Preview mode used by terse displays.
    pub fn preview() -> Self {
        Self {
            constraints: Some(PREVIEW_CONSTRAINTS),
            terms: Some(PREVIEW_TERMS),
            domain_items: Some(PREVIEW_DOMAIN_ITEMS),
        }
    }

    /// Full mode with no truncation.
    pub fn full() -> Self {
        Self {
            constraints: None,
            terms: None,
            domain_items: None,
        }
    }
}

/// Optional section inserted before `s.t.`.
#[derive(Debug, Clone)]
pub struct PrettySection {
    pub heading: String,
    pub entries: Vec<String>,
}

/// Grouped bounds line that can cover multiple variables.
#[derive(Debug, Clone)]
pub struct PrettyBoundGroup {
    pub text: String,
    pub vars: Vec<VariableId>,
}

/// Adapter hook for binding-specific labels and sections.
pub trait PrettyPrintAdapter {
    /// Optional variable label override.
    fn variable_label(&self, _model: &Model, _var_id: VariableId) -> Option<String> {
        None
    }

    /// Optional constraint label override.
    fn constraint_label(&self, _model: &Model, _constraint_id: ConstraintId) -> Option<String> {
        None
    }

    /// Extra sections shown before constraints.
    fn sections(&self, _model: &Model) -> Vec<PrettySection> {
        Vec::new()
    }

    /// Grouped bounds lines and covered variables.
    fn grouped_bounds(&self, _model: &Model) -> Vec<PrettyBoundGroup> {
        Vec::new()
    }
}

/// Default adapter used when no binding metadata is available.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPrettyPrintAdapter;

impl PrettyPrintAdapter for DefaultPrettyPrintAdapter {}

struct ConstraintRenderLine {
    lhs: String,
    op: &'static str,
    rhs: String,
}

impl Model {
    /// Render the model to ASCII using default labels.
    pub fn format_ascii(&self, options: PrettyPrintOptions) -> String {
        self.format_ascii_with_adapter(&DefaultPrettyPrintAdapter, options)
    }

    /// Render the model to ASCII using a custom adapter.
    pub fn format_ascii_with_adapter<A: PrettyPrintAdapter>(
        &self,
        adapter: &A,
        options: PrettyPrintOptions,
    ) -> String {
        let mut lines = Vec::new();
        lines.push(self.render_objective_line(adapter, options.terms));
        lines.push(String::new());

        for section in adapter.sections(self) {
            if section.entries.is_empty() {
                continue;
            }
            lines.push(format!("{}:", section.heading));
            for entry in section.entries {
                lines.push(format!(" {entry}"));
            }
            lines.push(String::new());
        }

        lines.push("s.t.".to_string());

        let rows = self.rows();
        let total_constraints = self.num_constraints();
        let constraint_limit = options
            .constraints
            .unwrap_or(total_constraints)
            .min(total_constraints);

        if constraint_limit == 0 {
            lines.push(" (none)".to_string());
        } else {
            let mut rendered_constraints = Vec::with_capacity(constraint_limit);
            for constraint_idx in 0..constraint_limit {
                let constraint_id = ConstraintId::new(constraint_idx as u32);
                let row = rows.get(constraint_idx).map_or(&[][..], Vec::as_slice);
                let mut lhs = self.format_linear_expression(row, options.terms, adapter);
                if let Some(label) = self.resolve_constraint_label(adapter, constraint_id) {
                    lhs = format!("{label}: {lhs}");
                }
                if let Ok(constraint) = self.get_constraint(constraint_id) {
                    rendered_constraints.push(Self::render_constraint_line(lhs, constraint.bounds));
                }
            }

            let lhs_width = rendered_constraints
                .iter()
                .map(|entry: &ConstraintRenderLine| entry.lhs.len())
                .max()
                .unwrap_or(0);
            for entry in rendered_constraints {
                lines.push(format!(
                    " {:lhs_width$} {:>2} {}",
                    entry.lhs,
                    entry.op,
                    entry.rhs,
                    lhs_width = lhs_width
                ));
            }
        }

        if constraint_limit < total_constraints {
            lines.push(format!(
                " ... ({} more constraints)",
                total_constraints - constraint_limit
            ));
        }

        let mut binary_vars = Vec::new();
        let mut integer_vars = Vec::new();
        let mut bounds_lines = Vec::new();
        let mut covered_vars = HashSet::new();
        for group in adapter.grouped_bounds(self) {
            bounds_lines.push(group.text);
            covered_vars.extend(group.vars);
        }

        for var_idx in 0..self.num_variables() {
            let var_id = VariableId::new(var_idx as u32);
            let Ok(var) = self.get_variable(var_id) else {
                continue;
            };
            let label = self.resolve_variable_label(adapter, var_id);
            if is_binary_variable(var.bounds, var.is_integer) {
                binary_vars.push(label);
                continue;
            }
            if var.is_integer {
                integer_vars.push(label.clone());
            }
            if covered_vars.contains(&var_id) {
                continue;
            }
            if let Some(line) = format_variable_bounds_line(&label, var.bounds) {
                bounds_lines.push(line);
            }
        }

        let has_domains =
            !binary_vars.is_empty() || !integer_vars.is_empty() || !bounds_lines.is_empty();
        if has_domains {
            lines.push(String::new());
        }
        if !binary_vars.is_empty() {
            lines.push(format_variable_group_line(
                "Binary",
                &binary_vars,
                options.domain_items,
            ));
        }
        if !integer_vars.is_empty() {
            lines.push(format_variable_group_line(
                "Integer",
                &integer_vars,
                options.domain_items,
            ));
        }
        if !bounds_lines.is_empty() {
            lines.push("Bounds:".to_string());
            let bounds_limit = options
                .domain_items
                .unwrap_or(bounds_lines.len())
                .min(bounds_lines.len());
            for bound_line in bounds_lines.iter().take(bounds_limit) {
                lines.push(format!(" {bound_line}"));
            }
            if bounds_limit < bounds_lines.len() {
                lines.push(format!(
                    " ... ({} more bounds)",
                    bounds_lines.len() - bounds_limit
                ));
            }
        }

        lines.join("\n")
    }

    fn render_objective_line<A: PrettyPrintAdapter>(
        &self,
        adapter: &A,
        max_terms: Option<usize>,
    ) -> String {
        let objective = self.objective();
        let Some(sense) = objective.sense else {
            return "Objective: (not set)".to_string();
        };

        let sense_label = match sense {
            Sense::Minimize => "Min",
            Sense::Maximize => "Max",
        };
        let expr = self.format_linear_expression(&objective.terms, max_terms, adapter);
        if let Some(name) = self.get_objective_name() {
            format!("{sense_label} {name}: {expr}")
        } else {
            format!("{sense_label} {expr}")
        }
    }

    fn render_constraint_line(lhs: String, bounds: Bounds) -> ConstraintRenderLine {
        if float_approx_equal(bounds.lower, bounds.upper) {
            return ConstraintRenderLine {
                lhs,
                op: "=",
                rhs: format_ascii_number(bounds.upper),
            };
        }
        if bounds.lower.is_infinite() && bounds.lower.is_sign_negative() && bounds.upper.is_finite()
        {
            return ConstraintRenderLine {
                lhs,
                op: "<=",
                rhs: format_ascii_number(bounds.upper),
            };
        }
        if bounds.upper.is_infinite() && bounds.upper.is_sign_positive() && bounds.lower.is_finite()
        {
            return ConstraintRenderLine {
                lhs,
                op: ">=",
                rhs: format_ascii_number(bounds.lower),
            };
        }
        if bounds.lower.is_finite() && bounds.upper.is_finite() {
            return ConstraintRenderLine {
                lhs,
                op: "in",
                rhs: format!(
                    "[{}, {}]",
                    format_ascii_number(bounds.lower),
                    format_ascii_number(bounds.upper)
                ),
            };
        }
        ConstraintRenderLine {
            lhs,
            op: "  ",
            rhs: "free".to_string(),
        }
    }

    fn format_linear_expression<A: PrettyPrintAdapter>(
        &self,
        terms: &[(VariableId, f64)],
        max_terms: Option<usize>,
        adapter: &A,
    ) -> String {
        let nonzero_terms: Vec<(VariableId, f64)> = terms
            .iter()
            .copied()
            .filter(|(_, coeff)| !float_approx_equal(*coeff, 0.0))
            .collect();
        if nonzero_terms.is_empty() {
            return "0".to_string();
        }

        let term_limit = max_terms
            .unwrap_or(nonzero_terms.len())
            .min(nonzero_terms.len());
        let mut rendered = String::new();

        for (idx, (var_id, coeff)) in nonzero_terms.iter().take(term_limit).enumerate() {
            let negative = *coeff < 0.0;
            let abs_coeff = coeff.abs();
            let label = self.resolve_variable_label(adapter, *var_id);
            let term_body = if float_approx_equal(abs_coeff, 1.0) {
                label
            } else {
                format!("{} {label}", format_ascii_number(abs_coeff))
            };

            if idx == 0 {
                if negative {
                    rendered.push('-');
                }
                rendered.push_str(&term_body);
            } else if negative {
                let _ = write!(rendered, " - {term_body}");
            } else {
                let _ = write!(rendered, " + {term_body}");
            }
        }

        if term_limit < nonzero_terms.len() {
            let _ = write!(
                rendered,
                " + ... ({} more terms)",
                nonzero_terms.len() - term_limit
            );
        }

        rendered
    }

    fn resolve_variable_label<A: PrettyPrintAdapter>(
        &self,
        adapter: &A,
        var_id: VariableId,
    ) -> String {
        if let Some(label) = adapter.variable_label(self, var_id) {
            return label;
        }
        self.get_variable_name(var_id)
            .map_or_else(|| format!("x[{}]", var_id.inner() + 1), ToString::to_string)
    }

    fn resolve_constraint_label<A: PrettyPrintAdapter>(
        &self,
        adapter: &A,
        constraint_id: ConstraintId,
    ) -> Option<String> {
        if let Some(label) = adapter.constraint_label(self, constraint_id) {
            return Some(label);
        }
        self.get_constraint_name(constraint_id)
            .map(ToString::to_string)
    }
}

/// Shared numeric formatter for ASCII pretty-print output.
pub fn format_ascii_number(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }

    let normalized = if value.to_bits() == (-0.0_f64).to_bits() {
        0.0
    } else {
        value
    };
    let mut rendered = format!("{normalized:.12}");
    while rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.pop();
    }
    if rendered == "-0" {
        "0".to_string()
    } else {
        rendered
    }
}

fn float_approx_equal(lhs: f64, rhs: f64) -> bool {
    if lhs.to_bits() == rhs.to_bits() {
        return true;
    }
    if !lhs.is_finite() || !rhs.is_finite() {
        return false;
    }
    let scale = lhs.abs().max(rhs.abs()).max(1.0);
    (lhs - rhs).abs() <= FLOAT_EQ_EPSILON * scale
}

fn is_binary_variable(bounds: Bounds, is_integer: bool) -> bool {
    is_integer && float_approx_equal(bounds.lower, 0.0) && float_approx_equal(bounds.upper, 1.0)
}

fn format_variable_group_line(
    label: &str,
    variables: &[String],
    max_items: Option<usize>,
) -> String {
    let limit = max_items.unwrap_or(variables.len()).min(variables.len());
    let mut line = String::new();
    let _ = write!(line, "{label}: ");
    if limit > 0 {
        line.push_str(&variables[..limit].join(", "));
    }
    if limit < variables.len() {
        if limit > 0 {
            line.push_str(", ");
        }
        let _ = write!(line, "... ({} more)", variables.len() - limit);
    }
    line
}

fn format_variable_bounds_line(label: &str, bounds: Bounds) -> Option<String> {
    let lower_finite = bounds.lower.is_finite();
    let upper_finite = bounds.upper.is_finite();
    if !lower_finite && !upper_finite {
        return None;
    }

    if lower_finite && upper_finite {
        return Some(format!(
            "{} <= {label} <= {}",
            format_ascii_number(bounds.lower),
            format_ascii_number(bounds.upper)
        ));
    }
    if lower_finite {
        return Some(format!("{} <= {label}", format_ascii_number(bounds.lower)));
    }
    Some(format!("{label} <= {}", format_ascii_number(bounds.upper)))
}

#[cfg(test)]
mod tests {
    use crate::model::{
        Model, PrettyBoundGroup, PrettyPrintAdapter, PrettyPrintOptions, PrettySection,
    };
    use crate::types::{Bounds, Constraint, Objective, Sense, Variable};
    use arco_expr::ids::{ConstraintId, VariableId};

    struct LabelAdapter;

    impl PrettyPrintAdapter for LabelAdapter {
        fn variable_label(&self, _model: &Model, var_id: VariableId) -> Option<String> {
            Some(format!("gen[{}]", var_id.inner()))
        }

        fn constraint_label(&self, _model: &Model, constraint_id: ConstraintId) -> Option<String> {
            Some(format!("c[{}]", constraint_id.inner()))
        }

        fn sections(&self, _model: &Model) -> Vec<PrettySection> {
            vec![PrettySection {
                heading: "Index sets".to_string(),
                entries: vec!["T = [0, 1]".to_string()],
            }]
        }

        fn grouped_bounds(&self, _model: &Model) -> Vec<PrettyBoundGroup> {
            vec![PrettyBoundGroup {
                text: "0 <= gen[t] <= 10  for t in T".to_string(),
                vars: vec![VariableId::new(0), VariableId::new(1)],
            }]
        }
    }

    #[test]
    fn format_ascii_supports_adapter_labels_and_sections() {
        let mut model = Model::new();
        let x0 = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .expect("var0");
        let x1 = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .expect("var1");
        let c = model
            .add_constraint(Constraint {
                bounds: Bounds::new(f64::NEG_INFINITY, 5.0),
            })
            .expect("constraint");
        model.set_coefficient(x0, c, 1.0).expect("coeff0");
        model.set_coefficient(x1, c, 2.0).expect("coeff1");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x0, 1.0), (x1, 3.0)],
            })
            .expect("objective");

        let rendered = model.format_ascii_with_adapter(&LabelAdapter, PrettyPrintOptions::full());
        assert!(rendered.contains("Min gen[0] + 3 gen[1]"));
        assert!(rendered.contains("Index sets:"));
        assert!(rendered.contains("s.t."));
        assert!(rendered.contains("c[0]: gen[0] + 2 gen[1] <= 5"));
        assert!(rendered.contains("0 <= gen[t] <= 10  for t in T"));
    }

    #[test]
    fn format_ascii_preview_truncates_constraints() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("var");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 1.0)],
            })
            .expect("objective");
        for rhs in 0..25 {
            let c = model
                .add_constraint(Constraint {
                    bounds: Bounds::new(f64::NEG_INFINITY, rhs as f64),
                })
                .expect("constraint");
            model.set_coefficient(x, c, 1.0).expect("coeff");
        }

        let rendered = model.format_ascii(PrettyPrintOptions::preview());
        assert!(rendered.contains("... (5 more constraints)"));
    }
}
