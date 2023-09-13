use serde::{Deserialize, Serialize};

use crate::{
    error::UnsatisfiedConstraintsError,
    scene::{NodeId, SceneSpec},
};

use self::input_count::InputsCountConstraint;

pub mod input_count;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Constraints {
    pub inputs_count: InputsCountConstraint,
}

impl Constraints {
    pub fn validate(
        &self,
        scene: &SceneSpec,
        node_id: &NodeId,
    ) -> Result<(), UnsatisfiedConstraintsError> {
        let node_spec = scene
            .nodes
            .iter()
            .find(|node| &node.node_id == node_id)
            .unwrap();

        self.inputs_count.validate(node_spec)?;

        Ok(())
    }
}
