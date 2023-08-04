use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::scene::{NodeId, SceneSpec, TransformNodeSpec};

#[derive(Debug, thiserror::Error)]
pub enum SpecValidationError {
    #[error("missing node with id {missing_node} used in transformation {transformation} is not defined in scene and it was not registered as an input")]
    MissingInputNodeForTransformation {
        missing_node: Arc<str>,
        transformation: Arc<str>,
    },
    #[error("missing node with id {missing_node} used in output {output} is not defined in scene and it was not registered as an input")]
    MissingInputNodeForOutput {
        missing_node: Arc<str>,
        output: Arc<str>,
    },
    #[error("detected cycle in scene graph, scene should acyclic graph")]
    CycleDetected,
    #[error("duplicate node id: {0}")]
    DuplicateNames(Arc<str>),
    #[error("unknown output, output with id {0} is not registered currently")]
    UnknownOutput(Arc<str>),
    #[error("unknown input, input with id {0} is not registered currently")]
    UnknownInput(Arc<str>),
}

impl SceneSpec {
    // Validate if SceneSpec represents valid scene:
    // - check if each transform have inputs that are either registered or are a transformation
    // itself
    // - check if each input pad of each output is a either registered input or a transformation
    // - check if each output in scene spec is registered output
    // - check if each input in scene spec is registered input
    //
    // TODO: check nodes ids uniqueness, check for unused nodes, ...
    pub fn validate(
        &self,
        registered_inputs: &HashSet<NodeId>,
        registered_outputs: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        let transform_iter = self.transforms.iter().map(|i| i.node_id.clone());
        let input_iter = self.inputs.iter().map(|i| i.input_id.0.clone());
        let defined_node_ids: Vec<NodeId> = transform_iter.chain(input_iter).collect();
        let node_ids: HashSet<NodeId> = defined_node_ids.iter().cloned().collect();

        self.validate_inputs(registered_inputs)?;
        self.validate_transform_inputs(&node_ids)?;
        self.validate_outputs(registered_outputs, &node_ids)?;
        self.validate_node_ids_uniqueness(&defined_node_ids)?;
        self.validate_cycles(registered_inputs)?;

        Ok(())
    }

    fn validate_transform_inputs(
        &self,
        defined_node_ids: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        for t in self.transforms.iter() {
            for input in &t.input_pads {
                if !defined_node_ids.contains(input) {
                    return Err(SpecValidationError::MissingInputNodeForTransformation {
                        missing_node: input.0.clone(),
                        transformation: t.node_id.0.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_inputs(
        &self,
        registered_inputs: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        for input in self.inputs.iter() {
            if !registered_inputs.contains(&input.input_id.0) {
                return Err(SpecValidationError::UnknownInput(
                    input.input_id.0 .0.clone(),
                ));
            }
        }

        Ok(())
    }

    fn validate_outputs(
        &self,
        registered_outputs: &HashSet<NodeId>,
        defined_node_ids: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        for out in self.outputs.iter() {
            let node_id = &out.input_pad;
            if !defined_node_ids.contains(node_id) {
                return Err(SpecValidationError::MissingInputNodeForOutput {
                    missing_node: out.input_pad.0.clone(),
                    output: node_id.0.clone(),
                });
            }
            if registered_outputs.get(&out.output_id.0).is_none() {
                return Err(SpecValidationError::UnknownOutput(node_id.0.clone()));
            }
        }

        Ok(())
    }

    fn validate_cycles(
        &self,
        registered_inputs: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        enum NodeState {
            BeingVisited,
            Visited,
        }

        let nodes: HashMap<&NodeId, &TransformNodeSpec> = self
            .transforms
            .iter()
            .map(|node| (&node.node_id, node))
            .collect();

        let mut visited: HashMap<NodeId, NodeState> = HashMap::new();

        fn visit(
            node: &NodeId,
            transform_nodes: &HashMap<&NodeId, &TransformNodeSpec>,
            input_nodes: &HashSet<NodeId>,
            visited: &mut HashMap<NodeId, NodeState>,
        ) -> Result<(), SpecValidationError> {
            if let Some(_input_node) = input_nodes.get(node) {
                return Ok(());
            }

            match visited.get(node) {
                Some(NodeState::BeingVisited) => return Err(SpecValidationError::CycleDetected),
                Some(NodeState::Visited) => return Ok(()),
                None => {}
            }

            visited.insert(node.clone(), NodeState::BeingVisited);

            for child in &transform_nodes.get(&node).unwrap().input_pads {
                visit(child, transform_nodes, input_nodes, visited)?;
            }

            visited.insert(node.clone(), NodeState::Visited);

            Ok(())
        }

        for output in &self.outputs {
            visit(&output.input_pad, &nodes, registered_inputs, &mut visited)?;
        }

        Ok(())
    }

    fn validate_node_ids_uniqueness(
        &self,
        defined_node_ids: &Vec<NodeId>,
    ) -> Result<(), SpecValidationError> {
        let mut nodes_ids: HashSet<NodeId> = HashSet::new();

        for node_id in defined_node_ids {
            match nodes_ids.get(node_id) {
                Some(_) => {
                    return Err(SpecValidationError::DuplicateNames(node_id.0.clone()));
                }
                None => {
                    nodes_ids.insert(node_id.clone());
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    };

    use crate::{
        scene::{
            InputId, InputSpec, NodeId, OutputId, OutputSpec, Resolution, SceneSpec,
            TransformNodeSpec, TransformParams,
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
        let trans_parms = TransformParams::Shader {
            shader_id: TransformationRegistryKey(Arc::from("shader")),
            shader_params: HashMap::new(),
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
            transform_params: trans_parms.clone(),
        };

        let b = TransformNodeSpec {
            node_id: b_id.clone(),
            input_pads: vec![a_id.clone()],
            resolution: res,
            transform_params: trans_parms.clone(),
        };

        let c = TransformNodeSpec {
            node_id: c_id.clone(),
            input_pads: vec![b_id.clone()],
            resolution: res,
            transform_params: trans_parms.clone(),
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

        let registered_inputs = HashSet::from([input_id]);
        let registered_outputs = HashSet::from([output_id]);

        assert!(matches!(
            scene_spec.validate(&registered_inputs, &registered_outputs),
            Err(SpecValidationError::CycleDetected)
        ));
    }
}
