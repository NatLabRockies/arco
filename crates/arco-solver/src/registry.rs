//! Solver registry and family metadata.

use std::collections::{BTreeMap, BTreeSet};

/// Supported solver transports in v1.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum SolverTransport {
    /// In-process linked backend.
    Embedded,
    /// External process invocation backend.
    ExternalProcess,
}

/// Capability support model for a solver family.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SolverCapabilityModel {
    /// Whether integer/binary variables are supported.
    pub supports_integer: bool,
    /// Whether warm starts are supported.
    pub warm_start: bool,
    /// Whether quadratic objective terms are supported.
    pub quadratic_objective: bool,
    /// Whether quadratic constraints are supported.
    pub quadratic_constraints: bool,
    /// Whether multi-objective solve is supported.
    pub multi_objective: bool,
    /// Whether IIS extraction is supported.
    pub iis: bool,
}

impl SolverCapabilityModel {
    /// Conservative defaults for LP/MIP-only families.
    pub fn lp_mip_default() -> Self {
        Self {
            supports_integer: true,
            warm_start: true,
            quadratic_objective: false,
            quadratic_constraints: false,
            multi_objective: false,
            iis: false,
        }
    }

    /// Conservative defaults for continuous-only NLP families.
    pub fn continuous_default() -> Self {
        Self {
            supports_integer: false,
            warm_start: true,
            quadratic_objective: true,
            quadratic_constraints: true,
            multi_objective: false,
            iis: false,
        }
    }
}

/// Family-level metadata in the solver registry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SolverFamily {
    /// Canonical family name (unique).
    pub name: String,
    /// Human-readable label.
    pub display_name: String,
    /// Supported transports.
    pub transports: BTreeSet<SolverTransport>,
    /// Capability model.
    pub capabilities: SolverCapabilityModel,
}

impl SolverFamily {
    /// Build an embedded family descriptor.
    pub fn embedded(
        name: impl Into<String>,
        display_name: impl Into<String>,
        capabilities: SolverCapabilityModel,
    ) -> Self {
        let mut transports = BTreeSet::new();
        transports.insert(SolverTransport::Embedded);
        Self {
            name: name.into(),
            display_name: display_name.into(),
            transports,
            capabilities,
        }
    }

    /// Build an external-process family descriptor.
    pub fn external_process(
        name: impl Into<String>,
        display_name: impl Into<String>,
        capabilities: SolverCapabilityModel,
    ) -> Self {
        let mut transports = BTreeSet::new();
        transports.insert(SolverTransport::ExternalProcess);
        Self {
            name: name.into(),
            display_name: display_name.into(),
            transports,
            capabilities,
        }
    }
}

/// Static registry model for known families.
#[derive(Debug, Clone, Default)]
pub struct SolverRegistry {
    families: BTreeMap<String, SolverFamily>,
}

impl SolverRegistry {
    /// Build an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the in-tree known family registry.
    pub fn with_builtin_families() -> Self {
        let mut registry = Self::new();
        registry.add_family(SolverFamily::embedded(
            "highs",
            "HiGHS",
            SolverCapabilityModel::lp_mip_default(),
        ));
        registry.add_family(SolverFamily::embedded(
            "xpress",
            "Xpress",
            SolverCapabilityModel::lp_mip_default(),
        ));
        registry.add_family(SolverFamily::embedded(
            "ipopt",
            "Ipopt",
            SolverCapabilityModel::continuous_default(),
        ));
        registry
    }

    /// Register a family.
    pub fn add_family(&mut self, family: SolverFamily) {
        self.families.insert(family.name.clone(), family);
    }

    /// Lookup family by name.
    pub fn family(&self, name: &str) -> Option<&SolverFamily> {
        self.families.get(name)
    }

    /// Iterate families.
    pub fn families(&self) -> impl Iterator<Item = &SolverFamily> {
        self.families.values()
    }
}
