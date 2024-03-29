use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::InputId;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Audio {
    pub inputs: Vec<InputAudio>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InputAudio {
    pub input_id: InputId,
    // (**default=`1.0`**) float in [0, 1] range representing input volume
    pub volume: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MixingStrategy {
    /// Firstly, input samples are summed. If the result is outside the i16 PCM range, it gets clipped.
    SumClip,
    /// Firstly, input samples are summed. If the result is outside the i16 PCM range,
    /// nearby summed samples are scaled down by factor, such that the summed wave is in the i16 PCM range.
    SumScale,
}
