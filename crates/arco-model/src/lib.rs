//! Canonical model for Arco.
//!
//! This crate owns Arco's canonical optimization model, model-local types,
//! sparse inspection/export views, snapshots, slack handles, and the legacy
//! model-based solver trait while solver contracts are migrated to target-based
//! APIs.

pub mod model;
pub mod slack;
pub mod types;

pub use model::{
    CoefficientView, ConstraintView, CscInput, DefaultPrettyPrintAdapter, InspectOptions, Model,
    ModelError, ModelSnapshot, ObjectiveView, PrettyBoundGroup, PrettyPrintAdapter,
    PrettyPrintOptions, PrettySection, SlackView, SnapshotMetadata, VariableView,
    format_ascii_number,
};

pub use slack::{ElasticHandle, SlackBound, SlackHandle, SlackVariables};
pub use types::{Bounds, Constraint, Objective, Sense, SimplifyLevel, Variable};
