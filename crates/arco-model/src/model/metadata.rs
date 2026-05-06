//! Metadata methods for variable and constraint naming.

use std::collections::{BTreeMap, HashMap};

use arco_expr::ids::{ConstraintId, VariableId};

use crate::model::Model;
use crate::model::error::ModelError;

impl Model {
    /// Set name for a variable.
    pub fn set_variable_name(&mut self, id: VariableId, name: String) -> Result<(), ModelError> {
        self.ensure_variable_exists(id)?;

        // Update forward mapping (id -> name)
        self.variable_names
            .get_or_insert_with(BTreeMap::new)
            .insert(id, name.clone());

        // Update reverse mapping (name -> id) for O(1) lookup
        self.variable_name_to_id
            .get_or_insert_with(HashMap::new)
            .insert(name, id);

        Ok(())
    }

    /// Get name for a variable.
    pub fn get_variable_name(&self, id: VariableId) -> Option<&str> {
        self.variable_names
            .as_ref()
            .and_then(|names| names.get(&id).map(|s| s.as_str()))
    }

    /// Set objective name.
    pub fn set_objective_name(&mut self, name: Option<String>) -> Result<(), ModelError> {
        self.objective_name = name;
        Ok(())
    }

    /// Get objective name.
    pub fn get_objective_name(&self) -> Option<&str> {
        self.objective_name.as_deref()
    }

    /// Lookup a variable by name.
    /// Uses O(1) HashMap lookup instead of O(n) linear scan.
    pub fn get_variable_by_name(&self, name: &str) -> Option<VariableId> {
        self.variable_name_to_id
            .as_ref()
            .and_then(|map| map.get(name).copied())
    }

    /// Set metadata for a variable.
    pub fn set_variable_metadata(
        &mut self,
        id: VariableId,
        metadata: serde_json::Value,
    ) -> Result<(), ModelError> {
        self.ensure_variable_exists(id)?;
        self.variable_metadata
            .get_or_insert_with(BTreeMap::new)
            .insert(id, metadata);
        Ok(())
    }

    /// Get metadata for a variable.
    pub fn get_variable_metadata(&self, id: VariableId) -> Option<&serde_json::Value> {
        self.variable_metadata
            .as_ref()
            .and_then(|meta| meta.get(&id))
    }

    /// Set name for a constraint.
    pub fn set_constraint_name(
        &mut self,
        id: ConstraintId,
        name: String,
    ) -> Result<(), ModelError> {
        self.ensure_constraint_exists(id)?;

        // Update forward mapping (id -> name)
        self.constraint_names
            .get_or_insert_with(BTreeMap::new)
            .insert(id, name.clone());

        // Update reverse mapping (name -> id) for O(1) lookup
        self.constraint_name_to_id
            .get_or_insert_with(HashMap::new)
            .insert(name, id);

        Ok(())
    }

    /// Get name for a constraint.
    pub fn get_constraint_name(&self, id: ConstraintId) -> Option<&str> {
        self.constraint_names
            .as_ref()
            .and_then(|names| names.get(&id).map(|s| s.as_str()))
    }

    /// Lookup a constraint by name.
    /// Uses O(1) HashMap lookup instead of O(n) linear scan.
    pub fn get_constraint_by_name(&self, name: &str) -> Option<ConstraintId> {
        self.constraint_name_to_id
            .as_ref()
            .and_then(|map| map.get(name).copied())
    }

    /// Set metadata for a constraint.
    pub fn set_constraint_metadata(
        &mut self,
        id: ConstraintId,
        metadata: serde_json::Value,
    ) -> Result<(), ModelError> {
        self.ensure_constraint_exists(id)?;
        self.constraint_metadata
            .get_or_insert_with(BTreeMap::new)
            .insert(id, metadata);
        Ok(())
    }

    /// Get metadata for a constraint.
    pub fn get_constraint_metadata(&self, id: ConstraintId) -> Option<&serde_json::Value> {
        self.constraint_metadata
            .as_ref()
            .and_then(|meta| meta.get(&id))
    }
}
