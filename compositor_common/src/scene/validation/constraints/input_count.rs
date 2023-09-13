use std::sync::Arc;

use crate::error::InputsCountValidationError;

pub enum InputsCountConstraint {
    Exact(u32),
    Bounded { minimal: u32, maximal: u32 },
}

impl InputsCountConstraint {
    pub fn validate(
        &self,
        defined_input_pads_count: u32,
        transformation_name: Arc<str>,
    ) -> Result<(), InputsCountValidationError> {
        match self {
            InputsCountConstraint::Exact(expected_input_pads_count) => {
                if *expected_input_pads_count == defined_input_pads_count {
                    Ok(())
                } else {
                    Err(Self::exact_inputs_error(
                        defined_input_pads_count,
                        *expected_input_pads_count,
                        transformation_name,
                    ))
                }
            }
            InputsCountConstraint::Bounded { minimal, maximal } => {
                if *minimal <= defined_input_pads_count && defined_input_pads_count <= *maximal {
                    Ok(())
                } else {
                    Err(InputsCountValidationError::InvalidBoundedInputsRequired {
                        transformation_name,
                        minimal_inputs_expected: *minimal,
                        maximal_inputs_expected: *maximal,
                        defined_input_pads_count,
                    })
                }
            }
        }
    }

    fn exact_inputs_error(
        defined_input_pads_count: u32,
        expected_input_pads_count: u32,
        transformation_name: Arc<str>,
    ) -> InputsCountValidationError {
        if expected_input_pads_count == 0 {
            InputsCountValidationError::NoInputsRequired {
                transformation_name,
                defined_input_pads_count,
            }
        } else if expected_input_pads_count == 1 {
            InputsCountValidationError::ExactlyOneInputRequired {
                transformation_name,
                defined_input_pads_count,
            }
        } else {
            InputsCountValidationError::ExactNumberOfInputsRequired {
                transformation_name,
                expected_input_pads_count,
                defined_input_pads_count,
            }
        }
    }
}
