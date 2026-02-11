//! Memory instrumentation and tracking for Arco optimization stages.
//!
//! This module provides utilities to capture, track, and analyze memory usage
//! across different stages of the optimization process.

use std::time::{Duration, Instant};
use sysinfo::System;

/// A snapshot of memory state at a specific point in time.
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Resident set size in bytes
    pub rss_bytes: u64,
    /// Timestamp when this snapshot was captured
    pub timestamp: Instant,
    /// Name of the stage (e.g., "declare", "optimize")
    pub stage: String,
}

/// Errors produced by memory instrumentation.
#[derive(Debug, Clone)]
pub enum MemoryError {
    ProcessNotFound { pid: u32 },
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::ProcessNotFound { pid } => {
                write!(f, "failed to locate process {}", pid)
            }
        }
    }
}

impl std::error::Error for MemoryError {}

fn read_process_rss_bytes() -> Result<u64, MemoryError> {
    let pid = sysinfo::Pid::from(std::process::id() as usize);

    // Only refresh the specific process we care about, not the entire system.
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[pid]),
        true,
        sysinfo::ProcessRefreshKind::nothing().with_memory(),
    );

    let process = sys.process(pid).ok_or(MemoryError::ProcessNotFound {
        pid: std::process::id(),
    })?;

    // sysinfo 0.33+ returns memory in bytes directly.
    Ok(process.memory())
}

impl MemorySnapshot {
    /// Capture current memory state for a given stage.
    ///
    /// # Errors
    ///
    /// Returns an error if the current process cannot be located.
    pub fn capture(stage: &str) -> Result<Self, MemoryError> {
        let rss_bytes = read_process_rss_bytes()?;

        Ok(MemorySnapshot {
            rss_bytes,
            timestamp: Instant::now(),
            stage: stage.to_string(),
        })
    }

    /// Calculate the difference between this snapshot and another.
    ///
    /// Returns the difference in RSS bytes (positive means growth).
    pub fn diff(&self, other: &Self) -> i64 {
        self.rss_bytes as i64 - other.rss_bytes as i64
    }
}

/// Capture RSS bytes for the current process.
pub fn capture_rss_bytes(_stage: &str) -> Option<u64> {
    read_process_rss_bytes().ok()
}

/// Compute RSS delta between two optional snapshots.
pub fn rss_delta(before: Option<u64>, after: Option<u64>) -> Option<i64> {
    match (before, after) {
        (Some(before), Some(after)) => Some(after as i64 - before as i64),
        _ => None,
    }
}

/// Marker returned by [`MeasurementRecorder::begin_stage`].
#[derive(Debug, Clone)]
pub struct StageStart {
    stage: String,
    started_at: Instant,
    rss_before_bytes: Option<u64>,
}

/// Memory and timing data for a stage.
#[derive(Debug, Clone)]
pub struct StageMeasurement {
    /// Stage name.
    pub stage: String,
    /// Stage duration.
    pub duration: Duration,
    /// RSS before the stage.
    pub rss_before_bytes: Option<u64>,
    /// RSS after the stage.
    pub rss_after_bytes: Option<u64>,
    /// RSS delta between after and before.
    pub rss_delta_bytes: Option<i64>,
}

/// Recorder for stage-level timing and memory metrics.
#[derive(Debug, Default)]
pub struct MeasurementRecorder {
    stages: Vec<StageMeasurement>,
}

impl MeasurementRecorder {
    /// Create a new recorder.
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Capture stage start timing and memory.
    pub fn begin_stage(&self, stage: &str) -> StageStart {
        StageStart {
            stage: stage.to_string(),
            started_at: Instant::now(),
            rss_before_bytes: capture_rss_bytes(stage),
        }
    }

    /// Capture stage end timing and memory and append a stage measurement.
    pub fn end_stage(&mut self, start: StageStart) {
        let rss_after_bytes = capture_rss_bytes(&start.stage);
        let measurement = StageMeasurement {
            stage: start.stage,
            duration: start.started_at.elapsed(),
            rss_before_bytes: start.rss_before_bytes,
            rss_after_bytes,
            rss_delta_bytes: rss_delta(start.rss_before_bytes, rss_after_bytes),
        };
        self.stages.push(measurement);
    }

    /// Return all captured stage measurements in order.
    pub fn stages(&self) -> &[StageMeasurement] {
        &self.stages
    }
}

/// A probe for tracking memory usage across multiple stages.
#[derive(Debug)]
pub struct MemoryProbe {
    snapshots: Vec<MemorySnapshot>,
}

impl MemoryProbe {
    /// Create a new memory probe.
    pub fn new() -> Self {
        MemoryProbe {
            snapshots: Vec::new(),
        }
    }

    /// Record a memory snapshot for a stage.
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot could not be captured.
    pub fn record(&mut self, stage: &str) -> Result<(), MemoryError> {
        let snapshot = MemorySnapshot::capture(stage)?;
        self.snapshots.push(snapshot);
        Ok(())
    }

    /// Get all recorded snapshots.
    pub fn snapshots(&self) -> &[MemorySnapshot] {
        &self.snapshots
    }

    /// Get the difference between the last two snapshots.
    pub fn last_diff(&self) -> Option<i64> {
        if self.snapshots.len() < 2 {
            return None;
        }
        let last = &self.snapshots[self.snapshots.len() - 1];
        let prev = &self.snapshots[self.snapshots.len() - 2];
        Some(last.diff(prev))
    }
}

impl Default for MemoryProbe {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{
        MeasurementRecorder, MemoryProbe, MemorySnapshot, capture_rss_bytes, rss_delta,
    };
    use std::time::Instant;

    #[test]
    fn test_memory_snapshot_capture() {
        let snapshot =
            MemorySnapshot::capture("test_stage").unwrap_or_else(|err| panic!("{}", err));
        assert_eq!(snapshot.stage, "test_stage");
        assert!(snapshot.rss_bytes > 0);
    }

    #[test]
    fn test_memory_snapshot_diff() {
        let snapshot1 = MemorySnapshot {
            rss_bytes: 1000,
            timestamp: Instant::now(),
            stage: "stage1".to_string(),
        };

        let snapshot2 = MemorySnapshot {
            rss_bytes: 1500,
            timestamp: Instant::now(),
            stage: "stage2".to_string(),
        };

        let diff = snapshot2.diff(&snapshot1);
        assert_eq!(diff, 500);
    }

    #[test]
    fn test_memory_probe() {
        let mut probe = MemoryProbe::new();
        probe
            .record("stage1")
            .unwrap_or_else(|err| panic!("{}", err));
        probe
            .record("stage2")
            .unwrap_or_else(|err| panic!("{}", err));

        assert_eq!(probe.snapshots().len(), 2);
        assert_eq!(probe.snapshots()[0].stage, "stage1");
        assert_eq!(probe.snapshots()[1].stage, "stage2");

        let diff = probe.last_diff();
        assert!(diff.is_some());
    }

    #[test]
    fn test_rss_delta() {
        assert_eq!(rss_delta(Some(100), Some(250)), Some(150));
        assert_eq!(rss_delta(Some(100), None), None);
        assert_eq!(rss_delta(None, Some(250)), None);
    }

    #[test]
    fn test_capture_rss_bytes() {
        let rss = capture_rss_bytes("test");
        assert!(rss.is_some());
    }

    #[test]
    fn test_measurement_recorder() {
        let mut recorder = MeasurementRecorder::new();
        let start = recorder.begin_stage("stage_a");
        recorder.end_stage(start);

        assert_eq!(recorder.stages().len(), 1);
        assert_eq!(recorder.stages()[0].stage, "stage_a");
        assert!(recorder.stages()[0].duration.as_nanos() > 0);
    }
}
