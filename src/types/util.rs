use std::fmt::Display;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Resolution {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Transition {
    /// Duration of a transition in milliseconds.
    pub duration_ms: f64,
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
pub struct AspectRatio(pub(super) String);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum Coord {
    Number(i32),
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Degree(pub f64);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum Framerate {
    String(String),
    U32(u32),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
pub struct RGBColor(pub String);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
pub struct RGBAColor(pub String);

#[derive(Debug, PartialEq)]
pub struct TypeError(String);

impl<E> From<E> for TypeError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Self(err.to_string())
    }
}

impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TypeError {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Self(msg.into())
    }
}
