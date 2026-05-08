pub mod algebra;
pub mod primitives;
pub mod source;

pub use primitives::{
    PrimitiveBuildError, build_arco_document, build_indexed_data, build_model_document,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}
