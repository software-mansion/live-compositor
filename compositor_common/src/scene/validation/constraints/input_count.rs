use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{error::UnsatisfiedConstraintsError, scene::NodeSpec};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InputsCountConstraint {
    Exactly(u32),
    // TODO validate that lower bound <= upper bound
    Range { lower_bound: u32, upper_bound: u32 },
}

impl InputsCountConstraint {
    pub fn check(&self, node_spec: &NodeSpec) -> Result<(), UnsatisfiedConstraintsError> {
        let defined_input_pads_count = node_spec.input_pads.len() as u32;
        let is_valid = match self {
            InputsCountConstraint::Exactly(expected_inputs_count) => {
                defined_input_pads_count == *expected_inputs_count
            }
            InputsCountConstraint::Range {
                lower_bound,
                upper_bound,
            } => *lower_bound < defined_input_pads_count && defined_input_pads_count < *upper_bound,
        };

        if is_valid {
            Ok(())
        } else {
            Err(UnsatisfiedConstraintsError::InvalidInputsCount {
                node_id: node_spec.node_id.clone(),
                identification_name: node_spec.identification_name(),
                input_count_constrain: self.clone(),
                defined_input_pads_count,
            })
        }
    }

    pub fn required_inputs_message(&self) -> Rc<str> {
        match &self {
            InputsCountConstraint::Exactly(expected) if *expected == 0 => Rc::from("no input pads"),
            InputsCountConstraint::Exactly(expected) if *expected == 1 => Rc::from("one input pad"),
            InputsCountConstraint::Exactly(expected) => {
                Rc::from(format!("exactly {} input pads", expected))
            }
            InputsCountConstraint::Range {
                lower_bound: minimal,
                upper_bound: maximal,
            } => Rc::from(format!(
                "at least {} and at most {} input pads",
                minimal, maximal
            )),
        }
    }
}
