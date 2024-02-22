use super::{Audio, InputAudio};

impl From<Audio> for compositor_pipeline::audio_mixer::types::AudioMixingParams {
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

impl From<InputAudio> for compositor_pipeline::audio_mixer::types::InputParams {
    fn from(value: InputAudio) -> Self {
        Self {
            input_id: value.input_id.into(),
        }
    }
}
