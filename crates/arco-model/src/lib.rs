//! Canonical model seam for Arco.
//!
//! The current canonical model implementation still lives in `arco-core`.
//! This crate provides the dependency target for authoring, validation, and
//! compile crates while that implementation is migrated behind the model seam.

pub use arco_core::{Bounds, Constraint, Model, ModelError, Objective, Sense, Variable};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_seam_reexports_core_model() {
        let model = Model::new();
        assert_eq!(model.num_variables(), 0);
    }
}
