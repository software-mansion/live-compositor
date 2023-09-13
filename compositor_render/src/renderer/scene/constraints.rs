use compositor_common::{
    error::UnsatisfiedConstraintsError,
    scene::{validation::constraints::Constraints, NodeSpec, SceneSpec},
};

pub fn validate_constraints<'a, I: Iterator<Item = (&'a NodeSpec, Constraints)>>(
    node_constraints: I,
    scene: &SceneSpec,
) -> Result<(), UnsatisfiedConstraintsError> {
    for (node_spec, node_constraint) in node_constraints {
        node_constraint.validate(scene, &node_spec.node_id)?;
    }

    Ok(())
}
