//! Canonical model for Arco.
//!
//! This crate owns Arco's canonical optimization model, model-local types,
//! sparse inspection/export views, snapshots, slack handles, and the legacy
//! model-based solver trait while solver contracts are migrated to target-based
//! APIs.

pub mod builder;
pub mod document;
pub mod expr;
pub mod ids;
pub mod indexed;
pub mod model;
pub mod slack;
pub mod types;

pub use builder::{Model32, Model64, ModelBuilder};
pub use ids::{ConstraintId, ExpressionId, VariableId};
pub use model::{
    CoefficientView, ConstraintView, CscInput, DefaultPrettyPrintAdapter, InspectOptions, Model,
    ModelError, ModelFingerprint, ModelPatch, ModelSnapshot, ModelView, ObjectiveView,
    PatchedModelView, PrettyBoundGroup, PrettyPrintAdapter, PrettyPrintOptions, PrettySection,
    SlackView, SnapshotMetadata, StructuralFacts, VariableView, format_ascii_number,
};

pub use slack::{ElasticHandle, SlackBound, SlackHandle, SlackVariables};
pub use types::{Bounds, Constraint, Objective, Sense, SimplifyLevel, Variable};
