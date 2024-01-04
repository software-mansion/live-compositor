use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::InputId;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct MixingProperties {
    input_id: InputId,
    // Temporal
    volume: Option<u32>,
    // ...
}
