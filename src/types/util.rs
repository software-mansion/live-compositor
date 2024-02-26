use std::{fmt::Display, str::FromStr};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

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
    pub easing_function: Option<EasingFunctionWrapper>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(transparent)]
pub struct EasingFunctionWrapper(
    #[serde(deserialize_with = "deserialize_easing_function")] pub EasingFunction,
);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "function_name", rename_all = "snake_case")]
pub enum EasingFunction {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    Bounce,
    CubicBezier { points: [f64; 4] },
}

impl Default for EasingFunction {
    fn default() -> Self {
        Self::Linear
    }
}

fn deserialize_easing_function<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = TypeError>,
    D: Deserializer<'de>,
{
    struct Visitor<T>(std::marker::PhantomData<T>);

    impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
    where
        T: Deserialize<'de> + FromStr<Err = TypeError>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or struct")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: serde::de::Error,
        {
            FromStr::from_str(value).map_err(|err| E::custom(err))
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(Visitor(std::marker::PhantomData))
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
