pub mod algebra;
pub mod artifacts;
pub mod compile;
pub mod pipeline;
pub mod semantic;
pub mod source;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}
