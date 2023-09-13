use crate::{
    error::ConstraintsValidationError,
    scene::{NodeId, SceneSpec},
};

use super::inputs::InputsCountConstraint;

pub struct NodeConstraints {
    pub inputs_count: InputsCountConstraint,
}

impl NodeConstraints {
    pub fn validate(
        &self,
        scene: &SceneSpec,
        node_id: &NodeId,
    ) -> Result<(), ConstraintsValidationError> {
        let node_spec = scene
            .nodes
            .iter()
            .find(|node| &node.node_id == node_id)
            .unwrap();

        self.inputs_count
            .validate(
                node_spec.input_pads.len() as u32,
                node_spec.transformation_name(),
            )
            .map_err(|err| {
                ConstraintsValidationError::InvalidInputsPads(err, node_spec.node_id.clone())
            })?;
        Ok(())
    }
}
