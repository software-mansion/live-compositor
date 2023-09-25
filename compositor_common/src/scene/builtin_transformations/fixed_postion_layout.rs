use serde::{Deserialize, Serialize};

use crate::{
    scene::{transition::TransitionValidationError, Resolution},
    util::{colors::RGBAColor, coord::Coord, degree::Degree},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TextureLayout {
    pub top: Option<Coord>,
    pub bottom: Option<Coord>,
    pub left: Option<Coord>,
    pub right: Option<Coord>,
    #[serde(default)]
    pub rotation: Degree,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FixedPositionLayoutSpec {
    pub resolution: Resolution,
    pub texture_layouts: Vec<TextureLayout>,
    #[serde(default)]
    pub background_color_rgba: RGBAColor,
}

impl FixedPositionLayoutSpec {
    pub(crate) fn validate_transition(
        start: &Self,
        end: &Self,
    ) -> Result<(), TransitionValidationError> {
        let transformation = "fixed_position_layout";
        if start.resolution != end.resolution {
            return Err(TransitionValidationError::UnsupportedFieldInterpolation(
                "resolution",
                transformation,
            ));
        }
        if start.background_color_rgba != end.background_color_rgba {
            return Err(TransitionValidationError::UnsupportedFieldInterpolation(
                "background_color_rgba",
                transformation,
            ));
        }
        if start.texture_layouts.len() != end.texture_layouts.len() {
            return Err(TransitionValidationError::StructureMismatch(
                "\"texture_layouts\" needs to be the same length.",
            ));
        }

        Ok(())
    }
}
