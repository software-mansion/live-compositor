pub struct InvalidInputsCountError();

#[derive(Debug, PartialEq, Eq)]
pub enum ValidNodeInputsCount {
    Exact(u32),
    Bounded { minimal: u32, maximal: u32 },
}

impl ValidNodeInputsCount {
    pub fn error_message_inputs_count(&self) -> String {
        match self {
            ValidNodeInputsCount::Exact(inputs_count) => format!("exactly {}", inputs_count),
            ValidNodeInputsCount::Bounded { minimal, maximal } => {
                format!("at least {} and at most {}", minimal, maximal)
            }
        }
    }

    pub fn validate(&self, inputs_count: u32) -> Result<(), InvalidInputsCountError> {
        match self {
            ValidNodeInputsCount::Exact(expected) => {
                if *expected == inputs_count {
                    Ok(())
                } else {
                    Err(InvalidInputsCountError())
                }
            }
            ValidNodeInputsCount::Bounded { minimal, maximal } => {
                if *minimal <= inputs_count && inputs_count <= *maximal {
                    Ok(())
                } else {
                    Err(InvalidInputsCountError())
                }
            }
        }
    }
}
