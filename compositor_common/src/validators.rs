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
    #[error("Unused nodes: {0:?}")]
    UnusedNodes(HashSet<Arc<str>>),
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
    pub fn validate(
        &self,
        registered_inputs: &HashSet<NodeId>,
        registered_outputs: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        let transform_iter = self.transforms.iter().map(|i| i.node_id.clone());
        let input_iter = self.inputs.iter().map(|i| i.input_id.0.clone());
        let defined_node_ids = transform_iter.chain(input_iter);
        let node_ids: HashSet<NodeId> = defined_node_ids.clone().collect();

        let transform_nodes: HashMap<NodeId, &TransformNodeSpec> = self
            .transforms
            .iter()
            .map(|node| (node.node_id.clone(), node))
            .collect();

        self.validate_inputs(registered_inputs)?;
        self.validate_transform_inputs(&node_ids)?;
        self.validate_outputs(registered_outputs, &node_ids)?;
        self.validate_node_ids_uniqueness(defined_node_ids)?;
        self.validate_cycles(registered_inputs, &transform_nodes)?;
        self.validate_nodes_are_used(registered_inputs, &transform_nodes)?;

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
        transform_nodes: &HashMap<NodeId, &TransformNodeSpec>,
    ) -> Result<(), SpecValidationError> {
        enum NodeState {
            BeingVisited,
            Visited,
        }

        let mut visited: HashMap<NodeId, NodeState> = HashMap::new();

        fn visit(
            node: &NodeId,
            transform_nodes: &HashMap<NodeId, &TransformNodeSpec>,
            inputs: &HashSet<NodeId>,
            visited: &mut HashMap<NodeId, NodeState>,
        ) -> Result<(), SpecValidationError> {
            if let Some(_input) = inputs.get(node) {
                return Ok(());
            }

            match visited.get(node) {
                Some(NodeState::BeingVisited) => return Err(SpecValidationError::CycleDetected),
                Some(NodeState::Visited) => return Ok(()),
                None => {}
            }

            visited.insert(node.clone(), NodeState::BeingVisited);

            for child in &transform_nodes.get(node).unwrap().input_pads {
                visit(child, transform_nodes, inputs, visited)?;
            }

            visited.insert(node.clone(), NodeState::Visited);

            Ok(())
        }

        for output in &self.outputs {
            visit(
                &output.input_pad,
                transform_nodes,
                registered_inputs,
                &mut visited,
            )?;
        }

        Ok(())
    }

    fn validate_node_ids_uniqueness<I: Iterator<Item = NodeId>>(
        &self,
        defined_node_ids: I,
    ) -> Result<(), SpecValidationError> {
        let mut nodes_ids: HashSet<NodeId> = HashSet::new();

        for node_id in defined_node_ids {
            if nodes_ids.contains(&node_id) {
                return Err(SpecValidationError::DuplicateNames(node_id.0.clone()));
            }
            nodes_ids.insert(node_id);
        }

        Ok(())
    }

    fn validate_nodes_are_used(
        &self,
        input_nodes: &HashSet<NodeId>,
        transform_nodes: &HashMap<NodeId, &TransformNodeSpec>,
    ) -> Result<(), SpecValidationError> {
        let mut visited: HashSet<NodeId> = HashSet::new();

        fn visit(
            node: &NodeId,
            transform_nodes: &HashMap<NodeId, &TransformNodeSpec>,
            inputs: &HashSet<NodeId>,
            visited: &mut HashSet<NodeId>,
        ) {
            if let Some(_input) = inputs.get(node) {
                return;
            }
            if visited.contains(node) {
                return;
            }

            for child in &transform_nodes.get(node).unwrap().input_pads {
                visit(child, transform_nodes, inputs, visited);
            }

            visited.insert(node.clone());
        }

        for output in &self.outputs {
            visit(
                &output.input_pad,
                transform_nodes,
                input_nodes,
                &mut visited,
            )
        }

        let mut unused_transforms: HashSet<Arc<str>> = HashSet::new();

        for transform_node in transform_nodes.keys() {
            if !visited.contains(transform_node) {
                unused_transforms.insert(transform_node.0.clone());
            }
        }

        if !unused_transforms.is_empty() {
            return Err(SpecValidationError::UnusedNodes(unused_transforms));
        }

        Ok(())
    }
}
