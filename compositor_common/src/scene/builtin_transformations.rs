use serde::{Deserialize, Serialize};

use crate::util::RGBAColor;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "transformation", rename_all = "snake_case")]
pub enum BuiltinTransformation {
    TransformToResolution(TransformToResolution),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "strategy", content = "background_color_rgba", rename_all = "snake_case")]
pub enum TransformToResolution {
    /// Rescales input in both axis to match output resolution
    Stretch,
    /// Scales input preserving aspect ratio and cuts equal parts
    /// from both sides in "sticking out" dimension
    Fill,
    /// Scales input preserving aspect ratio and
    /// fill the rest of the texture with the provided color
    Fit(RGBAColor),
}
