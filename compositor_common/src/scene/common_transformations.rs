use serde::{Deserialize, Serialize};

use crate::util::RGBAColor;

#[derive(Serialize, Deserialize, Clone)]
pub enum CommonTransformation {
    ConvertResolution(ConvertResolutionParams),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ConvertResolutionParams {
    Stretch,
    CropToFit,
    FillToFit(RGBAColor),
}
