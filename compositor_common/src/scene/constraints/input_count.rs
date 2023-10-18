use crate::{
    error::{InputCountConstraintValidationError, UnsatisfiedConstraintsError},
    scene::NodeSpec,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InputCountConstraint {
    Exact { fixed_count: u32 },
    // TODO validate that lower bound <= upper bound
    Range { lower_bound: u32, upper_bound: u32 },
}

impl InputCountConstraint {
    pub fn check(&self, node_spec: &NodeSpec) -> Result<(), UnsatisfiedConstraintsError> {
        let defined_input_pad_count = node_spec.input_pads.len() as u32;
        let is_valid = match self {
            InputCountConstraint::Exact { fixed_count } => defined_input_pad_count == *fixed_count,
            InputCountConstraint::Range {
                lower_bound,
                upper_bound,
            } => *lower_bound <= defined_input_pad_count && defined_input_pad_count <= *upper_bound,
        };

        if is_valid {
            Ok(())
        } else {
            Err(UnsatisfiedConstraintsError::InvalidInputsCount(
                InputCountConstraintValidationError {
                    node_identifier: (&node_spec.params).into(),
                    input_count_constrain: self.clone(),
                    defined_input_pad_count,
                },
            ))
        }
    }
}
