use crate::SolverSelection;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SolveRequest {
    pub selection: Option<SolverSelection>,
}

impl SolveRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_selection(mut self, selection: SolverSelection) -> Self {
        self.selection = Some(selection);
        self
    }
}
