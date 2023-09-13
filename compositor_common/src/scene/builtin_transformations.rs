use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::error::BuiltinSpecValidationError;
use crate::util::colors::RGBAColor;
use crate::util::coord::Coord;
use crate::util::degree::Degree;

use super::validation::constraints::input_count::InputsCountConstraint;
use super::validation::constraints::NodeConstraints;
use super::NodeSpec;
use super::Resolution;

pub const TILED_LAYOUT_MAX_INPUTS_COUNT: u32 = 16;
pub const FIXED_POSITION_LAYOUT_MAX_INPUTS_COUNT: u32 = 16;

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
    TiledLayout {
        #[serde(default)]
        background_color_rgba: RGBAColor,
        #[serde(default = "default_tile_aspect_ratio")]
        tile_aspect_ratio: (u32, u32),
        resolution: Resolution,
    },
    MirrorImage {
        #[serde(default)]
        mode: MirrorMode,
    },
    CornersRounding {
        border_radius: Coord,
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

#[derive(Debug, SerializeDisplay, DeserializeFromStr, Clone, Copy, PartialEq, Eq, Default)]
pub enum MirrorMode {
    #[default]
    Horizontal,
    Vertical,
    HorizontalAndVertical,
}

#[derive(Debug, thiserror::Error)]
#[error("\"mode\" field can only be set to \"vertical\" or \"horizontal\".")]
pub struct MirrorModeParseError;

impl Display for MirrorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirrorMode::Horizontal => write!(f, "horizontal"),
            MirrorMode::Vertical => write!(f, "vertical"),
            MirrorMode::HorizontalAndVertical => write!(f, "horizontal-vertical"),
        }
    }
}

impl FromStr for MirrorMode {
    type Err = MirrorModeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vertical" => Ok(Self::Vertical),
            "horizontal" => Ok(Self::Horizontal),
            "horizontal-vertical" => Ok(Self::HorizontalAndVertical),
            _ => Err(MirrorModeParseError),
        }
    }
}

impl BuiltinSpec {
    pub fn transformation_name(&self) -> &'static str {
        match self {
            BuiltinSpec::TransformToResolution { .. } => "transform_to_resolution",
            BuiltinSpec::FixedPositionLayout { .. } => "fixed_position_layout",
            BuiltinSpec::TiledLayout { .. } => "tiled_layout",
            BuiltinSpec::MirrorImage { .. } => "mirror_image",
            BuiltinSpec::CornersRounding { .. } => "corners_rounding",
        }
    }

    pub fn validate_params(&self, node_spec: &NodeSpec) -> Result<(), BuiltinSpecValidationError> {
        match self {
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
            BuiltinSpec::TiledLayout { .. }
            | BuiltinSpec::MirrorImage { .. }
            | BuiltinSpec::CornersRounding { .. }
            | BuiltinSpec::TransformToResolution { .. } => Ok(()),
        }
    }

    pub fn constrains(&self) -> NodeConstraints {
        match self {
            BuiltinSpec::TransformToResolution { .. } => NodeConstraints {
                inputs_count: InputsCountConstraint::Exact(1),
            },
            BuiltinSpec::FixedPositionLayout { .. } => NodeConstraints {
                inputs_count: InputsCountConstraint::Bounded {
                    minimal: 1,
                    maximal: FIXED_POSITION_LAYOUT_MAX_INPUTS_COUNT,
                },
            },
            BuiltinSpec::TiledLayout { .. } => NodeConstraints {
                inputs_count: InputsCountConstraint::Bounded {
                    minimal: 1,
                    maximal: TILED_LAYOUT_MAX_INPUTS_COUNT,
                },
            },
            BuiltinSpec::MirrorImage { .. } => NodeConstraints {
                inputs_count: InputsCountConstraint::Exact(1),
            },
            BuiltinSpec::CornersRounding { .. } => NodeConstraints {
                inputs_count: InputsCountConstraint::Exact(1),
            },
        }
    }
}

fn default_tile_aspect_ratio() -> (u32, u32) {
    (16, 9)
}
