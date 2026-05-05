use arco_ops::{ArcoOps, OpsExportFormat};
use arco_targets::AlgebraicProblem;
use std::io::Write;

pub use arco_ops::ExportError;

pub fn write_lp(problem: &AlgebraicProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    let buffer = ArcoOps::export_problem(problem, OpsExportFormat::Lp)?;
    writer
        .write_all(&buffer)
        .map_err(|source| ExportError::Io { source })
}

pub fn write_mps(problem: &AlgebraicProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    let buffer = ArcoOps::export_problem(problem, OpsExportFormat::Mps)?;
    writer
        .write_all(&buffer)
        .map_err(|source| ExportError::Io { source })
}
