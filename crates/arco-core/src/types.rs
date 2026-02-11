use arco_expr::ids::VariableId;

/// Optimization sense
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    Minimize,
    Maximize,
}

/// Simplification level for expression lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SimplifyLevel {
    #[default]
    None,
    Light,
}

impl SimplifyLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            SimplifyLevel::None => "none",
            SimplifyLevel::Light => "light",
        }
    }
}

/// Bounds for a variable or constraint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub lower: f64,
    pub upper: f64,
}

impl Bounds {
    pub fn new(lower: f64, upper: f64) -> Self {
        Self { lower, upper }
    }
}

/// A decision variable with bounds and integrality constraint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Variable {
    pub bounds: Bounds,
    pub is_integer: bool,
    pub is_active: bool,
}

impl Variable {
    /// Create a binary variable with bounds [0, 1] and integer constraint.
    pub fn binary() -> Self {
        Self {
            bounds: Bounds::new(0.0, 1.0),
            is_integer: true,
            is_active: true,
        }
    }

    /// Create a continuous variable with specified bounds.
    pub fn continuous(bounds: Bounds) -> Self {
        Self {
            bounds,
            is_integer: false,
            is_active: true,
        }
    }

    /// Create an integer variable with specified bounds.
    pub fn integer(bounds: Bounds) -> Self {
        Self {
            bounds,
            is_integer: true,
            is_active: true,
        }
    }
}

/// A constraint with lower and upper bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraint {
    pub bounds: Bounds,
}

/// Objective function with a sense and linear terms
#[derive(Debug, Clone)]
pub struct Objective {
    pub sense: Option<Sense>,
    pub terms: Vec<(VariableId, f64)>,
}

impl Objective {
    /// Create a new empty objective
    pub fn new() -> Self {
        Self {
            sense: None,
            terms: Vec::new(),
        }
    }
}

impl Default for Objective {
    fn default() -> Self {
        Self::new()
    }
}
