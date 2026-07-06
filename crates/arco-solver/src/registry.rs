use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SolverTransport {
    Embedded,
    ExternalProcess,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SolverCapabilityModel {
    pub supports_integer: bool,
    pub warm_start: bool,
    pub quadratic_objective: bool,
    pub quadratic_constraints: bool,
    pub multi_objective: bool,
    pub iis: bool,
}

impl SolverCapabilityModel {
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SolverFamily {
    pub name: String,
    pub display_name: String,
    pub transports: BTreeSet<SolverTransport>,
    pub capabilities: SolverCapabilityModel,
}

impl SolverFamily {
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

#[derive(Debug, Clone, Default)]
pub struct SolverRegistry {
    families: BTreeMap<String, SolverFamily>,
}

impl SolverRegistry {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn add_family(&mut self, family: SolverFamily) {
        self.families.insert(family.name.clone(), family);
    }

    pub fn family(&self, name: &str) -> Option<&SolverFamily> {
        self.families.get(name)
    }

    pub fn families(&self) -> impl Iterator<Item = &SolverFamily> {
        self.families.values()
    }
}
