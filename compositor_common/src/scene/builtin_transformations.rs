use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::util::{Coord, RGBAColor};

use super::{NodeId, Resolution};

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
}

impl BuiltinSpec {
    pub fn validate(
        &self,
        node_id: &NodeId,
        inputs: &Vec<NodeId>,
    ) -> Result<(), InvalidBuiltinTransformationSpec> {
        match self {
            BuiltinSpec::TransformToResolution { .. } => {
                if inputs.len() != 1 {
                    return Err(
                        InvalidBuiltinTransformationSpec::TransformToResolutionExactlyOneInput(
                            node_id.clone(),
                        ),
                    );
                }

                Ok(())
            }
            BuiltinSpec::FixedPositionLayout {
                texture_layouts, ..
            } => {
                if texture_layouts.len() != inputs.len() {
                    return Err(InvalidTextureLayoutsCount {
                        node: node_id.clone(),
                        texture_layouts_count: texture_layouts.len() as u32,
                        input_pads_count: inputs.len() as u32,
                    }
                    .into());
                }
                Ok(())
            }
        }
    }
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
    pub top: Coord,
    pub left: Coord,
    #[serde(default)]
    pub rotation: Degree,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Degree(pub i32);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InvalidBuiltinTransformationSpec {
    #[error(transparent)]
    InvalidTextureLayoutsCount(#[from] InvalidTextureLayoutsCount),
    #[error("Invalid node {0} specification. Nodes that use transformation \"transform_to_resolution\" need to have exactly one input pad.")]
    TransformToResolutionExactlyOneInput(NodeId),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub struct InvalidTextureLayoutsCount {
    node: NodeId,
    texture_layouts_count: u32,
    input_pads_count: u32,
}

impl Display for InvalidTextureLayoutsCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            node,
            texture_layouts_count,
            input_pads_count,
        } = self;
        if self.texture_layouts_count > self.input_pads_count {
            write!(f, "Too many texture layouts defined in node \"{node}\". There are {input_pads_count} input pads, but {texture_layouts_count} texture layouts, were provided.")
        } else {
            write!(f, "Missing texture layouts in node \"{node}\". There are {input_pads_count} input pads, but only {texture_layouts_count} texture layouts were provided.")
        }
    }
}
