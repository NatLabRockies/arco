//! Development tools and memory instrumentation for Arco.
//!
//! This crate provides utilities for monitoring and profiling Arco applications,
//! including memory usage tracking across different optimization stages.

pub mod memory;

pub use memory::{
    MeasurementRecorder, MemoryProbe, MemorySnapshot, StageMeasurement, StageStart,
    capture_rss_bytes, rss_delta,
};
