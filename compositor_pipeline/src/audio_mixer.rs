use std::sync::{Arc, Mutex};

use compositor_render::{error::UpdateSceneError, InputId, OutputId};
use tracing::trace;

use self::internal_audio_mixer::InternalAudioMixer;

mod internal_audio_mixer;
mod types;

pub use types::*;

#[derive(Debug, Clone)]
pub(super) struct AudioMixer(Arc<Mutex<InternalAudioMixer>>);

impl AudioMixer {
    pub fn new(output_sample_rate: u32) -> Self {
        Self(Arc::new(Mutex::new(InternalAudioMixer::new(
            output_sample_rate,
        ))))
    }

    pub fn mix_samples(&self, samples_set: InputSamplesSet) -> OutputSamplesSet {
        trace!(set=?samples_set, "Mixing samples");
        self.0.lock().unwrap().mix_samples(samples_set)
    }

    pub fn register_input(&self, input_id: InputId) {
        self.0.lock().unwrap().register_input(input_id)
    }

    pub fn unregister_input(&self, input_id: &InputId) {
        self.0.lock().unwrap().unregister_input(input_id)
    }

    pub fn register_output(
        &self,
        output_id: OutputId,
        audio: AudioMixingParams,
        mixing_strategy: MixingStrategy,
        channels: AudioChannels,
    ) {
        self.0
            .lock()
            .unwrap()
            .register_output(output_id, audio, mixing_strategy, channels)
    }

    pub fn unregister_output(&self, output_id: &OutputId) {
        self.0.lock().unwrap().unregister_output(output_id)
    }

    pub fn update_output(
        &self,
        output_id: &OutputId,
        audio: AudioMixingParams,
    ) -> Result<(), UpdateSceneError> {
        self.0.lock().unwrap().update_output(output_id, audio)
    }
}
