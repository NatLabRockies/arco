pub mod algebra;
pub mod compile;
pub mod pipeline;
pub mod semantic;
pub mod source;

use serde::{Deserialize, Serialize};

/// Optimization direction for an objective function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}
