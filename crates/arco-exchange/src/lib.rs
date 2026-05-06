//! Exchange seam for portable Arco IR.

use std::io::Write;

pub use arco_export::ExportError;
use arco_ir::PortableProblem;

pub fn write_lp(problem: &PortableProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    arco_export::write_portable_lp(problem, writer)
}

pub fn write_mps(problem: &PortableProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    arco_export::write_portable_mps(problem, writer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arco_ir::{PortableLinearObjective, PortableObjectiveSense};

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
}
