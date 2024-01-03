use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::AudioInputId;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct TrackProperties {
    input_id: AudioInputId,
    // Temporal
    volume: Option<u32>,
    // ...
}
