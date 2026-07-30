use std::collections::BTreeMap;

/// Solver-independent algorithm for linear programs and MIP relaxations.
///
/// Backends translate these semantic choices into their native controls. If a
/// backend cannot represent a selected algorithm, it returns an invalid-setting
/// error rather than silently choosing something else.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum LpAlgorithm {
    /// Let the backend select its default LP algorithm.
    #[default]
    Automatic,
    /// Primal simplex.
    PrimalSimplex,
    /// Dual simplex.
    DualSimplex,
    /// Interior-point or barrier algorithm.
    Barrier,
    /// Barrier followed by crossover to a basic solution.
    BarrierWithCrossover,
    /// First-order primal-dual method.
    PrimalDualFirstOrder,
    /// Run multiple supported LP algorithms concurrently.
    Concurrent,
}

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
    /// LP algorithm. `None` uses the backend default.
    #[cfg_attr(feature = "serde", serde(default))]
    pub lp_algorithm: Option<LpAlgorithm>,
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

    pub fn with_lp_algorithm(mut self, algorithm: LpAlgorithm) -> Self {
        self.lp_algorithm = Some(algorithm);
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
        if overlay.lp_algorithm.is_some() {
            merged.lp_algorithm = overlay.lp_algorithm;
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
            && self.lp_algorithm.is_none()
            && self.parameters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_builder_and_merge_preserve_overlay_precedence() {
        let base = SolverConfig::new()
            .with_threads(4)
            .with_time_limit(10.0)
            .with_lp_algorithm(LpAlgorithm::DualSimplex);
        let overlay = SolverConfig::new()
            .with_time_limit(20.0)
            .with_lp_algorithm(LpAlgorithm::Barrier)
            .with_parameter("solver.option", "enabled");

        let merged = base.merged_with(&overlay);

        assert_eq!(merged.threads, Some(4));
        assert_eq!(merged.time_limit, Some(20.0));
        assert_eq!(merged.lp_algorithm, Some(LpAlgorithm::Barrier));
        assert_eq!(
            merged.parameters.get("solver.option").map(String::as_str),
            Some("enabled")
        );
        assert!(!merged.is_empty());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn lp_algorithm_serializes_as_solver_independent_snake_case() {
        let config = SolverConfig::new().with_lp_algorithm(LpAlgorithm::BarrierWithCrossover);
        let encoded = serde_json::to_string(&config).expect("config should serialize");
        assert!(encoded.contains("\"lp_algorithm\":\"barrier_with_crossover\""));

        let decoded: SolverConfig =
            serde_json::from_str(&encoded).expect("config should deserialize");
        assert_eq!(
            decoded.lp_algorithm,
            Some(LpAlgorithm::BarrierWithCrossover)
        );
    }
}
