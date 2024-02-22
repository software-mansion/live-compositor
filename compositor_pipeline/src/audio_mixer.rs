use std::sync::{Arc, Mutex};

use compositor_render::{error::UpdateSceneError, OutputId};

use self::{
    internal_audio_mixer::InternalAudioMixer,
    types::{Audio, AudioChannels, AudioSamplesSet, OutputSamples},
};

mod internal_audio_mixer;
pub mod types;

#[derive(Debug, Clone)]
pub(super) struct AudioMixer(Arc<Mutex<InternalAudioMixer>>);

impl AudioMixer {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(InternalAudioMixer::new())))
    }

    pub fn mix_samples(&self, samples_set: AudioSamplesSet) -> OutputSamples {
        self.0.lock().unwrap().mix_samples(samples_set)
    }

    pub fn update_output(&self, output_id: OutputId, audio: Audio) -> Result<(), UpdateSceneError> {
        self.0.lock().unwrap().update_output(output_id, audio)
    }

    pub fn register_output(
        &self,
        output_id: OutputId,
        sample_rate: u32,
        channels: AudioChannels,
        initial_audio: Audio,
    ) {
        self.0
            .lock()
            .unwrap()
            .register_output(output_id, sample_rate, channels, initial_audio);
    }

    pub fn unregister_output(&self, output_id: &OutputId) {
        self.0.lock().unwrap().unregister_output(output_id);
    }
}
