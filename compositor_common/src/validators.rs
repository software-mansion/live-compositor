use std::{collections::HashSet, sync::Arc};

use crate::scene::{NodeId, SceneSpec};

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
    // TODO: check for cycles, check nodes ids uniqueness, check for unused nodes, ...
    pub fn validate(
        &self,
        registered_inputs: &HashSet<NodeId>,
        registered_outputs: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        let transform_iter = self.transforms.iter().map(|i| i.node_id.clone());
        let input_iter = self.inputs.iter().map(|i| i.input_id.0.clone());
        let defined_node_ids: HashSet<NodeId> = transform_iter.chain(input_iter).collect();

        self.validate_inputs(registered_inputs)?;
        self.validate_transforms(&defined_node_ids)?;
        self.validate_outputs(registered_outputs, &defined_node_ids)?;

        Ok(())
    }

    pub fn validate_transforms(
        &self,
        defined_node_ids: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        for t in self.transforms.iter() {
            for input in &t.input_pads {
                if defined_node_ids.get(input).is_none() {
                    return Err(SpecValidationError::MissingInputNodeForTransformation {
                        missing_node: input.0.clone(),
                        transformation: t.node_id.0.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn validate_inputs(
        &self,
        registered_inputs: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        for input in self.inputs.iter() {
            if registered_inputs.get(&input.input_id.0).is_none() {
                return Err(SpecValidationError::UnknownInput(
                    input.input_id.0 .0.clone(),
                ));
            }
        }

        Ok(())
    }

    pub fn validate_outputs(
        &self,
        registered_outputs: &HashSet<NodeId>,
        defined_node_ids: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        for out in self.outputs.iter() {
            let node_id = &out.input_pad;
            if defined_node_ids.get(node_id).is_none() {
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
}
