use std::{collections::HashSet, sync::Arc};

use crate::{
    error::{SceneSpecValidationError, UnusedNodesError},
    renderer_spec::RendererId,
    scene::{id::NodeId, id::OutputId, NodeParams, NodeSpec, OutputSpec, Resolution, SceneSpec},
};

#[test]
fn scene_validation_finds_cycle() {
    let resolution = Resolution {
        width: 1920,
        height: 1080,
    };
    let trans_params = NodeParams::Shader {
        shader_id: RendererId(Arc::from("shader")),
        shader_params: None,
        resolution,
    };

    let input_id = NodeId(Arc::from("input"));
    let a_id = NodeId(Arc::from("a"));
    let b_id = NodeId(Arc::from("b"));
    let c_id = NodeId(Arc::from("c"));
    let output_id = NodeId(Arc::from("output"));

    let a = NodeSpec {
        node_id: a_id.clone(),
        input_pads: vec![input_id.clone(), c_id.clone()],
        params: trans_params.clone(),
        fallback_id: None,
    };

    let b = NodeSpec {
        node_id: b_id.clone(),
        input_pads: vec![a_id],
        params: trans_params.clone(),
        fallback_id: None,
    };

    let c = NodeSpec {
        node_id: c_id.clone(),
        input_pads: vec![b_id],
        params: trans_params,
        fallback_id: None,
    };

    let output = OutputSpec {
        output_id: OutputId(output_id.clone()),
        input_pad: c_id.clone(),
    };

    let scene_spec = SceneSpec {
        nodes: vec![a, b, c],
        outputs: vec![output],
    };

    let registered_inputs = HashSet::from([&input_id]);
    let registered_outputs = HashSet::from([&output_id]);

    let validation_result = scene_spec.validate(&registered_inputs, &registered_outputs);
    assert_eq!(
        validation_result.err(),
        Some(SceneSpecValidationError::CycleDetected(c_id.clone()))
    );
}

#[test]
fn scene_validation_finds_unused_nodes() {
    let resolution = Resolution {
        width: 1920,
        height: 1080,
    };
    let trans_params = NodeParams::Shader {
        shader_id: RendererId(Arc::from("shader")),
        shader_params: None,
        resolution,
    };

    let input_id = NodeId(Arc::from("input"));
    let unused_input_id = NodeId(Arc::from("unused_input"));
    let a_id = NodeId(Arc::from("a"));
    let b_id = NodeId(Arc::from("b"));
    let c_id = NodeId(Arc::from("c"));
    let d_id = NodeId(Arc::from("d"));
    let output_id = NodeId(Arc::from("output"));

    let a = NodeSpec {
        node_id: a_id.clone(),
        input_pads: vec![input_id.clone()],
        params: trans_params.clone(),
        fallback_id: Some(d_id.clone()),
    };

    let b = NodeSpec {
        node_id: b_id.clone(),
        input_pads: vec![c_id.clone()],
        params: trans_params.clone(),
        fallback_id: None,
    };

    let c = NodeSpec {
        node_id: c_id.clone(),
        input_pads: vec![b_id.clone()],
        params: trans_params.clone(),
        fallback_id: None,
    };

    let d = NodeSpec {
        node_id: d_id,
        input_pads: vec![],
        params: trans_params,
        fallback_id: None,
    };

    let output = OutputSpec {
        output_id: OutputId(output_id.clone()),
        input_pad: a_id,
    };

    let scene_spec = SceneSpec {
        nodes: vec![a, b, c, d],
        outputs: vec![output],
    };

    let unused_nodes = HashSet::from([b_id, c_id]);

    let registered_inputs = HashSet::from([&input_id, &unused_input_id]);
    let registered_outputs = HashSet::from([&output_id]);

    assert_eq!(
        scene_spec
            .validate(&registered_inputs, &registered_outputs)
            .err(),
        Some(UnusedNodesError(unused_nodes).into())
    );
}
