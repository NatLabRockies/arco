//! IPOPT model-view adapter placeholder.

use arco_model::{ModelView, VariableId};
use arco_solver::SolverError;

pub struct ArcoProblem;

impl ArcoProblem {
    pub fn validate_supported_model(model: &(impl ModelView + ?Sized)) -> Result<(), SolverError> {
        if model.num_variables() == 0 {
            return Err(SolverError::EmptyModel);
        }
        for index in 0..model.num_variables() {
            let variable = model
                .variable(VariableId::new(index as u32))
                .ok_or(SolverError::InvalidVariableId(index as u32))?;
            if variable.is_integer {
                return Err(SolverError::SolverSpecific(
                    "IPOPT does not support integer variables".to_string(),
                ));
            }
        }
        Err(SolverError::SolverNotAvailable(
            "IPOPT model-view adapter is not implemented yet".to_string(),
        ))
    }
}
