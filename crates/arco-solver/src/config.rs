use std::collections::BTreeMap;

/// Configuration options for solver behavior.
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SolverConfig {
    /// Time limit in seconds. `None` means no limit.
    pub time_limit: Option<f64>,
    /// Relative MIP gap tolerance. `None` uses solver default.
    pub mip_gap: Option<f64>,
    /// Verbosity level. `None` uses solver default.
    pub verbosity: Option<u32>,
    /// Enable/disable presolve. `None` uses solver default.
    pub presolve: Option<bool>,
    /// Number of threads to use. `None` uses solver default.
    pub threads: Option<u32>,
    /// Feasibility tolerance. `None` uses solver default.
    pub tolerance: Option<f64>,
    /// Log solver output to console. `None` uses solver default.
    pub log_to_console: Option<bool>,
    /// Family-specific passthrough parameters.
    #[cfg_attr(feature = "serde", serde(default))]
    pub parameters: BTreeMap<String, String>,
}

impl SolverConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_time_limit(mut self, seconds: f64) -> Self {
        self.time_limit = Some(seconds);
        self
    }

    pub fn with_mip_gap(mut self, gap: f64) -> Self {
        self.mip_gap = Some(gap);
        self
    }

    pub fn with_verbosity(mut self, level: u32) -> Self {
        self.verbosity = Some(level);
        self
    }

    pub fn with_presolve(mut self, enabled: bool) -> Self {
        self.presolve = Some(enabled);
        self
    }

    pub fn with_threads(mut self, count: u32) -> Self {
        self.threads = Some(count);
        self
    }

    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = Some(tol);
        self
    }

    pub fn with_log_to_console(mut self, enabled: bool) -> Self {
        self.log_to_console = Some(enabled);
        self
    }

    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    pub(crate) fn merged_with(&self, overlay: &Self) -> Self {
        let mut merged = self.clone();
        if overlay.time_limit.is_some() {
            merged.time_limit = overlay.time_limit;
        }
        if overlay.mip_gap.is_some() {
            merged.mip_gap = overlay.mip_gap;
        }
        if overlay.verbosity.is_some() {
            merged.verbosity = overlay.verbosity;
        }
        if overlay.presolve.is_some() {
            merged.presolve = overlay.presolve;
        }
        if overlay.threads.is_some() {
            merged.threads = overlay.threads;
        }
        if overlay.tolerance.is_some() {
            merged.tolerance = overlay.tolerance;
        }
        if overlay.log_to_console.is_some() {
            merged.log_to_console = overlay.log_to_console;
        }
        for (key, value) in &overlay.parameters {
            merged.parameters.insert(key.clone(), value.clone());
        }
        merged
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.time_limit.is_none()
            && self.mip_gap.is_none()
            && self.verbosity.is_none()
            && self.presolve.is_none()
            && self.threads.is_none()
            && self.tolerance.is_none()
            && self.log_to_console.is_none()
            && self.parameters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_builder_and_merge_preserve_overlay_precedence() {
        let base = SolverConfig::new().with_threads(4).with_time_limit(10.0);
        let overlay = SolverConfig::new()
            .with_time_limit(20.0)
            .with_parameter("solver.option", "enabled");

        let merged = base.merged_with(&overlay);

        assert_eq!(merged.threads, Some(4));
        assert_eq!(merged.time_limit, Some(20.0));
        assert_eq!(
            merged.parameters.get("solver.option").map(String::as_str),
            Some("enabled")
        );
        assert!(!merged.is_empty());
    }
}
