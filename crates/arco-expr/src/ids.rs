macro_rules! define_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(transparent)]
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

define_id_type!(VariableId);
define_id_type!(ConstraintId);

#[cfg(test)]
mod tests {
    use super::{ConstraintId, VariableId};

    #[test]
    fn variable_id_roundtrip() {
        let id = VariableId::new(7);
        assert_eq!(id.inner(), 7);
    }

    #[test]
    fn constraint_id_roundtrip() {
        let id = ConstraintId::new(11);
        assert_eq!(id.inner(), 11);
    }
}
