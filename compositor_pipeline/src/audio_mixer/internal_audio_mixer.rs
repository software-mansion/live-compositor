use std::collections::HashMap;

use compositor_render::{error::UpdateSceneError, OutputId};

use super::types::{AudioChannels, AudioMixingParams, InputSamplesSet, OutputSamplesSet};

#[derive(Debug)]
struct OutputInfo {
    audio: AudioMixingParams,
    channels: AudioChannels,
}

#[derive(Debug)]
pub(super) struct InternalAudioMixer {
    outputs: HashMap<OutputId, OutputInfo>,
    output_sample_rate: u32,
}

impl InternalAudioMixer {
    pub fn new(output_sample_rate: u32) -> Self {
        Self {
            outputs: HashMap::new(),
            output_sample_rate,
        }
    }

    pub fn mix_samples(&mut self, samples_set: InputSamplesSet) -> OutputSamplesSet {
        OutputSamplesSet(HashMap::new())
    }

    pub fn register_output(
        &mut self,
        output_id: OutputId,
        audio: AudioMixingParams,
        channels: AudioChannels,
    ) {
        self.outputs
            .insert(output_id, OutputInfo { audio, channels });
    }

    pub fn unregister_output(&mut self, output_id: &OutputId) {
        self.outputs.remove(output_id);
    }

    pub fn update_output(
        &mut self,
        output_id: &OutputId,
        audio: AudioMixingParams,
    ) -> Result<(), UpdateSceneError> {
        match self.outputs.get_mut(output_id) {
            Some(output_info) => {
                output_info.audio = audio;
                Ok(())
            }
            None => Err(UpdateSceneError::OutputNotRegistered(output_id.clone())),
        }
    }
}
