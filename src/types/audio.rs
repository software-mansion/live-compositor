use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::InputId;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Audio {
    pub inputs: Vec<InputAudio>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputAudio {
    pub input_id: InputId,
    // (**default=`1.0`**) float in [0, 1] range representing input volume
    pub volume: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum MixingStrategy {
    /// Sums samples from inputs and scales down wave parts result near picks exceeding the i16 PCM range.
    SumClip,
    /// Sums samples from inputs and scales down wave parts result near picks exceeding the i16 PCM range.
    /// If the summed wave is in the i16 PCM range, input waves are summed without scaling and the result is the same as with `sum_clip`
    SumScale,
}
