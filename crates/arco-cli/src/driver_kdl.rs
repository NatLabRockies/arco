use crate::driver::DriverError;
use arco_ops::ArcoOps;
use arco_ops::kdl::pipeline::PipelineError;
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

fn pipeline_error_diagnostic(path: &Path, error: &PipelineError) -> KdlDiagnostic {
    let (line, column) = pipeline_error_location(path, error);
    let diagnostic = pipeline_error_inner_diagnostic(error);

    KdlDiagnostic {
        file: path.display().to_string(),
        line,
        column,
        severity: "error",
        message: error.to_string(),
        code: diagnostic.code().map(|code| code.to_string()),
        help: diagnostic.help().map(|help| help.to_string()),
    }
}

fn pipeline_error_inner_diagnostic(error: &PipelineError) -> &dyn Diagnostic {
    match error {
        PipelineError::Source(error) => error,
        PipelineError::Semantic(error) => error,
        PipelineError::Compile(error) => error,
    }
}

fn pipeline_error_location(path: &Path, error: &PipelineError) -> (Option<usize>, Option<usize>) {
    let PipelineError::Source(error) = error else {
        return (None, None);
    };

    let Some(span) = source_error_span(error) else {
        return (None, None);
    };

    span_line_column(path, span)
}

fn source_error_span(error: &arco_ops::kdl::source::SourceError) -> Option<SourceSpan> {
    match error {
        arco_ops::kdl::source::SourceError::MissingNode { span, .. }
        | arco_ops::kdl::source::SourceError::MissingArgument { span, .. }
        | arco_ops::kdl::source::SourceError::MissingProperty { span, .. }
        | arco_ops::kdl::source::SourceError::InvalidValue { span, .. }
        | arco_ops::kdl::source::SourceError::UnsupportedDeclaration { span, .. }
        | arco_ops::kdl::source::SourceError::InvalidAlgebra { span, .. } => Some(*span),
        arco_ops::kdl::source::SourceError::Io { .. }
        | arco_ops::kdl::source::SourceError::Kdl { .. } => None,
    }
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
