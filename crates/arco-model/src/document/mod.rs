//! Stable primitive document DTOs.

/// Primitive document kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Model,
    IndexedData,
    Arco,
}

/// Scalar encoding used by a primitive document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    F32,
    F64,
    Decimal,
}

/// Shared document header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentHeader {
    pub schema_version: u32,
    pub kind: DocumentKind,
    pub scalar_type: ScalarType,
}

impl DocumentHeader {
    pub fn v1(kind: DocumentKind, scalar_type: ScalarType) -> Self {
        Self {
            schema_version: 1,
            kind,
            scalar_type,
        }
    }
}

/// Canonical scalar string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalScalar(String);

impl CanonicalScalar {
    pub fn from_f64(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Self(value.to_string()))
        } else if value == f64::INFINITY {
            Some(Self("inf".to_string()))
        } else if value == f64::NEG_INFINITY {
            Some(Self("-inf".to_string()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable model document shell. Concrete rows are introduced as model consumers migrate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDocument {
    pub header: DocumentHeader,
    pub fingerprint: Option<u64>,
}

impl ModelDocument {
    pub fn new_f64() -> Self {
        Self {
            header: DocumentHeader::v1(DocumentKind::Model, ScalarType::F64),
            fingerprint: None,
        }
    }
}

/// Stable indexed-data document shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedDataDocument {
    pub header: DocumentHeader,
}

impl IndexedDataDocument {
    pub fn new_f64() -> Self {
        Self {
            header: DocumentHeader::v1(DocumentKind::IndexedData, ScalarType::F64),
        }
    }
}

/// Top-level primitive document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcoDocument {
    pub header: DocumentHeader,
    pub model: Option<ModelDocument>,
    pub indexed_data: Option<IndexedDataDocument>,
}

impl ArcoDocument {
    pub fn new_f64() -> Self {
        Self {
            header: DocumentHeader::v1(DocumentKind::Arco, ScalarType::F64),
            model: None,
            indexed_data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::document::{ArcoDocument, CanonicalScalar, DocumentKind};

    #[test]
    fn documents_carry_kind_and_schema_version() {
        let doc = ArcoDocument::new_f64();
        assert_eq!(doc.header.schema_version, 1);
        assert_eq!(doc.header.kind, DocumentKind::Arco);
    }

    #[test]
    fn scalar_strings_reject_nan() {
        assert!(CanonicalScalar::from_f64(f64::NAN).is_none());
        assert_eq!(
            CanonicalScalar::from_f64(f64::INFINITY).unwrap().as_str(),
            "inf"
        );
    }
}
