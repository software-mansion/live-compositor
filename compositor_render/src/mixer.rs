use std::sync::{Arc, Mutex};

use crate::{
    error::UpdateSceneError, scene::AudioComposition, AudioChannels, AudioSamplesSet, OutputId,
    OutputSamples,
};

use self::audio_mixer::InternalAudioMixer;

mod audio_mixer;

#[derive(Debug, Clone)]
pub struct AudioMixer(Arc<Mutex<InternalAudioMixer>>);

impl AudioMixer {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(InternalAudioMixer::new())))
    }

    pub fn mix_samples(&self, samples_set: AudioSamplesSet) -> OutputSamples {
        self.0.lock().unwrap().mix_samples(samples_set)
    }

    pub fn update_scene(
        &self,
        output_id: OutputId,
        audio: AudioComposition,
    ) -> Result<(), UpdateSceneError> {
        self.0.lock().unwrap().update_scene(output_id, audio)
    }

    pub fn register_output(
        &mut self,
        output_id: OutputId,
        sample_rate: u32,
        channels: AudioChannels,
        initial_composition: AudioComposition,
    ) {
        self.0.lock().unwrap().register_output(
            output_id,
            sample_rate,
            channels,
            initial_composition,
        );
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}
