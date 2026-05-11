use crate::driver::DriverError;
use arco_ops::{ArcoOps, OpsCompileError};
use miette::{Diagnostic, SourceSpan};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize, PartialEq)]
pub struct KdlCheckOutcome {
    pub valid: bool,
    pub json: String,
}

#[derive(Debug, Serialize, PartialEq)]
struct KdlCheckReport {
    valid: bool,
    diagnostics: Vec<KdlDiagnostic>,
}

#[derive(Debug, Serialize, PartialEq)]
struct KdlDiagnostic {
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<usize>,
    severity: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<String>,
}

pub fn kdl_check_file_json(path: &Path) -> Result<KdlCheckOutcome, DriverError> {
    let report = match ArcoOps::check_file(path) {
        Ok(_) => KdlCheckReport {
            valid: true,
            diagnostics: Vec::new(),
        },
        Err(error) => KdlCheckReport {
            valid: false,
            diagnostics: vec![pipeline_error_diagnostic(path, &error)],
        },
    };

    let valid = report.valid;
    let json = serde_json::to_string(&report).map_err(|source| DriverError::Json {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(KdlCheckOutcome { valid, json })
}

fn pipeline_error_diagnostic(path: &Path, error: &OpsCompileError) -> KdlDiagnostic {
    let diagnostic_path = pipeline_error_path(path, error);
    let (line, column) = pipeline_error_location(diagnostic_path, error);
    let diagnostic = pipeline_error_inner_diagnostic(error);

    KdlDiagnostic {
        file: diagnostic_path.display().to_string(),
        line,
        column,
        severity: "error",
        message: error.to_string(),
        code: diagnostic.code().map(|code| code.to_string()),
        help: diagnostic.help().map(|help| help.to_string()),
    }
}

fn pipeline_error_path<'a>(default: &'a Path, error: &'a OpsCompileError) -> &'a Path {
    let OpsCompileError::Source(error) = error else {
        return default;
    };

    match error {
        arco_ops::OpsSourceError::Io { path, .. }
        | arco_ops::OpsSourceError::Kdl { path, .. }
        | arco_ops::OpsSourceError::MissingNode { path, .. }
        | arco_ops::OpsSourceError::MissingArgument { path, .. }
        | arco_ops::OpsSourceError::MissingProperty { path, .. }
        | arco_ops::OpsSourceError::InvalidValue { path, .. }
        | arco_ops::OpsSourceError::UnsupportedDeclaration { path, .. }
        | arco_ops::OpsSourceError::InvalidInclude { path, .. }
        | arco_ops::OpsSourceError::InvalidAlgebra { path, .. } => path.as_path(),
    }
}

fn pipeline_error_inner_diagnostic(error: &OpsCompileError) -> &dyn Diagnostic {
    match error {
        OpsCompileError::Source(error) => error,
        OpsCompileError::Semantic(error) => error,
        OpsCompileError::Compile(error) => error,
    }
}

fn pipeline_error_location(path: &Path, error: &OpsCompileError) -> (Option<usize>, Option<usize>) {
    let OpsCompileError::Source(error) = error else {
        return (None, None);
    };

    let Some(span) = arco_ops::source_error_span(error) else {
        return (None, None);
    };

    span_line_column(path, span)
}

pub(crate) fn span_line_column(path: &Path, span: SourceSpan) -> (Option<usize>, Option<usize>) {
    let Ok(source) = std::fs::read_to_string(path) else {
        return (None, None);
    };
    let mut offset = span.offset().min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }

    let prefix = &source[..offset];
    let line = prefix
        .chars()
        .filter(|character| *character == '\n')
        .count()
        + 1;
    let column = prefix
        .rsplit('\n')
        .next()
        .map_or(1, |line_prefix| line_prefix.chars().count() + 1);

    (Some(line), Some(column))
}
