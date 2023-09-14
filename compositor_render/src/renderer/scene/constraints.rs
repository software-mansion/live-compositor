use compositor_common::scene::{validation::constraints::Constraints, NodeSpec, SceneSpec};

use super::UpdateSceneError;

pub fn validate_constraints<'a, I: Iterator<Item = (&'a NodeSpec, Constraints)>>(
    node_constraints: I,
    scene: &SceneSpec,
) -> Result<(), UpdateSceneError> {
    for (node_spec, node_constraint) in node_constraints {
        node_constraint
            .validate(scene, &node_spec.node_id)
            .map_err(|err| {
                UpdateSceneError::ConstraintsValidationError(err, node_spec.node_id.clone())
            })?;
    }

    Ok(())
}
