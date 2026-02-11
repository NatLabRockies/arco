//! Error types for block operations.

/// Error type for block operations.
#[derive(Debug, Clone)]
pub enum BlockError {
    /// Block with this name already exists.
    DuplicateBlock(String),
    /// Block not found.
    BlockNotFound(String),
    /// Cycle detected in block dependencies.
    CycleDetected(String),
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::DuplicateBlock(name) => {
                write!(f, "ARCO_BLOCK_501: Block '{name}' already exists")
            }
            BlockError::BlockNotFound(name) => {
                write!(f, "ARCO_BLOCK_501: Block '{name}' not found")
            }
            BlockError::CycleDetected(msg) => {
                write!(f, "ARCO_BLOCK_503: Cycle detected: {msg}")
            }
        }
    }
}

impl std::error::Error for BlockError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BlockError::DuplicateBlock("MyBlock".to_string());
        assert!(err.to_string().contains("ARCO_BLOCK_501"));
        assert!(err.to_string().contains("MyBlock"));

        let err = BlockError::BlockNotFound("Unknown".to_string());
        assert!(err.to_string().contains("not found"));

        let err = BlockError::CycleDetected("A -> B -> A".to_string());
        assert!(err.to_string().contains("ARCO_BLOCK_503"));
    }
}
