use super::{Audio, InputParams};

impl From<Audio> for compositor_pipeline::audio_mixer::types::Audio {
    fn from(value: Audio) -> Self {
        let mixed_inputs = value
            .inputs
            .iter()
            .map(|input_params| input_params.clone().into())
            .collect();

        Self {
            inputs: mixed_inputs,
        }
    }
}

impl From<InputParams> for compositor_pipeline::audio_mixer::types::InputParams {
    fn from(value: InputParams) -> Self {
        Self {
            input_id: value.input_id.into(),
        }
    }
}
