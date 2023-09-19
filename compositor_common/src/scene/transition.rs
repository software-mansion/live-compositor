use std::{f64::consts::PI, time::Duration};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};

use crate::util::InterpolationState;

use super::builtin_transformations::{BuiltinSpec, FixedPositionLayoutSpec};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransitionSpec {
    pub start: BuiltinSpec,
    pub end: BuiltinSpec,
    #[serde_as(as = "DurationMilliSeconds<f64>")]
    #[serde(rename = "transition_duration_ms")]
    pub transition_duration: Duration,
    #[serde(flatten)]
    pub interpolation: Interpolation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "interpolation", rename_all = "snake_case")]
pub enum Interpolation {
    Linear,
    Spring,
}

impl Interpolation {
    pub fn interpolate(&self, state: InterpolationState) -> InterpolationState {
        match self {
            Interpolation::Linear => state,
            Interpolation::Spring => {
                // TODO: placeholder implementation, this needs to be implemented better
                if state.0 < 0.2 {
                    InterpolationState(
                        f64::powf(state.0 * 5.0, 0.3)
                            + f64::exp(-state.0 * 14.0) * (f64::sin(10.0 * PI * state.0)),
                    )
                } else {
                    InterpolationState(
                        1.0 + f64::exp(-state.0 * 14.0) * (f64::sin(10.0 * PI * state.0)),
                    )
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum TransitionType {
    FixedPositionLayout(FixedPositionLayoutSpec, FixedPositionLayoutSpec),
}

impl TransitionType {
    fn validate(&self) -> Result<(), TransitionValidationError> {
        match self {
            TransitionType::FixedPositionLayout(start, end) => {
                FixedPositionLayoutSpec::validate_transition(start, end)
            }
        }
    }
}

impl TryFrom<(&BuiltinSpec, &BuiltinSpec)> for TransitionType {
    type Error = TransitionValidationError;

    fn try_from(value: (&BuiltinSpec, &BuiltinSpec)) -> Result<Self, Self::Error> {
        let transition = match value {
            (BuiltinSpec::FixedPositionLayout(s1), BuiltinSpec::FixedPositionLayout(s2)) => {
                Self::FixedPositionLayout(s1.clone(), s2.clone())
            }
            (start, end) => {
                return Err(TransitionValidationError::IncompatibleStartAndEnd(
                    start.transformation_name(),
                    end.transformation_name(),
                ))
            }
        };
        transition.validate()?;
        Ok(transition)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransitionValidationError {
    #[error("Transition from \"{0}\" to \"{1}\" is not supported.")]
    IncompatibleStartAndEnd(&'static str, &'static str),

    #[error("Field \"{0}\" in transformation \"{1}\" can not be interpolated.")] // TODO: See docs
    UnsupportedFieldInterpolation(&'static str, &'static str),

    #[error("Structure mismatch between start and end definition. {0}.")]
    StructureMismatch(&'static str),
}
