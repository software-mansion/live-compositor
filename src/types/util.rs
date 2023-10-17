use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalAlign {
    Left,
    Right,
    Justified,
    Center,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
    Justified,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum Coord {
    Number(i32),
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Degree(pub f64);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RGBColor(pub String);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RGBAColor(pub String);
