//! Python bindings for Arco optimization using PyO3.
//!
//! This crate exposes Arco's model builder and solver to Python with zero-copy access
//! to solution data through memoryview.

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../python/src/core.rs"
));
