//! Format-neutral diagnostics primitives for Arco.
//!
//! This crate is intentionally independent of authoring formats. KDL, CLI, and
//! Python layers can attach their own rendering while sharing stable codes,
//! severities, spans, and coarse provenance.

/// Stable diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiagnosticCode(pub &'static str);

impl DiagnosticCode {
    pub const fn new(code: &'static str) -> Self {
        Self(code)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

/// Stable identifier for an input or generated source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Byte-oriented source span. Line/column rendering belongs to authoring layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub start: u32,
    pub end: u32,
}

impl SourceSpan {
    pub fn new(start: u32, end: u32) -> Option<Self> {
        (start <= end).then_some(Self { start, end })
    }
}

/// Coarse origin for a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Provenance {
    UserInput {
        source: SourceId,
        span: Option<SourceSpan>,
    },
    Generated {
        phase: &'static str,
    },
    External {
        system: &'static str,
    },
}

/// Format-neutral diagnostic item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub provenance: Option<Provenance>,
}

impl Diagnostic {
    pub fn new(code: DiagnosticCode, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            provenance: None,
        }
    }

    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

/// Collection of diagnostics produced by one pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiagnosticReport {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Diagnostic, DiagnosticCode, DiagnosticReport, Severity, SourceSpan};

    #[test]
    fn source_span_rejects_reversed_ranges() {
        assert!(SourceSpan::new(2, 1).is_none());
        assert_eq!(SourceSpan::new(1, 2).unwrap().start, 1);
    }

    #[test]
    fn report_tracks_error_presence() {
        let mut report = DiagnosticReport::new();
        report.push(Diagnostic::new(
            DiagnosticCode::new("ARCO_TEST"),
            Severity::Warning,
            "warn",
        ));
        assert!(!report.has_errors());
        report.push(Diagnostic::new(
            DiagnosticCode::new("ARCO_TEST_ERR"),
            Severity::Error,
            "err",
        ));
        assert!(report.has_errors());
    }
}
