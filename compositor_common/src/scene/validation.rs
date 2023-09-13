use std::collections::{HashMap, HashSet};

use crate::error::{SceneSpecValidationError, UnusedNodesError};

use super::{NodeId, NodeSpec, OutputSpec, SceneSpec};

pub mod constraints;
pub mod inputs;

impl SceneSpec {
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

        Self::validate_input_pads_are_defined_on_node(&self.nodes, &defined_node_ids)?;
        Self::validate_input_pads_are_defined_on_output(&self.outputs, &defined_node_ids)?;
        Self::validate_outputs_registered(&self.outputs, registered_outputs)?;
        Self::validate_node_ids_uniqueness(defined_node_ids_iter, registered_inputs)?;
        Self::validate_cycles(&self.outputs, &transform_nodes)?;
        Self::validate_nodes_are_used(&self.outputs, &transform_nodes)?;
        Self::validate_node_params(&self.nodes)?;

        Ok(())
    }

    fn validate_input_pads_are_defined_on_node(
        nodes: &[NodeSpec],
        defined_node_ids: &HashSet<&NodeId>,
    ) -> Result<(), SceneSpecValidationError> {
        for t in nodes.iter() {
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
        outputs: &[OutputSpec],
        defined_node_ids: &HashSet<&NodeId>,
    ) -> Result<(), SceneSpecValidationError> {
        for out in outputs.iter() {
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
        outputs: &[OutputSpec],
        registered_outputs: &HashSet<&NodeId>,
    ) -> Result<(), SceneSpecValidationError> {
        for out in outputs.iter() {
            if registered_outputs.get(&out.output_id.0).is_none() {
                return Err(SceneSpecValidationError::UnknownOutput(
                    out.output_id.0.clone(),
                ));
            }
        }

        Ok(())
    }

    fn validate_node_ids_uniqueness<'a, I: Iterator<Item = &'a NodeId>>(
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
        outputs: &Vec<OutputSpec>,
        nodes: &HashMap<&NodeId, &NodeSpec>,
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

        for output in outputs {
            visit(&output.input_pad, nodes, &mut visited)?;
        }

        Ok(())
    }

    /// Assumes that all input pads refer to real nodes
    /// [`validate_input_pads_are_defined_on_node`] should be run before this function
    fn validate_nodes_are_used(
        outputs: &Vec<OutputSpec>,
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

        for output in outputs {
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

    fn validate_node_params(nodes: &Vec<NodeSpec>) -> Result<(), SceneSpecValidationError> {
        for spec in nodes {
            spec.validate_params().map_err(|err| {
                SceneSpecValidationError::InvalidNodeSpec(err, spec.node_id.clone())
            })?;
        }

        Ok(())
    }
}
