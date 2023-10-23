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
pub use fixed_postion_layout::HorizontalPosition;
pub use fixed_postion_layout::TextureLayout;
pub use fixed_postion_layout::VerticalPosition;

pub const TILED_LAYOUT_MAX_INPUTS_COUNT: u32 = 16;
pub const FIXED_POSITION_LAYOUT_MAX_INPUTS_COUNT: u32 = 16;

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinSpec {
    FitToResolution(FitToResolutionSpec),
    FillToResolution { resolution: Resolution },
    StretchToResolution { resolution: Resolution },
    FixedPositionLayout(FixedPositionLayoutSpec),
    TiledLayout(TiledLayoutSpec),
    MirrorImage { mode: MirrorMode },
    CornersRounding { border_radius: Coord },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FitToResolutionSpec {
    pub resolution: Resolution,
    pub background_color_rgba: RGBAColor,
    pub horizontal_alignment: HorizontalAlign,
    pub vertical_alignment: VerticalAlign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorMode {
    Horizontal,
    Vertical,
    HorizontalAndVertical,
}

lazy_static! {
    static ref FIT_TO_RESOLUTION_CONSTRAINTS: NodeConstraints =
        NodeConstraints(vec![Constraint::InputCount(InputCountConstraint::Exact {
            fixed_count: 1
        })]);
    static ref FILL_TO_RESOLUTION_CONSTRAINTS: NodeConstraints =
        NodeConstraints(vec![Constraint::InputCount(InputCountConstraint::Exact {
            fixed_count: 1
        })]);
    static ref STRETCH_TO_RESOLUTION_CONSTRAINTS: NodeConstraints =
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
            BuiltinSpec::FixedPositionLayout { .. } => "fixed_position_layout",
            BuiltinSpec::TiledLayout { .. } => "tiled_layout",
            BuiltinSpec::MirrorImage { .. } => "mirror_image",
            BuiltinSpec::CornersRounding { .. } => "corners_rounding",
            BuiltinSpec::FitToResolution(_) => "fit_to_resolution",
            BuiltinSpec::FillToResolution { .. } => "fill_to_resolution",
            BuiltinSpec::StretchToResolution { .. } => "stretch_to_resolution",
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
                Ok(())
            }
            BuiltinSpec::TiledLayout { .. }
            | BuiltinSpec::MirrorImage { .. }
            | BuiltinSpec::CornersRounding { .. }
            | BuiltinSpec::FitToResolution(_)
            | BuiltinSpec::FillToResolution { .. }
            | BuiltinSpec::StretchToResolution { .. } => Ok(()),
        }
    }

    pub fn constraints(&self) -> &'static NodeConstraints {
        match self {
            BuiltinSpec::FixedPositionLayout { .. } => &FIXED_POSITION_LAYOUT_CONSTRAINTS,
            BuiltinSpec::TiledLayout { .. } => &TILED_LAYOUT_CONSTRAINTS,
            BuiltinSpec::MirrorImage { .. } => &MIRROR_IMAGE_CONSTRAINTS,
            BuiltinSpec::CornersRounding { .. } => &CORNERS_ROUNDING_CONSTRAINTS,
            BuiltinSpec::FitToResolution(_) => &FIT_TO_RESOLUTION_CONSTRAINTS,
            BuiltinSpec::FillToResolution { .. } => &FILL_TO_RESOLUTION_CONSTRAINTS,
            BuiltinSpec::StretchToResolution { .. } => &STRETCH_TO_RESOLUTION_CONSTRAINTS,
        }
    }
}
