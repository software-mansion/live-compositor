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
                    return Err(InvalidBuiltinTransformationSpec::InvalidInputsCount {
                        node: node_id.clone(),
                        expected_inputs_count: 1,
                        specified_inputs_count: inputs.len() as u32,
                    });
                }

                Ok(())
            }
            BuiltinSpec::FixedPositionLayout {
                texture_layouts, ..
            } => {
                if texture_layouts.len() != inputs.len() {
                    return Err(
                        InvalidBuiltinTransformationSpec::InvalidTextureLayoutsCount {
                            node: node_id.clone(),
                            texture_layouts_count: texture_layouts.len() as u32,
                            input_pads_count: inputs.len() as u32,
                        },
                    );
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
    #[error("Invalid texture layouts for node: {node}. Texture layouts should specify TextureLayout for every input pad. Received: {texture_layouts_count}, expected: {input_pads_count}.")]
    InvalidTextureLayoutsCount {
        node: NodeId,
        texture_layouts_count: u32,
        input_pads_count: u32,
    },
    #[error("Node: {node} expected {expected_inputs_count} input pads, received {specified_inputs_count} input pads.")]
    InvalidInputsCount {
        node: NodeId,
        expected_inputs_count: u32,
        specified_inputs_count: u32,
    },
}
