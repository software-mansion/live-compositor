use crate::{
    scene::{transition::TransitionValidationError, Resolution},
    util::{colors::RGBAColor, coord::Coord, degree::Degree},
};

#[derive(Debug, Clone, PartialEq)]
pub struct TextureLayout {
    pub top: Option<Coord>,
    pub bottom: Option<Coord>,
    pub left: Option<Coord>,
    pub right: Option<Coord>,
    pub scale: f32,
    pub rotation: Degree,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixedPositionLayoutSpec {
    pub resolution: Resolution,
    pub texture_layouts: Vec<TextureLayout>,
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
