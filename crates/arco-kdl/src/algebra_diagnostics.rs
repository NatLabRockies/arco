/// Enhanced error reporting for algebra parse errors.
///
/// This module provides line/column information for algebra parse errors,
/// improving the UX when users make mistakes in constraint/expression formulas.
///
/// Example improved error:
/// ```
/// error: unexpected character `@`
///   ┌─ input.kdl:42:15
///   │
/// 42 │   dispatch[a,t] <= @capacity[a]
///   │               ^^
/// ```

use crate::algebra::ParseError;
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceSpan};
use std::fmt::Display;
use thiserror::Error;

/// An enriched algebra error with source context.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct RichAlgebraError {
    message: String,
    source_text: NamedSource<String>,
    span: SourceSpan,
}

impl Diagnostic for RichAlgebraError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new("arco::algebra::parse_error"))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new("check the algebra syntax in the expression"))
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source_text)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let label = LabeledSpan::new_with_span(
            Some("error occurred here".to_string()),
            self.span,
        );
        Some(Box::new(std::iter::once(label)))
    }
}

/// Convert a byte-offset-based ParseError into a rich diagnostic.
pub fn enrich_algebra_error(
    error: ParseError,
    source: &str,
    filename: &str,
    context_offset: usize, // Offset where the formula starts in the overall file
) -> RichAlgebraError {
    let position = error.position();
    let absolute_offset = context_offset + position;

    // Calculate line and column for better error display
    let (line, column) = offset_to_line_column(source, position);

    // Create a span for the error location (approximate 1-char width)
    let span = SourceSpan::new(absolute_offset.into(), 1usize.into());

    let message = format!("{} at line {}, column {}", error, line, column);

    RichAlgebraError {
        message,
        source_text: NamedSource::new(filename, source.to_string()),
        span,
    }
}

/// Convert byte offset to line and column numbers (1-indexed).
fn offset_to_line_column(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for (i, ch) in text.chars().enumerate() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_column_calculation() {
        let text = "line1\nline2\nline3";
        assert_eq!(offset_to_line_column(text, 0), (1, 1));    // 'l' in line1
        assert_eq!(offset_to_line_column(text, 6), (2, 1));    // 'l' in line2
        assert_eq!(offset_to_line_column(text, 12), (3, 1));  // 'l' in line3
    }
}
