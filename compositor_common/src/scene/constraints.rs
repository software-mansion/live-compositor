use crate::{
    error::UnsatisfiedConstraintsError,
    scene::{NodeId, SceneSpec},
};

use self::input_count::InputCountConstraint;

use super::NodeSpec;

pub mod input_count;

// TODO validate constraints aren't self-contradictory
#[derive(Debug)]
pub struct NodeConstraints(pub Vec<Constraint>);

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

#[derive(Debug, Clone)]
pub enum Constraint {
    InputCount(InputCountConstraint),
}

impl Constraint {
    fn check(&self, node_spec: &NodeSpec) -> Result<(), UnsatisfiedConstraintsError> {
        match self {
            Constraint::InputCount(constraint) => constraint.check(node_spec),
        }
    }
}
