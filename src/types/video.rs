use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Component;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    /// H264 video.
    H264,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Video {
    pub root: Component,
}
