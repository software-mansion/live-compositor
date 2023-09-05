use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::scene::{
    builtin_transformations::InvalidBuiltinTransformationSpec, NodeId, NodeParams, NodeSpec,
    OutputId, SceneSpec,
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SceneSpecValidationError {
    #[error("Unknown node \"{missing_node}\" used as an input in the node \"{node}\". Node is not defined in the scene and it was not registered as an input.")]
    UnknownInputPadOnNode { missing_node: NodeId, node: NodeId },
    #[error("Unknown node \"{missing_node}\" is connected to the output stream \"{output}\".")]
    UnknownInputPadOnOutput {
        missing_node: NodeId,
        output: OutputId,
    },
    #[error(
        "Unknown output stream \"{0}\". Register it first before using it in the scene definition."
    )]
    UnknownOutput(NodeId),
    #[error("Invalid node id. There is more than one node with the \"{0}\" id.")]
    DuplicateNodeNames(NodeId),
    #[error("Invalid node id. There is already an input stream with the \"{0}\" id.")]
    DuplicateNodeAndInputNames(NodeId),
    #[error("Cycles between nodes are not allowed. Node \"{0}\" depends on itself via input_pads or fallback option.")]
    CycleDetected(NodeId),
    #[error(transparent)]
    UnusedNodes(#[from] UnusedNodesError),
    #[error(transparent)]
    InvalidBuiltinTransformationParams(#[from] InvalidBuiltinTransformationSpec),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub struct UnusedNodesError(HashSet<NodeId>);

impl Display for UnusedNodesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut unused_nodes: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        unused_nodes.sort();
        write!(
            f,
            "There are unused nodes in the scene definition: {0}",
            unused_nodes.join(", ")
        )
    }
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
    ) -> Result<(), SceneSpecValidationError> {
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
        self.validate_node_ids_uniqueness(defined_node_ids_iter, registered_inputs)?;
        self.validate_cycles(&transform_nodes)?;
        self.validate_nodes_are_used(&transform_nodes)?;
        self.validate_builtin_transformations()?;

        Ok(())
    }

    fn validate_input_pads_are_defined_on_node(
        &self,
        defined_node_ids: &HashSet<&NodeId>,
    ) -> Result<(), SceneSpecValidationError> {
        for t in self.nodes.iter() {
            for input in &t.input_pads {
                if !defined_node_ids.contains(input) {
                    return Err(SceneSpecValidationError::UnknownInputPadOnNode {
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
    ) -> Result<(), SceneSpecValidationError> {
        // TODO: We want to stop allowing connecting inputs to outputs
        for out in self.outputs.iter() {
            let node_id = &out.input_pad;
            if !defined_node_ids.contains(node_id) {
                return Err(SceneSpecValidationError::UnknownInputPadOnOutput {
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
    ) -> Result<(), SceneSpecValidationError> {
        for out in self.outputs.iter() {
            if registered_outputs.get(&out.output_id.0).is_none() {
                return Err(SceneSpecValidationError::UnknownOutput(
                    out.output_id.0.clone(),
                ));
            }
        }

        Ok(())
    }

    fn validate_node_ids_uniqueness<'a, I: Iterator<Item = &'a NodeId>>(
        &self,
        defined_node_ids: I,
        registered_inputs: &HashSet<&NodeId>,
    ) -> Result<(), SceneSpecValidationError> {
        let mut nodes_ids: HashSet<&NodeId> = HashSet::new();

        for node_id in defined_node_ids {
            if !nodes_ids.insert(node_id) {
                if registered_inputs.contains(node_id) {
                    return Err(SceneSpecValidationError::DuplicateNodeAndInputNames(
                        node_id.clone(),
                    ));
                } else {
                    return Err(SceneSpecValidationError::DuplicateNodeNames(
                        node_id.clone(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Assumes that all input pads refer to real nodes
    /// [`validate_input_pads_are_defined_on_node`] should be run before this function
    fn validate_cycles(
        &self,
        transform_nodes: &HashMap<&NodeId, &NodeSpec>,
    ) -> Result<(), SceneSpecValidationError> {
        enum NodeState {
            BeingVisited,
            Visited,
        }

        let mut visited: HashMap<&NodeId, NodeState> = HashMap::new();

        fn visit<'a>(
            node_id: &'a NodeId,
            nodes: &'a HashMap<&NodeId, &NodeSpec>,
            visited: &mut HashMap<&'a NodeId, NodeState>,
        ) -> Result<(), SceneSpecValidationError> {
            let Some(node) = nodes.get(node_id) else {
                return Ok(());
            };

            match visited.get(node_id) {
                Some(NodeState::BeingVisited) => {
                    return Err(SceneSpecValidationError::CycleDetected(node_id.clone()))
                }
                Some(NodeState::Visited) => return Ok(()),
                None => {}
            }

            visited.insert(node_id, NodeState::BeingVisited);

            for child in &node.input_pads {
                visit(child, nodes, visited)?;
            }
            if let Some(fallback_id) = &node.fallback_id {
                visit(fallback_id, nodes, visited)?;
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
    ) -> Result<(), SceneSpecValidationError> {
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
            if let Some(fallback_id) = &node.fallback_id {
                visit(fallback_id, nodes, visited);
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
            return Err(UnusedNodesError(unused_transforms).into());
        }

        Ok(())
    }

    fn validate_builtin_transformations(&self) -> Result<(), SceneSpecValidationError> {
        for spec in &self.nodes {
            let NodeSpec {
                node_id,
                input_pads,
                params,
                ..
            } = spec;

            if let NodeParams::Builtin { transformation, .. } = params {
                transformation.validate(node_id, input_pads)?;
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod test;
