//! Solver configuration types.

/// Configuration options for solver behavior.
///
/// This struct provides a unified way to configure solver parameters
/// across different solver backends.
#[derive(Debug, Clone, Default)]
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
}

impl SolverConfig {
    /// Create a new configuration with all defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the time limit in seconds.
    pub fn with_time_limit(mut self, seconds: f64) -> Self {
        self.time_limit = Some(seconds);
        self
    }

    /// Set the relative MIP gap tolerance.
    pub fn with_mip_gap(mut self, gap: f64) -> Self {
        self.mip_gap = Some(gap);
        self
    }

    /// Set the verbosity level.
    pub fn with_verbosity(mut self, level: u32) -> Self {
        self.verbosity = Some(level);
        self
    }

    /// Enable or disable presolve.
    pub fn with_presolve(mut self, enabled: bool) -> Self {
        self.presolve = Some(enabled);
        self
    }

    /// Set the number of threads.
    pub fn with_threads(mut self, count: u32) -> Self {
        self.threads = Some(count);
        self
    }

    /// Set the feasibility tolerance.
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = Some(tol);
        self
    }

    /// Enable or disable console logging.
    pub fn with_log_to_console(mut self, enabled: bool) -> Self {
        self.log_to_console = Some(enabled);
        self
    }

    /// Check if this configuration is completely empty (all defaults).
    pub fn is_empty(&self) -> bool {
        self.time_limit.is_none()
            && self.mip_gap.is_none()
            && self.verbosity.is_none()
            && self.presolve.is_none()
            && self.threads.is_none()
            && self.tolerance.is_none()
            && self.log_to_console.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new_is_empty() {
        let config = SolverConfig::new();
        assert!(config.is_empty());
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = SolverConfig::new()
            .with_time_limit(60.0)
            .with_mip_gap(0.01)
            .with_verbosity(1)
            .with_presolve(true)
            .with_threads(4)
            .with_tolerance(1e-6)
            .with_log_to_console(false);

        assert!(!config.is_empty());
        assert_eq!(config.time_limit, Some(60.0));
        assert_eq!(config.mip_gap, Some(0.01));
        assert_eq!(config.verbosity, Some(1));
        assert_eq!(config.presolve, Some(true));
        assert_eq!(config.threads, Some(4));
        assert_eq!(config.tolerance, Some(1e-6));
        assert_eq!(config.log_to_console, Some(false));
    }

    #[test]
    fn test_config_partial_is_not_empty() {
        let config = SolverConfig::new().with_time_limit(30.0);
        assert!(!config.is_empty());
        assert_eq!(config.time_limit, Some(30.0));
        assert_eq!(config.mip_gap, None);
    }

    #[test]
    fn test_config_clone() {
        let config1 = SolverConfig::new().with_threads(8);
        let config2 = config1.clone();
        assert_eq!(config1.threads, config2.threads);
    }

    #[test]
    fn test_config_debug() {
        let config = SolverConfig::new().with_time_limit(10.0);
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("time_limit"));
        assert!(debug_str.contains("10.0"));
    }
}
