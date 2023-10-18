use crate::error::BuiltinSpecValidationError;
use crate::scene::builtin_transformations::tiled_layout::TiledLayoutSpec;
use crate::scene::constraints::input_count::InputCountConstraint;
use crate::util::align::HorizontalAlign;
use crate::util::align::VerticalAlign;
use crate::util::colors::RGBAColor;
use crate::util::coord::Coord;

use super::constraints::Constraint;
use super::constraints::NodeConstraints;
use super::NodeSpec;
use super::Resolution;

pub(crate) mod fixed_postion_layout;
pub mod tiled_layout;

pub use fixed_postion_layout::FixedPositionLayoutSpec;
pub use fixed_postion_layout::TextureLayout;

pub const TILED_LAYOUT_MAX_INPUTS_COUNT: u32 = 16;
pub const FIXED_POSITION_LAYOUT_MAX_INPUTS_COUNT: u32 = 16;

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinSpec {
    TransformToResolution {
        resolution: Resolution,
        strategy: TransformToResolutionStrategy,
    },
    FixedPositionLayout(FixedPositionLayoutSpec),
    TiledLayout(TiledLayoutSpec),
    MirrorImage {
        mode: MirrorMode,
    },
    CornersRounding {
        border_radius: Coord,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformToResolutionStrategy {
    /// Rescales input in both axis to match output resolution
    Stretch,
    /// Scales input preserving aspect ratio and cuts equal parts
    /// from both sides in "sticking out" dimension
    Fill,
    /// Scales input preserving aspect ratio and
    /// fill the rest of the texture with the provided color]
    Fit {
        background_color_rgba: RGBAColor,
        horizontal_alignment: HorizontalAlign,
        vertical_alignment: VerticalAlign,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorMode {
    Horizontal,
    Vertical,
    HorizontalAndVertical,
}

lazy_static! {
    static ref TRANSFORM_TO_RESOLUTION_CONSTRAINTS: NodeConstraints =
        NodeConstraints(vec![Constraint::InputCount(InputCountConstraint::Exact {
            fixed_count: 1
        })]);
    static ref FIXED_POSITION_LAYOUT_CONSTRAINTS: NodeConstraints =
        NodeConstraints(vec![Constraint::InputCount(InputCountConstraint::Range {
            lower_bound: 1,
            upper_bound: FIXED_POSITION_LAYOUT_MAX_INPUTS_COUNT,
        })]);
    static ref TILED_LAYOUT_CONSTRAINTS: NodeConstraints =
        NodeConstraints(vec![Constraint::InputCount(InputCountConstraint::Range {
            lower_bound: 1,
            upper_bound: FIXED_POSITION_LAYOUT_MAX_INPUTS_COUNT,
        })]);
    static ref MIRROR_IMAGE_CONSTRAINTS: NodeConstraints =
        NodeConstraints(vec![Constraint::InputCount(InputCountConstraint::Exact {
            fixed_count: 1
        })]);
    static ref CORNERS_ROUNDING_CONSTRAINTS: NodeConstraints =
        NodeConstraints(vec![Constraint::InputCount(InputCountConstraint::Exact {
            fixed_count: 1
        })]);
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
            BuiltinSpec::FixedPositionLayout(FixedPositionLayoutSpec {
                texture_layouts, ..
            }) => {
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

    pub fn constraints(&self) -> &'static NodeConstraints {
        match self {
            BuiltinSpec::TransformToResolution { .. } => &TRANSFORM_TO_RESOLUTION_CONSTRAINTS,
            BuiltinSpec::FixedPositionLayout { .. } => &FIXED_POSITION_LAYOUT_CONSTRAINTS,
            BuiltinSpec::TiledLayout { .. } => &TILED_LAYOUT_CONSTRAINTS,
            BuiltinSpec::MirrorImage { .. } => &MIRROR_IMAGE_CONSTRAINTS,
            BuiltinSpec::CornersRounding { .. } => &CORNERS_ROUNDING_CONSTRAINTS,
        }
    }
}
