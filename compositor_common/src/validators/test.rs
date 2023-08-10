use std::{collections::HashSet, sync::Arc};

use crate::{
    scene::{
        InputId, InputSpec, NodeId, OutputId, OutputSpec, Resolution, SceneSpec, TransformNodeSpec,
        TransformParams,
    },
    transformation::TransformationRegistryKey,
    validators::SpecValidationError,
};

#[test]
fn scene_validation_finds_cycle() {
    let res = Resolution {
        width: 1920,
        height: 1080,
    };
    let trans_params = TransformParams::Shader {
        shader_id: TransformationRegistryKey(Arc::from("shader")),
        shader_params: crate::scene::ShaderParam::U32(42),
    };

    let input_id = NodeId(Arc::from("input"));
    let a_id = NodeId(Arc::from("a"));
    let b_id = NodeId(Arc::from("b"));
    let c_id = NodeId(Arc::from("c"));
    let output_id = NodeId(Arc::from("output"));

    let input = InputSpec {
        input_id: InputId(input_id.clone()),
        resolution: res,
    };

    let a = TransformNodeSpec {
        node_id: a_id.clone(),
        input_pads: vec![input_id.clone(), c_id.clone()],
        resolution: res,
        transform_params: trans_params.clone(),
    };

    let b = TransformNodeSpec {
        node_id: b_id.clone(),
        input_pads: vec![a_id],
        resolution: res,
        transform_params: trans_params.clone(),
    };

    let c = TransformNodeSpec {
        node_id: c_id.clone(),
        input_pads: vec![b_id],
        resolution: res,
        transform_params: trans_params,
    };

    let output = OutputSpec {
        output_id: OutputId(output_id.clone()),
        input_pad: c_id,
    };

    let scene_spec = SceneSpec {
        inputs: vec![input],
        transforms: vec![a, b, c],
        outputs: vec![output],
    };

    let registered_inputs = HashSet::from([&input_id]);
    let registered_outputs = HashSet::from([&output_id]);

    assert!(matches!(
        scene_spec.validate(&registered_inputs, &registered_outputs),
        Err(SpecValidationError::CycleDetected)
    ));
}

#[test]
fn scene_validation_finds_unused_nodes() {
    let res = Resolution {
        width: 1920,
        height: 1080,
    };
    let trans_params = TransformParams::Shader {
        shader_id: TransformationRegistryKey(Arc::from("shader")),
        shader_params: crate::scene::ShaderParam::U32(42),
    };

    let input_id = NodeId(Arc::from("input"));
    let unused_input_id = NodeId(Arc::from("unused_input"));
    let a_id = NodeId(Arc::from("a"));
    let b_id = NodeId(Arc::from("b"));
    let c_id = NodeId(Arc::from("c"));
    let output_id = NodeId(Arc::from("output"));

    let input = InputSpec {
        input_id: InputId(input_id.clone()),
        resolution: res,
    };

    let unused_input = InputSpec {
        input_id: InputId(unused_input_id.clone()),
        resolution: res,
    };

    let a = TransformNodeSpec {
        node_id: a_id.clone(),
        input_pads: vec![input_id.clone()],
        resolution: res,
        transform_params: trans_params.clone(),
    };

    let b = TransformNodeSpec {
        node_id: b_id.clone(),
        input_pads: vec![c_id.clone()],
        resolution: res,
        transform_params: trans_params.clone(),
    };

    let c = TransformNodeSpec {
        node_id: c_id.clone(),
        input_pads: vec![b_id.clone()],
        resolution: res,
        transform_params: trans_params,
    };

    let output = OutputSpec {
        output_id: OutputId(output_id.clone()),
        input_pad: a_id,
    };

    let scene_spec = SceneSpec {
        inputs: vec![input, unused_input],
        transforms: vec![a, b, c],
        outputs: vec![output],
    };

    let unused_nodes = HashSet::from([unused_input_id.0.clone(), b_id.0, c_id.0]);

    let registered_inputs = HashSet::from([&input_id, &unused_input_id]);
    let registered_outputs = HashSet::from([&output_id]);

    assert_eq!(
        scene_spec.validate(&registered_inputs, &registered_outputs),
        Err(SpecValidationError::UnusedNodes(unused_nodes))
    );
}
