use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::scene::{InputSpec, NodeId, SceneSpec, TransformNodeSpec};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
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
    #[error("Invalid node params for node with id {0}")]
    InvalidTransformParams(Arc<str>),
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
        registered_inputs: &HashSet<&NodeId>,
        registered_outputs: &HashSet<&NodeId>,
    ) -> Result<(), SpecValidationError> {
        let transform_iter = self.transforms.iter().map(|i| &i.node_id);
        let input_iter = self.inputs.iter().map(|i| &i.input_id.0);
        let defined_node_ids = transform_iter.chain(input_iter.clone());

        let node_ids: HashSet<&NodeId> = defined_node_ids.clone().collect();

        let input_nodes: HashMap<&NodeId, &InputSpec> = self
            .inputs
            .iter()
            .map(|input| (&input.input_id.0, input))
            .collect();

        let transform_nodes: HashMap<&NodeId, &TransformNodeSpec> = self
            .transforms
            .iter()
            .map(|node| (&node.node_id, node))
            .collect();

        self.validate_inputs(registered_inputs)?;
        self.validate_transform_inputs(&node_ids)?;
        self.validate_outputs(registered_outputs, &node_ids)?;
        self.validate_node_ids_uniqueness(defined_node_ids)?;
        self.validate_cycles(registered_inputs, &transform_nodes)?;
        self.validate_nodes_are_used(&input_nodes, &transform_nodes)?;

        Ok(())
    }

    fn validate_transform_inputs(
        &self,
        defined_node_ids: &HashSet<&NodeId>,
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
        registered_inputs: &HashSet<&NodeId>,
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
        registered_outputs: &HashSet<&NodeId>,
        defined_node_ids: &HashSet<&NodeId>,
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
        registered_inputs: &HashSet<&NodeId>,
        transform_nodes: &HashMap<&NodeId, &TransformNodeSpec>,
    ) -> Result<(), SpecValidationError> {
        enum NodeState {
            BeingVisited,
            Visited,
        }

        let mut visited: HashMap<&NodeId, NodeState> = HashMap::new();

        fn visit<'a>(
            node: &'a NodeId,
            transform_nodes: &'a HashMap<&NodeId, &TransformNodeSpec>,
            inputs: &HashSet<&NodeId>,
            visited: &mut HashMap<&'a NodeId, NodeState>,
        ) -> Result<(), SpecValidationError> {
            if inputs.get(node).is_some() {
                return Ok(());
            }

            match visited.get(node) {
                Some(NodeState::BeingVisited) => return Err(SpecValidationError::CycleDetected),
                Some(NodeState::Visited) => return Ok(()),
                None => {}
            }

            visited.insert(node, NodeState::BeingVisited);

            for child in &transform_nodes.get(node).unwrap().input_pads {
                visit(child, transform_nodes, inputs, visited)?;
            }

            visited.insert(node, NodeState::Visited);

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

    fn validate_node_ids_uniqueness<'a, I: Iterator<Item = &'a NodeId>>(
        &self,
        defined_node_ids: I,
    ) -> Result<(), SpecValidationError> {
        let mut nodes_ids: HashSet<&NodeId> = HashSet::new();

        for node_id in defined_node_ids {
            if !nodes_ids.insert(node_id) {
                return Err(SpecValidationError::DuplicateNames(node_id.0.clone()));
            }
        }

        Ok(())
    }

    /// Assumes that all TransformNodeSpec inputs are correct
    /// (input_nodes or transform_nodes contains them)
    /// [`validate_transform_inputs`] should be run before this function
    fn validate_nodes_are_used(
        &self,
        input_nodes: &HashMap<&NodeId, &InputSpec>,
        transform_nodes: &HashMap<&NodeId, &TransformNodeSpec>,
    ) -> Result<(), SpecValidationError> {
        let mut visited: HashSet<&NodeId> = HashSet::new();

        fn visit<'a>(
            node: &'a NodeId,
            input_nodes: &HashMap<&NodeId, &InputSpec>,
            transform_nodes: &'a HashMap<&NodeId, &TransformNodeSpec>,
            visited: &mut HashSet<&'a NodeId>,
        ) {
            if input_nodes.contains_key(node) {
                visited.insert(node);
                return;
            }

            let transform_node = transform_nodes.get(node).unwrap();

            if visited.contains(node) {
                return;
            }

            for child in &transform_node.input_pads {
                visit(child, input_nodes, transform_nodes, visited);
            }

            visited.insert(node);
        }

        for output in &self.outputs {
            visit(
                &output.input_pad,
                input_nodes,
                transform_nodes,
                &mut visited,
            )
        }

        let nodes: HashSet<&NodeId> = input_nodes
            .keys()
            .chain(transform_nodes.keys())
            .copied()
            .collect();

        let unused_transforms: HashSet<Arc<str>> = nodes
            .difference(&visited)
            .map(|node| node.0.clone())
            .collect();

        if !unused_transforms.is_empty() {
            return Err(SpecValidationError::UnusedNodes(unused_transforms));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test;
