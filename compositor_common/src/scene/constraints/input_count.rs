use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
    error::{InputsCountConstraintValidationError, UnsatisfiedConstraintsError},
    scene::NodeSpec,
};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InputsCountConstraint {
    Exact { fixed_count: u32 },
    // TODO validate that lower bound <= upper bound
    Range { lower_bound: u32, upper_bound: u32 },
}

impl InputsCountConstraint {
    pub fn check(&self, node_spec: &NodeSpec) -> Result<(), UnsatisfiedConstraintsError> {
        let defined_input_pads_count = node_spec.input_pads.len() as u32;
        let is_valid = match self {
            InputsCountConstraint::Exact { fixed_count } => {
                defined_input_pads_count == *fixed_count
            }
            InputsCountConstraint::Range {
                lower_bound,
                upper_bound,
            } => {
                *lower_bound <= defined_input_pads_count && defined_input_pads_count <= *upper_bound
            }
        };

        if is_valid {
            Ok(())
        } else {
            Err(UnsatisfiedConstraintsError::InvalidInputsCount(
                InputsCountConstraintValidationError {
                    node_identifier: (&node_spec.params).into(),
                    input_count_constrain: self.clone(),
                    defined_input_pads_count,
                },
            ))
        }
    }

    pub fn required_inputs_message(&self) -> Rc<str> {
        match &self {
            InputsCountConstraint::Exact { fixed_count } if *fixed_count == 0 => {
                Rc::from("no input pads")
            }
            InputsCountConstraint::Exact { fixed_count } if *fixed_count == 1 => {
                Rc::from("one input pad")
            }
            InputsCountConstraint::Exact { fixed_count } => {
                Rc::from(format!("exactly {} input pads", fixed_count))
            }
            InputsCountConstraint::Range {
                lower_bound,
                upper_bound,
            } => Rc::from(format!(
                "at least {} and at most {} input pads",
                lower_bound, upper_bound
            )),
        }
    }
}
