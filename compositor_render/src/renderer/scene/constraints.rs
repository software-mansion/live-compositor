use compositor_common::{
    error::ConstraintsValidationError,
    scene::{
        validation::constraints::{input_count::InputsCountConstraint, NodeConstraints},
        NodeParams, NodeSpec, SceneSpec,
    },
};

pub fn validate_constraints(scene: &SceneSpec) -> Result<(), ConstraintsValidationError> {
    for (node_id, node_constraints) in scene
        .nodes
        .iter()
        .map(|node| (&node.node_id, node_constraints(node)))
    {
        node_constraints.validate(scene, node_id)?
    }

    Ok(())
}

pub fn node_constraints(node_spec: &NodeSpec) -> NodeConstraints {
    // TODO: make web renderer and shader constraints API configurable
    match &node_spec.params {
        NodeParams::WebRenderer { .. } => NodeConstraints {
            inputs_count: InputsCountConstraint::Bounded {
                minimal: 0,
                maximal: 16,
            },
        },
        NodeParams::Shader { .. } => NodeConstraints {
            inputs_count: InputsCountConstraint::Bounded {
                minimal: 0,
                maximal: 16,
            },
        },
        NodeParams::TextRenderer { .. } => NodeConstraints {
            inputs_count: InputsCountConstraint::Exact(0),
        },
        NodeParams::Image { .. } => NodeConstraints {
            inputs_count: InputsCountConstraint::Exact(0),
        },
        NodeParams::Builtin { transformation } => transformation.constrains(),
    }
}
