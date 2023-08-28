use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::scene::{
    builtin_transformations::BuiltinTransformationSpec, NodeId, NodeParams, NodeSpec, OutputId,
    SceneSpec,
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SpecValidationError {
    #[error("Unknown node with id {missing_node} used in node {node}. Node is not defined in the scene and it was not registered as an input.")]
    UnknownInputPadOnNode { missing_node: NodeId, node: NodeId },
    #[error("Unknown node with id {missing_node} used in output {output} is not defined in scene and it was not registered as an input")]
    UnknownInputPadOnOutput {
        missing_node: NodeId,
        output: OutputId,
    },
    #[error("Unknown output. Output with id {0} is not currently registered.")]
    UnknownOutput(NodeId),
    #[error("Duplicate node id: {0}. There is more than one node or input with the same name.")]
    DuplicateNames(NodeId),
    #[error("detected cycle in scene graph, scene should acyclic graph")]
    CycleDetected,
    #[error("Unused nodes: {0:?}")]
    UnusedNodes(HashSet<NodeId>),
    #[error(
        "Invalid builtin transformation parameters for node {0}. Provided spec: {1:?} Error: {2}"
    )]
    InvalidBuiltinParams(NodeId, BuiltinTransformationSpec, Arc<str>),
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
        let transform_iter = self.nodes.iter().map(|i| &i.node_id);
        let defined_node_ids_iter =
            Iterator::chain(transform_iter, registered_inputs.iter().copied());
        let defined_node_ids: HashSet<&NodeId> = defined_node_ids_iter.clone().collect();

        let transform_nodes: HashMap<&NodeId, &NodeSpec> = self
            .nodes
            .iter()
            .map(|node| (&node.node_id, node))
            .collect();

        self.validate_input_pads_are_defined_on_node(&defined_node_ids)?;
        self.validate_input_pads_are_defined_on_output(&defined_node_ids)?;
        self.validate_outputs_registered(registered_outputs)?;
        self.validate_node_ids_uniqueness(defined_node_ids_iter)?;
        self.validate_cycles(&transform_nodes)?;
        self.validate_nodes_are_used(&transform_nodes)?;
        self.validate_builtin_transformations(&transform_nodes)?;

        Ok(())
    }

    fn validate_input_pads_are_defined_on_node(
        &self,
        defined_node_ids: &HashSet<&NodeId>,
    ) -> Result<(), SpecValidationError> {
        for t in self.nodes.iter() {
            for input in &t.input_pads {
                if !defined_node_ids.contains(input) {
                    return Err(SpecValidationError::UnknownInputPadOnNode {
                        missing_node: input.clone(),
                        node: t.node_id.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_input_pads_are_defined_on_output(
        &self,
        defined_node_ids: &HashSet<&NodeId>,
    ) -> Result<(), SpecValidationError> {
        // TODO: We want to stop allowing connecting inputs to outputs
        for out in self.outputs.iter() {
            let node_id = &out.input_pad;
            if !defined_node_ids.contains(node_id) {
                return Err(SpecValidationError::UnknownInputPadOnOutput {
                    missing_node: out.input_pad.clone(),
                    output: out.output_id.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_outputs_registered(
        &self,
        registered_outputs: &HashSet<&NodeId>,
    ) -> Result<(), SpecValidationError> {
        for out in self.outputs.iter() {
            if registered_outputs.get(&out.output_id.0).is_none() {
                return Err(SpecValidationError::UnknownOutput(out.output_id.0.clone()));
            }
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
                return Err(SpecValidationError::DuplicateNames(node_id.clone()));
            }
        }

        Ok(())
    }

    /// Assumes that all input pads refer to real nodes
    /// [`validate_input_pads_are_defined_on_node`] should be run before this function
    fn validate_cycles(
        &self,
        transform_nodes: &HashMap<&NodeId, &NodeSpec>,
    ) -> Result<(), SpecValidationError> {
        enum NodeState {
            BeingVisited,
            Visited,
        }

        let mut visited: HashMap<&NodeId, NodeState> = HashMap::new();

        fn visit<'a>(
            node_id: &'a NodeId,
            transform_nodes: &'a HashMap<&NodeId, &NodeSpec>,
            visited: &mut HashMap<&'a NodeId, NodeState>,
        ) -> Result<(), SpecValidationError> {
            let Some(node) = transform_nodes.get(node_id) else {
                return Ok(());
            };

            match visited.get(node_id) {
                Some(NodeState::BeingVisited) => return Err(SpecValidationError::CycleDetected),
                Some(NodeState::Visited) => return Ok(()),
                None => {}
            }

            visited.insert(node_id, NodeState::BeingVisited);

            for child in &node.input_pads {
                visit(child, transform_nodes, visited)?;
            }

            visited.insert(node_id, NodeState::Visited);

            Ok(())
        }

        for output in &self.outputs {
            visit(&output.input_pad, transform_nodes, &mut visited)?;
        }

        Ok(())
    }

    /// Assumes that all input pads refer to real nodes
    /// [`validate_input_pads_are_defined_on_node`] should be run before this function
    fn validate_nodes_are_used(
        &self,
        transform_nodes: &HashMap<&NodeId, &NodeSpec>,
    ) -> Result<(), SpecValidationError> {
        let mut visited: HashSet<&NodeId> = HashSet::new();

        fn visit<'a>(
            node_id: &'a NodeId,
            nodes: &'a HashMap<&NodeId, &NodeSpec>,
            visited: &mut HashSet<&'a NodeId>,
        ) {
            let Some(node) = nodes.get(node_id) else {
                return;
            };

            if visited.contains(node_id) {
                return;
            }

            for child in &node.input_pads {
                visit(child, nodes, visited);
            }

            visited.insert(node_id);
        }

        for output in &self.outputs {
            visit(&output.input_pad, transform_nodes, &mut visited)
        }

        let nodes_ids: HashSet<&NodeId> = transform_nodes.keys().copied().collect();
        let mut unused_transforms = nodes_ids.difference(&visited).peekable();

        if unused_transforms.peek().is_some() {
            let unused_transforms = unused_transforms.copied().cloned().collect();
            return Err(SpecValidationError::UnusedNodes(unused_transforms));
        }

        Ok(())
    }

    fn validate_builtin_transformations(
        &self,
        transform_nodes: &HashMap<&NodeId, &NodeSpec>,
    ) -> Result<(), SpecValidationError> {
        for (&node_id, &node_spec) in transform_nodes {
            if let NodeParams::Builtin { transformation, .. } = &node_spec.params {
                match transformation {
                    BuiltinTransformationSpec::TransformToResolution(_) => {}
                    BuiltinTransformationSpec::FixedPositionLayout {
                        textures_layouts, ..
                    } => {
                        if node_spec.input_pads.len() != textures_layouts.len() {
                            return Err(SpecValidationError::InvalidBuiltinParams(
                                node_id.clone(),
                                transformation.clone(),
                                Arc::from("input_pads length should match textures_layouts length"),
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test;
