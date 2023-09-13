use std::rc::Rc;

use crate::{error::UnsatisfiedConstraintsError, scene::NodeSpec};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InputsCountConstraint {
    Exact(u32),
    Bounded { minimal: u32, maximal: u32 },
}

impl InputsCountConstraint {
    pub fn validate(&self, node_spec: &NodeSpec) -> Result<(), UnsatisfiedConstraintsError> {
        let defined_input_pads_count = node_spec.input_pads.len() as u32;
        let is_valid = match self {
            InputsCountConstraint::Exact(expected_inputs_count) => {
                defined_input_pads_count == *expected_inputs_count
            }
            InputsCountConstraint::Bounded { minimal, maximal } => {
                *minimal < defined_input_pads_count && defined_input_pads_count < *maximal
            }
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
            InputsCountConstraint::Exact(expected) if *expected == 0 => Rc::from("no input pads"),
            InputsCountConstraint::Exact(expected) if *expected == 1 => Rc::from("one input pad"),
            InputsCountConstraint::Exact(expected) => {
                Rc::from(format!("exactly {} input pads", expected))
            }
            InputsCountConstraint::Bounded { minimal, maximal } => Rc::from(format!(
                "at least {} and at most {} input pads",
                minimal, maximal
            )),
        }
    }
}
