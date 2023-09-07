use serde::{Deserialize, Serialize};

use crate::{
    error::BuiltinSpecValidationError,
    util::{Coord, Degree, RGBAColor},
};

use super::NodeSpec;
use super::Resolution;

pub const GRID_MAX_INPUTS_COUNT: u32 = 16;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "transformation", rename_all = "snake_case")]
pub enum BuiltinSpec {
    TransformToResolution {
        resolution: Resolution,
        #[serde(flatten)]
        strategy: TransformToResolutionStrategy,
    },
    FixedPositionLayout {
        resolution: Resolution,
        texture_layouts: Vec<TextureLayout>,
        #[serde(default)]
        background_color_rgba: RGBAColor,
    },
    Grid {
        #[serde(default)]
        background_color_rgba: RGBAColor,
        #[serde(default = "default_tile_aspect_ratio")]
        tile_aspect_ratio: (u32, u32),
        resolution: Resolution,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum TransformToResolutionStrategy {
    /// Rescales input in both axis to match output resolution
    Stretch,
    /// Scales input preserving aspect ratio and cuts equal parts
    /// from both sides in "sticking out" dimension
    Fill,
    /// Scales input preserving aspect ratio and
    /// fill the rest of the texture with the provided color]
    Fit {
        #[serde(default)]
        background_color_rgba: RGBAColor,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TextureLayout {
    pub top: Option<Coord>,
    pub bottom: Option<Coord>,
    pub left: Option<Coord>,
    pub right: Option<Coord>,
    #[serde(default)]
    pub rotation: Degree,
}

impl BuiltinSpec {
    pub fn validate(&self, node_spec: &NodeSpec) -> Result<(), BuiltinSpecValidationError> {
        match self {
            BuiltinSpec::TransformToResolution { .. } => {
                if node_spec.input_pads.len() != 1 {
                    return Err(BuiltinSpecValidationError::TransformToResolutionExactlyOneInput);
                }

                Ok(())
            }
            BuiltinSpec::FixedPositionLayout {
                texture_layouts, ..
            } => {
                if texture_layouts.len() != node_spec.input_pads.len() {
                    return Err(BuiltinSpecValidationError::FixedLayoutInvalidLayoutCount {
                        layout_count: texture_layouts.len() as u32,
                        input_count: node_spec.input_pads.len() as u32,
                    });
                }
                for layout in texture_layouts {
                    match layout {
                        TextureLayout {
                            top: None,
                            bottom: None,
                            ..
                        } => return Err(BuiltinSpecValidationError::FixedLayoutTopBottomRequired),
                        TextureLayout {
                            top: Some(_),
                            bottom: Some(_),
                            ..
                        } => return Err(BuiltinSpecValidationError::FixedLayoutTopBottomOnlyOne),
                        _ => (),
                    };
                    match layout {
                        TextureLayout {
                            left: None,
                            right: None,
                            ..
                        } => return Err(BuiltinSpecValidationError::FixedLayoutLeftRightRequired),
                        TextureLayout {
                            left: Some(_),
                            right: Some(_),
                            ..
                        } => return Err(BuiltinSpecValidationError::FixedLayoutLeftRightOnlyOne),
                        _ => (),
                    };
                }
                Ok(())
            }
            BuiltinSpec::Grid { .. } => {
                if node_spec.input_pads.is_empty() {
                    return Err(BuiltinSpecValidationError::GridNoInputs);
                }

                if node_spec.input_pads.len() > 16 {
                    return Err(BuiltinSpecValidationError::GridTooManyInputs);
                }

                Ok(())
            },
        }
    }
}

fn default_tile_aspect_ratio() -> (u32, u32) {
    (16, 9)
}
