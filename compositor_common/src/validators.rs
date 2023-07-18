use std::{collections::HashSet, sync::Arc};

use log::info;

use crate::scene::{NodeId, SceneSpec};

#[derive(Debug, thiserror::Error)]
pub enum SpecValidationError {
    #[error("missing node with id {0} used in transformation {1} is not defined in scene and it was not registered as an input")]
    MissingInputNodeForTransformationError(Arc<str>, Arc<str>),
    #[error("missing node with id {0} used in output {1} is not defined in scene and it was not registered as an input")]
    MissingInputNodeForOutputError(Arc<str>, Arc<str>),
    #[error("unknown output, output with id {0} is not registered currently")]
    UnknownOutputError(Arc<str>),
    #[error("unknown intput, input with id {0} is not registered currently")]
    UnknownInputError(Arc<str>),
}

impl SceneSpec {
    // Validate if SceneSpec represents valid scene:
    // - check if each transform have inputs that are either registered or are a transformation
    // itself
    // - check if each input pad of each output is a either registered input or a transformation
    // - check if each output in scene spec is registered output
    // - check if each input in scene spec is registered input
    //
    // TODO: check for cycles
    pub fn validate(
        &self,
        registered_inputs: &HashSet<NodeId>,
        registered_outputs: &HashSet<NodeId>,
    ) -> Result<(), SpecValidationError> {
        let defined_transforms: HashSet<NodeId> =
            self.transforms.iter().map(|i| i.node_id.clone()).collect();
        let defined_inputs: HashSet<NodeId> =
            self.inputs.iter().map(|i| i.input_id.0.clone()).collect();
        info!("{:?}", defined_transforms);
        for t in &self.transforms {
            for input in &t.input_pads {
                if defined_inputs
                    .get(&input)
                    .or(defined_transforms.get(input))
                    .is_none()
                {
                    return Err(SpecValidationError::MissingInputNodeForTransformationError(
                        input.0.clone(),
                        t.node_id.0.clone(),
                    ));
                }
            }
        }
        for out in &self.outputs {
            let node_id = &out.input_pad;
            if defined_inputs
                .get(&node_id)
                .or(defined_transforms.get(&node_id))
                .is_none()
            {
                return Err(SpecValidationError::MissingInputNodeForOutputError(
                    out.input_pad.0.clone(),
                    node_id.0.clone(),
                ));
            }
            if registered_outputs.get(&out.output_id.0).is_none() {
                return Err(SpecValidationError::UnknownOutputError(node_id.0.clone()));
            }
        }

        for input in defined_inputs.iter() {
            if registered_inputs.get(input).is_none() {
                return Err(SpecValidationError::UnknownInputError(input.0.clone()));
            }
        }

        Ok(())
    }
}
