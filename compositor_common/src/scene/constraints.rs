use serde::{Deserialize, Serialize};

use crate::{
    error::UnsatisfiedConstraintsError,
    scene::{NodeId, SceneSpec},
};

use self::input_count::InputsCountConstraint;

use super::NodeSpec;

pub mod input_count;

// TODO validate constraints aren't self-contradictory
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeConstraints(pub(crate) Vec<Constraint>);

impl NodeConstraints {
    pub fn check(
        &self,
        scene: &SceneSpec,
        node_id: &NodeId,
    ) -> Result<(), UnsatisfiedConstraintsError> {
        let node_spec = scene
            .nodes
            .iter()
            .find(|node| &node.node_id == node_id)
            .unwrap();

        for constraint in &self.0 {
            constraint.check(node_spec)?;
        }

        Ok(())
    }

    pub fn empty() -> Self {
        NodeConstraints(Vec::new())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum Constraint {
    InputsCount(InputsCountConstraint),
}

impl Constraint {
    fn check(&self, node_spec: &NodeSpec) -> Result<(), UnsatisfiedConstraintsError> {
        match self {
            Constraint::InputsCount(inputs_count_constraint) => {
                inputs_count_constraint.check(node_spec)
            }
        }
    }
}
