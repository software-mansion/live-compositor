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
    Stretch,
    CropScale,
    FillScale(RGBAColor),
}
