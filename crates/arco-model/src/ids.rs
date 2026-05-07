//! Compact identifier wrappers owned by the primitive model crate.
//!
//! `VariableId` and `ConstraintId` currently preserve the legacy type identity
//! while downstream crates migrate off `arco-expr` imports. `ExpressionId` is the
//! primitive identifier for stored symbolic expressions.

pub use arco_expr::ids::{ConstraintId, VariableId};

macro_rules! define_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(transparent)]
        #[doc = concat!(stringify!($name), " is a compact, type-safe numeric identifier.")]
        pub struct $name(u32);

        impl $name {
            /// Get the inner u32 value.
            pub fn inner(self) -> u32 {
                self.0
            }

            /// Create an ID from a u32 value.
            pub fn new(value: u32) -> Self {
                Self(value)
            }
        }
    };
}

define_id_type!(ExpressionId);

#[cfg(test)]
mod tests {
    use crate::ids::{ConstraintId, ExpressionId, VariableId};

    #[test]
    fn ids_roundtrip_inner_values() {
        assert_eq!(VariableId::new(3).inner(), 3);
        assert_eq!(ConstraintId::new(5).inner(), 5);
        assert_eq!(ExpressionId::new(7).inner(), 7);
    }
}
