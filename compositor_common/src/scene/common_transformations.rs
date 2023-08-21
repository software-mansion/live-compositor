use serde::{Deserialize, Serialize};

use crate::util::RGBAColor;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum CommonTransformation {
    ConvertResolution(ConvertResolutionParams),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "strategy", content = "color", rename_all = "snake_case")]
pub enum ConvertResolutionParams {
    /// Rescales input in both axis to match output resolution
    Stretch,
    /// Scales input preserving aspect ratio and cuts equal parts 
    /// from both sides in "sticking out" dimension
    CropScale,
    /// Scales input preserving aspect ratio and 
    /// fill the rest of the texture with the provided color
    FillScale(RGBAColor),
}
