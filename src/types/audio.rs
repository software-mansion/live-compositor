use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::InputId;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Audio {
    pub inputs: Vec<InputAudio>,
    pub mixing_strategy: Option<MixingStrategy>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputAudio {
    pub input_id: InputId,
    // (**default=`1.0`**) float in [0, 1] range representing input volume
    pub volume: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum MixingStrategy {
    SumClip,
    SumScale,
}
