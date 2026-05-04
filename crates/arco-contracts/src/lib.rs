//! Shared solver/platform contracts for Arco.

pub mod config;
pub mod profile;
pub mod registry;
pub mod request;
pub mod selection;
pub mod traits;
pub mod types;

pub use config::SolverConfig;
pub use profile::{SolverConfigDocument, SolverProfile, merged_profiles};
pub use registry::{SolverCapabilityModel, SolverFamily, SolverRegistry, SolverTransport};
pub use request::SolveRequest;
pub use selection::{ResolvedSelection, SelectionError, SolverSelection, resolve_selection};
pub use traits::{SolutionView, Solve};
pub use types::{Solution, SolverError, SolverStatus};
