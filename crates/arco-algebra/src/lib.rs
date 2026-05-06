//! Canonical algebra seam for Arco.
//!
//! Expression internals currently live in `arco-expr`; this crate is the stable
//! dependency target for canonical-model algebra during the migration.

pub use arco_expr::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebra_seam_reexports_variable_id() {
        let id = VariableId::new(0);
        assert_eq!(id.inner(), 0);
    }
}
