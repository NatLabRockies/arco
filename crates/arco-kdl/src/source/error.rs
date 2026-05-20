use arco_diagnostics::codes;
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode, SourceSpan};
use std::fmt::Display;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse kdl {path}: {source}")]
    Kdl {
        path: PathBuf,
        #[source]
        source: kdl::KdlError,
    },
    #[error("missing required node `{name}` in {path}")]
    MissingNode {
        name: &'static str,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("missing required argument {index} on node `{node}` in {path}")]
    MissingArgument {
        node: String,
        index: usize,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("missing required property `{property}` on node `{node}` in {path}")]
    MissingProperty {
        node: String,
        property: &'static str,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("unexpected value for `{field}` on node `{node}` in {path}")]
    InvalidValue {
        node: String,
        field: String,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("unsupported declaration `{name}` in {path}")]
    UnsupportedDeclaration {
        name: String,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("invalid include in {path}: {reason}")]
    InvalidInclude {
        reason: String,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("invalid algebra in `{node}` in {path}: {reason}")]
    InvalidAlgebra {
        node: String,
        reason: String,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
}

impl Diagnostic for SourceError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        let code = match self {
            Self::Io { .. } => codes::SOURCE_IO,
            Self::Kdl { .. } => codes::SOURCE_KDL,
            Self::MissingNode { .. } => codes::SOURCE_MISSING_NODE,
            Self::MissingArgument { .. } => codes::SOURCE_MISSING_ARGUMENT,
            Self::MissingProperty { .. } => codes::SOURCE_MISSING_PROPERTY,
            Self::InvalidValue { .. } => codes::SOURCE_INVALID_VALUE,
            Self::UnsupportedDeclaration { .. } => codes::SOURCE_UNSUPPORTED_DECLARATION,
            Self::InvalidInclude { .. } => codes::SOURCE_INVALID_INCLUDE,
            Self::InvalidAlgebra { .. } => codes::SOURCE_INVALID_ALGEBRA,
        };
        Some(Box::new(code))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match self {
            Self::MissingNode { name, .. } => Some(Box::new(format!(
                "add a `{name}` child declaration to this block"
            ))),
            Self::MissingArgument { index, .. } => Some(Box::new(format!(
                "add argument {index} to this declaration"
            ))),
            Self::MissingProperty { property, .. } => Some(Box::new(format!(
                "add a `{property}` property to this declaration"
            ))),
            Self::InvalidValue { field, .. } => Some(Box::new(format!(
                "replace `{field}` with a value of the expected type"
            ))),
            Self::UnsupportedDeclaration { .. } => Some(Box::new(
                "remove the declaration or add parser support for it",
            )),
            Self::InvalidInclude { .. } => Some(Box::new(
                "use `include \"path.kdl\"` only in the entrypoint file at top level or inside a model block",
            )),
            Self::InvalidAlgebra { .. } => Some(Box::new(
                "fix the algebra syntax so the expression can be parsed into the DSL AST",
            )),
            Self::Io { .. } | Self::Kdl { .. } => None,
        }
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        match self {
            Self::MissingNode { source_text, .. }
            | Self::MissingArgument { source_text, .. }
            | Self::MissingProperty { source_text, .. }
            | Self::InvalidValue { source_text, .. }
            | Self::UnsupportedDeclaration { source_text, .. }
            | Self::InvalidInclude { source_text, .. }
            | Self::InvalidAlgebra { source_text, .. } => Some(source_text.as_ref()),
            Self::Io { .. } | Self::Kdl { .. } => None,
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let labeled = match self {
            Self::MissingNode { span, .. }
            | Self::MissingArgument { span, .. }
            | Self::MissingProperty { span, .. }
            | Self::InvalidValue { span, .. }
            | Self::UnsupportedDeclaration { span, .. }
            | Self::InvalidInclude { span, .. }
            | Self::InvalidAlgebra { span, .. } => Some(LabeledSpan::new_with_span(
                Some("this declaration".to_string()),
                *span,
            )),
            Self::Io { .. } | Self::Kdl { .. } => None,
        }?;
        Some(Box::new(std::iter::once(labeled)))
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            Self::Kdl { source, .. } => Some(source),
            Self::Io { .. }
            | Self::MissingNode { .. }
            | Self::MissingArgument { .. }
            | Self::MissingProperty { .. }
            | Self::InvalidValue { .. }
            | Self::UnsupportedDeclaration { .. }
            | Self::InvalidInclude { .. }
            | Self::InvalidAlgebra { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SourceError;
    use miette::Diagnostic;
    use miette::{NamedSource, SourceOffset};
    use std::path::PathBuf;

    #[test]
    fn unsupported_declaration_exposes_diagnostic_code() {
        let source = NamedSource::new("test.kdl", "model X {}".to_string());
        let error = SourceError::UnsupportedDeclaration {
            name: "legacy_decl".to_string(),
            path: PathBuf::from("test.kdl"),
            source_text: Box::new(source),
            span: (SourceOffset::from(0), 1).into(),
        };

        let code = error.code().expect("diagnostic code").to_string();
        assert_eq!(code, "arco::source::unsupported_declaration");
    }
}
